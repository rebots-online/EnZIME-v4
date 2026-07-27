#![forbid(unsafe_code)]

//! Backup creation and restore for EnZIME Reader.
//!
//! Implements ARCHITECTURE.md §13.5 ("capability gating") and the backup /
//! restore behaviour it references:
//!
//! - **Versioned backup manifest** ([`manifest::BackupManifest`]). Each
//!   artifact is `magic "ENZB" | format_version | u32 manifest_len | manifest
//!   JSON | payload` (see [`artifact`]). The manifest SHA-256 is what gets
//!   stored in the §6.2 `backups.manifest_sha256` column.
//!
//! - **Optional encryption** ([`crypto`]). Argon2id passphrase KDF +
//!   XChaCha20-Poly1305 AEAD. Encryption is the §13.1 `backup.encrypted`
//!   paid capability. Plaintext backup is always allowed per §13.5 ("never
//!   block export of user-created data").
//!
//! - **Atomic export** ([`create::BackupCreator`]). Stage to a `.partial`
//!   file with `O_CREAT | O_EXCL`, fsync, then read-back-verify, then atomic
//!   rename. On any error the `.partial` is removed.
//!
//! - **Read-back verification**. A backup is `state = 'verified'` only after
//!   the just-written artifact has been reopened, decoded, decrypted (if
//!   applicable) and re-hashed — and the resulting SHA-256 matches the
//!   manifest's recorded `payload_sha256`. Never on the strength of having
//!   written the file.
//!
//! - **Failed-restore atomicity** ([`restore::BackupRestorer`]). Restore is
//!   two phases: a read-only `plan` and an atomic `commit`. All write
//!   atomicity is delegated to the [`source::BackupSource`] implementer,
//!   whose contract is that any error returned from `import_snapshot` leaves
//!   the live store untouched. This is the §13.5 rule: a failed restore
//!   never destroys or hides existing user data.
//!
//! The crate does **not** touch the §6.2 SQL schema; the [`create::VerifiedBackup`]
//! record it returns is what the storage-sqlite adapter persists into the
//! `backups` row, with the column names matching §6.2 character-for-character.
//!
//! The `ports::platform` port is being fixed in parallel; the Android SAF
//! byte pipe is therefore stubbed to `Err(Destination)` here and marked
//! `// I-10(b): swap when ports::platform lands`.

pub mod artifact;
pub mod capability;
pub mod create;
pub mod crypto;
pub mod destination;
pub mod error;
pub mod manifest;
pub mod payload;
pub mod restore;
pub mod settings;
pub mod source;

#[cfg(test)]
mod tests;

pub use capability::{CapabilityDecision, CapabilityDecisionSource};
pub use create::{
    BackupArtifactSummary, BackupCreateRequest, BackupCreator, BackupState, JobKind, JobReference,
    VerifiedBackup,
};
pub use destination::{BackupDestination, DestinationKind};
pub use error::BackupError;
pub use manifest::{BackupCounts, BackupManifest, CURRENT_SCHEMA_VERSION};
pub use payload::{
    AnnotationRecord, BackupFilter, ConversationMessage, LoreMemory, PackBinding, UserDataSnapshot,
};
pub use restore::{
    BackupRestoreCommitRequest, BackupRestorePlan, BackupRestorePlanRequest, BackupRestorer,
};
pub use settings::BackupSettings;
pub use source::{BackupSource, ConflictPolicy, RestoreConflict, RestoreCounts};
