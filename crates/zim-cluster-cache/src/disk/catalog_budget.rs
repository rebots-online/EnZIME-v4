use super::store::file_bytes;
use super::{
    store::{err, sql},
    DiskPackStore,
};
use rusqlite::params;
use rusqlite::OptionalExtension;
use std::path::Path;
impl DiskPackStore {
    pub(super) fn reserve_catalog_transfer(&self, length: u64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(err)?;
        let changed=conn.execute("UPDATE policy SET value=CAST(value AS INTEGER)+?1 WHERE key='catalog.network_reserved' AND CAST(value AS INTEGER)+?1 <= (SELECT CAST(value AS INTEGER) FROM policy WHERE key='catalog.network_limit')",[sql(length)?]).map_err(err)?;
        if changed != 1 {
            return Err("catalog_transfer_allowance".into());
        }
        Ok(())
    }
    pub(super) fn record_catalog_received(&self, length: u64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(err)?;
        if conn.execute("UPDATE policy SET value=CAST(value AS INTEGER)+?1 WHERE key='catalog.network_received' AND CAST(value AS INTEGER)+?1 <= (SELECT CAST(value AS INTEGER) FROM policy WHERE key='catalog.network_reserved')",params![sql(length)?]).map_err(err)?!=1 {return Err("catalog_transfer_accounting".into())}
        Ok(())
    }
}

impl DiskPackStore {
    pub(crate) fn reserve_metadata(&self, extra: u64) -> Result<(), String> {
        let budget = self.manifest()?.policy.metadata_budget_bytes;
        loop {
            let page_bytes = directory_bytes(&self.root.join("pages"))?;
            let db = file_bytes(&self.root.join("pack-index.sqlite"));
            // Reserve a full rollback journal plus bounded manifest staging.
            let needed = page_bytes
                .checked_add(db.checked_mul(2).ok_or("integer_overflow")?)
                .and_then(|n| n.checked_add(extra))
                .and_then(|n| n.checked_add(131072))
                .ok_or("integer_overflow")?;
            if needed <= budget {
                let max_pages = (budget - page_bytes - extra - 131072) / 2 / 4096;
                self.conn
                    .lock()
                    .map_err(err)?
                    .pragma_update(None, "max_page_count", max_pages)
                    .map_err(err)?;
                return Ok(());
            }
            let conn = self.conn.lock().map_err(err)?;
            let victim:Option<i64>=conn.query_row("SELECT page_index FROM pages WHERE NOT EXISTS(SELECT 1 FROM policy WHERE key='pin.page.'||page_index) ORDER BY COALESCE((SELECT CAST(value AS INTEGER) FROM policy WHERE key='lru.page.'||page_index),0),page_index LIMIT 1",[],|r|r.get(0)).optional().map_err(err)?;
            let Some(victim) = victim else {
                return Err("budget_exceeded: pinned metadata partition".into());
            };
            std::fs::remove_file(self.root.join("pages").join(victim.to_string())).map_err(err)?;
            conn.execute("DELETE FROM pages WHERE page_index=?1", [victim])
                .map_err(err)?;
            conn.execute("DELETE FROM policy WHERE key='lru.page.'||?1", [victim])
                .map_err(err)?;
        }
    }
}

pub(super) fn directory_bytes(path: &Path) -> Result<u64, String> {
    let mut n = 0u64;
    for item in std::fs::read_dir(path).map_err(err)? {
        let item = item.map_err(err)?;
        if item.file_type().map_err(err)?.is_file() {
            n = n
                .checked_add(item.metadata().map_err(err)?.len())
                .ok_or("integer_overflow")?;
        }
    }
    Ok(n)
}
