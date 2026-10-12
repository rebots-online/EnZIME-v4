use super::catalog_budget::directory_bytes;
use super::{DiskPackManifest, PinnedTransferSource};
use anzimmerman::range::ClusterByteRange;
use bytes::Bytes;
use futures_util::StreamExt;
use ports::transfer::{TransferRange, TransferSource};
use rusqlite::{params, Connection, OptionalExtension};
use sha2::{Digest, Sha256};
use std::{
    io::Read,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tokio::{io::AsyncWriteExt, sync::Mutex as AsyncMutex};

pub(crate) fn err(e: impl std::fmt::Display) -> String {
    let text = e.to_string();
    if text == "database or disk is full" {
        format!("budget_exceeded: {text}")
    } else {
        text
    }
}
pub(crate) fn sql(n: u64) -> Result<i64, String> {
    i64::try_from(n).map_err(|_| "integer_overflow".into())
}
pub(crate) fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(i64::MAX as u128) as i64
}
pub(super) fn file_bytes(path: &Path) -> u64 {
    std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
}
pub(super) fn digest_file(path: &Path) -> Result<String, String> {
    let mut file = std::fs::File::open(path).map_err(err)?;
    let mut hash = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = file.read(&mut buf).map_err(err)?;
        if n == 0 {
            break;
        }
        hash.update(&buf[..n]);
    }
    Ok(hex::encode(hash.finalize()))
}

