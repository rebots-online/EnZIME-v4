//! Backup restore pipeline.
//!
//! §13.5 contract enforced here:
//!
//! - "Denied paid actions never destroy or hide existing user data." A
//!   failed restore leaves existing data completely unchanged. This is
//!   achieved by **planning before touching the live store**: the
//!   `backup_restore_plan` command is a pure read; only
//!   `backup_restore_commit` writes, and only via the atomic
//!   [`BackupSource::import_snapshot`] contract.
//!
//! - "Never block export of user-created data." A restore never refuses to
//!   read user data out of a backup. The only gates are integrity (SHA-256
//!   mismatch → abort) and the supplied passphrase (wrong → abort with no
//!   state change).

use serde::{Deserialize, Serialize};

use crate::artifact;
use crate::create::{b64_decode, JobReference};
use crate::crypto;
use crate::destination::BackupDestination;
use crate::error::BackupError;
use crate::manifest::BackupManifest;
use crate::payload::UserDataSnapshot;
use crate::source::{BackupSource, ConflictPolicy, RestoreCounts};

/// §14.3 `BackupRestorePlanRequest`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BackupRestorePlanRequest {
    pub source_token: String,
    pub passphrase: Option<String>,
}

/// §14.3 `BackupRestorePlan`. The numeric fields are `usize` because they
/// are immediately derived from in-memory counts; serialisation preserves
/// their JSON numeric form.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BackupRestorePlan {
    pub backup_id: String,
    pub created_at_ms: i64,
    pub encrypted: bool,
    pub schema_version: u32,
    pub pack_bindings: u64,
    pub annotations: u64,
    pub conversations: u64,
    pub lore_memories: u64,
    pub conflicts: Vec<String>,
}

/// §14.3 `BackupRestoreCommitRequest`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BackupRestoreCommitRequest {
    pub backup_id: String,
    pub conflict_policy: String,
}

/// The restorer.
pub struct BackupRestorer<'a> {
    source: &'a dyn BackupSource,
}

impl<'a> BackupRestorer<'a> {
    pub fn new(source: &'a dyn BackupSource) -> Self {
        Self { source }
    }

    /// Decode + validate a backup artifact and surface a plan summary,
    /// WITHOUT writing to the live store. The conflict list is computed
    /// against the live data via [`BackupSource::detect_conflicts`].
    pub async fn plan(
        &self,
        destination: &BackupDestination,
        request: &BackupRestorePlanRequest,
    ) -> Result<(BackupRestorePlan, BackupManifest, UserDataSnapshot), BackupError> {
        let bytes = read_artifact(destination, &request.source_token).await?;
        let decoded = artifact::decode(&bytes)?;
        let manifest = decoded.manifest.clone();

        if manifest.encrypted && request.passphrase.as_deref().map_or(true, |p| p.is_empty()) {
            return Err(BackupError::PassphraseRequired);
        }

        // Decrypt (if needed) and verify payload SHA-256 against the
        // manifest. This is the gate that §13.5 implicitly requires — a
        // tampered or wrong-passphrase backup is rejected before any live
        // data is touched.
        let cleartext = if manifest.encrypted {
            let salt = b64_decode(manifest.kdf_salt_b64.as_deref().ok_or_else(|| {
                BackupError::FormatInvalid("encrypted manifest missing kdf_salt_b64".into())
            })?)?;
            let nonce_bytes = b64_decode(manifest.cipher_nonce_b64.as_deref().ok_or_else(|| {
                BackupError::FormatInvalid("encrypted manifest missing cipher_nonce_b64".into())
            })?)?;
            if nonce_bytes.len() != crypto::NONCE_LEN {
                return Err(BackupError::FormatInvalid(format!(
                    "cipher nonce is {} bytes, expected {}",
                    nonce_bytes.len(),
                    crypto::NONCE_LEN
                )));
            }
            let mut nonce = [0u8; crypto::NONCE_LEN];
            nonce.copy_from_slice(&nonce_bytes);
            let params = manifest.kdf_params.ok_or_else(|| {
                BackupError::FormatInvalid("encrypted manifest missing kdf_params".into())
            })?;
            let passphrase = request.passphrase.as_deref().expect("checked above");
            let key = crypto::derive_from_record(passphrase, &salt, &params)?;
            crypto::decrypt(&key, &nonce, &decoded.raw_payload)?
        } else {
            decoded.raw_payload.clone()
        };

        let snapshot: UserDataSnapshot = serde_json::from_slice(&cleartext)
            .map_err(|e| {
                // A payload that does not round-trip through (de)serialisation
                // is a verification failure — the bytes that came back from
                // the artifact are not the bytes the manifest's payload_sha256
                // was computed over.
                BackupError::VerificationFailed(format!(
                    "payload did not parse as UserDataSnapshot: {e}"
                ))
            })?;
        let recomputed = hex::encode({
            use sha2::{Digest, Sha256};
            let mut h = Sha256::new();
            h.update(&cleartext);
            h.finalize()
        });
        if recomputed != manifest.payload_sha256 {
            return Err(BackupError::VerificationFailed(format!(
                "payload SHA-256 '{recomputed}' != manifest '{}'",
                manifest.payload_sha256
            )));
        }
        if cleartext.len() as u64 != manifest.payload_byte_length {
            return Err(BackupError::VerificationFailed(format!(
                "payload length {} != manifest {}",
                cleartext.len(),
                manifest.payload_byte_length
            )));
        }

        let conflicts = self.source.detect_conflicts(&snapshot).await?;
        let conflict_strings = conflicts
            .iter()
            .map(|c| format!("{}:{}", c.entity_kind, c.primary_key))
            .collect();

        let plan = BackupRestorePlan {
            backup_id: manifest.backup_id.clone(),
            created_at_ms: manifest.created_at_ms,
            encrypted: manifest.encrypted,
            schema_version: manifest.schema_version,
            pack_bindings: manifest.counts.pack_bindings,
            annotations: manifest.counts.annotations,
            conversations: manifest.counts.conversations,
            lore_memories: manifest.counts.lore_memories,
            conflicts: conflict_strings,
        };
        Ok((plan, manifest, snapshot))
    }

