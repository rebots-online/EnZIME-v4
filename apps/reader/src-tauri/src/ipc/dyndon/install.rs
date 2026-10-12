use super::*;
pub async fn run_dyndon_install(
    state: &AppState,
    install_plan_id: &str,
) -> Result<DownloadJobDto, String> {
    let _gate = state.acquisition_gate.lock().await;
    let plan = persistence::load(state, install_plan_id).await?;
    let store_path = persistence::path(state, &plan.record)?;
    let existing = state
        .dyndon_stores
        .lock()
        .map_err(err_str)?
        .get(&store_path)
        .cloned();
    let store = if let Some(store) = existing {
        store
    } else {
        let store =
            DiskPackStore::reopen(store_path.clone(), Some(state.transfer_source.clone())).await?;
        state
            .dyndon_stores
            .lock()
            .map_err(err_str)?
            .insert(store_path.clone(), store.clone());
        store
    };
    let manifest = store.manifest()?;
    if manifest.pack_id != plan.record.pack_id
        || manifest.pack_version_id != plan.record.pack_version_id
    {
        return Err("manifest_identity_mismatch".into());
    }
    if plan.offline_catalog.is_none() {
        return Err("offline_catalog_upgrade_required: compute a new install plan".into());
    }
    let current_summary = store.planned_summary()?;
    if current_summary.selected_count != plan.record.summary.selected_count
        || current_summary.selected_bytes != plan.record.summary.selected_bytes
    {
        return Err("offline_catalog_plan_changed: compute a new install plan".into());
    }
    store.catalog_planned_progress()?;
    let job_id = {
        let conn = state.conn.lock().map_err(err_str)?;
        let prefix = format!("job-{install_plan_id}");
        let previous:Option<(String,String)>=conn.query_row("SELECT download_job_id,state FROM download_jobs WHERE download_job_id=?1 OR download_job_id LIKE ?2 ORDER BY created_at_ms DESC,download_job_id DESC LIMIT 1",params![prefix,format!("{prefix}-retry-%")],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(err_str)?;
        match previous {
            Some((id, status)) if status != "cancelled" => id,
            Some(_) => format!("{prefix}-retry-{}", ulid::Ulid::new()),
            None => prefix,
        }
    };
    let prior = {
        let conn = state.conn.lock().map_err(err_str)?;
        conn.execute("INSERT OR IGNORE INTO download_jobs(download_job_id,catalog_entry_id,source_uri,destination_storage_root_id,expected_size_bytes,expected_sha256,state,bytes_verified,etag,last_modified,created_at_ms,updated_at_ms,transfer_scope) VALUES(?1,?2,?3,?4,?5,?6,'planned',0,?7,?8,?9,?9,'partial')",params![job_id,plan.catalog_entry_id,manifest.identity.uri,plan.record.destination_storage_root_id,manifest.identity.size_bytes as i64,plan.expected_sha256,manifest.identity.etag,manifest.identity.last_modified,now_ms()]).map_err(err_str)?;
        conn.query_row(
            "SELECT state FROM download_jobs WHERE download_job_id=?1",
            [&job_id],
            |r| r.get::<_, String>(0),
        )
        .map_err(err_str)?
    };
    if prior == "completed" {
        register(state, store, store_path).await?;
        return job(state, &job_id);
    }
    if prior == "cancelled" {
        return Err("download_cancelled".into());
    }
    // Re-enter only legal durable transitions after interruption.
    if prior == "fetching" || prior == "verifying" || prior == "activating" {
        transition(state, &job_id, "failed")?;
    }
    if prior != "staged" {
        transition(state, &job_id, "fetching")?;
        store.set_offline(false).await?;
    }
    let outcome=async {
  let install=install_selected(store.clone());tokio::pin!(install);
  let mut ticks=tokio::time::interval(std::time::Duration::from_millis(100));
  let summary=loop {tokio::select!{ result=&mut install=>break result?,_ = ticks.tick()=>{
    let conn=state.conn.lock().map_err(err_str)?;let status:String=conn.query_row("SELECT state FROM download_jobs WHERE download_job_id=?1",[&job_id],|r|r.get(0)).map_err(err_str)?;
    if status=="cancelled"{return Err("download_cancelled".to_string())}
    conn.execute("UPDATE download_jobs SET bytes_retained=?2,updated_at_ms=?3 WHERE download_job_id=?1",params![job_id,store.retained_bytes()? as i64,now_ms()]).map_err(err_str)?;
  }}};
  if job(state,&job_id)?.state=="cancelled"{return Err("download_cancelled".to_string())}
  {let conn=state.conn.lock().map_err(err_str)?;conn.execute("UPDATE download_jobs SET bytes_retained=?2,updated_at_ms=?3 WHERE download_job_id=?1",params![job_id,summary.selected_bytes as i64,now_ms()]).map_err(err_str)?;}
  // Main-page decoding is an actual readability check; it does not verify the full artifact digest.
  let archive=open_disk_archive(store.clone()).await?;
  let main=archive.main_page().await.map_err(err_str)?.ok_or("main_page_missing")?;
  archive.article(&main).await.map_err(err_str)?;
  let available = store.catalog_progress()?;
  if available.readable_articles == 0 {
      return Err("offline_catalog_main_unavailable".to_string());
  }
  if prior!="staged" {transition(state,&job_id,"verifying")?;transition(state,&job_id,"staged")?;}
  transition(state,&job_id,"activating")?;
  activate(state,&plan,&manifest,summary.selected_bytes,&job_id)?;
  state.archives.lock().map_err(err_str)?.insert(plan.record.pack_id.clone(),Arc::new(zim_anzimmerman::RangeArchiveAdapter::new(archive)));
  state.dyndon_stores.lock().map_err(err_str)?.insert(store_path,store);
  job(state,&job_id)
 }.await;
    if let Err(error) = &outcome {
        let conn = state.conn.lock().map_err(err_str)?;
        let _=conn.execute("UPDATE download_jobs SET state='failed',error_code='dyndon_install',error_detail=?2,updated_at_ms=?3 WHERE download_job_id=?1 AND state IN ('fetching','verifying','activating')",params![job_id,error,now_ms()]);
    }
    outcome
}
fn transition(state: &AppState, id: &str, next: &str) -> Result<(), String> {
    let conn = state.conn.lock().map_err(err_str)?;
    storage_sqlite::write::set_download_job_state(&conn, id, next, now_ms()).map_err(err_str)?;
    Ok(())
}
fn job(state: &AppState, id: &str) -> Result<DownloadJobDto, String> {
    let conn = state.conn.lock().map_err(err_str)?;
    conn.query_row("SELECT state,bytes_verified,bytes_retained,expected_size_bytes,transfer_scope,error_detail FROM download_jobs WHERE download_job_id=?1",[id],|r|Ok(DownloadJobDto{download_job_id:id.into(),state:r.get(0)?,bytes_verified:r.get(1)?,bytes_retained:r.get(2)?,expected_size_bytes:r.get(3)?,transfer_scope:r.get(4)?,error_detail:r.get(5)?})).map_err(err_str)
}
fn activate(
    state: &AppState,
    plan: &SavedPlan,
    manifest: &DiskPackManifest,
    retained: u64,
    job_id: &str,
) -> Result<(), String> {
    let bytes = serde_json::to_vec(plan).map_err(err_str)?;
    let identity = digest(&serde_json::to_vec(&manifest.identity).map_err(err_str)?);
    let mut conn = state.conn.lock().map_err(err_str)?;
    let tx = conn.transaction().map_err(err_str)?;
    let catalog = catalog_entry_by_id(&tx, &plan.catalog_entry_id)
        .map_err(err_str)?
        .ok_or("catalog_entry_missing")?;
    insert_or_ignore_pack(
        &tx,
        &plan.record.pack_id,
        &catalog.corpus_id,
        &catalog.title,
        &catalog.description,
        &catalog.language_bcp47,
        "tailored_zim",
        now_ms(),
        now_ms(),
    )
    .map_err(err_str)?;
    tx.execute("INSERT INTO pack_versions(pack_version_id,pack_id,upstream_version,edition_manifest_sha256,artifact_sha256,artifact_size_bytes,format,verification_state,installed_at_ms,created_at_ms,source_identity_sha256,verification_scope) VALUES(?1,?2,?3,?4,?5,?6,'zim','source_pinned',?7,?7,?8,'partial') ON CONFLICT(pack_version_id) DO UPDATE SET edition_manifest_sha256=excluded.edition_manifest_sha256,installed_at_ms=excluded.installed_at_ms",params![plan.record.pack_version_id,plan.record.pack_id,plan.upstream_version,digest(&bytes),plan.expected_sha256,manifest.identity.size_bytes as i64,now_ms(),identity]).map_err(err_str)?;
    tx.execute("UPDATE pack_locations SET locator_kind='retained' WHERE locator_kind='active' AND pack_version_id IN(SELECT pack_version_id FROM pack_versions WHERE pack_id=?1)",[&plan.record.pack_id]).map_err(err_str)?;
    tx.execute("INSERT INTO pack_locations(pack_location_id,pack_version_id,storage_root_id,relative_path,locator_kind,byte_length,created_at_ms) VALUES(?1,?2,?3,?4,'active',?5,?6) ON CONFLICT(storage_root_id,relative_path) DO UPDATE SET locator_kind='active',byte_length=excluded.byte_length",params![format!("location-{}",plan.record.install_plan_id),plan.record.pack_version_id,plan.record.destination_storage_root_id,plan.record.store_relative_path,retained as i64,now_ms()]).map_err(err_str)?;
    activate_pack_version(
        &tx,
        &plan.record.pack_id,
        &plan.record.pack_version_id,
        now_ms(),
    )
    .map_err(err_str)?;
    tx.execute("UPDATE download_jobs SET state='completed',bytes_verified=0,bytes_retained=?2,error_code=NULL,error_detail=NULL,completed_at_ms=?3,updated_at_ms=?3 WHERE download_job_id=?1",params![job_id,retained as i64,now_ms()]).map_err(err_str)?;
    tx.commit().map_err(err_str)
}
async fn register(
    state: &AppState,
    store: Arc<DiskPackStore>,
    root: PathBuf,
) -> Result<(), String> {
    let manifest = store.manifest()?;
    let archive = open_disk_archive(store.clone()).await?;
    state.archives.lock().map_err(err_str)?.insert(
        manifest.pack_id.clone(),
        Arc::new(zim_anzimmerman::RangeArchiveAdapter::new(archive)),
    );
    state
        .dyndon_stores
        .lock()
        .map_err(err_str)?
        .insert(root, store);
    Ok(())
}
pub async fn restore_dyndon_archives(state: &AppState) -> Result<(), String> {
    let locations = {
        let conn = state.conn.lock().map_err(err_str)?;
        let mut query=conn.prepare("SELECT p.pack_id,v.pack_version_id,l.storage_root_id,l.relative_path,v.source_identity_sha256 FROM packs p JOIN pack_versions v ON v.pack_version_id=p.active_pack_version_id JOIN pack_locations l ON l.pack_version_id=v.pack_version_id WHERE v.verification_scope='partial' AND v.verification_state='source_pinned' AND l.locator_kind='active'").map_err(err_str)?;
        let rows = query
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                ))
            })
            .map_err(err_str)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(err_str)?
    };
    for (pack, version, root, relative, expected_identity) in locations {
        let result = async {
            let record = DynDonPlanRecord {
                install_plan_id: String::new(),
                pack_id: pack.clone(),
                pack_version_id: version.clone(),
                destination_storage_root_id: root,
                store_relative_path: relative,
                summary: PlanSummary {
                    selected_count: 0,
                    selected_bytes: 0,
                    skipped_count: 0,
                    metadata_reserved_bytes: 0,
                    workspace_reserved_bytes: 0,
                    disk_budget_bytes: 0,
                },
            };
            let store_path = persistence::path(state, &record)?;
            let store =
                DiskPackStore::reopen(store_path.clone(), Some(state.transfer_source.clone()))
                    .await?;
            store.set_offline(true).await?;
            let manifest = store.manifest()?;
            if manifest.pack_id != pack
                || manifest.pack_version_id != version
                || digest(&serde_json::to_vec(&manifest.identity).map_err(err_str)?)
                    != expected_identity
            {
                return Err("manifest_identity_mismatch".to_string());
            }
            register(state, store, store_path).await
        }
        .await;
        if let Err(error) = result {
            state
                .restore_errors
                .lock()
                .map_err(err_str)?
                .push(format!("{pack}: {error}"));
        }
    }
    Ok(())
}

pub fn cancel_dyndon_install(state: &AppState, install_plan_id: &str) -> Result<(), String> {
    if install_plan_id.len() > 100
        || !install_plan_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
    {
        return Err("invalid_plan_id".into());
    }
    let conn = state.conn.lock().map_err(err_str)?;
    let prefix = format!("job-{install_plan_id}");
    conn.execute("UPDATE download_jobs SET state='cancelled',updated_at_ms=?3 WHERE (download_job_id=?1 OR download_job_id LIKE ?2) AND state IN ('planned','fetching','paused','verifying','staged','failed')",params![prefix,format!("{prefix}-retry-%"),now_ms()]).map_err(err_str)?;
    Ok(())
}
