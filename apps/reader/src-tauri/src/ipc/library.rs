use super::*;
#[tauri::command]
pub async fn storage_roots_list(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<StorageDestinationDto>, String> {
    let conn = state.conn.clone();
    let app_data_dir = state.app_data_dir.clone();
    let rows = tokio::task::spawn_blocking(move || -> Result<Vec<StorageRootRow>, String> {
        let g = conn.lock().map_err(err_str)?;
        let existing = all_storage_roots(&g).map_err(err_str)?;
        if !existing.is_empty() {
            return Ok(existing);
        }
        // Seed the M1 default root. Stable id derived from the path so a
        // re-seed after a wipe is idempotent.
        let locator = app_data_dir
            .to_str()
            .ok_or_else(|| "internal: app_data_dir is not valid UTF-8".to_string())?
            .to_string();
        let storage_root_id = format!("sr-{}", storage_sqlite::sha256_hex(locator.as_bytes()));
        let free_bytes = probe_free_bytes(&app_data_dir);
        let now = now_ms();
        // `kind = 'app_private'` is the §6.2 CHECK value matching a
        // platform-managed private dir; the synthesized root is the closest
        // M1 analog of an app-private storage root.
        g.execute(
            "INSERT OR IGNORE INTO storage_roots (\n\
             \x20\x20        storage_root_id, kind, platform_locator, display_name,\n\
             \x20\x20        writable, capacity_bytes, free_bytes,\n\
             \x20\x20        last_probed_at_ms, created_at_ms, updated_at_ms\n\
             \x20\x20      ) VALUES (?1, 'app_private', ?2, ?3, 1, NULL, ?4, ?5, ?6, ?6)",
            params![
                storage_root_id,
                locator,
                app_data_dir
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("EnZIME Data")
                    .to_string(),
                free_bytes,
                now,
                now,
            ],
        )
        .map_err(err_str)?;
        all_storage_roots(&g).map_err(err_str)
    })
    .await
    .map_err(err_str)??;

    Ok(rows
        .into_iter()
        .filter(|r| r.writable)
        .map(|r| StorageDestinationDto {
            storage_root_id: r.storage_root_id,
            display_name: r.display_name,
            writable: r.writable,
        })
        .collect())
}

/// §14.4 `pack_list({ offset?, limit? })`. Pages installed packs. Article
/// count, byte size, main-page key, and last-opened timestamp are recovered
/// from the latest persisted selection revision (`canonical_json`) so the
/// Library view reflects the DynDon subset actually on device.
#[tauri::command]
pub async fn pack_list(
    state: tauri::State<'_, AppState>,
    offset: Option<u32>,
    limit: Option<u32>,
) -> Result<PageResultDto<LibraryPackManifest>, String> {
    let offset = offset.unwrap_or(0);
    let limit = limit.unwrap_or(50).clamp(1, 200);
    let conn = state.conn.clone();
    let mut items = tokio::task::spawn_blocking(move || -> Result<Vec<LibraryPackManifest>, String> {
        let g = conn.lock().map_err(err_str)?;
        let pack_rows = storage_sqlite::query::recent_packs(&g, limit as i64)
            .map_err(err_str)?;
        let mut out = Vec::with_capacity(pack_rows.len());
        for row in pack_rows {
            let article_count=0;
            let size_bytes=g.query_row("SELECT COALESCE(SUM(byte_length),0) FROM pack_locations WHERE pack_version_id=?1 AND locator_kind='active'",[&row.active_pack_version_id],|r|r.get::<_,u64>(0)).map_err(err_str)?;
            let main_page_key=None;
            let last_opened_at = last_opened_ms(&g, &row.pack_id)
                .map_err(err_str)?
                .map(|ms| format_iso(ms));
            out.push(LibraryPackManifest {
                pack_id: row.pack_id,
                display_name: row.display_name,
                description: row.description,
                language_bcp47: row.language_bcp47,
                article_count,
                size_bytes,
                last_opened_at,
                main_page_key,
            });
        }
        Ok(out)
    })
    .await
    .map_err(err_str)??;

    {
        let archives = state.archives.lock().map_err(err_str)?;
        for item in &mut items {
            if let Some(archive) = archives.get(&item.pack_id) {
                item.main_page_key = archive
                    .metadata()
                    .main_page
                    .as_ref()
                    .map(|key| key.0.clone());
            }
        }
    }
    // Query durable availability outside the application DB lock. A cached
    // archive descriptor may predate payload eviction and is not count truth.
    let active_versions = {
        let conn = state.conn.lock().map_err(err_str)?;
        items
            .iter()
            .map(|item| {
                let version: Option<String> = conn
                    .query_row(
                        "SELECT active_pack_version_id FROM packs WHERE pack_id=?1",
                        [&item.pack_id],
                        |row| row.get(0),
                    )
                    .map_err(err_str)?;
                Ok((item.pack_id.clone(), version))
            })
            .collect::<Result<std::collections::HashMap<_, _>, String>>()?
    };
    let stores = state
        .dyndon_stores
        .lock()
        .map_err(err_str)?
        .values()
        .cloned()
        .collect::<Vec<_>>();
    for store in stores {
        let manifest = store.manifest()?;
        if active_versions
            .get(&manifest.pack_id)
            .and_then(|id| id.as_deref())
            == Some(manifest.pack_version_id.as_str())
        {
            if let Some(item) = items
                .iter_mut()
                .find(|item| item.pack_id == manifest.pack_id)
            {
                item.article_count = store.catalog_progress()?.readable_articles;
            }
        }
    }
    let total = items.len() as u64;
    let next_offset = if items.len() as u32 >= limit {
        Some(offset + limit)
    } else {
        None
    };
    Ok(PageResultDto {
        items,
        total: Some(total),
        next_offset,
    })
}

/// §14.4 `catalog_search({ query, languageBcp47?, category?, offset?, limit? })`.
#[tauri::command]
pub async fn catalog_search(
    state: tauri::State<'_, AppState>,
    query: String,
    language_bcp47: Option<String>,
    category: Option<String>,
    offset: Option<u32>,
    limit: Option<u32>,
) -> Result<kiwix::CatalogPage, String> {
    let offset = offset.unwrap_or(0);
    let limit = limit.unwrap_or(20).clamp(1, 100);
    let page =
        kiwix::search_kiwix_catalog(&state, &query, language_bcp47.as_deref(), offset, limit)
            .await?;
    if let Some(category) = category.filter(|v| !v.trim().is_empty()) {
        let language = kiwix::language_code(language_bcp47.as_deref());
        let mut cached = kiwix::cached_catalog(
            &state,
            &query,
            language.as_deref(),
            Some(&category),
            offset,
            limit,
        )?;
        cached.notice =
            Some("Category filter applies to cached Kiwix results; total is unknown.".into());
        Ok(cached)
    } else {
        Ok(page)
    }
}

#[tauri::command]
pub async fn install_plan(
    state: tauri::State<'_, AppState>,
    catalog_entry_id: String,
    destination_storage_root_id: String,
    selection_profile_id: Option<String>,
    budget_bytes: Option<u64>,
) -> Result<InstallPlanDto, String> {
    dyndon::prepare_dyndon_install(
        &state,
        dyndon::DynDonInstallRequest {
            catalog_entry_id,
            destination_storage_root_id,
            budget_bytes: budget_bytes.unwrap_or(0),
            selection_profile_id,
        },
    )
    .await
}
#[tauri::command]
pub async fn install_start(
    state: tauri::State<'_, AppState>,
    install_plan_id: String,
) -> Result<DownloadJobDto, String> {
    dyndon::run_dyndon_install(&state, &install_plan_id).await
}

/// §14.4 `job_list({ offset?, limit? })`.
#[tauri::command]
pub async fn job_list(
    state: tauri::State<'_, AppState>,
    offset: Option<u32>,
    limit: Option<u32>,
) -> Result<PageResultDto<DownloadJobDto>, String> {
    let offset = offset.unwrap_or(0);
    let limit = limit.unwrap_or(50).clamp(1, 200);
    let conn = state.conn.clone();
    let rows = tokio::task::spawn_blocking(move || -> Result<Vec<DownloadJobRow>, String> {
        let g = conn.lock().map_err(err_str)?;
        page_download_jobs(&g, limit as i64, offset as i64).map_err(err_str)
    })
    .await
    .map_err(err_str)??;

    let total = rows.len() as u64;
    let items: Vec<DownloadJobDto> = rows
        .into_iter()
        .map(|r| DownloadJobDto {
            download_job_id: r.download_job_id,
            state: r.state,
            bytes_verified: r.bytes_verified.max(0) as u64,
            bytes_retained: r.bytes_retained.max(0) as u64,
            transfer_scope: r.transfer_scope,
            error_detail: r.error_detail,
            expected_size_bytes: r.expected_size_bytes.max(0) as u64,
        })
        .collect();
    let next_offset = if items.len() as u32 >= limit {
        Some(offset + limit)
    } else {
        None
    };
    Ok(PageResultDto {
        items,
        total: Some(total),
        next_offset,
    })
}

/// Format a unix-millis timestamp as an RFC 3339 string (the wire shape the
/// frontend expects for `lastOpenedAt`). Tiny chrono-free implementation so
/// the reader crate does not pull in `chrono` just for this one field.
pub(super) fn format_iso(ms: i64) -> String {
    let secs = (ms / 1000).max(0) as i64;
    let days = secs.div_euclid(86_400);
    let tod = secs.rem_euclid(86_400) as u32;
    let hour = tod / 3_600;
    let min = (tod % 3_600) / 60;
    let sec = tod % 60;
    let (year, month, day) = civil_from_days(days);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hour, min, sec
    )
}

/// Howard Hinnant's days-from-epoch → civil date algorithm.
pub(super) fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as i64; // [0, 146096]
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) as i64; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = if mp < 10 {
        mp as u32 + 3
    } else {
        mp as u32 - 9
    }; // [1, 12]
    let year = if m <= 2 { y + 1 } else { y };
    (year, m, d)
}

#[tauri::command]
pub async fn install_cancel(
    state: tauri::State<'_, AppState>,
    install_plan_id: String,
) -> Result<(), String> {
    dyndon::cancel_dyndon_install(&state, &install_plan_id)
}
