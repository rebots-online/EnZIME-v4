//! Wall-clock milliseconds since the UNIX epoch.
//!
//! Used by `BackendHealth::checked_at_ms`. Kept dependency-free by reading
//! `SystemTime::now()` once and converting to milliseconds.

use std::time::{SystemTime, UNIX_EPOCH};

/// Return the current wall-clock time as milliseconds since the UNIX epoch.
/// Returns 0 if the system clock is before the epoch (which would itself be a
/// bug worth surfacing elsewhere).
pub(crate) fn now_unix_ms() -> i64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_millis() as i64,
        Err(_) => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn now_is_positive() {
        let t = now_unix_ms();
        assert!(t > 1_700_000_000_000, "checked_at_ms should be a recent timestamp, got {t}");
    }
}
