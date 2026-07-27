//! §7.4 conformance fixtures, executed as real integration assertions.
//!
//! Each test name matches one fixture ID from ARCHITECTURE.md §7.4. The
//! `support::ZimBuilder` writes genuine v6 ZIM bytes; the assertions
//! exercise the engine against the resulting file.

mod support;

use std::sync::Arc;

use ports::zim::{
    TitleSearchQuery, ZimArchive, ZimEngine, ZimError, ZimOpenRequest,
};
use support::{ZimBuilder, NO_PAGE, ZIM_MAGIC};
use zim_anzimmerman::{AnZimmermanZimEngine, ENGINE_ID};

async fn open(path: &str) -> Arc<dyn ZimArchive> {
    let req = ZimOpenRequest {
        path: path.to_string(),
        expected_sha256: None,
        expected_size_bytes: None,
    };
    AnZimmermanZimEngine::new()
        .open(req)
        .await
        .expect("open archive")
}

fn minimal_html_article(title: &str, body: &str) -> Vec<u8> {
    format!("<html><head><title>{}</title></head><body>{}</body></html>", title, body)
        .into_bytes()
}

#[tokio::test]
async fn zim_minimal_article_v1() {
    // Fixture: one HTML article plus the standard metadata entries.
    let builder = ZimBuilder {
        metadata: vec![
            ("Title".into(), "Main Page".into()),
            ("Description".into(), "A one-article ZIM".into()),
            ("Language".into(), "eng".into()),
            ("Creator".into(), "conformance".into()),
            ("Publisher".into(), "conformance".into()),
            ("Date".into(), "2026-07-27".into()),
            ("Counter".into(), "article=1;text/plain;charset=UTF-8=6".into()),
        ],
        main_page_index: Some(0),
        ..ZimBuilder::default()
    }
    .add_article(
        b'A',
        "Main_Page",
        "Main Page",
        "text/html",
        minimal_html_article("Main Page", "<h1>Hello ZIM</h1><p>World.</p>"),
    );
    let path = support::write_temp(&builder);

    let req = ZimOpenRequest {
        path: path.to_string_lossy().to_string(),
        expected_sha256: None,
        expected_size_bytes: None,
    };
    let engine = AnZimmermanZimEngine::new();
    let metadata = engine.probe(&req).await.expect("probe");
    assert_eq!(metadata.article_count, 1, "M/Counter absent -> header count");
    assert_eq!(metadata.title, "Main Page");
    assert_eq!(metadata.creator, "conformance");
    assert_eq!(metadata.language_bcp47, "eng");

    let archive = open(&path.to_string_lossy()).await;
    let doc = archive
        .article(&domain::identity::ArticleKey("A/Main_Page".into()))
        .await
        .expect("article");
    assert_eq!(doc.title, "Main Page");
    assert!(doc.normalized_text.contains("Hello ZIM"));
    assert!(doc.normalized_text.contains("World."));
    assert_eq!(doc.content_digest.algorithm, domain::identity::DigestAlgorithm::Sha256);
    assert_eq!(doc.content_digest.hex.len(), 64);
}

#[tokio::test]
async fn zim_redirect_chain_v1() {
    // Fixture: three-hop redirect chain A -> B -> C -> D, all valid.
    let builder = ZimBuilder::default()
        .add_article(b'A', "Target", "Target Article", "text/html", minimal_html_article("Target", "<p>resolved</p>"))
        // path index 1 -> redirect to 2
        .add_redirect(b'A', "Hop1", "Hop1", 2)
        // path index 2 -> redirect to 3
        .add_redirect(b'A', "Hop2", "Hop2", 3)
        // path index 3 -> redirect to 0 (Target)
        .add_redirect(b'A', "Hop3", "Hop3", 0);
    let path = support::write_temp(&builder);
    let archive = open(&path.to_string_lossy()).await;

    let resolved = archive
        .resolve_redirect(&domain::identity::EntryKey("A/Hop1".into()), 16)
        .await
        .expect("resolve");
    assert_eq!(resolved.0, "A/Target");
}

#[tokio::test]
async fn zim_redirect_cycle_v1() {
    // Fixture: A redirects to B redirects back to A — must be rejected.
    let builder = ZimBuilder::default()
        .add_redirect(b'A', "A", "A", 1)
        .add_redirect(b'A', "B", "B", 0);
    let path = support::write_temp(&builder);
    let archive = open(&path.to_string_lossy()).await;
    let err = match archive
        .resolve_redirect(&domain::identity::EntryKey("A/A".into()), 16)
        .await
    {
        Ok(_) => panic!("expected RedirectCycle, got Ok"),
        Err(e) => e,
    };
    assert!(matches!(err, ZimError::RedirectCycle), "got {:?}", err);
}

