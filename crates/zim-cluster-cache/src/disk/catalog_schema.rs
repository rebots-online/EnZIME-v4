use super::store::err;
pub(super) fn ensure_catalog_schema(conn: &rusqlite::Connection) -> Result<(), String> {
    conn.execute_batch("CREATE TABLE IF NOT EXISTS catalog_entries(entry_index INTEGER PRIMARY KEY,entry_key TEXT NOT NULL UNIQUE,raw_entry BLOB NOT NULL,mime_type TEXT,owner_cluster INTEGER NOT NULL);
        CREATE INDEX IF NOT EXISTS catalog_entries_owner ON catalog_entries(owner_cluster,entry_index);
        CREATE INDEX IF NOT EXISTS clusters_catalog_rank ON clusters(priority ASC,article_count ASC,cluster_index DESC);").map_err(err)
}
/// A durable complete marker is invalid when its payload disappeared or changed.
/// Walk by primary key, retaining one hash buffer and one row at a time.
pub(super) fn reconcile_payload(store: &super::DiskPackStore) -> Result<(), String> {
    use rusqlite::OptionalExtension;
    let mut after = -1i64;
    loop {
        let row=store.conn.lock().map_err(err)?.query_row("SELECT cluster_index,byte_length,stored_sha256 FROM clusters WHERE cluster_index>?1 AND state='complete' ORDER BY cluster_index LIMIT 1",[after],|r|Ok((r.get::<_,u32>(0)?,r.get::<_,u64>(1)?,r.get::<_,Option<String>>(2)?))).optional().map_err(err)?;
        let Some((id, length, hash)) = row else { break };
        after = i64::from(id);
        let path = store.root.join("clusters").join(format!("{id}.bin"));
        if super::store::file_bytes(&path) != length
            || super::store::digest_file(&path).ok() != hash
        {
            store
                .conn
                .lock()
                .map_err(err)?
                .execute(
                    "UPDATE clusters SET state='missing',stored_sha256=NULL WHERE cluster_index=?1",
                    [id],
                )
                .map_err(err)?;
        }
    }
    Ok(())
}
