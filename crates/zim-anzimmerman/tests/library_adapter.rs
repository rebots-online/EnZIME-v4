use anzimmerman::ZimArchive as _;
use futures_util::StreamExt;
use ports::zim::ZimArchive as _;
use std::io::Write;
const FIXTURE: &[u8] = include_bytes!("../../../VENDOR/AnZIMmermanLib-src/AnZimmermanLIB/AnZimmermanLib/tests/fixtures/rust-consolidation.zim");
fn same_json(a: impl serde::Serialize, b: impl serde::Serialize) {
    assert_eq!(
        serde_json::to_value(a).unwrap(),
        serde_json::to_value(b).unwrap()
    );
}
#[tokio::test]
async fn canonical_and_adapter_preserve_fields_paths_layout_and_stream() {
    let mut file = tempfile::NamedTempFile::new().unwrap();
    file.write_all(FIXTURE).unwrap();
    let path = file.path().to_str().unwrap().to_owned();
    let native = anzimmerman::archive::open_archive(&anzimmerman::ZimOpenRequest {
        path: path.clone(),
        expected_sha256: None,
        expected_size_bytes: None,
    })
    .await
    .unwrap();
    let adapter = zim_anzimmerman::archive::open_archive(&ports::zim::ZimOpenRequest {
        path,
        expected_sha256: None,
        expected_size_bytes: None,
    })
    .await
    .unwrap();
    same_json(native.metadata(), adapter.metadata());
    same_json(
        native.main_page().await.unwrap(),
        adapter.main_page().await.unwrap(),
    );
    for key in ["A/Café", "A/Home", "I/pixel.txt"] {
        same_json(
            native
                .entry(&anzimmerman::EntryKey(key.into()))
                .await
                .unwrap(),
            adapter
                .entry(&domain::identity::EntryKey(key.into()))
                .await
                .unwrap(),
        );
        same_json(
            native
                .resolve_redirect(&anzimmerman::EntryKey(key.into()), 8)
                .await
                .unwrap(),
            adapter
                .resolve_redirect(&domain::identity::EntryKey(key.into()), 8)
                .await
                .unwrap(),
        );
    }
    let article = adapter
        .article(&domain::identity::ArticleKey("A/Café".into()))
        .await
        .unwrap();
    assert_eq!(article.raw_html.as_ref(), b"<p>Library fixture</p>");
    same_json(
        native
            .article(&anzimmerman::ArticleKey("A/Café".into()))
            .await
            .unwrap(),
        article,
    );
    same_json(
        native
            .resource(&anzimmerman::EntryKey("I/pixel.txt".into()))
            .await
            .unwrap(),
        adapter
            .resource(&domain::identity::EntryKey("I/pixel.txt".into()))
            .await
            .unwrap(),
    );
    let mut ns = native.entries().await.unwrap();
    let mut ads = adapter.entries().await.unwrap();
    while let Some(entry) = ns.next().await {
        same_json(entry.unwrap(), ads.next().await.unwrap().unwrap());
    }
    assert!(ads.next().await.is_none());
    let n = native.layout_snapshot().unwrap();
    let a = adapter.layout_snapshot().unwrap();
    assert_eq!(n.cluster_byte_ranges, a.cluster_byte_ranges);
    assert_eq!(n.entries.len(), a.entries.len());
    for (n, a) in n.entries.into_iter().zip(a.entries) {
        assert_eq!(
            (
                n.entry_key,
                n.namespace,
                n.path,
                n.title,
                n.mime_type,
                n.cluster_index,
                n.blob_index
            ),
            (
                a.entry_key,
                a.namespace,
                a.path,
                a.title,
                a.mime_type,
                a.cluster_index,
                a.blob_index
            )
        );
        same_json(n.kind, a.kind);
        same_json(n.redirect_target, a.redirect_target);
    }
    same_json(
        native
            .search_titles(anzimmerman::TitleSearchQuery {
                text: "Caf".into(),
                offset: 0,
                limit: 2,
            })
            .await
            .unwrap(),
        adapter
            .search_titles(ports::zim::TitleSearchQuery {
                text: "Caf".into(),
                offset: 0,
                limit: 2,
            })
            .await
            .unwrap(),
    );
    same_json(
        native.verify_structure().await.unwrap(),
        adapter.verify_structure().await.unwrap(),
    );
}
#[test]
fn malformed_header_errors_keep_payload() {
    for bytes in [&[][..], &[0; 80][..]] {
        let canonical = anzimmerman::header::parse_header(bytes).unwrap_err();
        let adapter = zim_anzimmerman::header::parse_header(bytes).unwrap_err();
        assert_eq!(canonical.to_string(), adapter.to_string());
        assert!(matches!(adapter, ports::zim::ZimError::InvalidHeader(_)));
    }
}
