//! `transfer.http.v1` — §8.2 `TransferSource` over HTTP/HTTPS.
//!
//! Implements the trait at `ports::transfer::TransferSource` against `reqwest`
//! 0.13 with `rustls`. Honours §8.5 transfer limits:
//!
//! * `transfer.max_redirects` — enforced by the client's redirect policy.
//! * `transfer.probe_max_bytes` — bounds the ranged probe body the source will
//!   request when discovering size / range support.
//! * `transfer.retry_count`, `transfer.retry_base_ms`, `transfer.retry_max_ms`
//!   — exponential backoff over connection setup only; mid-stream transport
//!   errors are surfaced to the caller (which can resume via `open_range`).
//!
//! Critical §8.5 rule: an HTTP `200` to a range request is **never** accepted
//! as a ranged response. If the caller passed `expected_etag`, the source also
//! fails the request when the server's `ETag` differs. The byte stream enforces
//! `transfer.chunk_bytes`-aligned accounting and rejects any response that
//! exceeds the declared ceiling — there is no unbounded buffering anywhere.

use async_trait::async_trait;
use futures_util::TryStreamExt;
use ports::transfer::{
    TransferByteStream, TransferError, TransferProbe, TransferProbeRequest, TransferRange,
    TransferSource, TRANSFER_MAX_REDIRECTS, TRANSFER_PROBE_MAX_BYTES, TRANSFER_RETRY_BASE_MS,
    TRANSFER_RETRY_COUNT, TRANSFER_RETRY_MAX_MS,
};

mod limit;
mod parse;
mod retry;

use limit::LimitedStream;
use retry::retry_transient;

/// Release identifier for this adapter — see the `TransferSource` table in §8.2.
const SOURCE_KIND: &str = "transfer.http.v1";

/// HTTP-backed `TransferSource`. One client can be shared across many requests;
/// `reqwest::Client` keeps an internal connection pool.
pub struct HttpTransferSource {
    client: reqwest::Client,
}

impl HttpTransferSource {
    /// Build a source using §8.5's `transfer.max_redirects` cap. TLS is
    /// provided by `rustls`; the client never reads or stores credentials.
    pub fn new() -> Result<Self, TransferError> {
        let max_redirects = usize::try_from(TRANSFER_MAX_REDIRECTS)
            .map_err(|_| TransferError::Transport("redirect count overflow".into()))?;
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::limited(max_redirects))
            .build()
            .map_err(|e| TransferError::Transport(format!("client build: {e}")))?;
        Ok(Self { client })
    }

    /// Wrap an externally-constructed `reqwest::Client`. The caller is
    /// responsible for redirect policy and TLS configuration.
    pub fn with_client(client: reqwest::Client) -> Self {
        Self { client }
    }
}

impl Default for HttpTransferSource {
    fn default() -> Self {
        Self::new().expect("a reqwest client with rustls always builds")
    }
}

