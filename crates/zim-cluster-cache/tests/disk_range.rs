use anzimmerman::{
    identity::{ArticleKey, EntryKey},
    model::ZimMetadata,
    range::{ClusterByteRange, RandomAccessSource, RangeReadLimits, RemoteZimIndex},
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
const TIB: u64 = 1024 * 1024 * 1024 * 1024;
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
fn generated(n: usize) -> Vec<u8> {
    let mut b = support::ZimBuilder::default();
    for i in 0..n {
        b = b.add_article(
            b'A',
            format!("a{i:05}"),
            "Title",
            "text/html",
            vec![b'x'; if n == 40 { 10 + (i % 6) * 50 } else { 10 }],
        );
    }
    b.main_page_index = Some(0);
    let mut bytes = vec![];
    b.write(&mut bytes);
    bytes
}
#[tokio::test]
async fn canonical_fixture_installs_and_reopens_offline_with_resource() {
    let bytes=include_bytes!("../../../VENDOR/AnZIMmermanLib-src/AnZimmermanLIB/AnZimmermanLib/tests/fixtures/rust-consolidation.zim").to_vec();
    let len = TIB;
    let (root, source, store, index) = opened(bytes, len).await;
    let main = index.main_entry().await.unwrap().unwrap();
    let cluster = match main.raw {
        anzimmerman::directory::RawEntry::Article(article) => article.cluster_number,
        _ => panic!("resolved article"),
    };
    let plan = plan_disk_fit(
        store.clone(),
        index,
        dyndon::SelectionIntent {
            domains: vec![],
            breadth_limit: None,
            depth_limit: None,
            budget_bytes: 1024 * 1024,
        },
        &[cluster],
    )
    .await
    .unwrap();
    assert!(plan.selected_count >= 2);
    install_selected(store.clone()).await.unwrap();
    let archive = open_disk_archive(store.clone()).await.unwrap();
    let article = archive.article(&ArticleKey("A/Café".into())).await.unwrap();
    assert!(article
        .raw_html
        .windows(15)
        .any(|w| w == b"Library fixture"));
    assert_eq!(
        archive
            .resource(&EntryKey("I/pixel.txt".into()))
            .await
            .unwrap()
            .bytes,
        b"pixel"[..]
    );
    let before = source.requests.lock().unwrap().len();
    drop(archive);
    drop(store);
    let reopened = DiskPackStore::reopen(root.clone(), None).await.unwrap();
    let archive = open_disk_archive(reopened.clone()).await.unwrap();
    assert_eq!(
        archive
            .article(&ArticleKey("A/Café".into()))
            .await
            .unwrap()
            .raw_html,
        article.raw_html
    );
    assert_eq!(
        archive
            .resource(&EntryKey("I/pixel.txt".into()))
            .await
            .unwrap()
            .bytes,
        b"pixel"[..]
    );
    assert_eq!(source.requests.lock().unwrap().len(), before);
    assert!(reopened.used_bytes().await.unwrap() < 16 * 1024 * 1024);
    std::fs::write(
        root.join("clusters").join(format!("{cluster}.bin")),
        b"corrupt",
    )
    .unwrap();
    assert!(archive.article(&ArticleKey("A/Café".into())).await.is_err());
    let check = rusqlite::Connection::open(root.join("pack-index.sqlite")).unwrap();
    let state: String = check
        .query_row(
            "SELECT state FROM clusters WHERE cluster_index=?1",
            [cluster],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(state, "missing");
    drop(check);
    drop(archive);
    drop(reopened);
    let repaired = DiskPackStore::reopen(root, Some(source.clone()))
        .await
        .unwrap();
    repaired.set_offline(false).await.unwrap();
    assert_eq!(
        open_disk_archive(repaired)
            .await
            .unwrap()
            .article(&ArticleKey("A/Café".into()))
            .await
            .unwrap()
            .raw_html,
        article.raw_html
    );
    assert!(source.requests.lock().unwrap().len() > before);
}
#[tokio::test]
async fn metadata_pinning_lru_and_sparse_source_are_bounded() {
    let source = Arc::new(Source::new(vec![], TIB));
    let root = root();
    let store = DiskPackStore::create(root.clone(), manifest(TIB, String::new()), source.clone())
        .await
        .unwrap();
    let pinned = DiskRangeSource::new(store.clone());
    let planning = DiskRangeSource::new_for_planning(store.clone());
    pinned.read_range(0, 80).await.unwrap();
    for page in 1..100 {
        planning.read_range(page * 65536, 8).await.unwrap();
    }
    pinned.read_range(99 * 65536, 8).await.unwrap();
    for page in 100..150 {
        planning.read_range(page * 65536, 8).await.unwrap();
    }
    store.set_offline(true).await.unwrap();
    assert!(pinned.read_range(0, 80).await.is_ok());
    assert!(pinned.read_range(99 * 65536, 8).await.is_ok());
    assert!(planning.read_range(65536, 8).await.is_err());
    let requests = source.requests.lock().unwrap();
    assert!(requests.iter().all(|(_, n)| *n <= 65536));
    assert!(requests.iter().map(|(_, n)| n).sum::<u64>() < 16 * 1024 * 1024);
    drop(requests);
    assert!(
        store.used_bytes().await.unwrap() <= store.manifest().unwrap().policy.metadata_budget_bytes
    );
    let conn = rusqlite::Connection::open(root.join("pack-index.sqlite")).unwrap();
    let pages: u64 = conn
        .query_row("SELECT COUNT(*) FROM pages", [], |r| r.get(0))
        .unwrap();
    assert!(pages < 40);
}
#[tokio::test]
async fn payload_quota_evicts_only_unpinned_and_resume_ignores_partial_files() {
    let source = Arc::new(Source::new(vec![], TIB));
    let root = root();
    let store = DiskPackStore::create(root.clone(), manifest(TIB, String::new()), source)
        .await
        .unwrap();
    let mib = 1024 * 1024;
    store
        .ensure_cluster(
            0,
            ClusterByteRange {
                start: 0,
                length: 2 * mib,
            },
            true,
        )
        .await
        .unwrap();
    store
        .ensure_cluster(
            1,
            ClusterByteRange {
                start: 2 * mib,
                length: 6 * mib,
            },
            false,
        )
        .await
        .unwrap();
    store
        .ensure_cluster(
            2,
            ClusterByteRange {
                start: 8 * mib,
                length: 6 * mib,
            },
            false,
        )
        .await
        .unwrap();
    assert!(root.join("clusters/0.bin").exists());
    assert!(!root.join("clusters/1.bin").exists());
    assert!(root.join("clusters/2.bin").exists());
    assert_eq!(store.retained_bytes().unwrap(), 8 * mib);
    assert!(store
        .ensure_cluster(
            3,
            ClusterByteRange {
                start: 14 * mib,
                length: 11 * mib
            },
            false
        )
        .await
        .unwrap_err()
        .contains("budget_exceeded"));
    assert!(root.join("clusters/2.bin").exists());
    std::fs::write(root.join("clusters/9.part"), b"partial").unwrap();
    drop(store);
    let reopened = DiskPackStore::reopen(root.clone(), None).await.unwrap();
    assert!(!root.join("clusters/9.part").exists());
    assert!(root.join("clusters/0.bin").exists());
    assert!(reopened
        .ensure_cluster(
            9,
            ClusterByteRange {
                start: 99,
                length: 7
            },
            false
        )
        .await
        .is_err());
}
#[tokio::test]
async fn disk_planner_matches_dyndon_global_filters_and_greedy_ties() {
    let bytes = generated(40);
    let len = bytes.len() as u64;
    let (root, _, store, index) = opened(bytes, len).await;
    let all = dyndon::SelectionIntent {
        domains: vec![],
        breadth_limit: None,
        depth_limit: None,
        budget_bytes: 1_000_000,
    };
    plan_disk_fit(store.clone(), index.clone(), all, &[])
        .await
        .unwrap();
    let conn = rusqlite::Connection::open(root.join("pack-index.sqlite")).unwrap();
    let mut entries = vec![];
    for i in 0..40u32 {
        let range = index.cluster_range(i).await.unwrap();
        let domain = if i % 3 == 0 {
            None
        } else {
            Some(format!("D{}", i % 3))
        };
        let topic = if i % 4 == 0 {
            None
        } else {
            Some(format!("T{}", i % 4))
        };
        let priority = i % 5 + 1;
        let count = i % 7;
        conn.execute("UPDATE clusters SET domain=?2,topic=?3,priority=?4,article_count=?5 WHERE cluster_index=?1",rusqlite::params![i,domain,topic,priority,count]).unwrap();
        entries.push(dyndon::ClusterManifestEntry {
            cluster_index: i,
            byte_offset: range.start,
            byte_len: range.length,
            domain,
            topic,
            priority,
            article_count: count,
        });
    }
    let manifest = dyndon::ClusterManifest {
        pack_version_id: "test".into(),
        cluster_count: 40,
        total_bytes: entries.iter().map(|e| e.byte_len).sum(),
        entries,
    };
    for breadth in [None, Some(0), Some(2), Some(8)] {
        for depth in [None, Some(0), Some(2)] {
            for filtered in [false, true] {
                let intent = dyndon::SelectionIntent {
                    domains: if filtered { vec!["D1".into()] } else { vec![] },
                    breadth_limit: breadth,
                    depth_limit: depth,
                    budget_bytes: 277,
                };
                let expected = dyndon::plan_fit(&manifest, &intent);
                let actual = plan_disk_fit(store.clone(), index.clone(), intent, &[])
                    .await
                    .unwrap();
                let selected:Vec<u32>=conn.prepare("SELECT cluster_index FROM clusters WHERE selected=1 ORDER BY priority DESC,article_count DESC,cluster_index ASC").unwrap().query_map([],|r|r.get(0)).unwrap().collect::<Result<_,_>>().unwrap();
                assert_eq!(selected, expected.selected);
                assert_eq!(actual.selected_bytes, expected.total_bytes);
                assert_eq!(actual.skipped_count, expected.skipped_count as u64);
            }
        }
    }
}
#[tokio::test]
async fn planner_batches_beyond_4096_candidates_without_changing_fit() {
    let bytes = generated(4200);
    let len = bytes.len() as u64;
    let (root, _, store, index) = opened(bytes, len).await;
    let plan = plan_disk_fit(
        store,
        index,
        dyndon::SelectionIntent {
            domains: vec![],
            breadth_limit: None,
            depth_limit: None,
            budget_bytes: 19 * 4190,
        },
        &[],
    )
    .await
    .unwrap();
    assert_eq!(plan.selected_count, 4190);
    assert_eq!(plan.skipped_count, 10);
    let conn = rusqlite::Connection::open(root.join("pack-index.sqlite")).unwrap();
    let peak: u32 = conn
        .query_row(
            "SELECT CAST(value AS INTEGER) FROM policy WHERE key='plan.peak_batch'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(peak, 4096);
}

#[tokio::test]
async fn distinct_semantic_groups_remain_inside_physical_metadata_quota() {
    let bytes = generated(1200);
    let len = bytes.len() as u64;
    let (root, _, store, index) = opened(bytes, len).await;
    let mut intent = dyndon::SelectionIntent {
        domains: vec![],
        breadth_limit: None,
        depth_limit: None,
        budget_bytes: 1_000_000,
    };
    plan_disk_fit(store.clone(), index.clone(), intent.clone(), &[])
        .await
        .unwrap();
    let conn = rusqlite::Connection::open(root.join("pack-index.sqlite")).unwrap();
    conn.execute("UPDATE clusters SET domain='D'||cluster_index,topic='T'||cluster_index,priority=cluster_index%7+1",[]).unwrap();
    intent.breadth_limit = Some(1000);
    intent.depth_limit = Some(3);
    let result = plan_disk_fit(store.clone(), index, intent, &[0]).await;
    result.unwrap();
    let policy = store.manifest().unwrap().policy;
    assert!(store.used_bytes().await.unwrap() <= policy.metadata_budget_bytes);
    {
        let selected: i64 = conn
            .query_row(
                "SELECT selected FROM clusters WHERE cluster_index=0",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(selected, 1);
    }
}
