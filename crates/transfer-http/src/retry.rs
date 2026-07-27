//! Exponential-backoff retry for transient transport-level failures.
//!
//! Implements §8.5 `transfer.retry_count`, `transfer.retry_base_ms`,
//! `transfer.retry_max_ms`. Applied to connection setup only — once a stream
//! has been returned to the caller, transport errors are surfaced (the caller
//! can resume via a fresh `open_range` with a shifted range).

use ports::transfer::{TransferError, TRANSFER_RETRY_BASE_MS, TRANSFER_RETRY_COUNT, TRANSFER_RETRY_MAX_MS};
use std::time::Duration;

fn is_retryable(error: &TransferError) -> bool {
    matches!(
        error,
        TransferError::Offline | TransferError::Transport(_)
    )
}

/// Compute the backoff delay for `attempt` (1-based). Returns the smaller of
/// `base * 2^(attempt-1)` and the configured cap.
fn backoff_for(attempt: u32) -> Duration {
    let exp = attempt.saturating_sub(1);
    let raw = TRANSFER_RETRY_BASE_MS.saturating_mul(2u64.saturating_pow(exp));
    let capped = std::cmp::min(raw, TRANSFER_RETRY_MAX_MS);
    Duration::from_millis(capped)
}

/// Run `op` up to `TRANSFER_RETRY_COUNT` times, retrying only on `Offline`
/// and `Transport` errors with exponential backoff. Any other error is
/// surfaced immediately.
pub(crate) async fn retry_transient<T, F, Fut>(mut op: F) -> Result<T, TransferError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, TransferError>>,
{
    let max_attempts = TRANSFER_RETRY_COUNT.max(1);
    let mut attempt: u32 = 0;
    loop {
        attempt = attempt.saturating_add(1);
        match op().await {
            Ok(value) => return Ok(value),
            Err(error) => {
                if attempt >= max_attempts || !is_retryable(&error) {
                    return Err(error);
                }
                tokio::time::sleep(backoff_for(attempt)).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_grows_then_caps() {
        let first = backoff_for(1);
        let second = backoff_for(2);
        let third = backoff_for(3);
        assert!(second > first);
        assert!(third > second);
        // Eventually we hit the cap.
        let far = backoff_for(60);
        assert!(far <= Duration::from_millis(TRANSFER_RETRY_MAX_MS));
    }

    #[test]
    fn is_retryable_classification() {
        assert!(is_retryable(&TransferError::Offline));
        assert!(is_retryable(&TransferError::Transport("net".into())));
        assert!(!is_retryable(&TransferError::NotFound));
        assert!(!is_retryable(&TransferError::PermissionDenied));
        assert!(!is_retryable(&TransferError::SourceChanged));
        assert!(!is_retryable(&TransferError::RangeUnsupported));
        assert!(!is_retryable(&TransferError::InvalidRangeResponse));
        assert!(!is_retryable(&TransferError::ResponseLimitExceeded));
    }

    #[tokio::test]
    async fn succeeds_first_try() {
        let count = std::sync::atomic::AtomicU32::new(0);
        let result: Result<u32, TransferError> = retry_transient(|| async {
            count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(7u32)
        })
        .await;
        assert_eq!(result.unwrap(), 7);
        assert_eq!(
            count.load(std::sync::atomic::Ordering::SeqCst),
            1
        );
    }

    #[tokio::test]
    async fn non_retryable_short_circuits() {
        let count = std::sync::atomic::AtomicU32::new(0);
        let result: Result<u32, TransferError> = retry_transient(|| async {
            count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Err(TransferError::NotFound)
        })
        .await;
        assert!(matches!(result, Err(TransferError::NotFound)));
        assert_eq!(count.load(std::sync::atomic::Ordering::SeqCst), 1);
    }
}