    /// Replay a planned snapshot into the live store under the supplied
    /// conflict policy. All write atomicity is delegated to
    /// [`BackupSource::import_snapshot`] — by contract, any error returned
    /// here leaves the live store unchanged.
    pub async fn commit(
        &self,
        snapshot: UserDataSnapshot,
        request: &BackupRestoreCommitRequest,
    ) -> Result<(JobReference, RestoreCounts), BackupError> {
        let policy = ConflictPolicy::from_request_str(&request.conflict_policy)?;
        let counts = self.source.import_snapshot(snapshot, policy).await?;
        // The job_id is generated locally; the Tauri layer will own the
        // durable-job table mapping. We surface a fresh ULID so the UI can
        // track progress.
        let job_id = ulid::Ulid::new().to_string();
        let job = JobReference {
            job_id,
            job_kind: crate::create::JobKind::BackupRestore,
        };
        Ok((job, counts))
    }
}

/// Open the artifact at `destination` and return its bytes.
///
/// For `File` and `Directory` destinations the source_token is the
/// `displayPath`; we honour §14.1 by not inventing a second path field —
/// the destination already carries the locator.
async fn read_artifact(
    destination: &BackupDestination,
    source_token: &str,
) -> Result<Vec<u8>, BackupError> {
    let path = match destination.kind {
        crate::destination::DestinationKind::File => {
            if source_token != destination.locator {
                return Err(BackupError::Destination(
                    "source_token does not match destination locator; \
                     caller must re-resolve before calling restore"
                        .into(),
                ));
            }
            destination
                .local_path
                .clone()
                .ok_or_else(|| BackupError::Destination("file destination has no local_path".into()))?
        }
        crate::destination::DestinationKind::Directory => {
            // The source_token for a Directory destination is the
            // user-picked file inside the directory; we trust the caller's
            // resolution.
            std::path::PathBuf::from(source_token)
        }
        crate::destination::DestinationKind::AndroidDocument => {
            return Err(BackupError::Destination(
                "AndroidDocument read pipes live in ports::platform (I-10(b))".into(),
            ));
        }
    };
    let bytes = tokio::fs::read(&path)
        .await
        .map_err(|e| BackupError::Destination(format!("read {}: {e}", path.display())))?;
    Ok(bytes)
}
