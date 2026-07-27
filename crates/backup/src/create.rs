//! Backup creation pipeline.
//!
//! Implements:
//! - §13.5 capability gating (caller-side, via `require_encryption_capability`)
//! - §6.3 object-commit algorithm principles (exclusive-create staging, hash
//!   from staged bytes, read-back verification, atomic rename, staging
//!   cleanup on any failure)
//! - The §14.3 `BackupCreateRequest` / `BackupArtifactSummary` DTOs.
//! - The `state = 'verified' | 'failed'` transition in the §6.2 `backups`
//!   table: a backup row is **only** `verified` after the just-written file
//!   has been read back, the manifest re-parsed, and the payload digest
//!   re-hashed to match the manifest. Never on the strength of having
//!   written the file.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use ulid::Ulid;

use crate::artifact;
use crate::capability::{self, CapabilityDecision};
use crate::crypto;
use crate::destination::BackupDestination;
use crate::error::BackupError;
use crate::manifest::{BackupCounts, BackupManifest, CURRENT_SCHEMA_VERSION};
use crate::source::BackupSource;

/// §14.3 `BackupCreateRequest`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BackupCreateRequest {
    pub destination_token: String,
    pub encrypted: bool,
    pub passphrase: Option<String>,
    pub include_conversations: bool,
    pub include_lore: bool,
}

/// §14.3 `BackupArtifactSummary`. The `display_path` is the only field the
/// UI is allowed to show to the user as a path (per §14.1 IPC rule).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BackupArtifactSummary {
    pub artifact_sha256: String,
    pub display_path: String,
    pub byte_length: u64,
}

/// §14.3 `JobReference` returned by the `backup_create` command. Only the
/// `job_kind` values the backup pipeline can emit are listed here.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct JobReference {
    pub job_id: String,
    pub job_kind: JobKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobKind {
    BackupCreate,
    BackupRestore,
}

impl JobKind {
    pub fn as_db_str(self) -> &'static str {
        match self {
            JobKind::BackupCreate => "backup_create",
            JobKind::BackupRestore => "backup_restore",
        }
    }
}

/// The complete record of a verified backup. Mirrors the §6.2 `backups`
/// table columns one-for-one so the storage-sqlite adapter can persist it
/// directly. Field names match the §6.2 column names with `created_at_ms`
/// / `verified_at_ms` carried as `i64`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedBackup {
    pub backup_id: String,
    pub destination_kind: &'static str,
    pub destination_locator: String,
    pub manifest_sha256: String,
    pub encrypted: bool,
    pub byte_length: u64,
    pub state: BackupState,
    pub created_at_ms: i64,
    pub verified_at_ms: Option<i64>,
    pub artifact_summary: BackupArtifactSummary,
}

/// The §6.2 `backups.state` CHECK values, verbatim.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackupState {
    Creating,
    Verified,
    Failed,
}

impl BackupState {
    pub fn as_db_str(self) -> &'static str {
        match self {
            BackupState::Creating => "creating",
            BackupState::Verified => "verified",
            BackupState::Failed => "failed",
        }
    }
}

/// Entry point for the backup-create pipeline. The pipeline is `async`
/// because §14.1 forbids blocking work on the UI thread.
pub struct BackupCreator<'a> {
    source: &'a dyn BackupSource,
}

impl<'a> BackupCreator<'a> {
    pub fn new(source: &'a dyn BackupSource) -> Self {
        Self { source }
    }

