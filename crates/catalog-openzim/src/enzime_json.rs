//! `EnzimeJsonCatalogSource` — `CatalogSource` implementation that reads the
//! EnZIME-native JSON catalog format. The catalog carries a top-level
//! `signature` envelope over a canonical serialization of its `entries`.
//!
//! EnZIME catalog format (version `enzime-catalog-v1`):
//!
//! ```json
//! {
//!   "version": "enzime-catalog-v1",
//!   "signature": { "algorithm": "...", "key_id": "...",
//!                  "signed_domain": "...", "value": [bytes...] },
//!   "entries": [ { ...CatalogEntry-compatible fields... } ]
//! }
//! ```
//!
//! The canonical signed payload is `serde_json::to_vec(&parsed.entries)` —
//! i.e. the entries array serialized with default (sorted-key) serde_json
//! output. Catalog producers MUST sign that exact byte sequence.
//!
//! Catalogs missing the `signature` envelope are rejected with
//! `CatalogError::InvalidSignature` when `require_signature` is set (the
//! default). Per-entry `signature_url` values point at *artifact* signatures
//! (independent of the catalog signature) and are passed through verbatim.

use crate::openzim_xml::fetch_bytes;
use crate::store::{
    diff_refresh, entry_matches, epoch_millis, find_entry, paginate,
    SharedSnapshot, Snapshot,
};
use crate::validate::{
    validate_artifact_sha256, validate_artifact_size_bytes, validate_artifact_url,
    validate_catalog_entry_id, validate_language_bcp47, validate_non_empty,
    validate_signature_url,
};
use async_trait::async_trait;
use domain::trust::SignatureEnvelope;
use ports::catalog::{
    CatalogEntry, CatalogError, CatalogPage, CatalogQuery, CatalogRefreshReport,
    CatalogSource, TrustPresentation,
};
use ports::signature::SignatureProvider;
use ports::transfer::TransferSource;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Default ceiling on a single catalog download.
const DEFAULT_MAX_CATALOG_BYTES: u64 = 64 * 1024 * 1024;

/// Default signing domain. The embedded `signature.signed_domain` field of the
/// envelope must match this value for verification to proceed.
const DEFAULT_SIGN_DOMAIN: &str = "catalog.enzime.json.v1";

/// The §8.2 release-implementation id for this adapter.
pub const SOURCE_ID: &str = "catalog.enzime.json.v1";

/// The format version marker this adapter accepts.
const ACCEPTED_VERSION: &str = "enzime-catalog-v1";

pub struct EnzimeJsonCatalogSource {
    source_url: String,
    fetcher: Arc<dyn TransferSource>,
    signature: Option<Arc<dyn SignatureProvider>>,
    sign_domain: String,
    require_signature: bool,
    maximum_catalog_bytes: u64,
    state: SharedSnapshot,
}

impl EnzimeJsonCatalogSource {
    pub fn new(
        source_url: impl Into<String>,
        fetcher: Arc<dyn TransferSource>,
        signature: Option<Arc<dyn SignatureProvider>>,
    ) -> Self {
        Self {
            source_url: source_url.into(),
            fetcher,
            signature,
            sign_domain: DEFAULT_SIGN_DOMAIN.to_string(),
            require_signature: true,
            maximum_catalog_bytes: DEFAULT_MAX_CATALOG_BYTES,
            state: std::sync::RwLock::new(Snapshot::default()),
        }
    }

    pub fn with_sign_domain(mut self, domain: impl Into<String>) -> Self {
        self.sign_domain = domain.into();
        self
    }

    pub fn require_signature(mut self, required: bool) -> Self {
        self.require_signature = required;
        self
    }

    pub fn with_maximum_catalog_bytes(mut self, max: u64) -> Self {
        self.maximum_catalog_bytes = max;
        self
    }
}

#[async_trait]
impl CatalogSource for EnzimeJsonCatalogSource {
    fn source_id(&self) -> &str {
        SOURCE_ID
    }

