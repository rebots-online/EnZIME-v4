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
    TransferSource, TRANSFER_MAX_REDIRECTS, TRANSFER_PROBE_MAX_BYTES,
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
            .redirect(reqwest::redirect::Policy::custom(move |attempt| {
                if attempt.previous().len() >= max_redirects {
                    attempt.error("redirect limit")
                } else if !safe_url(attempt.url()) {
                    attempt.error("insecure redirect")
                } else {
                    attempt.follow()
                }
            }))
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

impl HttpTransferSource {
    async fn range_response(
        &self,
        uri: &str,
        range: TransferRange,
        expected_etag: Option<&str>,
        identity: Option<&TransferProbe>,
    ) -> Result<TransferByteStream, TransferError> {
        check_url(uri)?;
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
        let last_modified = identity.and_then(|i| i.last_modified.clone());
        let if_range_value = expected_etag.map(|s| s.to_string());
        let response = retry_transient(|| {
            let client = client.clone();
            let uri = uri_owned.clone();
            let range_value = range_header.clone();
            let if_range = if_range_value.clone();
            let last_modified = last_modified.clone();
            async move {
                let mut request = client
                    .get(&uri)
                    .header(reqwest::header::ACCEPT_ENCODING, "identity")
                    .header(reqwest::header::RANGE, range_value);
                if let Some(etag) = if_range.as_deref() {
                    request = request.header(reqwest::header::IF_RANGE, etag);
                }
                if if_range.is_none() {
                    if let Some(value) = last_modified {
                        request = request.header(reqwest::header::IF_UNMODIFIED_SINCE, value);
                    }
                }
                request.send().await.map_err(map_send_error)
            }
        })
        .await?;

        if response.status() == reqwest::StatusCode::PRECONDITION_FAILED {
            return Err(TransferError::SourceChanged);
        }
        classify_status(response.status())?;
        check_response(&response)?;

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
        let parsed =
            parse::content_range(content_range).ok_or(TransferError::InvalidRangeResponse)?;
        if parsed.start != range.start_byte || parsed.end_inclusive != last_byte {
            return Err(TransferError::InvalidRangeResponse);
        }

        if let Some(expected) = expected_etag {
            if header_string(&response, reqwest::header::ETAG).as_deref() != Some(expected) {
                return Err(TransferError::SourceChanged);
            }
        }
        if let Some(identity) = identity {
            if parsed.total != Some(identity.size_bytes) {
                return Err(TransferError::SourceChanged);
            }
            if expected_etag.is_none()
                && header_string(&response, reqwest::header::LAST_MODIFIED)
                    != identity.last_modified
            {
                return Err(TransferError::SourceChanged);
            }
        }
        let stream = response.bytes_stream().map_err(map_stream_error);
        let limited = LimitedStream::new(stream, expected_len, Some(expected_len));
        Ok(Box::pin(limited))
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

    async fn probe(&self, request: TransferProbeRequest) -> Result<TransferProbe, TransferError> {
        check_url(&request.uri)?;
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
                    .header(reqwest::header::ACCEPT_ENCODING, "identity")
                    .header(reqwest::header::RANGE, range_value)
                    .send()
                    .await
                    .map_err(map_send_error)
            }
        })
        .await?;

        classify_status(response.status())?;

        check_response(&response)?;
        let uri = response.url().to_string();
        let (accepts_ranges, size_bytes) = interpret_probe_status(&response)?;
        if accepts_ranges {
            if response.status() != reqwest::StatusCode::PARTIAL_CONTENT || size_bytes == 0 {
                return Err(TransferError::InvalidRangeResponse);
            }
            let value = header_string(&response, reqwest::header::CONTENT_RANGE)
                .and_then(|v| parse::content_range(&v))
                .ok_or(TransferError::InvalidRangeResponse)?;
            if value.start != 0 || value.end_inclusive != last_byte.min(size_bytes - 1) {
                return Err(TransferError::InvalidRangeResponse);
            }
        }

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
        self.range_response(uri, range, expected_etag, None).await
    }
    async fn open_pinned_range(
        &self,
        uri: &str,
        range: TransferRange,
        identity: &TransferProbe,
    ) -> Result<TransferByteStream, TransferError> {
        let strong = identity
            .etag
            .as_deref()
            .filter(|e| e.len() >= 2 && e.starts_with('"') && e.ends_with('"'));
        if strong.is_none() && identity.last_modified.is_none() {
            return Err(TransferError::Transport(
                "source_identity_unavailable".into(),
            ));
        }
        self.range_response(uri, range, strong, Some(identity))
            .await
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
fn interpret_probe_status(response: &reqwest::Response) -> Result<(bool, u64), TransferError> {
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
    if error.is_redirect() {
        TransferError::PermissionDenied
    } else if error.is_connect() || error.is_request() || error.is_timeout() {
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
    use ports::transfer::{TRANSFER_RETRY_BASE_MS, TRANSFER_RETRY_COUNT, TRANSFER_RETRY_MAX_MS};

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

fn safe_url(url: &reqwest::Url) -> bool {
    url.username().is_empty()
        && url.password().is_none()
        && (url.scheme() == "https"
            || (url.scheme() == "http" && url.host_str() == Some("127.0.0.1")))
}
fn check_url(uri: &str) -> Result<(), TransferError> {
    let url = reqwest::Url::parse(uri).map_err(|_| TransferError::PermissionDenied)?;
    if safe_url(&url) {
        Ok(())
    } else {
        Err(TransferError::PermissionDenied)
    }
}
fn check_response(response: &reqwest::Response) -> Result<(), TransferError> {
    if !safe_url(response.url()) {
        return Err(TransferError::PermissionDenied);
    }
    if header_string(response, reqwest::header::CONTENT_ENCODING)
        .is_some_and(|v| !v.eq_ignore_ascii_case("identity"))
    {
        return Err(TransferError::InvalidRangeResponse);
    }
    Ok(())
}