#[async_trait]
impl TransferSource for HttpTransferSource {
    fn source_kind(&self) -> &'static str {
        SOURCE_KIND
    }

    async fn probe(
        &self,
        request: TransferProbeRequest,
    ) -> Result<TransferProbe, TransferError> {
        if request.maximum_response_bytes == 0 {
            return Err(TransferError::ResponseLimitExceeded);
        }
        // §8.5 `transfer.probe_max_bytes` caps what we are willing to receive
        // in order to discover the metadata.
        let probe_cap = request.maximum_response_bytes.min(TRANSFER_PROBE_MAX_BYTES);
        let last_byte = probe_cap
            .checked_sub(1)
            .ok_or(TransferError::ResponseLimitExceeded)?;
        let range_value = format!("bytes=0-{last_byte}");

        let client = self.client.clone();
        let uri = request.uri.clone();
        let response = retry_transient(|| {
            let client = client.clone();
            let uri = uri.clone();
            let range_value = range_value.clone();
            async move {
                client
                    .get(&uri)
                    .header(reqwest::header::RANGE, range_value)
                    .send()
                    .await
                    .map_err(map_send_error)
            }
        })
        .await?;

        classify_status(response.status())?;

        let uri = request.uri.clone();
        let (accepts_ranges, size_bytes) = interpret_probe_status(&response)?;

        let etag = header_string(&response, reqwest::header::ETAG);
        let last_modified = header_string(&response, reqwest::header::LAST_MODIFIED);
        let content_type = header_string(&response, reqwest::header::CONTENT_TYPE);

        Ok(TransferProbe {
            uri,
            size_bytes,
            accepts_ranges,
            etag,
            last_modified,
            content_type,
        })
    }

    async fn open_range(
        &self,
        uri: &str,
        range: TransferRange,
        expected_etag: Option<&str>,
    ) -> Result<TransferByteStream, TransferError> {
        if range.end_byte_exclusive <= range.start_byte {
            return Err(TransferError::InvalidRangeResponse);
        }
        let last_byte = range
            .end_byte_exclusive
            .checked_sub(1)
            .ok_or(TransferError::InvalidRangeResponse)?;
        let expected_len = range
            .end_byte_exclusive
            .checked_sub(range.start_byte)
            .ok_or(TransferError::InvalidRangeResponse)?;
        let range_header = format!("bytes={}-{last_byte}", range.start_byte);

        let client = self.client.clone();
        let uri_owned = uri.to_string();
        let if_range_value = expected_etag.map(|s| s.to_string());
        let response = retry_transient(|| {
            let client = client.clone();
            let uri = uri_owned.clone();
            let range_value = range_header.clone();
            let if_range = if_range_value.clone();
            async move {
                let mut request = client
                    .get(&uri)
                    .header(reqwest::header::RANGE, range_value);
                if let Some(etag) = if_range.as_deref() {
                    request = request.header(reqwest::header::IF_RANGE, etag);
                }
                request.send().await.map_err(map_send_error)
            }
        })
        .await?;

        classify_status(response.status())?;

        // §8.5: HTTP `200` to a range request is not accepted as a ranged
        // response. If `expected_etag` was supplied the server's behaviour
        // means "the source changed"; otherwise it means "the server ignored
        // the Range header", which is an invalid ranged response.
        if response.status() == reqwest::StatusCode::OK {
            return Err(if expected_etag.is_some() {
                TransferError::SourceChanged
            } else {
                TransferError::InvalidRangeResponse
            });
        }
        if response.status() != reqwest::StatusCode::PARTIAL_CONTENT {
            return Err(TransferError::InvalidRangeResponse);
        }

        let content_range = response
            .headers()
            .get(reqwest::header::CONTENT_RANGE)
            .and_then(|v| v.to_str().ok())
            .ok_or(TransferError::InvalidRangeResponse)?;
        let parsed = parse::content_range(content_range)
            .ok_or(TransferError::InvalidRangeResponse)?;
        if parsed.start != range.start_byte || parsed.end_inclusive != last_byte {
            return Err(TransferError::InvalidRangeResponse);
        }

        if let Some(expected) = expected_etag {
            if let Some(actual) =
                header_string(&response, reqwest::header::ETAG).as_deref()
            {
                if actual != expected {
                    return Err(TransferError::SourceChanged);
                }
            }
        }

        let stream = response.bytes_stream().map_err(map_stream_error);
        let limited = LimitedStream::new(stream, expected_len, Some(expected_len));
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

        let client = self.client.clone();
        let uri_owned = uri.to_string();
        let response = retry_transient(|| {
            let client = client.clone();
            let uri = uri_owned.clone();
            async move { client.get(&uri).send().await.map_err(map_send_error) }
        })
        .await?;

        classify_status(response.status())?;

        let stream = response.bytes_stream().map_err(map_stream_error);
        let limited = LimitedStream::new(stream, maximum_response_bytes, None);
        Ok(Box::pin(limited))
    }
}

fn header_string(
    response: &reqwest::Response,
    name: reqwest::header::HeaderName,
) -> Option<String> {
    response
        .headers()
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_owned())
}

