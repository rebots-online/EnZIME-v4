#[path = "support/mod.rs"]
#[allow(dead_code)]
mod support;
use anzimmerman::{
    cluster_stream::*, directory::RawEntry, header::*, identity::*, model::*, range::*,
};
use async_trait::async_trait;
use bytes::Bytes;
use futures_util::StreamExt;
use std::{
    collections::BTreeMap,
    io::Cursor,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

struct Source(Bytes);
#[async_trait]
impl RandomAccessSource for Source {
    fn len(&self) -> u64 {
        self.0.len() as u64
    }
    async fn read_range(&self, offset: u64, length: u64) -> Result<Bytes, ZimError> {
        Ok(self.0.slice(offset as usize..(offset + length) as usize))
    }
}
struct Blobs {
    bytes: Bytes,
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
        let bytes = self
            .bytes
            .slice(range.start as usize..(range.start + range.length) as usize);
        let mut store = tempfile::tempfile().unwrap();
        let decoded = decode_cluster_to(&mut Cursor::new(bytes), &mut store, Default::default())?;
        read_cluster_blob(&mut store, decoded, blob, maximum)
    }
}
fn fixture() -> Bytes {
    let mut builder = support::ZimBuilder::default()
        .add_article(
            b'A',
            "Alpha",
            "Alpha",
            "text/html",
            b"<p>alpha</p>".to_vec(),
        )
        .add_article(
            b'A',
            "Missing",
            "Missing",
            "text/html",
            b"<p>missing</p>".to_vec(),
        )
        .add_article(
            b'A',
            "Unvisited",
            "Unvisited",
            "text/html",
            b"<p>unvisited</p>".to_vec(),
        )
        .add_article(b'I', "pixel", "pixel", "image/png", b"pixel".to_vec())
        .add_redirect(b'R', "Home", "Home", 5)
        .add_redirect(b'R', "Next", "Next", 2);
    builder.main_page_index = Some(4);
    let mut bytes = Vec::new();
    builder.write(&mut bytes);
    bytes.into()
}
fn metadata() -> ZimMetadata {
    ZimMetadata {
        uuid: String::new(),
        title: "offline".into(),
        description: String::new(),
        language_bcp47: "en".into(),
        creator: String::new(),
        publisher: String::new(),
        date: String::new(),
        article_count: 2,
        media_count: 1,
        main_page: None,
        checksum: None,
    }
}

#[tokio::test]
async fn canonical_records_capture_exact_consumed_bytes_and_chain() {
    let bytes = fixture();
    let remote = RemoteZimIndex::open(Arc::new(Source(bytes.clone())), Default::default())
        .await
        .unwrap();
    let records = remote.resolve_records(4, 16).await.unwrap();
    assert_eq!(
        records.iter().map(|r| r.entry.index).collect::<Vec<_>>(),
        vec![4, 5, 2]
    );
    for record in records {
        let at = remote.header().path_ptr_pos as usize + record.entry.index as usize * 8;
        let pointer = u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap()) as usize;
        let (raw, consumed) = anzimmerman::directory::parse_entry(&bytes[pointer..]).unwrap();
        assert_eq!(
            record.raw_bytes.as_ref(),
            &bytes[pointer..pointer + consumed]
        );
        assert_eq!(record.entry.raw.url(), raw.url());
    }
    assert!(remote.resolve_records(4, 1).await.is_err());
    assert!(remote.resolve_records(99, 16).await.is_err());
}