pub struct DiskPackStore {
    pub(crate) root: PathBuf,
    pub(crate) conn: Mutex<Connection>,
    manifest: Mutex<DiskPackManifest>,
    source: Option<Arc<dyn TransferSource>>,
    pub(crate) source_len: u64,
    pub(super) cluster_gate: Arc<AsyncMutex<()>>,
    pub(super) metadata_gate: AsyncMutex<()>,
    pub(super) catalog_gate: Arc<AsyncMutex<()>>,
}
impl DiskPackStore {
    pub fn manifest(&self) -> Result<DiskPackManifest, String> {
        Ok(self.manifest.lock().map_err(err)?.clone())
    }
    pub async fn create(
        root: PathBuf,
        manifest: DiskPackManifest,
        source: Arc<dyn TransferSource>,
    ) -> Result<Arc<Self>, String> {
        manifest.policy.validate()?;
        sql(manifest.identity.size_bytes)?;
        if manifest.format_version != 1 || manifest.identity.size_bytes < 80 {
            return Err("invalid_manifest".into());
        }
        std::fs::create_dir_all(&root).map_err(err)?;
        if root.join("manifest.json").exists() {
            return Err("pack_already_exists".into());
        }
        let free = fs2::available_space(&root).map_err(err)?;
        let margin = (256 * 1024 * 1024).max(free / 20);
        if manifest.policy.disk_budget_bytes > free.saturating_sub(margin) {
            return Err("budget_exceeded: insufficient free storage with safety reserve".into());
        }
        let pinned = Arc::new(PinnedTransferSource::new(source, &manifest.identity)?)
            as Arc<dyn TransferSource>;
        let store = Arc::new(Self::construct(root, manifest, Some(pinned))?);
        store.save_manifest().await?;
        Ok(store)
    }
    pub async fn reopen(
        root: PathBuf,
        source: Option<Arc<dyn TransferSource>>,
    ) -> Result<Arc<Self>, String> {
        let path = root.join("manifest.json");
        if file_bytes(&path) > 65536 {
            return Err("invalid_manifest_size".into());
        }
        let mut manifest: DiskPackManifest =
            serde_json::from_slice(&std::fs::read(path).map_err(err)?).map_err(err)?;
        manifest.policy.validate()?;
        sql(manifest.identity.size_bytes)?;
        if manifest.identity.size_bytes < 80 {
            return Err("invalid_manifest".into());
        }
        if manifest.format_version != 1 {
            return Err("unsupported_pack_format".into());
        }
        if source.is_none() {
            manifest.policy.offline = true;
        }
        let source = source
            .map(|s| {
                PinnedTransferSource::new(s, &manifest.identity)
                    .map(|s| Arc::new(s) as Arc<dyn TransferSource>)
            })
            .transpose()?;
        let store = Arc::new(Self::construct(root, manifest, source)?);
        // Only known numeric staging paths inside our pack root are removed.
        for folder in ["clusters", "pages", "workspace"] {
            for item in std::fs::read_dir(store.root.join(folder)).map_err(err)? {
                let path = item.map_err(err)?.path();
                let name = path.file_name().and_then(|v| v.to_str()).unwrap_or("");
                let numeric = name
                    .strip_suffix(".part")
                    .or_else(|| name.strip_suffix(".decoded"));
                if numeric.is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit())) {
                    std::fs::remove_file(path).map_err(err)?;
                }
            }
        }
        store.conn.lock().map_err(err)?.execute("UPDATE jobs SET state='paused',bytes_stored=(SELECT COALESCE(SUM(byte_length),0) FROM clusters WHERE state='complete') WHERE state='fetching'",[]).map_err(err)?;
        super::catalog_schema::reconcile_payload(&store)?;
        store.save_manifest().await?;
        Ok(store)
    }
    fn construct(
        root: PathBuf,
        manifest: DiskPackManifest,
        source: Option<Arc<dyn TransferSource>>,
    ) -> Result<Self, String> {
        for folder in ["pages", "clusters", "workspace"] {
            std::fs::create_dir_all(root.join(folder)).map_err(err)?;
        }
        let conn = Connection::open(root.join("pack-index.sqlite")).map_err(err)?;
        conn.execute_batch("PRAGMA cache_size=-2048; PRAGMA temp_store=FILE; PRAGMA journal_mode=DELETE; PRAGMA journal_size_limit=1048576;
        CREATE TABLE IF NOT EXISTS clusters(cluster_index INTEGER PRIMARY KEY,start_byte INTEGER NOT NULL,byte_length INTEGER NOT NULL,priority INTEGER NOT NULL DEFAULT 1,article_count INTEGER NOT NULL DEFAULT 0,domain TEXT,topic TEXT,selected INTEGER NOT NULL DEFAULT 0,pinned INTEGER NOT NULL DEFAULT 0,state TEXT NOT NULL DEFAULT 'missing',stored_sha256 TEXT,last_used INTEGER NOT NULL DEFAULT 0);
        CREATE TABLE IF NOT EXISTS pages(page_index INTEGER PRIMARY KEY,byte_length INTEGER NOT NULL,stored_sha256 TEXT NOT NULL);
        CREATE TABLE IF NOT EXISTS jobs(job_id TEXT PRIMARY KEY,state TEXT NOT NULL,bytes_stored INTEGER NOT NULL DEFAULT 0,error TEXT);
        CREATE TABLE IF NOT EXISTS policy(key TEXT PRIMARY KEY,value TEXT NOT NULL);").map_err(err)?;
        let budget = manifest.policy.metadata_budget_bytes;
        let page_bytes = directory_bytes(&root.join("pages"))?;
        let db_bytes = file_bytes(&root.join("pack-index.sqlite"));
        if page_bytes
            .saturating_add(db_bytes.saturating_mul(2))
            .saturating_add(131072)
            > budget
        {
            return Err("budget_exceeded: existing metadata exceeds partition".into());
        }
        conn.pragma_update(
            None,
            "max_page_count",
            (budget - page_bytes - 131072) / 8192,
        )
        .map_err(err)?;
        super::catalog_schema::ensure_catalog_schema(&conn)?;
        Ok(Self {
            source_len: manifest.identity.size_bytes,
            root,
            conn: Mutex::new(conn),
            manifest: Mutex::new(manifest),
            source,
            cluster_gate: Arc::new(AsyncMutex::new(())),
            metadata_gate: AsyncMutex::new(()),
            catalog_gate: Arc::new(AsyncMutex::new(())),
        })
    }
    async fn save_manifest(&self) -> Result<(), String> {
        let bytes = serde_json::to_vec(&self.manifest()?).map_err(err)?;
        if bytes.len() > 65536 {
            return Err("invalid_manifest_size".into());
        }
        let part = self.root.join("manifest.part");
        let mut file = tokio::fs::File::create(&part).await.map_err(err)?;
        file.write_all(&bytes).await.map_err(err)?;
        file.sync_all().await.map_err(err)?;
        drop(file);
        tokio::fs::rename(part, self.root.join("manifest.json"))
            .await
            .map_err(err)
    }
    pub async fn set_offline(&self, offline: bool) -> Result<(), String> {
        let _gate = self.cluster_gate.lock().await;
        let _metadata = self.metadata_gate.lock().await;
        self.manifest.lock().map_err(err)?.policy.offline = offline;
        self.save_manifest().await
    }
    pub async fn used_bytes(&self) -> Result<u64, String> {
        let mut n = 0u64;
        for folder in ["pages", "clusters", "workspace"] {
            n = n
                .checked_add(directory_bytes(&self.root.join(folder))?)
                .ok_or("integer_overflow")?;
        }
        for file in [
            "manifest.json",
            "manifest.part",
            "pack-index.sqlite",
            "pack-index.sqlite-journal",
        ] {
            n = n
                .checked_add(file_bytes(&self.root.join(file)))
                .ok_or("integer_overflow")?;
        }
        Ok(n)
    }
    pub(crate) async fn read_page(
        &self,
        index: u64,
        pinned: bool,
        catalog: bool,
    ) -> Result<Bytes, String> {
        let _guard = self.metadata_gate.lock().await;
        let start = index.checked_mul(65536).ok_or("integer_overflow")?;
        if start >= self.source_len {
            return Err("invalid_page".into());
        }
        let length = 65536.min(self.source_len - start);
        let path = self.root.join("pages").join(index.to_string());
        let record: Option<(u64, String)> = self
            .conn
            .lock()
            .map_err(err)?
            .query_row(
                "SELECT byte_length,stored_sha256 FROM pages WHERE page_index=?1",
                [sql(index)?],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .map_err(err)?;
        if let Some((size, hash)) = record {
            if size == length
                && file_bytes(&path) == length
                && digest_file(&path).ok().as_deref() == Some(&hash)
            {
                self.touch_page(index, pinned)?;
                return Ok(Bytes::from(std::fs::read(path).map_err(err)?));
            }
            let _ = std::fs::remove_file(&path);
            self.conn
                .lock()
                .map_err(err)?
                .execute("DELETE FROM pages WHERE page_index=?1", [sql(index)?])
                .map_err(err)?;
        }
        if self.manifest()?.policy.offline {
            return Err("offline_metadata_missing".into());
        }
        let stale = self.root.join("pages").join(format!("{index}.part"));
        let _ = std::fs::remove_file(stale);
        self.reserve_metadata(length)?;
        let part = self.root.join("pages").join(format!("{index}.part"));
        if catalog {
            self.reserve_catalog_transfer(length)?;
        }
        let hash = self.fetch_file(start, length, &part, catalog).await?;
        tokio::fs::rename(&part, &path).await.map_err(err)?;
        self.conn
            .lock()
            .map_err(err)?
            .execute(
                "INSERT OR REPLACE INTO pages VALUES(?1,?2,?3)",
                params![sql(index)?, sql(length)?, hash],
            )
            .map_err(err)?;
        self.touch_page(index, pinned)?;
        Ok(Bytes::from(std::fs::read(path).map_err(err)?))
    }
    fn touch_page(&self, index: u64, pinned: bool) -> Result<(), String> {
        let conn = self.conn.lock().map_err(err)?;
        if pinned {
            conn.execute(
                "INSERT OR REPLACE INTO policy VALUES(?1,'1')",
                [format!("pin.page.{index}")],
            )
            .map_err(err)?;
        }
        conn.execute(
            "INSERT OR REPLACE INTO policy VALUES(?1,?2)",
            params![format!("lru.page.{index}"), now().to_string()],
        )
        .map_err(err)?;
        Ok(())
    }
    async fn fetch_file(
        &self,
        start: u64,
        length: u64,
        path: &Path,
        catalog: bool,
    ) -> Result<String, String> {
        let manifest = self.manifest()?;
        let source = self.source.as_ref().ok_or("offline_source_unavailable")?;
        let mut stream = source
            .open_range(
                &manifest.identity.uri,
                TransferRange {
                    start_byte: start,
                    end_byte_exclusive: start.checked_add(length).ok_or("integer_overflow")?,
                },
                manifest.identity.etag.as_deref(),
            )
            .await
            .map_err(err)?;
        let result = async {
            let mut file = tokio::fs::File::create(path).await.map_err(err)?;
            let mut hash = Sha256::new();
            let mut count = 0u64;
            while let Some(chunk) = stream.next().await {
                let bytes = chunk.map_err(err)?;
                count = count
                    .checked_add(bytes.len() as u64)
                    .ok_or("integer_overflow")?;
                if count > length {
                    return Err("range_length_exceeded".into());
                }
                if catalog {
                    self.record_catalog_received(bytes.len() as u64)?;
                }
                for block in bytes.chunks(65536) {
                    file.write_all(block).await.map_err(err)?;
                    hash.update(block);
                }
            }
            if count != length {
                return Err("range_short".into());
            }
            file.sync_all().await.map_err(err)?;
            Ok(hex::encode(hash.finalize()))
        }
        .await;
        if result.is_err() {
            let _ = tokio::fs::remove_file(path).await;
        }
        result
    }
    pub async fn ensure_cluster(
        &self,
        index: u32,
        range: ClusterByteRange,
        pinned: bool,
    ) -> Result<PathBuf, String> {
        let _gate = self.cluster_gate.lock().await;
        self.ensure_inner(index, range, pinned).await
    }
    pub(super) async fn ensure_inner(
        &self,
        index: u32,
        range: ClusterByteRange,
        pinned: bool,
    ) -> Result<PathBuf, String> {
        let _metadata = self.metadata_gate.lock().await;
        if range.length == 0
            || range
                .start
                .checked_add(range.length)
                .is_none_or(|n| n > self.source_len)
        {
            return Err("invalid_cluster_range".into());
        }
        let path = self.root.join("clusters").join(format!("{index}.bin"));
        let row: Option<(u64, u64, Option<String>)> = self
            .conn
            .lock()
            .map_err(err)?
            .query_row(
                "SELECT start_byte,byte_length,stored_sha256 FROM clusters WHERE cluster_index=?1",
                [index],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()
            .map_err(err)?;
        if let Some((start, len, hash)) = &row {
            if *start != range.start || *len != range.length {
                return Err("source_changed: cluster mapping".into());
            }
            if let Some(hash) = hash {
                if file_bytes(&path) == range.length
                    && digest_file(&path).ok().as_deref() == Some(hash)
                {
                    self.conn.lock().map_err(err)?.execute("UPDATE clusters SET state='complete',pinned=MAX(pinned,?2),last_used=?3 WHERE cluster_index=?1",params![index,pinned,now()]).map_err(err)?;
                    return Ok(path);
                }
            }
        }
        self.conn
            .lock()
            .map_err(err)?
            .execute(
                "UPDATE clusters SET state='missing',stored_sha256=NULL WHERE cluster_index=?1",
                [index],
            )
            .map_err(err)?;
        if self.manifest()?.policy.offline {
            return Err("offline_cluster_missing_or_corrupt".into());
        }
        self.reserve_metadata(65536)?;
        let stale = self.root.join("clusters").join(format!("{index}.part"));
        let _ = std::fs::remove_file(stale);

        let _ = std::fs::remove_file(&path);
        self.conn.lock().map_err(err)?.execute("INSERT INTO clusters(cluster_index,start_byte,byte_length,pinned) VALUES(?1,?2,?3,?4) ON CONFLICT(cluster_index) DO UPDATE SET state='missing',stored_sha256=NULL,pinned=MAX(pinned,excluded.pinned)",params![index,sql(range.start)?,sql(range.length)?,pinned]).map_err(err)?;
        let budget = self.manifest()?.policy.payload_budget()?;
        if range.length > budget {
            return Err("budget_exceeded: cluster exceeds payload budget".into());
        }
        while directory_bytes(&self.root.join("clusters"))?
            .checked_add(range.length)
            .ok_or("integer_overflow")?
            > budget
        {
            let conn = self.conn.lock().map_err(err)?;
            let victim:Option<u32>=conn.query_row("SELECT cluster_index FROM clusters WHERE state='complete' AND pinned=0 AND cluster_index<>?1 ORDER BY last_used,cluster_index LIMIT 1",[index],|r|r.get(0)).optional().map_err(err)?;
            let Some(v) = victim else {
                return Err("budget_exceeded: pinned payload partition".into());
            };
            std::fs::remove_file(self.root.join("clusters").join(format!("{v}.bin")))
                .map_err(err)?;
            conn.execute(
                "UPDATE clusters SET state='missing',stored_sha256=NULL WHERE cluster_index=?1",
                [v],
            )
            .map_err(err)?;
        }
        self.verify_weak_identity().await?;
        self.conn.lock().map_err(err)?.execute("INSERT INTO jobs(job_id,state) VALUES('download','fetching') ON CONFLICT(job_id) DO UPDATE SET state='fetching',error=NULL",[]).map_err(err)?;
        let part = self.root.join("clusters").join(format!("{index}.part"));
        let hash = match self
            .fetch_file(range.start, range.length, &part, false)
            .await
        {
            Ok(h) => h,
            Err(e) => {
                self.conn
                    .lock()
                    .map_err(err)?
                    .execute(
                        "UPDATE jobs SET state='paused',error=?1 WHERE job_id='download'",
                        [&e],
                    )
                    .map_err(err)?;
                return Err(e);
            }
        };
        tokio::fs::rename(part, &path).await.map_err(err)?;
        self.conn.lock().map_err(err)?.execute("UPDATE clusters SET state='complete',stored_sha256=?2,last_used=?3 WHERE cluster_index=?1",params![index,hash,now()]).map_err(err)?;
        self.conn.lock().map_err(err)?.execute("UPDATE jobs SET bytes_stored=(SELECT COALESCE(SUM(byte_length),0) FROM clusters WHERE state='complete') WHERE job_id='download'",[]).map_err(err)?;
        Ok(path)
    }
    async fn verify_weak_identity(&self) -> Result<(), String> {
        let manifest = self.manifest()?;
        if manifest
            .identity
            .etag
            .as_ref()
            .is_some_and(|e| e.len() >= 2 && e.starts_with('"') && e.ends_with('"'))
        {
            return Ok(());
        }
        let source = self.source.as_ref().ok_or("offline_source_unavailable")?;
        let mut s = source
            .open_range(
                &manifest.identity.uri,
                TransferRange {
                    start_byte: 0,
                    end_byte_exclusive: 80,
                },
                None,
            )
            .await
            .map_err(err)?;
        let mut bytes = Vec::with_capacity(80);
        while let Some(b) = s.next().await {
            let b = b.map_err(err)?;
            if bytes.len() + b.len() > 80 {
                return Err("source_changed".into());
            }
            bytes.extend_from_slice(&b);
        }
        let header = anzimmerman::header::parse_header(&bytes).map_err(err)?;
        if hex::encode(header.uuid) != manifest.identity.zim_uuid {
            return Err("source_changed: UUID".into());
        }
        Ok(())
    }
}
