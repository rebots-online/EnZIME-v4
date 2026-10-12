//! `transfer.file.v1` — §8.2 `TransferSource` over local files.
//!
//! Implements `ports::transfer::TransferSource` against the local filesystem
//! via `tokio::fs`. A local file always honours range requests (the OS seek +
//! read primitives are exact), so `accepts_ranges` is `true` for any file that
//! exists. `expected_etag` is accepted but ignored: there is no server-side
//! identity for a local file, and the §8.4 durable state machine re-verifies
//! the SHA-256 from the staged bytes before activation regardless.
//!
//! URIs accepted:
//!
//! * `file:///abs/path` and `file://localhost/abs/path` (RFC 8089)
//! * `/abs/path` (bare absolute path — handy for tests and platforms where
//!   the URI layer would just add overhead)
//!
//! Byte limits: `open_full` honours `maximum_response_bytes` strictly;
//! `open_range` honours the requested `TransferRange`. The underlying
//! `tokio::fs::File::take(n)` enforces the cap at the read site, and a
//! `LimitedStream` wrapper enforces it again at the stream site so a file that
//! is mutated mid-read cannot push the caller past its declared ceiling.

use async_trait::async_trait;
use futures_util::StreamExt;
use ports::transfer::{
    TransferByteStream, TransferError, TransferProbe, TransferProbeRequest, TransferRange,
    TransferSource, TRANSFER_CHUNK_BYTES,
};
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncSeekExt, SeekFrom};
use tokio_util::io::ReaderStream;

mod limit;

use limit::LimitedStream;

/// Release identifier for this adapter — see the `TransferSource` table in §8.2.
const SOURCE_KIND: &str = "transfer.file.v1";

/// Local-filesystem `TransferSource`. Stateless beyond the URI resolution at
/// each call; safe to share across tasks.
#[derive(Clone, Copy, Debug, Default)]
pub struct LocalFileTransferSource;

impl LocalFileTransferSource {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl TransferSource for LocalFileTransferSource {
    fn source_kind(&self) -> &'static str {
        SOURCE_KIND
    }

    async fn probe(
        &self,
        request: TransferProbeRequest,
    ) -> Result<TransferProbe, TransferError> {
        let path = resolve_uri(&request.uri)?;
        let metadata = match tokio::fs::metadata(&path).await {
            Ok(m) => m,
            Err(error) => return Err(map_io_error(error)),
        };
        if !metadata.is_file() {
            return Err(TransferError::NotFound);
        }
        Ok(TransferProbe {
            uri: request.uri,
            size_bytes: metadata.len(),
            accepts_ranges: true,
            etag: None,
            last_modified: None,
            content_type: None,
        })
    }

    async fn open_range(
        &self,
        uri: &str,
        range: TransferRange,
        _expected_etag: Option<&str>,
    ) -> Result<TransferByteStream, TransferError> {
        if range.end_byte_exclusive <= range.start_byte {
            return Err(TransferError::InvalidRangeResponse);
        }
        let expected_len = range
            .end_byte_exclusive
            .checked_sub(range.start_byte)
            .ok_or(TransferError::InvalidRangeResponse)?;
        let path = resolve_uri(uri)?;
        let mut file = tokio::fs::File::open(&path).await.map_err(map_io_error)?;
        if range.start_byte > 0 {
            file.seek(SeekFrom::Start(range.start_byte))
                .await
                .map_err(map_io_error)?;
        }
        let chunk_capacity = chunk_capacity();
        let reader = file.take(expected_len);
        let rs = ReaderStream::with_capacity(reader, chunk_capacity);
        let mapped = rs.map(|item| match item {
            Ok(bytes) => Ok(bytes),
            Err(error) => Err(map_io_error(error)),
        });
        let limited = LimitedStream::new(mapped, expected_len, Some(expected_len));
        Ok(Box::pin(limited))
    }

    async fn open_full(
        &self,
        uri: &str,
        maximum_response_bytes: u64,
    ) -> Result<TransferByteStream, TransferError> {
        if maximum_response_bytes == 0 {
            return Err(TransferError::ResponseLimitExceeded);
        }
        let path = resolve_uri(uri)?;
        let file = tokio::fs::File::open(&path).await.map_err(map_io_error)?;
        let chunk_capacity = chunk_capacity();
        let reader = file.take(maximum_response_bytes);
        let rs = ReaderStream::with_capacity(reader, chunk_capacity);
        let mapped = rs.map(|item| match item {
            Ok(bytes) => Ok(bytes),
            Err(error) => Err(map_io_error(error)),
        });
        let limited = LimitedStream::new(mapped, maximum_response_bytes, None);
        Ok(Box::pin(limited))
    }
}

/// Translate a `file://` URI (or bare absolute path) into a `PathBuf`. Any
/// other scheme is a transport-level error — the file adapter only owns the
/// `file` scheme and a bare path; HTTP, HTTPS, etc. belong to `transfer-http`.
fn resolve_uri(uri: &str) -> Result<PathBuf, TransferError> {
    let path_str: String = if let Some(rest) = uri.strip_prefix("file://") {
        if let Some(after_host) = rest.strip_prefix("localhost/") {
            format!("/{after_host}")
        } else if rest.starts_with('/') {
            rest.to_string()
        } else {
            return Err(TransferError::Transport(format!(
                "unsupported file uri (non-local authority): {uri}"
            )));
        }
    } else if uri.starts_with('/') {
        uri.to_string()
    } else {
        return Err(TransferError::Transport(format!(
            "unsupported uri scheme (expected file:// or absolute path): {uri}"
        )));
    };
    Ok(PathBuf::from(path_str))
}

