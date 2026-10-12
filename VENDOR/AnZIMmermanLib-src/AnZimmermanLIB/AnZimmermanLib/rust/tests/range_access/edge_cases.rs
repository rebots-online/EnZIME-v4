use super::*;
#[tokio::test]
async fn physical_clusters_before_directory_and_pointer_metadata() {
    let original = fixture();
    let h = parse_header(&original).unwrap();
    let directory = u64at(&original, h.path_ptr_pos as usize) as usize;
    let cluster = u64at(&original, h.cluster_ptr_pos as usize) as usize;
    let payload = &original[cluster..h.checksum_pos as usize];
    let shift = payload.len();
    let mut bytes = [
        original[..directory].to_vec(),
        payload.to_vec(),
        original[directory..cluster].to_vec(),
        original[h.checksum_pos as usize..].to_vec(),
    ]
    .concat();
    set64(&mut bytes, 32, h.path_ptr_pos + shift as u64);
    set64(&mut bytes, 48, h.cluster_ptr_pos + shift as u64);
    for i in 0..h.entry_count {
        let old = u64at(&original, h.path_ptr_pos as usize + i as usize * 8);
        set64(
            &mut bytes,
            h.path_ptr_pos as usize + shift + i as usize * 8,
            old + shift as u64,
        );
    }
    for i in 0..h.cluster_count {
        let old = u64at(&original, h.cluster_ptr_pos as usize + i as usize * 8);
        set64(
            &mut bytes,
            h.cluster_ptr_pos as usize + shift + i as usize * 8,
            directory as u64 + old - cluster as u64,
        );
    }
    for checksum in [true, false] {
        let mut bytes = bytes.clone();
        if !checksum {
            set64(&mut bytes, 72, 0);
        }
        let src = source(bytes.clone());
        let index = Arc::new(
            RemoteZimIndex::open(src.clone(), Default::default())
                .await
                .unwrap(),
        );
        assert_eq!(
            index
                .cluster_range(h.cluster_count - 1)
                .await
                .unwrap()
                .start
                + index
                    .cluster_range(h.cluster_count - 1)
                    .await
                    .unwrap()
                    .length,
            index.header().path_ptr_pos
        );
        let archive = RangeZimArchive::open(
            index.clone(),
            Arc::new(Blobs {
                source: src,
                calls: Mutex::new(0),
            }),
            seed(),
            Default::default(),
        )
        .await
        .unwrap();
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
                .resource(&EntryKey("C/pixel.txt".into()))
                .await
                .unwrap()
                .bytes
                .as_ref(),
            b"pixel"
        );
        use std::io::Write;
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(&bytes).unwrap();
        let local = anzimmerman::archive::open_archive(&ZimOpenRequest {
            path: file.path().to_string_lossy().into_owned(),
            expected_sha256: None,
            expected_size_bytes: None,
        })
        .await
        .unwrap();
        for (i, (start, length)) in local
            .layout_snapshot()
            .unwrap()
            .cluster_byte_ranges
            .into_iter()
            .enumerate()
        {
            assert_eq!(
                index.cluster_range(i as u32).await.unwrap(),
                ClusterByteRange { start, length }
            );
        }
    }
}
#[tokio::test]
async fn search_clamps_results_offsets_and_can_be_cancelled_with_ready_source() {
    let mut builder = support::ZimBuilder::default();
    for i in 0..500 {
        builder = builder.add_article(
            b'A',
            format!("{i:04}"),
            format!("Article {i:04}"),
            "text/html",
            b"<p>test</p>".to_vec(),
        );
    }
    let mut bytes = vec![];
    builder.write(&mut bytes);
    let src = source(bytes);
    let index = Arc::new(
        RemoteZimIndex::open(src.clone(), Default::default())
            .await
            .unwrap(),
    );
    let archive = RangeZimArchive::open(
        index.clone(),
        Arc::new(Blobs {
            source: src,
            calls: Mutex::new(0),
        }),
        seed(),
        Default::default(),
    )
    .await
    .unwrap();
    let hits = archive
        .search_titles(TitleSearchQuery {
            text: "Article".into(),
            offset: 10,
            limit: u32::MAX,
        })
        .await
        .unwrap();
    assert_eq!(hits.len(), 200);
    assert_eq!(hits[0].title, "Article 0010");
    assert_eq!(hits[199].title, "Article 0209");
    assert!(index.cache_counts().await.1 <= 8);
    let future = archive.search_titles(TitleSearchQuery {
        text: "Article".into(),
        offset: 0,
        limit: 1,
    });
    futures_util::pin_mut!(future);
    let mut context = std::task::Context::from_waker(futures_util::task::noop_waker_ref());
    assert!(std::future::Future::poll(future.as_mut(), &mut context).is_pending());
}
struct Offline;
#[async_trait]
impl ClusterBlobSource for Offline {
    async fn read_blob(
        &self,
        _: u32,
        _: ClusterByteRange,
        _: u32,
        _: u64,
    ) -> Result<Bytes, ZimError> {
        Err(ZimError::Io("offline_missing_cluster".into()))
    }
}
#[tokio::test]
async fn offline_missing_payload_and_metadata_remain_errors() {
    let index = Arc::new(
        RemoteZimIndex::open(source(fixture()), Default::default())
            .await
            .unwrap(),
    );
    let archive = RangeZimArchive::open(index, Arc::new(Offline), seed(), Default::default())
        .await
        .unwrap();
    assert!(
        matches!(archive.article(&ArticleKey("A/Café".into())).await,Err(ZimError::Io(s)) if s=="offline_missing_cluster")
    );
    assert!(
        matches!(archive.resource(&EntryKey("C/pixel.txt".into())).await,Err(ZimError::Io(s)) if s=="offline_missing_cluster")
    );
    struct MissingMetadata;
    #[async_trait]
    impl RandomAccessSource for MissingMetadata {
        fn len(&self) -> u64 {
            1024
        }
        async fn read_range(&self, _: u64, _: u64) -> Result<Bytes, ZimError> {
            Err(ZimError::Io("offline_missing_page".into()))
        }
    }
    assert!(
        matches!(RemoteZimIndex::open(Arc::new(MissingMetadata),Default::default()).await,Err(ZimError::Io(s)) if s=="offline_missing_page")
    );
}
#[tokio::test]
async fn invalid_cluster_indices_special_records_and_response_sizes() {
    let mut bytes = fixture();
    let h = parse_header(&bytes).unwrap();
    let ptr = u64at(&bytes, h.path_ptr_pos as usize) as usize;
    bytes[ptr + 8..ptr + 12].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(RemoteZimIndex::open(source(bytes), Default::default())
        .await
        .unwrap()
        .entry_at(0)
        .await
        .is_err());
    for tag in [0xfffdu16, 0xfffe] {
        let mut bytes = fixture();
        let ptr = u64at(&bytes, h.path_ptr_pos as usize) as usize;
        let mut record = vec![];
        record.extend_from_slice(&tag.to_le_bytes());
        record.extend_from_slice(&[0, b'X', 0, 0, 0, 0, b'd', 0, b'd', 0]);
        bytes[ptr..ptr + record.len()].copy_from_slice(&record);
        let entry = RemoteZimIndex::open(source(bytes), Default::default())
            .await
            .unwrap()
            .entry_at(0)
            .await
            .unwrap();
        assert_eq!(entry.index, 0);
        assert_eq!(entry.key.0, "X/d");
        assert!(entry.mime_type.is_none());
    }
    let mut bytes = fixture();
    let table = u64at(&bytes, 48) as usize;
    let start = u64at(&bytes, table);
    set64(&mut bytes, table + 8, start);
    assert!(RemoteZimIndex::open(source(bytes), Default::default())
        .await
        .unwrap()
        .cluster_range(0)
        .await
        .is_err());
    assert!(RemoteZimIndex::open(
        source(fixture()),
        RangeReadLimits {
            max_mime_bytes: 4,
            ..Default::default()
        }
    )
    .await
    .is_err());
    assert!(
        RemoteZimIndex::open(source(vec![0; 79]), Default::default())
            .await
            .is_err()
    );
    struct Excess;
    #[async_trait]
    impl RandomAccessSource for Excess {
        fn len(&self) -> u64 {
            100
        }
        async fn read_range(&self, _: u64, length: u64) -> Result<Bytes, ZimError> {
            Ok(vec![0; length as usize + 1].into())
        }
    }
    assert!(PagedRangeReader::new(Arc::new(Excess), Default::default())
        .unwrap()
        .read_exact(0, 80)
        .await
        .is_err());
}
