use super::*;
const PLAN_MAX: u64 = 65536;
pub(super) fn path(state: &AppState, record: &DynDonPlanRecord) -> Result<PathBuf, String> {
    if record.store_relative_path != format!("packs/{}", record.pack_version_id)
        || !record
            .pack_version_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
    {
        return Err("invalid_plan_path".into());
    }
    let conn = state.conn.lock().map_err(err_str)?;
    let root = storage_root_by_id(&conn, &record.destination_storage_root_id)
        .map_err(err_str)?
        .ok_or("storage_root_missing")?;
    Ok(std::fs::canonicalize(root.platform_locator)
        .map_err(err_str)?
        .join(&record.store_relative_path))
}
pub(super) async fn save(state: &AppState, plan: &SavedPlan) -> Result<(), String> {
    let bytes = serde_json::to_vec(plan).map_err(err_str)?;
    if bytes.len() as u64 > PLAN_MAX {
        return Err("plan_too_large".into());
    }
    let directory = state.app_data_dir.join("dyndon-plans");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(err_str)?;
    let root = path(state, &plan.record)?;
    for final_path in [
        root.join("install-plan.json"),
        directory.join(format!("{}.json", plan.record.install_plan_id)),
    ] {
        let partial = final_path.with_extension("json.part");
        tokio::fs::write(&partial, &bytes).await.map_err(err_str)?;
        tokio::fs::rename(&partial, &final_path)
            .await
            .map_err(err_str)?;
    }
    Ok(())
}
pub(super) async fn load(state: &AppState, id: &str) -> Result<SavedPlan, String> {
    if id.len() > 100 || !id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') {
        return Err("invalid_plan_id".into());
    }
    let filename = state
        .app_data_dir
        .join("dyndon-plans")
        .join(format!("{id}.json"));
    if tokio::fs::metadata(&filename).await.map_err(err_str)?.len() > PLAN_MAX {
        return Err("plan_too_large".into());
    }
    let bytes = tokio::fs::read(filename).await.map_err(err_str)?;
    let plan: SavedPlan = serde_json::from_slice(&bytes).map_err(err_str)?;
    if plan.record.install_plan_id != id {
        return Err("plan_identity_mismatch".into());
    }
    path(state, &plan.record)?;
    Ok(plan)
}
