use super::*;
// ===========================================================================
// Existing commands (bootstrap / home / readiness)
// ===========================================================================

#[tauri::command]
pub async fn app_bootstrap(state: tauri::State<'_, AppState>) -> Result<AppBootstrap, String> {
    let conn = state.conn.clone();
    let (first_launch, schema_applied) = tokio::task::spawn_blocking(move || {
        let g = conn.lock().map_err(err_str)?;
        let setup_done = storage_sqlite::query::setup_completed(&g).map_err(err_str)?;
        let schema_applied =
            storage_sqlite::query::schema_version_one_applied(&g).map_err(err_str)?;
        Ok::<_, String>((!setup_done, schema_applied))
    })
    .await
    .map_err(err_str)??;

    let readiness = with_restore_errors(&state, build_readiness(schema_applied, now_ms()))?;
    Ok(AppBootstrap {
        schema_version: 2,
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        platform: platform_string(),
        first_launch,
        routes: M1_ROUTES
            .iter()
            .map(|(path, enabled)| RouteDescriptor {
                path: (*path).to_string(),
                enabled: *enabled,
            })
            .collect(),
        capabilities: Vec::new(),
        readiness,
    })
}

#[tauri::command]
pub async fn readiness_get(state: tauri::State<'_, AppState>) -> Result<SystemReadiness, String> {
    let conn = state.conn.clone();
    let schema_applied = tokio::task::spawn_blocking(move || {
        let g = conn.lock().map_err(err_str)?;
        storage_sqlite::query::schema_version_one_applied(&g).map_err(err_str)
    })
    .await
    .map_err(err_str)??;
    with_restore_errors(&state, build_readiness(schema_applied, now_ms()))
}

#[tauri::command]
pub async fn home_snapshot(state: tauri::State<'_, AppState>) -> Result<HomeSnapshot, String> {
    let conn = state.conn.clone();
    let snapshot = tokio::task::spawn_blocking(move || -> Result<HomeSnapshot, String> {
        let g = conn.lock().map_err(err_str)?;
        let recent_rows = storage_sqlite::query::recent_packs(&g, 12).map_err(err_str)?;
        let history_rows = storage_sqlite::query::recent_reader_history(&g, 1).map_err(err_str)?;
        let task_rows = storage_sqlite::query::open_tasks(&g).map_err(err_str)?;
        let schema_applied =
            storage_sqlite::query::schema_version_one_applied(&g).map_err(err_str)?;

        let recent_packs = recent_rows
            .into_iter()
            .map(|row| {
                let mut pack = map_pack(row);
                for version in &mut pack.versions {
                    version.verification_state = g
                        .query_row(
                            "SELECT verification_state FROM pack_versions WHERE pack_version_id=?1",
                            [&version.pack_version_id],
                            |r| r.get(0),
                        )
                        .map_err(err_str)?;
                }
                Ok(pack)
            })
            .collect::<Result<Vec<_>, String>>()?;
        let continue_reading = history_rows.into_iter().next().map(map_history);
        let task_inbox = task_rows.into_iter().map(map_task).collect();
        let readiness = build_readiness(schema_applied, now_ms());

        Ok(HomeSnapshot {
            recent_packs,
            continue_reading,
            task_inbox,
            // M1: no memory-inbox entries are modeled yet; the frontend binds
            // this as an array, so an empty vec is the truthful shape.
            memory_inbox: Vec::new(),
            readiness,
        })
    })
    .await
    .map_err(err_str)??;
    Ok(snapshot)
}

// ===========================================================================
// Reader commands — REPOINTED to installed packs (file-picker path killed)
// ===========================================================================

fn with_restore_errors(
    state: &AppState,
    mut readiness: SystemReadiness,
) -> Result<SystemReadiness, String> {
    let errors = state.restore_errors.lock().map_err(err_str)?;
    if !errors.is_empty() {
        readiness.overall = "warning".into();
        readiness.summary = "Some installed packs could not be reopened".into();
        readiness.flags.push(ReadinessFlag {
            id: "offline-packs".into(),
            level: "warning".into(),
            label: "Pack storage unavailable".into(),
            detail: errors.join("; "),
            setting_path: "/library".into(),
        });
    }
    Ok(readiness)
}
