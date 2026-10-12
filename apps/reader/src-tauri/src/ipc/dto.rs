use super::*;
// ===========================================================================
// DTOs — readiness
// ===========================================================================

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemReadiness {
    pub overall: String,
    pub summary: String,
    pub flags: Vec<ReadinessFlag>,
    /// Frontend binds this as `checkedAt: string`. M1 does not render it; an
    /// empty string is truthful until a real clock-string is wired.
    pub checked_at: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadinessFlag {
    pub id: String,
    pub level: String,
    pub label: String,
    pub detail: String,
    pub setting_path: String,
}

pub(super) fn build_readiness(schema_applied: bool, _checked_at_ms: i64) -> SystemReadiness {
    let overall = if schema_applied { "ready" } else { "warning" };
    // When the system is ready there are no outstanding flags — the frontend
    // then renders "No outstanding configuration issues." Only surface a flag
    // when something genuinely needs attention.
    let flags = if schema_applied {
        Vec::new()
    } else {
        vec![ReadinessFlag {
            id: "schema".to_string(),
            level: "warning".to_string(),
            label: "Schema pending".to_string(),
            detail: "Initial schema not yet applied.".to_string(),
            setting_path: "/settings".to_string(),
        }]
    };
    SystemReadiness {
        overall: overall.to_string(),
        summary: if schema_applied {
            "M1 baseline met".to_string()
        } else {
            "schema pending".to_string()
        },
        flags,
        checked_at: String::new(),
    }
}

// ===========================================================================
// DTOs — bootstrap
// ===========================================================================

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppBootstrap {
    pub schema_version: i32,
    pub app_version: String,
    pub platform: String,
    pub first_launch: bool,
    pub routes: Vec<RouteDescriptor>,
    pub capabilities: Vec<serde_json::Value>,
    pub readiness: SystemReadiness,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteDescriptor {
    pub path: String,
    pub enabled: bool,
}

pub(super) const M1_ROUTES: &[(&str, bool)] = &[
    ("/", true),
    ("/library", true),
    ("/reader/:packId/:articleKey", true),
    ("/ask", true),
    ("/notes", true),
    ("/memory", true),
    ("/settings", true),
    ("/setup", true),
];

// ===========================================================================
// DTOs — home snapshot
// ===========================================================================

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryInboxEntryDto {
    pub memory_id: String,
    pub summary: String,
    pub source: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HomeSnapshot {
    pub recent_packs: Vec<PackManifest>,
    pub continue_reading: Option<ReaderHistoryItem>,
    pub task_inbox: Vec<UserTask>,
    /// Frontend binds this as an array (`MemoryInboxEntry[]`), not a count.
    pub memory_inbox: Vec<MemoryInboxEntryDto>,
    pub readiness: SystemReadiness,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackManifest {
    pub pack_id: String,
    pub corpus_id: String,
    pub display_name: String,
    pub description: String,
    pub language_bcp47: String,
    pub pack_kind: String,
    pub active_version_id: Option<String>,
    pub versions: Vec<PackVersionSummary>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackVersionSummary {
    pub pack_version_id: String,
    pub upstream_version: String,
    pub artifact_sha256: Option<String>,
    pub artifact_size_bytes: i64,
    pub verification_state: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReaderHistoryItem {
    pub pack_id: String,
    pub article_key: String,
    pub title: String,
    pub visited_at_ms: i64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserTask {
    pub task_id: String,
    pub task_kind: String,
    pub title: String,
    pub detail: serde_json::Value,
    pub priority: i64,
    pub created_at_ms: i64,
}

// ===========================================================================
// DTOs — reader (repointed to INSTALLED packs)
// ===========================================================================

/// `reader_open(packId)` response. Minimal M1 shape: the pack id, its title,
/// and the main-page article key the reader should render first.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReaderOpenResponse {
    pub pack_id: String,
    pub title: String,
    pub main_page_key: Option<String>,
}

/// `reader_article({packId, articleKey})` response. The frontend renders
/// `rawHtml` in a sandboxed iframe.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArticleResponse {
    pub pack_id: String,
    pub article_key: String,
    pub title: String,
    pub raw_html: String,
    pub mime_type: String,
    pub internal_links: Vec<InternalLinkDto>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InternalLinkDto {
    pub href: String,
    pub article_key: Option<String>,
    pub entry_key: Option<String>,
    pub unsupported_reason: Option<String>,
}

// ===========================================================================
// DTOs — Library / DynDon vertical (§14.4 wire contract)
// ===========================================================================

/// `storage_roots_list` row.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageDestinationDto {
    pub storage_root_id: String,
    pub display_name: String,
    pub writable: bool,
}

/// `pack_list` row — installed pack manifest for the Library view.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryPackManifest {
    pub pack_id: String,
    pub display_name: String,
    pub description: String,
    pub language_bcp47: String,
    pub article_count: u64,
    pub size_bytes: u64,
    pub last_opened_at: Option<String>,
    pub main_page_key: Option<String>,
}

/// `catalog_search` row — a searchable catalog entry.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryCatalogEntry {
    pub catalog_entry_id: String,
    pub title: String,
    pub description: String,
    pub language_bcp47: String,
    pub category: String,
    pub publisher: String,
    pub artifact_url: String,
    pub artifact_size_bytes: u64,
    pub artifact_sha256: Option<String>,
}

/// `install_plan` / `job_list` paging envelope (mirrors annotationService).
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PageResultDto<T> {
    pub items: Vec<T>,
    pub total: Option<u64>,
    pub next_offset: Option<u32>,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferModeDto {
    Ranged,
    Full,
    LocalCopy,
}

impl From<TransferMode> for TransferModeDto {
    fn from(m: TransferMode) -> Self {
        match m {
            TransferMode::Ranged => TransferModeDto::Ranged,
            TransferMode::Full => TransferModeDto::Full,
            TransferMode::LocalCopy => TransferModeDto::LocalCopy,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallPlanVerificationDto {
    pub expected_size_bytes: u64,
    pub expected_sha256: Option<String>,
    pub signature_required: bool,
    pub zim_structure_check: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OfflineCatalogProgressDto {
    pub scanned_entries: u32,
    pub total_entries: u32,
    pub planned_article_count: u64,
    pub planned_resource_count: u64,
    pub planned_alias_count: u64,
    pub network_allowance_bytes: u64,
    pub network_reserved_bytes: u64,
    pub network_received_bytes: u64,
    pub complete: bool,
    pub stop_reason: Option<String>,
}
impl From<zim_cluster_cache::disk::OfflineCatalogProgress> for OfflineCatalogProgressDto {
    fn from(progress: zim_cluster_cache::disk::OfflineCatalogProgress) -> Self {
        Self {
            scanned_entries: progress.scanned_entries,
            total_entries: progress.total_entries,
            planned_article_count: progress.readable_articles,
            planned_resource_count: progress.readable_resources,
            planned_alias_count: progress.retained_aliases,
            network_allowance_bytes: progress.network_allowance_bytes,
            network_reserved_bytes: progress.network_reserved_bytes,
            network_received_bytes: progress.network_received_bytes,
            complete: progress.complete,
            stop_reason: progress.stop_reason,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallPlanDto {
    pub install_plan_id: String,
    pub catalog_entry: LibraryCatalogEntry,
    pub destination: StorageDestinationDto,
    pub transfer_mode: TransferModeDto,
    pub download_bytes: u64,
    pub temporary_bytes: u64,
    pub final_bytes: u64,
    pub free_bytes_before: u64,
    pub free_bytes_after: u64,
    pub verification: InstallPlanVerificationDto,
    pub offline_catalog: Option<OfflineCatalogProgressDto>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadJobDto {
    pub download_job_id: String,
    pub state: String,
    pub bytes_verified: u64,
    pub bytes_retained: u64,
    pub transfer_scope: String,
    pub error_detail: Option<String>,
    pub expected_size_bytes: u64,
}