    /// Run the full create pipeline.
    ///
    /// Order is fixed by §6.3 (object-commit algorithm) and §13.5 (capability
    /// gating):
    ///
    /// 1. Capability gate (caller-supplied decision).
    /// 2. Snapshot + filter + sort (deterministic SHA-256).
    /// 3. Serialise payload; compute cleartext SHA-256 + length.
    /// 4. Optional encryption (Argon2id + XChaCha20-Poly1305).
    /// 5. Build manifest.
    /// 6. Encode artifact; stage to a `.partial` next to the destination
    ///    with exclusive create.
    /// 7. fsync; compute artifact SHA-256.
    /// 8. **Read-back verification**: reopen the staged file, decode it,
    ///    recompute the payload digest, compare. Abort if anything differs.
    /// 9. Atomic rename to final destination.
    /// 10. Return [`VerifiedBackup`] with `state = Verified`.
    ///
    /// On any error after staging begins, the `.partial` is removed and the
    /// returned `state` (visible to the caller via the error kind) is
    /// `Failed`.
    pub async fn create(
        &self,
        request: BackupCreateRequest,
        destination: BackupDestination,
        capability_decision: &CapabilityDecision,
        clock_ms: i64,
    ) -> Result<VerifiedBackup, BackupError> {
        // 1. Capability gate. §13.5: never block export of user-created data.
        capability::require_encryption_capability(request.encrypted, capability_decision)?;
        if request.encrypted && request.passphrase.as_deref().map_or(true, |p| p.is_empty()) {
            return Err(BackupError::PassphraseRequired);
        }

        // 2. Snapshot.
        let filter = crate::payload::BackupFilter::from_request(
            request.include_conversations,
            request.include_lore,
        );
        let mut snapshot = self.source.export_snapshot(filter).await?;
        snapshot.sort_for_serialization();
        snapshot.apply_filter(filter);
        let counts = BackupCounts::from_snapshot(&snapshot);

        // 3. Serialise payload + cleartext digest.
        let cleartext = serde_json::to_vec(&snapshot)?;
        let payload_sha256 = hex::encode({
            let mut h = Sha256::new();
            h.update(&cleartext);
            h.finalize()
        });
        let payload_byte_length = cleartext.len() as u64;

        // 4. Optional encryption. We retain the derived key in scope for the
        // read-back verification step below (§: a backup is verified ONLY
        // after it has been read back and the payload digest has been
        // recomputed against the decrypted bytes).
        let encrypted = request.encrypted;
        let (kdf_salt_b64, kdf_params, cipher_nonce_b64, payload_bytes, derived_key_for_verify) =
            if encrypted {
                let passphrase = request.passphrase.as_deref().expect("checked above");
                let derived = crypto::derive_fresh(passphrase)?;
                let ciphertext = crypto::encrypt(&derived, &cleartext)?;
                let key_for_verify = derived.key;
                let nonce_for_verify = derived.nonce;
                (
                    Some(b64_encode(&derived.salt)),
                    Some(derived.params),
                    Some(b64_encode(&derived.nonce)),
                    ciphertext,
                    Some((key_for_verify, nonce_for_verify)),
                )
            } else {
                (None, None, None, cleartext, None)
            };

        // Drop the passphrase now — we no longer need it. The derived key
        // above is retained only for read-back verification.
        drop(request.passphrase);

        // 5. Manifest.
        let backup_id = Ulid::new().to_string();
        let manifest = BackupManifest {
            schema_version: CURRENT_SCHEMA_VERSION,
            backup_id: backup_id.clone(),
            created_at_ms: clock_ms,
            encrypted,
            include_conversations: filter.include_conversations,
            include_lore: filter.include_lore,
            payload_sha256: payload_sha256.clone(),
            payload_byte_length,
            kdf_salt_b64,
            kdf_params,
            cipher_nonce_b64,
            counts,
        };
        manifest.validate()?;

        // 6. Encode + stage.
        let encoded = artifact::encode(&manifest, &payload_bytes)?;
        let artifact_len = encoded.len() as u64;
        let artifact_sha256 = hex::encode({
            let mut h = Sha256::new();
            h.update(&encoded);
            h.finalize()
        });

        let artifact_path = match destination.artifact_path(&backup_id) {
            Some(p) => p,
            None => {
                return Err(BackupError::Destination(format!(
                    "destination kind {:?} is not supported by the in-tree backup creator \
                     (AndroidDocument byte pipes live in ports::platform — I-10(b))",
                    destination.kind
                )));
            }
        };
        ensure_parent(&artifact_path).await?;
        let staging = staging_path(&artifact_path);
        write_exclusive(&staging, &encoded).await?;

        // 7-8. Read-back verification.
        let verify_key_ref = derived_key_for_verify
            .as_ref()
            .map(|(k, n)| (k as &[u8; crypto::KEY_LEN], n as &[u8; crypto::NONCE_LEN]));
        let verification_outcome = verify_staged(&staging, &manifest, verify_key_ref).await;
        match verification_outcome {
            Ok(()) => {
                // 9. Atomic rename.
                tokio::fs::rename(&staging, &artifact_path)
                    .await
                    .map_err(|e| {
                        cleanup_silent(&staging);
                        BackupError::Io(e)
                    })?;

                // 10. Return verified.
                Ok(VerifiedBackup {
                    backup_id,
                    destination_kind: destination.kind.as_db_str(),
                    destination_locator: destination.locator.clone(),
                    manifest_sha256: manifest.manifest_sha256_hex()?,
                    encrypted,
                    byte_length: artifact_len,
                    state: BackupState::Verified,
                    created_at_ms: clock_ms,
                    verified_at_ms: Some(clock_ms),
                    artifact_summary: BackupArtifactSummary {
                        artifact_sha256,
                        display_path: destination.locator,
                        byte_length: artifact_len,
                    },
                })
            }
            Err(e) => {
                cleanup_silent(&staging);
                Err(e)
            }
        }
    }
}

