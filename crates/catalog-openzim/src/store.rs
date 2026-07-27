//! In-memory store and shared catalog-source behaviour used by both adapters.
//!
//! A catalog adapter holds a small read-optimized snapshot of the last
//! successfully-refreshed catalog. `refresh` re-fetches, re-verifies and
//! atomically swaps the snapshot; `search` and `entry` read from it without
//! any I/O.

use crate::CatalogError;
use ports::catalog::{
    CatalogEntry, CatalogPage, CatalogQuery, CatalogRefreshReport,
};
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

/// Wall-clock milliseconds since the Unix epoch. Used for `refreshed_at_ms`.
pub(crate) fn epoch_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Inner mutable state for a catalog source.
pub(crate) struct Snapshot {
    pub entries: Vec<CatalogEntry>,
    pub last_refresh: Option<CatalogRefreshReport>,
    pub last_refreshed_at_ms: i64,
}

impl Default for Snapshot {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            last_refresh: None,
            last_refreshed_at_ms: 0,
        }
    }
}

pub(crate) type SharedSnapshot = RwLock<Snapshot>;

/// Compute the added/updated/removed diff between the previous and the freshly
/// normalized entry sets. Used to populate `CatalogRefreshReport`.
pub(crate) fn diff_refresh(
    old: &[CatalogEntry],
    new: &[CatalogEntry],
) -> (u64, u64, u64) {
    let mut old_map: HashMap<&str, &CatalogEntry> =
        HashMap::with_capacity(old.len());
    for e in old {
        old_map.insert(e.catalog_entry_id.as_str(), e);
    }
    let mut new_map: HashMap<&str, &CatalogEntry> =
        HashMap::with_capacity(new.len());
    for e in new {
        new_map.insert(e.catalog_entry_id.as_str(), e);
    }

    let mut added = 0u64;
    let mut updated = 0u64;
    let mut removed = 0u64;

    for (id, new_entry) in &new_map {
        match old_map.get(id) {
            None => added += 1,
            Some(old_entry) => {
                // An entry is "updated" if any observable field changed. We
                // compare canonical JSON serializations so that *any* drift
                // (sha256, size, version, title, language, ...) is detected.
                let old_ser =
                    serde_json::to_string(*old_entry).unwrap_or_default();
                let new_ser =
                    serde_json::to_string(new_entry).unwrap_or_default();
                if old_ser != new_ser {
                    updated += 1;
                }
            }
        }
    }
    for id in old_map.keys() {
        if !new_map.contains_key(*id) {
            removed += 1;
        }
    }

    (added, updated, removed)
}

/// Apply the text / language / category / size filters from `CatalogQuery`
/// against an entry. All non-`None` filters must pass (logical AND); within
/// `languages` and `categories`, any match passes (logical OR).
pub(crate) fn entry_matches(
    entry: &CatalogEntry,
    query: &CatalogQuery,
) -> bool {
    if let Some(text) = query.text.as_deref() {
        let needle = text.trim().to_lowercase();
        if !needle.is_empty() {
            let hay = format!(
                "{} {} {} {}",
                entry.title, entry.description, entry.corpus_id, entry.category
            )
            .to_lowercase();
            if !hay.contains(&needle) {
                return false;
            }
        }
    }

    if !query.languages.is_empty() {
        let entry_lang = entry.language_bcp47.to_lowercase();
        let any = query
            .languages
            .iter()
            .any(|l| l.to_lowercase() == entry_lang);
        if !any {
            return false;
        }
    }

    if !query.categories.is_empty() {
        let entry_cat = entry.category.to_lowercase();
        let any = query
            .categories
            .iter()
            .any(|c| c.to_lowercase() == entry_cat);
        if !any {
            return false;
        }
    }

    if let Some(max) = query.max_size_bytes {
        if entry.artifact_size_bytes > max {
            return false;
        }
    }

    true
}

/// Slice the filtered entries to the requested page. The total reported to
/// callers is the count of filtered entries (independent of pagination), and
/// `next_offset` is set only when more rows remain.
pub(crate) fn paginate(
    filtered: &[CatalogEntry],
    query: &CatalogQuery,
    refreshed_at_ms: i64,
) -> CatalogPage {
    let total = filtered.len() as u64;
    let offset = query.offset as usize;
    let limit = query.limit as usize;

    let start = offset.min(filtered.len());
    let end = start.saturating_add(limit).min(filtered.len());
    let page: Vec<CatalogEntry> =
        filtered[start..end].iter().cloned().collect();

    let next_offset = if end < filtered.len() {
        Some(end as u32)
    } else {
        None
    };

    CatalogPage {
        entries: page,
        total: Some(total),
        next_offset,
        refreshed_at_ms,
    }
}

/// Look up a single entry by id. Caller has already taken the read lock.
pub(crate) fn find_entry<'a>(
    entries: &'a [CatalogEntry],
    catalog_entry_id: &str,
) -> Result<&'a CatalogEntry, CatalogError> {
    entries
        .iter()
        .find(|e| e.catalog_entry_id == catalog_entry_id)
        .ok_or(CatalogError::NotFound)
}
