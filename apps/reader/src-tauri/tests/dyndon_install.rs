use domain::{
    annotation::*,
    identity::{ArticleKey, EntryKey, PackId},
};
use enzime_reader_lib::ipc::{dyndon::*, kiwix::*};
use ports::annotation::AnnotationService;
use std::sync::{atomic::Ordering, Arc};
#[path = "dyndon_install/support.rs"]
mod support;
use support::*;
#[tokio::test]
async fn native_http_partial_install_restart_and_annotation() {
    let bytes = fixture();
    let full = bytes.len() as u64;
    let (source, server) = serve(bytes, false).await;
    let root = root();
    let state = make_state(&root, source.clone());
    let page = search_kiwix_catalog(&state, "", Some("en"), 0, 20)
        .await
        .unwrap();
    assert_eq!(page.source, "live");
    assert_eq!(page.total, Some(1));
    let id = page.items[0].catalog_entry_id.clone();
    assert_eq!(page.items[0].artifact_sha256, None);
    let request = DynDonInstallRequest {
        catalog_entry_id: id,
        destination_storage_root_id: "chosen".into(),
        budget_bytes: 16 * 1024 * 1024,
        selection_profile_id: None,
    };
    let plan = prepare_dyndon_install(&state, request.clone())
        .await
        .unwrap();
    assert!(plan.download_bytes < full);
    assert_eq!(plan.verification.expected_size_bytes, full);
    assert_eq!(plan.verification.expected_sha256, None);
    let job = run_dyndon_install(&state, &plan.install_plan_id)
        .await
        .unwrap();
    assert_eq!(job.state, "completed");
    assert_eq!(job.bytes_verified, 0);
    assert_eq!(job.transfer_scope, "partial");
    assert!(job.bytes_retained > 0 && job.bytes_retained < full);
    let same = run_dyndon_install(&state, &plan.install_plan_id)
        .await
        .unwrap();
    assert_eq!(same.download_job_id, job.download_job_id);
    let (pack, archive) = {
        let archives = state.archives.lock().unwrap();
        let (p, a) = archives.iter().next().unwrap();
        (p.clone(), a.clone())
    };
    let article = archive.article(&ArticleKey("A/Home".into())).await.unwrap();
    assert!(article.normalized_text.contains("Offline canvas article"));
    assert_eq!(
        archive
            .resource(&EntryKey("I/pixel.txt".into()))
            .await
            .unwrap()
            .bytes
            .as_ref(),
        b"pixel"
    );
    let note = state
        .annotations
        .create(AnnotationDraft {
            annotation_type: AnnotationType::Comment,
            selector: SelectorBundle {
                article: ArticleSelector {
                    pack_id: PackId(pack.clone()),
                    article_key: article.article_key.clone(),
                    source_revision_digest: None,
                },
                text_quote: None,
                text_position: None,
                dom_path: None,
                image_region: None,
                captured_context: "Offline canvas article".into(),
            },
            body_text: Some("Persistent typed annotation".into()),
            style: AnnotationStyle {
                color_token: "amber".into(),
                emphasis: "normal".into(),
                stroke_width: None,
            },
            notebook_id: None,
            tag_ids: vec![],
            sidecar_layer_id: annotations::LOCAL_LAYER_ID.into(),
        })
        .await
        .unwrap();
    {
        let conn = state.conn.lock().unwrap();
        let row:(Option<String>,String,String,String)=conn.query_row("SELECT artifact_sha256,source_identity_sha256,verification_scope,verification_state FROM pack_versions",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).unwrap();
        assert_eq!(row.0, None);
        assert_eq!(row.1.len(), 64);
        assert_eq!(row.2, "partial");
        assert_eq!(row.3, "source_pinned");
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM download_jobs", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }
    assert!(!root.join("app/packs").exists());
    let locations = std::fs::read_dir(root.join("chosen/packs"))
        .unwrap()
        .count();
    assert_eq!(locations, 1);
    drop(archive);
    drop(state);
    source.offline.store(true, Ordering::SeqCst);
    server.abort();
    let before = source.requests.load(Ordering::SeqCst);
    let restarted = make_state(&root, source.clone());
    restore_dyndon_archives(&restarted).await.unwrap();
    assert!(restarted.restore_errors.lock().unwrap().is_empty());
    let archive = restarted
        .archives
        .lock()
        .unwrap()
        .get(&pack)
        .unwrap()
        .clone();
    archive.article(&ArticleKey("A/Home".into())).await.unwrap();
    archive
        .resource(&EntryKey("I/pixel.txt".into()))
        .await
        .unwrap();
    assert_eq!(source.requests.load(Ordering::SeqCst), before);
    let notes = restarted
        .annotations
        .list_for_article(&PackId(pack), &ArticleKey("A/Home".into()))
        .await
        .unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].annotation_id, note.annotation_id);
    let cached = search_kiwix_catalog(&restarted, "", Some("eng"), 0, 20)
        .await
        .unwrap();
    assert_eq!(cached.source, "cache");
    assert_eq!(cached.total, None);
    assert_eq!(cached.items.len(), 1);
    drop(archive);
    drop(restarted);
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn failed_and_cancelled_downloads_resume_durable_plan_without_false_activation() {
    let (source, server) = serve(fixture(), false).await;
    let root = root();
    let state = make_state(&root, source.clone());
    let entry = search_kiwix_catalog(&state, "", None, 0, 20)
        .await
        .unwrap()
        .items
        .remove(0);
    let request = DynDonInstallRequest {
        catalog_entry_id: entry.catalog_entry_id,
        destination_storage_root_id: "chosen".into(),
        budget_bytes: 16 * 1024 * 1024,
        selection_profile_id: None,
    };
    for budget in [1, u64::MAX] {
        let mut invalid = request.clone();
        invalid.budget_bytes = budget;
        assert!(prepare_dyndon_install(&state, invalid)
            .await
            .unwrap_err()
            .contains("budget_exceeded"));
    }
    let plan = prepare_dyndon_install(&state, request).await.unwrap();
    source.fail_range.store(true, Ordering::SeqCst);
    assert!(run_dyndon_install(&state, &plan.install_plan_id)
        .await
        .is_err());
    {
        let conn = state.conn.lock().unwrap();
        assert_eq!(
            conn.query_row("SELECT state FROM download_jobs", [], |r| r
                .get::<_, String>(0))
                .unwrap(),
            "failed"
        );
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM packs", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            0
        );
    }
    source.delay_range.store(true, Ordering::SeqCst);
    let cancel = async {
        loop {
            let status = {
                let conn = state.conn.lock().unwrap();
                conn.query_row("SELECT state FROM download_jobs", [], |r| {
                    r.get::<_, String>(0)
                })
                .unwrap()
            };
            if status == "fetching" {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        cancel_dyndon_install(&state, &plan.install_plan_id).unwrap();
    };
    let (result, ()) = tokio::join!(run_dyndon_install(&state, &plan.install_plan_id), cancel);
    assert!(result.unwrap_err().contains("cancelled"));
    source.delay_range.store(false, Ordering::SeqCst);
    let final_job = run_dyndon_install(&state, &plan.install_plan_id)
        .await
        .unwrap();
    assert_eq!(final_job.state, "completed");
    {
        let conn = state.conn.lock().unwrap();
        let counts: (i64, i64) = conn
            .query_row(
                "SELECT COUNT(*),SUM(state='cancelled') FROM download_jobs",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(counts, (2, 1));
    }
    // Corrupt the location of one second pack; the valid first pack still restores.
    {
        let conn = state.conn.lock().unwrap();
        conn.execute_batch("INSERT INTO packs SELECT 'bad-pack',corpus_id,display_name,description,language_bcp47,pack_kind,NULL,created_at_ms,updated_at_ms FROM packs LIMIT 1; INSERT INTO pack_versions SELECT 'bad-version','bad-pack',upstream_version,edition_manifest_sha256,artifact_sha256,artifact_size_bytes,format,verification_state,installed_at_ms,created_at_ms,source_identity_sha256,verification_scope FROM pack_versions LIMIT 1;UPDATE packs SET active_pack_version_id='bad-version' WHERE pack_id='bad-pack';INSERT INTO pack_locations VALUES('bad-location','bad-version','chosen','packs/bad-version','active',0,NULL,1);").unwrap();
    }
    drop(state);
    server.abort();
    source.offline.store(true, Ordering::SeqCst);
    let restarted = make_state(&root, source);
    restore_dyndon_archives(&restarted).await.unwrap();
    assert_eq!(restarted.archives.lock().unwrap().len(), 1);
    assert_eq!(restarted.restore_errors.lock().unwrap().len(), 1);
    drop(restarted);
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn published_digest_distinct_budget_editions_share_one_active_location() {
    let (source, server) = serve(fixture(), true).await;
    let root = root();
    let state = make_state(&root, source);
    let entry = search_kiwix_catalog(&state, "", None, 0, 20)
        .await
        .unwrap()
        .items
        .remove(0);
    let mut request = DynDonInstallRequest {
        catalog_entry_id: entry.catalog_entry_id,
        destination_storage_root_id: "chosen".into(),
        budget_bytes: 16 * 1024 * 1024,
        selection_profile_id: None,
    };
    let first = prepare_dyndon_install(&state, request.clone())
        .await
        .unwrap();
    let original = state
        .dyndon_stores
        .lock()
        .unwrap()
        .values()
        .next()
        .unwrap()
        .clone();
    let repeat = prepare_dyndon_install(&state, request.clone())
        .await
        .unwrap();
    assert_eq!(first.install_plan_id, repeat.install_plan_id);
    assert!(Arc::ptr_eq(
        &original,
        state.dyndon_stores.lock().unwrap().values().next().unwrap()
    ));
    run_dyndon_install(&state, &first.install_plan_id)
        .await
        .unwrap();
    request.budget_bytes = 17 * 1024 * 1024;
    let second = prepare_dyndon_install(&state, request).await.unwrap();
    assert_ne!(first.install_plan_id, second.install_plan_id);
    let done = run_dyndon_install(&state, &second.install_plan_id)
        .await
        .unwrap();
    assert_eq!(done.bytes_verified, 0);
    {
        let conn = state.conn.lock().unwrap();
        let counts:(i64,i64,i64)=conn.query_row("SELECT COUNT(*),COUNT(DISTINCT source_identity_sha256),SUM(artifact_sha256 IS NOT NULL) FROM pack_versions",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap();
        assert_eq!(counts, (2, 1, 2));
        let locations:(i64,i64)=conn.query_row("SELECT SUM(locator_kind='active'),SUM(locator_kind='retained') FROM pack_locations",[],|r|Ok((r.get(0)?,r.get(1)?))).unwrap();
        assert_eq!(locations, (1, 1));
    }
    assert_eq!(original.manifest().unwrap().metadata.article_count, 0);
    drop(original);
    drop(state);
    server.abort();
    std::fs::remove_dir_all(root).unwrap();
}
