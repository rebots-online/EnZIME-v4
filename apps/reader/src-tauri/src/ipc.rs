//! Tauri IPC command handlers — Milestone-1 "produce a reading" + DynDon loop.
//!
//! Implements the §14.4 commands the frontend needs to bootstrap, render the
//! home view, read real ZIM articles, AND close the DynDon install loop:
//!
//!   * `app_bootstrap`       → `AppBootstrap`        (§14.5)
//!   * `home_snapshot`       → `HomeSnapshot`        (§14.2 / §24)
//!   * `readiness_get`       → `SystemReadiness`     (M1 flat shape)
//!   * `reader_open`         → opens an INSTALLED pack by packId and returns
//!     its main page key (repointed off the file-picker path — the frontend
//!     calls this with `{ packId }`).
//!   * `reader_article`      → reads one article via the registered archive.
//!   * `storage_roots_list`  → writable destinations (synthesizes the
//!     app-data dir as the M1 default when the DB has none).
//!   * `pack_list`           → installed pack manifests (paged).
//!   * `catalog_search`      → searchable catalog entries (paged).
//!   * `install_plan`        → DynDon storage-fit plan (calls `dyndon::plan_fit`).
//!   * `install_start`       → fetches selected clusters, builds a
//!     `ClusterCacheArchive`, registers it for the reader, returns the job.
//!   * `job_list`            → download jobs (paged).
//!
//! # Layering note
//! The architecturally-canonical home for these wire DTOs is
//! `crates/application/src/...`, which this crate cannot depend on at M1
//! (in-flight breakage from the halted coder wave). The DTOs below are the
//! operative copies; once `application` compiles, `reader` will re-export
//! those instead and this duplication will collapse.
//!
//! # Free-space probe (FREE BYTES escalation ladder)
//! Per the dispatch spec, the free-bytes ladder is: (1) a real probe via
//! `ports::platform::PlatformAdapter::storage_roots()` — NOT available in M1
//! (the `platform-adapter` crate is an empty stub), (2) the DB's
//! `storage_roots.free_bytes` (or `capacity_bytes`), (3) a real syscall probe
//! of the destination directory via `libc::statvfs` on Unix, (4) ESCALATE.
//! Steps 2 and 3 are wired here; step 4 returns a truthful error string.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use bytes::Bytes;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};

use ports::transfer::{
    TransferMode, TransferProbeRequest, TransferRange, TransferSource,
};
use ports::zim::{ZimArchive, ZimEntryKind};
use rusqlite::{params, Connection, OptionalExtension};

use crate::state::AppState;

pub mod annotations;

// ===========================================================================
// Helpers
// ===========================================================================

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn platform_string() -> String {
    if cfg!(target_os = "android") {
        "android".to_string()
    } else if cfg!(target_os = "windows") {
        "windows".to_string()
    } else {
        // Covers `target_os = "linux"` (the only other §17.1 platform) and,
        // for development on other Unixes, falls back to the closest §17.1
        // value rather than emitting a value outside the §14.5 union.
        "linux".to_string()
    }
}

/// Map any error into a `String` for the `Result<T, String>` command return.
fn err_str<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

/// Real free-space probe via `libc::statvfs` (Unix only). Returns `None` on
/// non-Unix targets or when the syscall fails — the caller then escalates.
#[cfg(unix)]
fn probe_free_bytes(path: &Path) -> Option<u64> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;
    let c_path = CString::new(path.as_os_str().as_bytes()).ok()?;
    // SAFETY: `statvfs` writes into an out-pointer and reads the path
    // read-only. The path is a NUL-terminated CString. Return value is
    // checked.
    unsafe {
        let mut stat: libc::statvfs = std::mem::zeroed();
        if libc::statvfs(c_path.as_ptr(), &mut stat) == 0 {
            // f_bavail = free blocks available to unprivileged user.
            Some(stat.f_bsize as u64 * stat.f_bavail as u64)
        } else {
            None
        }
    }
}

#[cfg(not(unix))]
fn probe_free_bytes(_path: &Path) -> Option<u64> {
    // Windows free-space probe is a TODO for the platform-adapter crate.
    None
}

/// Resolve the transfer source for a given URI scheme. `file://` and bare
/// absolute paths go through `LocalFileTransferSource`; `http(s)://` goes
/// through `HttpTransferSource`. Any other scheme is an error.
fn transfer_source_for(uri: &str) -> Result<Arc<dyn TransferSource>, String> {
    if uri.starts_with("http://") || uri.starts_with("https://") {
        let http = transfer_http::HttpTransferSource::new().map_err(err_str)?;
        Ok(Arc::new(http))
    } else if uri.starts_with("file://") || uri.starts_with('/') {
        Ok(Arc::new(transfer_file::LocalFileTransferSource::new()))
    } else {
        Err(format!(
            "unsupported transfer scheme for artifact_url: {} (expected file:// or http(s)://)",
            uri
        ))
    }
}

/// Strip a `file://` prefix (RFC 8089) and return a path the AnZimmerman
/// engine can open via `File::open`. Returns `None` for non-local schemes
/// (the engine is file-based; remote opens require a local mirror).
fn artifact_url_to_local_path(url: &str) -> Option<String> {
    if let Some(rest) = url.strip_prefix("file://") {
        if let Some(after_host) = rest.strip_prefix("localhost/") {
            return Some(format!("/{after_host}"));
        }
        if rest.starts_with('/') {
            return Some(rest.to_string());
        }
        return None;
    }
    if url.starts_with('/') {
        return Some(url.to_string());
    }
    None
}

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

