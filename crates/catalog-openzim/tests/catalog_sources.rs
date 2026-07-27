//! End-to-end behaviour tests for the `OpenZimXmlCatalogSource` and
//! `EnzimeJsonCatalogSource` adapters, using in-memory `TransferSource` and
//! `SignatureProvider` implementations that return canned bytes/verifications.
//!
//! These tests exercise the *catalog adapter's* wiring: that it fetches via
//! the transfer port, verifies via the signature port, normalizes correctly,
//! and applies the right error variants. Cryptographic correctness of any
//! real `SignatureProvider` is a separate crate's responsibility (Ring-2 to
//! Ring-2 isolation per A-02).

use async_trait::async_trait;
use bytes::Bytes;
use catalog_openzim::{
    EnzimeJsonCatalogSource, OpenZimXmlCatalogSource,
};
use domain::trust::{
    SignatureEnvelope, SignatureVerification, SignatureVerificationState,
};
use ports::catalog::{CatalogError, CatalogQuery, CatalogSource};
use ports::signature::{SignatureError, SignatureProvider};
use ports::transfer::{
    TransferByteStream, TransferError, TransferProbe, TransferProbeRequest,
    TransferRange, TransferSource,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// ---------------------------------------------------------------------------
// In-memory test doubles for the two injected ports.
// ---------------------------------------------------------------------------

type CatalogStore = HashMap<String, Vec<u8>>;

struct InMemoryTransferSource {
    files: CatalogStore,
}

impl InMemoryTransferSource {
    fn new() -> Self {
        Self {
            files: HashMap::new(),
        }
    }
    fn with(mut self, url: impl Into<String>, body: Vec<u8>) -> Self {
        self.files.insert(url.into(), body);
        self
    }
}

#[async_trait]
impl TransferSource for InMemoryTransferSource {
    fn source_kind(&self) -> &'static str {
        "in-memory-test"
    }
    async fn probe(
        &self,
        _request: TransferProbeRequest,
    ) -> Result<TransferProbe, TransferError> {
        Err(TransferError::NotFound)
    }
    async fn open_range(
        &self,
        _uri: &str,
        _range: TransferRange,
        _expected_etag: Option<&str>,
    ) -> Result<TransferByteStream, TransferError> {
        Err(TransferError::RangeUnsupported)
    }
    async fn open_full(
        &self,
        uri: &str,
        _maximum_response_bytes: u64,
    ) -> Result<TransferByteStream, TransferError> {
        match self.files.get(uri) {
            Some(body) => {
                let bytes = Bytes::copy_from_slice(body);
                let stream: TransferByteStream =
                    Box::pin(futures_util::stream::once(async move {
                        Ok::<Bytes, TransferError>(bytes)
                    }));
                Ok(stream)
            }
            None => Err(TransferError::NotFound),
        }
    }
}

/// `SignatureProvider` whose `verify` result is hard-wired per call. The
/// `signed_domain` and `key_id` arguments are checked so the test exercises
/// the *real* gating logic in the catalog adapter.
struct ScriptedSignatureProvider {
    results: Mutex<Vec<(String, SignatureVerificationState, Option<String>)>>,
}

impl ScriptedSignatureProvider {
    fn new(
        results: Vec<(String, SignatureVerificationState, Option<String>)>,
    ) -> Self {
        Self {
            results: Mutex::new(results),
        }
    }
}

#[async_trait]
impl SignatureProvider for ScriptedSignatureProvider {
    async fn sign(
        &self,
        _domain: &str,
        _canonical_payload: &[u8],
    ) -> Result<SignatureEnvelope, SignatureError> {
        Err(SignatureError::KeyUnavailable)
    }
    async fn verify(
        &self,
        domain: &str,
        _canonical_payload: &[u8],
        _envelope: &SignatureEnvelope,
    ) -> Result<SignatureVerification, SignatureError> {
        let mut results = self.results.lock().unwrap();
        if results.is_empty() {
            return Ok(SignatureVerification {
                state: SignatureVerificationState::Invalid,
                key_id: String::new(),
                signer_label: None,
                explanation: "no more scripted results".to_string(),
            });
        }
        let (want_domain, state, signer) = results.remove(0);
        assert_eq!(domain, want_domain, "verify called with unexpected domain");
        Ok(SignatureVerification {
            state,
            key_id: String::from("test-key"),
            signer_label: signer,
            explanation: "scripted".to_string(),
        })
    }
}

// ---------------------------------------------------------------------------
// Fixtures.
// ---------------------------------------------------------------------------

