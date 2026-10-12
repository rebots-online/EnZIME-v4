use super::*;
use async_trait::async_trait;
use bytes::Bytes;
use domain::identity::ArticleKey;
use ports::zim::*;
use sha2::{Digest, Sha256};
use storage_sqlite::annotations::*;

fn request(path: &str, method: &str, range: Option<&str>) -> Request<Vec<u8>> {
    let mut r = Request::builder().uri(path).method(method);
    if let Some(range) = range {
        r = r.header("Range", range);
    }
    r.body(vec![]).unwrap()
}
fn fixture() -> AppState {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    storage_sqlite::MigrationRunner::new(&conn)
        .apply_initial()
        .unwrap();
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../target/reader-resource-tests")
        .join(ulid::Ulid::new().to_string());
    let state = AppState::new(conn, root).unwrap();
    {
        let conn = state.conn.lock().unwrap();
        conn.execute("INSERT INTO packs (pack_id,corpus_id,display_name,description,language_bcp47,pack_kind,created_at_ms,updated_at_ms) VALUES ('pack','corpus','Test','','en','full_zim',1,1)",[]).unwrap();
        insert_sidecar_layer(
            &conn,
            &SidecarLayerRow {
                sidecar_layer_id: "local".into(),
                name: "Local".into(),
                owner_device_id: None,
                visibility: "private".into(),
                trust_state: "local".into(),
                source_artifact_id: None,
                created_at_ms: 1,
                updated_at_ms: 1,
                annotation_count: 0,
            },
        )
        .unwrap();
        insert_annotation(
            &conn,
            &AnnotationRow {
                annotation_id: "note".into(),
                annotation_type: "voice".into(),
                pack_id: "pack".into(),
                article_key: "A/Main".into(),
                notebook_id: None,
                sidecar_layer_id: "local".into(),
                current_revision: 1,
                resolution_state: "resolved".into(),
                pinned: false,
                deleted_at_ms: None,
                created_at_ms: 1,
                updated_at_ms: 1,
            },
        )
        .unwrap();
    }
    let data = b"OggS0123456789";
    let digest = ContentDigest::sha256_from_hex(hex::encode(Sha256::digest(data))).unwrap();
    state
        .annotations
        .object_store()
        .commit(object_store::CommitRequest {
            transaction_id: "fixture",
            audit_event_id_on_failure: "fixture",
            max_size: 128,
            expected_length: data.len() as u64,
            expected_digest: &digest,
            bytes: &data[..],
            verifier: &object_store::AcceptRawBytes,
        })
        .unwrap();
    insert_annotation_asset(
        &state.conn.lock().unwrap(),
        &AnnotationAssetRow {
            annotation_asset_id: "voice".into(),
            annotation_id: "note".into(),
            asset_kind: "voice_opus".into(),
            object_sha256: digest.hex().into(),
            mime_type: "audio/ogg;codecs=opus".into(),
            byte_length: data.len() as i64,
            duration_ms: Some(1200),
            created_at_ms: 1,
        },
    )
    .unwrap();
    state
}
#[test]
fn routes_decode_once_and_reject_traversal() {
    assert_eq!(
        parse_resource_path("/pack/wiki/I/caf%C3%A9/a%20b.png").unwrap(),
        ResourceRoute::Pack {
            pack_id: "wiki".into(),
            entry_key: "I/café/a b.png".into()
        }
    );
    assert!(parse_resource_path("/pack/wiki/-/style.css").is_ok());
    for path in [
        "/pack/wiki/I/../secret",
        "/pack/wiki/I/%2E%2E/secret",
        "/pack/wiki/I/x%2Fy",
        "/pack/wiki/I/%5C",
        "/pack/wiki/I/%00",
        "/pack/wiki/I/%GG",
        "/pack/wiki/I/%FF",
        "/annotation/a/",
        "/pack//I/a",
    ] {
        assert!(parse_resource_path(path).is_err(), "{path}");
    }
    assert_eq!(
        parse_resource_path("/pack/wiki/I/%252e%252e").unwrap(),
        ResourceRoute::Pack {
            pack_id: "wiki".into(),
            entry_key: "I/%2e%2e".into()
        }
    );
}
#[test]
fn ranges_and_mimes_are_explicit() {
    assert_eq!(byte_range(None, 10).unwrap(), (0, 10, false));
    assert_eq!(byte_range(Some("bytes=2-5"), 10).unwrap(), (2, 6, true));
    assert_eq!(byte_range(Some("bytes=-3"), 10).unwrap(), (7, 10, true));
    assert_eq!(byte_range(Some("bytes=7-"), 10).unwrap(), (7, 10, true));
    assert_eq!(byte_range(Some("bytes=-99"), 10).unwrap(), (0, 10, true));
    for h in [
        "bytes=-0",
        "bytes=10-",
        "bytes=3-2",
        "bytes=0-1,3-4",
        "items=1-2",
        "bytes=+1-3",
    ] {
        assert!(byte_range(Some(h), 10).is_err());
    }
    for mime in [
        "audio/ogg;codecs=opus",
        "video/webm",
        "image/svg+xml",
        "font/woff2",
        "text/css",
    ] {
        assert!(passive_mime(mime));
    }
    for mime in [
        "text/html",
        "application/javascript",
        "text/xml",
        "application/octet-stream",
    ] {
        assert!(!passive_mime(mime));
    }
}
#[test]
fn stylesheets_rewrite_root_urls_and_preserve_relative_urls() {
    let css = rewrite_stylesheet("@import '/-/base.css'; .a{background:url(../I/a.png)} .b{src:url('/I/font.woff2')} @media screen{.c{background:u\\72l(/I/a.png)}}","wiki").unwrap();
    assert!(css.contains("/pack/wiki/-/base.css"), "{css}");
    assert!(css.contains("../I/a.png"));
    assert!(css.contains("/pack/wiki/I/font.woff2"));
    assert!(css.contains("/pack/wiki/I/a.png"));
    for css in [
        "@import 'https://evil/style.css';",
        "a{background:url(//evil/x)}",
        "a{background:url(javascript:evil)}",
        "a{width:expression(1)}",
        r#"a{background:image-set("https://evil/x" 1x)}"#,
        "a{background:url('/../x')}",
    ] {
        assert!(rewrite_stylesheet(css, "wiki").is_err(), "{css}");
    }
}
#[tokio::test]
async fn durable_asset_handler_enforces_membership_deleted_heads_and_ranges() {
    let state = fixture();
    let metadata = list_assets(&state, "note").unwrap();
    assert_eq!(metadata.len(), 1);
    assert!(metadata[0].resource_url.ends_with("/annotation/note/voice"));
    assert_eq!(metadata[0].duration_ms, Some(1200));
    let full = handle(&state, request("/annotation/note/voice", "GET", None)).await;
    assert_eq!(full.status(), 200);
    assert_eq!(full.body(), b"OggS0123456789");
    assert_eq!(full.headers()["Content-Length"], "14");
    assert_eq!(full.headers()["X-Content-Type-Options"], "nosniff");
    let partial = handle(
        &state,
        request("/annotation/note/voice", "GET", Some("bytes=4-7")),
    )
    .await;
    assert_eq!(partial.status(), 206);
    assert_eq!(partial.body(), b"0123");
    assert_eq!(partial.headers()["Content-Range"], "bytes 4-7/14");
    let suffix = handle(
        &state,
        request("/annotation/note/voice", "GET", Some("bytes=-2")),
    )
    .await;
    assert_eq!(suffix.body(), b"89");
    let head = handle(
        &state,
        request("/annotation/note/voice", "HEAD", Some("bytes=4-7")),
    )
    .await;
    assert_eq!(head.status(), 206);
    assert!(head.body().is_empty());
    assert_eq!(head.headers()["Content-Length"], "4");
    let bad = handle(
        &state,
        request("/annotation/note/voice", "GET", Some("bytes=99-")),
    )
    .await;
    assert_eq!(bad.status(), 416);
    assert_eq!(bad.headers()["Content-Range"], "bytes */14");
    for path in ["/annotation/absent/voice", "/annotation/note/wrong"] {
        assert_eq!(
            handle(&state, request(path, "GET", None)).await.status(),
            404
        );
    }
    assert_eq!(
        handle(&state, request("/annotation/note/voice", "POST", None))
            .await
            .status(),
        405
    );
    soft_delete_annotation(&state.conn.lock().unwrap(), "note", 9).unwrap();
    assert!(list_assets(&state, "note").is_err());
    assert_eq!(
        handle(&state, request("/annotation/note/voice", "GET", None))
            .await
            .status(),
        404
    );
}
struct Archive {
    meta: ZimMetadata,
}
impl Archive {
    fn new() -> Self {
        Self {
            meta: ZimMetadata {
                uuid: "fixture".into(),
                title: "Fixture".into(),
                description: String::new(),
                language_bcp47: "en".into(),
                creator: String::new(),
                publisher: String::new(),
                date: String::new(),
                article_count: 0,
                media_count: 1,
                main_page: None,
                checksum: None,
            },
        }
    }
}
#[async_trait]
impl ZimArchive for Archive {
    fn metadata(&self) -> &ZimMetadata {
        &self.meta
    }
    async fn main_page(&self) -> Result<Option<ArticleKey>, ZimError> {
        Ok(None)
    }
    async fn entry(&self, key: &EntryKey) -> Result<ZimEntryDescriptor, ZimError> {
        if key.0 == "I/missing" {
            return Err(ZimError::EntryNotFound(key.0.clone()));
        }
        Ok(ZimEntryDescriptor {
            entry_key: key.clone(),
            namespace: 'I',
            path: key.0.clone(),
            title: String::new(),
            mime_type: "image/png".into(),
            kind: ZimEntryKind::Resource,
            redirect_target: None,
            byte_length: 8,
        })
    }
    async fn resource(&self, key: &EntryKey) -> Result<ResourceDocument, ZimError> {
        Ok(ResourceDocument {
            entry_key: key.clone(),
            mime_type: if key.0 == "-/style.css" {
                "text/css"
            } else if key.0 == "I/evil" {
                "text/html"
            } else {
                "image/png"
            }
            .into(),
            bytes: if key.0 == "-/style.css" {
                Bytes::from_static(b"a{background:url(/I/a.png)}")
            } else {
                Bytes::from_static(b"PNG01234")
            },
            content_digest: ContentDigest::sha256_from_hex("00".repeat(32)).unwrap(),
        })
    }
    async fn article(&self, _: &ArticleKey) -> Result<ArticleDocument, ZimError> {
        Err(ZimError::UnreadablePath)
    }
    async fn resolve_redirect(&self, key: &EntryKey, _: u8) -> Result<EntryKey, ZimError> {
        Ok(key.clone())
    }
    async fn search_titles(&self, _: TitleSearchQuery) -> Result<Vec<TitleSearchHit>, ZimError> {
        Ok(vec![])
    }
    async fn entries(&self) -> Result<ZimEntryStream, ZimError> {
        Ok(Box::pin(futures_util::stream::empty()))
    }
    async fn verify_structure(&self) -> Result<ZimVerificationReport, ZimError> {
        Err(ZimError::UnreadablePath)
    }
}
#[tokio::test]
async fn pack_handler_uses_registered_archive_and_blocks_active_resources() {
    let state = fixture();
    state
        .archives
        .lock()
        .unwrap()
        .insert("pack".into(), Arc::new(Archive::new()));
    let response = handle(
        &state,
        request("/pack/pack/I/picture", "GET", Some("bytes=3-")),
    )
    .await;
    assert_eq!(response.status(), 206);
    assert_eq!(response.body(), b"01234");
    assert_eq!(
        handle(&state, request("/pack/unknown/I/picture", "GET", None))
            .await
            .status(),
        404
    );
    assert_eq!(
        handle(&state, request("/pack/pack/I/missing", "GET", None))
            .await
            .status(),
        404
    );
    assert_eq!(
        handle(&state, request("/pack/pack/I/evil", "GET", None))
            .await
            .status(),
        415
    );
}

#[tokio::test]
async fn stylesheet_handler_rewrites_scoped_urls_and_asset_length_is_checked() {
    let state = fixture();
    state
        .archives
        .lock()
        .unwrap()
        .insert("pack".into(), Arc::new(Archive::new()));
    let css = handle(&state, request("/pack/pack/-/style.css", "GET", None)).await;
    assert_eq!(css.status(), 200);
    assert!(std::str::from_utf8(css.body())
        .unwrap()
        .contains("/pack/pack/I/a.png"));
    assert_eq!(
        css.headers()["Content-Length"]
            .to_str()
            .unwrap()
            .parse::<usize>()
            .unwrap(),
        css.body().len()
    );
    state
        .conn
        .lock()
        .unwrap()
        .execute(
            "UPDATE annotation_assets SET byte_length=99 WHERE annotation_asset_id='voice'",
            [],
        )
        .unwrap();
    let corrupt = handle(&state, request("/annotation/note/voice", "GET", None)).await;
    assert_eq!(corrupt.status(), 409);
    assert!(!std::str::from_utf8(corrupt.body())
        .unwrap()
        .contains("reader-resource-tests"));
}
