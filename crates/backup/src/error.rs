//! Errors returned by the backup create / restore pipeline.
//!
//! Variants are matched 1:1 to the failure modes the §13.5 capability
//! gating rules and the §14 IPC `AppErrorDto` codes require callers to
//! surface to the UI:
//!
//! - [`BackupError::CapabilityDenied`] → IPC `capability_denied`
//! - [`BackupError::InvalidRequest`]    → IPC `invalid_request`
//! - [`BackupError::FormatInvalid`]     → IPC `format_invalid`
//! - [`BackupError::VerificationFailed`] → IPC `verification_failed`
//! - [`BackupError::PassphraseRequired`] / [`BackupError::PassphraseWrong`]
//!   → IPC `invalid_request` / `verification_failed`
//! - [`BackupError::Conflict`]          → IPC `revision_conflict`
//! - [`BackupError::Destination`]       → IPC `choose_storage`
//! - [`BackupError::Io`]                → IPC `internal`

use std::io;

#[derive(Debug, thiserror::Error)]
pub enum BackupError {
    /// Caller asked for an encrypted backup but the `backup.encrypted`
    /// capability was denied. Per §13.5 ("never block export of
    /// user-created data") the caller can retry with `encrypted: false`.
    #[error("capability denied: {capability}")]
    CapabilityDenied { capability: &'static str },

    /// Request shape is unusable (e.g. `encrypted: true` with empty
    /// passphrase, or an unknown conflict policy).
    #[error("invalid request: {0}")]
    InvalidRequest(String),

    /// Artifact bytes do not parse as a known EnZIME backup (bad magic,
    /// unknown schema version, truncated, etc.).
    #[error("backup artifact format is invalid: {0}")]
    FormatInvalid(String),

    /// A SHA-256 read back from the just-written artifact did not match the
    /// manifest, OR the payload digest recorded in the manifest does not
    /// match the payload we just decrypted. A backup is marked `verified`
    /// only after this check has passed — never on the strength of having
    /// written the file.
    #[error("verification failed: {0}")]
    VerificationFailed(String),

    /// `encrypted: true` was supplied without a passphrase.
    #[error("encrypted backup requested but no passphrase was supplied")]
    PassphraseRequired,

    /// Decryption produced an invalid authentication tag (Poly1305) — almost
    /// always a wrong passphrase or a tampered artifact.
    #[error("passphrase is wrong or artifact has been tampered with")]
    PassphraseWrong,

    /// Restore would clobber existing rows in a way the supplied
    /// `ConflictPolicy` does not license. The caller surfaces this as a
    /// `revision_conflict` and asks the user to pick a stricter policy.
    #[error("restore conflict: {0}")]
    Conflict(String),

    /// Destination token does not resolve, or the resolved destination
    /// cannot be written to / read from.
    #[error("destination error: {0}")]
    Destination(String),

    /// Underlying IO failure (file system full, permission denied, …).
    #[error("io error: {0}")]
    Io(#[from] io::Error),

    /// (De)serialization of the manifest or payload failed.
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}
