use anzimmerman::{
    identity::{ArticleKey, EntryKey},
    model::ZimMetadata,
    range::{RangeReadLimits, RemoteZimIndex},
};
use async_trait::async_trait;
use bytes::Bytes;
use ports::transfer::*;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use zim_cluster_cache::disk::*;
#[allow(dead_code)]
#[path = "../../../VENDOR/AnZIMmermanLib-src/AnZimmermanLIB/AnZimmermanLib/rust/tests/support/mod.rs"]
mod support;
struct Source {
    bytes: Arc<Vec<u8>>,
    len: u64,
    requests: Mutex<Vec<(u64, u64)>>,
}
impl Source {
    fn new(bytes: Vec<u8>, len: u64) -> Self {
        Self {
            bytes: Arc::new(bytes),
            len,
            requests: Mutex::new(vec![]),
        }
    }
}
#[async_trait]
impl TransferSource for Source {
    fn source_kind(&self) -> &'static str {
        "fixture"
    }
    async fn probe(&self, r: TransferProbeRequest) -> Result<TransferProbe, TransferError> {
        Ok(TransferProbe {
            uri: r.uri,
            size_bytes: self.len,
            accepts_ranges: true,
            etag: Some("\"fixture-v1\"".into()),
            last_modified: None,
            content_type: None,
        })
    }
    async fn open_range(
        &self,
        _: &str,
        r: TransferRange,
        etag: Option<&str>,
    ) -> Result<TransferByteStream, TransferError> {
        assert_eq!(etag, Some("\"fixture-v1\""));
        if r.end_byte_exclusive > self.len {
            return Err(TransferError::InvalidRangeResponse);
        }
        self.requests
            .lock()
            .unwrap()
            .push((r.start_byte, r.end_byte_exclusive - r.start_byte));
        let data = self.bytes.clone();
        Ok(Box::pin(
            async_stream::try_stream! {let mut at=r.start_byte;while at<r.end_byte_exclusive {let n=(r.end_byte_exclusive-at).min(65536) as usize;let mut part=vec![0;n];if at<data.len() as u64 {let available=n.min(data.len()-at as usize);part[..available].copy_from_slice(&data[at as usize..at as usize+available]);}at+=n as u64;yield Bytes::from(part);}},
        ))
    }
    async fn open_full(&self, _: &str, _: u64) -> Result<TransferByteStream, TransferError> {
        panic!("full archive transfer forbidden")
    }
}
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/disk-range-tests")
        .join(format!(
            "{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
}
fn manifest(len: u64, uuid: String) -> DiskPackManifest {
    let budget = 16 * 1024 * 1024;
    DiskPackManifest {
        format_version: 1,
        pack_id: "fixture".into(),
        pack_version_id: "fixture-v1".into(),
        identity: SourceIdentity {
            uri: "https://fixture.invalid/archive.zim".into(),
            size_bytes: len,
            etag: Some("\"fixture-v1\"".into()),
            last_modified: None,
            zim_uuid: uuid,
        },
        policy: DiskPackPolicy {
            disk_budget_bytes: budget,
            metadata_budget_bytes: budget / 8,
            workspace_budget_bytes: budget / 4,
            offline: false,
        },
        metadata: ZimMetadata {
            uuid: String::new(),
            title: "Fixture".into(),
            description: String::new(),
            language_bcp47: "en".into(),
            creator: String::new(),
            publisher: String::new(),
            date: String::new(),
            article_count: 0,
            media_count: 0,
            main_page: None,
            checksum: None,
        },
        main_page: Some("A/Café".into()),
        state: "planned".into(),
    }
}
async fn opened(
    bytes: Vec<u8>,
    len: u64,
) -> (
    PathBuf,
    Arc<Source>,
    Arc<DiskPackStore>,
    Arc<RemoteZimIndex>,
) {
    let uuid = hex::encode(anzimmerman::header::parse_header(&bytes).unwrap().uuid);
    let source = Arc::new(Source::new(bytes, len));
    let root = root();
    let store = DiskPackStore::create(root.clone(), manifest(len, uuid), source.clone())
        .await
        .unwrap();
    let index = Arc::new(
        RemoteZimIndex::open(
            Arc::new(DiskRangeSource::new(store.clone())),
            RangeReadLimits::default(),
        )
        .await
        .unwrap(),
    );
    (root, source, store, index)
}
fn fixture() -> Vec<u8> {
    let mut builder = support::ZimBuilder::default();
    for i in 0..100 {
        builder = builder.add_article(
            b'A',
            format!("a{i:05}"),
            format!("Article {i:05} {}", "x".repeat(2000)),
            "text/html",
            format!("<p>payload {i}</p>").into_bytes(),
        );
    }
    builder = builder
        .add_redirect(b'A', "zalias", "Alias", 101)
        .add_redirect(b'A', "zalias2", "Alias2", 90)
        .add_article(
            b'I',
            "image.txt",
            "Image",
            "text/plain",
            b"image payload".to_vec(),
        );
    builder.main_page_index = Some(0);
    let mut bytes = vec![];
    builder.write(&mut bytes);
    bytes
}
async fn planned() -> (
    PathBuf,
    Arc<Source>,
    Arc<DiskPackStore>,
    Arc<RemoteZimIndex>,
) {
    let bytes = fixture();
    let len = bytes.len() as u64;
    let result = opened(bytes, len).await;
    plan_disk_fit(
        result.2.clone(),
        result.3.clone(),
        dyndon::SelectionIntent {
            domains: vec![],
            breadth_limit: None,
            depth_limit: None,
            budget_bytes: 1024 * 1024,
        },
        &[0],
    )
    .await
    .unwrap();
    result
}
#[tokio::test]
async fn unvisited_article_alias_resource_survive_offline_restart() {
    let (root, source, store, index) = planned().await;
    let progress = discover_offline_entries(store.clone(), index)
        .await
        .unwrap();
    assert!(progress.complete);
    assert_eq!(progress.readable_articles, 100);
    assert_eq!(progress.retained_aliases, 2);
    assert_eq!(progress.readable_resources, 1);
    assert_eq!(store.catalog_progress().unwrap().readable_articles, 0);
    install_selected(store.clone()).await.unwrap();
    let before = source.requests.lock().unwrap().len();
    drop(store);
    let reopened = DiskPackStore::reopen(root, None).await.unwrap();
    let archive = open_disk_archive(reopened.clone()).await.unwrap();
    let hits = archive
        .search_titles(anzimmerman::model::TitleSearchQuery {
            text: "00090".into(),
            offset: 0,
            limit: 10,
        })
        .await
        .unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].article_key.0, "A/a00090");
    let article = archive
        .article(&ArticleKey("A/a00090".into()))
        .await
        .unwrap();
    assert_eq!(&article.raw_html[..], b"<p>payload 90</p>");
    assert_eq!(
        archive
            .article(&ArticleKey("A/zalias".into()))
            .await
            .unwrap()
            .raw_html,
        article.raw_html
    );
    assert_eq!(
        &archive
            .resource(&EntryKey("I/image.txt".into()))
            .await
            .unwrap()
            .bytes[..],
        b"image payload"
    );
    assert_eq!(source.requests.lock().unwrap().len(), before);
    assert!(reopened.used_bytes().await.unwrap() < 16 * 1024 * 1024);
}
#[tokio::test]
async fn missing_payload_is_not_advertised_after_reopen() {
    let (root, _, store, index) = planned().await;
    discover_offline_entries(store.clone(), index)
        .await
        .unwrap();
    install_selected(store.clone()).await.unwrap();
    drop(store);
    std::fs::remove_file(root.join("clusters/90.bin")).unwrap();
    let store = DiskPackStore::reopen(root, None).await.unwrap();
    assert_eq!(store.catalog_progress().unwrap().readable_articles, 99);
    let archive = open_disk_archive(store).await.unwrap();
    assert!(archive
        .article(&ArticleKey("A/a00090".into()))
        .await
        .is_err());
}
#[tokio::test]
async fn legacy_catalog_requires_explicit_upgrade() {
    let (_, _, store, _) = planned().await;
    assert!(open_disk_archive(store)
        .await
        .err()
        .unwrap()
        .contains("offline_catalog_upgrade_required"));
}
#[tokio::test]
async fn allowance_exhaustion_is_partial_durable_and_never_refunded() {
    let (root, source, store, index) = planned().await;
    let db = rusqlite::Connection::open(root.join("pack-index.sqlite")).unwrap();
    db.execute("INSERT INTO policy VALUES('catalog.network_limit','0')", [])
        .unwrap();
    let before = source.requests.lock().unwrap().len();
    let progress = discover_offline_entries(store.clone(), index)
        .await
        .unwrap();
    assert!(!progress.complete);
    assert_eq!(progress.stop_reason.as_deref(), Some("transfer_allowance"));
    assert!(progress.scanned_entries > 0 && progress.scanned_entries < 100);
    assert_eq!(progress.network_reserved_bytes, 0);
    assert_eq!(source.requests.lock().unwrap().len(), before);
    install_selected(store.clone()).await.unwrap();
    drop(store);
    let reopened = DiskPackStore::reopen(root.clone(), Some(source.clone()))
        .await
        .unwrap();
    let index = Arc::new(
        RemoteZimIndex::open(
            Arc::new(DiskRangeSource::new(reopened.clone())),
            RangeReadLimits::default(),
        )
        .await
        .unwrap(),
    );
    let resumed = discover_offline_entries(reopened.clone(), index)
        .await
        .unwrap();
    assert_eq!(resumed.scanned_entries, progress.scanned_entries);
    assert_eq!(resumed.network_reserved_bytes, 0);
    assert_eq!(
        source.requests.lock().unwrap().len(),
        before + progress.readable_articles as usize
    );
    reopened.set_offline(true).await.unwrap();
    let archive = open_disk_archive(reopened).await.unwrap();
    assert!(archive
        .article(&ArticleKey("A/a00000".into()))
        .await
        .is_ok());
    assert!(archive
        .article(&ArticleKey("A/a00090".into()))
        .await
        .is_err());
}
#[tokio::test]
async fn source_identity_change_rejects_catalog() {
    let (root, _, store, index) = planned().await;
    discover_offline_entries(store.clone(), index)
        .await
        .unwrap();
    drop(store);
    let path = root.join("manifest.json");
    let mut value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    value["identity"]["etag"] = serde_json::json!("\"changed\"");
    std::fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
    let reopened = DiskPackStore::reopen(root, None).await.unwrap();
    assert!(open_disk_archive(reopened)
        .await
        .err()
        .unwrap()
        .contains("source_changed"));
}
#[tokio::test]
async fn missing_main_catalog_fails_before_payload_transfer() {
    let (root, source, store, index) = planned().await;
    discover_offline_entries(store.clone(), index)
        .await
        .unwrap();
    let db = rusqlite::Connection::open(root.join("pack-index.sqlite")).unwrap();
    db.execute("DELETE FROM catalog_entries WHERE entry_index=0", [])
        .unwrap();
    let before = source.requests.lock().unwrap().len();
    assert!(install_selected(store)
        .await
        .err()
        .unwrap()
        .contains("catalog_main_not_admitted"));
    assert_eq!(source.requests.lock().unwrap().len(), before);
}
#[tokio::test]
async fn metadata_pressure_prunes_selection_within_physical_partition() {
    let mut builder = support::ZimBuilder::default();
    for i in 0..600 {
        builder = builder.add_article(
            b'A',
            format!("a{i:05}"),
            "x".repeat(2000),
            "text/html",
            b"<p>tiny</p>".to_vec(),
        );
    }
    builder.main_page_index = Some(0);
    let mut bytes = vec![];
    builder.write(&mut bytes);
    let len = bytes.len() as u64;
    let (root, _, store, index) = opened(bytes, len).await;
    plan_disk_fit(
        store.clone(),
        index.clone(),
        dyndon::SelectionIntent {
            domains: vec![],
            breadth_limit: None,
            depth_limit: None,
            budget_bytes: 1024 * 1024,
        },
        &[0],
    )
    .await
    .unwrap();
    let progress = discover_offline_entries(store.clone(), index)
        .await
        .unwrap();
    let summary = store.planned_summary().unwrap();
    assert!(summary.selected_count > 0 && summary.selected_count < 600);
    assert_eq!(progress.stop_reason.as_deref(), Some("metadata_budget"));
    install_selected(store.clone()).await.unwrap();
    let mut metadata = 0;
    for entry in std::fs::read_dir(root.join("pages")).unwrap() {
        metadata += entry.unwrap().metadata().unwrap().len();
    }
    for name in [
        "pack-index.sqlite",
        "pack-index.sqlite-journal",
        "manifest.json",
        "manifest.part",
    ] {
        metadata += std::fs::metadata(root.join(name))
            .map(|m| m.len())
            .unwrap_or(0);
    }
    assert!(
        metadata <= store.manifest().unwrap().policy.metadata_budget_bytes,
        "{metadata}"
    );
    assert!(store.used_bytes().await.unwrap() <= 16 * 1024 * 1024);
    let archive = open_disk_archive(store).await.unwrap();
    assert!(archive
        .article(&ArticleKey("A/a00000".into()))
        .await
        .is_ok());
}
#[tokio::test]
async fn mandatory_catalog_cannot_bypass_pinned_metadata_budget() {
    use anzimmerman::range::RandomAccessSource;
    let mut bytes = fixture();
    bytes.resize(3 * 1024 * 1024, 0);
    let len = bytes.len() as u64;
    let (_, _, store, index) = opened(bytes, len).await;
    plan_disk_fit(
        store.clone(),
        index.clone(),
        dyndon::SelectionIntent {
            domains: vec![],
            breadth_limit: None,
            depth_limit: None,
            budget_bytes: 1024 * 1024,
        },
        &[0],
    )
    .await
    .unwrap();
    let source = DiskRangeSource::new(store.clone());
    for page in 0..48 {
        if source.read_range(page * 65536, 1).await.is_err() {
            break;
        }
    }
    let error = discover_offline_entries(store, index).await.unwrap_err();
    assert!(error.contains("budget_exceeded"), "{error}");
}
#[tokio::test]
async fn online_miss_persists_canonical_record_before_offline_use() {
    let (root, _, store, index) = planned().await;
    discover_offline_entries(store.clone(), index)
        .await
        .unwrap();
    install_selected(store.clone()).await.unwrap();
    let db = rusqlite::Connection::open(root.join("pack-index.sqlite")).unwrap();
    db.execute("DELETE FROM catalog_entries WHERE entry_index=90", [])
        .unwrap();
    let archive = open_disk_archive(store.clone()).await.unwrap();
    assert!(archive
        .article(&ArticleKey("A/a00090".into()))
        .await
        .is_ok());
    assert_eq!(store.catalog_progress().unwrap().readable_articles, 100);
    store.set_offline(true).await.unwrap();
    assert!(archive
        .article(&ArticleKey("A/a00090".into()))
        .await
        .is_ok());
    db.execute(
        "UPDATE catalog_entries SET mime_type='image/png' WHERE entry_index=90",
        [],
    )
    .unwrap();
    assert!(archive
        .article(&ArticleKey("A/a00090".into()))
        .await
        .err()
        .unwrap()
        .to_string()
        .contains("catalog_mime_corrupt"));
}
