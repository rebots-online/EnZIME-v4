//! Read-side projections over the §6.2 schema.
//!
//! Plain row structs plus query functions. Deliberately free of `serde`
//! derives: the storage layer returns raw rows and the application/IPC layer
//! owns the wire-format DTOs (A-02 layering). Every query here is a bounded
//! `SELECT` against indexed columns — no writes, no full-table scans.

#![allow(clippy::needless_pass_by_value)]

use rusqlite::{params, Connection, OptionalExtension};

// ===========================================================================
// §6.2 `reader_history` — continue-reading source
// ===========================================================================

/// One row of `reader_history` (§6.2). The most recent such row per pack is
/// the "continue reading" entry surfaced on the home view (§24).
#[derive(Clone, Debug)]
pub struct ReaderHistoryRow {
    pub history_id: String,
    pub pack_id: String,
    pub article_key: String,
    pub title: String,
    pub visited_at_ms: i64,
}

/// Return the `limit` most-recent `reader_history` rows across all packs,
/// newest first (matches `idx_reader_history_recent`).
pub fn recent_reader_history(
    conn: &Connection,
    limit: i64,
) -> Result<Vec<ReaderHistoryRow>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT history_id, pack_id, article_key, title, visited_at_ms\n\
         \x20\x20FROM reader_history\n\
         \x20\x20ORDER BY visited_at_ms DESC\n\
         \x20\x20LIMIT ?1",
    )?;
    let rows = stmt.query_map(params![limit], |row| {
        Ok(ReaderHistoryRow {
            history_id: row.get(0)?,
            pack_id: row.get(1)?,
            article_key: row.get(2)?,
            title: row.get(3)?,
            visited_at_ms: row.get(4)?,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

// ===========================================================================
// §6.2 `packs` + `pack_versions` — recent packs
// ===========================================================================

/// A `packs` row joined to its active `pack_versions` row (if any).
#[derive(Clone, Debug)]
pub struct PackRow {
    pub pack_id: String,
    pub corpus_id: String,
    pub display_name: String,
    pub description: String,
    pub language_bcp47: String,
    pub pack_kind: String,
    pub active_pack_version_id: Option<String>,
    /// Active version's `upstream_version`, if an active version is set.
    pub active_upstream_version: Option<String>,
    /// Active version's `artifact_sha256`, if an active version is set.
    pub active_artifact_sha256: Option<String>,
    /// Active version's `artifact_size_bytes`, if an active version is set.
    pub active_artifact_size_bytes: Option<i64>,
}

/// Return up to `limit` packs, newest by `updated_at_ms` first, each carrying
/// its active version summary when one is set. `packs.active_pack_version_id`
/// is nullable; the LEFT JOIN keeps packs with no active version.
pub fn recent_packs(conn: &Connection, limit: i64) -> Result<Vec<PackRow>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT p.pack_id,\n\
         \x20\x20      p.corpus_id,\n\
         \x20\x20      p.display_name,\n\
         \x20\x20      p.description,\n\
         \x20\x20      p.language_bcp47,\n\
         \x20\x20      p.pack_kind,\n\
         \x20\x20      p.active_pack_version_id,\n\
         \x20\x20      pv.upstream_version,\n\
         \x20\x20      pv.artifact_sha256,\n\
         \x20\x20      pv.artifact_size_bytes\n\
         \x20\x20 FROM packs p\n\
         \x20\x20 LEFT JOIN pack_versions pv\n\
         \x20\x20   ON pv.pack_version_id = p.active_pack_version_id\n\
         \x20\x20 ORDER BY p.updated_at_ms DESC\n\
         \x20\x20 LIMIT ?1",
    )?;
    let rows = stmt.query_map(params![limit], |row| {
        Ok(PackRow {
            pack_id: row.get(0)?,
            corpus_id: row.get(1)?,
            display_name: row.get(2)?,
            description: row.get(3)?,
            language_bcp47: row.get(4)?,
            pack_kind: row.get(5)?,
            active_pack_version_id: row.get(6)?,
            active_upstream_version: row.get(7)?,
            active_artifact_sha256: row.get(8)?,
            active_artifact_size_bytes: row.get(9)?,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

// ===========================================================================
// §6.2 `user_tasks` — task inbox
// ===========================================================================

/// A `user_tasks` row (§6.2).
#[derive(Clone, Debug)]
pub struct UserTaskRow {
    pub task_id: String,
    pub task_kind: String,
    pub title: String,
    pub detail_json: String,
    pub status: String,
    pub priority: i64,
    pub created_at_ms: i64,
}

/// Return all `user_tasks` whose `status = 'open'`, ordered by the
/// `idx_user_tasks_open` ordering (priority DESC, then created_at_ms).
pub fn open_tasks(conn: &Connection) -> Result<Vec<UserTaskRow>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT task_id, task_kind, title, detail_json, status, priority, created_at_ms\n\
         \x20\x20FROM user_tasks\n\
         \x20\x20WHERE status = 'open'\n\
         \x20\x20ORDER BY priority DESC, created_at_ms",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(UserTaskRow {
            task_id: row.get(0)?,
            task_kind: row.get(1)?,
            title: row.get(2)?,
            detail_json: row.get(3)?,
            status: row.get(4)?,
            priority: row.get(5)?,
            created_at_ms: row.get(6)?,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

// ===========================================================================
// §6.2 `lore_candidates` — memory inbox count
// ===========================================================================

/// Count `lore_candidates` whose `status = 'inbox'` (§24 `memoryInboxCount`).
pub fn memory_inbox_count(conn: &Connection) -> Result<i64, rusqlite::Error> {
    conn.query_row(
        "SELECT COUNT(*) FROM lore_candidates WHERE status = 'inbox'",
        [],
        |row| row.get(0),
    )
}

// ===========================================================================
// §6.2 `app_meta` — first-launch detection
// ===========================================================================

/// `true` iff the `app_meta` key `setup.completed` is absent (§16.7). Per the
/// §14.5 `AppBootstrap.firstLaunch` contract, a missing key means this is the
/// first launch; presence of the key (any JSON value) means setup has run.
pub fn setup_completed(conn: &Connection) -> Result<bool, rusqlite::Error> {
    let value: Option<String> = conn
        .query_row(
            "SELECT value_json FROM app_meta WHERE key = 'setup.completed'",
            [],
            |row| row.get(0),
        )
        .optional()?;
    Ok(value.is_some())
}

/// `true` iff the initial migration (version 1) is recorded in
/// `schema_migrations`. The M1 readiness baseline treats a present schema as
/// the sole "ready" signal.
pub fn schema_version_one_applied(conn: &Connection) -> Result<bool, rusqlite::Error> {
    let version: Option<i64> = conn
        .query_row(
            "SELECT version FROM schema_migrations WHERE version = 1",
            [],
            |row| row.get(0),
        )
        .optional()?;
    Ok(version.is_some())
}