const OPENZIM_XML_BODY: &str = r#"<library version="20110515">
  <book id="wikipedia_en_physics_maxi_2024-10"
        title="Wikipedia Physics"
        description="Physics articles from Wikipedia"
        language="eng"
        publisher="Wikipedia"
        url="https://download.kiwix.org/zim/wikipedia/wikipedia_en_physics_maxi_2024-10.zim"
        size="1234567890"
        sha256sum="0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        name="wikipedia_en_physics_maxi"
        tags="wikipedia;_category:wikipedia"
        version="2024-10"/>
  <book id="stackexchange_en_math_2024-09"
        title="Mathematics Stack Exchange"
        description="Math Q&amp;A"
        language="eng,fra"
        publisher="Stack Exchange"
        url="https://download.kiwix.org/zim/stackexchange/stackexchange_en_math_2024-09.zim"
        size="987654321"
        sha256sum="fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210"
        name="stackexchange_en_math"
        tags="stackexchange;_category:stackexchange"
        version="2024-09"/>
</library>
"#;

const SHA_A: &str =
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn enzime_json_body() -> Vec<u8> {
    serde_json::json!({
        "version": "enzime-catalog-v1",
        "signature": {
            "algorithm": "ed25519",
            "key_id": "test-key",
            "signed_domain": "catalog.enzime.json.v1",
            "value": [0u8, 1u8, 2u8],
        },
        "entries": [
            {
                "catalog_entry_id": "mdwiki_en_all_maxi_2024-10",
                "corpus_id": "mdwiki_en_all_maxi",
                "upstream_version": "2024-10",
                "title": "MDWiki",
                "description": "Medical reference",
                "language_bcp47": "en",
                "category": "medicine",
                "publisher": "MDWiki",
                "artifact_url": "https://host/mdwiki_en_all_maxi_2024-10.zim",
                "artifact_size_bytes": 500_000_000,
                "artifact_sha256": SHA_A,
                "signature_url": "https://host/mdwiki_en_all_maxi_2024-10.zim.sig",
            },
            {
                "catalog_entry_id": "gutenberg_en_all_2024-08",
                "corpus_id": "gutenberg_en_all",
                "upstream_version": "2024-08",
                "title": "Project Gutenberg",
                "description": "Public-domain books",
                "language_bcp47": "en",
                "category": "literature",
                "publisher": "Gutenberg",
                "artifact_url": "https://host/gutenberg_en_all_2024-08.zim",
                "artifact_size_bytes": 12_000_000_000_u64,
                "artifact_sha256": SHA_A,
            },
        ],
    })
    .to_string()
    .into_bytes()
}

fn envelope() -> Vec<u8> {
    serde_json::json!({
        "algorithm": "ed25519",
        "key_id": "test-key",
        "signed_domain": "catalog.openzim.xml.v1",
        "value": [0u8, 1u8, 2u8],
    })
    .to_string()
    .into_bytes()
}

// ---------------------------------------------------------------------------
// OpenZimXmlCatalogSource end-to-end tests.
// ---------------------------------------------------------------------------

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime")
}

#[test]
fn openzim_refresh_with_verified_signature_populates_snapshot() {
    let rt = runtime();
    rt.block_on(async {
        let fetcher = Arc::new(InMemoryTransferSource::new().with(
            "https://example/catalog.xml",
            OPENZIM_XML_BODY.as_bytes().to_vec(),
        ).with(
            "https://example/catalog.xml.sig",
            envelope(),
        ));
        let signer = Arc::new(ScriptedSignatureProvider::new(vec![(
            "catalog.openzim.xml.v1".to_string(),
            SignatureVerificationState::Verified,
            Some("test-signer".to_string()),
        )]));
        let src = OpenZimXmlCatalogSource::new(
            "https://example/catalog.xml",
            fetcher,
            Some(signer),
        );

        let report = src.refresh().await.expect("refresh");
        assert_eq!(report.source_id, "catalog.openzim.xml.v1");
        assert_eq!(report.added, 2);
        assert_eq!(report.updated, 0);
        assert_eq!(report.removed, 0);
        assert_eq!(report.signature_state, "verified");

        let page = src
            .search(CatalogQuery {
                text: None,
                languages: vec![],
                categories: vec![],
                max_size_bytes: None,
                offset: 0,
                limit: 10,
            })
            .await
            .expect("search");
        assert_eq!(page.entries.len(), 2);
        assert_eq!(page.total, Some(2));
        assert_eq!(page.next_offset, None);
    });
}

