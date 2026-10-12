//! Bounded Kiwix acquisition orchestration. Format knowledge stays in AnZimmermanLib.
use super::*;
use std::path::PathBuf;
use zim_cluster_cache::disk::{
    discover_offline_entries, install_selected, open_disk_archive, plan_disk_fit, DiskPackManifest,
    DiskPackPolicy, DiskPackStore, DiskRangeSource, OfflineCatalogProgress, PlanSummary,
    SourceIdentity,
};
mod install;
mod persistence;
mod prepare;
pub use install::{cancel_dyndon_install, restore_dyndon_archives, run_dyndon_install};
pub use prepare::prepare_dyndon_install;
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DynDonInstallRequest {
    pub catalog_entry_id: String,
    pub destination_storage_root_id: String,
    pub budget_bytes: u64,
    pub selection_profile_id: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DynDonPlanRecord {
    pub install_plan_id: String,
    pub pack_id: String,
    pub pack_version_id: String,
    pub store_relative_path: String,
    pub destination_storage_root_id: String,
    pub summary: PlanSummary,
}
#[derive(Serialize, Deserialize)]
struct SavedPlan {
    record: DynDonPlanRecord,
    catalog_entry_id: String,
    expected_sha256: Option<String>,
    upstream_version: String,
    #[serde(default)]
    offline_catalog: Option<OfflineCatalogProgress>,
}
fn digest(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    hex::encode(Sha256::digest(bytes))
}
