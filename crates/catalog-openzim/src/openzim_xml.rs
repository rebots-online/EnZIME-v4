//! `OpenZimXmlCatalogSource` — `CatalogSource` implementation that reads the
//! OpenZIM library XML format (the `<library version="..."><book .../>...`
//! schema served by library.kiwix.org and mirrored elsewhere) and normalizes
//! every `<book>` into the canonical `CatalogEntry` declared by
//! `ports::catalog`.
//!
//! Format reference: https://wiki.openzim.org/wiki/Library_Catalog. Each
//! `<book>` element carries flat attributes (`id`, `title`, `language`,
//! `size`, `sha256sum`, `url`, ...). Missing or malformed attributes are not
//! silently coerced — the refresh fails with `CatalogError::Malformed`.
//!
//! The raw XML bytes are the canonical payload for catalog-signature
//! verification. Catalogs without a sidecar `SignatureEnvelope` are rejected
//! with `CatalogError::InvalidSignature` when `require_signature` is set
//! (the default), per the §8.2 trust model — "unsigned and untrusted inputs
//! are never silently coerced".

use crate::store::{
    diff_refresh, entry_matches, epoch_millis, find_entry, paginate,
    SharedSnapshot, Snapshot,
};
use crate::validate::{
    validate_artifact_sha256, validate_artifact_size_bytes, validate_artifact_url,
    validate_catalog_entry_id, validate_language_bcp47, validate_non_empty,
};
use async_trait::async_trait;
use bytes::Bytes;
use domain::trust::{SignatureEnvelope, SignatureVerificationState};
use futures_util::StreamExt;
use ports::catalog::{
    CatalogEntry, CatalogError, CatalogPage, CatalogQuery, CatalogRefreshReport,
    CatalogSource, TrustPresentation,
};
use ports::signature::{SignatureError, SignatureProvider};
use ports::transfer::{TransferByteStream, TransferError, TransferSource};
use serde::Deserialize;
use std::sync::Arc;

/// Default ceiling on a single catalog download. The catalog itself is
/// metadata-only; 64 MiB is far above any realistic OpenZIM library file
/// while still bounding untrusted input.
const DEFAULT_MAX_CATALOG_BYTES: u64 = 64 * 1024 * 1024;

/// Default signing domain passed to `SignatureProvider::verify`. Operators can
/// override per-source.
const DEFAULT_SIGN_DOMAIN: &str = "catalog.openzim.xml.v1";

/// The §8.2 release-implementation id for this adapter.
pub const SOURCE_ID: &str = "catalog.openzim.xml.v1";

/// `OpenZimXmlCatalogSource` fetches an OpenZIM library XML catalog from a
/// configured `source_url` via an injected `TransferSource`, verifies its
/// signature when a `SignatureProvider` is wired in, and serves normalized
/// `CatalogEntry` rows from an in-memory snapshot.
pub struct OpenZimXmlCatalogSource {
    source_url: String,
    /// Where the catalog's `SignatureEnvelope` JSON lives. Defaults to
    /// `<source_url>.sig` when `None`.
    signature_url_override: Option<String>,
    fetcher: Arc<dyn TransferSource>,
    signature: Option<Arc<dyn SignatureProvider>>,
    sign_domain: String,
    require_signature: bool,
    maximum_catalog_bytes: u64,
    state: SharedSnapshot,
}

impl OpenZimXmlCatalogSource {
    /// Build a source that requires a verified signature. Callers inject a
    /// `TransferSource` (typically `HttpTransferSource` from `transfer-http`)
    /// and optionally a `SignatureProvider`.
    pub fn new(
        source_url: impl Into<String>,
        fetcher: Arc<dyn TransferSource>,
        signature: Option<Arc<dyn SignatureProvider>>,
    ) -> Self {
        Self {
            source_url: source_url.into(),
            signature_url_override: None,
            fetcher,
            signature,
            sign_domain: DEFAULT_SIGN_DOMAIN.to_string(),
            require_signature: true,
            maximum_catalog_bytes: DEFAULT_MAX_CATALOG_BYTES,
            state: std::sync::RwLock::new(Snapshot::default()),
        }
    }

