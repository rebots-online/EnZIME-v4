//! The source a backup reads from on create, and the sink a restore writes
//! back to on commit. Both are **local typed sentinels** that the
//! `storage-sqlite` Ring-1 crate will implement once it lands — see the
//! per-method I-10(b) markers.
//!
//! None of these traits touch the §6.2 schema directly. They are deliberately
//! coarse-grained so that the entire backup/restore transaction can be made
//! atomic by the implementer (e.g. one SQLite transaction per
//! `restore_commit`).

use async_trait::async_trait;

use crate::error::BackupError;
use crate::payload::{BackupFilter, UserDataSnapshot};

/// What a restore does when the backup and the live DB both contain the same
/// primary-key row. Mirrors the §14.3 `BackupRestoreCommitRequest`
/// `conflictPolicy` enum character-for-character.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConflictPolicy {
    /// Keep the local row; do not overwrite from the backup.
    KeepLocal,
    /// Replace the local row with the backup's copy.
    KeepBackup,
    /// Keep both; the imported row gets a fresh primary key so neither is
    /// lost.
    KeepBoth,
}

impl ConflictPolicy {
    /// Parse the §14.3 string form. Unknown strings become
    /// [`BackupError::InvalidRequest`].
    pub fn from_request_str(s: &str) -> Result<Self, BackupError> {
        match s {
            "keep_local" => Ok(ConflictPolicy::KeepLocal),
            "keep_backup" => Ok(ConflictPolicy::KeepBackup),
            "keep_both" => Ok(ConflictPolicy::KeepBoth),
            other => Err(BackupError::InvalidRequest(format!(
                "conflictPolicy '{other}' is not one of keep_local|keep_backup|keep_both"
            ))),
        }
    }
}

/// One conflict discovered while planning a restore. `entity_kind` matches
/// the §14.3 `BackupRestorePlan.conflicts` strings: `"pack_binding"`,
/// `"annotation"`, `"conversation"`, `"lore_memory"`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RestoreConflict {
    pub entity_kind: &'static str,
    pub primary_key: String,
}

/// The source/sink the backup pipeline talks to. Local typed sentinel — the
/// concrete implementation will live in `storage-sqlite` and operate inside
/// a single SQLite transaction so that §13.5 ("never destroy or hide
/// existing user data") is enforced: any error mid-commit rolls back and
/// leaves existing data untouched.
///
/// // I-10(b): swap for a real `storage-sqlite::UserDataSource` when it lands.
#[async_trait]
pub trait BackupSource: Send + Sync {
    /// Read the user-data snapshot that will be serialised into the backup.
    /// Apply `filter` (drop conversations/lore the user opted out of) before
    /// returning.
    ///
    /// Implementations must be **read-only** — §A-06 places user data above
    /// engines, and a backup must not mutate the source.
    async fn export_snapshot(&self, filter: BackupFilter) -> Result<UserDataSnapshot, BackupError>;

    /// Pre-flight check: given an incoming snapshot (e.g. one just decrypted
    /// from a backup file), return the list of primary-key conflicts with
    /// the live data. This is what populates the §14.3
    /// `BackupRestorePlan.conflicts` array.
    ///
    /// Implementations are read-only; this is *planning*, not commit.
    async fn detect_conflicts(
        &self,
        incoming: &UserDataSnapshot,
    ) -> Result<Vec<RestoreConflict>, BackupError>;

    /// Apply `incoming` to the live data store under `policy`. The
    /// implementation MUST be atomic: a [`BackupError`] returned from this
    /// method guarantees **zero** rows in the live store were added,
    /// modified or removed. This is the §13.5 contract — a failed restore
    /// leaves existing data completely unchanged.
    ///
    /// Returns the final row counts per entity kind so the caller can
    /// surface a summary to the UI.
    async fn import_snapshot(
        &self,
        incoming: UserDataSnapshot,
        policy: ConflictPolicy,
    ) -> Result<RestoreCounts, BackupError>;
}

/// Per-entity-kind post-restore counts. Used to populate the restore
/// summary returned to the UI.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RestoreCounts {
    pub pack_bindings: u64,
    pub annotations: u64,
    pub conversations: u64,
    pub lore_memories: u64,
}
