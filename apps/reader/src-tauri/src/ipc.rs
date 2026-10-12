//! Native IPC composition. DynDon discovery, bounded acquisition and offline
//! archive restoration are implemented in focused sibling modules.

use std::path::Path;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use bytes::Bytes;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};

use ports::transfer::{TransferMode, TransferProbeRequest, TransferRange, TransferSource};
use rusqlite::{params, Connection, OptionalExtension};

use crate::state::AppState;

pub mod annotations;
pub mod dyndon;
pub mod kiwix;

// ===========================================================================
// Helpers
// ===========================================================================

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn platform_string() -> String {
    if cfg!(target_os = "android") {
        "android".to_string()
    } else if cfg!(target_os = "windows") {
        "windows".to_string()
    } else {
        // Covers `target_os = "linux"` (the only other §17.1 platform) and,
        // for development on other Unixes, falls back to the closest §17.1
        // value rather than emitting a value outside the §14.5 union.
        "linux".to_string()
    }
}

/// Map any error into a `String` for the `Result<T, String>` command return.
fn err_str<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

/// Real free-space probe via `libc::statvfs` (Unix only). Returns `None` on
/// non-Unix targets or when the syscall fails — the caller then escalates.
#[cfg(unix)]
fn probe_free_bytes(path: &Path) -> Option<u64> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;
    let c_path = CString::new(path.as_os_str().as_bytes()).ok()?;
    // SAFETY: `statvfs` writes into an out-pointer and reads the path
    // read-only. The path is a NUL-terminated CString. Return value is
    // checked.
    unsafe {
        let mut stat: libc::statvfs = std::mem::zeroed();
        if libc::statvfs(c_path.as_ptr(), &mut stat) == 0 {
            // f_bavail = free blocks available to unprivileged user.
            Some(stat.f_bsize as u64 * stat.f_bavail as u64)
        } else {
            None
        }
    }
}

#[cfg(not(unix))]
fn probe_free_bytes(_path: &Path) -> Option<u64> {
    // Windows free-space probe is a TODO for the platform-adapter crate.
    None
}

/// Resolve the transfer source for a given URI scheme. `file://` and bare
/// absolute paths go through `LocalFileTransferSource`; `http(s)://` goes
mod dto;
mod home;
mod library;
mod reader;
mod rows;
pub use dto::*;
pub use home::*;
pub use library::*;
pub use reader::*;
use rows::*;