    /// Override the sidecar URL used to fetch the catalog signature. When
    /// unset, `<source_url>.sig` is used.
    pub fn with_signature_url(mut self, url: impl Into<String>) -> Self {
        self.signature_url_override = Some(url.into());
        self
    }

    /// Override the signing domain passed to `SignatureProvider::verify`.
    pub fn with_sign_domain(mut self, domain: impl Into<String>) -> Self {
        self.sign_domain = domain.into();
        self
    }

    /// Toggle whether missing/invalid catalog signatures are fatal. The
    /// default is `true` (reject unsigned catalogs). Operators of trusted
    /// local mirrors may opt out.
    pub fn require_signature(mut self, required: bool) -> Self {
        self.require_signature = required;
        self
    }

    /// Override the per-refresh byte ceiling for the catalog payload.
    pub fn with_maximum_catalog_bytes(mut self, max: u64) -> Self {
        self.maximum_catalog_bytes = max;
        self
    }

    fn signature_url(&self) -> String {
        self.signature_url_override
            .clone()
            .unwrap_or_else(|| format!("{}.sig", self.source_url))
    }
}

#[async_trait]
impl CatalogSource for OpenZimXmlCatalogSource {
    fn source_id(&self) -> &str {
        SOURCE_ID
    }

    async fn refresh(&self) -> Result<CatalogRefreshReport, CatalogError> {
        let raw =
            fetch_bytes(self.fetcher.as_ref(), &self.source_url, self.maximum_catalog_bytes)
                .await?;

        // Verify signature BEFORE parsing untrusted input. A failed or missing
        // signature never reaches the XML parser.
        let trust = verify_catalog_signature(
            self.signature.as_deref(),
            &self.sign_domain,
            &raw,
            self.require_signature,
            self.fetcher.as_ref(),
            &self.signature_url(),
            self.maximum_catalog_bytes,
        )
        .await?;

        let entries = parse_openzim_xml(&raw)?;
        let refreshed_at_ms = epoch_millis();

        let (added, updated, removed) = {
            let state = self
                .state
                .read()
                .expect("catalog snapshot lock poisoned");
            diff_refresh(&state.entries, &entries)
        };

        let report = CatalogRefreshReport {
            source_id: SOURCE_ID.to_string(),
            added,
            updated,
            removed,
            signature_state: trust.state.clone(),
            refreshed_at_ms,
        };

        {
            let mut state = self
                .state
                .write()
                .expect("catalog snapshot lock poisoned");
            state.entries = entries;
            state.last_refreshed_at_ms = refreshed_at_ms;
            state.last_refresh = Some(report.clone());
        }

        Ok(report)
    }

    async fn search(
        &self,
        query: CatalogQuery,
    ) -> Result<CatalogPage, CatalogError> {
        let (filtered, refreshed_at_ms) = {
            let state = self
                .state
                .read()
                .expect("catalog snapshot lock poisoned");
            let filtered: Vec<CatalogEntry> = state
                .entries
                .iter()
                .filter(|e| entry_matches(e, &query))
                .cloned()
                .collect();
            (filtered, state.last_refreshed_at_ms)
        };
        Ok(paginate(&filtered, &query, refreshed_at_ms))
    }

    async fn entry(
        &self,
        catalog_entry_id: &str,
    ) -> Result<CatalogEntry, CatalogError> {
        let state = self
            .state
            .read()
            .expect("catalog snapshot lock poisoned");
        find_entry(&state.entries, catalog_entry_id).cloned()
    }
}

// ---------------------------------------------------------------------------
// Signature verification shared between the two adapters (openzim_xml uses the
// raw catalog bytes; enzime_json uses canonicalized entry bytes).
// ---------------------------------------------------------------------------

