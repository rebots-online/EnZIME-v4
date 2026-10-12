pub use crate::error::ZimError;
use crate::identity::{ArticleKey, ContentDigest, EntryKey};
use async_trait::async_trait;
use bytes::Bytes;
use futures_core::Stream;
use serde::{Deserialize, Serialize};
use std::{pin::Pin, sync::Arc};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ZimOpenRequest {
    pub path: String,
    pub expected_sha256: Option<String>,
    pub expected_size_bytes: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ZimMetadata {
    pub uuid: String,
    pub title: String,
    pub description: String,
    pub language_bcp47: String,
    pub creator: String,
    pub publisher: String,
    pub date: String,
    pub article_count: u64,
    pub media_count: u64,
    pub main_page: Option<ArticleKey>,
    pub checksum: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ZimEntryDescriptor {
    pub entry_key: EntryKey,
    pub namespace: char,
    pub path: String,
    pub title: String,
    pub mime_type: String,
    pub kind: ZimEntryKind,
    pub redirect_target: Option<EntryKey>,
    pub byte_length: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZimEntryKind {
    Article,
    Resource,
    Redirect,
    Metadata,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArticleDocument {
    pub article_key: ArticleKey,
    pub title: String,
    pub mime_type: String,
    pub raw_html: Bytes,
    pub normalized_text: String,
    pub content_digest: ContentDigest,
    pub internal_links: Vec<InternalLink>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InternalLink {
    pub href: String,
    pub target: InternalLinkTarget,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum InternalLinkTarget {
    Article { article_key: ArticleKey },
    Resource { entry_key: EntryKey },
    Unsupported { reason: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceDocument {
    pub entry_key: EntryKey,
    pub mime_type: String,
    pub bytes: Bytes,
    pub content_digest: ContentDigest,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TitleSearchQuery {
    pub text: String,
    pub offset: u32,
    pub limit: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TitleSearchHit {
    pub article_key: ArticleKey,
    pub title: String,
    pub score: f32,
}

pub type ZimEntryStream = Pin<Box<dyn Stream<Item = Result<ZimEntryDescriptor, ZimError>> + Send>>;

#[async_trait]
pub trait ZimEngine: Send + Sync {
    fn engine_id(&self) -> &'static str;
    fn engine_version(&self) -> String;
    async fn probe(&self, request: &ZimOpenRequest) -> Result<ZimMetadata, ZimError>;
    async fn open(&self, request: ZimOpenRequest) -> Result<Arc<dyn ZimArchive>, ZimError>;
}

#[async_trait]
pub trait ZimArchive: Send + Sync {
    fn metadata(&self) -> &ZimMetadata;
    async fn main_page(&self) -> Result<Option<ArticleKey>, ZimError>;
    async fn entry(&self, key: &EntryKey) -> Result<ZimEntryDescriptor, ZimError>;
    async fn article(&self, key: &ArticleKey) -> Result<ArticleDocument, ZimError>;
    async fn resource(&self, key: &EntryKey) -> Result<ResourceDocument, ZimError>;
    async fn resolve_redirect(&self, key: &EntryKey, max_hops: u8) -> Result<EntryKey, ZimError>;
    async fn search_titles(&self, query: TitleSearchQuery)
        -> Result<Vec<TitleSearchHit>, ZimError>;
    async fn entries(&self) -> Result<ZimEntryStream, ZimError>;
    async fn verify_structure(&self) -> Result<ZimVerificationReport, ZimError>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ZimVerificationReport {
    pub valid: bool,
    pub engine_id: String,
    pub engine_version: String,
    pub checked_entries: u64,
    pub warnings: Vec<ZimVerificationFinding>,
    pub errors: Vec<ZimVerificationFinding>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ZimVerificationFinding {
    pub code: String,
    pub entry_key: Option<EntryKey>,
    pub detail: String,
}
