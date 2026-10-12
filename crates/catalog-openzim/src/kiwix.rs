//! Native, bounded Kiwix OPDS discovery and Metalink resolution.
//! A published digest is metadata, not evidence of verified content or a signature.
use crate::openzim_xml::fetch_bytes;
use ports::{catalog::CatalogError, transfer::TransferSource};
use quick_xml::{events::Event, name::ResolveResult, NsReader};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Arc};
use url::Url;

const ATOM: &str = "http://www.w3.org/2005/Atom";
const DC: &str = "http://purl.org/dc/terms/";
const SEARCH: &str = "http://a9.com/-/spec/opensearch/1.1/";
const METALINK: &str = "urn:ietf:params:xml:ns:metalink";
const OPDS_LIMIT: u64 = 4 * 1024 * 1024;
const METALINK_LIMIT: u64 = 8 * 1024 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KiwixQuery {
    pub text: Option<String>,
    pub language: Option<String>,
    pub offset: u32,
    pub limit: u32,
}
impl Default for KiwixQuery {
    fn default() -> Self {
        Self {
            text: None,
            language: None,
            offset: 0,
            limit: 20,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KiwixEntry {
    pub id: String,
    pub title: String,
    pub description: String,
    pub language: String,
    pub name: String,
    pub updated: String,
    pub publisher: String,
    pub category: String,
    pub acquisition_url: String,
    pub advertised_size_bytes: u64,
    pub browse_url: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KiwixPage {
    pub entries: Vec<KiwixEntry>,
    pub total: Option<u64>,
    pub offset: u32,
    pub next_offset: Option<u32>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PublishedDigest {
    pub algorithm: String,
    pub hex: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KiwixAcquisition {
    pub file_name: String,
    pub size_bytes: u64,
    pub mirrors: Vec<String>,
    pub digests: Vec<PublishedDigest>,
}
impl KiwixAcquisition {
    pub fn sha256(&self) -> Option<&str> {
        self.digests
            .iter()
            .find(|d| d.algorithm == "sha-256")
            .map(|d| d.hex.as_str())
    }
}
pub struct KiwixCatalogClient {
    endpoint: Url,
    fetcher: Arc<dyn TransferSource>,
}
impl KiwixCatalogClient {
    pub fn new(fetcher: Arc<dyn TransferSource>) -> Self {
        Self {
            endpoint: Url::parse("https://library.kiwix.org/catalog/v2/entries")
                .expect("constant URL"),
            fetcher,
        }
    }
    pub fn with_endpoint(
        fetcher: Arc<dyn TransferSource>,
        endpoint: &str,
    ) -> Result<Self, CatalogError> {
        Ok(Self {
            endpoint: https_url(endpoint, endpoint)?,
            fetcher,
        })
    }
    fn query_url(&self, query: &KiwixQuery) -> Result<Url, CatalogError> {
        if query.limit == 0 || query.limit > 100 {
            return Err(bad("count must be 1..100"));
        }
        let mut url = self.endpoint.clone();
        url.set_query(None);
        {
            let mut pairs = url.query_pairs_mut();
            pairs
                .append_pair("count", &query.limit.to_string())
                .append_pair("start", &query.offset.to_string());
            if let Some(text) = query.text.as_deref().filter(|s| !s.trim().is_empty()) {
                pairs.append_pair("q", text.trim());
            }
            if let Some(lang) = query.language.as_deref().filter(|s| !s.trim().is_empty()) {
                pairs.append_pair("lang", lang.trim());
            }
        }
        Ok(url)
    }
    pub async fn search(&self, query: &KiwixQuery) -> Result<KiwixPage, CatalogError> {
        let url = self.query_url(query)?;
        let bytes = fetch_bytes(self.fetcher.as_ref(), url.as_str(), OPDS_LIMIT).await?;
        parse_kiwix_opds(&bytes, url.as_str())
    }
    pub async fn resolve(&self, entry: &KiwixEntry) -> Result<KiwixAcquisition, CatalogError> {
        let url = https_url(&entry.acquisition_url, &entry.acquisition_url)?;
        if url.path().ends_with(".meta4") {
            let bytes = fetch_bytes(self.fetcher.as_ref(), url.as_str(), METALINK_LIMIT).await?;
            parse_kiwix_metalink(&bytes, url.as_str())
        } else if url.path().ends_with(".zim") {
            if entry.advertised_size_bytes == 0 {
                return Err(bad("zero archive length"));
            }
            Ok(KiwixAcquisition {
                file_name: url
                    .path_segments()
                    .and_then(|s| s.last())
                    .unwrap_or_default()
                    .to_owned(),
                size_bytes: entry.advertised_size_bytes,
                mirrors: vec![url.into()],
                digests: vec![],
            })
        } else {
            Err(bad("unsupported acquisition document"))
        }
    }
}

fn bad(message: impl Into<String>) -> CatalogError {
    CatalogError::Malformed(message.into())
}
fn https_url(base: &str, reference: &str) -> Result<Url, CatalogError> {
    let base = Url::parse(base).map_err(|e| bad(e.to_string()))?;
    let url = base.join(reference).map_err(|e| bad(e.to_string()))?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
    {
        return Err(bad("expected HTTPS URL without credentials or fragment"));
    }
    Ok(url)
}
fn required(value: &str, field: &str) -> Result<String, CatalogError> {
    if value.trim().is_empty() {
        Err(bad(format!("missing {field}")))
    } else {
        Ok(value.trim().to_owned())
    }
}
fn number(value: &str, field: &str) -> Result<u64, CatalogError> {
    value
        .trim()
        .parse()
        .map_err(|_| bad(format!("invalid {field}")))
}
#[derive(Default, Debug)]
struct XmlNode {
    namespace: String,
    name: String,
    attrs: BTreeMap<String, String>,
    text: String,
    children: Vec<XmlNode>,
}
impl XmlNode {
    fn is(&self, ns: &str, name: &str) -> bool {
        self.namespace == ns && self.name == name
    }
    fn child(&self, ns: &str, name: &str) -> Option<&Self> {
        self.children.iter().find(|n| n.is(ns, name))
    }
    fn value(&self, ns: &str, name: &str) -> &str {
        self.child(ns, name)
            .map(|n| n.text.trim())
            .unwrap_or_default()
    }
    fn attr(&self, name: &str) -> &str {
        self.attrs.get(name).map(String::as_str).unwrap_or_default()
    }
}
fn parse_xml(bytes: &[u8], limit: u64) -> Result<XmlNode, CatalogError> {
    if bytes.len() as u64 > limit {
        return Err(bad("XML exceeds metadata limit"));
    }
    let mut reader = NsReader::from_reader(bytes);
    let mut stack: Vec<XmlNode> = Vec::new();
    let mut root = None;
    let mut count = 0;
    loop {
        let (ns, event) = reader
            .read_resolved_event()
            .map_err(|e| bad(e.to_string()))?;
        match event {
            Event::Start(ref e) | Event::Empty(ref e) => {
                count += 1;
                if count > 100_000 || stack.len() >= 32 {
                    return Err(bad("XML structure limit exceeded"));
                }
                let namespace = match ns {
                    ResolveResult::Bound(n) => std::str::from_utf8(n.as_ref())
                        .map_err(|e| bad(e.to_string()))?
                        .to_owned(),
                    ResolveResult::Unbound => String::new(),
                    ResolveResult::Unknown(_) => return Err(bad("unknown XML prefix")),
                };
                let mut node = XmlNode {
                    namespace,
                    name: std::str::from_utf8(e.local_name().as_ref())
                        .map_err(|e| bad(e.to_string()))?
                        .to_owned(),
                    ..Default::default()
                };
                for a in e.attributes() {
                    let a = a.map_err(|e| bad(e.to_string()))?;
                    // Only unqualified attributes are meaningful to these schemas.
                    let key =
                        std::str::from_utf8(a.key.as_ref()).map_err(|e| bad(e.to_string()))?;
                    if !key.contains(':') {
                        node.attrs.insert(
                            key.to_owned(),
                            a.unescape_value()
                                .map_err(|e| bad(e.to_string()))?
                                .into_owned(),
                        );
                    }
                }
                if matches!(event, Event::Start(_)) {
                    stack.push(node);
                } else if let Some(parent) = stack.last_mut() {
                    parent.children.push(node);
                } else if root.replace(node).is_some() {
                    return Err(bad("multiple XML roots"));
                }
            }
            Event::End(_) => {
                let node = stack.pop().ok_or_else(|| bad("unexpected XML close"))?;
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(node);
                } else if root.replace(node).is_some() {
                    return Err(bad("multiple XML roots"));
                }
            }
            Event::Text(e) => {
                let value = e.unescape().map_err(|e| bad(e.to_string()))?;
                if let Some(node) = stack.last_mut() {
                    node.text.push_str(&value);
                } else if !value.trim().is_empty() {
                    return Err(bad("text outside XML root"));
                }
            }
            Event::CData(e) => {
                let value = std::str::from_utf8(e.as_ref()).map_err(|e| bad(e.to_string()))?;
                stack
                    .last_mut()
                    .ok_or_else(|| bad("CDATA outside root"))?
                    .text
                    .push_str(value);
            }
            Event::DocType(_) => return Err(bad("DTD is forbidden")),
            Event::Decl(e) => {
                if let Some(encoding) = e.encoding() {
                    let encoding = encoding.map_err(|e| bad(e.to_string()))?;
                    if !encoding.eq_ignore_ascii_case(b"utf-8") {
                        return Err(bad("only UTF-8 XML supported"));
                    }
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    if !stack.is_empty() {
        return Err(bad("unclosed XML element"));
    }
    root.ok_or_else(|| bad("missing XML root"))
}

pub fn parse_kiwix_opds(bytes: &[u8], base: &str) -> Result<KiwixPage, CatalogError> {
    let root = parse_xml(bytes, OPDS_LIMIT)?;
    if !root.is(ATOM, "feed") {
        return Err(bad("expected Atom feed"));
    }
    let pagination = |name| {
        let v = root.value(SEARCH, name);
        if v.is_empty() {
            root.value(ATOM, name)
        } else {
            v
        }
    };
    let total = if pagination("totalResults").is_empty() {
        None
    } else {
        Some(number(pagination("totalResults"), "totalResults")?)
    };
    let offset = if pagination("startIndex").is_empty() {
        0
    } else {
        u32::try_from(number(pagination("startIndex"), "startIndex")?)
            .map_err(|_| bad("offset overflow"))?
    };
    let mut entries = Vec::new();
    for node in root.children.iter().filter(|n| n.is(ATOM, "entry")) {
        let link = node
            .children
            .iter()
            .find(|n| {
                n.is(ATOM, "link")
                    && (n.attr("rel") == "http://opds-spec.org/acquisition"
                        || n.attr("rel")
                            .starts_with("http://opds-spec.org/acquisition/"))
                    && n.attr("type") == "application/x-zim"
            })
            .ok_or_else(|| bad("entry missing ZIM acquisition link"))?;
        let advertised_size_bytes = number(link.attr("length"), "acquisition length")?;
        if advertised_size_bytes == 0 {
            return Err(bad("zero acquisition length"));
        }
        let lang = if node.value(DC, "language").is_empty() {
            node.value(ATOM, "language")
        } else {
            node.value(DC, "language")
        };
        let browse_url = node
            .children
            .iter()
            .find(|n| n.is(ATOM, "link") && n.attr("type") == "text/html")
            .map(|n| https_url(base, n.attr("href")).map(String::from))
            .transpose()?;
        let publisher = node
            .child(ATOM, "publisher")
            .map(|n| n.value(ATOM, "name"))
            .unwrap_or_default();
        entries.push(KiwixEntry {
            id: required(node.value(ATOM, "id"), "entry id")?,
            title: required(node.value(ATOM, "title"), "title")?,
            description: node.value(ATOM, "summary").to_owned(),
            language: required(lang, "language")?,
            name: node.value(ATOM, "name").to_owned(),
            updated: node.value(ATOM, "updated").to_owned(),
            publisher: publisher.to_owned(),
            category: node.value(ATOM, "category").to_owned(),
            acquisition_url: https_url(base, link.attr("href"))?.into(),
            advertised_size_bytes,
            browse_url,
        });
        if entries.len() > 100 {
            return Err(bad("OPDS page exceeds 100 entries"));
        }
    }
    let next = offset
        .checked_add(entries.len() as u32)
        .ok_or_else(|| bad("pagination overflow"))?;
    let next_offset = if !entries.is_empty() && total.map(|t| u64::from(next) < t).unwrap_or(false)
    {
        Some(next)
    } else {
        None
    };
    Ok(KiwixPage {
        entries,
        total,
        offset,
        next_offset,
    })
}

pub fn parse_kiwix_metalink(bytes: &[u8], base: &str) -> Result<KiwixAcquisition, CatalogError> {
    let root = parse_xml(bytes, METALINK_LIMIT)?;
    if !root.is(METALINK, "metalink") {
        return Err(bad("expected Metalink document"));
    }
    let files: Vec<_> = root
        .children
        .iter()
        .filter(|n| n.is(METALINK, "file"))
        .collect();
    if files.len() != 1 {
        return Err(bad("Metalink must describe exactly one file"));
    }
    let file = files[0];
    let file_name = required(file.attr("name"), "file name")?;
    if !file_name.ends_with(".zim") || file_name.contains('/') || file_name.contains('\\') {
        return Err(bad("invalid ZIM filename"));
    }
    let size_bytes = number(file.value(METALINK, "size"), "Metalink size")?;
    if size_bytes == 0 {
        return Err(bad("zero Metalink size"));
    }
    let mut digests: Vec<PublishedDigest> = Vec::new();
    for hash in file.children.iter().filter(|n| n.is(METALINK, "hash")) {
        let algorithm = hash.attr("type").to_ascii_lowercase();
        let width = match algorithm.as_str() {
            "sha-256" => 64,
            "sha-1" => 40,
            "md5" => 32,
            _ => continue,
        };
        let hex = hash.text.trim().to_ascii_lowercase();
        if hex.len() != width || !hex.bytes().all(|c| c.is_ascii_hexdigit()) {
            return Err(bad("invalid published digest"));
        }
        if let Some(old) = digests.iter().find(|d| d.algorithm == algorithm) {
            if old.hex != hex {
                return Err(bad("conflicting published digests"));
            }
        } else {
            digests.push(PublishedDigest { algorithm, hex });
        }
    }
    let mut ranked = Vec::new();
    for node in file.children.iter().filter(|n| n.is(METALINK, "url")) {
        let reference = node.text.trim();
        if reference.starts_with("http://") || reference.starts_with("ftp://") {
            continue;
        }
        let url = https_url(base, reference)?;
        if !url.path().ends_with(".zim") {
            return Err(bad("mirror is not a ZIM URL"));
        }
        let priority = if node.attr("priority").is_empty() {
            999999
        } else {
            number(node.attr("priority"), "mirror priority")?
        };
        ranked.push((priority, String::from(url)));
    }
    ranked.sort_by_key(|(priority, _)| *priority);
    let mut mirrors = Vec::new();
    for (_, url) in ranked {
        if !mirrors.contains(&url) {
            mirrors.push(url);
        }
    }
    if mirrors.is_empty() {
        return Err(bad("no HTTPS ZIM mirror"));
    }
    Ok(KiwixAcquisition {
        file_name,
        size_bytes,
        mirrors,
        digests,
    })
}

#[cfg(test)]
mod tests;
