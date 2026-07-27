//! Errors raised by the object store and its migration runner.

use std::io;
use std::path::PathBuf;

use domain::identity::IdentityError;
use thiserror::Error;

/// All failures the object store can produce.
///
/// The variants are split so callers can distinguish verification failures
/// (which the §6.3 algorithm routes to quarantine) from infrastructure
/// failures (which the caller may retry).
#[derive(Debug, Error)]
pub enum ObjectStoreError {
    /// The caller passed a storage root that is not an absolute path.
    #[error("storage root must be absolute, got {0}")]
    RelativeRoot(String),

    /// The storage root exists but is not a directory we can write into.
    #[error("storage root is not a directory: {0}")]
    NotADirectory(PathBuf),

    /// The on-disk layout marker is present but unreadable or wrong shape.
    #[error("layout marker is corrupt at {path}: {reason}")]
    LayoutMarkerCorrupt { path: PathBuf, reason: String },

    /// The on-disk layout version is older/newer than this crate understands.
    #[error("layout version mismatch: crate supports <= {supported}, found {found}")]
    LayoutVersion { found: u32, supported: u32 },

    /// A caller-supplied identifier (transaction id, audit event id, staging
    /// key) was rejected because it could be confused with a path.
    ///
    /// The object store constructs filesystem paths from these identifiers;
    /// rejecting path-bearing identifiers here is the A-05 ("identity is not
    /// location") enforcement for the storage layer.
    #[error("identifier is not a valid path component: {0}")]
    InvalidIdentifier(String),

    /// A domain identity value failed its own validation.
    #[error("domain identity rejected: {0}")]
    Identity(#[from] IdentityError),

    /// An identifier that should be a 64-character lowercase SHA-256 hex was
    /// not one.
    #[error("expected 64-char lowercase sha256 hex, got {0}")]
    MalformedDigestHex(String),

    /// The caller attempted to start a transaction whose staging directory
    /// already exists. Each install-transaction-id must be unique while live.
    #[error("staging directory already exists for transaction {0}")]
    StagingExists(String),

    /// A quarantine slot for the given audit event id already exists. The
    /// audit event id must be unique per quarantined object.
    #[error("quarantine directory already exists for audit event {0}")]
    QuarantineExists(String),

    /// The streamed object exceeded the declared maximum size. The bytes
    /// streamed so far are reported along with the limit that was hit.
    #[error("streamed {streamed} bytes exceeded declared maximum {max}")]
    ExceededMaxSize { streamed: u64, max: u64 },

    /// The number of bytes streamed does not match the length the caller
    /// declared. Both verification (this variant) and the digest comparison
    /// (`DigestMismatch`) must pass before the bytes can be activated.
    #[error("staged length {staged} != expected length {expected}")]
    LengthMismatch { staged: u64, expected: u64 },

    /// The SHA-256 computed from the staged bytes does not match the digest
    /// the caller declared. This is the A-07 invariant in its negative form:
    /// the bytes never become an active object.
    #[error("computed digest {computed} != expected digest {expected}")]
    DigestMismatch { computed: String, expected: String },

    /// The format verifier inspected the staged bytes and rejected them.
    /// The contained string is the verifier's reason.
    #[error("format verifier rejected staged bytes: {0}")]
    FormatRejected(String),

    /// An object with the same content-addressed path already exists, but
    /// with different bytes than the staging file we just verified. On a
    /// content-addressed store this should be impossible (same digest => same
    /// content); reaching this variant indicates filesystem tampering.
    #[error("object already exists at {path} with different content")]
    ExistingObjectConflict { path: PathBuf },

    /// The caller asked for an object that is not present in the store.
    #[error("object not found: {0}")]
    NotFound(String),

    /// The underlying filesystem operation failed.
    #[error("io error: {0}")]
    Io(#[from] io::Error),

    /// A migration registry operation failed (unknown version, tampered
    /// marker, mismatched sha256, etc.).
    #[error("migration error: {0}")]
    Migration(#[from] MigrationError),
}

/// Failures specific to the migration runner.
#[derive(Debug, Error)]
pub enum MigrationError {
    /// The on-disk migration registry is at a version this crate does not
    /// understand.
    #[error("registry version {found} is newer than crate maximum {max}")]
    FutureVersion { found: u32, max: u32 },

    /// The recorded SHA-256 for an applied migration does not match the
    /// SHA-256 of the migration the crate would apply for that version. This
    /// is evidence that either the on-disk marker file or the compiled
    /// migration has been tampered with.
    #[error("sha256 mismatch for migration version {version}: marker has {marker}, crate has {crate_sha256}")]
    ShaMismatch {
        version: u32,
        marker: String,
        crate_sha256: String,
    },

    /// The marker file is present but unreadable or malformed.
    #[error("migration marker at {path} is corrupt: {reason}")]
    MarkerCorrupt { path: PathBuf, reason: String },

    /// The preflight check for a migration failed.
    #[error("preflight for migration {version} ({name}) failed: {reason}")]
    PreflightFailed {
        version: u32,
        name: String,
        reason: String,
    },

    /// The postflight check for a migration failed.
    #[error("postflight for migration {version} ({name}) failed: {reason}")]
    PostflightFailed {
        version: u32,
        name: String,
        reason: String,
    },

    /// An I/O error during migration application.
    #[error("io error during migration: {0}")]
    Io(#[from] io::Error),
}

/// The verdict returned by an [`crate::verifier::ObjectFormatVerifier`] when
/// it rejects the staged bytes.
///
/// This is a struct (not an enum variant) so verifiers can carry an
/// operator-facing reason string into the quarantine audit trail verbatim.
#[derive(Debug, Clone)]
pub struct FormatRejection {
    /// Short, machine-readable token (e.g. `"truncated"`, `"bad-magic"`).
    pub code: String,
    /// Human-readable explanation that will be persisted alongside the
    /// quarantined bytes.
    pub reason: String,
}

impl FormatRejection {
    pub fn new(code: impl Into<String>, reason: impl Into<String>) -> Self {
        FormatRejection {
            code: code.into(),
            reason: reason.into(),
        }
    }
}

impl std::fmt::Display for FormatRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.reason)
    }
}
