//! Stream wrapper that enforces the caller-supplied byte ceilings and the
//! §8.5 "never buffer unbounded" rule for the local file adapter.
//!
//! This is a private duplicate of the same primitive used by `transfer-http`.
//! Both adapters need byte-limit enforcement on top of streams whose error
//! type is already `TransferError`; sharing a single Ring 2 crate would force
//! a Ring 2 → Ring 2 dependency that A-02 does not require, and inlining the
//! primitive keeps the file adapter self-contained.

use bytes::Bytes;
use futures_core::stream::FusedStream;
use futures_core::Stream;
use ports::transfer::TransferError;
use std::pin::Pin;
use std::task::{Context, Poll};

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

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
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
                        return Poll::Ready(Some(Err(
                            TransferError::InvalidRangeResponse,
                        )));
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
