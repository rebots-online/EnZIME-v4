//! Where a backup artifact lives on disk.
//!
//! Mirrors the §6.2 `backups.destination_kind` CHECK constraint exactly
//! (`'file' | 'directory' | 'android_document'`) plus the matching
//! `destination_locator` string. The Tauri layer translates the opaque
//! `destinationToken` returned by `backup_choose_destination` into this
//! enum before calling the creator; the backup crate itself never
//! interprets platform tokens (the `ports::platform` port is being fixed
//! in parallel — I-10(b)).

use serde::{Deserialize, Serialize};

use crate::error::BackupError;

/// The §6.2 `destination_kind` column, verbatim.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DestinationKind {
    /// The destination locator is a single file path that will be created
    /// (or overwritten atomically) with the backup artifact.
    File,
    /// The destination locator is a directory; the backup crate picks a
    /// unique file name inside it.
    Directory,
    /// The destination locator is an opaque, platform-mediated document
    /// reference (Android SAF tree / document URI). The actual byte pipe
    /// is owned by `ports::platform::PlatformAdapter` (I-10(b)); the backup
    /// crate records the locator verbatim and never resolves it to a path.
    AndroidDocument,
}

impl DestinationKind {
    /// The string stored in `backups.destination_kind`. Matches the §6.2
    /// CHECK constraint character-for-character; do not rename.
    pub fn as_db_str(self) -> &'static str {
        match self {
            DestinationKind::File => "file",
            DestinationKind::Directory => "directory",
            DestinationKind::AndroidDocument => "android_document",
        }
    }

    /// Inverse of [`as_db_str`]; used when reading a `backups` row back.
    pub fn from_db_str(s: &str) -> Result<Self, BackupError> {
        match s {
            "file" => Ok(DestinationKind::File),
            "directory" => Ok(DestinationKind::Directory),
            "android_document" => Ok(DestinationKind::AndroidDocument),
            other => Err(BackupError::FormatInvalid(format!(
                "destination_kind '{other}' is not one of file|directory|android_document"
            ))),
        }
    }
}

/// The §6.2 `destination_kind` + `destination_locator` pair, together with
/// the on-disk path used by the local `file` / `directory` flows. The
/// `local_path` is `None` for `AndroidDocument` (the platform adapter owns
/// the byte pipe there — I-10(b)).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackupDestination {
    pub kind: DestinationKind,
    pub locator: String,
    /// Absolute filesystem path for `File` and `Directory` destinations.
    /// `None` for `AndroidDocument`.
    pub local_path: Option<std::path::PathBuf>,
}

impl BackupDestination {
    /// Build a `File` destination pointing at `path`. The locator stored in
    /// the DB is the absolute path rendered as a string (per §14.1 the only
    /// time a raw filesystem path is returned to the UI is when the user
    /// picked it; in that case the DTO field is `displayPath`).
    pub fn file(path: impl Into<std::path::PathBuf>) -> Result<Self, BackupError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(BackupError::Destination("file path is empty".into()));
        }
        let locator = path
            .to_str()
            .ok_or_else(|| BackupError::Destination("file path is not UTF-8".into()))?
            .to_string();
        Ok(Self {
            kind: DestinationKind::File,
            locator,
            local_path: Some(path),
        })
    }

    /// Build a `Directory` destination. The artifact file inside the
    /// directory is named `<backup_id>.enzb`.
    pub fn directory(path: impl Into<std::path::PathBuf>) -> Result<Self, BackupError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(BackupError::Destination("directory path is empty".into()));
        }
        let locator = path
            .to_str()
            .ok_or_else(|| BackupError::Destination("directory path is not UTF-8".into()))?
            .to_string();
        Ok(Self {
            kind: DestinationKind::Directory,
            locator,
            local_path: Some(path),
        })
    }

    /// Build an `AndroidDocument` destination. The locator is the opaque
    /// platform document reference; the backup crate writes/reads through
    /// the `BackupSink` / `BackupSource` byte pipes only.
    pub fn android_document(opaque_locator: impl Into<String>) -> Self {
        Self {
            kind: DestinationKind::AndroidDocument,
            locator: opaque_locator.into(),
            local_path: None,
        }
    }

    /// Final on-disk artifact path for `File` and `Directory` destinations.
    /// Returns `None` for `AndroidDocument`.
    pub fn artifact_path(&self, backup_id: &str) -> Option<std::path::PathBuf> {
        match self.kind {
            DestinationKind::File => self.local_path.clone(),
            DestinationKind::Directory => self
                .local_path
                .as_ref()
                .map(|d| d.join(format!("{backup_id}.enzb"))),
            DestinationKind::AndroidDocument => None,
        }
    }
}
