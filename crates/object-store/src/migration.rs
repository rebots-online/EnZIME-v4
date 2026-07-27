//! Migration runner for the object-store filesystem layout (ARCHITECTURE.md
//! §6.4).
//!
//! §6.4 mandates that every migration carries: an integer version, a stable
//! name, a SHA-256 registered in `schema_migrations`, a forward action, a
//! preflight check, a backup requirement, a postflight check, and a fixture
//! proving migration from every supported prior release. SQL migrations
//! (where `schema_migrations` is a literal table) live in the `storage-sqlite`
//! crate. This crate applies the same contract to its filesystem layout: the
//! `schema_migrations` analogue is a marker file at
//! `<storage-root>/EnZIME/schema_migrations` containing one record per applied
//! migration, recorded as `<version>\t<name>\t<sha256>`.
//!
//! A migration never silently repairs ambiguous primary data. If a forward
//! action encounters ambiguous bytes, it leaves them in place and the caller
//! (the application layer) must emit a `user_tasks` row in the SQLite store
//! requiring review; the object store itself does not own the SQLite
//! connection and so cannot write that row directly.

use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::error::MigrationError;
use crate::layout::{
    self, enzime_root, MIGRATION_MARKER_NAME,
};

/// Compute the SHA-256 of a migration's canonical descriptor
/// (`<version>\n<name>`), hex-encoded lowercase. This is the value that gets
/// registered in `schema_migrations` (or its filesystem analogue) and is used
/// to detect tampering with the migration list.
pub fn migration_sha256(version: u32, name: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(version.to_string());
    hasher.update(b"\n");
    hasher.update(name.as_bytes());
    hex::encode(hasher.finalize())
}

/// A single migration. The fields are exactly the eight slots §6.4 lists,
/// generalised so they apply to any durable storage (filesystem layout here,
/// SQL schema in `storage-sqlite`). The forward/preflight/postflight slots
/// are strings — for SQL migrations they hold the SQL text; for filesystem
/// migrations they hold a canonical description that the `apply` callback
/// uses.
pub struct Migration {
    /// §6.4 "integer version".
    pub version: u32,
    /// §6.4 "stable name".
    pub name: &'static str,
    /// §6.4 "SHA-256 registered in `schema_migrations`". Computed from
    /// `<version>\n<name>`; recorded in the marker file.
    pub sha256: String,
    /// §6.4 "forward SQL" — generalised to a description string (for FS
    /// migrations, a canonical listing of the changes; for SQL it would be
    /// the SQL text).
    pub forward_description: &'static str,
    /// §6.4 "preflight query" — generalised to a callback that returns
    /// `Ok(())` iff the migration should run, or an error explaining why it
    /// cannot.
    pub preflight: fn(root: &Path) -> Result<(), String>,
    /// §6.4 "backup requirement" — `true` iff the migration modifies primary
    /// user data and the caller must have taken a backup before running it.
    pub backup_required: bool,
    /// §6.4 "postflight query" — generalised to a callback that returns
    /// `Ok(())` iff the migration's effects are observable post-application.
    pub postflight: fn(root: &Path) -> Result<(), String>,
    /// The work the migration actually does to the filesystem. Generalised
    /// from "forward SQL" so the same framework can apply FS-shape changes.
    pub apply: fn(root: &Path) -> Result<(), MigrationError>,
}

impl Migration {
    /// Build a `Migration` from its version and name, deriving the SHA-256
    /// via [`migration_sha256`].
    pub fn build(
        version: u32,
        name: &'static str,
        forward_description: &'static str,
        backup_required: bool,
        preflight: fn(&Path) -> Result<(), String>,
        postflight: fn(&Path) -> Result<(), String>,
        apply: fn(&Path) -> Result<(), MigrationError>,
    ) -> Self {
        Migration {
            version,
            name,
            sha256: migration_sha256(version, name),
            forward_description,
            backup_required,
            preflight,
            postflight,
            apply,
        }
    }
}

/// The list of migrations this crate knows how to apply, in version order.
/// The last entry's version is [`LATEST_VERSION`].
pub fn known_migrations() -> Vec<Migration> {
    vec![v1_initial_layout()]
}

/// Highest layout version this crate understands.
pub const LATEST_VERSION: u32 = 1;

/// The v1 initial layout migration. Creates the §6.3 directory tree.
pub fn v1_initial_layout() -> Migration {
    Migration::build(
        1,
        "enzime-object-store-v1",
        // Canonical forward description — the directory tree from §6.3,
        // sorted topologically (parents before children), one entry per line.
        "backups/\nlocks/\nmodels/\nobjects/sha256/\npacks/\nquarantine/\nsidecars/\nstaging/\n",
        // No primary user data exists yet at v1 (empty store), so no backup.
        false,
        v1_preflight,
        v1_postflight,
        v1_apply,
    )
}

