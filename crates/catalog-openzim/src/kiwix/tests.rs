use super::*;
use async_trait::async_trait;
use bytes::Bytes;
use ports::transfer::{
    TransferByteStream, TransferError, TransferProbe, TransferProbeRequest, TransferRange,
};
use std::sync::Mutex;
const FEED: &str = r#"<a:feed xmlns:a="http://www.w3.org/2005/Atom" xmlns:s="http://a9.com/-/spec/opensearch/1.1/" xmlns:d="http://purl.org/dc/terms/"><s:totalResults>65</s:totalResults><s:startIndex>20</s:startIndex><a:entry><a:id>urn:uuid:abc</a:id><a:title>Science &amp; Math</a:title><a:summary><![CDATA[A < B]]></a:summary><d:language>eng</d:language><a:name>wikipedia_en_100</a:name><a:publisher><a:name>openZIM</a:name></a:publisher><a:link rel="http://opds-spec.org/acquisition/open-access" type="application/x-zim" href="https://download.example/x.zim.meta4" length="333430784" /></a:entry></a:feed>"#;
fn metalink(inner: &str) -> String {
    format!(
        r#"<metalink xmlns="urn:ietf:params:xml:ns:metalink"><file name="x.zim"><size>333429922</size>{inner}</file></metalink>"#
    )
}
#[test]
fn namespaced_feed_preserves_identity_and_pagination() {
    let page = parse_kiwix_opds(FEED.as_bytes(), "https://library.kiwix.org/").unwrap();
    assert_eq!(page.total, Some(65));
    assert_eq!(page.offset, 20);
    assert_eq!(page.next_offset, Some(21));
    assert_eq!(page.entries[0].title, "Science & Math");
    assert_eq!(page.entries[0].description, "A < B");
    assert_eq!(page.entries[0].publisher, "openZIM");
    assert_eq!(page.entries[0].advertised_size_bytes, 333430784);
    assert_eq!(page.entries[0].language, "eng");
    assert!(
        parse_kiwix_opds(FEED.replace(ATOM, "urn:spoof").as_bytes(), "https://x/").is_err()
    );
}
#[test]
fn metalink_exact_size_hash_and_https_priorities() {
    let text = metalink(&format!(
        r#"<url priority="2">https://b.example/x.zim</url><url priority="1">http://plain.example/x.zim</url><url priority="1">https://a.example/x.zim</url><url priority="9">https://a.example/x.zim</url><hash type="sha-256">{}</hash>"#,
        "AB".repeat(32)
    ));
    let result = parse_kiwix_metalink(text.as_bytes(), "https://x/x.meta4").unwrap();
    assert_eq!(result.size_bytes, 333429922);
    assert_eq!(
        result.mirrors,
        vec!["https://a.example/x.zim", "https://b.example/x.zim"]
    );
    assert_eq!(result.sha256(), Some("ab".repeat(32).as_str()));
}
#[test]
fn absent_hash_is_truthfully_absent_and_bad_hash_rejected() {
    let xml = metalink("<url>https://a.example/x.zim</url>");
    assert!(parse_kiwix_metalink(xml.as_bytes(), "https://x/")
        .unwrap()
        .sha256()
        .is_none());
    let xml = metalink("<hash type=\"sha-256\">123</hash><url>https://a.example/x.zim</url>");
    assert!(parse_kiwix_metalink(xml.as_bytes(), "https://x/").is_err());
    assert!(parse_kiwix_metalink(
        metalink("<url>http://a.example/x.zim</url>").as_bytes(),
        "https://x/"
    )
    .is_err());
}
#[test]
fn hostile_documents_and_unsafe_urls_are_rejected() {
    for xml in [
        "<!DOCTYPE x><x/>",
        "<x/><y/>",
        "<x>",
        "<bad:x/>",
        "<?xml version=\"1.0\" encoding=\"UTF-16\"?><x/>",
    ] {
        assert!(parse_xml(xml.as_bytes(), 1024).is_err(), "{xml}");
    }
    assert!(parse_xml(
        format!("{}{}", "<x>".repeat(33), "</x>".repeat(33)).as_bytes(),
        1024
    )
    .is_err());
    assert!(parse_xml(b"<x/>", 3).is_err());
    for value in [
        "http://x/a.zim",
        "https://user:secret@x/a.zim",
        "https://x/a.zim#frag",
        "javascript:evil()",
    ] {
        assert!(https_url("https://x/", value).is_err());
    }
}
#[derive(Default)]
struct Mock {
    calls: Mutex<Vec<(String, u64)>>,
    oversized: bool,
}
#[async_trait]
impl TransferSource for Mock {
    fn source_kind(&self) -> &'static str {
        "fixture"
    }
    async fn probe(&self, _: TransferProbeRequest) -> Result<TransferProbe, TransferError> {
        panic!("no ZIM probe during metadata fetch")
    }
    async fn open_range(
        &self,
        _: &str,
        _: TransferRange,
        _: Option<&str>,
    ) -> Result<TransferByteStream, TransferError> {
        panic!("no ZIM range during metadata fetch")
    }
    async fn open_full(
        &self,
        uri: &str,
        limit: u64,
    ) -> Result<TransferByteStream, TransferError> {
        self.calls.lock().unwrap().push((uri.to_owned(), limit));
        let bytes = if self.oversized {
            vec![b' '; OPDS_LIMIT as usize + 1]
        } else if uri.ends_with(".meta4") {
            metalink("<url>https://a.example/x.zim</url>").into_bytes()
        } else {
            FEED.as_bytes().to_vec()
        };
        Ok(Box::pin(futures_util::stream::iter(vec![Ok(Bytes::from(
            bytes,
        ))])))
    }
}
#[tokio::test]
async fn search_encodes_query_and_resolve_only_downloads_metadata() {
    let transfer = Arc::new(Mock::default());
    let client = KiwixCatalogClient::new(transfer.clone());
    let query = KiwixQuery {
        text: Some("a&b + π".into()),
        language: Some("eng".into()),
        offset: 20,
        limit: 1,
    };
    let page = client.search(&query).await.unwrap();
    let acquisition = client.resolve(&page.entries[0]).await.unwrap();
    assert_eq!(acquisition.size_bytes, 333429922);
    let calls = transfer.calls.lock().unwrap();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].1, OPDS_LIMIT);
    assert_eq!(calls[1].1, METALINK_LIMIT);
    let url = Url::parse(&calls[0].0).unwrap();
    assert!(url.query_pairs().any(|(k, v)| k == "q" && v == "a&b + π"));
    assert!(client
        .query_url(&KiwixQuery {
            limit: 0,
            ..Default::default()
        })
        .is_err());
}
#[tokio::test]
async fn adapter_enforces_ceiling_even_if_transport_does_not() {
    let client = KiwixCatalogClient::new(Arc::new(Mock {
        oversized: true,
        ..Default::default()
    }));
    assert!(client.search(&KiwixQuery::default()).await.is_err());
}
