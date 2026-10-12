use super::*;

#[tokio::test]
async fn shared_fixture_matches_expected() {
    use sha2::{Digest, Sha256};
    use std::io::Write;
    const BYTES: &[u8] = include_bytes!("../../../tests/fixtures/rust-consolidation.zim");
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/rust-consolidation.expected.json"
    ))
    .unwrap();
    assert_eq!(hex::encode(Sha256::digest(BYTES)), expected["sha256"]);
    let mut file = tempfile::NamedTempFile::new().unwrap();
    file.write_all(BYTES).unwrap();
    let archive = open(file.path().to_str().unwrap()).await;
    assert_eq!(archive.metadata().uuid, expected["uuid"]);
    assert_eq!(
        archive.main_page().await.unwrap().unwrap().0,
        expected["main_page"]
    );
    for entry in expected["entries"].as_array().unwrap() {
        let key = entry["key"].as_str().unwrap();
        let actual = archive
            .entry(&anzimmerman::EntryKey(key.into()))
            .await
            .unwrap();
        assert_eq!(actual.title, entry["title"]);
        assert_eq!(actual.mime_type, entry["mime"]);
        if let Some(target) = entry.get("redirect") {
            assert_eq!(
                archive
                    .resolve_redirect(&anzimmerman::EntryKey(key.into()), 8)
                    .await
                    .unwrap()
                    .0,
                target.as_str().unwrap()
            );
        } else if key.starts_with("A/") {
            assert_eq!(
                archive
                    .article(&anzimmerman::ArticleKey(key.into()))
                    .await
                    .unwrap()
                    .raw_html
                    .as_ref(),
                entry["payload"].as_str().unwrap().as_bytes()
            );
        } else {
            assert_eq!(
                archive
                    .resource(&anzimmerman::EntryKey(key.into()))
                    .await
                    .unwrap()
                    .bytes
                    .as_ref(),
                entry["payload"].as_str().unwrap().as_bytes()
            );
        }
    }
}