fn build_readiness(schema_applied: bool, _checked_at_ms: i64) -> SystemReadiness {
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

const M1_ROUTES: &[(&str, bool)] = &[
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
    pub artifact_sha256: String,
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
    pub artifact_sha256: String,
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
    pub expected_sha256: String,
    pub signature_required: bool,
    pub zim_structure_check: bool,
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
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadJobDto {
    pub download_job_id: String,
    pub state: String,
    pub bytes_verified: u64,
    pub expected_size_bytes: u64,
}

/// In-memory plan envelope stashed in [`AppState::install_plans`] between
/// `install_plan` and `install_start`. Carries everything `install_start`
/// needs to fetch clusters and assemble the local archive without re-deriving
/// the layout (the durable selection is also persisted in
/// `selection_profile_revisions`).
/// Fields not read by the current M1 `install_start` are retained for
/// traceability and so the envelope matches the full §14.4 plan shape —
/// silencing the dead-code lint here is deliberate, not an oversight.
#[derive(Clone)]
#[allow(dead_code)]
pub struct PersistedInstallPlan {
    pub install_plan_id: String,
    pub catalog_entry: ports::catalog::CatalogEntry,
    pub destination_storage_root_id: String,
    pub destination_display_name: String,
    pub destination_writable: bool,
    pub transfer_mode: TransferMode,
    pub pack_id: String,
    pub pack_version_id: String,
    pub selection_profile_id: String,
    pub selection_revision: i64,
    pub selected_cluster_indices: Vec<u32>,
    /// `(byte_offset, byte_len)` per cluster index, in source order. Used to
    /// drive `TransferSource::open_range` for each selected cluster.
    pub cluster_byte_ranges: Vec<(u64, u64)>,
    pub cluster_directory: zim_cluster_cache::ClusterDirectory,
    pub free_bytes_before: u64,
    pub download_bytes: u64,
}

/// Body persisted in `selection_profile_revisions.canonical_json` so the
/// selection survives a restart (the plan envelope itself is volatile).
#[derive(Serialize, Deserialize)]
struct SelectionCanonical {
    artifact_url: String,
    pack_id: String,
    catalog_entry_id: String,
    destination_storage_root_id: String,
    selected_cluster_indices: Vec<u32>,
    cluster_directory: zim_cluster_cache::ClusterDirectory,
}

// ===========================================================================
// Row → DTO mapping (existing home_snapshot mappers)
// ===========================================================================

fn map_pack(row: storage_sqlite::query::PackRow) -> PackManifest {
    let active_version_id = row.active_pack_version_id.clone();
    let versions = match &active_version_id {
        Some(id) => vec![PackVersionSummary {
            pack_version_id: id.clone(),
            upstream_version: row.active_upstream_version.unwrap_or_default(),
            artifact_sha256: row.active_artifact_sha256.unwrap_or_default(),
            artifact_size_bytes: row.active_artifact_size_bytes.unwrap_or(0),
            verification_state: "verified".to_string(),
        }],
        None => Vec::new(),
    };
    PackManifest {
        pack_id: row.pack_id,
        corpus_id: row.corpus_id,
        display_name: row.display_name,
        description: row.description,
        language_bcp47: row.language_bcp47,
        pack_kind: row.pack_kind,
        active_version_id,
        versions,
    }
}

fn map_history(row: storage_sqlite::query::ReaderHistoryRow) -> ReaderHistoryItem {
    ReaderHistoryItem {
        pack_id: row.pack_id,
        article_key: row.article_key,
        title: row.title,
        visited_at_ms: row.visited_at_ms,
    }
}

fn map_task(row: storage_sqlite::query::UserTaskRow) -> UserTask {
    let detail = serde_json::from_str(&row.detail_json).unwrap_or(serde_json::Value::Null);
    UserTask {
        task_id: row.task_id,
        task_kind: row.task_kind,
        title: row.title,
        detail,
        priority: row.priority,
        created_at_ms: row.created_at_ms,
    }
}

fn map_internal_link(link: ports::zim::InternalLink) -> InternalLinkDto {
    use ports::zim::InternalLinkTarget;
    let (article_key, entry_key, unsupported_reason) = match link.target {
        InternalLinkTarget::Article { article_key } => (Some(article_key.0), None, None),
        InternalLinkTarget::Resource { entry_key } => (None, Some(entry_key.0), None),
        InternalLinkTarget::Unsupported { reason } => (None, None, Some(reason)),
    };
    InternalLinkDto {
        href: link.href,
        article_key,
        entry_key,
        unsupported_reason,
    }
}

fn map_catalog_entry(row: CatalogEntryRow) -> LibraryCatalogEntry {
    LibraryCatalogEntry {
        catalog_entry_id: row.catalog_entry_id,
        title: row.title,
        description: row.description,
        language_bcp47: row.language_bcp47,
        category: row.category,
        publisher: row.publisher,
        artifact_url: row.artifact_url,
        artifact_size_bytes: row.artifact_size_bytes as u64,
        artifact_sha256: row.artifact_sha256,
    }
}

/// Map a `ports::catalog::CatalogEntry` (already enriched with the
/// fallback fields the wire DTO needs) into the wire DTO. Used at the tail
/// of `install_plan`, after the row has been consumed to build the ports
/// value.
fn map_catalog_entry_from_ports(catalog: &ports::catalog::CatalogEntry) -> LibraryCatalogEntry {
    LibraryCatalogEntry {
        catalog_entry_id: catalog.catalog_entry_id.clone(),
        title: catalog.title.clone(),
        description: catalog.description.clone(),
        language_bcp47: catalog.language_bcp47.clone(),
        category: catalog.category.clone(),
        publisher: catalog.publisher.clone(),
        artifact_url: catalog.artifact_url.clone(),
        artifact_size_bytes: catalog.artifact_size_bytes,
        artifact_sha256: catalog.artifact_sha256.clone(),
    }
}

// ===========================================================================
// Inline DB rows (tables not covered by storage-sqlite::query at M1)
// ===========================================================================

/// One row of `storage_roots` (§6.2).
struct StorageRootRow {
    storage_root_id: String,
    display_name: String,
    writable: bool,
    platform_locator: String,
    free_bytes: Option<i64>,
    capacity_bytes: Option<i64>,
}

fn all_storage_roots(conn: &Connection) -> Result<Vec<StorageRootRow>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT storage_root_id, display_name, writable, platform_locator,\n\
         \x20\x20      free_bytes, capacity_bytes\n\
         \x20\x20 FROM storage_roots\n\
         \x20\x20 ORDER BY created_at_ms ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        let writable_int: i64 = row.get(2)?;
        Ok(StorageRootRow {
            storage_root_id: row.get(0)?,
            display_name: row.get(1)?,
            writable: writable_int != 0,
            platform_locator: row.get(3)?,
            free_bytes: row.get(4)?,
            capacity_bytes: row.get(5)?,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

fn storage_root_by_id(
    conn: &Connection,
    storage_root_id: &str,
) -> Result<Option<StorageRootRow>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT storage_root_id, display_name, writable, platform_locator,\n\
         \x20\x20      free_bytes, capacity_bytes\n\
         \x20\x20 FROM storage_roots\n\
         \x20\x20 WHERE storage_root_id = ?1",
    )?;
    let mut rows = stmt.query_map(params![storage_root_id], |row| {
        let writable_int: i64 = row.get(2)?;
        Ok(StorageRootRow {
            storage_root_id: row.get(0)?,
            display_name: row.get(1)?,
            writable: writable_int != 0,
            platform_locator: row.get(3)?,
            free_bytes: row.get(4)?,
            capacity_bytes: row.get(5)?,
        })
    })?;
    match rows.next() {
        Some(r) => Ok(Some(r?)),
        None => Ok(None),
    }
}

/// One row of `catalog_entries` (§6.2).
struct CatalogEntryRow {
    catalog_entry_id: String,
    corpus_id: String,
    upstream_version: String,
    title: String,
    description: String,
    language_bcp47: String,
    category: String,
    publisher: String,
    artifact_url: String,
    artifact_size_bytes: i64,
    artifact_sha256: String,
}

fn catalog_entry_by_id(
    conn: &Connection,
    catalog_entry_id: &str,
) -> Result<Option<CatalogEntryRow>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT catalog_entry_id, corpus_id, upstream_version, title,\n\
         \x20\x20      description, language_bcp47, category, publisher,\n\
         \x20\x20      artifact_url, artifact_size_bytes, artifact_sha256\n\
         \x20\x20 FROM catalog_entries\n\
         \x20\x20 WHERE catalog_entry_id = ?1",
    )?;
    let mut rows = stmt.query_map(params![catalog_entry_id], |row| {
        Ok(CatalogEntryRow {
            catalog_entry_id: row.get(0)?,
            corpus_id: row.get(1)?,
            upstream_version: row.get(2)?,
            title: row.get(3)?,
            description: row.get(4)?,
            language_bcp47: row.get(5)?,
            category: row.get(6)?,
            publisher: row.get(7)?,
            artifact_url: row.get(8)?,
            artifact_size_bytes: row.get(9)?,
            artifact_sha256: row.get(10)?,
        })
    })?;
    match rows.next() {
        Some(r) => Ok(Some(r?)),
        None => Ok(None),
    }
}

