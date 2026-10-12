use super::store::{err, sql};
use super::{DiskPackStore, DiskRangeSource};
use anzimmerman::range::{ClusterByteRange, RangeReadLimits, RemoteZimIndex};
use dyndon::{ClusterManifest, ClusterManifestEntry, SelectionIntent};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlanSummary {
    pub selected_count: u64,
    pub selected_bytes: u64,
    pub skipped_count: u64,
    pub metadata_reserved_bytes: u64,
    pub workspace_reserved_bytes: u64,
    pub disk_budget_bytes: u64,
}
fn summary(store: &DiskPackStore, skipped: u64) -> Result<PlanSummary, String> {
    let policy = store.manifest()?.policy;
    let (count, bytes) = store
        .conn
        .lock()
        .map_err(err)?
        .query_row(
            "SELECT COUNT(*),COALESCE(SUM(byte_length),0) FROM clusters WHERE selected=1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(err)?;
    Ok(PlanSummary {
        selected_count: count,
        selected_bytes: bytes,
        skipped_count: skipped,
        metadata_reserved_bytes: policy.metadata_budget_bytes,
        workspace_reserved_bytes: policy.workspace_budget_bytes,
        disk_budget_bytes: policy.disk_budget_bytes,
    })
}
pub async fn plan_disk_fit(
    store: Arc<DiskPackStore>,
    index: Arc<RemoteZimIndex>,
    intent: SelectionIntent,
    pinned_clusters: &[u32],
) -> Result<PlanSummary, String> {
    let _catalog = store.catalog_gate.lock().await;
    let count = index.header().cluster_count;
    let planning = RemoteZimIndex::open(
        Arc::new(DiskRangeSource::new_for_planning(store.clone())),
        RangeReadLimits::default(),
    )
    .await
    .map_err(err)?;
    if planning.header().uuid != index.header().uuid {
        return Err("source_changed".into());
    }
    store
        .conn
        .lock()
        .map_err(err)?
        .execute(
            "INSERT OR REPLACE INTO policy VALUES('plan.peak_batch','0')",
            [],
        )
        .map_err(err)?;
    for start in (0..count).step_by(4096) {
        let end = count.min(start.saturating_add(4096));
        let mut batch = Vec::with_capacity((end - start) as usize);
        for id in start..end {
            batch.push((id, planning.cluster_range(id).await.map_err(err)?));
        }
        let _metadata = store.metadata_gate.lock().await;
        store.reserve_metadata(128 * 1024)?;
        let mut conn = store.conn.lock().map_err(err)?;
        conn.execute(
            "UPDATE policy SET value=MAX(CAST(value AS INTEGER),?1) WHERE key='plan.peak_batch'",
            [batch.len() as u32],
        )
        .map_err(err)?;
        let tx = conn.transaction().map_err(err)?;
        for (id, range) in batch {
            tx.execute("INSERT INTO clusters(cluster_index,start_byte,byte_length) VALUES(?1,?2,?3) ON CONFLICT(cluster_index) DO UPDATE SET start_byte=excluded.start_byte,byte_length=excluded.byte_length",params![id,sql(range.start)?,sql(range.length)?]).map_err(err)?;
        }
        tx.commit().map_err(err)?;
    }
    let _metadata = store.metadata_gate.lock().await;
    select_candidates(&store, &intent, pinned_clusters)
}
/// Global grouping/filtering is performed on disk before bounded greedy batches.
fn select_candidates(
    store: &DiskPackStore,
    intent: &SelectionIntent,
    pinned: &[u32],
) -> Result<PlanSummary, String> {
    store.reserve_metadata(128 * 1024)?;
    let policy = store.manifest()?.policy;
    let budget = policy.payload_budget()?.min(intent.budget_bytes);
    let mut conn = store.conn.lock().map_err(err)?;
    stage_candidates(&conn, intent)?;
    conn.execute("UPDATE clusters SET selected=0", [])
        .map_err(err)?;
    for id in pinned {
        if conn
            .execute(
                "UPDATE clusters SET pinned=1,selected=1 WHERE cluster_index=?1",
                [id],
            )
            .map_err(err)?
            != 1
        {
            return Err("pinned_cluster_missing".into());
        }
    }
    // Preserve any previously pinned main-page payload across replanning.
    conn.execute("UPDATE clusters SET selected=1 WHERE pinned=1", [])
        .map_err(err)?;
    conn.execute("UPDATE plan_eligible SET pinned=1 WHERE EXISTS(SELECT 1 FROM clusters c WHERE c.cluster_index=plan_eligible.cluster_index AND c.pinned=1)", []).map_err(err)?;
    let mut used: u64 = conn
        .query_row(
            "SELECT COALESCE(SUM(byte_length),0) FROM clusters WHERE selected=1",
            [],
            |r| r.get(0),
        )
        .map_err(err)?;
    if used > budget {
        return Err("budget_exceeded: main-page cluster does not fit".into());
    }
    let mut offset = 0u64;
    let mut skipped = 0u64;
    loop {
        let batch = {
            let mut stmt = conn.prepare(CANDIDATE_SCAN).map_err(err)?;
            let rows = stmt
                .query_map([sql(offset)?], |r| {
                    Ok(ClusterManifestEntry {
                        cluster_index: r.get(0)?,
                        byte_offset: r.get(1)?,
                        byte_len: r.get(2)?,
                        domain: r.get(3)?,
                        topic: r.get(4)?,
                        priority: r.get(5)?,
                        article_count: r.get(6)?,
                    })
                })
                .map_err(err)?;
            rows.collect::<Result<Vec<_>, _>>().map_err(err)?
        };
        if batch.is_empty() {
            break;
        }
        offset = offset
            .checked_add(batch.len() as u64)
            .ok_or("integer_overflow")?;
        let total = batch.iter().try_fold(0u64, |n, e| {
            n.checked_add(e.byte_len).ok_or("integer_overflow")
        })?;
        let fit = dyndon::plan_fit(
            &ClusterManifest {
                pack_version_id: String::new(),
                cluster_count: batch.len() as u32,
                total_bytes: total,
                entries: batch,
            },
            &SelectionIntent {
                domains: vec![],
                breadth_limit: None,
                depth_limit: None,
                budget_bytes: budget - used,
            },
        );
        let tx = conn.transaction().map_err(err)?;
        for id in fit.selected {
            tx.execute(
                "UPDATE clusters SET selected=1 WHERE cluster_index=?1",
                [id],
            )
            .map_err(err)?;
        }
        tx.commit().map_err(err)?;
        used = used
            .checked_add(fit.total_bytes)
            .ok_or("integer_overflow")?;
        skipped = skipped
            .checked_add(fit.skipped_count as u64)
            .ok_or("integer_overflow")?;
    }
    conn.execute(
        "INSERT OR REPLACE INTO policy VALUES('plan.skipped',?1)",
        [skipped.to_string()],
    )
    .map_err(err)?;
    conn.execute_batch("DROP TABLE plan_eligible; DROP TABLE plan_domains; DROP TABLE plan_groups; DROP TABLE plan_priorities;")
        .map_err(err)?;
    drop(conn);
    summary(store, skipped)
}
impl DiskPackStore {
    pub fn planned_summary(&self) -> Result<PlanSummary, String> {
        let skipped = self
            .conn
            .lock()
            .map_err(err)?
            .query_row("SELECT COUNT(*) FROM clusters WHERE selected=0", [], |r| {
                r.get(0)
            })
            .map_err(err)?;
        summary(self, skipped)
    }
    /// Complete stored payload bytes, excluding metadata and decode workspace.
    /// This accounting does not certify the digest of the original full archive.
    pub fn retained_bytes(&self) -> Result<u64, String> {
        self.conn
            .lock()
            .map_err(err)?
            .query_row(
                "SELECT COALESCE(SUM(byte_length),0) FROM clusters WHERE state='complete'",
                [],
                |row| row.get(0),
            )
            .map_err(err)
    }
}
pub async fn install_selected(store: Arc<DiskPackStore>) -> Result<PlanSummary, String> {
    let _catalog = store.catalog_gate.lock().await;
    let bootstrap = RemoteZimIndex::open(
        Arc::new(DiskRangeSource::new(store.clone())),
        RangeReadLimits::default(),
    )
    .await
    .map_err(err)?;
    super::DiskEntryCatalog::new(store.clone(), bootstrap.header().clone())?
        .validate_planned_main()?;
    let mut after = -1i64;
    loop {
        let row = {
            let conn = store.conn.lock().map_err(err)?;
            use rusqlite::OptionalExtension;
            conn.query_row("SELECT cluster_index,start_byte,byte_length,pinned FROM clusters WHERE selected=1 AND cluster_index>?1 ORDER BY cluster_index LIMIT 1",[after],|r|Ok((r.get::<_,u32>(0)?,r.get::<_,u64>(1)?,r.get::<_,u64>(2)?,r.get::<_,bool>(3)?))).optional().map_err(err)?
        };
        let Some((id, start, length, pinned)) = row else {
            break;
        };
        store
            .ensure_cluster(id, ClusterByteRange { start, length }, pinned)
            .await?;
        after = i64::from(id);
    }
    let skipped = {
        let conn = store.conn.lock().map_err(err)?;
        conn.execute(
            "UPDATE jobs SET state='complete',error=NULL WHERE job_id='download'",
            [],
        )
        .map_err(err)?;
        conn.query_row(
            "SELECT value FROM policy WHERE key='plan.skipped'",
            [],
            |r| r.get::<_, String>(0),
        )
        .unwrap_or_else(|_| "0".into())
        .parse()
        .map_err(err)?
    };
    summary(&store, skipped)
}

// All sorting indices are built while the tables are empty. Their subsequent
// growth and rollback journals are charged to the main database quota.
const CANDIDATE_SCAN: &str = "SELECT cluster_index,start_byte,byte_length,domain,topic,priority,article_count FROM plan_eligible INDEXED BY plan_eligible_rank WHERE pinned=0 ORDER BY priority DESC,article_count DESC,cluster_index ASC LIMIT 4096 OFFSET ?1";
const GROUP_SCAN: &str = "SELECT domain_present,domain,topic_present,topic FROM plan_groups INDEXED BY plan_groups_rank ORDER BY score DESC,domain_present ASC,domain ASC,topic_present ASC,topic ASC LIMIT ?1";
const PRIORITY_SCAN: &str = "SELECT priority FROM plan_priorities ORDER BY priority DESC LIMIT ?1";
fn stage_candidates(conn: &rusqlite::Connection, intent: &SelectionIntent) -> Result<(), String> {
    let tx = conn.unchecked_transaction().map_err(err)?;
    let conn = &*tx;
    conn.execute_batch("DROP TABLE IF EXISTS plan_eligible; DROP TABLE IF EXISTS plan_groups;
        DROP TABLE IF EXISTS plan_domains; DROP TABLE IF EXISTS plan_priorities;
        CREATE TABLE plan_domains(domain TEXT PRIMARY KEY) WITHOUT ROWID;
        CREATE TABLE plan_priorities(priority INTEGER PRIMARY KEY,selected INTEGER NOT NULL DEFAULT 0);
        CREATE TABLE plan_groups(domain_present INTEGER,domain TEXT,topic_present INTEGER,topic TEXT,score INTEGER,selected INTEGER NOT NULL DEFAULT 0,PRIMARY KEY(domain_present,domain,topic_present,topic)) WITHOUT ROWID;
        CREATE INDEX plan_groups_rank ON plan_groups(score DESC,domain_present ASC,domain ASC,topic_present ASC,topic ASC);
        CREATE TABLE plan_eligible(cluster_index INTEGER PRIMARY KEY,start_byte INTEGER,byte_length INTEGER,domain TEXT,topic TEXT,priority INTEGER,article_count INTEGER,pinned INTEGER);
        CREATE INDEX plan_eligible_rank ON plan_eligible(priority DESC,article_count DESC,cluster_index ASC,start_byte,byte_length,domain,topic,pinned);").map_err(err)?;
    for domain in &intent.domains {
        conn.execute("INSERT OR IGNORE INTO plan_domains VALUES(?1)", [domain])
            .map_err(err)?;
    }
    let mut stmt=conn.prepare("SELECT cluster_index,start_byte,byte_length,domain,topic,priority,article_count,pinned FROM clusters WHERE ?1 OR domain IN(SELECT domain FROM plan_domains)").map_err(err)?;
    let mut rows = stmt.query([intent.domains.is_empty()]).map_err(err)?;
    while let Some(row) = rows.next().map_err(err)? {
        let domain: Option<String> = row.get(3).map_err(err)?;
        let topic: Option<String> = row.get(4).map_err(err)?;
        let priority: i64 = row.get(5).map_err(err)?;
        conn.execute(
            "INSERT INTO plan_eligible VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
            params![
                row.get::<_, i64>(0).map_err(err)?,
                row.get::<_, i64>(1).map_err(err)?,
                row.get::<_, i64>(2).map_err(err)?,
                domain,
                topic,
                priority,
                row.get::<_, i64>(6).map_err(err)?,
                row.get::<_, i64>(7).map_err(err)?
            ],
        )
        .map_err(err)?;
        conn.execute("INSERT INTO plan_groups(domain_present,domain,topic_present,topic,score) VALUES(?1,?2,?3,?4,?5) ON CONFLICT(domain_present,domain,topic_present,topic) DO UPDATE SET score=score+excluded.score",params![domain.is_some(),domain.as_deref().unwrap_or(""),topic.is_some(),topic.as_deref().unwrap_or(""),priority]).map_err(err)?;
    }
    drop(rows);
    drop(stmt);
    if let Some(n) = intent.breadth_limit {
        let mut stmt = conn.prepare(GROUP_SCAN).map_err(err)?;
        let mut rows = stmt.query([n]).map_err(err)?;
        while let Some(row) = rows.next().map_err(err)? {
            conn.execute("UPDATE plan_groups SET selected=1 WHERE domain_present=?1 AND domain=?2 AND topic_present=?3 AND topic=?4",params![row.get::<_,i64>(0).map_err(err)?,row.get::<_,String>(1).map_err(err)?,row.get::<_,i64>(2).map_err(err)?,row.get::<_,String>(3).map_err(err)?]).map_err(err)?;
        }
        drop(rows);
        drop(stmt);
        conn.execute("DELETE FROM plan_eligible WHERE NOT EXISTS(SELECT 1 FROM plan_groups g WHERE g.domain_present=(plan_eligible.domain IS NOT NULL) AND g.domain=COALESCE(plan_eligible.domain,'') AND g.topic_present=(plan_eligible.topic IS NOT NULL) AND g.topic=COALESCE(plan_eligible.topic,'') AND g.selected=1)",[]).map_err(err)?;
    }
    if let Some(n) = intent.depth_limit {
        let mut stmt = conn
            .prepare("SELECT priority FROM plan_eligible")
            .map_err(err)?;
        let mut rows = stmt.query([]).map_err(err)?;
        while let Some(row) = rows.next().map_err(err)? {
            conn.execute(
                "INSERT OR IGNORE INTO plan_priorities(priority) VALUES(?1)",
                [row.get::<_, i64>(0).map_err(err)?],
            )
            .map_err(err)?;
        }
        drop(rows);
        drop(stmt);
        let mut stmt = conn.prepare(PRIORITY_SCAN).map_err(err)?;
        let mut rows = stmt.query([n]).map_err(err)?;
        while let Some(row) = rows.next().map_err(err)? {
            conn.execute(
                "UPDATE plan_priorities SET selected=1 WHERE priority=?1",
                [row.get::<_, i64>(0).map_err(err)?],
            )
            .map_err(err)?;
        }
        drop(rows);
        drop(stmt);
        conn.execute("DELETE FROM plan_eligible WHERE NOT EXISTS(SELECT 1 FROM plan_priorities p WHERE p.priority=plan_eligible.priority AND p.selected=1)",[]).map_err(err)?;
    }
    tx.commit().map_err(err)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn production_ordering_scans_never_create_temporary_sort_trees() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE clusters(cluster_index INTEGER PRIMARY KEY,start_byte INTEGER,byte_length INTEGER,domain TEXT,topic TEXT,priority INTEGER,article_count INTEGER,pinned INTEGER);").unwrap();
        stage_candidates(
            &conn,
            &SelectionIntent {
                domains: vec![],
                breadth_limit: Some(3),
                depth_limit: Some(2),
                budget_bytes: 1,
            },
        )
        .unwrap();
        for query in [CANDIDATE_SCAN, GROUP_SCAN, PRIORITY_SCAN] {
            let mut stmt = conn
                .prepare(&format!("EXPLAIN QUERY PLAN {query}"))
                .unwrap();
            let plans: Vec<String> = stmt
                .query_map([1], |r| r.get(3))
                .unwrap()
                .collect::<Result<_, _>>()
                .unwrap();
            assert!(
                !plans.iter().any(|p| p.contains("TEMP B-TREE")),
                "{query}: {plans:?}"
            );
        }
    }
}
