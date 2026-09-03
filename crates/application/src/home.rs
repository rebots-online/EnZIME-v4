//! Home snapshot use case (§14.4 `home_snapshot` → `HomeSnapshot`).
//!
//! Owns the §14.2 / §24 wire DTOs and assembles the snapshot from the row
//! representations built by the IPC layer. The application crate is not
//! permitted to depend on the concrete `storage-sqlite` provider (A-02), so
//! the row → DTO mapping happens at the IPC boundary; this module supplies the
//! DTO types and the final aggregation.

use serde::Serialize;

use crate::readiness::SystemReadiness;

/// §14.2 `HomeSnapshot`.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HomeSnapshot {
    pub recent_packs: Vec<PackManifest>,
    pub continue_reading: Option<ReaderHistoryItem>,
    pub open_tasks: Vec<UserTask>,
    pub memory_inbox_count: i64,
    pub readiness: SystemReadiness,
}

/// §14.2 `PackManifest`.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackManifest {
    pub pack_id: String,
    pub corpus_id: String,
    pub display_name: String,
    pub description: String,
    pub language_bcp47: String,
    /// `"full_zim"` | `"tailored_zim"` | `"enzime_pack"`.
    pub pack_kind: String,
    pub active_version_id: Option<String>,
    pub versions: Vec<PackVersionSummary>,
}

/// §14.2 `PackVersionSummary`.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackVersionSummary {
    pub pack_version_id: String,
    pub upstream_version: String,
    pub artifact_sha256: String,
    pub artifact_size_bytes: i64,
    /// `"unverified"` | `"verified"` | `"quarantined"`.
    pub verification_state: String,
}

/// §14.2 `ReaderHistoryItem`.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReaderHistoryItem {
    pub pack_id: String,
    pub article_key: String,
    pub title: String,
    pub visited_at_ms: i64,
}

/// §14.2 `UserTask`.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserTask {
    pub task_id: String,
    /// One of the §6.2 `task_kind` enum values.
    pub task_kind: String,
    pub title: String,
    /// The raw `detail_json` value parsed to a JSON value.
    pub detail: serde_json::Value,
    pub priority: i64,
    pub created_at_ms: i64,
}

/// Aggregate the assembled pieces into a [`HomeSnapshot`].
pub fn assemble(
    recent_packs: Vec<PackManifest>,
    continue_reading: Option<ReaderHistoryItem>,
    open_tasks: Vec<UserTask>,
    memory_inbox_count: i64,
    readiness: SystemReadiness,
) -> HomeSnapshot {
    HomeSnapshot {
        recent_packs,
        continue_reading,
        open_tasks,
        memory_inbox_count,
        readiness,
    }
}