struct Catalog {
    header: ZimHeader,
    rows: BTreeMap<u32, IndexedEntry>,
    ranges: BTreeMap<u32, ClusterByteRange>,
    accesses: AtomicUsize,
}
#[async_trait]
impl ZimEntryIndex for Catalog {
    fn header(&self) -> &ZimHeader {
        &self.header
    }
    async fn entry_at(&self, index: u32) -> Result<IndexedEntry, ZimError> {
        self.accesses.fetch_add(1, Ordering::Relaxed);
        self.rows
            .get(&index)
            .cloned()
            .ok_or_else(|| ZimError::EntryNotFound(index.to_string()))
    }
    async fn find_entry(&self, key: &EntryKey) -> Result<Option<IndexedEntry>, ZimError> {
        Ok(self.rows.values().find(|e| e.key == *key).cloned())
    }
    async fn cluster_range(&self, index: u32) -> Result<ClusterByteRange, ZimError> {
        self.ranges
            .get(&index)
            .copied()
            .ok_or_else(|| ZimError::EntryNotFound(index.to_string()))
    }
    async fn entries(self: Arc<Self>) -> Result<IndexedEntryStream, ZimError> {
        Ok(Box::pin(async_stream::try_stream! {
            for entry in self.rows.values() { yield entry.clone(); }
        }))
    }
}
async fn catalog(bytes: Bytes) -> Catalog {
    let remote = RemoteZimIndex::open(Arc::new(Source(bytes)), Default::default())
        .await
        .unwrap();
    let mut catalog = Catalog {
        header: remote.header().clone(),
        rows: BTreeMap::new(),
        ranges: BTreeMap::new(),
        accesses: AtomicUsize::new(0),
    };
    for id in [0, 2, 3, 4, 5] {
        let entry = remote.entry_at(id).await.unwrap();
        if let RawEntry::Article(article) = &entry.raw {
            catalog.ranges.insert(
                article.cluster_number,
                remote.cluster_range(article.cluster_number).await.unwrap(),
            );
        }
        catalog.rows.insert(id, entry);
    }
    // The sparse provider retains original IDs; its reported source may be huge.
    catalog.header.entry_count = 1_000_000;
    catalog
}
#[tokio::test]
async fn sparse_original_ids_open_unvisited_article_resource_search_and_stream() {
    let bytes = fixture();
    let catalog = Arc::new(catalog(bytes.clone()).await);
    let archive = RangeZimArchive::open_index(
        catalog.clone(),
        Arc::new(Blobs { bytes }),
        metadata(),
        Default::default(),
    )
    .await
    .unwrap();
    assert_eq!(archive.main_page().await.unwrap().unwrap().0, "A/Unvisited");
    assert_eq!(
        archive
            .article(&ArticleKey("A/Unvisited".into()))
            .await
            .unwrap()
            .raw_html
            .as_ref(),
        b"<p>unvisited</p>"
    );
    assert_eq!(
        archive
            .article(&ArticleKey("R/Home".into()))
            .await
            .unwrap()
            .normalized_text,
        "unvisited"
    );
    assert_eq!(
        archive
            .resource(&EntryKey("I/pixel".into()))
            .await
            .unwrap()
            .bytes
            .as_ref(),
        b"pixel"
    );
    assert!(matches!(
        archive.article(&ArticleKey("A/Missing".into())).await,
        Err(ZimError::EntryNotFound(_))
    ));
    let hits = archive
        .search_titles(TitleSearchQuery {
            text: "unvisited".into(),
            offset: 0,
            limit: 10,
        })
        .await
        .unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].article_key.0, "A/Unvisited");
    let mut entries = archive.entries().await.unwrap();
    let mut keys = Vec::new();
    while let Some(entry) = entries.next().await {
        keys.push(entry.unwrap().entry_key.0);
    }
    assert_eq!(
        keys,
        vec!["A/Alpha", "A/Unvisited", "I/pixel", "R/Home", "R/Next"]
    );
    assert!(catalog.accesses.load(Ordering::Relaxed) < 20);
}
#[tokio::test]
async fn sparse_redirect_cycles_missing_targets_and_hard_hop_bound() {
    let mut index = catalog(fixture()).await;
    assert!(index
        .resolve_entry(&EntryKey("R/Home".into()), 1)
        .await
        .is_err());
    if let RawEntry::Redirect(r) = &mut index.rows.get_mut(&5).unwrap().raw {
        r.redirect_index = 4;
    }
    assert!(matches!(
        index.resolve_entry(&EntryKey("R/Home".into()), 32).await,
        Err(ZimError::RedirectCycle)
    ));
    if let RawEntry::Redirect(r) = &mut index.rows.get_mut(&5).unwrap().raw {
        r.redirect_index = 1;
    }
    assert!(matches!(
        index.resolve_entry(&EntryKey("R/Home".into()), 16).await,
        Err(ZimError::EntryNotFound(_))
    ));
    for id in 100..=133 {
        let mut entry = index.rows[&4].clone();
        entry.index = id;
        entry.key = EntryKey(format!("R/{id}"));
        if let RawEntry::Redirect(r) = &mut entry.raw {
            r.redirect_index = if id == 133 { 2 } else { id + 1 };
        }
        index.rows.insert(id, entry);
    }
    assert!(index
        .resolve_entry(&EntryKey("R/100".into()), 255)
        .await
        .is_err());
}
