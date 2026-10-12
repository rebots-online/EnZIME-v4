use super::*;
// ===========================================================================
// Row → DTO mapping (existing home_snapshot mappers)
// ===========================================================================

pub(super) fn map_pack(row: storage_sqlite::query::PackRow) -> PackManifest {
    let active_version_id = row.active_pack_version_id.clone();
    let versions = match &active_version_id {
        Some(id) => vec![PackVersionSummary {
            pack_version_id: id.clone(),
            upstream_version: row.active_upstream_version.unwrap_or_default(),
            artifact_sha256: row.active_artifact_sha256.clone(),
            artifact_size_bytes: row.active_artifact_size_bytes.unwrap_or(0),
            verification_state: if row.active_artifact_sha256.is_some() {
                "verified"
            } else {
                "source_pinned"
            }
            .to_string(),
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

pub(super) fn map_history(row: storage_sqlite::query::ReaderHistoryRow) -> ReaderHistoryItem {
    ReaderHistoryItem {
        pack_id: row.pack_id,
        article_key: row.article_key,
        title: row.title,
        visited_at_ms: row.visited_at_ms,
    }
}

pub(super) fn map_task(row: storage_sqlite::query::UserTaskRow) -> UserTask {
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

pub(super) fn map_internal_link(link: ports::zim::InternalLink) -> InternalLinkDto {
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

pub(super) fn map_catalog_entry(row: CatalogEntryRow) -> LibraryCatalogEntry {
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

// ===========================================================================
// Inline DB rows (tables not covered by storage-sqlite::query at M1)
// ===========================================================================

/// One row of `storage_roots` (§6.2).
pub(super) struct StorageRootRow {
    pub(super) storage_root_id: String,
    pub(super) display_name: String,
    pub(super) writable: bool,
    pub(super) platform_locator: String,
}

pub(super) fn all_storage_roots(conn: &Connection) -> Result<Vec<StorageRootRow>, rusqlite::Error> {
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
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

pub(super) fn storage_root_by_id(
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
        })
    })?;
    match rows.next() {
        Some(r) => Ok(Some(r?)),
        None => Ok(None),
    }
}

/// One row of `catalog_entries` (§6.2).
pub(super) struct CatalogEntryRow {
    pub(super) catalog_entry_id: String,
    pub(super) corpus_id: String,
    pub(super) title: String,
    pub(super) description: String,
    pub(super) language_bcp47: String,
    pub(super) category: String,
    pub(super) publisher: String,
    pub(super) artifact_url: String,
    pub(super) artifact_size_bytes: i64,
    pub(super) artifact_sha256: Option<String>,
}

pub(super) fn catalog_entry_by_id(
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

pub(super) fn search_catalog_entries(
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
        (Some(lang), Some(cat)) => {
            stmt.query_map(params![query, lang, cat, limit, offset], mapper)?
        }
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
pub(super) struct DownloadJobRow {
    pub(super) download_job_id: String,
    pub(super) state: String,
    pub(super) bytes_verified: i64,
    pub(super) bytes_retained: i64,
    pub(super) transfer_scope: String,
    pub(super) error_detail: Option<String>,
    pub(super) expected_size_bytes: i64,
}

pub(super) fn page_download_jobs(
    conn: &Connection,
    limit: i64,
    offset: i64,
) -> Result<Vec<DownloadJobRow>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT download_job_id, state, bytes_verified, expected_size_bytes, bytes_retained, transfer_scope, error_detail\n\
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
            bytes_retained: row.get(4)?,
            transfer_scope: row.get(5)?,
            error_detail: row.get(6)?,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

/// Most-recent `reader_history.visited_at_ms` for a pack, if any.
pub(super) fn last_opened_ms(
    conn: &Connection,
    pack_id: &str,
) -> Result<Option<i64>, rusqlite::Error> {
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
pub(super) fn insert_or_ignore_pack(
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
        params![
            pack_id,
            display_name,
            description,
            language_bcp47,
            updated_at_ms
        ],
    )?;
    Ok(())
}

/// Point `packs.active_pack_version_id` at a verified `pack_versions` row.
/// The §6.2 trigger `trg_pack_active_version_same_pack` is the authority.
pub(super) fn activate_pack_version(
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