/// Translate the response's status into a `TransferError` for any non-success
/// case. `OK` and `PARTIAL_CONTENT` are accepted here — the caller further
/// refines based on whether a range was requested.
fn classify_status(status: reqwest::StatusCode) -> Result<(), TransferError> {
    if status.is_success() {
        return Ok(());
    }
    Err(match status.as_u16() {
        404 | 410 => TransferError::NotFound,
        401 | 403 | 407 => TransferError::PermissionDenied,
        416 => TransferError::RangeUnsupported,
        _ if status.is_server_error() || status == reqwest::StatusCode::TOO_MANY_REQUESTS => {
            TransferError::Transport(format!("server status {status}"))
        }
        _ => TransferError::Transport(format!("unexpected status {status}")),
    })
}

/// Interpret the probe response, returning `(accepts_ranges, size_bytes)`.
fn interpret_probe_status(
    response: &reqwest::Response,
) -> Result<(bool, u64), TransferError> {
    let status = response.status();
    let headers = response.headers();

    if status == reqwest::StatusCode::PARTIAL_CONTENT {
        let total = headers
            .get(reqwest::header::CONTENT_RANGE)
            .and_then(|v| v.to_str().ok())
            .and_then(parse::content_range_total)
            .ok_or(TransferError::InvalidRangeResponse)?;
        return Ok((true, total));
    }

    if status == reqwest::StatusCode::OK {
        let size = headers
            .get(reqwest::header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);
        let accepts = headers
            .get(reqwest::header::ACCEPT_RANGES)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.eq_ignore_ascii_case("bytes"))
            .unwrap_or(false);
        return Ok((accepts, size));
    }

    Err(TransferError::Transport(format!(
        "unexpected probe status {status}"
    )))
}

/// Map a `reqwest::Error` produced by `Client::send` (a transport-level error
/// — `reqwest` returns successful responses even for HTTP error statuses).
fn map_send_error(error: reqwest::Error) -> TransferError {
    if error.is_connect() || error.is_request() || error.is_timeout() {
        TransferError::Offline
    } else if error.is_timeout() {
        TransferError::Transport(format!("timeout: {error}"))
    } else if error.is_redirect() {
        TransferError::Transport(format!("redirect: {error}"))
    } else {
        TransferError::Transport(error.to_string())
    }
}

/// Map a `reqwest::Error` produced by the streaming body. Once we have started
/// streaming the connection has been established; transport errors here are
/// surfaced without retry (the caller may resume via a fresh `open_range`).
fn map_stream_error(error: reqwest::Error) -> TransferError {
    if error.is_timeout() {
        TransferError::Transport(format!("stream timeout: {error}"))
    } else {
        TransferError::Transport(format!("stream: {error}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_status_maps_common_codes() {
        assert!(classify_status(reqwest::StatusCode::OK).is_ok());
        assert!(classify_status(reqwest::StatusCode::PARTIAL_CONTENT).is_ok());
        assert!(matches!(
            classify_status(reqwest::StatusCode::NOT_FOUND),
            Err(TransferError::NotFound)
        ));
        assert!(matches!(
            classify_status(reqwest::StatusCode::FORBIDDEN),
            Err(TransferError::PermissionDenied)
        ));
        assert!(matches!(
            classify_status(reqwest::StatusCode::RANGE_NOT_SATISFIABLE),
            Err(TransferError::RangeUnsupported)
        ));
    }

    #[test]
    fn retry_constants_are_sane() {
        use std::time::Duration;
        assert!(TRANSFER_RETRY_COUNT >= 1);
        assert!(TRANSFER_RETRY_BASE_MS >= 1);
        assert!(TRANSFER_RETRY_MAX_MS >= TRANSFER_RETRY_BASE_MS);
        assert!(TRANSFER_PROBE_MAX_BYTES >= 1);
        let _ = Duration::from_millis(TRANSFER_RETRY_MAX_MS);
    }
}
