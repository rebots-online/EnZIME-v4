//! Directory layout of `<storage-root>/EnZIME/` (ARCHITECTURE.md §6.3).
//!
//! The layout is fixed:
//!
//! ```text
//! <storage-root>/EnZIME/
//! ├── objects/sha256/ab/<62 remaining hex characters>
//! ├── staging/<install-transaction-id>/
//! ├── quarantine/<audit-event-id>/
//! ├── packs/<pack-id>/<pack-version-id>/
//! ├── models/<model-id>/<artifact-sha256>/
//! ├── sidecars/
//! ├── backups/
//! └── locks/
//! ```
//!
//! This module owns the names and the path-construction helpers. It does
//! not touch the filesystem — that is the job of [`crate::store::ObjectStore`]
//! and [`crate::migration`].

use std::path::{Path, PathBuf};

use crate::error::ObjectStoreError;

/// Top-level directory under the storage root. Constant so every path is
/// constructed from the same name.
pub const ENZIME_DIR: &str = "EnZIME";

/// Content-addressed object tree root. Below this, objects are sharded by
/// digest algorithm and the first two hex characters of the digest.
pub const OBJECTS_DIR: &str = "objects";
/// Currently the only supported digest algorithm directory.
pub const SHA256_DIR: &str = "sha256";

/// Per-install-transaction staging area. One sub-directory per
/// install-transaction-id; inside it, one staging file per object.
pub const STAGING_DIR: &str = "staging";

/// Quarantine area. One sub-directory per audit-event-id, holding bytes that
/// failed verification so they are preserved for forensic review and never
/// become active.
pub const QUARANTINE_DIR: &str = "quarantine";

/// Pack artifacts (`.zim`/`.enzime` files and their per-version side data).
pub const PACKS_DIR: &str = "packs";

/// Model artifacts, addressed by `<model-id>/<artifact-sha256>`.
pub const MODELS_DIR: &str = "models";

/// Sidecar files (signed annotation/lore exports per §17).
pub const SIDECARS_DIR: &str = "sidecars";

/// Encrypted backup artifacts.
pub const BACKUPS_DIR: &str = "backups";

/// Filesystem lock files (one per active long-running operation).
pub const LOCKS_DIR: &str = "locks";

/// Name of the on-disk marker that records which layout migrations have been
/// applied. This is the filesystem analogue of the SQLite `schema_migrations`
/// table owned by the `storage-sqlite` crate (§6.4). It lives at the root of
/// the EnZIME directory so it is visible alongside the layout it describes.
pub const MIGRATION_MARKER_NAME: &str = "schema_migrations";

/// All sub-directory names that §6.3 lists, in the order they appear in the
/// spec. Used by the v1 layout migration and by postflight checks.
pub const LAYOUT_SUBDIRS: &[&str] = &[
    OBJECTS_DIR,
    STAGING_DIR,
    QUARANTINE_DIR,
    PACKS_DIR,
    MODELS_DIR,
    SIDECARS_DIR,
    BACKUPS_DIR,
    LOCKS_DIR,
];

/// Number of hex characters of a SHA-256 digest used as the shard prefix.
/// The spec writes the layout as `objects/sha256/ab/<62 remaining hex>`, so
/// the shard is two characters and the remaining filename is sixty-two.
pub const SHA256_SHARD_LEN: usize = 2;
/// Length of a full lowercase SHA-256 hex string.
pub const SHA256_HEX_LEN: usize = 64;

/// The absolute path of the EnZIME directory under a storage root.
pub fn enzime_root(storage_root: &Path) -> PathBuf {
    storage_root.join(ENZIME_DIR)
}

/// `objects/sha256` under the given EnZIME root.
pub fn objects_root(enzime_root: &Path) -> PathBuf {
    enzime_root.join(OBJECTS_DIR).join(SHA256_DIR)
}

/// `staging` under the given EnZIME root.
pub fn staging_root(enzime_root: &Path) -> PathBuf {
    enzime_root.join(STAGING_DIR)
}

/// `quarantine` under the given EnZIME root.
pub fn quarantine_root(enzime_root: &Path) -> PathBuf {
    enzime_root.join(QUARANTINE_DIR)
}

/// `packs` under the given EnZIME root.
pub fn packs_root(enzime_root: &Path) -> PathBuf {
    enzime_root.join(PACKS_DIR)
}

/// `models` under the given EnZIME root.
pub fn models_root(enzime_root: &Path) -> PathBuf {
    enzime_root.join(MODELS_DIR)
}

/// `sidecars` under the given EnZIME root.
pub fn sidecars_root(enzime_root: &Path) -> PathBuf {
    enzime_root.join(SIDECARS_DIR)
}

