use async_trait::async_trait;
use enzime_reader_lib::state::AppState;
use ports::transfer::*;
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};
#[allow(dead_code)]
#[path = "../../../../../VENDOR/AnZIMmermanLib-src/AnZimmermanLIB/AnZimmermanLib/rust/tests/support/mod.rs"]
mod support;
pub struct FixtureTransport {
    pub base: String,
    pub http: transfer_http::HttpTransferSource,
    pub offline: AtomicBool,
    pub requests: AtomicU64,
    pub fail_range: AtomicBool,
    pub delay_range: AtomicBool,
}
impl FixtureTransport {
    fn uri(&self, uri: &str) -> String {
        let path = uri
            .split_once("fixture.test")
            .map(|(_, p)| p)
            .unwrap_or("/catalog");
        if uri.contains("library.kiwix.org") {
            format!("{}/catalog", self.base)
        } else if uri.starts_with(&self.base) {
            uri.into()
        } else {
            format!("{}{path}", self.base)
        }
    }
    fn check(&self) -> Result<(), TransferError> {
        self.requests.fetch_add(1, Ordering::SeqCst);
        if self.offline.load(Ordering::SeqCst) {
            Err(TransferError::InvalidRangeResponse)
        } else {
            Ok(())
        }
    }
}
#[async_trait]
impl TransferSource for FixtureTransport {
    fn source_kind(&self) -> &'static str {
        "http-fixture"
    }
    async fn probe(&self, r: TransferProbeRequest) -> Result<TransferProbe, TransferError> {
        self.check()?;
        self.http
            .probe(TransferProbeRequest {
                uri: self.uri(&r.uri),
                maximum_response_bytes: r.maximum_response_bytes,
            })
            .await
    }
    async fn open_range(
        &self,
        uri: &str,
        range: TransferRange,
        etag: Option<&str>,
    ) -> Result<TransferByteStream, TransferError> {
        self.check()?;
        if self.fail_range.swap(false, Ordering::SeqCst) {
            return Err(TransferError::InvalidRangeResponse);
        }
        self.http.open_range(&self.uri(uri), range, etag).await
    }
    async fn open_pinned_range(
        &self,
        uri: &str,
        range: TransferRange,
        identity: &TransferProbe,
    ) -> Result<TransferByteStream, TransferError> {
        self.check()?;
        if self.fail_range.swap(false, Ordering::SeqCst) {
            return Err(TransferError::InvalidRangeResponse);
        }
        while self.delay_range.load(Ordering::SeqCst) {
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
        self.http
            .open_pinned_range(&self.uri(uri), range, identity)
            .await
    }
    async fn open_full(&self, uri: &str, max: u64) -> Result<TransferByteStream, TransferError> {
        self.check()?;
        assert!(!uri.ends_with(".zim"), "whole archive transfer forbidden");
        self.http.open_full(&self.uri(uri), max).await
    }
}
pub fn fixture() -> Vec<u8> {
    let mut builder = support::ZimBuilder::default()
        .add_article(
            b'A',
            "Home",
            "Home",
            "text/html",
            b"<p>Offline canvas article</p><img src='../I/pixel.txt'>".to_vec(),
        )
        .add_article(b'I', "pixel.txt", "Pixel", "text/plain", b"pixel".to_vec());
    for index in 0..100 {
        builder = builder.add_article(
            b'A',
            format!("a{index:05}"),
            format!("Unvisited {index:05} {}", "t".repeat(1980)),
            "text/html",
            format!("<p>unvisited payload {index}</p>").into_bytes(),
        );
    }
    builder = builder
        .add_redirect(b'R', "Retained", "Retained alias", 92)
        .add_redirect(b'R', "Via", "Via alias", 102)
        .add_article(
            b'I',
            "unvisited.txt",
            "Unvisited resource",
            "text/plain",
            b"offline resource".to_vec(),
        )
        .add_article(
            b'Z',
            "Large",
            "Large",
            "application/octet-stream",
            vec![9; 20 * 1024 * 1024],
        );
    builder.main_page_index = Some(0);
    builder.uuid = [19; 16];
    let mut bytes = vec![];
    builder.write(&mut bytes);
    bytes
}
pub async fn serve(
    bytes: Vec<u8>,
    published_sha256: bool,
) -> (Arc<FixtureTransport>, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let bytes = Arc::new(bytes);
    let archive_length = bytes.len();
    let hash = if published_sha256 {
        use sha2::{Digest, Sha256};
        format!(
            "<hash type='sha-256'>{}</hash>",
            hex::encode(Sha256::digest(bytes.as_slice()))
        )
    } else {
        format!("<hash type='sha-1'>{}</hash>", "a".repeat(40))
    };
    let metadata=format!("<metalink xmlns='urn:ietf:params:xml:ns:metalink'><file name='fixture.zim'><size>{archive_length}</size>{hash}<url priority='1'>https://fixture.test/archive.zim</url></file></metalink>" );
    let opds="<feed xmlns='http://www.w3.org/2005/Atom' xmlns:dc='http://purl.org/dc/terms/' xmlns:opensearch='http://a9.com/-/spec/opensearch/1.1/'><opensearch:totalResults>1</opensearch:totalResults><entry><id>fixture-id</id><title>Fixture</title><summary>Offline article</summary><updated>2026-10-12</updated><dc:language>eng</dc:language><name>fixture</name><link rel='http://opds-spec.org/acquisition' type='application/x-zim' href='https://fixture.test/archive.meta4' length='999'/></entry></feed>".as_bytes().to_vec();
    let task = tokio::spawn(async move {
        loop {
            let Ok((mut socket, _)) = listener.accept().await else {
                break;
            };
            let bytes = bytes.clone();
            let opds = opds.clone();
            let metadata = metadata.clone();
            tokio::spawn(async move {
                let mut request = vec![];
                loop {
                    let mut buffer = [0; 4096];
                    let n = socket.read(&mut buffer).await.unwrap_or(0);
                    if n == 0 {
                        return;
                    }
                    request.extend_from_slice(&buffer[..n]);
                    if request.windows(4).any(|v| v == b"\r\n\r\n") {
                        break;
                    }
                    if request.len() > 16384 {
                        return;
                    }
                }
                let request = String::from_utf8_lossy(&request);
                let first = request.lines().next().unwrap_or("");
                let head = first.starts_with("HEAD");
                let path = first.split_whitespace().nth(1).unwrap_or("");
                let full = if path == "/catalog" {
                    opds.as_slice()
                } else if path.ends_with(".meta4") {
                    metadata.as_bytes()
                } else {
                    bytes.as_slice()
                };
                let range = request
                    .lines()
                    .find_map(|l| {
                        l.to_ascii_lowercase()
                            .strip_prefix("range: bytes=")
                            .map(str::to_owned)
                    })
                    .and_then(|v| {
                        let (a, b) = v.split_once('-')?;
                        Some((a.parse::<usize>().ok()?, b.parse::<usize>().ok()?))
                    });
                let (status, body, extra) = if let Some((a, b)) = range {
                    if b >= full.len() {
                        return;
                    }
                    (
                        "206 Partial Content",
                        &full[a..=b],
                        format!("Content-Range: bytes {a}-{b}/{}\r\n", full.len()),
                    )
                } else {
                    ("200 OK", full, String::new())
                };
                let headers=format!("HTTP/1.1 {status}\r\nContent-Length: {}\r\nAccept-Ranges: bytes\r\nETag: \"fixture-v1\"\r\n{extra}Connection: close\r\n\r\n",body.len());
                let _ = socket.write_all(headers.as_bytes()).await;
                if !head {
                    let _ = socket.write_all(body).await;
                }
            });
        }
    });
    (
        Arc::new(FixtureTransport {
            base,
            http: transfer_http::HttpTransferSource::new().unwrap(),
            offline: AtomicBool::new(false),
            requests: AtomicU64::new(0),
            fail_range: AtomicBool::new(false),
            delay_range: AtomicBool::new(false),
        }),
        task,
    )
}
pub fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../target/dyndon-install-tests")
        .join(ulid::Ulid::new().to_string())
}
pub fn make_state(root: &std::path::Path, source: Arc<FixtureTransport>) -> AppState {
    std::fs::create_dir_all(root.join("chosen")).unwrap();
    std::fs::create_dir_all(root.join("app")).unwrap();
    let conn = storage_sqlite::open_with_pragmas(&root.join("app/reader.db")).unwrap();
    storage_sqlite::MigrationRunner::new(&conn)
        .apply_initial()
        .unwrap();
    storage_sqlite::apply_dyndon_schema(&conn).unwrap();
    conn.execute("INSERT OR IGNORE INTO storage_roots(storage_root_id,display_name,kind,platform_locator,writable,created_at_ms,updated_at_ms) VALUES('chosen','Chosen','app_private',?1,1,1,1)",[root.join("chosen").to_str().unwrap()]).unwrap();
    let state = AppState::new_with_transfer(conn, root.join("app"), source).unwrap();
    state.annotations.ensure_local_layer().unwrap();
    state
}
