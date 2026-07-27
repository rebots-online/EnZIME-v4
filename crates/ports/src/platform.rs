use async_trait::async_trait;
use serde::{Deserialize, Serialize};

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