/// `backups` under the given EnZIME root.
pub fn backups_root(enzime_root: &Path) -> PathBuf {
    enzime_root.join(BACKUPS_DIR)
}

/// `locks` under the given EnZIME root.
pub fn locks_root(enzime_root: &Path) -> PathBuf {
    enzime_root.join(LOCKS_DIR)
}

/// Split a 64-character lowercase SHA-256 hex string into the two-character
/// shard and the sixty-two-character remainder, returning them by reference
/// into the input.
pub fn split_sha256_hex(hex: &str) -> Result<(&str, &str), ObjectStoreError> {
    if hex.len() != SHA256_HEX_LEN
        || !hex
            .chars()
            .all(|c| matches!(c, '0'..='9' | 'a'..='f'))
    {
        return Err(ObjectStoreError::MalformedDigestHex(hex.to_string()));
    }
    let (shard, rest) = hex.split_at(SHA256_SHARD_LEN);
    Ok((shard, rest))
}

/// The full content-addressed path for an object with the given SHA-256 hex,
/// relative to (i.e. inside) the EnZIME root.
///
/// Format: `objects/sha256/<2-char shard>/<62-char remainder>`.
pub fn object_relpath(sha256_hex: &str) -> Result<PathBuf, ObjectStoreError> {
    let (shard, rest) = split_sha256_hex(sha256_hex)?;
    Ok(PathBuf::from(OBJECTS_DIR)
        .join(SHA256_DIR)
        .join(shard)
        .join(rest))
}

/// Validate that a string is safe to use as a single path component.
///
/// The §6.3 layout uses caller-supplied identifiers (`install-transaction-id`,
/// `audit-event-id`, `pack-id`, `pack-version-id`, `model-id`) directly as
/// path components. The A-05 invariant ("identity is not location") requires
/// us to reject anything that could escape its intended directory.
pub fn validate_path_component(raw: &str) -> Result<(), ObjectStoreError> {
    if raw.trim().is_empty() {
        return Err(ObjectStoreError::InvalidIdentifier(
            "identifier is empty".to_string(),
        ));
    }
    if raw.len() > 255 {
        // 255 bytes is the typical NAME_MAX on Linux/macOS ext4/apfs/hfs+.
        return Err(ObjectStoreError::InvalidIdentifier(format!(
            "identifier exceeds 255 bytes (was {})",
            raw.len()
        )));
    }
    if raw.contains('/')
        || raw.contains('\\')
        || raw.contains('\0')
        || raw == "."
        || raw == ".."
        || raw.starts_with('.')
        || raw.contains("..")
    {
        return Err(ObjectStoreError::InvalidIdentifier(format!(
            "identifier contains a path or URL component: {raw}"
        )));
    }
    // Reject Windows drive prefixes ("C:" etc.) — the same check the domain
    // identity module applies.
    let mut chars = raw.chars();
    if let (Some(letter), Some(':')) = (chars.next(), chars.next()) {
        if letter.is_ascii_alphabetic() {
            return Err(ObjectStoreError::InvalidIdentifier(format!(
                "identifier contains a path or URL component: {raw}"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_hex_round_trips() {
        let hex = "ab0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcd";
        let (shard, rest) = split_sha256_hex(hex).unwrap();
        assert_eq!(shard, "ab");
        assert_eq!(rest.len(), 62);
        assert_eq!(format!("{shard}{rest}"), hex);
    }

    #[test]
    fn split_hex_rejects_uppercase_and_short() {
        assert!(split_sha256_hex("AB").is_err());
        assert!(split_sha256_hex(
            "ab0123456789abcdef0123456789abcdef0123456789abcdef0123456789ABCDEF"
        )
        .is_err());
    }

    #[test]
    fn object_relpath_shape() {
        let hex = "zz"; // invalid, ensures we use a real one below
        assert!(split_sha256_hex(hex).is_err());
        let real = "ab0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcd";
        let p = object_relpath(real).unwrap();
        assert_eq!(
            p,
            PathBuf::from("objects/sha256/ab/0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcd")
        );
    }

    #[test]
    fn path_component_rejects_traversal() {
        assert!(validate_path_component("").is_err());
        assert!(validate_path_component("a/b").is_err());
        assert!(validate_path_component("a\\b").is_err());
        assert!(validate_path_component("..").is_err());
        assert!(validate_path_component(".").is_err());
        assert!(validate_path_component(".hidden").is_err());
        assert!(validate_path_component("C:drive").is_err());
        assert!(validate_path_component("a\0b").is_err());
        assert!(validate_path_component("ok-name_1").is_ok());
        assert!(validate_path_component("a..b").is_err());
    }
}
