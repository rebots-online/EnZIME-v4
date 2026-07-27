//! Transfer port — §8.2 transfer probe/range/stream types, the
//! `TransferSource` trait, the `TransferError` enum, §8.3 install-plan types
//! and §8.5 transfer limit constants. Provider implementations live in Ring 2.

use async_trait::async_trait;
use bytes::Bytes;
use domain::identity::PackVersionId;
use futures_core::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

use crate::catalog::CatalogEntry;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransferProbeRequest {
    pub uri: String,
    pub maximum_response_bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransferProbe {
    pub uri: String,
    pub size_bytes: u64,
    pub accepts_ranges: bool,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub content_type: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransferRange {
    pub start_byte: u64,
    pub end_byte_exclusive: u64,
}

pub type TransferByteStream =
    Pin<Box<dyn Stream<Item = Result<Bytes, TransferError>> + Send>>;

#[async_trait]
pub trait TransferSource: Send + Sync {
    fn source_kind(&self) -> &'static str;
    async fn probe(
        &self,
        request: TransferProbeRequest,
    ) -> Result<TransferProbe, TransferError>;
    async fn open_range(
        &self,
        uri: &str,
        range: TransferRange,
        expected_etag: Option<&str>,
    ) -> Result<TransferByteStream, TransferError>;
    async fn open_full(
        &self,
        uri: &str,
        maximum_response_bytes: u64,
    ) -> Result<TransferByteStream, TransferError>;
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum TransferError {
    #[error("offline")]
    Offline,
    #[error("source changed")]
    SourceChanged,
    #[error("range unsupported")]
    RangeUnsupported,
    #[error("range response invalid")]
    InvalidRangeResponse,
    #[error("response exceeds limit")]
    ResponseLimitExceeded,
    #[error("source not found")]
    NotFound,
    #[error("permission denied")]
    PermissionDenied,
    #[error("transport error: {0}")]
    Transport(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstallPlan {
    pub install_plan_id: String,
    pub catalog_entry: CatalogEntry,
    pub destination: StorageDestination,
    pub transfer_mode: TransferMode,
    pub download_bytes: u64,
    pub temporary_bytes: u64,
    pub final_bytes: u64,
    pub free_bytes_before: u64,
    pub free_bytes_after: u64,
    pub verification: VerificationPlan,
    pub replaces_pack_version_id: Option<PackVersionId>,
    pub retained_pack_version_id: Option<PackVersionId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StorageDestination {
    pub storage_root_id: String,
    pub display_name: String,
    pub writable: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferMode {
    Ranged,
    Full,
    LocalCopy,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerificationPlan {
    pub expected_size_bytes: u64,
    pub expected_sha256: String,
    pub signature_required: bool,
    pub zim_structure_check: bool,
}

// §8.5 transfer limits — release values. HTTP `200` to a range request is not
// accepted as a ranged response; the user may explicitly approve conversion to
// a bounded full transfer.
pub const TRANSFER_CHUNK_BYTES: u64 = 8 * 1024 * 1024;
pub const TRANSFER_MAX_PARALLEL_CHUNKS_ANDROID: u32 = 2;
pub const TRANSFER_MAX_PARALLEL_CHUNKS_DESKTOP: u32 = 4;
pub const TRANSFER_MAX_REDIRECTS: u32 = 5;
pub const TRANSFER_PROBE_MAX_BYTES: u64 = 1024 * 1024;
pub const TRANSFER_CATALOG_MAX_BYTES: u64 = 32 * 1024 * 1024;
pub const TRANSFER_RETRY_COUNT: u32 = 5;
pub const TRANSFER_RETRY_BASE_MS: u64 = 500;
pub const TRANSFER_RETRY_MAX_MS: u64 = 30000;
