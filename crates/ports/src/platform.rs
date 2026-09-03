use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::inference::AcceleratorKind;

/// Errors produced by a platform adapter (§17.2). Verbatim from
/// ARCHITECTURE.md §17.2.
#[derive(Clone, Debug, thiserror::Error)]
pub enum PlatformError {
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    #[error("capability unavailable: {0}")]
    CapabilityUnavailable(String),
    #[error("storage root unavailable: {0}")]
    StorageRootUnavailable(String),
    #[error("invalid or expired file grant: {0}")]
    InvalidGrant(String),
    #[error("cancelled by user")]
    Cancelled,
    #[error("unsupported on this platform: {0}")]
    UnsupportedOnPlatform(String),
    #[error("I/O error: {0}")]
    Io(String),
}

/// Raw device capability probe (§17.2).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlatformCapabilityReport {
    pub cpu: String,
    pub ram_bytes: u64,
    pub free_storage_bytes: u64,
    pub accelerators: Vec<AcceleratorKind>,
    pub background_transfer_granted: bool,
    pub notifications_granted: bool,
    pub microphone_granted: bool,
    pub inference_viable: bool,
}

/// Maps one-for-one to the `kind` CHECK on the `storage_roots` table (§6.2).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageRootKind {
    AppPrivate,
    UserSelected,
    Removable,
}

/// Raw storage-root probe (§17.2). Column-aligned with `storage_roots` (§6.2).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlatformStorageRoot {
    pub storage_root_id: String,
    pub kind: StorageRootKind,
    pub platform_locator: String,
    pub display_name: String,
    pub writable: bool,
    pub capacity_bytes: Option<u64>,
    pub free_bytes: Option<u64>,
}

/// Thermal classification used by the §8 transfer policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThermalLoad {
    Nominal,
    Moderate,
    Severe,
}

/// Power/battery observation (§17.2).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PowerState {
    pub on_charger: bool,
    pub battery_percent: Option<u8>,
    pub battery_saver: bool,
    pub thermal_load: ThermalLoad,
}

/// Connectivity classification used by the §8 transfer policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkState {
    Offline,
    Metered,
    Unmetered,
}

#[async_trait]
pub trait PlatformAdapter: Send + Sync {
    fn platform(&self) -> PlatformKind;
    async fn capability_report(&self) -> Result<PlatformCapabilityReport, PlatformError>;
    async fn storage_roots(&self) -> Result<Vec<PlatformStorageRoot>, PlatformError>;
    async fn choose_import_file(
        &self,
        accepted_media_types: &[String],
    ) -> Result<PlatformFileToken, PlatformError>;
    async fn choose_export_file(
        &self,
        suggested_name: &str,
        media_type: &str,
    ) -> Result<PlatformFileToken, PlatformError>;
    async fn open_read(
        &self,
        token: &PlatformFileToken,
    ) -> Result<Box<dyn tokio::io::AsyncRead + Unpin + Send>, PlatformError>;
    async fn open_write(
        &self,
        token: &PlatformFileToken,
        truncate: bool,
    ) -> Result<Box<dyn tokio::io::AsyncWrite + Unpin + Send>, PlatformError>;
    async fn power_state(&self) -> Result<PowerState, PlatformError>;
    async fn network_state(&self) -> Result<NetworkState, PlatformError>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlatformKind {
    Android,
    Linux,
    Windows,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlatformFileToken {
    pub opaque_token: String,
    pub display_name: String,
    pub persistable: bool,
}