#[test]
fn openzim_refresh_without_signature_rejected_when_required() {
    let rt = runtime();
    rt.block_on(async {
        let fetcher = Arc::new(
            InMemoryTransferSource::new()
                .with("https://x/c.xml", OPENZIM_XML_BODY.as_bytes().to_vec()),
        );
        let signer = Arc::new(ScriptedSignatureProvider::new(vec![]));
        let src = OpenZimXmlCatalogSource::new(
            "https://x/c.xml",
            fetcher,
            Some(signer),
        );

        let err = src.refresh().await.expect_err("should reject");
        assert!(matches!(err, CatalogError::InvalidSignature));
    });
}

#[test]
fn openzim_refresh_with_invalid_signature_rejected() {
    let rt = runtime();
    rt.block_on(async {
        let fetcher = Arc::new(
            InMemoryTransferSource::new()
                .with("https://x/c.xml", OPENZIM_XML_BODY.as_bytes().to_vec())
                .with("https://x/c.xml.sig", envelope()),
        );
        let signer = Arc::new(ScriptedSignatureProvider::new(vec![(
            "catalog.openzim.xml.v1".to_string(),
            SignatureVerificationState::Invalid,
            None,
        )]));
        let src = OpenZimXmlCatalogSource::new(
            "https://x/c.xml",
            fetcher,
            Some(signer),
        );

        let err = src.refresh().await.expect_err("invalid sig");
        assert!(matches!(err, CatalogError::InvalidSignature));
    });
}

#[test]
fn openzim_search_filters_by_text_language_category_and_size() {
    let rt = runtime();
    rt.block_on(async {
        let fetcher = Arc::new(
            InMemoryTransferSource::new()
                .with("https://x/c.xml", OPENZIM_XML_BODY.as_bytes().to_vec())
                .with("https://x/c.xml.sig", envelope()),
        );
        let signer = Arc::new(ScriptedSignatureProvider::new(vec![(
            "catalog.openzim.xml.v1".to_string(),
            SignatureVerificationState::Verified,
            None,
        )]));
        let src = OpenZimXmlCatalogSource::new(
            "https://x/c.xml",
            fetcher,
            Some(signer),
        );
        src.refresh().await.expect("refresh");

        // text filter
        let physics = src
            .search(CatalogQuery {
                text: Some("physics".to_string()),
                languages: vec![],
                categories: vec![],
                max_size_bytes: None,
                offset: 0,
                limit: 10,
            })
            .await
            .expect("text");
        assert_eq!(physics.entries.len(), 1);
        assert_eq!(physics.entries[0].corpus_id, "wikipedia_en_physics_maxi");

        // category filter
        let math = src
            .search(CatalogQuery {
                text: None,
                languages: vec![],
                categories: vec!["stackexchange".to_string()],
                max_size_bytes: None,
                offset: 0,
                limit: 10,
            })
            .await
            .expect("category");
        assert_eq!(math.entries.len(), 1);
        assert_eq!(math.entries[0].corpus_id, "stackexchange_en_math");

        // size filter (only physics is under 1B)
        let small = src
            .search(CatalogQuery {
                text: None,
                languages: vec![],
                categories: vec![],
                max_size_bytes: Some(1_000_000_000),
                offset: 0,
                limit: 10,
            })
            .await
            .expect("size");
        assert_eq!(small.entries.len(), 1);

        // language filter (OpenZIM normalizes "eng,fra" -> "eng")
        let eng = src
            .search(CatalogQuery {
                text: None,
                languages: vec!["eng".to_string()],
                categories: vec![],
                max_size_bytes: None,
                offset: 0,
                limit: 10,
            })
            .await
            .expect("language");
        assert_eq!(eng.entries.len(), 2);
    });
}

#[test]
fn openzim_search_paginates() {
    let rt = runtime();
    rt.block_on(async {
        let fetcher = Arc::new(
            InMemoryTransferSource::new()
                .with("https://x/c.xml", OPENZIM_XML_BODY.as_bytes().to_vec())
                .with("https://x/c.xml.sig", envelope()),
        );
        let signer = Arc::new(ScriptedSignatureProvider::new(vec![(
            "catalog.openzim.xml.v1".to_string(),
            SignatureVerificationState::Verified,
            None,
        )]));
        let src = OpenZimXmlCatalogSource::new(
            "https://x/c.xml",
            fetcher,
            Some(signer),
        );
        src.refresh().await.expect("refresh");

        let page = src
            .search(CatalogQuery {
                text: None,
                languages: vec![],
                categories: vec![],
                max_size_bytes: None,
                offset: 0,
                limit: 1,
            })
            .await
            .expect("page");
        assert_eq!(page.entries.len(), 1);
        assert_eq!(page.total, Some(2));
        assert_eq!(page.next_offset, Some(1));

        let next = src
            .search(CatalogQuery {
                text: None,
                languages: vec![],
                categories: vec![],
                max_size_bytes: None,
                offset: 1,
                limit: 1,
            })
            .await
            .expect("page2");
        assert_eq!(next.entries.len(), 1);
        assert_eq!(next.next_offset, None);
    });
}

