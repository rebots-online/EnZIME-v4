use super::*;

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
            (
                "Counter".into(),
                "article=1;text/plain;charset=UTF-8=6".into(),
            ),
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
    assert_eq!(
        metadata.article_count, 1,
        "M/Counter absent -> header count"
    );
    assert_eq!(metadata.title, "Main Page");
    assert_eq!(metadata.creator, "conformance");
    assert_eq!(metadata.language_bcp47, "eng");

    let archive = open(&path.to_string_lossy()).await;
    let doc = archive
        .article(&anzimmerman::identity::ArticleKey("A/Main_Page".into()))
        .await
        .expect("article");
    assert_eq!(doc.title, "Main Page");
    assert!(doc.normalized_text.contains("Hello ZIM"));
    assert!(doc.normalized_text.contains("World."));
    assert_eq!(
        doc.content_digest.algorithm,
        anzimmerman::identity::DigestAlgorithm::Sha256
    );
    assert_eq!(doc.content_digest.hex.len(), 64);
}

#[tokio::test]
async fn zim_redirect_chain_v1() {
    // Fixture: three-hop redirect chain A -> B -> C -> D, all valid.
    let builder = ZimBuilder::default()
        .add_article(
            b'A',
            "Target",
            "Target Article",
            "text/html",
            minimal_html_article("Target", "<p>resolved</p>"),
        )
        // path index 1 -> redirect to 2
        .add_redirect(b'A', "Hop1", "Hop1", 2)
        // path index 2 -> redirect to 3
        .add_redirect(b'A', "Hop2", "Hop2", 3)
        // path index 3 -> redirect to 0 (Target)
        .add_redirect(b'A', "Hop3", "Hop3", 0);
    let path = support::write_temp(&builder);
    let archive = open(&path.to_string_lossy()).await;

    let resolved = archive
        .resolve_redirect(&anzimmerman::identity::EntryKey("A/Hop1".into()), 16)
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
        .resolve_redirect(&anzimmerman::identity::EntryKey("A/A".into()), 16)
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
        .add_article(
            b'A',
            "Article",
            "Article",
            "text/html",
            minimal_html_article("Article", "<p>x</p>"),
        )
        .add_article(b'S', "style.css", "style.css", "text/css", css.clone())
        .add_article(b'I', "img.png", "img.png", "image/png", png.clone())
        .add_article(b'I', "img.svg", "img.svg", "image/svg+xml", svg.clone())
        .add_article(b'W', "sound.opus", "sound.opus", "audio/ogg", audio.clone());
    let path = support::write_temp(&builder);
    let archive = open(&path.to_string_lossy()).await;

    let css_doc = archive
        .resource(&anzimmerman::identity::EntryKey("S/style.css".into()))
        .await
        .expect("css");
    assert_eq!(css_doc.mime_type, "text/css");
    assert_eq!(&css_doc.bytes[..], &css[..]);

    let png_doc = archive
        .resource(&anzimmerman::identity::EntryKey("I/img.png".into()))
        .await
        .expect("png");
    assert_eq!(png_doc.mime_type, "image/png");
    assert_eq!(&png_doc.bytes[..], &png[..]);

    let svg_doc = archive
        .resource(&anzimmerman::identity::EntryKey("I/img.svg".into()))
        .await
        .expect("svg");
    assert_eq!(svg_doc.mime_type, "image/svg+xml");
    assert_eq!(&svg_doc.bytes[..], &svg[..]);

    let audio_doc = archive
        .resource(&anzimmerman::identity::EntryKey("W/sound.opus".into()))
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
        .entry(&anzimmerman::identity::EntryKey("A/Москва/Привет".into()))
        .await
        .expect("unicode entry");
    assert_eq!(desc.path, "Москва/Привет");
    assert_eq!(desc.title, "Москва — Привет, мир!");

    let doc = archive
        .article(&anzimmerman::identity::ArticleKey("A/Москва/Привет".into()))
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
    let mut reader = ZeroReader {
        remaining: BOMB_BYTES,
    };
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
        .article(&anzimmerman::identity::ArticleKey("A/Big".into()))
        .await
    {
        Ok(_) => panic!("expected DecompressionLimit, got Ok"),
        Err(e) => e,
    };
    assert!(matches!(err, ZimError::DecompressionLimit), "got {:?}", err);
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
    let mut builder = ZimBuilder::default().add_article(
        b'A',
        "X",
        "X",
        "text/html",
        minimal_html_article("X", "<p>x</p>"),
    );
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
    let builder = ZimBuilder::default().add_article(b'A', "Links", "Links", "text/html", html);
    let path = support::write_temp(&builder);
    let archive = open(&path.to_string_lossy()).await;

    let doc = archive
        .article(&anzimmerman::identity::ArticleKey("A/Links".into()))
        .await
        .expect("article");
    use anzimmerman::model::InternalLinkTarget as T;
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
    let builder =
        ZimBuilder::default().add_article(b'A', "Scripted", "Scripted", "text/html", html.clone());
    let path = support::write_temp(&builder);
    let archive = open(&path.to_string_lossy()).await;

    let doc = archive
        .article(&anzimmerman::identity::ArticleKey("A/Scripted".into()))
        .await
        .expect("article");
    assert!(!doc.normalized_text.contains("document.cookie"));
    assert!(!doc.normalized_text.contains("leak"));
    assert!(doc.normalized_text.contains("Headline"));
    assert!(doc.normalized_text.contains("Visible text."));
    // raw_html is untouched — sanitizer runs upstream.
    assert_eq!(&doc.raw_html[..], &html[..]);
}