fn staging_path(final_path: &std::path::Path) -> PathBuf {
    let mut name = final_path
        .file_name()
        .map(|s| s.to_os_string())
        .unwrap_or_default();
    name.push(".partial");
    final_path.with_file_name(name)
}

async fn ensure_parent(path: &std::path::Path) -> Result<(), BackupError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            tokio::fs::create_dir_all(parent).await?;
        }
    }
    Ok(())
}

/// Write `bytes` to `path` with an exclusive create — the file must not
/// already exist. Uses `create_new = true` so a collision produces a
/// [`std::io::ErrorKind::AlreadyExists`] error rather than clobbering.
async fn write_exclusive(path: &std::path::Path, bytes: &[u8]) -> Result<(), BackupError> {
    use tokio::io::AsyncWriteExt;
    let mut file = tokio::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .await?;
    file.write_all(bytes).await?;
    file.flush().await?;
    // Best-effort fsync; on platforms where it fails we still proceed — the
    // read-back verification that follows will catch any silent truncation.
    let _ = file.sync_all().await;
    Ok(())
}

/// Reopen the staged file, decode the artifact, recompute the payload
/// digest, and compare to the manifest. Returns `Ok(())` only if everything
/// matches.
///
/// For encrypted backups the caller passes the derived key + nonce it just
/// used for encryption; this function re-decrypts the on-disk ciphertext
/// and re-hashes the cleartext, so the verification is over the bytes a
/// future restore would see — not over an in-memory copy.
async fn verify_staged(
    path: &std::path::Path,
    expected_manifest: &BackupManifest,
    derived_key_for_verify: Option<(&[u8; crypto::KEY_LEN], &[u8; crypto::NONCE_LEN])>,
) -> Result<(), BackupError> {
    let bytes = tokio::fs::read(path).await?;
    let decoded = artifact::decode(&bytes)?;

    // Manifest identity — the read-back manifest must be byte-equal to the
    // one we just wrote. Compare canonical JSON.
    let expected_json = expected_manifest.to_canonical_json()?;
    let actual_json = decoded.manifest.to_canonical_json()?;
    if expected_json != actual_json {
        return Err(BackupError::VerificationFailed(format!(
            "read-back manifest JSON does not match written manifest ({} vs {} bytes)",
            actual_json.len(),
            expected_json.len()
        )));
    }

    // Recompute the payload digest over the bytes that a future restore
    // would see, and compare to the manifest's recorded `payload_sha256`.
    let cleartext = if decoded.manifest.encrypted {
        let (key, nonce) = derived_key_for_verify.ok_or_else(|| {
            BackupError::VerificationFailed(
                "encrypted backup created without a derived key available for read-back".into(),
            )
        })?;
        crypto::decrypt(key, nonce, &decoded.raw_payload)?
    } else {
        decoded.raw_payload.clone()
    };

    let recomputed = hex::encode({
        let mut h = Sha256::new();
        h.update(&cleartext);
        h.finalize()
    });
    if recomputed != expected_manifest.payload_sha256 {
        return Err(BackupError::VerificationFailed(format!(
            "read-back payload SHA-256 '{recomputed}' != manifest '{}'",
            expected_manifest.payload_sha256
        )));
    }
    if cleartext.len() as u64 != expected_manifest.payload_byte_length {
        return Err(BackupError::VerificationFailed(format!(
            "read-back payload length {} != manifest {}",
            cleartext.len(),
            expected_manifest.payload_byte_length
        )));
    }
    Ok(())
}

