//! The content-addressed object store (ARCHITECTURE.md §6.3).
//!
//! [`ObjectStore`] owns the §6.3 commit algorithm: a transaction-scoped
//! staging file is created with exclusive create, bytes are streamed under a
//! declared maximum size, the file is fsynced, the SHA-256 is recomputed from
//! the staged bytes, length and digest are compared against the caller's
//! declared values, the format verifier runs, the content-addressed directory
//! is created, and the staged file is atomically renamed into place. On any
//! verification failure the staged bytes are quarantined under the supplied
//! audit-event-id — they never become an active object (invariant A-07).
//!
//! Object files are immutable (§6.3): the same content-addressed path will
//! never be overwritten. A second commit of different bytes that hash to the
//! same digest is impossible by SHA-256's collision resistance; the same
//! bytes committed twice is a no-op that returns the existing object.

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use domain::identity::{ContentDigest, DigestAlgorithm};

use crate::digest::hash_file;
use crate::error::ObjectStoreError;
use crate::layout::{
    enzime_root, object_relpath, quarantine_root, staging_root, validate_path_component,
};
use crate::migration;
use crate::verifier::ObjectFormatVerifier;

/// A handle to an open object store rooted at `<storage-root>/EnZIME/`.
///
/// Construction runs the §6.4 migration runner (`migration::run_pending`) so
/// that opening a store on a fresh storage root produces the full §6.3 layout
/// before any object is committed.
#[derive(Debug, Clone)]
pub struct ObjectStore {
    storage_root: PathBuf,
}

/// Inputs to a single object commit.
///
/// Every commit declares the digest and length it expects the streamed bytes
/// to have. The §6.3 algorithm computes the digest from the staged bytes and
/// refuses to activate the object unless both match exactly.
pub struct CommitRequest<'a, R: Read> {
    /// The install-transaction-id (§6.3 staging path component). Validated as
    /// a single path component.
    pub transaction_id: &'a str,
    /// The audit-event-id under which the staged bytes will be quarantined if
    /// any verification step fails. Validated as a single path component.
    /// Distinct failed commits must use distinct audit-event-ids.
    pub audit_event_id_on_failure: &'a str,
    /// Hard upper bound on the number of bytes that may be streamed. The
    /// stream is interrupted the moment this limit is exceeded.
    pub max_size: u64,
    /// The exact length the caller expects the streamed bytes to total. Must
    /// be `<= max_size`.
    pub expected_length: u64,
    /// The digest the caller expects the streamed bytes to have. Must use the
    /// `Sha256` algorithm.
    pub expected_digest: &'a ContentDigest,
    /// Byte source. Read to EOF or until `max_size` is exceeded.
    pub bytes: R,
    /// Format verifier run on the staged file after the digest check.
    pub verifier: &'a dyn ObjectFormatVerifier,
}

impl<R: Read> std::fmt::Debug for CommitRequest<'_, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommitRequest")
            .field("transaction_id", &self.transaction_id)
            .field(
                "audit_event_id_on_failure",
                &self.audit_event_id_on_failure,
            )
            .field("max_size", &self.max_size)
            .field("expected_length", &self.expected_length)
            .field("expected_digest", &self.expected_digest)
            .field("verifier", &"<dyn ObjectFormatVerifier>")
            .finish()
    }
}

/// The successful result of a commit: the canonical digest of the activated
/// object, its byte length, and its absolute path inside the store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommittedObject {
    pub digest: ContentDigest,
    pub length: u64,
    pub path: PathBuf,
}

impl ObjectStore {
    /// Open (or initialise) the object store at `<storage_root>/EnZIME/`.
    ///
    /// `storage_root` must be an absolute path. If the directory tree does
    /// not yet exist, the §6.4 migration runner creates it. If it already
    /// exists, the migration runner brings it up to the latest known layout
    /// version and refuses to proceed if the on-disk marker records a future
    /// version or shows signs of tampering.
    pub fn open(storage_root: &Path) -> Result<Self, ObjectStoreError> {
        if !storage_root.is_absolute() {
            return Err(ObjectStoreError::RelativeRoot(
                storage_root.display().to_string(),
            ));
        }
        if storage_root.exists() && !storage_root.is_dir() {
            return Err(ObjectStoreError::NotADirectory(storage_root.to_path_buf()));
        }
        fs::create_dir_all(storage_root)?;
        migration::run_pending(storage_root)?;
        Ok(ObjectStore {
            storage_root: storage_root.to_path_buf(),
        })
    }