#[tokio::test]
async fn zim_resource_matrix_v1() {
    // Fixture: CSS, image, SVG, audio — all readable as resources.
    let css = b"body{color:black}".to_vec();
    let png = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    let svg = b"<svg xmlns='http://www.w3.org/2000/svg'/>".to_vec();
    let audio = b"ID3fake-audio-bytes".to_vec();
    let builder = ZimBuilder::default()
        .add_article(b'A', "Article", "Article", "text/html", minimal_html_article("Article", "<p>x</p>"))
        .add_article(b'S', "style.css", "style.css", "text/css", css.clone())
        .add_article(b'I', "img.png", "img.png", "image/png", png.clone())
        .add_article(b'I', "img.svg", "img.svg", "image/svg+xml", svg.clone())
        .add_article(b'W', "sound.opus", "sound.opus", "audio/ogg", audio.clone());
    let path = support::write_temp(&builder);
    let archive = open(&path.to_string_lossy()).await;

    let css_doc = archive
        .resource(&domain::identity::EntryKey("S/style.css".into()))
        .await
        .expect("css");
    assert_eq!(css_doc.mime_type, "text/css");
    assert_eq!(&css_doc.bytes[..], &css[..]);

    let png_doc = archive
        .resource(&domain::identity::EntryKey("I/img.png".into()))
        .await
        .expect("png");
    assert_eq!(png_doc.mime_type, "image/png");
    assert_eq!(&png_doc.bytes[..], &png[..]);

    let svg_doc = archive
        .resource(&domain::identity::EntryKey("I/img.svg".into()))
        .await
        .expect("svg");
    assert_eq!(svg_doc.mime_type, "image/svg+xml");
    assert_eq!(&svg_doc.bytes[..], &svg[..]);

    let audio_doc = archive
        .resource(&domain::identity::EntryKey("W/sound.opus".into()))
        .await
        .expect("audio");
    assert_eq!(audio_doc.mime_type, "audio/ogg");
    assert_eq!(&audio_doc.bytes[..], &audio[..]);
}

#[tokio::test]
async fn zim_unicode_paths_v1() {
    // Fixture: non-ASCII URLs and titles survive a round trip.
    let builder = ZimBuilder::default().add_article(
        b'A',
        "Москва/Привет",
        "Москва — Привет, мир!",
        "text/html",
        minimal_html_article("Москва", "<p>Здравствуй, мир!</p>"),
    );
    let path = support::write_temp(&builder);
    let archive = open(&path.to_string_lossy()).await;

    let desc = archive
        .entry(&domain::identity::EntryKey("A/Москва/Привет".into()))
        .await
        .expect("unicode entry");
    assert_eq!(desc.path, "Москва/Привет");
    assert_eq!(desc.title, "Москва — Привет, мир!");

    let doc = archive
        .article(&domain::identity::ArticleKey("A/Москва/Привет".into()))
        .await
        .expect("unicode article");
    assert!(doc.normalized_text.contains("Здравствуй, мир!"));
}

#[tokio::test]
async fn zim_large_cluster_guard_v1() {
    // Fixture: a real raw-DEFLATE decompression bomb. The compressed payload
    // is a few MiB; its decompressed form is `BOMB_BYTES` (>> the engine's
    // 2 GiB decompression budget). Engine must refuse with `DecompressionLimit`.
    const BOMB_BYTES: u64 = 3 * 1024 * 1024 * 1024; // 3 GiB > MAX_DECOMPRESSED_CLUSTER_BYTES

    struct ZeroReader {
        remaining: u64,
    }
    impl std::io::Read for ZeroReader {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            if self.remaining == 0 {
                return Ok(0);
            }
            let n = (buf.len() as u64).min(self.remaining) as usize;
            for b in &mut buf[..n] {
                *b = 0;
            }
            self.remaining -= n as u64;
            Ok(n)
        }
    }

    // Decompressed payload = [offset_table] || [3 GiB of zeros].
    // offset_table: [8, 3GiB+8] (start, end of the single blob).
    let mut inner = Vec::with_capacity(8);
    inner.extend_from_slice(&8u32.to_le_bytes());
    inner.extend_from_slice(&(8u32.wrapping_add(BOMB_BYTES as u32)).to_le_bytes());

    use std::io::Write;
    let mut enc = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    enc.write_all(&inner).unwrap();
    let mut reader = ZeroReader { remaining: BOMB_BYTES };
    std::io::copy(&mut reader, &mut enc).unwrap();
    let mut zlib = enc.finish().expect("encode");
    // ZlibEncoder wraps raw DEFLATE in a 2-byte header and 4-byte adler32
    // trailer; ZIM clusters store raw DEFLATE alone.
    zlib.drain(0..2);
    zlib.truncate(zlib.len().saturating_sub(4));
    let mut on_disk = Vec::with_capacity(1 + zlib.len());
    on_disk.push(2u8); // COMPRESSION_ZLIB
    on_disk.extend_from_slice(&zlib);

    let builder = ZimBuilder {
        raw_cluster_override: Some(on_disk),
        ..ZimBuilder::default()
    }
    .add_article(b'A', "Big", "Big", "text/html", b"<p>x</p>".to_vec());
    let path = support::write_temp(&builder);
    let archive = open(&path.to_string_lossy()).await;

    let err = match archive
        .article(&domain::identity::ArticleKey("A/Big".into()))
        .await
    {
        Ok(_) => panic!("expected DecompressionLimit, got Ok"),
        Err(e) => e,
    };
    assert!(
        matches!(err, ZimError::DecompressionLimit),
        "got {:?}",
        err
    );
}

