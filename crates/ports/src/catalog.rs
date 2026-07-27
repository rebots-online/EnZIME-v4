//! Catalog port — §8.2 catalog query/page/entry types, the `CatalogSource`
//! trait and the `CatalogError` enum. Provider implementations live in Ring 2.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CatalogQuery {
    pub text: Option<String>,
    pub languages: Vec<String>,
    pub categories: Vec<String>,
    pub max_size_bytes: Option<u64>,
    pub offset: u32,
    pub limit: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CatalogPage {
    pub entries: Vec<CatalogEntry>,
    pub total: Option<u64>,
    pub next_offset: Option<u32>,
    pub refreshed_at_ms: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CatalogEntry {
    pub catalog_entry_id: String,
    pub corpus_id: String,
    pub upstream_version: String,
    pub title: String,
    pub description: String,
    pub language_bcp47: String,
    pub category: String,
    pub publisher: String,
    pub artifact_url: String,
    pub artifact_size_bytes: u64,
    pub artifact_sha256: String,
    pub signature_url: Option<String>,
    pub trust: TrustPresentation,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrustPresentation {
    pub state: String,
    pub signer: Option<String>,
    pub explanation: String,
}

#[async_trait]
pub trait CatalogSource: Send + Sync {
    fn source_id(&self) -> &str;
    async fn refresh(&self) -> Result<CatalogRefreshReport, CatalogError>;
    async fn search(&self, query: CatalogQuery) -> Result<CatalogPage, CatalogError>;
    async fn entry(&self, catalog_entry_id: &str) -> Result<CatalogEntry, CatalogError>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CatalogRefreshReport {
    pub source_id: String,
    pub added: u64,
    pub updated: u64,
    pub removed: u64,
    pub signature_state: String,
    pub refreshed_at_ms: i64,
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum CatalogError {
    #[error("offline")]
    Offline,
    #[error("authentication failed")]
    AuthenticationFailed,
    #[error("catalog signature invalid")]
    InvalidSignature,
    #[error("catalog malformed: {0}")]
    Malformed(String),
    #[error("entry not found")]
    NotFound,
    #[error("transport error: {0}")]
    Transport(String),
}