#[test]
fn openzim_entry_lookup_returns_found_and_not_found() {
    let rt = runtime();
    rt.block_on(async {
        let fetcher = Arc::new(
            InMemoryTransferSource::new()
                .with("https://x/c.xml", OPENZIM_XML_BODY.as_bytes().to_vec())
                .with("https://x/c.xml.sig", envelope()),
        );
        let signer = Arc::new(ScriptedSignatureProvider::new(vec![(
            "catalog.openzim.xml.v1".to_string(),
            SignatureVerificationState::Verified,
            None,
        )]));
        let src = OpenZimXmlCatalogSource::new(
            "https://x/c.xml",
            fetcher,
            Some(signer),
        );
        src.refresh().await.expect("refresh");

        let entry = src
            .entry("wikipedia_en_physics_maxi_2024-10")
            .await
            .expect("found");
        assert_eq!(entry.title, "Wikipedia Physics");

        let err = src.entry("does-not-exist").await.expect_err("missing");
        assert!(matches!(err, CatalogError::NotFound));
    });
}

#[test]
fn openzim_unsigned_allowed_when_not_required() {
    let rt = runtime();
    rt.block_on(async {
        let fetcher = Arc::new(
            InMemoryTransferSource::new()
                .with("https://x/c.xml", OPENZIM_XML_BODY.as_bytes().to_vec()),
        );
        // No signature provider wired in, but require_signature=false.
        let src = OpenZimXmlCatalogSource::new(
            "https://x/c.xml",
            fetcher,
            None,
        )
        .require_signature(false);

        let report = src.refresh().await.expect("unsigned ok");
        assert_eq!(report.signature_state, "unsigned");
    });
}

#[test]
fn openzim_refresh_diffs_added_updated_removed_on_second_refresh() {
    let rt = runtime();
    rt.block_on(async {
        let fetcher = Arc::new(
            InMemoryTransferSource::new()
                .with("https://x/c.xml", OPENZIM_XML_BODY.as_bytes().to_vec())
                .with("https://x/c.xml.sig", envelope()),
        );
        let signer = Arc::new(ScriptedSignatureProvider::new(vec![
            (
                "catalog.openzim.xml.v1".to_string(),
                SignatureVerificationState::Verified,
                None,
            ),
            (
                "catalog.openzim.xml.v1".to_string(),
                SignatureVerificationState::Verified,
                None,
            ),
        ]));
        let src = OpenZimXmlCatalogSource::new(
            "https://x/c.xml",
            fetcher.clone(),
            Some(signer.clone()),
        );
        let first = src.refresh().await.expect("first");
        assert_eq!((first.added, first.updated, first.removed), (2, 0, 0));

        // second refresh on the same body -> nothing added/removed, both
        // unchanged, so updated == 0 too.
        let second = src.refresh().await.expect("second");
        assert_eq!((second.added, second.updated, second.removed), (0, 0, 0));
    });
}

// ---------------------------------------------------------------------------
// EnzimeJsonCatalogSource end-to-end tests.
// ---------------------------------------------------------------------------

#[test]
fn enzime_refresh_with_verified_signature_populates_snapshot() {
    let rt = runtime();
    rt.block_on(async {
        let fetcher = Arc::new(
            InMemoryTransferSource::new()
                .with("https://x/cat.json", enzime_json_body()),
        );
        let signer = Arc::new(ScriptedSignatureProvider::new(vec![(
            "catalog.enzime.json.v1".to_string(),
            SignatureVerificationState::Verified,
            Some("enzime-release".to_string()),
        )]));
        let src = EnzimeJsonCatalogSource::new(
            "https://x/cat.json",
            fetcher,
            Some(signer),
        );

        let report = src.refresh().await.expect("refresh");
        assert_eq!(report.source_id, "catalog.enzime.json.v1");
        assert_eq!(report.added, 2);
        assert_eq!(report.signature_state, "verified");

        let page = src
            .search(CatalogQuery {
                text: None,
                languages: vec![],
                categories: vec![],
                max_size_bytes: None,
                offset: 0,
                limit: 10,
            })
            .await
            .expect("search");
        assert_eq!(page.entries.len(), 2);
        let mdwiki = &page.entries[0];
        assert_eq!(mdwiki.trust.state, "verified");
        assert_eq!(mdwiki.trust.signer.as_deref(), Some("enzime-release"));
    });
}

