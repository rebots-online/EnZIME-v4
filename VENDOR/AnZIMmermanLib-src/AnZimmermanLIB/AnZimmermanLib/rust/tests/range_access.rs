#[path = "support/mod.rs"]
#[allow(dead_code)]
mod support;
use anzimmerman::{cluster_stream::*, header::*, identity::*, model::*, range::*};
use async_trait::async_trait;
use bytes::Bytes;
use futures_util::StreamExt;
use std::{
    io::Cursor,
    sync::{Arc, Mutex},
};
struct Sparse {
    length: u64,
    segments: Vec<(u64, Vec<u8>)>,
    requests: Mutex<Vec<(u64, u64)>>,
    short: bool,
}
#[async_trait]
impl RandomAccessSource for Sparse {
    fn len(&self) -> u64 {
        self.length
    }
    async fn read_range(&self, offset: u64, length: u64) -> Result<Bytes, ZimError> {
        assert!(length <= 65536, "unbounded underlying request");
        self.requests.lock().unwrap().push((offset, length));
        // Force requests to overlap so the cache-capacity test exercises suspended misses.
        tokio::task::yield_now().await;
        if offset
            .checked_add(length)
            .is_none_or(|end| end > self.length)
        {
            return Err(ZimError::Io("outside source".into()));
        }
        let mut output = vec![0; length as usize];
        for (base, bytes) in &self.segments {
            let start = offset.max(*base);
            let end = (offset + length).min(*base + bytes.len() as u64);
            if start < end {
                output[(start - offset) as usize..(end - offset) as usize]
                    .copy_from_slice(&bytes[(start - base) as usize..(end - base) as usize]);
            }
        }
        if self.short {
            output.pop();
        }
        Ok(output.into())
    }
}
fn source(bytes: Vec<u8>) -> Arc<Sparse> {
    Arc::new(Sparse {
        length: bytes.len() as u64,
        segments: vec![(0, bytes)],
        requests: Mutex::new(vec![]),
        short: false,
    })
}
fn fixture() -> Vec<u8> {
    let mut b = support::ZimBuilder::default()
        .add_article(b'A', "Café", "Café", "text/html", b"<p>chosen</p>".to_vec())
        .add_redirect(b'A', "Home", "Home", 0)
        .add_article(
            b'C',
            "Article",
            "Article",
            "text/html",
            b"<p>C namespace</p>".to_vec(),
        )
        .add_article(
            b'C',
            "pixel.txt",
            "pixel.txt",
            "text/plain",
            b"pixel".to_vec(),
        );
    b.main_page_index = Some(1);
    let mut out = vec![];
    b.write(&mut out);
    out
}
fn u64at(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}
fn set64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}
fn seed() -> ZimMetadata {
    ZimMetadata {
        uuid: "wrong".into(),
        title: "catalog".into(),
        description: "".into(),
        language_bcp47: "en".into(),
        creator: "".into(),
        publisher: "".into(),
        date: "".into(),
        article_count: 0,
        media_count: 0,
        main_page: None,
        checksum: None,
    }
}
struct Blobs {
    source: Arc<dyn RandomAccessSource>,
    calls: Mutex<usize>,
}
#[async_trait]
impl ClusterBlobSource for Blobs {
    async fn read_blob(
        &self,
        _: u32,
        range: ClusterByteRange,
        blob: u32,
        maximum: u64,
    ) -> Result<Bytes, ZimError> {
        *self.calls.lock().unwrap() += 1;
        let bytes = self.source.read_range(range.start, range.length).await?;
        let mut file = tempfile::tempfile().unwrap();
        let d = decode_cluster_to(&mut Cursor::new(bytes), &mut file, Default::default())?;
        read_cluster_blob(&mut file, d, blob, maximum)
    }
}
#[tokio::test]
async fn virtual_tib_opens_lazily_reads_article_and_bounds_retained_pages() {
    let mut bytes = fixture();
    let original = parse_header(&bytes).unwrap();
    let table = original.cluster_ptr_pos as usize;
    let first = u64at(&bytes, table);
    let payload = bytes[first as usize..original.checksum_pos as usize].to_vec();
    let far = (1u64 << 40) - 65536;
    for i in 0..original.cluster_count {
        let old = u64at(&bytes, table + i as usize * 8);
        set64(&mut bytes, table + i as usize * 8, far + old - first);
    }
    set64(&mut bytes, 72, far + payload.len() as u64);
    bytes.truncate(first as usize);
    let source = Arc::new(Sparse {
        length: 1 << 40,
        segments: vec![(0, bytes), (far, payload)],
        requests: Mutex::new(vec![]),
        short: false,
    });
    let index = Arc::new(
        RemoteZimIndex::open(source.clone(), Default::default())
            .await
            .unwrap(),
    );
    let blobs = Arc::new(Blobs {
        source: source.clone(),
        calls: Mutex::new(0),
    });
    let archive = RangeZimArchive::open(index.clone(), blobs.clone(), seed(), Default::default())
        .await
        .unwrap();
    assert_eq!(*blobs.calls.lock().unwrap(), 0);
    assert_eq!(archive.main_page().await.unwrap().unwrap().0, "A/Café");
    assert_eq!(archive.metadata().article_count, 0);
    assert_eq!(archive.metadata().uuid, "0".repeat(32));
    assert_eq!(
        archive
            .article(&ArticleKey("A/Home".into()))
            .await
            .unwrap()
            .raw_html
            .as_ref(),
        b"<p>chosen</p>"
    );
    assert_eq!(
        archive
            .article(&ArticleKey("C/Article".into()))
            .await
            .unwrap()
            .raw_html
            .as_ref(),
        b"<p>C namespace</p>"
    );
    assert!(matches!(
        archive
            .entry(&EntryKey("C/pixel.txt".into()))
            .await
            .unwrap()
            .kind,
        ZimEntryKind::Resource
    ));
    assert!(archive
        .article(&ArticleKey("C/pixel.txt".into()))
        .await
        .is_err());
    assert_eq!(
        archive
            .resource(&EntryKey("C/pixel.txt".into()))
            .await
            .unwrap()
            .bytes
            .as_ref(),
        b"pixel"
    );
    let report = archive.verify_structure().await.unwrap();
    assert!(report.valid);
    assert!(report
        .warnings
        .iter()
        .any(|w| w.code == "partial_metadata_validation"));
    let log = source.requests.lock().unwrap();
    assert!(log.iter().all(|(_, n)| *n <= 65536));
    assert!(log.iter().map(|(_, n)| n).sum::<u64>() < 1048576);
    assert!(index.cache_counts().await.1 <= 8);
}
#[tokio::test]
async fn exact_reads_lru_concurrency_and_limits() {
    let source = Arc::new(Sparse {
        length: 32 * 65536,
        segments: vec![],
        requests: Mutex::new(vec![]),
        short: false,
    });
    let reader = Arc::new(
        PagedRangeReader::new(
            source.clone(),
            RangeReadLimits {
                cached_pages: 2,
                ..Default::default()
            },
        )
        .unwrap(),
    );
    let tasks = (0..20).map(|i| {
        let r = reader.clone();
        async move { r.read_exact(i * 65536, 12).await.unwrap() }
    });
    futures_util::future::join_all(tasks).await;
    assert_eq!(reader.cache_counts().await, (2, 2));
    assert_eq!(reader.read_exact(65530, 100).await.unwrap().len(), 100);
    assert!(reader.read_exact(u64::MAX, 2).await.is_err());
    assert!(reader.read_exact(0, 1048577).await.is_err());
    let short = Arc::new(Sparse {
        length: 100,
        segments: vec![],
        requests: Mutex::new(vec![]),
        short: true,
    });
    assert!(PagedRangeReader::new(short, Default::default())
        .unwrap()
        .read_exact(0, 80)
        .await
        .is_err());
    for limits in [
        RangeReadLimits {
            page_bytes: 0,
            ..Default::default()
        },
        RangeReadLimits {
            cached_pages: 17,
            ..Default::default()
        },
        RangeReadLimits {
            max_entry_bytes: 65537,
            ..Default::default()
        },
        RangeReadLimits {
            max_mime_bytes: 1048577,
            ..Default::default()
        },
        RangeReadLimits {
            max_blob_bytes: 33554433,
            ..Default::default()
        },
        RangeReadLimits {
            max_redirect_hops: 33,
            ..Default::default()
        },
    ] {
        assert!(PagedRangeReader::new(source.clone(), limits).is_err());
    }
}
#[tokio::test]
async fn unicode_byte_order_numeric_redirects_cycles_and_lazy_stream() {
    let mut builder = support::ZimBuilder::default()
        .add_article(b'A', "Z", "Z", "text/html", b"z".to_vec())
        .add_article(b'A', "a", "a", "text/html", b"a".to_vec())
        .add_article(b'A', "é", "é", "text/html", b"e".to_vec())
        .add_redirect(b'R', "one", "one", 4)
        .add_redirect(b'R', "two", "two", 2);
    builder.main_page_index = Some(3);
    let mut bytes = vec![];
    builder.write(&mut bytes);
    let source = source(bytes);
    let index = Arc::new(
        RemoteZimIndex::open(source.clone(), Default::default())
            .await
            .unwrap(),
    );
    for (i, key) in ["A/Z", "A/a", "A/é"].iter().enumerate() {
        assert_eq!(
            index
                .find_entry(&EntryKey((*key).into()))
                .await
                .unwrap()
                .unwrap()
                .index,
            i as u32
        );
    }
    assert!(index
        .find_entry(&EntryKey("A/É".into()))
        .await
        .unwrap()
        .is_none());
    assert_eq!(index.main_entry().await.unwrap().unwrap().key.0, "A/é");
    assert!(index
        .resolve_entry(&EntryKey("R/one".into()), 1)
        .await
        .is_err());
    let archive = RangeZimArchive::open(
        index.clone(),
        Arc::new(Blobs {
            source,
            calls: Mutex::new(0),
        }),
        seed(),
        Default::default(),
    )
    .await
    .unwrap();
    let before = archive.verify_structure().await.unwrap().checked_entries;
    let mut stream = archive.entries().await.unwrap();
    assert_eq!(
        archive.verify_structure().await.unwrap().checked_entries,
        before
    );
    assert_eq!(stream.next().await.unwrap().unwrap().entry_key.0, "A/Z");
    assert_eq!(
        archive.verify_structure().await.unwrap().checked_entries,
        before + 1
    );
    drop(stream);
    assert!(archive
        .search_titles(TitleSearchQuery {
            text: "a".into(),
            offset: u32::MAX,
            limit: u32::MAX
        })
        .await
        .unwrap()
        .is_empty());
    let mut cycle = support::ZimBuilder::default()
        .add_redirect(b'A', "one", "one", 1)
        .add_redirect(b'A', "two", "two", 0);
    cycle.main_page_index = Some(0);
    let mut bytes = vec![];
    cycle.write(&mut bytes);
    let idx = RemoteZimIndex::open(source_fn(bytes), Default::default())
        .await
        .unwrap();
    assert!(matches!(
        idx.main_entry().await,
        Err(ZimError::RedirectCycle)
    ));
}
fn source_fn(bytes: Vec<u8>) -> Arc<Sparse> {
    source(bytes)
}
#[tokio::test]
async fn rejects_bad_tables_mime_entries_indices_and_counts() {
    for offset in [32, 48, 56, 72] {
        let mut bytes = fixture();
        set64(&mut bytes, offset, u64::MAX);
        assert!(RemoteZimIndex::open(source(bytes), Default::default())
            .await
            .is_err());
    }
    let mut bytes = fixture();
    bytes[24..28].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(RemoteZimIndex::open(source(bytes), Default::default())
        .await
        .is_err());
    let mut bytes = fixture();
    let h = parse_header(&bytes).unwrap();
    for b in &mut bytes[80..h.path_ptr_pos as usize] {
        *b = b'x';
    }
    assert!(RemoteZimIndex::open(source(bytes), Default::default())
        .await
        .is_err());
    let mut bytes = fixture();
    let table = u64at(&bytes, 32) as usize;
    let ptr = u64at(&bytes, table) as usize;
    bytes[ptr] = 255;
    bytes[ptr + 1] = 127;
    let idx = RemoteZimIndex::open(source(bytes), Default::default())
        .await
        .unwrap();
    assert!(idx.entry_at(0).await.is_err());
    assert!(idx.entry_at(99).await.is_err());
    assert!(idx.cluster_range(99).await.is_err());
    let mut bytes = fixture();
    let ptr = u64at(&bytes, u64at(&bytes, 32) as usize) as usize;
    bytes[ptr + 16] = 255;
    let idx = RemoteZimIndex::open(source(bytes), Default::default())
        .await
        .unwrap();
    assert!(idx.entry_at(0).await.is_err());
    let mut bytes = fixture();
    let table = u64at(&bytes, 32) as usize;
    let end = bytes.len() as u64 - 2;
    set64(&mut bytes, table, end);
    let idx = RemoteZimIndex::open(source(bytes), Default::default())
        .await
        .unwrap();
    assert!(idx.entry_at(0).await.is_err());
}
#[tokio::test]
async fn cluster_boundaries_before_after_metadata_and_noncontiguous_entries() {
    let bytes = fixture();
    let mut h = parse_header(&bytes).unwrap();
    let size = bytes.len() as u64;
    assert_eq!(
        cluster_end(&h, 80, Some(95), size).unwrap(),
        95.min(h.path_ptr_pos).min(h.cluster_ptr_pos)
    );
    assert_eq!(
        cluster_end(&h, h.cluster_ptr_pos + 24, None, size).unwrap(),
        h.checksum_pos
    );
    h.checksum_pos = 0;
    assert_eq!(
        cluster_end(&h, h.cluster_ptr_pos + 24, None, size).unwrap(),
        size
    );
    assert!(cluster_end(&h, 100, Some(100), size).is_err());
    h.title_ptr_pos = size + 1;
    assert!(cluster_end(&h, 100, None, size).is_err());
    // Move directory entries apart, keeping logical pointer order. No next-entry extent assumption is valid.
    let mut bytes = fixture();
    let h = parse_header(&bytes).unwrap();
    let table = h.path_ptr_pos as usize;
    let ptr = u64at(&bytes, table);
    let next = u64at(&bytes, table + 8);
    let entry = bytes[ptr as usize..next as usize].to_vec();
    set64(&mut bytes, table, 131072);
    let source = Arc::new(Sparse {
        length: 262144,
        segments: vec![(0, bytes), (131072, entry)],
        requests: Mutex::new(vec![]),
        short: false,
    });
    let idx = RemoteZimIndex::open(source, Default::default())
        .await
        .unwrap();
    assert_eq!(idx.entry_at(0).await.unwrap().key.0, "A/Café");
}

#[path = "range_access/edge_cases.rs"]
mod edge_cases;