#[tokio::test]
async fn zim_truncated_header_v1() {
    // Fixture: file shorter than the 80-byte header.
    let tmp = tempfile::NamedTempFile::new().expect("tempfile");
    std::io::Write::write_all(&mut tmp.reopen().unwrap(), &[0u8; 30]).unwrap();
    let path = tmp.into_temp_path().keep().unwrap();

    let req = ZimOpenRequest {
        path: path.to_string_lossy().to_string(),
        expected_sha256: None,
        expected_size_bytes: None,
    };
    let err = match AnZimmermanZimEngine::new().open(req).await {
        Ok(_) => panic!("expected InvalidHeader, got Ok"),
        Err(e) => e,
    };
    assert!(
        matches!(err, ZimError::InvalidHeader(ref msg) if msg.contains("80") || msg.contains("header")),
        "got {:?}",
        err
    );
}

#[tokio::test]
async fn zim_out_of_bounds_pointer_v1() {
    // Fixture: file whose path pointer table points past EOF.
    let mut bytes = Vec::new();
    let mut builder = ZimBuilder::default()
        .add_article(b'A', "X", "X", "text/html", minimal_html_article("X", "<p>x</p>"));
    builder.main_page_index = None;
    builder.write(&mut bytes);
    // Corrupt the first path pointer so it points past EOF.
    let path_ptr_table_pos = read_u64_le(&bytes, 32) as usize;
    let file_len = bytes.len();
    bytes[path_ptr_table_pos..path_ptr_table_pos + 8]
        .copy_from_slice(&((file_len as u64) * 2).to_le_bytes());

    let tmp = tempfile::NamedTempFile::new().expect("tempfile");
    std::io::Write::write_all(&mut tmp.reopen().unwrap(), &bytes).unwrap();
    let path = tmp.into_temp_path().keep().unwrap();

    let req = ZimOpenRequest {
        path: path.to_string_lossy().to_string(),
        expected_sha256: None,
        expected_size_bytes: None,
    };
    let err = match AnZimmermanZimEngine::new().open(req).await {
        Ok(_) => panic!("expected bounds error, got Ok"),
        Err(e) => e,
    };
    assert!(
        matches!(
            err,
            ZimError::InvalidHeader(_) | ZimError::AllocationLimit | ZimError::Io(_)
        ),
        "got {:?}",
        err
    );
}

#[tokio::test]
async fn zim_internal_links_v1() {
    // Fixture: deterministic link rewriting inside an article.
    let html = b"<html><body>\
        <a href='A/Foo'>foo</a>\
        <a href='I/img.png'>img</a>\
        <a href='https://example.com/'>ex</a>\
        <a href='#frag'>frag</a>\
        <a href='Bare'>bare</a>\
        </body></html>"
        .to_vec();
    let builder = ZimBuilder::default()
        .add_article(b'A', "Links", "Links", "text/html", html);
    let path = support::write_temp(&builder);
    let archive = open(&path.to_string_lossy()).await;

    let doc = archive
        .article(&domain::identity::ArticleKey("A/Links".into()))
        .await
        .expect("article");
    use ports::zim::InternalLinkTarget as T;
    let kinds: Vec<&str> = doc
        .internal_links
        .iter()
        .map(|l| match &l.target {
            T::Article { .. } => "A",
            T::Resource { .. } => "R",
            T::Unsupported { .. } => "U",
        })
        .collect();
    assert_eq!(kinds, vec!["A", "R", "U", "U", "A"]);
}