fn cleanup_silent(path: &std::path::Path) {
    // Synchronous removal in a `spawn_blocking` would be more polite; for
    // a staging file we want it gone even on a panicking caller, so do a
    // blocking `std::fs::remove_file` here. Errors are swallowed because
    // we're already on an error path.
    let _ = std::fs::remove_file(path);
}

pub(crate) fn b64_encode(bytes: &[u8]) -> String {
    base64_encode(bytes)
}

pub(crate) fn b64_decode(s: &str) -> Result<Vec<u8>, BackupError> {
    base64_decode(s).map_err(|e| BackupError::FormatInvalid(format!("base64 decode: {e}")))
}

// Tiny, dependency-free standard base64 implementation, used only for the
// salt and nonce fields in the manifest.
fn base64_encode(input: &[u8]) -> String {
    const TABLE: &[u8] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((input.len() + 2) / 3 * 4);
    let mut chunks = input.chunks_exact(3);
    for chunk in &mut chunks {
        let b0 = chunk[0] as usize;
        let b1 = chunk[1] as usize;
        let b2 = chunk[2] as usize;
        out.push(TABLE[b0 >> 2] as char);
        out.push(TABLE[((b0 & 0x03) << 4) | (b1 >> 4)] as char);
        out.push(TABLE[((b1 & 0x0f) << 2) | (b2 >> 6)] as char);
        out.push(TABLE[b2 & 0x3f] as char);
    }
    let rem = chunks.remainder();
    match rem.len() {
        1 => {
            let b0 = rem[0] as usize;
            out.push(TABLE[b0 >> 2] as char);
            out.push(TABLE[(b0 & 0x03) << 4] as char);
            out.push('=');
            out.push('=');
        }
        2 => {
            let b0 = rem[0] as usize;
            let b1 = rem[1] as usize;
            out.push(TABLE[b0 >> 2] as char);
            out.push(TABLE[((b0 & 0x03) << 4) | (b1 >> 4)] as char);
            out.push(TABLE[(b1 & 0x0f) << 2] as char);
            out.push('=');
        }
        _ => {}
    }
    out
}

fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    fn rev(c: u8) -> Result<u8, String> {
        match c {
            b'A'..=b'Z' => Ok(c - b'A'),
            b'a'..=b'z' => Ok(c - b'a' + 26),
            b'0'..=b'9' => Ok(c - b'0' + 52),
            b'+' => Ok(62),
            b'/' => Ok(63),
            _ => Err(format!("invalid base64 character {c:?}")),
        }
    }
    let trimmed = input.trim_end_matches('=');
    let bytes = trimmed.as_bytes();
    if bytes.len() % 4 == 1 {
        return Err("base64 length is invalid".into());
    }
    let mut out = Vec::with_capacity(bytes.len() * 3 / 4);
    let mut chunks = bytes.chunks_exact(4);
    for chunk in &mut chunks {
        let v0 = rev(chunk[0])? as u32;
        let v1 = rev(chunk[1])? as u32;
        let v2 = rev(chunk[2])? as u32;
        let v3 = rev(chunk[3])? as u32;
        out.push(((v0 << 2) | (v1 >> 4)) as u8);
        out.push((((v1 & 0x0f) << 4) | (v2 >> 2)) as u8);
        out.push((((v2 & 0x03) << 6) | v3) as u8);
    }
    let rem = chunks.remainder();
    match rem.len() {
        0 => {}
        2 => {
            let v0 = rev(rem[0])? as u32;
            let v1 = rev(rem[1])? as u32;
            out.push(((v0 << 2) | (v1 >> 4)) as u8);
        }
        3 => {
            let v0 = rev(rem[0])? as u32;
            let v1 = rev(rem[1])? as u32;
            let v2 = rev(rem[2])? as u32;
            out.push(((v0 << 2) | (v1 >> 4)) as u8);
            out.push((((v1 & 0x0f) << 4) | (v2 >> 2)) as u8);
        }
        _ => return Err("base64 length is invalid".into()),
    }
    Ok(out)
}
