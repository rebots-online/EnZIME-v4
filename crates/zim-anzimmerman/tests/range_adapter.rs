use domain::identity::{ArticleKey, EntryKey};
use ports::zim::{TitleSearchQuery, ZimArchive};
use std::sync::Arc;
use zim_anzimmerman::RangeArchiveAdapter;
#[tokio::test]
async fn canonical_archive_conversion_preserves_utf8_and_payloads() {
    let path=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../VENDOR/AnZIMmermanLib-src/AnZimmermanLIB/AnZimmermanLib/tests/fixtures/rust-consolidation.zim");
    let inner = anzimmerman::archive::open_archive(&anzimmerman::model::ZimOpenRequest {
        path: path.to_string_lossy().into_owned(),
        expected_sha256: None,
        expected_size_bytes: None,
    })
    .await
    .unwrap();
    let adapter = RangeArchiveAdapter::new(Arc::new(inner));
    assert_eq!(adapter.main_page().await.unwrap().unwrap().0, "A/Home");
    assert_eq!(
        adapter
            .resolve_redirect(&EntryKey("A/Home".into()), 16)
            .await
            .unwrap()
            .0,
        "A/Café"
    );
    assert!(adapter
        .article(&ArticleKey("A/Café".into()))
        .await
        .unwrap()
        .normalized_text
        .contains("Library fixture"));
    assert_eq!(
        adapter
            .resource(&EntryKey("I/pixel.txt".into()))
            .await
            .unwrap()
            .bytes
            .as_ref(),
        b"pixel"
    );
    assert!(!adapter
        .search_titles(TitleSearchQuery {
            text: "Café".into(),
            offset: 0,
            limit: 10
        })
        .await
        .unwrap()
        .is_empty());
}