pub(crate) async fn fetch_bytes(
    fetcher: &dyn TransferSource,
    url: &str,
    maximum_bytes: u64,
) -> Result<Vec<u8>, CatalogError> {
    let stream = fetcher
        .open_full(url, maximum_bytes)
        .await
        .map_err(transfer_to_catalog)?;
    collect_stream(stream, maximum_bytes).await
}

async fn collect_stream(
    mut stream: TransferByteStream,
    maximum_bytes: u64,
) -> Result<Vec<u8>, CatalogError> {
    let mut buf: Vec<u8> = Vec::new();
    while let Some(chunk) = stream.next().await {
        let chunk: Bytes =
            chunk.map_err(|e| CatalogError::Transport(format!("{e}")))?;
        let new_len = buf
            .len()
            .saturating_add(chunk.len());
        if new_len as u64 > maximum_bytes {
            return Err(CatalogError::Malformed(format!(
                "catalog payload exceeds {maximum_bytes} bytes"
            )));
        }
        buf.extend_from_slice(&chunk);
    }
    Ok(buf)
}

fn transfer_to_catalog(err: TransferError) -> CatalogError {
    match err {
        TransferError::Offline => CatalogError::Offline,
        TransferError::NotFound => CatalogError::Transport(
            "catalog source returned not-found".to_string(),
        ),
        TransferError::PermissionDenied => CatalogError::AuthenticationFailed,
        TransferError::ResponseLimitExceeded => {
            CatalogError::Malformed("catalog exceeds transfer limit".to_string())
        }
        other => CatalogError::Transport(format!("{other}")),
    }
}

/// Shared signature verification. Fetches the sidecar envelope, asks the
/// injected `SignatureProvider` to verify the canonical payload, and decides
/// whether the result is acceptable for this source.
pub(crate) async fn verify_catalog_signature(
    signature: Option<&dyn SignatureProvider>,
    sign_domain: &str,
    canonical_payload: &[u8],
    require_signature: bool,
    fetcher: &dyn TransferSource,
    signature_url: &str,
    maximum_bytes: u64,
) -> Result<TrustPresentation, CatalogError> {
    let Some(signature) = signature else {
        // No signature provider wired in.
        if require_signature {
            return Err(CatalogError::InvalidSignature);
        }
        return Ok(unsigned_presentation(
            "no signature provider configured",
        ));
    };

    let sig_bytes = match fetch_bytes(fetcher, signature_url, maximum_bytes).await {
        Ok(b) => b,
        Err(CatalogError::Transport(_)) | Err(CatalogError::Offline) => {
            if require_signature {
                return Err(CatalogError::InvalidSignature);
            }
            return Ok(unsigned_presentation(
                "signature sidecar unreachable",
            ));
        }
        Err(CatalogError::Malformed(_)) => {
            // Sidecar exists but is too large — treat as malformed signature.
            return Err(CatalogError::InvalidSignature);
        }
        Err(other) => return Err(other),
    };

    // The envelope JSON itself is untrusted; a parse failure means the
    // sidecar is not a real signature envelope, so the catalog is treated as
    // unauthenticated.
    let envelope: SignatureEnvelope = serde_json::from_slice(&sig_bytes)
        .map_err(|_| CatalogError::InvalidSignature)?;

    let verification = signature
        .verify(sign_domain, canonical_payload, &envelope)
        .await
        .map_err(sig_to_catalog_invalid)?;

    match verification.state {
        SignatureVerificationState::Verified => Ok(TrustPresentation {
            state: "verified".to_string(),
            signer: verification.signer_label.clone(),
            explanation: verification.explanation.clone(),
        }),
        SignatureVerificationState::Unsigned => {
            if require_signature {
                Err(CatalogError::InvalidSignature)
            } else {
                Ok(TrustPresentation {
                    state: "unsigned".to_string(),
                    signer: verification.signer_label.clone(),
                    explanation: verification.explanation.clone(),
                })
            }
        }
        SignatureVerificationState::Invalid
        | SignatureVerificationState::UnknownKey
        | SignatureVerificationState::RevokedKey => {
            Err(CatalogError::InvalidSignature)
        }
    }
}

