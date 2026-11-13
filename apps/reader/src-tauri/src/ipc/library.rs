use super::*;
use crate::shared_storage::StorageMedium;

/// §14.4 `shared_storage_request_access`. Android opens the system's shared
/// storage permission screen through the mobile plugin; desktop needs no
/// access prompt and succeeds immediately.
#[tauri::command]
pub async fn shared_storage_request_access() -> Result<(), String> {
    crate::shared_storage::request_access().await
}

/// §14.4 `storage_roots_list`. Enumerates only the real shared roots under
/// this build's compiled shared-storage policy (internal common root and
/// actually mounted SD cards). Namespace directories are created first, then
/// canonical paths are deduplicated — so a card discovered through both the
/// compiled configuration and volume enumeration yields exactly one row.
/// Writability is proven on the real cross-process `.acquisition.lock` file
/// (create/write, no truncation, no operation lock) before `writable=true` is
/// returned; an unwritable card root is omitted and an inaccessible primary
/// root surfaces an actionable error. A card's mount point is never created,
/// and historical `storage_roots` rows are never returned or re-seeded. On
/// Android a missing shared-file access grant surfaces the actionable
/// `shared_storage_access_required` error — catalog browsing is unaffected.
#[tauri::command]
pub async fn storage_roots_list(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<StorageDestinationDto>, String> {
    let roots = state.shared_storage.roots().await?;
    let conn = state.conn.clone();
    let rows = tokio::task::spawn_blocking(
        move || -> Result<Vec<StorageDestinationDto>, String> {
            let g = conn.lock().map_err(err_str)?;
            let mut out = Vec::with_capacity(roots.len());
            // Deduplicate canonical paths AFTER creating the namespace
            // directories: a fresh card canonicalizes to nothing until its
            // namespace folder exists, so pre-creation deduplication would
            // return duplicate rows for one physical medium.
            let mut seen: Vec<std::path::PathBuf> = Vec::new();
            for root in roots {
                // The primary common directory is created when missing. A
                // card root appears only when its mount parent already
                // exists (guaranteed by `roots()`); a card that cannot hold
                // its namespace directory is simply absent, never faked.
                if let Err(e) = std::fs::create_dir_all(&root.path) {
                    if root.medium == StorageMedium::Sdcard {
                        continue;
                    }
                    return Err(format!(
                        "shared_storage_unavailable: cannot create the shared storage location \
                         '{}' ({e}). Check the storage medium and try again.",
                        root.path.display()
                    ));
                }
                let canonical = std::fs::canonicalize(&root.path).map_err(err_str)?;
                if seen.contains(&canonical) {
                    // Same medium via configuration and enumeration.
                    continue;
                }
                seen.push(canonical.clone());
                // Creating the directory and having free space does not
                // establish write access on an existing read-only directory.
                // Prove it on the real `.acquisition.lock`: open for
                // create+write without truncating it and without taking the
                // operation lock, then close immediately.
                let lock_probe = std::fs::OpenOptions::new()
                    .create(true)
                    .truncate(false)
                    .write(true)
                    .read(true)
                    .open(canonical.join(".acquisition.lock"));
                if let Err(e) = lock_probe {
                    if root.medium == StorageMedium::Sdcard {
                        // An unwritable card is omitted, never advertised.
                        continue;
                    }
                    return Err(format!(
                        "shared_storage_unavailable: the shared storage location '{}' is not \
                         writable ({e}). Allow shared storage access for EnZIME, verify the \
                         medium is mounted read-write, then try again.",
                        canonical.display()
                    ));
                }
                let locator = canonical
                    .to_str()
                    .ok_or_else(|| {
                        "shared_storage_unavailable: shared storage path is not valid UTF-8"
                            .to_string()
                    })?
                    .to_string();
                let storage_root_id =
                    format!("sr-{}", storage_sqlite::sha256_hex(locator.as_bytes()));
                let free_bytes = fs2::available_space(&canonical).map_err(err_str)?;
                let available_bytes =
                    zim_cluster_cache::disk::DiskPackPolicy::available_bytes(free_bytes);
                let now = now_ms();
                let display_name = root.medium.label().to_string();
                g.execute(
                    "INSERT INTO storage_roots (\n\
                     \x20\x20        storage_root_id, kind, platform_locator, display_name,\n\
                     \x20\x20        writable, capacity_bytes, free_bytes,\n\
                     \x20\x20        last_probed_at_ms, created_at_ms, updated_at_ms\n\
                     \x20\x20      ) VALUES (?1, ?2, ?3, ?4, 1, NULL, ?5, ?6, ?6, ?6)\n\
                     \x20\x20      ON CONFLICT(storage_root_id) DO UPDATE SET\n\
                     \x20\x20        kind=excluded.kind, platform_locator=excluded.platform_locator,\n\
                     \x20\x20        display_name=excluded.display_name, writable=1,\n\
                     \x20\x20        free_bytes=excluded.free_bytes,\n\
                     \x20\x20        last_probed_at_ms=excluded.last_probed_at_ms,\n\
                     \x20\x20        updated_at_ms=excluded.updated_at_ms",
                    params![
                        storage_root_id,
                        root.medium.root_kind(),
                        locator,
                        display_name,
                        free_bytes as i64,
                        now,
                    ],
                )
                .map_err(err_str)?;
                out.push(StorageDestinationDto {
                    storage_root_id,
                    display_name,
                    writable: true,
                    free_bytes,
                    available_bytes,
                    // Every destination this build returns is a shared
                    // common root; sharing is never optional.
                    shared: true,
                    medium: root.medium,
                });
            }
            Ok(out)
        },
    )
    .await
    .map_err(err_str)??;
    Ok(rows)
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
            Some("Category filter applies to cached catalog results; total is unknown.".into());
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
    cut_strategy: Option<zim_cluster_cache::disk::CutStrategy>,
    preferred_domain: Option<String>,
) -> Result<InstallPlanDto, String> {
    tokio::time::timeout(std::time::Duration::from_secs(180), dyndon::prepare_dyndon_install(
        &state,
        dyndon::DynDonInstallRequest {
            catalog_entry_id,
            destination_storage_root_id,
            budget_bytes: budget_bytes.unwrap_or(0),
            selection_profile_id,
            cut_strategy: cut_strategy.unwrap_or_default(),
            preferred_domain,
        },
    ))
    .await.map_err(|_| "Planning timed out while inspecting the source. Retry or choose a smaller archive.".to_string())?
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
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
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