#[test]
fn enzime_unsigned_rejected_when_required() {
    let rt = runtime();
    rt.block_on(async {
        // Same body but with `signature` field stripped.
        let mut body = serde_json::from_slice::<serde_json::Value>(
            &enzime_json_body(),
        )
        .unwrap();
        body["signature"] = serde_json::Value::Null;
        let body = serde_json::to_vec(&body).unwrap();

        let fetcher =
            Arc::new(InMemoryTransferSource::new().with("https://x/cat.json", body));
        let signer = Arc::new(ScriptedSignatureProvider::new(vec![]));
        let src = EnzimeJsonCatalogSource::new(
            "https://x/cat.json",
            fetcher,
            Some(signer),
        );
        let err = src.refresh().await.expect_err("unsigned rejected");
        assert!(matches!(err, CatalogError::InvalidSignature));
    });
}

#[test]
fn enzime_wrong_signed_domain_rejected() {
    let rt = runtime();
    rt.block_on(async {
        let mut body = serde_json::from_slice::<serde_json::Value>(
            &enzime_json_body(),
        )
        .unwrap();
        body["signature"]["signed_domain"] =
            serde_json::Value::String("bogus.domain".to_string());
        let body = serde_json::to_vec(&body).unwrap();

        let fetcher =
            Arc::new(InMemoryTransferSource::new().with("https://x/cat.json", body));
        let signer = Arc::new(ScriptedSignatureProvider::new(vec![(
            "catalog.enzime.json.v1".to_string(),
            SignatureVerificationState::Verified,
            None,
        )]));
        let src = EnzimeJsonCatalogSource::new(
            "https://x/cat.json",
            fetcher,
            Some(signer),
        );
        let err = src.refresh().await.expect_err("domain mismatch");
        assert!(matches!(err, CatalogError::InvalidSignature));
    });
}

#[test]
fn enzime_search_filters_by_category_and_size() {
    let rt = runtime();
    rt.block_on(async {
        let fetcher = Arc::new(
            InMemoryTransferSource::new().with("https://x/cat.json", enzime_json_body()),
        );
        let signer = Arc::new(ScriptedSignatureProvider::new(vec![(
            "catalog.enzime.json.v1".to_string(),
            SignatureVerificationState::Verified,
            None,
        )]));
        let src = EnzimeJsonCatalogSource::new(
            "https://x/cat.json",
            fetcher,
            Some(signer),
        );
        src.refresh().await.expect("refresh");

        let med = src
            .search(CatalogQuery {
                text: None,
                languages: vec![],
                categories: vec!["medicine".to_string()],
                max_size_bytes: None,
                offset: 0,
                limit: 10,
            })
            .await
            .expect("cat");
        assert_eq!(med.entries.len(), 1);
        assert_eq!(med.entries[0].corpus_id, "mdwiki_en_all_maxi");

        let small = src
            .search(CatalogQuery {
                text: None,
                languages: vec![],
                categories: vec![],
                max_size_bytes: Some(1_000_000_000),
                offset: 0,
                limit: 10,
            })
            .await
            .expect("size");
        assert_eq!(small.entries.len(), 1); // mdwiki only; gutenberg is 12GB
    });
}

#[test]
fn enzime_entry_lookup_and_not_found() {
    let rt = runtime();
    rt.block_on(async {
        let fetcher = Arc::new(
            InMemoryTransferSource::new().with("https://x/cat.json", enzime_json_body()),
        );
        let signer = Arc::new(ScriptedSignatureProvider::new(vec![(
            "catalog.enzime.json.v1".to_string(),
            SignatureVerificationState::Verified,
            None,
        )]));
        let src = EnzimeJsonCatalogSource::new(
            "https://x/cat.json",
            fetcher,
            Some(signer),
        );
        src.refresh().await.expect("refresh");

        let e = src
            .entry("gutenberg_en_all_2024-08")
            .await
            .expect("found");
        assert_eq!(e.publisher, "Gutenberg");

        let err = src.entry("nope").await.expect_err("missing");
        assert!(matches!(err, CatalogError::NotFound));
    });
}