fn sig_to_catalog_invalid(err: SignatureError) -> CatalogError {
    let _ = err;
    CatalogError::InvalidSignature
}

fn unsigned_presentation(reason: &str) -> TrustPresentation {
    TrustPresentation {
        state: "unsigned".to_string(),
        signer: None,
        explanation: format!("Catalog is unsigned ({reason})."),
    }
}

// ---------------------------------------------------------------------------
// OpenZIM library XML parsing.
// ---------------------------------------------------------------------------

/// quick-xml + serde view of `<library version="..."><book .../>...</library>`.
#[derive(Deserialize)]
struct OpenZimLibrary {
    #[serde(rename = "book", default)]
    books: Vec<OpenZimBook>,
}

#[derive(Deserialize)]
struct OpenZimBook {
    #[serde(rename = "@id", default)]
    id: String,
    #[serde(rename = "@title", default)]
    title: String,
    #[serde(rename = "@description", default)]
    description: String,
    #[serde(rename = "@language", default)]
    language: String,
    #[serde(rename = "@publisher", default)]
    publisher: String,
    #[serde(rename = "@url", default)]
    url: String,
    #[serde(rename = "@size", default)]
    size: String,
    #[serde(rename = "@sha256sum", default)]
    sha256sum: String,
    #[serde(rename = "@name", default)]
    name: String,
    #[serde(rename = "@tags", default)]
    tags: String,
    #[serde(rename = "@version", default)]
    version: String,
    // OpenZIM's `flavour` distinguishes variants (maxi/mini/nopic). The §8.2
    // CatalogEntry has no dedicated field for it, so we capture it for
    // completeness and to keep the deserializer total over the documented
    // attribute set. Underscore-prefixed because the architecture's canonical
    // entry shape does not surface it.
    #[serde(rename = "@flavour", default)]
    _flavour: String,
}

/// Parse and normalize the entire library XML into canonical entries. The
/// first malformed `<book>` aborts the whole refresh — never silently dropped.
pub(crate) fn parse_openzim_xml(raw: &[u8]) -> Result<Vec<CatalogEntry>, CatalogError> {
    let raw_str = std::str::from_utf8(raw).map_err(|e| {
        CatalogError::Malformed(format!("catalog is not valid UTF-8: {e}"))
    })?;
    let library: OpenZimLibrary = quick_xml::de::from_str(raw_str).map_err(|e| {
        CatalogError::Malformed(format!("catalog XML parse failed: {e}"))
    })?;

    let mut out = Vec::with_capacity(library.books.len());
    for book in library.books {
        out.push(normalize_book(&book)?);
    }
    Ok(out)
}

fn normalize_book(book: &OpenZimBook) -> Result<CatalogEntry, CatalogError> {
    validate_catalog_entry_id(&book.id)?;
    validate_non_empty("title", &book.title)?;
    let corpus_id = if book.name.trim().is_empty() {
        book.id.clone()
    } else {
        book.name.clone()
    };
    validate_non_empty("corpus_id", &corpus_id)?;
    let upstream_version = if book.version.trim().is_empty() {
        "unknown".to_string()
    } else {
        book.version.clone()
    };

    let language = normalize_openzim_language(&book.language)?;
    validate_language_bcp47(&language)?;

    let category = extract_openzim_category(&book.tags);

    let publisher = if book.publisher.trim().is_empty() {
        "unknown".to_string()
    } else {
        book.publisher.clone()
    };
    validate_non_empty("publisher", &publisher)?;

    let description = book.description.clone();

    validate_artifact_url(&book.url)?;
    let size: u64 = book
        .size
        .parse::<u64>()
        .map_err(|e| {
            CatalogError::Malformed(format!(
                "artifact_size_bytes is not a valid u64: {e}"
            ))
        })?;
    validate_artifact_size_bytes(size)?;
    validate_artifact_sha256(&book.sha256sum)?;

    Ok(CatalogEntry {
        catalog_entry_id: book.id.clone(),
        corpus_id,
        upstream_version,
        title: book.title.clone(),
        description,
        language_bcp47: language,
        category,
        publisher,
        artifact_url: book.url.clone(),
        artifact_size_bytes: size,
        artifact_sha256: book.sha256sum.clone(),
        // OpenZIM library XML does not declare per-artifact signature URLs.
        signature_url: None,
        trust: unsigned_artifact_presentation(),
    })
}

