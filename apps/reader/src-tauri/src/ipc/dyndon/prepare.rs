use super::*;
use anzimmerman::{
    directory::RawEntry,
    model::ZimMetadata,
    range::{RandomAccessSource, RangeReadLimits, RemoteZimIndex},
    ArticleKey, ZimError,
};
use ports::transfer::TransferProbe;
struct ProbedRangeSource {
    source: Arc<dyn TransferSource>,
    probe: TransferProbe,
}
#[async_trait::async_trait]
impl RandomAccessSource for ProbedRangeSource {
    fn len(&self) -> u64 {
        self.probe.size_bytes
    }
    async fn read_range(&self, offset: u64, length: u64) -> Result<Bytes, ZimError> {
        if length > 65536 || offset.checked_add(length).is_none_or(|e| e > self.len()) {
            return Err(ZimError::AllocationLimit);
        }
        let fail = |e: String| ZimError::InvalidHeader(e);
        let mut stream = self
            .source
            .open_pinned_range(
                &self.probe.uri,
                TransferRange {
                    start_byte: offset,
                    end_byte_exclusive: offset + length,
                },
                &self.probe,
            )
            .await
            .map_err(|e| fail(e.to_string()))?;
        let mut bytes = Vec::with_capacity(length as usize);
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| fail(e.to_string()))?;
            if bytes.len() + chunk.len() > length as usize {
                return Err(fail("range_overflow".into()));
            }
            bytes.extend_from_slice(&chunk)
        }
        if bytes.len() != length as usize {
            return Err(fail("truncated_range".into()));
        }
        Ok(bytes.into())
    }
}
pub async fn prepare_dyndon_install(
    state: &AppState,
    request: DynDonInstallRequest,
) -> Result<InstallPlanDto, String> {
    if request.budget_bytes != 0 && request.budget_bytes < 16 * 1024 * 1024 {
        return Err(format!(
            "budget_exceeded: minimum storage budget is 16 MiB including index and decoding workspace; requested {} bytes",
            request.budget_bytes
        ));
    }
    let _gate = state.acquisition_gate.lock().await;
    let entry = super::super::kiwix::load_entry(state, &request.catalog_entry_id)?;
    let root = {
        let conn = state.conn.lock().map_err(err_str)?;
        storage_root_by_id(&conn, &request.destination_storage_root_id)
            .map_err(err_str)?
            .ok_or("storage_root_missing")?
    };
    if !root.writable {
        return Err("storage_root_read_only".into());
    }
    let free = fs2::available_space(&root.platform_locator).map_err(err_str)?;
    let reserve = (256 * 1024 * 1024u64).max(free / 20);
    let available = free.saturating_sub(reserve);
    let budget = if request.budget_bytes == 0 {
        (1024 * 1024 * 1024).min(available)
    } else {
        request.budget_bytes
    };
    if available < 16 * 1024 * 1024 {
        return Err(format!(
            "budget_exceeded: destination needs at least 16 MiB after safety reserve; available {available} bytes"
        ));
    }
    if budget > available {
        return Err(format!(
            "budget_exceeded: requested {budget}, available after reserve {available}"
        ));
    }
    let acquisition =
        catalog_openzim::kiwix::KiwixCatalogClient::new(state.transfer_source.clone())
            .resolve(&entry)
            .await
            .map_err(err_str)?;
    let mut last_error = "no usable range mirror".to_string();
    let mut chosen = None;
    for uri in acquisition.mirrors.iter().take(8) {
        match state
            .transfer_source
            .probe(TransferProbeRequest {
                uri: uri.clone(),
                maximum_response_bytes: 65536,
            })
            .await
        {
            Ok(probe)
                if probe.accepts_ranges
                    && (entry.acquisition_url.ends_with(".zim")
                        || probe.size_bytes == acquisition.size_bytes) =>
            {
                chosen = Some(probe);
                break;
            }
            Ok(_) => last_error = "mirror length mismatch or range support unavailable".into(),
            Err(e) => last_error = e.to_string(),
        }
    }
    let probe = chosen.ok_or(last_error)?;
    let index = RemoteZimIndex::open(
        Arc::new(ProbedRangeSource {
            source: state.transfer_source.clone(),
            probe: probe.clone(),
        }),
        RangeReadLimits::default(),
    )
    .await
    .map_err(err_str)?;
    let main = index
        .main_entry()
        .await
        .map_err(err_str)?
        .ok_or("main_page_missing")?;
    let main_cluster = match &main.raw {
        RawEntry::Article(a) => a.cluster_number,
        _ => return Err("main_page_is_not_article".into()),
    };
    let identity = SourceIdentity {
        uri: probe.uri.clone(),
        size_bytes: probe.size_bytes,
        etag: probe.etag.clone(),
        last_modified: probe.last_modified.clone(),
        zim_uuid: hex::encode(index.header().uuid),
    };
    let identity_hash = digest(&serde_json::to_vec(&identity).map_err(err_str)?);
    let pack_id = format!("pack-{}", digest(entry.id.as_bytes()));
    let intent = selection_intent(state, request.selection_profile_id.as_deref(), budget)?;
    let edition_hash =
        digest(&serde_json::to_vec(&(&identity_hash, &intent, budget)).map_err(err_str)?);
    let pack_version_id = format!("version-{edition_hash}");
    let main_page = main.key.0.clone();
    let metadata = ZimMetadata {
        uuid: identity.zim_uuid.clone(),
        title: entry.title.clone(),
        description: entry.description.clone(),
        language_bcp47: entry.language.clone(),
        creator: String::new(),
        publisher: entry.publisher.clone(),
        date: entry.updated.clone(),
        article_count: 0,
        media_count: 0,
        main_page: Some(ArticleKey(main_page.clone())),
        checksum: None,
    };
    let relative = format!("packs/{pack_version_id}");
    let store_root = std::fs::canonicalize(&root.platform_locator)
        .map_err(err_str)?
        .join(&relative);
    let policy = DiskPackPolicy {
        disk_budget_bytes: budget,
        metadata_budget_bytes: (64 * 1024 * 1024).min(budget / 8),
        workspace_budget_bytes: (512 * 1024 * 1024).min(budget / 4),
        offline: false,
    };
    let manifest = DiskPackManifest {
        format_version: 1,
        pack_id: pack_id.clone(),
        pack_version_id: pack_version_id.clone(),
        identity,
        policy,
        metadata,
        main_page: Some(main_page),
        state: "source_pinned".into(),
    };
    let existing = state
        .dyndon_stores
        .lock()
        .map_err(err_str)?
        .get(&store_root)
        .cloned();
    let store = if let Some(store) = existing {
        store
    } else if store_root.join("manifest.json").exists() {
        DiskPackStore::reopen(store_root.clone(), Some(state.transfer_source.clone())).await?
    } else {
        DiskPackStore::create(store_root.clone(), manifest, state.transfer_source.clone()).await?
    };
    let retained_manifest = store.manifest()?;
    if retained_manifest.pack_id != pack_id
        || retained_manifest.pack_version_id != pack_version_id
        || digest(&serde_json::to_vec(&retained_manifest.identity).map_err(err_str)?)
            != identity_hash
    {
        return Err("manifest_identity_mismatch".into());
    }
    store.set_offline(false).await?;
    state
        .dyndon_stores
        .lock()
        .map_err(err_str)?
        .insert(store_root, store.clone());
    let index = Arc::new(
        RemoteZimIndex::open(
            Arc::new(DiskRangeSource::new(store.clone())),
            RangeReadLimits::default(),
        )
        .await
        .map_err(err_str)?,
    );
    // Persist and pin the actual redirect target's metadata before planning sweeps.
    index.main_entry().await.map_err(err_str)?;

    plan_disk_fit(store.clone(), index.clone(), intent, &[main_cluster]).await?;
    discover_offline_entries(store.clone(), index).await?;
    let offline_catalog = store.catalog_planned_progress()?;
    let summary = store.planned_summary()?;
    let plan_id = format!(
        "plan-{}",
        digest(format!("{}:{}:{budget}", pack_version_id, root.storage_root_id).as_bytes())
    );
    let plan = SavedPlan {
        record: DynDonPlanRecord {
            install_plan_id: plan_id.clone(),
            pack_id,
            pack_version_id,
            store_relative_path: relative,
            destination_storage_root_id: root.storage_root_id.clone(),
            summary: summary.clone(),
        },
        catalog_entry_id: request.catalog_entry_id.clone(),
        expected_sha256: acquisition.sha256().map(str::to_owned),
        upstream_version: entry.updated,
        offline_catalog: Some(offline_catalog.clone()),
    };
    persistence::save(state, &plan).await?;
    let catalog = {
        let conn = state.conn.lock().map_err(err_str)?;
        conn.execute("UPDATE catalog_entries SET artifact_size_bytes=?2,artifact_sha256=?3 WHERE catalog_entry_id=?1",params![request.catalog_entry_id,probe.size_bytes as i64,plan.expected_sha256]).map_err(err_str)?;
        map_catalog_entry(
            catalog_entry_by_id(&conn, &request.catalog_entry_id)
                .map_err(err_str)?
                .ok_or("catalog_entry_missing")?,
        )
    };
    Ok(InstallPlanDto {
        install_plan_id: plan_id,
        catalog_entry: catalog,
        destination: StorageDestinationDto {
            storage_root_id: root.storage_root_id,
            display_name: root.display_name,
            writable: true,
        },
        transfer_mode: TransferModeDto::Ranged,
        download_bytes: summary.selected_bytes,
        temporary_bytes: summary.workspace_reserved_bytes,
        final_bytes: summary.selected_bytes + summary.metadata_reserved_bytes,
        free_bytes_before: free,
        free_bytes_after: free - budget,
        offline_catalog: Some(OfflineCatalogProgressDto::from(offline_catalog)),
        verification: InstallPlanVerificationDto {
            expected_size_bytes: probe.size_bytes,
            expected_sha256: plan.expected_sha256,
            signature_required: false,
            zim_structure_check: true,
        },
    })
}
fn selection_intent(
    state: &AppState,
    id: Option<&str>,
    budget: u64,
) -> Result<::dyndon::SelectionIntent, String> {
    let Some(id) = id else {
        return Ok(::dyndon::SelectionIntent {
            domains: vec![],
            breadth_limit: None,
            depth_limit: None,
            budget_bytes: budget,
        });
    };
    let conn = state.conn.lock().map_err(err_str)?;
    let json:String=conn.query_row("SELECT canonical_json FROM selection_profile_revisions WHERE profile_id=?1 ORDER BY revision DESC LIMIT 1",[id],|r|r.get(0)).map_err(err_str)?;
    let mut intent: ::dyndon::SelectionIntent = serde_json::from_str(&json).map_err(err_str)?;
    intent.budget_bytes = budget;
    Ok(intent)
}
