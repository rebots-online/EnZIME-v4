use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceIdentity {
    pub uri: String,
    pub size_bytes: u64,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub zim_uuid: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiskPackPolicy {
    pub disk_budget_bytes: u64,
    pub metadata_budget_bytes: u64,
    pub workspace_budget_bytes: u64,
    pub offline: bool,
}
impl DiskPackPolicy {
    pub fn payload_budget(&self) -> Result<u64, String> {
        self.disk_budget_bytes
            .checked_sub(self.metadata_budget_bytes)
            .and_then(|n| n.checked_sub(self.workspace_budget_bytes))
            .ok_or_else(|| "budget_exceeded: reservations exceed total".into())
    }
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.disk_budget_bytes < 16 * 1024 * 1024
            || self.disk_budget_bytes > i64::MAX as u64
            || self.metadata_budget_bytes != (64 * 1024 * 1024).min(self.disk_budget_bytes / 8)
            || self.workspace_budget_bytes != (512 * 1024 * 1024).min(self.disk_budget_bytes / 4)
        {
            return Err("invalid_disk_policy".into());
        }
        self.payload_budget()?;
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiskPackManifest {
    pub format_version: u32,
    pub pack_id: String,
    pub pack_version_id: String,
    pub identity: SourceIdentity,
    pub policy: DiskPackPolicy,
    pub metadata: anzimmerman::model::ZimMetadata,
    pub main_page: Option<String>,
    pub state: String,
}
