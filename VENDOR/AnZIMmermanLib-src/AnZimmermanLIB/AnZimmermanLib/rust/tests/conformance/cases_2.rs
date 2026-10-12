use super::*;

#[tokio::test]
async fn engine_id_and_version_match_release_table() {
    let engine = AnZimmermanZimEngine::new();
    assert_eq!(engine.engine_id(), ENGINE_ID);
    assert_eq!(engine.engine_id(), "zim.anzimmerman.v1");
    let v = engine.engine_version();
    assert!(!v.is_empty(), "engine_version must not be empty");
}

#[tokio::test]
async fn main_page_resolves_to_article_key() {
    let builder = ZimBuilder {
        main_page_index: Some(0),
        ..ZimBuilder::default()
    }
    .add_article(
        b'A',
        "Home",
        "Home",
        "text/html",
        minimal_html_article("Home", "<p>home</p>"),
    );
    let path = support::write_temp(&builder);
    let archive = open(&path.to_string_lossy()).await;
    let mp = archive
        .main_page()
        .await
        .expect("main_page")
        .expect("some main page");
    assert_eq!(mp.0, "A/Home");
}

#[tokio::test]
async fn entries_stream_yields_descriptors() {
    let builder = ZimBuilder::default()
        .add_article(
            b'A',
            "X",
            "X",
            "text/html",
            minimal_html_article("X", "<p>x</p>"),
        )
        .add_article(
            b'A',
            "Y",
            "Y",
            "text/html",
            minimal_html_article("Y", "<p>y</p>"),
        );
    let path = support::write_temp(&builder);
    let archive = open(&path.to_string_lossy()).await;
    let mut stream = archive.entries().await.expect("entries");
    use futures_util::StreamExt;
    let mut collected: Vec<String> = Vec::new();
    while let Some(item) = stream.next().await {
        let desc = item.expect("entry");
        collected.push(desc.entry_key.0);
    }
    assert_eq!(collected, vec!["A/X", "A/Y"]);
}

#[tokio::test]
async fn verify_structure_passes_for_minimal_archive() {
    let builder = ZimBuilder::default().add_article(
        b'A',
        "X",
        "X",
        "text/html",
        minimal_html_article("X", "<p>x</p>"),
    );
    let path = support::write_temp(&builder);
    let archive = open(&path.to_string_lossy()).await;
    let report = archive.verify_structure().await.expect("verify");
    assert!(report.valid, "errors: {:?}", report.errors);
    assert_eq!(report.engine_id, "zim.anzimmerman.v1");
    assert!(report.checked_entries >= 1);
}

#[tokio::test]
async fn zlib_clusters_decompress_correctly() {
    // Sanity: when zlib_clusters is set, the engine still reads back the
    // original bytes — i.e. the "raw DEFLATE" assumption in our writer
    // matches the reader's `DeflateDecoder` usage.
    let builder = ZimBuilder {
        zlib_clusters: true,
        ..ZimBuilder::default()
    }
    .add_article(
        b'A',
        "Compressed",
        "Compressed",
        "text/html",
        minimal_html_article("Compressed", "<p>deflate round-trip works</p>"),
    );
    let path = support::write_temp(&builder);
    let archive = open(&path.to_string_lossy()).await;
    let doc = archive
        .article(&anzimmerman::identity::ArticleKey("A/Compressed".into()))
        .await
        .expect("article");
    assert!(doc.normalized_text.contains("deflate round-trip works"));
}

#[tokio::test]
async fn unreadable_path_returns_unreadable_path_variant() {
    let req = ZimOpenRequest {
        path: "/nonexistent/path/that/does/not/exist.zim".into(),
        expected_sha256: None,
        expected_size_bytes: None,
    };
    let err = match AnZimmermanZimEngine::new().open(req).await {
        Ok(_) => panic!("expected UnreadablePath, got Ok"),
        Err(e) => e,
    };
    assert!(matches!(err, ZimError::UnreadablePath), "got {:?}", err);
}

#[tokio::test]
async fn wrong_magic_returns_invalid_header() {
    let mut buf = Vec::new();
    ZimBuilder::default()
        .add_article(
            b'A',
            "X",
            "X",
            "text/html",
            minimal_html_article("X", "<p>x</p>"),
        )
        .write(&mut buf);
    // Corrupt the magic.
    buf[0..4].copy_from_slice(&0xDEAD_BEEFu32.to_le_bytes());
    let tmp = tempfile::NamedTempFile::new().expect("tempfile");
    std::io::Write::write_all(&mut tmp.reopen().unwrap(), &buf).unwrap();
    let path = tmp.into_temp_path().keep().unwrap();

    let req = ZimOpenRequest {
        path: path.to_string_lossy().to_string(),
        expected_sha256: None,
        expected_size_bytes: None,
    };
    let err = match AnZimmermanZimEngine::new().open(req).await {
        Ok(_) => panic!("expected InvalidHeader (magic), got Ok"),
        Err(e) => e,
    };
    assert!(matches!(err, ZimError::InvalidHeader(_)), "got {:?}", err);
}