    async fn refresh(&self) -> Result<CatalogRefreshReport, CatalogError> {
        let raw = fetch_bytes(
            self.fetcher.as_ref(),
            &self.source_url,
            self.maximum_catalog_bytes,
        )
        .await?;

        let parsed: EnzimeCatalogDoc = parse_enzime_json(&raw)?;

        // Canonical payload for signature verification = serialized entries
        // (serde_json default = sorted keys, deterministic).
        let canonical = serde_json::to_vec(&parsed.entries).map_err(|e| {
            CatalogError::Malformed(format!(
                "canonical entry serialization failed: {e}"
            ))
        })?;

        let trust = verify_inline_signature(
            self.signature.as_deref(),
            &self.sign_domain,
            &canonical,
            parsed.signature.as_ref(),
            self.require_signature,
        )
        .await?;

        let entries = normalize_entries(parsed.entries, &trust)?;
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
// Parsing & normalization
// ---------------------------------------------------------------------------

/// On-the-wire EnZIME catalog document. The `entries` use a lenient shape
/// (every required field is optional here) so a single normalization pass
/// can reject specific malformed entries instead of aborting at the serde
/// layer.
#[derive(Deserialize, Debug)]
struct EnzimeCatalogDoc {
    version: String,
    #[serde(default)]
    signature: Option<SignatureEnvelope>,
    #[serde(default)]
    entries: Vec<EnzimeEntryDoc>,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
struct EnzimeEntryDoc {
    #[serde(default)]
    catalog_entry_id: String,
    #[serde(default)]
    corpus_id: String,
    #[serde(default)]
    upstream_version: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    language_bcp47: String,
    #[serde(default)]
    category: String,
    #[serde(default)]
    publisher: String,
    #[serde(default)]
    artifact_url: String,
    #[serde(default)]
    artifact_size_bytes: u64,
    #[serde(default)]
    artifact_sha256: String,
    #[serde(default)]
    signature_url: Option<String>,
}

fn parse_enzime_json(raw: &[u8]) -> Result<EnzimeCatalogDoc, CatalogError> {
    let doc: EnzimeCatalogDoc = serde_json::from_slice(raw).map_err(|e| {
        CatalogError::Malformed(format!("catalog JSON parse failed: {e}"))
    })?;
    if doc.version != ACCEPTED_VERSION {
        return Err(CatalogError::Malformed(format!(
            "unsupported catalog version '{}' (expected '{}')",
            doc.version, ACCEPTED_VERSION
        )));
    }
    Ok(doc)
}

fn normalize_entries(
    docs: Vec<EnzimeEntryDoc>,
    trust: &TrustPresentation,
) -> Result<Vec<CatalogEntry>, CatalogError> {
    let mut out = Vec::with_capacity(docs.len());
    for doc in docs {
        validate_catalog_entry_id(&doc.catalog_entry_id)?;
        validate_non_empty("corpus_id", &doc.corpus_id)?;
        validate_non_empty("title", &doc.title)?;
        validate_non_empty("upstream_version", &doc.upstream_version)?;
        validate_language_bcp47(&doc.language_bcp47)?;
        validate_non_empty("category", &doc.category)?;
        validate_non_empty("publisher", &doc.publisher)?;
        validate_artifact_url(&doc.artifact_url)?;
        validate_artifact_size_bytes(doc.artifact_size_bytes)?;
        validate_artifact_sha256(&doc.artifact_sha256)?;
        if let Some(sig_url) = doc.signature_url.as_deref() {
            validate_signature_url(sig_url)?;
        }

        out.push(CatalogEntry {
            catalog_entry_id: doc.catalog_entry_id,
            corpus_id: doc.corpus_id,
            upstream_version: doc.upstream_version,
            title: doc.title,
            description: doc.description,
            language_bcp47: doc.language_bcp47,
            category: doc.category,
            publisher: doc.publisher,
            artifact_url: doc.artifact_url,
            artifact_size_bytes: doc.artifact_size_bytes,
            artifact_sha256: doc.artifact_sha256,
            signature_url: doc.signature_url,
            trust: trust.clone(),
        });
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Signature verification (inline envelope)
// ---------------------------------------------------------------------------

async fn verify_inline_signature(
    signature: Option<&dyn SignatureProvider>,
    sign_domain: &str,
    canonical_payload: &[u8],
    envelope: Option<&SignatureEnvelope>,
    require_signature: bool,
) -> Result<TrustPresentation, CatalogError> {
    let Some(signature) = signature else {
        if require_signature {
            return Err(CatalogError::InvalidSignature);
        }
        return Ok(TrustPresentation {
            state: "unsigned".to_string(),
            signer: None,
            explanation: "No signature provider configured.".to_string(),
        });
    };

    let Some(envelope) = envelope else {
        if require_signature {
            return Err(CatalogError::InvalidSignature);
        }
        return Ok(TrustPresentation {
            state: "unsigned".to_string(),
            signer: None,
            explanation: "Catalog does not carry a signature envelope."
                .to_string(),
        });
    };

    if envelope.signed_domain != sign_domain {
        return Err(CatalogError::InvalidSignature);
    }

    let verification = signature
        .verify(sign_domain, canonical_payload, envelope)
        .await
        .map_err(|_| CatalogError::InvalidSignature)?;

    match verification.state {
        domain::trust::SignatureVerificationState::Verified => {
            Ok(TrustPresentation {
                state: "verified".to_string(),
                signer: verification.signer_label.clone(),
                explanation: verification.explanation.clone(),
            })
        }
        domain::trust::SignatureVerificationState::Unsigned => {
            if require_signature {
                Err(CatalogError::InvalidSignature)
            } else {
                Ok(TrustPresentation {
                    state: "unsigned".to_string(),
                    signer: verification.signer_label,
                    explanation: verification.explanation,
                })
            }
        }
        _ => Err(CatalogError::InvalidSignature),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry_doc(id: &str) -> EnzimeEntryDoc {
        EnzimeEntryDoc {
            catalog_entry_id: id.to_string(),
            corpus_id: id.to_string(),
            upstream_version: "1.0".to_string(),
            title: id.to_string(),
            description: String::new(),
            language_bcp47: "en".to_string(),
            category: "general".to_string(),
            publisher: "enzime".to_string(),
            artifact_url: "https://host/a.zim".to_string(),
            artifact_size_bytes: 100,
            artifact_sha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
            signature_url: None,
        }
    }

    fn trust() -> TrustPresentation {
        TrustPresentation {
            state: "verified".to_string(),
            signer: None,
            explanation: String::new(),
        }
    }

    #[test]
    fn parses_minimal_unsigned_doc_when_allowed() {
        let json = serde_json::json!({
            "version": "enzime-catalog-v1",
            "entries": [ serde_json::json!({
                "catalog_entry_id": "x",
                "corpus_id": "x",
                "upstream_version": "1.0",
                "title": "X",
                "language_bcp47": "en",
                "category": "general",
                "publisher": "p",
                "artifact_url": "https://h/x.zim",
                "artifact_size_bytes": 5,
                "artifact_sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            })],
        });
        let doc = parse_enzime_json(&serde_json::to_vec(&json).unwrap()).expect("parse");
        let entries = normalize_entries(doc.entries, &trust()).expect("normalize");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].catalog_entry_id, "x");
    }

    #[test]
    fn rejects_wrong_version() {
        let json = serde_json::json!({
            "version": "enzime-catalog-v2",
            "entries": [],
        });
        let err = parse_enzime_json(&serde_json::to_vec(&json).unwrap())
            .expect_err("version");
        assert!(matches!(err, CatalogError::Malformed(_)));
    }

    #[test]
    fn rejects_bad_url_scheme() {
        let mut doc = entry_doc("bad");
        doc.artifact_url = "ftp://h/x.zim".to_string();
        let err = normalize_entries(vec![doc], &trust()).expect_err("scheme");
        assert!(matches!(err, CatalogError::Malformed(_)));
    }

    #[test]
    fn rejects_uppercase_sha256() {
        let mut doc = entry_doc("bad");
        doc.artifact_sha256 =
            "A123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                .to_string();
        let err = normalize_entries(vec![doc], &trust()).expect_err("upper sha");
        assert!(matches!(err, CatalogError::Malformed(_)));
    }

    #[test]
    fn rejects_zero_size() {
        let mut doc = entry_doc("bad");
        doc.artifact_size_bytes = 0;
        let err = normalize_entries(vec![doc], &trust()).expect_err("zero");
        assert!(matches!(err, CatalogError::Malformed(_)));
    }

    #[test]
    fn rejects_whitespace_id() {
        let doc = entry_doc("has space");
        let err = normalize_entries(vec![doc], &trust()).expect_err("ws");
        assert!(matches!(err, CatalogError::Malformed(_)));
    }

    #[test]
    fn rejects_bad_language() {
        let mut doc = entry_doc("x");
        doc.language_bcp47 = "en género".to_string();
        let err = normalize_entries(vec![doc], &trust()).expect_err("lang");
        assert!(matches!(err, CatalogError::Malformed(_)));
    }
}
