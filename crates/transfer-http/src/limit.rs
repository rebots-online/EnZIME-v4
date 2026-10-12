//! Stream wrapper that enforces §8.5's "never buffer unbounded" rule and the
//! caller-supplied byte ceilings. Wraps any `Stream<Item = Result<Bytes, _>>`
//! whose error type is already `TransferError`.

use bytes::Bytes;
use futures_core::stream::FusedStream;
use futures_core::Stream;
use ports::transfer::TransferError;
use std::pin::Pin;
use std::task::{Context, Poll};

/// A `Stream<Item = Result<Bytes, TransferError>>` that:
///
/// * emits `ResponseLimitExceeded` as soon as the cumulative byte count would
///   exceed `max_bytes`,
/// * emits `InvalidRangeResponse` at end-of-stream if the caller declared an
///   exact expected length and the stream did not deliver it,
/// * terminates (returns `None` forever) after the first error so consumers
///   never observe a "good" chunk after a "bad" one.
///
/// The wrapper is `Unpin` provided the underlying stream is `Unpin`, which
/// holds for `reqwest::Response::bytes_stream()` and for
/// `tokio_util::io::ReaderStream`.
pub(crate) struct LimitedStream<S> {
    inner: S,
    max_bytes: u64,
    expected_total: Option<u64>,
    received: u64,
    terminated: bool,
}

impl<S> LimitedStream<S> {
    pub(crate) fn new(inner: S, max_bytes: u64, expected_total: Option<u64>) -> Self {
        Self {
            inner,
            max_bytes,
            expected_total,
            received: 0,
            terminated: false,
        }
    }
}

impl<S> Stream for LimitedStream<S>
where
    S: Stream<Item = Result<Bytes, TransferError>> + Unpin,
{
    type Item = Result<Bytes, TransferError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.terminated {
            return Poll::Ready(None);
        }
        let this = self.as_mut().get_mut();
        let inner = Pin::new(&mut this.inner);
        match inner.poll_next(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => {
                this.terminated = true;
                if let Some(expected) = this.expected_total {
                    if this.received != expected {
                        return Poll::Ready(Some(Err(TransferError::InvalidRangeResponse)));
                    }
                }
                Poll::Ready(None)
            }
            Poll::Ready(Some(Err(error))) => {
                this.terminated = true;
                Poll::Ready(Some(Err(error)))
            }
            Poll::Ready(Some(Ok(bytes))) => {
                let new_total = match this.received.checked_add(bytes.len() as u64) {
                    Some(v) => v,
                    None => {
                        this.terminated = true;
                        return Poll::Ready(Some(Err(TransferError::ResponseLimitExceeded)));
                    }
                };
                if new_total > this.max_bytes {
                    this.terminated = true;
                    return Poll::Ready(Some(Err(TransferError::ResponseLimitExceeded)));
                }
                this.received = new_total;
                Poll::Ready(Some(Ok(bytes)))
            }
        }
    }
}

impl<S> FusedStream for LimitedStream<S>
where
    S: Stream<Item = Result<Bytes, TransferError>> + Unpin,
{
    fn is_terminated(&self) -> bool {
        self.terminated
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::StreamExt;

    #[tokio::test]
    async fn under_limit_passes_through() {
        let inner = futures_util::stream::iter([
            Ok(Bytes::from_static(b"abc")),
            Ok(Bytes::from_static(b"defg")),
        ]);
        let mut s = LimitedStream::new(inner, 100, None);
        let a = s.next().await.unwrap().unwrap();
        let b = s.next().await.unwrap().unwrap();
        assert_eq!(a.len() + b.len(), 7);
        assert!(s.next().await.is_none());
    }

    #[tokio::test]
    async fn over_limit_errors() {
        let inner = futures_util::stream::iter([
            Ok(Bytes::from_static(b"abc")),
            Ok(Bytes::from_static(b"defg")),
        ]);
        let mut s = LimitedStream::new(inner, 5, None);
        let first = s.next().await.unwrap().unwrap();
        assert_eq!(first.as_ref(), b"abc");
        let second = s.next().await.unwrap();
        assert!(matches!(second, Err(TransferError::ResponseLimitExceeded)));
        assert!(s.next().await.is_none());
    }

    #[tokio::test]
    async fn short_stream_with_expected_len_errors() {
        let inner = futures_util::stream::iter([Ok(Bytes::from_static(b"ab"))]);
        let mut s = LimitedStream::new(inner, 100, Some(10));
        let first = s.next().await.unwrap().unwrap();
        assert_eq!(first.as_ref(), b"ab");
        let end = s.next().await;
        assert!(matches!(
            end,
            Some(Err(TransferError::InvalidRangeResponse))
        ));
        assert!(s.next().await.is_none());
    }

    #[tokio::test]
    async fn exact_length_succeeds() {
        let inner = futures_util::stream::iter([Ok(Bytes::from_static(b"hello"))]);
        let mut s = LimitedStream::new(inner, 5, Some(5));
        let first = s.next().await.unwrap().unwrap();
        assert_eq!(first.as_ref(), b"hello");
        assert!(s.next().await.is_none());
    }
}