#[tokio::test]
async fn length_mismatch_detected() {
    let mut buf = Vec::new();
    ZimBuilder::default()
        .add_article(
            b'A',
            "X",
            "X",
            "text/html",
            minimal_html_article("X", "<p>x</p>"),
        )
        .write(&mut buf);
    let tmp = tempfile::NamedTempFile::new().expect("tempfile");
    std::io::Write::write_all(&mut tmp.reopen().unwrap(), &buf).unwrap();
    let path = tmp.into_temp_path().keep().unwrap();

    let req = ZimOpenRequest {
        path: path.to_string_lossy().to_string(),
        expected_sha256: None,
        expected_size_bytes: Some(buf.len() as u64 + 999),
    };
    let err = match AnZimmermanZimEngine::new().open(req).await {
        Ok(_) => panic!("expected LengthMismatch, got Ok"),
        Err(e) => e,
    };
    assert!(matches!(err, ZimError::LengthMismatch), "got {:?}", err);
}

#[tokio::test]
async fn digest_mismatch_detected() {
    let mut buf = Vec::new();
    ZimBuilder::default()
        .add_article(
            b'A',
            "X",
            "X",
            "text/html",
            minimal_html_article("X", "<p>x</p>"),
        )
        .write(&mut buf);
    let tmp = tempfile::NamedTempFile::new().expect("tempfile");
    std::io::Write::write_all(&mut tmp.reopen().unwrap(), &buf).unwrap();
    let path = tmp.into_temp_path().keep().unwrap();

    let req = ZimOpenRequest {
        path: path.to_string_lossy().to_string(),
        expected_sha256: Some("00".repeat(32)),
        expected_size_bytes: None,
    };
    let err = match AnZimmermanZimEngine::new().open(req).await {
        Ok(_) => panic!("expected DigestMismatch, got Ok"),
        Err(e) => e,
    };
    assert!(matches!(err, ZimError::DigestMismatch), "got {:?}", err);
}

#[tokio::test]
async fn digest_match_passes() {
    let mut buf = Vec::new();
    ZimBuilder::default()
        .add_article(
            b'A',
            "X",
            "X",
            "text/html",
            minimal_html_article("X", "<p>x</p>"),
        )
        .write(&mut buf);
    use sha2::{Digest as _, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(&buf);
    let hex = format!("{:x}", hasher.finalize());

    let tmp = tempfile::NamedTempFile::new().expect("tempfile");
    std::io::Write::write_all(&mut tmp.reopen().unwrap(), &buf).unwrap();
    let path = tmp.into_temp_path().keep().unwrap();

    let req = ZimOpenRequest {
        path: path.to_string_lossy().to_string(),
        expected_sha256: Some(hex),
        expected_size_bytes: None,
    };
    let _archive = AnZimmermanZimEngine::new()
        .open(req)
        .await
        .expect("digest matches — open succeeds");
}

#[tokio::test]
async fn title_search_ranks_prefix_above_substring() {
    let builder = ZimBuilder::default()
        .add_article(
            b'A',
            "Python",
            "Python",
            "text/html",
            minimal_html_article("Python", "<p>x</p>"),
        )
        .add_article(
            b'A',
            "Python_(programming_language)",
            "Python (programming language)",
            "text/html",
            minimal_html_article("Python", "<p>x</p>"),
        )
        .add_article(
            b'A',
            "Monty_Python",
            "Monty Python",
            "text/html",
            minimal_html_article("Python", "<p>x</p>"),
        )
        .add_article(
            b'A',
            "Reptile",
            "Reptile",
            "text/html",
            minimal_html_article("Reptile", "<p>x</p>"),
        );
    let path = support::write_temp(&builder);
    let archive = open(&path.to_string_lossy()).await;
    let hits = archive
        .search_titles(TitleSearchQuery {
            text: "python".into(),
            offset: 0,
            limit: 10,
        })
        .await
        .expect("search");
    // Exact match ("Python") should be the first hit; no reptile hit.
    assert!(!hits.is_empty());
    assert_eq!(hits[0].title, "Python");
    assert!(hits.iter().all(|h| h.title != "Reptile"));
}

#[tokio::test]
async fn redirect_to_missing_target_errors() {
    let builder = ZimBuilder::default().add_redirect(b'A', "Dangling", "Dangling", 999);
    let path = support::write_temp(&builder);
    let archive = open(&path.to_string_lossy()).await;
    let err = match archive
        .resolve_redirect(&anzimmerman::identity::EntryKey("A/Dangling".into()), 16)
        .await
    {
        Ok(_) => panic!("expected EntryNotFound, got Ok"),
        Err(e) => e,
    };
    assert!(matches!(err, ZimError::EntryNotFound(_)), "got {:?}", err);
}

#[tokio::test]
async fn entry_not_found_for_unknown_path() {
    let builder = ZimBuilder::default().add_article(
        b'A',
        "Real",
        "Real",
        "text/html",
        minimal_html_article("Real", "<p>x</p>"),
    );
    let path = support::write_temp(&builder);
    let archive = open(&path.to_string_lossy()).await;
    let err = match archive
        .entry(&anzimmerman::identity::EntryKey("A/Missing".into()))
        .await
    {
        Ok(_) => panic!("expected EntryNotFound, got Ok"),
        Err(e) => e,
    };
    assert!(matches!(err, ZimError::EntryNotFound(_)), "got {:?}", err);
}


// Silence unused-import warnings when the upstream constants are referenced
// only for documentation alignment.
#[allow(dead_code)]
const _GUARDS: (u32, u32) = (ZIM_MAGIC, NO_PAGE);