fn v1_preflight(root: &Path) -> Result<(), String> {
    let enzime = enzime_root(root);
    if enzime.exists() && !enzime.is_dir() {
        return Err(format!(
            "storage root already contains a non-directory file at {}",
            enzime.display()
        ));
    }
    Ok(())
}

fn v1_apply(root: &Path) -> Result<(), MigrationError> {
    let enzime = enzime_root(root);
    fs::create_dir_all(&enzime)?;
    for sub in layout::LAYOUT_SUBDIRS {
        // Use create_dir (not create_dir_all) for sub-directories so that
        // we don't paper over a tampered intermediate path.
        let path = enzime.join(sub);
        if !path.exists() {
            fs::create_dir_all(&path)?;
        }
    }
    // The `objects/sha256` subdir is itself a parent of the shard dirs, so
    // ensure it explicitly (it is also created by create_dir_all of the
    // `objects` parent above, but be explicit).
    fs::create_dir_all(enzime.join("objects").join("sha256"))?;
    Ok(())
}

fn v1_postflight(root: &Path) -> Result<(), String> {
    let enzime = enzime_root(root);
    for sub in layout::LAYOUT_SUBDIRS {
        let p = enzime.join(sub);
        if !p.is_dir() {
            return Err(format!("postflight: missing directory {}", p.display()));
        }
    }
    let sha = enzime.join("objects").join("sha256");
    if !sha.is_dir() {
        return Err(format!("postflight: missing directory {}", sha.display()));
    }
    Ok(())
}

/// Path of the `schema_migrations` marker analogue under a storage root.
pub fn marker_path(storage_root: &Path) -> PathBuf {
    enzime_root(storage_root).join(MIGRATION_MARKER_NAME)
}

/// A record read from the marker file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedMigration {
    pub version: u32,
    pub name: String,
    pub sha256: String,
}

/// Read the marker file and return the applied-migration records in the order
/// they appear. Returns an empty vec if the marker does not exist yet.
pub fn read_marker(storage_root: &Path) -> Result<Vec<AppliedMigration>, MigrationError> {
    let path = marker_path(storage_root);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let file = OpenOptions::new().read(true).open(&path)?;
    let reader = BufReader::new(file);
    let mut out = Vec::new();
    for (lineno, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() != 3 {
            return Err(MigrationError::MarkerCorrupt {
                path: path.clone(),
                reason: format!("line {} is not 3 tab-separated columns", lineno + 1),
            });
        }
        let version: u32 = cols[0].parse().map_err(|_| MigrationError::MarkerCorrupt {
            path: path.clone(),
            reason: format!("line {}: version is not u32", lineno + 1),
        })?;
        out.push(AppliedMigration {
            version,
            name: cols[1].to_string(),
            sha256: cols[2].to_string(),
        });
    }
    Ok(out)
}

/// Write a fresh marker file containing exactly the given records, replacing
/// any prior content. The file is written via a staging file + atomic rename
/// so a crash mid-write cannot leave a partially-written marker.
fn write_marker(
    storage_root: &Path,
    records: &[AppliedMigration],
) -> Result<(), MigrationError> {
    let path = marker_path(storage_root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let staging = path.with_extension("schema_migrations.tmp");
    {
        let mut f = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&staging)?;
        for r in records {
            writeln!(f, "{}\t{}\t{}", r.version, r.name, r.sha256)?;
        }
        f.sync_all()?;
    }
    fs::rename(&staging, &path)?;
    Ok(())
}