#[tokio::test]
async fn zim_scripted_article_v1_text_extraction() {
    // The §7.3 rendering-boundary sanitizer runs upstream of this crate;
    // this fixture exercises the crate's own contract — that the
    // `ArticleDocument.normalized_text` contains zero leakage of <script>
    // bodies, and that `raw_html` is delivered unchanged for the
    // upstream sanitizer to process.
    let html = b"<html><body>\
        <h1>Headline</h1>\
        <script>document.cookie='leak'</script>\
        <style>.x{color:red}</style>\
        <p>Visible text.</p>\
        </body></html>"
        .to_vec();
    let builder = ZimBuilder::default()
        .add_article(b'A', "Scripted", "Scripted", "text/html", html.clone());
    let path = support::write_temp(&builder);
    let archive = open(&path.to_string_lossy()).await;

    let doc = archive
        .article(&domain::identity::ArticleKey("A/Scripted".into()))
        .await
        .expect("article");
    assert!(!doc.normalized_text.contains("document.cookie"));
    assert!(!doc.normalized_text.contains("leak"));
    assert!(doc.normalized_text.contains("Headline"));
    assert!(doc.normalized_text.contains("Visible text."));
    // raw_html is untouched — sanitizer runs upstream.
    assert_eq!(&doc.raw_html[..], &html[..]);
}

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
    .add_article(b'A', "Home", "Home", "text/html", minimal_html_article("Home", "<p>home</p>"));
    let path = support::write_temp(&builder);
    let archive = open(&path.to_string_lossy()).await;
    let mp = archive.main_page().await.expect("main_page").expect("some main page");
    assert_eq!(mp.0, "A/Home");
}

#[tokio::test]
async fn entries_stream_yields_descriptors() {
    let builder = ZimBuilder::default()
        .add_article(b'A', "X", "X", "text/html", minimal_html_article("X", "<p>x</p>"))
        .add_article(b'A', "Y", "Y", "text/html", minimal_html_article("Y", "<p>y</p>"));
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
    let builder = ZimBuilder::default()
        .add_article(b'A', "X", "X", "text/html", minimal_html_article("X", "<p>x</p>"));
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
        .article(&domain::identity::ArticleKey("A/Compressed".into()))
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
        .add_article(b'A', "X", "X", "text/html", minimal_html_article("X", "<p>x</p>"))
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
        .add_article(b'A', "X", "X", "text/html", minimal_html_article("X", "<p>x</p>"))
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
        .add_article(b'A', "X", "X", "text/html", minimal_html_article("X", "<p>x</p>"))
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
        .add_article(b'A', "X", "X", "text/html", minimal_html_article("X", "<p>x</p>"))
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
        .add_article(b'A', "Python", "Python", "text/html", minimal_html_article("Python", "<p>x</p>"))
        .add_article(b'A', "Python_(programming_language)", "Python (programming language)", "text/html", minimal_html_article("Python", "<p>x</p>"))
        .add_article(b'A', "Monty_Python", "Monty Python", "text/html", minimal_html_article("Python", "<p>x</p>"))
        .add_article(b'A', "Reptile", "Reptile", "text/html", minimal_html_article("Reptile", "<p>x</p>"));
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
    let builder = ZimBuilder::default()
        .add_redirect(b'A', "Dangling", "Dangling", 999);
    let path = support::write_temp(&builder);
    let archive = open(&path.to_string_lossy()).await;
    let err = match archive
        .resolve_redirect(&domain::identity::EntryKey("A/Dangling".into()), 16)
        .await
    {
        Ok(_) => panic!("expected EntryNotFound, got Ok"),
        Err(e) => e,
    };
    assert!(matches!(err, ZimError::EntryNotFound(_)), "got {:?}", err);
}

#[tokio::test]
async fn entry_not_found_for_unknown_path() {
    let builder = ZimBuilder::default()
        .add_article(b'A', "Real", "Real", "text/html", minimal_html_article("Real", "<p>x</p>"));
    let path = support::write_temp(&builder);
    let archive = open(&path.to_string_lossy()).await;
    let err = match archive
        .entry(&domain::identity::EntryKey("A/Missing".into()))
        .await
    {
        Ok(_) => panic!("expected EntryNotFound, got Ok"),
        Err(e) => e,
    };
    assert!(matches!(err, ZimError::EntryNotFound(_)), "got {:?}", err);
}

fn read_u64_le(buf: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(buf[at..at + 8].try_into().unwrap())
}

// Silence unused-import warnings when the upstream constants are referenced
// only for documentation alignment.
#[allow(dead_code)]
const _GUARDS: (u32, u32) = (ZIM_MAGIC, NO_PAGE);