    /// Absolute path of the EnZIME directory under the storage root.
    pub fn enzime_root(&self) -> PathBuf {
        enzime_root(&self.storage_root)
    }

    /// Absolute path of the content-addressed object for `digest`, regardless
    /// of whether it currently exists.
    pub fn object_path(&self, digest: &ContentDigest) -> Result<PathBuf, ObjectStoreError> {
        require_sha256(digest)?;
        let rel = object_relpath(digest.hex())?;
        Ok(self.enzime_root().join(rel))
    }

    /// `true` iff an object with `digest` currently exists in the store.
    pub fn contains(&self, digest: &ContentDigest) -> Result<bool, ObjectStoreError> {
        Ok(self.object_path(digest)?.is_file())
    }

    /// Open an existing object for reading.
    pub fn read(&self, digest: &ContentDigest) -> Result<File, ObjectStoreError> {
        let path = self.object_path(digest)?;
        if !path.is_file() {
            return Err(ObjectStoreError::NotFound(digest.hex().to_string()));
        }
        // Open with read-only access. Object files are immutable (§6.3), so
        // callers should never need write access; we do not grant it.
        let file = OpenOptions::new().read(true).open(&path)?;
        Ok(file)
    }

    /// Execute the §6.3 11-step object commit.
    ///
    /// On success the staging directory for `transaction_id` is removed
    /// (step 11) and the activated object is returned. On any verification
    /// failure the staging directory is moved to
    /// `quarantine/<audit-event-id-on-failure>/` and the original error is
    /// returned — the bytes are preserved for audit and never become an
    /// active object (invariant A-07).
    ///
    /// Steps 9 ("commit metadata transaction") and 10 ("atomically update the
    /// active pointer") of the §6.3 algorithm belong to the application layer
    /// that owns the SQLite metadata database and the per-pack active-version
    /// pointer; they are intentionally not part of this primitive. This method
    /// performs steps 1–8 and 11.
    pub fn commit<R: Read>(
        &self,
        req: CommitRequest<'_, R>,
    ) -> Result<CommittedObject, ObjectStoreError> {
        require_sha256(req.expected_digest)?;
        validate_path_component(req.transaction_id)?;
        validate_path_component(req.audit_event_id_on_failure)?;
        if req.expected_length > req.max_size {
            return Err(ObjectStoreError::ExceededMaxSize {
                streamed: req.expected_length,
                max: req.max_size,
            });
        }

        let staging_dir = staging_root(&self.enzime_root()).join(req.transaction_id);
        let staging_file = staging_dir.join(req.expected_digest.hex());

        // Step 1: create the transaction-scoped staging file with exclusive
        // create. The staging directory itself is created first (also with
        // exclusive semantics: create_dir errors if it already exists).
        fs::create_dir_all(staging_root(&self.enzime_root()))?;
        match fs::create_dir(&staging_dir) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                return Err(ObjectStoreError::StagingExists(
                    req.transaction_id.to_string(),
                ));
            }
            Err(e) => return Err(e.into()),
        }
        let file = match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&staging_file)
        {
            Ok(f) => f,
            Err(e) => {
                // Best-effort cleanup of the empty staging dir before
                // surfacing the error; the staging file was never written so
                // there is nothing to quarantine.
                let _ = fs::remove_dir_all(&staging_dir);
                return Err(e.into());
            }
        };

        // Step 2: stream bytes while enforcing declared maximum size. We
        // write through a buffered writer so the disk receives largish
        // writes, then drop the writer before step 3.
        let _streamed = stream_to(req.bytes, file, req.max_size, &staging_dir, req.audit_event_id_on_failure)?;

        // Step 3: flush file content (fsync). The file is already closed by
        // stream_to's sync_all+drop; re-open is unnecessary because sync_all
        // was called inside stream_to. We re-open only at step 4 to compute
        // the digest from disk.

        // Step 4: calculate SHA-256 from staged bytes (re-read from disk).
        let (computed_hex, computed_len) =
            with_quarantine_on_error(
                &staging_dir,
                req.audit_event_id_on_failure,
                || hash_file(&staging_file),
            )?;

        // Step 5: compare length and digest. Both must match exactly.
        if computed_len != req.expected_length {
            return Err(quarantine_and_fail(
                staging_dir,
                req.audit_event_id_on_failure,
                ObjectStoreError::LengthMismatch {
                    staged: computed_len,
                    expected: req.expected_length,
                },
            ));
        }
        if computed_hex != req.expected_digest.hex() {
            return Err(quarantine_and_fail(
                staging_dir,
                req.audit_event_id_on_failure,
                ObjectStoreError::DigestMismatch {
                    computed: computed_hex,
                    expected: req.expected_digest.hex().to_string(),
                },
            ));
        }

        // Step 6: run format verifier. A rejection quarantines the bytes.
        if let Err(rejection) = (req.verifier).verify(&staging_file) {
            return Err(quarantine_and_fail(
                staging_dir,
                req.audit_event_id_on_failure,
                ObjectStoreError::FormatRejected(rejection.to_string()),
            ));
        }

        // Step 7: create the content-addressed object directory (the shard).
        let object_path = self.object_path(req.expected_digest)?;
        if let Some(parent) = object_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Step 8: atomically rename into the object store. POSIX rename is
        // atomic on the same filesystem; the staging area and the object
        // area are siblings under the same EnZIME root, so this is the same
        // filesystem by construction.
        if object_path.exists() {
            // Content-addressed path already exists. The bytes have already
            // been verified to match this digest, so by SHA-256 collision
            // resistance the existing object has identical content. Remove
            // the now-redundant staging file and return the existing object.
            // We do NOT overwrite — object files are immutable (§6.3).
            fs::remove_file(&staging_file)?;
            fs::remove_dir_all(&staging_dir)?;
            return Ok(CommittedObject {
                digest: req.expected_digest.clone(),
                length: computed_len,
                path: object_path,
            });
        }
        fs::rename(&staging_file, &object_path)?;

        // Step 11: remove the now-empty staging directory.
        fs::remove_dir_all(&staging_dir)?;

        Ok(CommittedObject {
            digest: req.expected_digest.clone(),
            length: computed_len,
            path: object_path,
        })
    }

    /// Move an entire staging directory (with its staged file inside) into
    /// `quarantine/<audit_event_id>/`. Returns the quarantine path.
    ///
    /// Public so that callers performing multi-step transactions can
    /// quarantine a failed batch even if the failure happened outside
    /// [`ObjectStore::commit`].
    pub fn quarantine(
        &self,
        audit_event_id: &str,
        staging_path: &Path,
    ) -> Result<PathBuf, ObjectStoreError> {
        validate_path_component(audit_event_id)?;
        let q = quarantine_root(&self.enzime_root()).join(audit_event_id);
        fs::create_dir_all(quarantine_root(&self.enzime_root()))?;
        match fs::rename(staging_path, &q) {
            Ok(()) => Ok(q),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                Err(ObjectStoreError::QuarantineExists(audit_event_id.to_string()))
            }
            Err(e) => Err(e.into()),
        }
    }
}

