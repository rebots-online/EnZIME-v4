use super::{catalog::OfflineCatalogProgress, store::err, DiskPackStore, DiskRangeSource};
use anzimmerman::{
    directory::RawEntry,
    header::NO_PAGE,
    range::{CanonicalEntryRecord, RangeReadLimits, RemoteZimIndex},
};
use rusqlite::{params, OptionalExtension};
use sha2::{Digest, Sha256};
use std::sync::Arc;
pub(super) fn identity(store: &DiskPackStore) -> Result<String, String> {
    Ok(hex::encode(Sha256::digest(
        serde_json::to_vec(&store.manifest()?.identity).map_err(err)?,
    )))
}
fn initialize(store: &DiskPackStore, total: u32) -> Result<(), String> {
    store.reserve_metadata(128 * 1024)?;
    let manifest = store.manifest()?;
    let source = identity(store)?;
    let conn = store.conn.lock().map_err(err)?;
    let existing: Option<String> = conn
        .query_row(
            "SELECT value FROM policy WHERE key='catalog.source_identity'",
            [],
            |r| r.get(0),
        )
        .optional()
        .map_err(err)?;
    if existing.as_ref().is_some_and(|x| x != &source) {
        return Err("source_changed: catalog identity".into());
    }
    let tx = conn.unchecked_transaction().map_err(err)?;
    for (key, value) in [
        ("version", "1".to_string()),
        ("source_identity", source),
        ("cursor", "0".into()),
        ("complete", "0".into()),
        ("stop_reason", "".into()),
        ("total", total.to_string()),
        (
            "network_limit",
            (256 * 1024 * 1024u64)
                .min(manifest.policy.disk_budget_bytes / 4)
                .to_string(),
        ),
        ("network_reserved", "0".into()),
        ("network_received", "0".into()),
    ] {
        tx.execute(
            "INSERT OR IGNORE INTO policy VALUES(?1,?2)",
            params![format!("catalog.{key}"), value],
        )
        .map_err(err)?;
    }
    tx.commit().map_err(err)
}
pub(super) fn admit_entry_chain(
    store: &DiskPackStore,
    records: &[CanonicalEntryRecord],
    terminal_cluster: u32,
) -> Result<(), String> {
    let mut conn = store.conn.lock().map_err(err)?;
    let tx = conn.transaction().map_err(err)?;
    let exists: bool = tx
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM clusters WHERE cluster_index=?1)",
            [terminal_cluster],
            |r| r.get(0),
        )
        .map_err(err)?;
    if !exists {
        return Err("catalog_cluster_mapping_missing".into());
    }
    for record in records {
        if record.raw_bytes.len() > 65536 {
            return Err("catalog_entry_too_large".into());
        }
        if let RawEntry::Article(article) = &record.entry.raw {
            let mime = record
                .entry
                .mime_type
                .as_deref()
                .ok_or("catalog_mime_missing")?;
            let key = format!("catalog.mime.{}", article.mimetype_index);
            tx.execute(
                "INSERT OR IGNORE INTO policy VALUES(?1,?2)",
                params![key, mime],
            )
            .map_err(err)?;
            let saved: String = tx
                .query_row("SELECT value FROM policy WHERE key=?1", [key], |r| r.get(0))
                .map_err(err)?;
            if saved != mime {
                return Err("source_changed: catalog MIME".into());
            }
        }
        tx.execute("INSERT INTO catalog_entries VALUES(?1,?2,?3,?4,?5) ON CONFLICT(entry_index) DO UPDATE SET entry_key=excluded.entry_key,raw_entry=excluded.raw_entry,mime_type=excluded.mime_type,owner_cluster=excluded.owner_cluster",params![record.entry.index,record.entry.key.0,record.raw_bytes.as_ref(),record.entry.mime_type,terminal_cluster]).map_err(err)?;
    }
    tx.commit().map_err(err)
}
pub(super) async fn admit_with_budget(
    store: &DiskPackStore,
    records: &[CanonicalEntryRecord],
    cluster: u32,
    prune: bool,
) -> Result<bool, String> {
    let _metadata = store.metadata_gate.lock().await;
    let size = records.iter().try_fold(65536u64, |n, r| {
        n.checked_add(
            (r.raw_bytes.len()
                + r.entry.key.0.len()
                + r.entry.mime_type.as_ref().map_or(0, String::len)
                + 256) as u64
                * 4,
        )
        .ok_or("integer_overflow")
    })?;
    loop {
        let free: u64 = store
            .conn
            .lock()
            .map_err(err)?
            .query_row("PRAGMA freelist_count", [], |r| r.get(0))
            .map_err(err)?;
        let growth = size.saturating_sub(free.saturating_mul(8192));
        match store
            .reserve_metadata(growth)
            .and_then(|_| admit_entry_chain(store, records, cluster))
        {
            Ok(()) => return Ok(true),
            Err(e) if e.contains("budget_exceeded") && prune => {
                let conn = store.conn.lock().map_err(err)?;
                let victim:Option<u32>=conn.query_row("SELECT cluster_index FROM clusters INDEXED BY clusters_catalog_rank WHERE selected=1 AND pinned=0 ORDER BY priority ASC,article_count ASC,cluster_index DESC LIMIT 1",[],|r|r.get(0)).optional().map_err(err)?;
                let Some(victim) = victim else {
                    return Err("budget_exceeded: mandatory main catalog does not fit".into());
                };
                let tx = conn.unchecked_transaction().map_err(err)?;
                tx.execute(
                    "UPDATE clusters SET selected=0 WHERE cluster_index=?1",
                    [victim],
                )
                .map_err(err)?;
                tx.execute(
                    "DELETE FROM catalog_entries WHERE owner_cluster=?1",
                    [victim],
                )
                .map_err(err)?;
                tx.execute(
                    "UPDATE policy SET value='metadata_budget' WHERE key='catalog.stop_reason'",
                    [],
                )
                .map_err(err)?;
                tx.commit().map_err(err)?;
                if victim == cluster {
                    return Ok(false);
                }
            }
            Err(e) => return Err(e),
        }
    }
}
fn terminal(records: &[CanonicalEntryRecord]) -> Option<u32> {
    match &records.last()?.entry.raw {
        RawEntry::Article(a) => Some(a.cluster_number),
        _ => None,
    }
}
fn selected(store: &DiskPackStore, id: u32) -> Result<bool, String> {
    store
        .conn
        .lock()
        .map_err(err)?
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM clusters WHERE cluster_index=?1 AND selected=1)",
            [id],
            |r| r.get(0),
        )
        .map_err(err)
}
fn progress(
    store: &DiskPackStore,
    cursor: u32,
    complete: bool,
    reason: Option<&str>,
) -> Result<(), String> {
    let conn = store.conn.lock().map_err(err)?;
    let tx = conn.unchecked_transaction().map_err(err)?;
    for (key, value) in [
        ("cursor", cursor.to_string()),
        ("complete", u8::from(complete).to_string()),
        ("stop_reason", reason.unwrap_or("").to_string()),
    ] {
        tx.execute(
            "UPDATE policy SET value=?2 WHERE key=?1",
            params![format!("catalog.{key}"), value],
        )
        .map_err(err)?;
    }
    tx.commit().map_err(err)
}
pub async fn discover_offline_entries(
    store: Arc<DiskPackStore>,
    index: Arc<RemoteZimIndex>,
) -> Result<OfflineCatalogProgress, String> {
    let _gate = store.catalog_gate.lock().await;
    {
        let _metadata = store.metadata_gate.lock().await;
        initialize(&store, index.header().entry_count)?;
    }
    let index = Arc::new(
        RemoteZimIndex::open(
            Arc::new(DiskRangeSource::new_for_catalog(store.clone())),
            RangeReadLimits::default(),
        )
        .await
        .map_err(err)?,
    );
    if hex::encode(index.header().uuid) != store.manifest()?.identity.zim_uuid {
        return Err("source_changed: catalog UUID".into());
    }
    let main = index.header().main_page;
    if main != NO_PAGE {
        let records = index.resolve_records(main, 16).await.map_err(|e| {
            let message = e.to_string();
            if message.contains("catalog_transfer_allowance") || message.contains("budget_exceeded")
            {
                format!("budget_exceeded: mandatory main catalog unavailable: {message}")
            } else {
                message
            }
        })?;
        let cluster = terminal(&records).ok_or("catalog_main_not_article")?;
        if !selected(&store, cluster)? {
            return Err("catalog_main_not_selected".into());
        }
        if !admit_with_budget(&store, &records, cluster, true).await? {
            return Err("budget_exceeded: mandatory main catalog pruned".into());
        }
    }
    let mut cursor = store.catalog_progress()?.scanned_entries;
    let mut stop = None;
    while cursor < index.header().entry_count {
        let records = match index.resolve_records(cursor, 16).await {
            Ok(records) => records,
            Err(e) if e.to_string().contains("catalog_transfer_allowance") => {
                stop = Some("transfer_allowance");
                break;
            }
            Err(e) if e.to_string().contains("budget_exceeded") => {
                stop = Some("metadata_budget");
                break;
            }
            Err(e) => return Err(err(e)),
        };
        if let Some(cluster) = terminal(&records) {
            if selected(&store, cluster)? {
                match admit_with_budget(&store, &records, cluster, true).await {
                    Ok(true) => {}
                    Ok(false) => stop = Some("metadata_budget"),
                    Err(e) if e.contains("budget_exceeded") => {
                        stop = Some("metadata_budget");
                        break;
                    }
                    Err(e) => return Err(e),
                }
            }
        }
        let pressure: bool = store
            .conn
            .lock()
            .map_err(err)?
            .query_row(
                "SELECT value='metadata_budget' FROM policy WHERE key='catalog.stop_reason'",
                [],
                |r| r.get(0),
            )
            .map_err(err)?;
        if pressure {
            stop = Some("metadata_budget");
        }
        cursor += 1;
        progress(&store, cursor, false, stop)?;
    }
    {
        let _metadata = store.metadata_gate.lock().await;
        store.reserve_metadata(65536)?;
        store.conn.lock().map_err(err)?.execute("UPDATE clusters SET selected=0 WHERE selected=1 AND pinned=0 AND NOT EXISTS(SELECT 1 FROM catalog_entries e WHERE e.owner_cluster=clusters.cluster_index)",[]).map_err(err)?;
        // Leave actual SQLite page capacity for hashes, state transitions and job rows.
        loop {
            let conn = store.conn.lock().map_err(err)?;
            let free: u64 = conn
                .query_row("PRAGMA freelist_count", [], |r| r.get(0))
                .map_err(err)?;
            let pages: u64 = conn
                .query_row("PRAGMA page_count", [], |r| r.get(0))
                .map_err(err)?;
            let max: u64 = conn
                .query_row("PRAGMA max_page_count", [], |r| r.get(0))
                .map_err(err)?;
            let count: u64 = conn
                .query_row("SELECT COUNT(*) FROM clusters WHERE selected=1", [], |r| {
                    r.get(0)
                })
                .map_err(err)?;
            if free + max.saturating_sub(pages) >= 16 + count.div_ceil(16) {
                break;
            }
            let victim:Option<u32>=conn.query_row("SELECT cluster_index FROM clusters INDEXED BY clusters_catalog_rank WHERE selected=1 AND pinned=0 ORDER BY priority ASC,article_count ASC,cluster_index DESC LIMIT 1",[],|r|r.get(0)).optional().map_err(err)?;
            let victim = victim
                .ok_or("budget_exceeded: mandatory main catalog leaves no install capacity")?;
            let tx = conn.unchecked_transaction().map_err(err)?;
            tx.execute(
                "DELETE FROM catalog_entries WHERE owner_cluster=?1",
                [victim],
            )
            .map_err(err)?;
            tx.execute(
                "UPDATE clusters SET selected=0 WHERE cluster_index=?1",
                [victim],
            )
            .map_err(err)?;
            tx.commit().map_err(err)?;
            stop = Some("metadata_budget");
        }
        progress(
            &store,
            cursor,
            cursor == index.header().entry_count && stop.is_none(),
            stop,
        )?;
    }
    store.catalog_planned_progress()
}