/// Map a `std::io::Error` from `tokio::fs` to a `TransferError`. We are
/// deliberately conservative: any unexpected I/O kind becomes a `Transport`
/// error rather than a guess.
fn map_io_error(error: std::io::Error) -> TransferError {
    match error.kind() {
        std::io::ErrorKind::NotFound => TransferError::NotFound,
        std::io::ErrorKind::PermissionDenied => TransferError::PermissionDenied,
        _ => TransferError::Transport(format!("io: {error}")),
    }
}

/// Return the read-buffer capacity for `ReaderStream`, capped by the platform
/// `usize` and §8.5 `transfer.chunk_bytes`.
fn chunk_capacity() -> usize {
    usize::try_from(TRANSFER_CHUNK_BYTES).unwrap_or(usize::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_file_uri_with_host() {
        let p = resolve_uri("file://localhost/etc/hostname").unwrap();
        assert_eq!(p, PathBuf::from("/etc/hostname"));
    }

    #[test]
    fn resolve_file_uri_no_authority() {
        let p = resolve_uri("file:///etc/hostname").unwrap();
        assert_eq!(p, PathBuf::from("/etc/hostname"));
    }

    #[test]
    fn resolve_bare_absolute_path() {
        let p = resolve_uri("/etc/hostname").unwrap();
        assert_eq!(p, PathBuf::from("/etc/hostname"));
    }

    #[test]
    fn resolve_rejects_non_local_authority() {
        let err = resolve_uri("file://example.com/path").unwrap_err();
        assert!(matches!(err, TransferError::Transport(_)));
    }

    #[test]
    fn resolve_rejects_other_schemes() {
        let err = resolve_uri("https://example.com/path").unwrap_err();
        assert!(matches!(err, TransferError::Transport(_)));
        let err = resolve_uri("relative/path").unwrap_err();
        assert!(matches!(err, TransferError::Transport(_)));
    }

    #[test]
    fn map_io_error_classifies_common_kinds() {
        assert!(matches!(
            map_io_error(std::io::Error::from(std::io::ErrorKind::NotFound)),
            TransferError::NotFound
        ));
        assert!(matches!(
            map_io_error(std::io::Error::from(std::io::ErrorKind::PermissionDenied)),
            TransferError::PermissionDenied
        ));
        assert!(matches!(
            map_io_error(std::io::Error::from(std::io::ErrorKind::UnexpectedEof)),
            TransferError::Transport(_)
        ));
    }

    /// Probe + ranged read of an existing local file must round-trip the
    /// declared byte range exactly. This is the one behaviour that genuinely
    /// needs a fixture: it asserts the wiring between `resolve_uri`,
    /// `File::seek`, `ReaderStream` and `LimitedStream`.
    #[tokio::test]
    async fn probe_and_read_range_from_real_file() {
        // `CARGO_TARGET_TMPDIR` is cargo's per-test scratch directory under
        // `target/`; using it keeps the test from touching `/tmp` or any
        // mutable directory outside the workspace build tree.
        let base = std::env::var("CARGO_TARGET_TMPDIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."));
        let dir = base.join("transfer-file-test-fixture");
        tokio::fs::create_dir_all(&dir)
            .await
            .expect("create test dir");
        // The source adapter (correctly) rejects relative paths, and
        // CARGO_TARGET_TMPDIR is unset for unit tests — absolutize.
        let dir = tokio::fs::canonicalize(&dir)
            .await
            .expect("absolute test dir");
        let path = dir.join("fixture.bin");
        let payload: Vec<u8> = (0u32..4096).flat_map(|n| n.to_le_bytes().to_vec()).collect();
        tokio::fs::write(&path, &payload)
            .await
            .expect("write fixture");
        let uri = path.to_str().expect("utf8 path").to_string();

        let source = LocalFileTransferSource::new();
        let probe = source
            .probe(TransferProbeRequest {
                uri: uri.clone(),
                maximum_response_bytes: 16,
            })
            .await
            .expect("probe");
        assert_eq!(probe.size_bytes, payload.len() as u64);
        assert!(probe.accepts_ranges);

        let stream = source
            .open_range(
                &uri,
                TransferRange {
                    start_byte: 8,
                    end_byte_exclusive: 24,
                },
                None,
            )
            .await
            .expect("open range");
        let collected = collect_bytes(stream).await;
        assert_eq!(collected.len(), 16);
        assert_eq!(&collected[..], &payload[8..24]);

        // Reading beyond the end of the file is a clean error, never a silent
        // short read.
        let over_end = source
            .open_range(
                &uri,
                TransferRange {
                    start_byte: payload.len() as u64 - 4,
                    end_byte_exclusive: payload.len() as u64 + 8,
                },
                None,
            )
            .await;
        match over_end {
            Err(TransferError::InvalidRangeResponse) => {}
            Err(TransferError::RangeUnsupported) => {}
            Err(TransferError::Transport(_)) => {}
            Ok(stream) => {
                let result = collect_result(stream).await;
                assert!(
                    matches!(result, Err(TransferError::InvalidRangeResponse)),
                    "expected InvalidRangeResponse, got {:?}",
                    result
                );
            }
            Err(other) => panic!("unexpected error: {other:?}"),
        }

        let _ = tokio::fs::remove_file(&path).await;
        let _ = tokio::fs::remove_dir_all(&dir).await;
    }

    async fn collect_bytes(stream: TransferByteStream) -> Vec<u8> {
        match collect_result(stream).await {
            Ok(v) => v,
            Err(e) => panic!("stream errored: {e:?}"),
        }
    }

    async fn collect_result(
        mut stream: TransferByteStream,
    ) -> Result<Vec<u8>, TransferError> {
        let mut out = Vec::new();
        while let Some(item) = stream.next().await {
            match item {
                Ok(b) => out.extend_from_slice(&b),
                Err(e) => return Err(e),
            }
        }
        Ok(out)
    }
}