fn require_sha256(digest: &ContentDigest) -> Result<(), ObjectStoreError> {
    if digest.algorithm() != DigestAlgorithm::Sha256 {
        return Err(ObjectStoreError::MalformedDigestHex(format!(
            "non-sha256 digest algorithm: {:?}",
            digest.algorithm()
        )));
    }
    Ok(())
}

/// Drive the caller's reader into the staging file, enforcing the declared
/// maximum size. Returns the number of bytes actually written.
fn stream_to<R: Read>(
    mut reader: R,
    mut file: File,
    max_size: u64,
    staging_dir: &Path,
    audit_event_id_on_failure: &str,
) -> Result<u64, ObjectStoreError> {
    let mut written: u64 = 0;
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) => {
                // I/O failure while streaming is treated as a verification
                // failure for safety: the partial bytes are quarantined so
                // that nothing unverified can leak out via the staging area.
                return Err(quarantine_and_fail_io(
                    staging_dir.to_path_buf(),
                    audit_event_id_on_failure.to_string(),
                    ObjectStoreError::Io(e),
                ));
            }
        };
        let new_total = written.checked_add(n as u64).expect("byte count overflow");
        if new_total > max_size {
            // Exceeded the declared maximum: quarantine partial bytes and
            // surface the size violation.
            return Err(quarantine_and_fail_io(
                staging_dir.to_path_buf(),
                audit_event_id_on_failure.to_string(),
                ObjectStoreError::ExceededMaxSize {
                    streamed: new_total,
                    max: max_size,
                },
            ));
        }
        if let Err(e) = file.write_all(&buf[..n]) {
            return Err(quarantine_and_fail_io(
                staging_dir.to_path_buf(),
                audit_event_id_on_failure.to_string(),
                ObjectStoreError::Io(e),
            ));
        }
        written = new_total;
    }
    // §6.3 step 3: flush file content. sync_all flushes the kernel page
    // cache to disk so that the digest computed in step 4 reads back bytes
    // that are actually durable.
    if let Err(e) = file.sync_all() {
        return Err(quarantine_and_fail_io(
            staging_dir.to_path_buf(),
            audit_event_id_on_failure.to_string(),
            ObjectStoreError::Io(e),
        ));
    }
    // Drop `file` to close before step 4 re-opens for reading.
    drop(file);
    Ok(written)
}

