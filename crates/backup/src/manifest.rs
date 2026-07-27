//! Versioned backup manifest.
//!
//! The manifest is the only metadata a backup records in the §6.2 `backups`
//! row (via its SHA-256) and the only metadata a restore trusts when
//! planning a replay. It is itself JSON, serialised with `serde_json` and
//! `BTreeMap`-style stable ordering so the same payload produces the same
//! manifest SHA-256 on every platform.

use serde::{Deserialize, Serialize};

use crate::payload::UserDataSnapshot;

/// Schema version of the manifest. Bumped only on an incompatible change to
/// the manifest shape or the payload shape; restores refuse to proceed past
/// an unknown future schema with [`BackupError::FormatInvalid`].
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// Magic bytes at the head of an artifact file. "ENZB" = EnZIME Backup.
/// The reader rejects any artifact whose first four bytes are not these.
pub const ARTIFACT_MAGIC: &[u8; 4] = b"ENZB";

/// Version byte that follows `ARTIFACT_MAGIC`. Bumped only if the on-disk
/// container layout (magic / version / manifest-length / manifest /
/// payload) changes — independent of [`CURRENT_SCHEMA_VERSION`].
pub const ARTIFACT_FORMAT_VERSION: u8 = 1;

/// The serialisable manifest. Every field that must be checkable on
/// read-back is here; nothing is computed lazily on the restore path.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BackupManifest {
    /// Always [`CURRENT_SCHEMA_VERSION`] when written; rejected on read if
    /// greater than the running binary supports.
    pub schema_version: u32,
    /// ULID assigned at backup-create time and stored as the §6.2
    /// `backups.backup_id` primary key.
    pub backup_id: String,
    /// Wall-clock the backup was initiated, ms since the Unix epoch. Same
    /// value goes into the §6.2 `backups.created_at_ms` column.
    pub created_at_ms: i64,
    /// `true` iff the payload is XChaCha20-Poly1305 encrypted.
    pub encrypted: bool,
    /// §14.3 `BackupCreateRequest.includeConversations`.
    pub include_conversations: bool,
    /// §14.3 `BackupCreateRequest.includeLore`.
    pub include_lore: bool,
    /// SHA-256 of the **cleartext** payload bytes, lowercase hex. The
    /// restore code recomputes this over the decrypted bytes and refuses
    /// to commit if it differs.
    pub payload_sha256: String,
    /// Cleartext payload length, bytes. Lets a planner show progress before
    /// decryption and lets the verifier abort early if the cipher length is
    /// obviously wrong.
    pub payload_byte_length: u64,
    /// Argon2id salt, base64-encoded, when `encrypted == true`. `None` for
    /// plaintext backups.
    pub kdf_salt_b64: Option<String>,
    /// Argon2id parameters used. Stored so a future reader can reproduce
    /// the KDF even if library defaults change.
    pub kdf_params: Option<Argon2ParamsRecord>,
    /// XChaCha20-Poly1305 nonce, base64-encoded, when `encrypted == true`.
    pub cipher_nonce_b64: Option<String>,
    /// Record counts for the §14.3 `BackupRestorePlan` summary. Filled in
    /// from the snapshot before the manifest is sealed.
    pub counts: BackupCounts,
}

/// Counts surfaced as the §14.3 `BackupRestorePlan` summary numbers.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct BackupCounts {
    pub pack_bindings: u64,
    pub annotations: u64,
    pub conversations: u64,
    pub lore_memories: u64,
}

impl BackupCounts {
    pub fn from_snapshot(snapshot: &UserDataSnapshot) -> Self {
        Self {
            pack_bindings: snapshot.pack_binding_count() as u64,
            annotations: snapshot.annotation_count() as u64,
            conversations: snapshot.conversation_count() as u64,
            lore_memories: snapshot.lore_memory_count() as u64,
        }
    }
}

/// Snapshot of the Argon2id parameters used for a particular encrypted
/// backup. Recorded so a reader can rebuild the KDF even if the default
/// parameters in the binary change.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Argon2ParamsRecord {
    pub m_cost_kib: u32,
    pub t_cost: u32,
    pub p_cost: u32,
}

impl BackupManifest {
    /// Serialise to canonical JSON bytes (sorted keys, no trailing newline,
    /// no pretty-printing) so the SHA-256 stored in `backups.manifest_sha256`
    /// is reproducible across platforms.
    pub fn to_canonical_json(&self) -> Result<Vec<u8>, crate::error::BackupError> {
        // `serde_json` with `to_vec` produces non-deterministic key order
        // for `struct`s (it follows field declaration order, which IS
        // stable across builds, but the architecture's manifest_sha256 is
        // required to be reproducible across platforms — we therefore pin
        // the serialiser explicitly).
        let mut bytes = serde_json::to_vec(self)?;
        // Stable, lexicographic key order is what produces a reproducible
        // digest across `serde_json` versions. We re-serialise via a Value
        // round-trip with `preserve_order` disabled. `serde_json::Value`
        // uses BTreeMap internally when the `preserve_order` feature is
        // off (it is off in our `Cargo.toml`).
        let value: serde_json::Value = serde_json::from_slice(&bytes)?;
        bytes = serde_json::to_vec(&value)?;
        Ok(bytes)
    }

    /// SHA-256 of the canonical JSON serialisation, lowercase hex. This is
    /// the value that goes in the §6.2 `backups.manifest_sha256` column
    /// (which is CHECK length 64).
    pub fn manifest_sha256_hex(&self) -> Result<String, crate::error::BackupError> {
        use sha2::{Digest, Sha256};
        let bytes = self.to_canonical_json()?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let digest = hasher.finalize();
        Ok(hex::encode(digest))
    }

    /// Validate the manifest against the architecture's invariants before
    /// we trust any byte of the payload. Returns the parsed manifest or a
    /// [`crate::error::BackupError::FormatInvalid`].
    pub fn validate(&self) -> Result<(), crate::error::BackupError> {
        if self.schema_version > CURRENT_SCHEMA_VERSION {
            return Err(crate::error::BackupError::FormatInvalid(format!(
                "manifest schema_version {} is newer than the running binary supports ({})",
                self.schema_version, CURRENT_SCHEMA_VERSION
            )));
        }
        if self.backup_id.trim().is_empty() {
            return Err(crate::error::BackupError::FormatInvalid(
                "manifest backup_id is empty".into(),
            ));
        }
        if self.payload_sha256.len() != 64
            || !self
                .payload_sha256
                .bytes()
                .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
        {
            return Err(crate::error::BackupError::FormatInvalid(format!(
                "payload_sha256 is not 64 lowercase hex characters: '{}'",
                self.payload_sha256
            )));
        }
        if self.encrypted {
            if self.kdf_salt_b64.is_none()
                || self.cipher_nonce_b64.is_none()
                || self.kdf_params.is_none()
            {
                return Err(crate::error::BackupError::FormatInvalid(
                    "encrypted manifest missing kdf_salt_b64 / cipher_nonce_b64 / kdf_params"
                        .into(),
                ));
            }
        } else if self.kdf_salt_b64.is_some()
            || self.cipher_nonce_b64.is_some()
            || self.kdf_params.is_some()
        {
            return Err(crate::error::BackupError::FormatInvalid(
                "plaintext manifest must not carry encryption parameters".into(),
            ));
        }
        Ok(())
    }
}
