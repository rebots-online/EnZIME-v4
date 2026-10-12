use super::{
    catalog_discovery::{admit_with_budget, identity},
    store::err,
    DiskPackStore, DiskRangeSource,
};
use anzimmerman::{
    directory::{parse_entry, RawEntry},
    header::ZimHeader,
    range::{
        ClusterByteRange, IndexedEntry, IndexedEntryStream, RangeReadLimits, RemoteZimIndex,
        ZimEntryIndex,
    },
    EntryKey, ZimError,
};
use async_trait::async_trait;
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OfflineCatalogProgress {
    pub scanned_entries: u32,
    pub total_entries: u32,
    pub readable_articles: u64,
    pub readable_resources: u64,
    pub retained_aliases: u64,
    pub network_allowance_bytes: u64,
    pub network_reserved_bytes: u64,
    pub network_received_bytes: u64,
    pub complete: bool,
    pub stop_reason: Option<String>,
}
impl DiskPackStore {
    pub fn catalog_progress(&self) -> Result<OfflineCatalogProgress, String> {
        self.catalog_counts(false)
    }
    pub fn catalog_planned_progress(&self) -> Result<OfflineCatalogProgress, String> {
        self.catalog_counts(true)
    }
    fn catalog_counts(&self, planned: bool) -> Result<OfflineCatalogProgress, String> {
        let conn = self.conn.lock().map_err(err)?;
        let value = |key: &str| -> Result<String, String> {
            conn.query_row(
                "SELECT value FROM policy WHERE key=?1",
                [format!("catalog.{key}")],
                |r| r.get(0),
            )
            .map_err(err)
        };
        let number = |key: &str| -> Result<u64, String> { value(key)?.parse().map_err(err) };
        let predicate = if planned {
            "c.selected=1"
        } else {
            "c.state='complete'"
        };
        let query=format!("SELECT COALESCE(SUM(e.mime_type LIKE 'text/html%'),0),COALESCE(SUM(e.mime_type IS NOT NULL AND e.mime_type NOT LIKE 'text/html%'),0),COALESCE(SUM(e.mime_type IS NULL),0) FROM catalog_entries e JOIN clusters c ON c.cluster_index=e.owner_cluster WHERE {predicate}");
        let (articles, resources, aliases) = conn
            .query_row(&query, [], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .map_err(err)?;
        let reason = value("stop_reason")?;
        Ok(OfflineCatalogProgress {
            scanned_entries: number("cursor")?.try_into().map_err(err)?,
            total_entries: number("total")?.try_into().map_err(err)?,
            readable_articles: articles,
            readable_resources: resources,
            retained_aliases: aliases,
            network_allowance_bytes: number("network_limit")?,
            network_reserved_bytes: number("network_reserved")?,
            network_received_bytes: number("network_received")?,
            complete: number("complete")? == 1,
            stop_reason: if reason.is_empty() {
                None
            } else {
                Some(reason)
            },
        })
    }
}
pub struct DiskEntryCatalog {
    store: Arc<DiskPackStore>,
    header: ZimHeader,
    remote: Option<Arc<RemoteZimIndex>>,
}
type Row = (u32, String, Vec<u8>, Option<String>);
fn decode(row: Row) -> Result<IndexedEntry, ZimError> {
    let (index, key, bytes, mime) = row;
    let (raw, used) = parse_entry(&bytes)?;
    if used != bytes.len()
        || key != format!("{}/{}", raw.namespace() as char, raw.url())
        || matches!(raw, RawEntry::Article(_)) != mime.is_some()
    {
        return Err(ZimError::Io("catalog_record_corrupt".into()));
    }
    Ok(IndexedEntry {
        index,
        key: EntryKey(key),
        raw,
        mime_type: mime,
    })
}
fn row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Row> {
    Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
}
impl DiskEntryCatalog {
    pub fn new(store: Arc<DiskPackStore>, header: ZimHeader) -> Result<Self, String> {
        let expected = identity(&store)?;
        {
            let conn = store.conn.lock().map_err(err)?;
            let version: Option<String> = conn
                .query_row(
                    "SELECT value FROM policy WHERE key='catalog.version'",
                    [],
                    |r| r.get(0),
                )
                .optional()
                .map_err(err)?;
            if version.as_deref() != Some("1") {
                return Err("offline_catalog_upgrade_required".into());
            }
            let source: String = conn
                .query_row(
                    "SELECT value FROM policy WHERE key='catalog.source_identity'",
                    [],
                    |r| r.get(0),
                )
                .map_err(err)?;
            if source != expected {
                return Err("source_changed: catalog identity".into());
            }
        }
        Ok(Self {
            store,
            header,
            remote: None,
        })
    }
    fn decode_entry(&self, row: Row) -> Result<IndexedEntry, ZimError> {
        let entry = decode(row)?;
        if let RawEntry::Article(article) = &entry.raw {
            let mime: String = self
                .store
                .conn
                .lock()
                .map_err(|e| ZimError::Io(err(e)))?
                .query_row(
                    "SELECT value FROM policy WHERE key=?1",
                    [format!("catalog.mime.{}", article.mimetype_index)],
                    |r| r.get(0),
                )
                .map_err(|e| ZimError::Io(err(e)))?;
            if entry.mime_type.as_deref() != Some(&mime) {
                return Err(ZimError::Io("catalog_mime_corrupt".into()));
            }
        }
        Ok(entry)
    }
    fn batch(&self, after: i64) -> Result<Vec<Row>, ZimError> {
        let conn = self.store.conn.lock().map_err(|e| ZimError::Io(err(e)))?;
        let mut stmt=conn.prepare("SELECT e.entry_index,e.entry_key,e.raw_entry,e.mime_type FROM catalog_entries e JOIN clusters c ON c.cluster_index=e.owner_cluster WHERE e.entry_index>?1 AND c.state='complete' ORDER BY e.entry_index LIMIT 128").map_err(|e|ZimError::Io(err(e)))?;
        let rows = stmt
            .query_map([after], row)
            .map_err(|e| ZimError::Io(err(e)))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| ZimError::Io(err(e)))
    }
    pub(super) fn validate_planned_main(&self) -> Result<(), String> {
        use anzimmerman::header::NO_PAGE;
        if self.header.main_page == NO_PAGE {
            return Err("catalog_main_missing".into());
        }
        let mut index = self.header.main_page;
        let mut seen = std::collections::HashSet::new();
        for _ in 0..=16 {
            if !seen.insert(index) {
                return Err("catalog_main_redirect_cycle".into());
            }
            let record=self.store.conn.lock().map_err(err)?.query_row("SELECT e.entry_index,e.entry_key,e.raw_entry,e.mime_type FROM catalog_entries e JOIN clusters c ON c.cluster_index=e.owner_cluster WHERE e.entry_index=?1 AND c.selected=1",[index],row).optional().map_err(err)?.ok_or("catalog_main_not_admitted")?;
            match self.decode_entry(record).map_err(err)?.raw {
                RawEntry::Article(_) => return Ok(()),
                RawEntry::Redirect(r) => index = r.redirect_index,
                _ => return Err("catalog_main_not_article".into()),
            }
        }
        Err("catalog_main_redirect_limit".into())
    }
    pub fn with_remote(mut self, index: Arc<RemoteZimIndex>) -> Self {
        self.remote = Some(index);
        self
    }
    fn lookup(
        &self,
        by_key: bool,
        value: &str,
        available: bool,
    ) -> Result<Option<IndexedEntry>, ZimError> {
        let query=format!("SELECT e.entry_index,e.entry_key,e.raw_entry,e.mime_type FROM catalog_entries e JOIN clusters c ON c.cluster_index=e.owner_cluster WHERE {}=?1 {}",if by_key{"e.entry_key"}else{"e.entry_index"},if available{"AND c.state='complete'"}else{""});
        let result = self
            .store
            .conn
            .lock()
            .map_err(|e| ZimError::Io(err(e)))?
            .query_row(&query, [value], row)
            .optional()
            .map_err(|e| ZimError::Io(err(e)))?;
        result.map(|r| self.decode_entry(r)).transpose()
    }
    async fn remote_index(&self) -> Result<Arc<RemoteZimIndex>, ZimError> {
        if self.remote.is_none() || self.store.manifest().map_err(ZimError::Io)?.policy.offline {
            return Err(ZimError::EntryNotFound("offline catalog".into()));
        }
        Ok(Arc::new(
            RemoteZimIndex::open(
                Arc::new(DiskRangeSource::new_for_catalog(self.store.clone())),
                RangeReadLimits::default(),
            )
            .await?,
        ))
    }
    async fn fallback(&self, index: u32) -> Result<IndexedEntry, ZimError> {
        if self.store.manifest().map_err(ZimError::Io)?.policy.offline {
            return Err(ZimError::EntryNotFound(index.to_string()));
        }
        let remote = self.remote_index().await?;
        let _gate = self.store.catalog_gate.lock().await;
        let records = remote.resolve_records(index, 16).await?;
        let cluster = match &records
            .last()
            .ok_or_else(|| ZimError::EntryNotFound(index.to_string()))?
            .entry
            .raw
        {
            RawEntry::Article(a) => a.cluster_number,
            _ => return Err(ZimError::EntryNotFound(index.to_string())),
        };
        admit_with_budget(&self.store, &records, cluster, false)
            .await
            .map_err(ZimError::Io)?;
        Ok(records[0].entry.clone())
    }
}
#[async_trait]
impl ZimEntryIndex for DiskEntryCatalog {
    fn header(&self) -> &ZimHeader {
        &self.header
    }
    async fn entry_at(&self, index: u32) -> Result<IndexedEntry, ZimError> {
        let offline = self.store.manifest().map_err(ZimError::Io)?.policy.offline;
        if let Some(entry) = self.lookup(false, &index.to_string(), offline)? {
            return Ok(entry);
        }
        self.fallback(index).await
    }
    async fn find_entry(&self, key: &EntryKey) -> Result<Option<IndexedEntry>, ZimError> {
        let offline = self.store.manifest().map_err(ZimError::Io)?.policy.offline;
        if let Some(entry) = self.lookup(true, &key.0, offline)? {
            return Ok(Some(entry));
        }
        if offline {
            return Ok(None);
        }
        if self.remote.is_some() {
            let remote = self.remote_index().await?;
            if let Some(entry) = remote.find_entry(key).await? {
                return self.fallback(entry.index).await.map(Some);
            }
        }
        Ok(None)
    }
    async fn cluster_range(&self, index: u32) -> Result<ClusterByteRange, ZimError> {
        self.store
            .conn
            .lock()
            .map_err(|e| ZimError::Io(err(e)))?
            .query_row(
                "SELECT start_byte,byte_length FROM clusters WHERE cluster_index=?1",
                [index],
                |r| {
                    Ok(ClusterByteRange {
                        start: r.get(0)?,
                        length: r.get(1)?,
                    })
                },
            )
            .map_err(|e| ZimError::Io(err(e)))
    }
    async fn entries(self: Arc<Self>) -> Result<IndexedEntryStream, ZimError> {
        Ok(Box::pin(async_stream::try_stream! {
            let mut after=-1i64;
            loop {
                let batch=self.batch(after)?;
                if batch.is_empty(){break}
                for record in batch {after=i64::from(record.0);yield self.decode_entry(record)?;}
            }
        }))
    }
}