/// Run `f`; if it errors, move the staging directory to quarantine and
/// surface the original error wrapped in the appropriate `ObjectStoreError`.
fn with_quarantine_on_error<T, F>(
    staging_dir: &Path,
    audit_event_id: &str,
    f: F,
) -> Result<T, ObjectStoreError>
where
    F: FnOnce() -> Result<T, ObjectStoreError>,
{
    match f() {
        Ok(v) => Ok(v),
        Err(e) => Err(quarantine_and_fail(
            staging_dir.to_path_buf(),
            audit_event_id,
            e,
        )),
    }
}

/// Move `staging_dir` into `quarantine/<audit_event_id>/` and return `err`.
fn quarantine_and_fail(
    staging_dir: PathBuf,
    audit_event_id: &str,
    err: ObjectStoreError,
) -> ObjectStoreError {
    quarantine_and_fail_io(staging_dir, audit_event_id.to_string(), err)
}

fn quarantine_and_fail_io(
    staging_dir: PathBuf,
    audit_event_id: String,
    err: ObjectStoreError,
) -> ObjectStoreError {
    if !staging_dir.exists() {
        return err;
    }
    // Compute the quarantine root from the staging dir's parent's parent:
    //   <root>/EnZIME/staging/<tx>  ->  <root>/EnZIME/quarantine/<audit>
    // Walk up two levels to reach the EnZIME root.
    let enzime_root = match staging_dir
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
    {
        Some(p) => p,
        None => return err,
    };
    let q_root = quarantine_root(&enzime_root);
    if let Err(create_err) = fs::create_dir_all(&q_root) {
        // If we cannot create the quarantine dir we still surface the
        // original verification error; the create error is logged into the
        // chain only via the original err's Display. We do not mask the
        // primary failure with the secondary one.
        let _ = create_err;
        return err;
    }
    let q = q_root.join(&audit_event_id);
    // If rename fails (e.g. audit_event_id already used), fall back to a
    // unique suffix so the bytes are at least preserved somewhere under
    // quarantine rather than being lost.
    if fs::rename(&staging_dir, &q).is_err() {
        let mut i = 0u32;
        loop {
            let candidate = q_root.join(format!("{audit_event_id}-{i}"));
            if fs::rename(&staging_dir, &candidate).is_ok() {
                break;
            }
            i += 1;
            if i > 1_000_000 {
                break;
            }
        }
    }
    err
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::verifier::{AcceptRawBytes, MagicPrefixVerifier};
    use domain::identity::ContentDigest;

    fn test_root(subdir: &str) -> PathBuf {
        let base = std::env::var_os("CARGO_TARGET_TMPDIR")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        let root = base.join("object-store-commit").join(subdir);
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn sha256_hex(bytes: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(bytes);
        hex::encode(h.finalize())
    }

    fn digest_of(bytes: &[u8]) -> ContentDigest {
        ContentDigest::sha256_from_hex(sha256_hex(bytes)).unwrap()
    }

    #[test]
    fn commit_activates_object_with_correct_digest_and_path() {
        let root = test_root("happy");
        let store = ObjectStore::open(&root).unwrap();
        let payload = b"hello enzime object store";
        let digest = digest_of(payload);
        let req = CommitRequest {
            transaction_id: "tx-1",
            audit_event_id_on_failure: "audit-1",
            max_size: 1024,
            expected_length: payload.len() as u64,
            expected_digest: &digest,
            bytes: &payload[..],
            verifier: &AcceptRawBytes,
        };
        let committed = store.commit(req).unwrap();
        assert_eq!(committed.digest, digest);
        assert_eq!(committed.length, payload.len() as u64);
        assert!(committed.path.is_file());
        // Path shape: <root>/EnZIME/objects/sha256/<2 hex>/<62 hex>
        let rel = committed
            .path
            .strip_prefix(store.enzime_root())
            .unwrap()
            .to_string_lossy()
            .to_string();
        let expected_suffix = crate::layout::object_relpath(digest.hex()).unwrap()
            .to_string_lossy().to_string();
        assert_eq!(rel, expected_suffix);

        // Staging directory has been removed.
        let staging = store.enzime_root().join("staging").join("tx-1");
        assert!(!staging.exists());

        // Object is readable and matches.
        let mut f = store.read(&digest).unwrap();
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).unwrap();
        assert_eq!(buf, payload);

        assert!(store.contains(&digest).unwrap());
    }

    #[test]
    fn commit_same_bytes_twice_is_idempotent_no_overwrite() {
        let root = test_root("idempotent_commit");
        let store = ObjectStore::open(&root).unwrap();
        let payload = b"same payload";
        let digest = digest_of(payload);
        let mk = |tx: &'static str| CommitRequest {
            transaction_id: tx,
            audit_event_id_on_failure: "audit",
            max_size: 1024,
            expected_length: payload.len() as u64,
            expected_digest: &digest,
            bytes: &payload[..],
            verifier: &AcceptRawBytes,
        };
        let c1 = store.commit(mk("tx-1")).unwrap();
        let c2 = store.commit(mk("tx-2")).unwrap();
        assert_eq!(c1.path, c2.path);
        // Both staging directories cleaned up.
        assert!(!store.enzime_root().join("staging").join("tx-1").exists());
        assert!(!store.enzime_root().join("staging").join("tx-2").exists());
    }

    #[test]
    fn digest_mismatch_is_quarantined_not_activated() {
        let root = test_root("digest_mismatch");
        let store = ObjectStore::open(&root).unwrap();
        let payload = b"actual bytes";
        let wrong = digest_of(b"different bytes");
        let req = CommitRequest {
            transaction_id: "tx-bad",
            audit_event_id_on_failure: "audit-digest",
            max_size: 1024,
            expected_length: payload.len() as u64,
            expected_digest: &wrong,
            bytes: &payload[..],
            verifier: &AcceptRawBytes,
        };
        let err = store.commit(req).unwrap_err();
        match err {
            ObjectStoreError::DigestMismatch { .. } => {}
            other => panic!("expected DigestMismatch, got {other:?}"),
        }
        // Object was never activated.
        assert!(!store.contains(&wrong).unwrap());
        // Staging moved to quarantine/<audit>/.
        let q = store.enzime_root().join("quarantine").join("audit-digest");
        assert!(q.is_dir(), "quarantine dir missing at {}", q.display());
        // The staged bytes are preserved inside quarantine.
        let staged = q.join(wrong.hex());
        assert!(staged.is_file());
        // Staging directory for tx-bad is gone.
        assert!(!store.enzime_root().join("staging").join("tx-bad").exists());
    }

    #[test]
    fn length_mismatch_is_quarantined() {
        let root = test_root("length_mismatch");
        let store = ObjectStore::open(&root).unwrap();
        let payload = b"twelve bytes";
        let real_digest = digest_of(payload);
        let req = CommitRequest {
            transaction_id: "tx-len",
            audit_event_id_on_failure: "audit-len",
            max_size: 1024,
            expected_length: 999,
            expected_digest: &real_digest,
            bytes: &payload[..],
            verifier: &AcceptRawBytes,
        };
        match store.commit(req).unwrap_err() {
            ObjectStoreError::LengthMismatch { staged, expected } => {
                assert_eq!(staged, payload.len() as u64);
                assert_eq!(expected, 999);
            }
            other => panic!("expected LengthMismatch, got {other:?}"),
        }
        let q = store.enzime_root().join("quarantine").join("audit-len");
        assert!(q.is_dir());
        assert!(!store.contains(&real_digest).unwrap());
    }

    #[test]
    fn max_size_exceeded_is_quarantined() {
        let root = test_root("max_size");
        let store = ObjectStore::open(&root).unwrap();
        // 10-byte payload, max_size of 4 — streaming will exceed.
        let payload = b"0123456789";
        // expected_digest is irrelevant (we won't get there); use a real one
        // of an empty string just to satisfy require_sha256.
        let dummy = digest_of(b"");
        let req = CommitRequest {
            transaction_id: "tx-max",
            audit_event_id_on_failure: "audit-max",
            max_size: 4,
            expected_length: 4,
            expected_digest: &dummy,
            bytes: &payload[..],
            verifier: &AcceptRawBytes,
        };
        match store.commit(req).unwrap_err() {
            ObjectStoreError::ExceededMaxSize { streamed, max } => {
                assert!(streamed > max);
                assert_eq!(max, 4);
            }
            other => panic!("expected ExceededMaxSize, got {other:?}"),
        }
        let q = store.enzime_root().join("quarantine").join("audit-max");
        assert!(q.is_dir());
    }

    #[test]
    fn format_verifier_rejection_is_quarantined() {
        let root = test_root("format_reject");
        let store = ObjectStore::open(&root).unwrap();
        // Payload starts with the wrong magic.
        let payload = [0xffu8, 0xff, 0xff, 0xff, 0xaa];
        let digest = digest_of(&payload);
        let v = MagicPrefixVerifier::new([0x5a, 0x49, 0x4d, 0x04]);
        let req = CommitRequest {
            transaction_id: "tx-fmt",
            audit_event_id_on_failure: "audit-fmt",
            max_size: 1024,
            expected_length: payload.len() as u64,
            expected_digest: &digest,
            bytes: &payload[..],
            verifier: &v,
        };
        match store.commit(req).unwrap_err() {
            ObjectStoreError::FormatRejected(msg) => {
                assert!(msg.contains("bad-magic"), "got: {msg}");
            }
            other => panic!("expected FormatRejected, got {other:?}"),
        }
        let q = store.enzime_root().join("quarantine").join("audit-fmt");
        assert!(q.is_dir());
        assert!(!store.contains(&digest).unwrap());
    }

    #[test]
    fn format_verifier_accept_leads_to_activation() {
        let root = test_root("format_accept");
        let store = ObjectStore::open(&root).unwrap();
        // Payload starts with the ZIM magic 0x5a 0x49 0x4d 0x04.
        let mut payload = vec![0x5a, 0x49, 0x4d, 0x04];
        payload.extend_from_slice(b"trailing bytes for the rest of the file");
        let digest = digest_of(&payload);
        let v = MagicPrefixVerifier::new([0x5a, 0x49, 0x4d, 0x04]);
        let req = CommitRequest {
            transaction_id: "tx-ok",
            audit_event_id_on_failure: "audit-never",
            max_size: 1024,
            expected_length: payload.len() as u64,
            expected_digest: &digest,
            bytes: &payload[..],
            verifier: &v,
        };
        let committed = store.commit(req).unwrap();
        assert!(committed.path.is_file());
        assert!(store.contains(&digest).unwrap());
    }

    #[test]
    fn duplicate_transaction_id_is_rejected_without_overwriting() {
        let root = test_root("dup_tx");
        let store = ObjectStore::open(&root).unwrap();
        let payload = b"x";
        let digest = digest_of(payload);
        // Pre-create a stale staging dir for tx-stale to simulate a prior
        // in-flight commit. commit() must refuse rather than clobber it.
        fs::create_dir_all(
            store.enzime_root().join("staging").join("tx-stale"),
        )
        .unwrap();
        let req = CommitRequest {
            transaction_id: "tx-stale",
            audit_event_id_on_failure: "audit-stale",
            max_size: 1024,
            expected_length: payload.len() as u64,
            expected_digest: &digest,
            bytes: &payload[..],
            verifier: &AcceptRawBytes,
        };
        match store.commit(req).unwrap_err() {
            ObjectStoreError::StagingExists(tx) => assert_eq!(tx, "tx-stale"),
            other => panic!("expected StagingExists, got {other:?}"),
        }
        // Existing staging dir is preserved (not clobbered).
        assert!(store.enzime_root().join("staging").join("tx-stale").exists());
    }

    #[test]
    fn path_bearing_transaction_id_is_rejected() {
        let root = test_root("bad_id");
        let store = ObjectStore::open(&root).unwrap();
        let payload = b"x";
        let digest = digest_of(payload);
        let req = CommitRequest {
            transaction_id: "../escape",
            audit_event_id_on_failure: "audit",
            max_size: 1024,
            expected_length: 1,
            expected_digest: &digest,
            bytes: &payload[..],
            verifier: &AcceptRawBytes,
        };
        match store.commit(req).unwrap_err() {
            ObjectStoreError::InvalidIdentifier(_) => {}
            other => panic!("expected InvalidIdentifier, got {other:?}"),
        }
    }

    #[test]
    fn read_missing_object_is_not_found() {
        let root = test_root("missing_read");
        let store = ObjectStore::open(&root).unwrap();
        let d = digest_of(b"never committed");
        match store.read(&d).unwrap_err() {
            ObjectStoreError::NotFound(s) => assert_eq!(s, d.hex()),
            other => panic!("expected NotFound, got {other:?}"),
        }
    }

    #[test]
    fn commit_many_objects_populates_shard_subdirs() {
        let root = test_root("shards");
        let store = ObjectStore::open(&root).unwrap();
        for i in 0u8..16 {
            let payload = vec![i; 32];
            let digest = digest_of(&payload);
            let tx_name: &'static str = match i {
                0 => "tx-0", 1 => "tx-1", 2 => "tx-2", 3 => "tx-3",
                4 => "tx-4", 5 => "tx-5", 6 => "tx-6", 7 => "tx-7",
                8 => "tx-8", 9 => "tx-9", 10 => "tx-10", 11 => "tx-11",
                12 => "tx-12", 13 => "tx-13", 14 => "tx-14", 15 => "tx-15",
                _ => unreachable!(),
            };
            let req = CommitRequest {
                transaction_id: tx_name,
                audit_event_id_on_failure: "audit-fixed",
                max_size: 1024,
                expected_length: payload.len() as u64,
                expected_digest: &digest,
                bytes: &payload[..],
                verifier: &AcceptRawBytes,
            };
            store.commit(req).unwrap();
        }
        // Objects landed under various 2-char shard directories.
        let sha_root = store.enzime_root().join("objects").join("sha256");
        let shard_count = fs::read_dir(&sha_root).unwrap().count();
        assert!(shard_count >= 1);
    }
}