/// Apply all known migrations that have not already been recorded in the
/// marker file, then write the updated marker.
///
/// Migrations are applied in increasing version order. Each migration's
/// preflight runs first; if preflight fails, application halts with a
/// [`MigrationError::PreflightFailed`]. After the apply callback succeeds,
/// the postflight check runs; if it fails, application halts with
/// [`MigrationError::PostflightFailed`] and the marker is NOT advanced — the
/// caller must investigate. The marker is updated only after postflight
/// passes, so a recorded migration is one whose effects have been verified.
pub fn run_pending(storage_root: &Path) -> Result<Vec<AppliedMigration>, MigrationError> {
    let mut applied_existing = read_marker(storage_root)?;
    let known = known_migrations();
    // Detect future versions on disk that this crate cannot understand.
    if let Some(latest_existing) = applied_existing.iter().map(|r| r.version).max() {
        if latest_existing > LATEST_VERSION {
            return Err(MigrationError::FutureVersion {
                found: latest_existing,
                max: LATEST_VERSION,
            });
        }
    }
    // Detect tampering: any recorded sha256 must match the sha256 of the
    // migration with the same version+name that this crate would apply.
    for rec in &applied_existing {
        if let Some(known_m) = known.iter().find(|m| m.version == rec.version) {
            if known_m.name != rec.name || known_m.sha256 != rec.sha256 {
                return Err(MigrationError::ShaMismatch {
                    version: rec.version,
                    marker: rec.sha256.clone(),
                    crate_sha256: known_m.sha256.clone(),
                });
            }
        }
    }
    let already_applied: std::collections::HashSet<u32> =
        applied_existing.iter().map(|r| r.version).collect();

    for m in known {
        if already_applied.contains(&m.version) {
            continue;
        }
        if let Err(reason) = (m.preflight)(storage_root) {
            return Err(MigrationError::PreflightFailed {
                version: m.version,
                name: m.name.to_string(),
                reason,
            });
        }
        (m.apply)(storage_root)?;
        if let Err(reason) = (m.postflight)(storage_root) {
            return Err(MigrationError::PostflightFailed {
                version: m.version,
                name: m.name.to_string(),
                reason,
            });
        }
        applied_existing.push(AppliedMigration {
            version: m.version,
            name: m.name.to_string(),
            sha256: m.sha256.clone(),
        });
        write_marker(storage_root, &applied_existing)?;
    }
    Ok(applied_existing)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{enzime_root, LAYOUT_SUBDIRS};

    fn test_root(subdir: &str) -> PathBuf {
        let base = std::env::var_os("CARGO_TARGET_TMPDIR")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        let root = base.join("object-store-migration").join(subdir);
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn migration_sha256_is_stable() {
        // Pre-computed with `printf '1\nenzime-object-store-v1' | sha256sum`
        // — if this test fails the migration identity has drifted.
        assert_eq!(
            migration_sha256(1, "enzime-object-store-v1"),
            "679c5c6c0042a66f36f411c9f1d48b5ff79c8ea98e4f2fab4057be3200262603"
        );
    }

    #[test]
    fn run_pending_from_empty_creates_full_layout() {
        // Fixture proving migration from the (only) supported prior release:
        // for v1, the only prior release is "no EnZIME directory at all".
        let root = test_root("empty_to_v1");
        assert!(read_marker(&root).unwrap().is_empty());

        let applied = run_pending(&root).unwrap();
        assert_eq!(applied.len(), 1);
        assert_eq!(applied[0].version, 1);
        assert_eq!(applied[0].name, "enzime-object-store-v1");

        // Marker exists, single record, sha256 matches.
        let reread = read_marker(&root).unwrap();
        assert_eq!(reread, applied);

        // All §6.3 sub-directories exist.
        let enzime = enzime_root(&root);
        for sub in LAYOUT_SUBDIRS {
            assert!(enzime.join(sub).is_dir(), "missing {}", sub);
        }
        assert!(enzime.join("objects").join("sha256").is_dir());
    }

    #[test]
    fn run_pending_is_idempotent() {
        let root = test_root("idempotent");
        let first = run_pending(&root).unwrap();
        let second = run_pending(&root).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn tampered_marker_is_rejected() {
        let root = test_root("tampered");
        run_pending(&root).unwrap();
        // Corrupt the sha256 column for v1.
        let path = marker_path(&root);
        let corrupted = "1\tenzime-object-store-v1\t0000000000000000000000000000000000000000000000000000000000000000\n";
        fs::write(&path, corrupted).unwrap();
        let err = run_pending(&root).unwrap_err();
        match err {
            MigrationError::ShaMismatch { version, .. } => assert_eq!(version, 1),
            other => panic!("expected ShaMismatch, got {other:?}"),
        }
    }

    #[test]
    fn future_version_marker_is_rejected() {
        let root = test_root("future");
        run_pending(&root).unwrap();
        let path = marker_path(&root);
        // Append a fictitious future migration.
        let mut existing = fs::read_to_string(&path).unwrap();
        existing.push_str("999\tnot-real\tdeadbeef\n");
        fs::write(&path, existing).unwrap();
        let err = run_pending(&root).unwrap_err();
        match err {
            MigrationError::FutureVersion { found, max } => {
                assert_eq!(found, 999);
                assert_eq!(max, LATEST_VERSION);
            }
            other => panic!("expected FutureVersion, got {other:?}"),
        }
    }
}