/// OpenZIM catalogs may declare `language="eng,fra"` (ISO 639-3, possibly
/// comma-separated). We pick the primary tag (first), lower-case it, and
/// pass it through. Three-letter codes are valid BCP47 subtags so no
/// mapping is required; downstream readers can canonicalize further if they
/// wish.
fn normalize_openzim_language(raw: &str) -> Result<String, CatalogError> {
    let primary = raw
        .split(',')
        .map(str::trim)
        .find(|s| !s.is_empty())
        .ok_or_else(|| {
            CatalogError::Malformed("language is empty".to_string())
        })?;
    Ok(primary.to_lowercase())
}

/// OpenZIM encodes category as a `_category:<name>` token inside the
/// semicolon-separated `tags` attribute. Fall back to "general" when absent
/// so the (non-optional) `category` field always carries a meaningful value.
fn extract_openzim_category(tags: &str) -> String {
    for token in tags.split(';') {
        let token = token.trim();
        if let Some(rest) = token.strip_prefix("_category:") {
            let rest = rest.trim();
            if !rest.is_empty() {
                return rest.to_string();
            }
        }
    }
    "general".to_string()
}

/// Per-entry trust for OpenZIM-sourced entries. The catalog-level signature
/// (when verified) covers every entry; the per-entry presentation reflects
/// that via a generic "catalog_signed" marker. Catalog-level verification
/// happens during refresh and is reflected in the refresh report's
/// `signature_state`.
fn unsigned_artifact_presentation() -> TrustPresentation {
    TrustPresentation {
        state: "catalog_signed".to_string(),
        signer: None,
        explanation: "Per-entry trust is delegated to the catalog-level \
                      signature verified during refresh."
            .to_string(),
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) const SAMPLE_XML: &str = r#"
<library version="20110515">
  <book
    id="wikipedia_en_physics_maxi_2024-10"
    title="Wikipedia Physics"
    description="Physics articles from Wikipedia"
    language="eng"
    publisher="Wikipedia"
    url="https://download.kiwix.org/zim/wikipedia/wikipedia_en_physics_maxi_2024-10.zim"
    size="1234567890"
    sha256sum="0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
    name="wikipedia_en_physics_maxi"
    tags="wikipedia;_category:wikipedia;_pictures:yes"
    version="2024-10"
    flavour="maxi"
  />
  <book
    id="stackexchange_en_math_2024-09"
    title="Mathematics Stack Exchange"
    description="Q&amp;A for people studying math at any level"
    language="eng,fra"
    publisher="Stack Exchange"
    url="https://download.kiwix.org/zim/stackexchange/stackexchange_en_math_2024-09.zim"
    size="987654321"
    sha256sum="fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210"
    name="stackexchange_en_math"
    tags="stackexchange;_category:stackexchange"
    version="2024-09"
  />
</library>
"#;

    #[test]
    fn parses_valid_openzim_xml() {
        let entries = parse_openzim_xml(SAMPLE_XML.as_bytes()).expect("xml");
        assert_eq!(entries.len(), 2);

        let physics = &entries[0];
        assert_eq!(physics.catalog_entry_id, "wikipedia_en_physics_maxi_2024-10");
        assert_eq!(physics.corpus_id, "wikipedia_en_physics_maxi");
        assert_eq!(physics.upstream_version, "2024-10");
        assert_eq!(physics.title, "Wikipedia Physics");
        assert_eq!(physics.language_bcp47, "eng");
        assert_eq!(physics.category, "wikipedia");
        assert_eq!(physics.publisher, "Wikipedia");
        assert_eq!(physics.artifact_size_bytes, 1_234_567_890);
        assert_eq!(
            physics.artifact_sha256,
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        );
        assert_eq!(physics.signature_url, None);

        let math = &entries[1];
        assert_eq!(math.corpus_id, "stackexchange_en_math");
        assert_eq!(math.language_bcp47, "eng"); // primary tag from "eng,fra"
        assert_eq!(math.category, "stackexchange");
        assert_eq!(math.upstream_version, "2024-09");
        assert_eq!(math.description, "Q&A for people studying math at any level");
    }

    #[test]
    fn rejects_malformed_xml() {
        let bad = "not xml at all";
        let err = parse_openzim_xml(bad.as_bytes()).expect_err("garbage");
        assert!(matches!(err, CatalogError::Malformed(_)));
    }

    #[test]
    fn rejects_book_with_bad_sha256() {
        let xml = r#"<library><book id="x" title="t" language="eng" url="https://x/z.zim" size="10" sha256sum="tooshort"/></library>"#;
        let err = parse_openzim_xml(xml.as_bytes()).expect_err("short sha");
        assert!(matches!(err, CatalogError::Malformed(_)));
    }

    #[test]
    fn rejects_book_with_bad_url_scheme() {
        let xml = r#"<library><book id="x" title="t" language="eng" url="ftp://x/z.zim" size="10" sha256sum="0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"/></library>"#;
        let err = parse_openzim_xml(xml.as_bytes()).expect_err("bad scheme");
        assert!(matches!(err, CatalogError::Malformed(_)));
    }

    #[test]
    fn rejects_book_with_zero_size() {
        let xml = r#"<library><book id="x" title="t" language="eng" url="https://x/z.zim" size="0" sha256sum="0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"/></library>"#;
        let err = parse_openzim_xml(xml.as_bytes()).expect_err("zero size");
        assert!(matches!(err, CatalogError::Malformed(_)));
    }

    #[test]
    fn rejects_book_with_location_bearing_id() {
        let xml = r#"<library><book id="../escape" title="t" language="eng" url="https://x/z.zim" size="10" sha256sum="0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"/></library>"#;
        let err = parse_openzim_xml(xml.as_bytes()).expect_err("bad id");
        assert!(matches!(err, CatalogError::Malformed(_)));
    }

    #[test]
    fn defaults_unknown_category_to_general() {
        let xml = r#"<library><book id="x" title="t" language="eng" url="https://x/z.zim" size="10" sha256sum="0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef" tags="foo;bar"/></library>"#;
        let entries = parse_openzim_xml(xml.as_bytes()).expect("ok");
        assert_eq!(entries[0].category, "general");
    }

    #[test]
    fn diff_refresh_counts_added_updated_removed() {
        let mk = |id: &str, sha: &str| CatalogEntry {
            catalog_entry_id: id.to_string(),
            corpus_id: id.to_string(),
            upstream_version: "1".to_string(),
            title: id.to_string(),
            description: String::new(),
            language_bcp47: "en".to_string(),
            category: "general".to_string(),
            publisher: "p".to_string(),
            artifact_url: "https://x/y.zim".to_string(),
            artifact_size_bytes: 1,
            artifact_sha256: sha.to_string(),
            signature_url: None,
            trust: TrustPresentation {
                state: "verified".to_string(),
                signer: None,
                explanation: String::new(),
            },
        };
        let sha_a = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let sha_b = "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210";
        let old = vec![mk("a", sha_a), mk("b", sha_a)];
        let new = vec![mk("a", sha_b), mk("c", sha_a)];
        let (added, updated, removed) = diff_refresh(&old, &new);
        assert_eq!((added, updated, removed), (1, 1, 1));
    }
}