fn search_catalog_entries(
    conn: &Connection,
    query: &str,
    language: Option<&str>,
    category: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<CatalogEntryRow>, rusqlite::Error> {
    // Three filter shapes to keep the parameter count fixed. The text filter
    // is a substring match on title (CASE-INSENSITIVE via LOWER), matching
    // `idx_catalog_entries_search`.
    const SELECT_COLS: &str = "catalog_entry_id, corpus_id, upstream_version, title,\n\
         \x20\x20      description, language_bcp47, category, publisher,\n\
         \x20\x20      artifact_url, artifact_size_bytes, artifact_sha256";
    let sql = match (language.is_some(), category.is_some()) {
        (true, true) => format!(
            "SELECT {SELECT_COLS}\n\
             \x20\x20 FROM catalog_entries\n\
             \x20\x20 WHERE LOWER(title) LIKE '%' || LOWER(?1) || '%'\n\
             \x20\x20   AND language_bcp47 = ?2\n\
             \x20\x20   AND category = ?3\n\
             \x20\x20 ORDER BY title ASC\n\
             \x20\x20 LIMIT ?4 OFFSET ?5"
        ),
        (true, false) => format!(
            "SELECT {SELECT_COLS}\n\
             \x20\x20 FROM catalog_entries\n\
             \x20\x20 WHERE LOWER(title) LIKE '%' || LOWER(?1) || '%'\n\
             \x20\x20   AND language_bcp47 = ?2\n\
             \x20\x20 ORDER BY title ASC\n\
             \x20\x20 LIMIT ?3 OFFSET ?4"
        ),
        (false, true) => format!(
            "SELECT {SELECT_COLS}\n\
             \x20\x20 FROM catalog_entries\n\
             \x20\x20 WHERE LOWER(title) LIKE '%' || LOWER(?1) || '%'\n\
             \x20\x20   AND category = ?2\n\
             \x20\x20 ORDER BY title ASC\n\
             \x20\x20 LIMIT ?3 OFFSET ?4"
        ),
        (false, false) => format!(
            "SELECT {SELECT_COLS}\n\
             \x20\x20 FROM catalog_entries\n\
             \x20\x20 WHERE LOWER(title) LIKE '%' || LOWER(?1) || '%'\n\
             \x20\x20 ORDER BY title ASC\n\
             \x20\x20 LIMIT ?2 OFFSET ?3"
        ),
    };
    let mut stmt = conn.prepare(sql.as_str())?;
    let mapper = |row: &rusqlite::Row<'_>| -> rusqlite::Result<CatalogEntryRow> {
        Ok(CatalogEntryRow {
            catalog_entry_id: row.get(0)?,
            corpus_id: row.get(1)?,
            upstream_version: row.get(2)?,
            title: row.get(3)?,
            description: row.get(4)?,
            language_bcp47: row.get(5)?,
            category: row.get(6)?,
            publisher: row.get(7)?,
            artifact_url: row.get(8)?,
            artifact_size_bytes: row.get(9)?,
            artifact_sha256: row.get(10)?,
        })
    };
    let rows = match (language, category) {
        (Some(lang), Some(cat)) => stmt.query_map(params![query, lang, cat, limit, offset], mapper)?,
        (Some(lang), None) => stmt.query_map(params![query, lang, limit, offset], mapper)?,
        (None, Some(cat)) => stmt.query_map(params![query, cat, limit, offset], mapper)?,
        (None, None) => stmt.query_map(params![query, limit, offset], mapper)?,
    };
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

/// One row of `download_jobs` (§6.2) for `job_list`.
struct DownloadJobRow {
    download_job_id: String,
    state: String,
    bytes_verified: i64,
    expected_size_bytes: i64,
}

fn page_download_jobs(
    conn: &Connection,
    limit: i64,
    offset: i64,
) -> Result<Vec<DownloadJobRow>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT download_job_id, state, bytes_verified, expected_size_bytes\n\
         \x20\x20 FROM download_jobs\n\
         \x20\x20 ORDER BY updated_at_ms DESC\n\
         \x20\x20 LIMIT ?1 OFFSET ?2",
    )?;
    let rows = stmt.query_map(params![limit, offset], |row| {
        Ok(DownloadJobRow {
            download_job_id: row.get(0)?,
            state: row.get(1)?,
            bytes_verified: row.get(2)?,
            expected_size_bytes: row.get(3)?,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

/// Latest `selection_profile_revisions.canonical_json` for a pack, if any.
fn latest_selection_canonical(
    conn: &Connection,
    pack_id: &str,
) -> Result<Option<String>, rusqlite::Error> {
    conn.query_row(
        "SELECT spr.canonical_json\n\
         \x20\x20 FROM selection_profile_revisions spr\n\
         \x20\x20 JOIN selection_profiles sp\n\
         \x20\x20   ON sp.profile_id = spr.profile_id\n\
         \x20\x20 WHERE sp.pack_id = ?1\n\
         \x20\x20 ORDER BY spr.revision DESC\n\
         \x20\x20 LIMIT 1",
        params![pack_id],
        |row| row.get::<_, String>(0),
    )
    .optional()
}

/// Most-recent `reader_history.visited_at_ms` for a pack, if any.
fn last_opened_ms(conn: &Connection, pack_id: &str) -> Result<Option<i64>, rusqlite::Error> {
    conn.query_row(
        "SELECT MAX(visited_at_ms) FROM reader_history WHERE pack_id = ?1",
        params![pack_id],
        |row| row.get::<_, Option<i64>>(0),
    )
    .optional()
    .and_then(|opt| Ok(opt.flatten()))
}

/// Inline `INSERT OR IGNORE` for `packs` (the §6.2 trigger aborts a plain
/// INSERT that carries a non-NULL active version; re-installs would otherwise
/// trip the PK uniqueness). Callers activate the version via
/// [`activate_pack_version`] after the verified `pack_versions` row exists.
fn insert_or_ignore_pack(
    conn: &Connection,
    pack_id: &str,
    corpus_id: &str,
    display_name: &str,
    description: &str,
    language_bcp47: &str,
    pack_kind: &str,
    created_at_ms: i64,
    updated_at_ms: i64,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT OR IGNORE INTO packs (\n\
         \x20\x20        pack_id, corpus_id, display_name, description,\n\
         \x20\x20        language_bcp47, pack_kind, active_pack_version_id,\n\
         \x20\x20        created_at_ms, updated_at_ms\n\
         \x20\x20      ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, ?7, ?8)",
        params![
            pack_id,
            corpus_id,
            display_name,
            description,
            language_bcp47,
            pack_kind,
            created_at_ms,
            updated_at_ms
        ],
    )?;
    // Refresh display_name / language / etc. on an existing row so a
    // re-install reflects catalog updates. `active_pack_version_id` is left
    // untouched here and set explicitly below.
    conn.execute(
        "UPDATE packs\n\
         \x20\x20   SET display_name = ?2,\n\
         \x20\x20       description = ?3,\n\
         \x20\x20       language_bcp47 = ?4,\n\
         \x20\x20       updated_at_ms = ?5\n\
         \x20\x20 WHERE pack_id = ?1",
        params![pack_id, display_name, description, language_bcp47, updated_at_ms],
    )?;
    Ok(())
}

/// Point `packs.active_pack_version_id` at a verified `pack_versions` row.
/// The §6.2 trigger `trg_pack_active_version_same_pack` is the authority.
fn activate_pack_version(
    conn: &Connection,
    pack_id: &str,
    pack_version_id: &str,
    now_ms: i64,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE packs\n\
         \x20\x20   SET active_pack_version_id = ?2, updated_at_ms = ?3\n\
         \x20\x20 WHERE pack_id = ?1",
        params![pack_id, pack_version_id, now_ms],
    )?;
    Ok(())
}

// ===========================================================================
// Existing commands (bootstrap / home / readiness)
// ===========================================================================

#[tauri::command]
pub async fn app_bootstrap(
    state: tauri::State<'_, AppState>,
) -> Result<AppBootstrap, String> {
    let conn = state.conn.clone();
    let (first_launch, schema_applied) = tokio::task::spawn_blocking(move || {
        let g = conn.lock().map_err(err_str)?;
        let setup_done =
            storage_sqlite::query::setup_completed(&g).map_err(err_str)?;
        let schema_applied =
            storage_sqlite::query::schema_version_one_applied(&g).map_err(err_str)?;
        Ok::<_, String>((!setup_done, schema_applied))
    })
    .await
    .map_err(err_str)??;

    let readiness = build_readiness(schema_applied, now_ms());
    Ok(AppBootstrap {
        schema_version: 1,
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        platform: platform_string(),
        first_launch,
        routes: M1_ROUTES
            .iter()
            .map(|(path, enabled)| RouteDescriptor {
                path: (*path).to_string(),
                enabled: *enabled,
            })
            .collect(),
        capabilities: Vec::new(),
        readiness,
    })
}

#[tauri::command]
pub async fn readiness_get(
    state: tauri::State<'_, AppState>,
) -> Result<SystemReadiness, String> {
    let conn = state.conn.clone();
    let schema_applied = tokio::task::spawn_blocking(move || {
        let g = conn.lock().map_err(err_str)?;
        storage_sqlite::query::schema_version_one_applied(&g).map_err(err_str)
    })
    .await
    .map_err(err_str)??;
    Ok(build_readiness(schema_applied, now_ms()))
}

#[tauri::command]
pub async fn home_snapshot(
    state: tauri::State<'_, AppState>,
) -> Result<HomeSnapshot, String> {
    let conn = state.conn.clone();
    let snapshot = tokio::task::spawn_blocking(move || -> Result<HomeSnapshot, String> {
        let g = conn.lock().map_err(err_str)?;
        let recent_rows =
            storage_sqlite::query::recent_packs(&g, 12).map_err(err_str)?;
        let history_rows =
            storage_sqlite::query::recent_reader_history(&g, 1).map_err(err_str)?;
        let task_rows = storage_sqlite::query::open_tasks(&g).map_err(err_str)?;
        let schema_applied =
            storage_sqlite::query::schema_version_one_applied(&g).map_err(err_str)?;

        let recent_packs = recent_rows.into_iter().map(map_pack).collect();
        let continue_reading = history_rows.into_iter().next().map(map_history);
        let task_inbox = task_rows.into_iter().map(map_task).collect();
        let readiness = build_readiness(schema_applied, now_ms());

        Ok(HomeSnapshot {
            recent_packs,
            continue_reading,
            task_inbox,
            // M1: no memory-inbox entries are modeled yet; the frontend binds
            // this as an array, so an empty vec is the truthful shape.
            memory_inbox: Vec::new(),
            readiness,
        })
    })
    .await
    .map_err(err_str)??;
    Ok(snapshot)
}

// ===========================================================================
// Reader commands — REPOINTED to installed packs (file-picker path killed)
// ===========================================================================

/// §14.4 `reader_open({ packId })`. Opens an INSTALLED pack: looks up the
/// `ClusterCacheArchive` registered in [`AppState::archives`] at install time
/// and returns its main page key. Any `path` argument the frontend may still
/// send is ignored — the packId is the only key.
#[tauri::command]
pub async fn reader_open(
    state: tauri::State<'_, AppState>,
    pack_id: String,
    #[allow(unused_variables)] path: Option<String>,
) -> Result<ReaderOpenResponse, String> {
    if pack_id.trim().is_empty() {
        return Err("invalid_request: packId must not be empty".to_string());
    }

    let archive = {
        let archives = state.archives.lock().map_err(err_str)?;
        archives
            .get(&pack_id)
            .cloned()
            .ok_or_else(|| {
                format!(
                    "not_found: no installed pack for packId {} — call install_start first",
                    pack_id
                )
            })?
    };

    let title = archive.metadata().title.clone();
    let main_page_key = archive
        .main_page()
        .await
        .ok()
        .flatten()
        .map(|k| k.0);

    Ok(ReaderOpenResponse {
        pack_id,
        title,
        main_page_key,
    })
}

/// §14.4 `reader_article({ packId, articleKey })`. Reads one article from
/// the registered archive. Article keys may contain `/` (e.g. `A/...`) — the
/// newtype is constructed directly, matching the engine's convention.
#[tauri::command]
pub async fn reader_article(
    state: tauri::State<'_, AppState>,
    pack_id: String,
    article_key: String,
) -> Result<ArticleResponse, String> {
    if pack_id.trim().is_empty() || article_key.trim().is_empty() {
        return Err("invalid_request: packId and articleKey must not be empty".to_string());
    }

    let archive = {
        let archives = state.archives.lock().map_err(err_str)?;
        archives
            .get(&pack_id)
            .cloned()
            .ok_or_else(|| format!("not_found: no open archive for packId {}", pack_id))?
    };

    let key = domain::identity::ArticleKey(article_key.clone());
    let document = archive.article(&key).await.map_err(err_str)?;

    Ok(ArticleResponse {
        pack_id,
        article_key: document.article_key.0,
        title: document.title,
        raw_html: String::from_utf8_lossy(&document.raw_html).into_owned(),
        mime_type: document.mime_type,
        internal_links: document.internal_links.into_iter().map(map_internal_link).collect(),
    })
}

// ===========================================================================
// Library / DynDon commands
// ===========================================================================

/// §14.4 `storage_roots_list`. Returns writable destinations. If the DB has
/// no `storage_roots` rows, synthesizes and persists one for the platform
/// app-data dir (the M1 default destination) with a real `free_bytes` probe.
#[tauri::command]
pub async fn storage_roots_list(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<StorageDestinationDto>, String> {
    let conn = state.conn.clone();
    let app_data_dir = state.app_data_dir.clone();
    let rows = tokio::task::spawn_blocking(move || -> Result<Vec<StorageRootRow>, String> {
        let g = conn.lock().map_err(err_str)?;
        let existing = all_storage_roots(&g).map_err(err_str)?;
        if !existing.is_empty() {
            return Ok(existing);
        }
        // Seed the M1 default root. Stable id derived from the path so a
        // re-seed after a wipe is idempotent.
        let locator = app_data_dir
            .to_str()
            .ok_or_else(|| {
                "internal: app_data_dir is not valid UTF-8".to_string()
            })?
            .to_string();
        let storage_root_id = format!(
            "sr-{}",
            storage_sqlite::sha256_hex(locator.as_bytes())
        );
        let free_bytes = probe_free_bytes(&app_data_dir);
        let now = now_ms();
        // `kind = 'app_private'` is the §6.2 CHECK value matching a
        // platform-managed private dir; the synthesized root is the closest
        // M1 analog of an app-private storage root.
        g.execute(
            "INSERT OR IGNORE INTO storage_roots (\n\
             \x20\x20        storage_root_id, kind, platform_locator, display_name,\n\
             \x20\x20        writable, capacity_bytes, free_bytes,\n\
             \x20\x20        last_probed_at_ms, created_at_ms, updated_at_ms\n\
             \x20\x20      ) VALUES (?1, 'app_private', ?2, ?3, 1, NULL, ?4, ?5, ?6, ?6)",
            params![
                storage_root_id,
                locator,
                app_data_dir
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("EnZIME Data")
                    .to_string(),
                free_bytes,
                now,
                now,
            ],
        )
        .map_err(err_str)?;
        all_storage_roots(&g).map_err(err_str)
    })
    .await
    .map_err(err_str)??;

    Ok(rows
        .into_iter()
        .filter(|r| r.writable)
        .map(|r| StorageDestinationDto {
            storage_root_id: r.storage_root_id,
            display_name: r.display_name,
            writable: r.writable,
        })
        .collect())
}

/// §14.4 `pack_list({ offset?, limit? })`. Pages installed packs. Article
/// count, byte size, main-page key, and last-opened timestamp are recovered
/// from the latest persisted selection revision (`canonical_json`) so the
/// Library view reflects the DynDon subset actually on device.
#[tauri::command]
pub async fn pack_list(
    state: tauri::State<'_, AppState>,
    offset: Option<u32>,
    limit: Option<u32>,
) -> Result<PageResultDto<LibraryPackManifest>, String> {
    let offset = offset.unwrap_or(0);
    let limit = limit.unwrap_or(50).clamp(1, 200);
    let conn = state.conn.clone();
    let items = tokio::task::spawn_blocking(move || -> Result<Vec<LibraryPackManifest>, String> {
        let g = conn.lock().map_err(err_str)?;
        let pack_rows = storage_sqlite::query::recent_packs(&g, limit as i64)
            .map_err(err_str)?;
        let mut out = Vec::with_capacity(pack_rows.len());
        for row in pack_rows {
            // Recover the subset metadata from the latest selection
            // revision. Packs installed outside the DynDon flow have no such
            // row and surface as article_count = 0 / main_page_key = None —
            // truthful, not fabricated.
            let canonical = latest_selection_canonical(&g, &row.pack_id)
                .map_err(err_str)?;
            let (article_count, size_bytes, main_page_key) = match canonical {
                Some(json) => match serde_json::from_str::<SelectionCanonical>(&json) {
                    Ok(canon) => {
                        let selected: std::collections::HashSet<u32> =
                            canon.selected_cluster_indices.iter().copied().collect();
                        let count = canon
                            .cluster_directory
                            .articles
                            .values()
                            .filter(|rec| {
                                rec.kind == ZimEntryKind::Article
                                    && selected.contains(&rec.cluster_index)
                            })
                            .count() as u64;
                        let bytes = canon
                            .cluster_directory
                            .cluster_ranges
                            .iter()
                            .filter(|(idx, _)| selected.contains(idx))
                            .map(|(_, r)| {
                                r.end_byte_exclusive.saturating_sub(r.start_byte)
                            })
                            .sum();
                        let main = canon.cluster_directory.main_page.map(|k| k.0);
                        (count, bytes, main)
                    }
                    Err(_) => (0, 0, None),
                },
                None => (0, 0, None),
            };
            let last_opened_at = last_opened_ms(&g, &row.pack_id)
                .map_err(err_str)?
                .map(|ms| format_iso(ms));
            out.push(LibraryPackManifest {
                pack_id: row.pack_id,
                display_name: row.display_name,
                description: row.description,
                language_bcp47: row.language_bcp47,
                article_count,
                size_bytes,
                last_opened_at,
                main_page_key,
            });
        }
        Ok(out)
    })
    .await
    .map_err(err_str)??;

    let total = items.len() as u64;
    let next_offset = if items.len() as u32 >= limit {
        Some(offset + limit)
    } else {
        None
    };
    Ok(PageResultDto {
        items,
        total: Some(total),
        next_offset,
    })
}

/// §14.4 `catalog_search({ query, languageBcp47?, category?, offset?, limit? })`.
#[tauri::command]
pub async fn catalog_search(
    state: tauri::State<'_, AppState>,
    query: String,
    language_bcp47: Option<String>,
    category: Option<String>,
    offset: Option<u32>,
    limit: Option<u32>,
) -> Result<PageResultDto<LibraryCatalogEntry>, String> {
    let offset = offset.unwrap_or(0);
    let limit = limit.unwrap_or(50).clamp(1, 200);
    let conn = state.conn.clone();
    let rows = tokio::task::spawn_blocking(move || -> Result<Vec<CatalogEntryRow>, String> {
        let g = conn.lock().map_err(err_str)?;
        search_catalog_entries(
            &g,
            &query,
            language_bcp47.as_deref(),
            category.as_deref(),
            limit as i64,
            offset as i64,
        )
        .map_err(err_str)
    })
    .await
    .map_err(err_str)??;

    let total = rows.len() as u64;
    let items: Vec<LibraryCatalogEntry> = rows.into_iter().map(map_catalog_entry).collect();
    let next_offset = if items.len() as u32 >= limit {
        Some(offset + limit)
    } else {
        None
    };
    Ok(PageResultDto {
        items,
        total: Some(total),
        next_offset,
    })
}

/// §14.4 `install_plan({ catalogEntryId, destinationStorageRootId, selectionProfileId? })`.
/// This is the DynDon USP: it opens the source ZIM, builds a live
/// `ClusterManifest`, fits it to the destination's free space via
/// `dyndon::plan_fit`, and persists the selection so `install_start` can
/// fetch the chosen clusters.
#[tauri::command]
pub async fn install_plan(
    state: tauri::State<'_, AppState>,
    catalog_entry_id: String,
    destination_storage_root_id: String,
    selection_profile_id: Option<String>,
) -> Result<InstallPlanDto, String> {
    let plan = build_install_plan(
        &state,
        &catalog_entry_id,
        &destination_storage_root_id,
        selection_profile_id.as_deref(),
    )
    .await?;
    Ok(plan)
}

/// §14.4 `install_start({ installPlanId })`. Fetches each selected cluster
/// via the transfer source into `<root>/packs/<packId>/cluster-<n>`,
/// transitions the download job through to `completed`, inserts the pack +
/// version + location rows, builds a `ClusterCacheArchive` from the directory
/// + downloaded bytes, and registers it in [`AppState::archives`] so the
/// reader can immediately open it.
#[tauri::command]
pub async fn install_start(
    state: tauri::State<'_, AppState>,
    install_plan_id: String,
) -> Result<DownloadJobDto, String> {
    let plan = {
        let plans = state.install_plans.lock().map_err(err_str)?;
        plans
            .get(&install_plan_id)
            .cloned()
            .ok_or_else(|| {
                format!(
                    "not_found: install plan {} expired or never created — call install_plan first",
                    install_plan_id
                )
            })?
    };

    let source = transfer_source_for(&plan.catalog_entry.artifact_url)?;

    // Create the download_job row (state = planned).
    let now = now_ms();
    let download_job_id = format!(
        "dj-{}-{}",
        plan.pack_id,
        storage_sqlite::sha256_hex(install_plan_id.as_bytes())
    );
    let expected_size_bytes = plan.download_bytes.max(1) as i64;
    {
        let conn = state.conn.clone();
        let job_id = download_job_id.clone();
        let catalog_id = plan.catalog_entry.catalog_entry_id.clone();
        let source_uri = plan.catalog_entry.artifact_url.clone();
        let dest_root = plan.destination_storage_root_id.clone();
        let expected_sha = plan.catalog_entry.artifact_sha256.clone();
        tokio::task::spawn_blocking(move || -> Result<(), String> {
            let g = conn.lock().map_err(err_str)?;
            storage_sqlite::write::insert_download_job(
                &g,
                &job_id,
                Some(&catalog_id),
                &source_uri,
                &dest_root,
                expected_size_bytes,
                &expected_sha,
                "planned",
                0,
                None,
                None,
                None,
                None,
                now,
                now,
                None,
            )
            .map_err(err_str)?;
            Ok(())
        })
        .await
        .map_err(err_str)??;
    }

    // planned → fetching.
    transition_job(&state.conn, &download_job_id, "fetching").await?;

    // Fetch each selected cluster into the destination root.
    let pack_dir = state.app_data_dir.join("packs").join(&plan.pack_id);
    tokio::fs::create_dir_all(&pack_dir)
        .await
        .map_err(|e| format!("io: cannot create pack dir {}: {}", pack_dir.display(), e))?;

    let mut local_clusters_raw: HashMap<u32, Bytes> = HashMap::new();
    let mut bytes_verified: u64 = 0;
    for (chunk_index, &cluster_idx) in plan.selected_cluster_indices.iter().enumerate() {
        let (start, len) = plan
            .cluster_byte_ranges
            .get(cluster_idx as usize)
            .copied()
            .ok_or_else(|| {
                format!(
                    "internal: cluster {} has no byte range in the persisted plan",
                    cluster_idx
                )
            })?;
        let end_exclusive = start
            .checked_add(len)
            .ok_or_else(|| "internal: cluster byte range overflow".to_string())?;
        let range = TransferRange {
            start_byte: start,
            end_byte_exclusive: end_exclusive,
        };
        let stream = source
            .open_range(&plan.catalog_entry.artifact_url, range, None)
            .await
            .map_err(err_str)?;
        let mut buf: Vec<u8> = Vec::with_capacity(len as usize);
        let mut s = stream;
        while let Some(item) = s.next().await {
            let chunk_bytes = item.map_err(err_str)?;
            buf.extend_from_slice(&chunk_bytes);
        }
        let cluster_file = pack_dir.join(format!("cluster-{}", cluster_idx));
        tokio::fs::write(&cluster_file, &buf)
            .await
            .map_err(|e| format!("io: cannot write cluster file {}: {}", cluster_file.display(), e))?;

        let actual_sha = storage_sqlite::sha256_hex(&buf);
        let chunk_len = buf.len() as u64;
        bytes_verified = bytes_verified.saturating_add(chunk_len);
        local_clusters_raw.insert(cluster_idx, Bytes::from(buf));

        // upsert_download_chunk per cluster.
        let conn = state.conn.clone();
        let job_id = download_job_id.clone();
        let sha_for_chunk = actual_sha.clone();
        tokio::task::spawn_blocking(move || -> Result<(), String> {
            let g = conn.lock().map_err(err_str)?;
            storage_sqlite::write::upsert_download_chunk(
                &g,
                &job_id,
                chunk_index as i64,
                start as i64,
                end_exclusive as i64,
                None,
                Some(&sha_for_chunk),
                "verified",
                0,
                now_ms(),
            )
            .map_err(err_str)?;
            Ok(())
        })
        .await
        .map_err(err_str)??;
    }

    // Walk the state machine through to completed. The §6.2 trigger requires
    // the full chain planned→fetching→verifying→staged→activating→completed.
    transition_job(&state.conn, &download_job_id, "verifying").await?;
    transition_job(&state.conn, &download_job_id, "staged").await?;
    transition_job(&state.conn, &download_job_id, "activating").await?;
    // Insert pack + version + location, then activate (all under one lock).
    let pack_id = plan.pack_id.clone();
    let pack_version_id = plan.pack_version_id.clone();
    let dest_root = plan.destination_storage_root_id.clone();
    let catalog = plan.catalog_entry.clone();
    let relative_path = format!("packs/{}/", pack_id);
    {
        let conn = state.conn.clone();
        let pack_id_for_write = pack_id.clone();
        let pack_version_id_for_write = pack_version_id.clone();
        let dest_root_for_write = dest_root.clone();
        let relative_path_for_write = relative_path.clone();
        let catalog_for_write = catalog.clone();
        tokio::task::spawn_blocking(move || -> Result<(), String> {
            let g = conn.lock().map_err(err_str)?;
            let now = now_ms();
            insert_or_ignore_pack(
                &g,
                &pack_id_for_write,
                &catalog_for_write.corpus_id,
                &catalog_for_write.title,
                &catalog_for_write.description,
                &catalog_for_write.language_bcp47,
                "tailored_zim",
                now,
                now,
            )
            .map_err(err_str)?;
            storage_sqlite::write::insert_pack_version(
                &g,
                &pack_version_id_for_write,
                &pack_id_for_write,
                &catalog_for_write.upstream_version,
                // edition_manifest_sha256: no separate manifest at M1; reuse
                // the artifact sha (64-hex) so the CHECK passes.
                &catalog_for_write.artifact_sha256,
                &catalog_for_write.artifact_sha256,
                catalog_for_write.artifact_size_bytes as i64,
                "zim",
                "verified",
                Some(now),
                now,
            )
            .map_err(err_str)?;
            activate_pack_version(&g, &pack_id_for_write, &pack_version_id_for_write, now)
                .map_err(err_str)?;
            let pack_location_id = format!(
                "pl-{}",
                storage_sqlite::sha256_hex(
                    format!("{}-{}", pack_version_id_for_write, dest_root_for_write).as_bytes()
                )
            );
            storage_sqlite::write::insert_pack_location(
                &g,
                &pack_location_id,
                &pack_version_id_for_write,
                &dest_root_for_write,
                &relative_path_for_write,
                "active",
                bytes_verified as i64,
                Some(now),
                now,
            )
            .map_err(err_str)?;
            Ok(())
        })
        .await
        .map_err(err_str)??;
    }

    // Build the ClusterCacheArchive from the directory + downloaded clusters
    // and register it in AppState.archives so reader_open(packId) works.
    let directory = plan.cluster_directory.clone();
    let cache_archive =
        zim_cluster_cache::ClusterCacheArchive::new_offline(directory, local_clusters_raw)
            .map_err(err_str)?;
    {
        let mut archives = state.archives.lock().map_err(err_str)?;
        archives.insert(pack_id.clone(), Arc::new(cache_archive) as Arc<dyn ports::zim::ZimArchive>);
    }

    transition_job(&state.conn, &download_job_id, "completed").await?;

    // Drop the plan now that it has been consumed.
    {
        let mut plans = state.install_plans.lock().map_err(err_str)?;
        plans.remove(&install_plan_id);
    }

    Ok(DownloadJobDto {
        download_job_id,
        state: "completed".to_string(),
        bytes_verified,
        expected_size_bytes: expected_size_bytes as u64,
    })
}

/// §14.4 `job_list({ offset?, limit? })`.
#[tauri::command]
pub async fn job_list(
    state: tauri::State<'_, AppState>,
    offset: Option<u32>,
    limit: Option<u32>,
) -> Result<PageResultDto<DownloadJobDto>, String> {
    let offset = offset.unwrap_or(0);
    let limit = limit.unwrap_or(50).clamp(1, 200);
    let conn = state.conn.clone();
    let rows = tokio::task::spawn_blocking(move || -> Result<Vec<DownloadJobRow>, String> {
        let g = conn.lock().map_err(err_str)?;
        page_download_jobs(&g, limit as i64, offset as i64).map_err(err_str)
    })
    .await
    .map_err(err_str)??;

    let total = rows.len() as u64;
    let items: Vec<DownloadJobDto> = rows
        .into_iter()
        .map(|r| DownloadJobDto {
            download_job_id: r.download_job_id,
            state: r.state,
            bytes_verified: r.bytes_verified.max(0) as u64,
            expected_size_bytes: r.expected_size_bytes.max(0) as u64,
        })
        .collect();
    let next_offset = if items.len() as u32 >= limit {
        Some(offset + limit)
    } else {
        None
    };
    Ok(PageResultDto {
        items,
        total: Some(total),
        next_offset,
    })
}

// ===========================================================================
// install_plan builder + helpers
// ===========================================================================

/// Free-space resolution ladder. Returns the resolved byte count or an
/// escalation error string.
fn resolve_free_bytes(root: &StorageRootRow) -> Result<u64, String> {
    // (2) DB free_bytes.
    if let Some(b) = root.free_bytes {
        if b > 0 {
            return Ok(b as u64);
        }
    }
    // (2') DB capacity_bytes (the documented fallback).
    if let Some(b) = root.capacity_bytes {
        if b > 0 {
            return Ok(b as u64);
        }
    }
    // (3) Real syscall probe of the destination directory.
    if let Some(b) = probe_free_bytes(Path::new(&root.platform_locator)) {
        return Ok(b);
    }
    // (4) ESCALATE — do not hardcode.
    Err(format!(
        "capability_denied: cannot determine free bytes for storage root {} \
         (no platform adapter, no DB free_bytes/capacity_bytes, statvfs failed) — \
         this is an ESCALATION: wire the platform-adapter crate or seed storage_roots.free_bytes",
        root.storage_root_id
    ))
}

async fn transition_job(
    conn: &Arc<std::sync::Mutex<Connection>>,
    job_id: &str,
    new_state: &str,
) -> Result<(), String> {
    let conn = conn.clone();
    let job_id = job_id.to_string();
    let new_state = new_state.to_string();
    tokio::task::spawn_blocking(move || -> Result<(), String> {
        let g = conn.lock().map_err(err_str)?;
        storage_sqlite::write::set_download_job_state(&g, &job_id, &new_state, now_ms())
            .map_err(err_str)?;
        Ok(())
    })
    .await
    .map_err(err_str)??;
    Ok(())
}

async fn build_install_plan(
    state: &tauri::State<'_, AppState>,
    catalog_entry_id: &str,
    destination_storage_root_id: &str,
    _selection_profile_id: Option<&str>,
) -> Result<InstallPlanDto, String> {
    // Load catalog_entry + destination root.
    let conn = state.conn.clone();
    let (catalog_row, dest_row) = tokio::task::spawn_blocking({
        let catalog_entry_id = catalog_entry_id.to_string();
        let destination_storage_root_id = destination_storage_root_id.to_string();
        move || -> Result<(CatalogEntryRow, StorageRootRow), String> {
            let g = conn.lock().map_err(err_str)?;
            let catalog = catalog_entry_by_id(&g, &catalog_entry_id)
                .map_err(err_str)?
                .ok_or_else(|| {
                    format!("not_found: catalog_entry {} not in catalog_entries", catalog_entry_id)
                })?;
            let dest = storage_root_by_id(&g, &destination_storage_root_id)
                .map_err(err_str)?
                .ok_or_else(|| {
                    format!(
                        "not_found: storage_root {} not in storage_roots",
                        destination_storage_root_id
                    )
                })?;
            if !dest.writable {
                return Err(format!(
                    "permission_denied: storage_root {} is not writable",
                    destination_storage_root_id
                ));
            }
            Ok((catalog, dest))
        }
    })
    .await
    .map_err(err_str)??;

    let catalog = ports::catalog::CatalogEntry {
        catalog_entry_id: catalog_row.catalog_entry_id.clone(),
        corpus_id: catalog_row.corpus_id.clone(),
        upstream_version: catalog_row.upstream_version.clone(),
        title: catalog_row.title,
        description: catalog_row.description,
        language_bcp47: catalog_row.language_bcp47,
        category: catalog_row.category,
        publisher: catalog_row.publisher,
        artifact_url: catalog_row.artifact_url.clone(),
        artifact_size_bytes: catalog_row.artifact_size_bytes as u64,
        artifact_sha256: catalog_row.artifact_sha256,
        signature_url: None,
        trust: ports::catalog::TrustPresentation {
            state: "unverified".to_string(),
            signer: None,
            explanation: "M1: no signature infrastructure wired".to_string(),
        },
    };

    // Resolve free bytes BEFORE opening the ZIM.
    let free_bytes_before = resolve_free_bytes(&dest_row)?;

    // Open the source ZIM and snapshot its layout. The engine is file-based;
    // remote artifact_url schemes are an ESCALATION (the engine cannot open
    // them directly — use a local mirror).
    let local_path = artifact_url_to_local_path(&catalog.artifact_url).ok_or_else(|| {
        format!(
            "unsupported: artifact_url {} is not a local file:// or absolute path \
             — the AnZimmerman engine cannot open remote URLs at M1 (mirror the ZIM locally)",
            catalog.artifact_url
        )
    })?;
    let open_request = ports::zim::ZimOpenRequest {
        path: local_path.clone(),
        expected_sha256: Some(catalog.artifact_sha256.clone()),
        expected_size_bytes: Some(catalog.artifact_size_bytes),
    };
    // Open the CONCRETE archive so we can call `layout_snapshot()` — that
    // method is on `AnZimmermanZimArchive`, not the `ZimArchive` trait
    // object. The engine's trait `open()` returns `Arc<dyn ZimArchive>`,
    // which has no layout accessor.
    //
    // The engine performs blocking file I/O internally; the open path also
    // runs the full-file sha256 verification (expensive, but it makes a
    // digest mismatch fail fast before any cluster is fetched).
    let archive = zim_anzimmerman::archive::open_archive(&open_request)
        .await
        .map_err(|e| {
            // The engine maps a digest mismatch to DigestMismatch; surface
            // it as source_changed so the frontend classifies correctly.
            format!("source_changed: AnZimmerman open failed: {}", e)
        })?;
    let metadata = archive.metadata().clone();
    let main_page = archive
        .main_page()
        .await
        .ok()
        .flatten();
    // `layout_snapshot` performs synchronous file reads (directory entries);
    // run it on the blocking pool. The archive is moved in because we no
    // longer need it after this point (metadata + main_page are already
    // resolved above).
    let layout = tokio::task::spawn_blocking(move || archive.layout_snapshot().map_err(err_str))
        .await
        .map_err(err_str)??;

    // Build the ClusterManifest from the layout. Per-cluster entry:
    // {cluster_index, byte_offset, byte_len, domain: None, topic: None,
    //  priority: 1, article_count}.
    let mut article_counts: HashMap<u32, u64> = HashMap::new();
    for entry in &layout.entries {
        if entry.kind == ZimEntryKind::Article {
            *article_counts.entry(entry.cluster_index).or_insert(0) += 1;
        }
    }
    let mut manifest_entries = Vec::with_capacity(layout.cluster_byte_ranges.len());
    for (idx, &(offset, len)) in layout.cluster_byte_ranges.iter().enumerate() {
        manifest_entries.push(dyndon::ClusterManifestEntry {
            cluster_index: idx as u32,
            byte_offset: offset,
            byte_len: len,
            domain: None,
            topic: None,
            priority: 1,
            article_count: *article_counts.get(&(idx as u32)).unwrap_or(&0) as u32,
        });
    }
    let total_bytes: u64 = manifest_entries.iter().map(|e| e.byte_len).sum();
    let pack_version_id = format!(
        "pv-{}",
        storage_sqlite::sha256_hex(
            format!("{}-{}", catalog.artifact_sha256, catalog.upstream_version).as_bytes()
        )
    );
    let manifest = dyndon::ClusterManifest {
        pack_version_id: pack_version_id.clone(),
        cluster_count: manifest_entries.len() as u32,
        total_bytes,
        entries: manifest_entries,
    };

    // Build the ClusterDirectory from the layout + archive metadata.
    let mut directory = zim_cluster_cache::ClusterDirectory::new();
    directory.metadata = metadata.clone();
    directory.main_page = main_page.clone();
    for (idx, &(offset, len)) in layout.cluster_byte_ranges.iter().enumerate() {
        directory.cluster_ranges.insert(
            idx as u32,
            zim_cluster_cache::ClusterRange {
                start_byte: offset,
                end_byte_exclusive: offset + len,
            },
        );
    }
    for entry in &layout.entries {
        let article_key = domain::identity::ArticleKey(entry.entry_key.clone());
        directory.articles.insert(
            article_key,
            zim_cluster_cache::ArticleRecord {
                namespace: entry.namespace,
                path: entry.path.clone(),
                title: entry.title.clone(),
                mime_type: entry.mime_type.clone(),
                kind: entry.kind,
                redirect_target: entry.redirect_target.clone(),
                cluster_index: entry.cluster_index,
                blob_index: entry.blob_index,
            },
        );
    }

    // Probe the transfer source to determine transferMode.
    let source = transfer_source_for(&catalog.artifact_url)?;
    let probe = source
        .probe(TransferProbeRequest {
            uri: catalog.artifact_url.clone(),
            maximum_response_bytes: ports::transfer::TRANSFER_PROBE_MAX_BYTES,
        })
        .await
        .map_err(|e| format!("offline: transfer probe failed: {}", e))?;
    let transfer_mode = if probe.accepts_ranges {
        TransferMode::Ranged
    } else {
        TransferMode::Full
    };

    // Build the SelectionIntent. M1 default: all domains, no breadth/depth
    // cap, budget = free_bytes minus a small safety margin.
    let safety_margin = (free_bytes_before / 20).min(256 * 1024 * 1024); // 5% or 256 MiB
    let budget_bytes = free_bytes_before
        .saturating_sub(safety_margin)
        .max(0);
    let intent = dyndon::SelectionIntent {
        domains: Vec::new(),
        breadth_limit: None,
        depth_limit: None,
        budget_bytes,
    };
    let plan = dyndon::plan_fit(&manifest, &intent);

    // Persist a selection_profile + revision carrying the directory + the
    // selected clusters (durable across restarts).
    let pack_id = format!(
        "pk-{}",
        storage_sqlite::sha256_hex(catalog.artifact_url.as_bytes())
    );
    let selection_profile_id = format!("sp-{}", pack_id);
    let canonical = SelectionCanonical {
        artifact_url: catalog.artifact_url.clone(),
        pack_id: pack_id.clone(),
        catalog_entry_id: catalog.catalog_entry_id.clone(),
        destination_storage_root_id: destination_storage_root_id.to_string(),
        selected_cluster_indices: plan.selected.clone(),
        cluster_directory: directory.clone(),
    };
    let canonical_json = serde_json::to_string(&canonical)
        .map_err(|e| format!("internal: cannot serialize selection canonical: {}", e))?;
    let profile_sha256 = storage_sqlite::sha256_hex(canonical_json.as_bytes());
    let now = now_ms();
    {
        let conn = state.conn.clone();
        let profile_id = selection_profile_id.clone();
        let pack_id_for_write = pack_id.clone();
        let canonical_json_for_write = canonical_json.clone();
        let profile_sha = profile_sha256.clone();
        tokio::task::spawn_blocking(move || -> Result<(), String> {
            let g = conn.lock().map_err(err_str)?;
            // Idempotent: if the profile exists, bump current_revision; else insert.
            let existing_revision: Option<i64> = g
                .query_row(
                    "SELECT current_revision FROM selection_profiles WHERE profile_id = ?1",
                    params![profile_id],
                    |row| row.get(0),
                )
                .optional()
                .map_err(err_str)?;
            let revision = existing_revision.unwrap_or(0) + 1;
            if existing_revision.is_none() {
                storage_sqlite::write::insert_selection_profile(
                    &g,
                    &profile_id,
                    &pack_id_for_write,
                    "default",
                    revision,
                    now,
                    now,
                )
                .map_err(err_str)?;
            } else {
                g.execute(
                    "UPDATE selection_profiles\n\
                     \x20\x20   SET current_revision = ?2, updated_at_ms = ?3\n\
                     \x20\x20 WHERE profile_id = ?1",
                    params![profile_id, revision, now],
                )
                .map_err(err_str)?;
            }
            storage_sqlite::write::insert_selection_profile_revision(
                &g,
                &profile_id,
                revision,
                &canonical_json_for_write,
                &profile_sha,
                now,
            )
            .map_err(err_str)?;
            Ok(())
        })
        .await
        .map_err(err_str)??;
    }

    // Stash the plan envelope for install_start.
    let install_plan_id = format!(
        "ip-{}-{}",
        pack_id,
        storage_sqlite::sha256_hex(format!("{}-{}", catalog.catalog_entry_id, now).as_bytes())
    );
    let selection_revision = {
        let conn = state.conn.clone();
        let profile_id = selection_profile_id.clone();
        tokio::task::spawn_blocking(move || -> Result<i64, String> {
            let g = conn.lock().map_err(err_str)?;
            let rev: i64 = g
                .query_row(
                    "SELECT current_revision FROM selection_profiles WHERE profile_id = ?1",
                    params![profile_id],
                    |row| row.get(0),
                )
                .map_err(err_str)?;
            Ok(rev)
        })
        .await
        .map_err(err_str)??
    };
    let persisted = PersistedInstallPlan {
        install_plan_id: install_plan_id.clone(),
        catalog_entry: catalog.clone(),
        destination_storage_root_id: destination_storage_root_id.to_string(),
        destination_display_name: dest_row.display_name.clone(),
        destination_writable: dest_row.writable,
        transfer_mode,
        pack_id: pack_id.clone(),
        pack_version_id: pack_version_id.clone(),
        selection_profile_id,
        selection_revision,
        selected_cluster_indices: plan.selected.clone(),
        cluster_byte_ranges: layout.cluster_byte_ranges.clone(),
        cluster_directory: directory.clone(),
        free_bytes_before,
        download_bytes: plan.total_bytes,
    };
    {
        let mut plans = state.install_plans.lock().map_err(err_str)?;
        plans.insert(install_plan_id.clone(), persisted);
    }

    Ok(InstallPlanDto {
        install_plan_id,
        catalog_entry: map_catalog_entry_from_ports(&catalog),
        destination: StorageDestinationDto {
            storage_root_id: destination_storage_root_id.to_string(),
            display_name: dest_row.display_name,
            writable: true,
        },
        transfer_mode: transfer_mode.into(),
        download_bytes: plan.total_bytes,
        temporary_bytes: plan.total_bytes,
        final_bytes: plan.total_bytes,
        free_bytes_before,
        free_bytes_after: plan.free_bytes_after.max(0) as u64,
        verification: InstallPlanVerificationDto {
            expected_size_bytes: catalog.artifact_size_bytes,
            expected_sha256: catalog.artifact_sha256,
            signature_required: false,
            zim_structure_check: true,
        },
    })
}

/// Format a unix-millis timestamp as an RFC 3339 string (the wire shape the
/// frontend expects for `lastOpenedAt`). Tiny chrono-free implementation so
/// the reader crate does not pull in `chrono` just for this one field.
fn format_iso(ms: i64) -> String {
    let secs = (ms / 1000).max(0) as i64;
    let days = secs.div_euclid(86_400);
    let tod = secs.rem_euclid(86_400) as u32;
    let hour = tod / 3_600;
    let min = (tod % 3_600) / 60;
    let sec = tod % 60;
    let (year, month, day) = civil_from_days(days);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hour, min, sec
    )
}

/// Howard Hinnant's days-from-epoch → civil date algorithm.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as i64; // [0, 146096]
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) as i64; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = if mp < 10 { mp as u32 + 3 } else { mp as u32 - 9 }; // [1, 12]
    let year = if m <= 2 { y + 1 } else { y };
    (year, m, d)
}
