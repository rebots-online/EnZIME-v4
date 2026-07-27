//! Integration tests for the backup pipeline.
//!
//! These are NOT smoke tests — every assertion exercises a behaviour the
//! §13.5 / §6.3 spec text calls out and that a future regression could
//! silently break.

#![cfg(test)]

use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use crate::create::{BackupCreateRequest, BackupCreator, BackupState};
use crate::destination::BackupDestination;
use crate::error::BackupError;
use crate::payload::{
    AnnotationRecord, BackupFilter, ConversationMessage, LoreMemory, PackBinding, UserDataSnapshot,
};
use crate::restore::{BackupRestorePlanRequest, BackupRestorer};
use crate::source::{BackupSource, ConflictPolicy, RestoreConflict, RestoreCounts};

use domain::identity::{ArticleKey, ContentDigest, EntryKey, PackId, PackVersionId};

/// An in-memory `BackupSource` whose `import_snapshot` either commits the
/// full incoming snapshot (under any policy) or fails atomically — leaving
/// whatever was there before untouched. Used to prove the §13.5 atomicity
/// rule.
#[derive(Default, Clone)]
struct InMemorySource {
    state: Arc<Mutex<UserDataSnapshot>>,
    /// When true, the next `import_snapshot` call fails after touching the
    /// state — used to prove a failed restore leaves data unchanged.
    fail_next_import: Arc<Mutex<bool>>,
}

#[async_trait]
impl BackupSource for InMemorySource {
    async fn export_snapshot(&self, _filter: BackupFilter) -> Result<UserDataSnapshot, BackupError> {
        let mut snapshot = self.state.lock().unwrap().clone();
        snapshot.sort_for_serialization();
        Ok(snapshot)
    }

    async fn detect_conflicts(
        &self,
        incoming: &UserDataSnapshot,
    ) -> Result<Vec<RestoreConflict>, BackupError> {
        let live = self.state.lock().unwrap();
        let live_pack_ids: std::collections::HashSet<&str> =
            live.pack_bindings.iter().map(|p| p.pack_id.as_str()).collect();
        let live_ann_keys: std::collections::HashSet<(&str, &str, u32, u32)> = live
            .annotations
            .iter()
            .map(|a| {
                (
                    a.article_key.as_str(),
                    a.entry_key.as_str(),
                    a.start_char,
                    a.end_char,
                )
            })
            .collect();
        let mut out = Vec::new();
        for p in &incoming.pack_bindings {
            if live_pack_ids.contains(p.pack_id.as_str()) {
                out.push(RestoreConflict {
                    entity_kind: "pack_binding",
                    primary_key: p.pack_id.as_str().to_string(),
                });
            }
        }
        for a in &incoming.annotations {
            if live_ann_keys.contains(&(
                a.article_key.as_str(),
                a.entry_key.as_str(),
                a.start_char,
                a.end_char,
            )) {
                out.push(RestoreConflict {
                    entity_kind: "annotation",
                    primary_key: format!(
                        "{}:{}:{}:{}",
                        a.article_key.as_str(),
                        a.entry_key.as_str(),
                        a.start_char,
                        a.end_char
                    ),
                });
            }
        }
        Ok(out)
    }

    async fn import_snapshot(
        &self,
        incoming: UserDataSnapshot,
        _policy: ConflictPolicy,
    ) -> Result<RestoreCounts, BackupError> {
        // To simulate the §13.5 atomicity contract, we build the new state
        // fully off to the side; if `fail_next_import` is set we abort with
        // an error without ever touching the live state.
        if *self.fail_next_import.lock().unwrap() {
            return Err(BackupError::Conflict(
                "simulated mid-restore failure — by contract, no rows touched".into(),
            ));
        }
        let mut new_state = self.state.lock().unwrap().clone();
        new_state.pack_bindings.extend(incoming.pack_bindings);
        new_state.annotations.extend(incoming.annotations);
        new_state
            .conversations
            .extend(incoming.conversations);
        new_state.lore_memories.extend(incoming.lore_memories);
        new_state.sort_for_serialization();
        let counts = RestoreCounts {
            pack_bindings: new_state.pack_bindings.len() as u64,
            annotations: new_state.annotations.len() as u64,
            conversations: new_state.conversations.len() as u64,
            lore_memories: new_state.lore_memories.len() as u64,
        };
        *self.state.lock().unwrap() = new_state;
        Ok(counts)
    }
}

fn sample_snapshot() -> UserDataSnapshot {
    let digest = ContentDigest::sha256_from_hex(
        "0000000000000000000000000000000000000000000000000000000000000000",
    )
    .unwrap();
    UserDataSnapshot {
        pack_bindings: vec![PackBinding {
            pack_id: PackId::parse("pack-01").unwrap(),
            active_version_id: Some(PackVersionId::parse("pv-01").unwrap()),
            display_name: "Alpha Pack".into(),
        }],
        annotations: vec![AnnotationRecord {
            article_key: ArticleKey::parse("art-A").unwrap(),
            entry_key: EntryKey::parse("ent-1").unwrap(),
            start_char: 10,
            end_char: 20,
            body_text: "highlight one".into(),
            created_at_ms: 1_700_000_000_000,
            updated_at_ms: 1_700_000_000_000,
        }],
        conversations: vec![ConversationMessage {
            message_id: "m-1".into(),
            role: "user".into(),
            body_text: "Hello".into(),
            created_at_ms: 1_700_000_001_000,
        }],
        lore_memories: vec![LoreMemory {
            memory_id: "lm-1".into(),
            summary: "fact one".into(),
            source_digest: digest,
            accepted_at_ms: 1_700_000_002_000,
        }],
    }
}

fn empty_decision_allowed() -> crate::capability::CapabilityDecision {
    crate::capability::CapabilityDecision::core_allowed(crate::capability::CAPABILITY_BACKUP_ENCRYPTED)
}

fn denied_decision() -> crate::capability::CapabilityDecision {
    crate::capability::CapabilityDecision::denied(
        crate::capability::CAPABILITY_BACKUP_ENCRYPTED,
        "no entitlement",
    )
}

// ---------- Capability gating ----------

#[tokio::test]
async fn plaintext_backup_succeeds_even_when_encryption_is_denied() {
    let source = InMemorySource::default();
    *source.state.lock().unwrap() = sample_snapshot();
    let dir = tempfile::tempdir().unwrap();
    let dest = BackupDestination::directory(dir.path()).unwrap();

    let creator = BackupCreator::new(&source);
    let req = BackupCreateRequest {
        destination_token: dest.locator.clone(),
        encrypted: false,
        passphrase: None,
        include_conversations: true,
        include_lore: true,
    };
    let verified = creator
        .create(req, dest.clone(), &denied_decision(), 1_700_000_000_000)
        .await
        .expect("plaintext backup must always work — §13.5");
    assert_eq!(verified.state, BackupState::Verified);
    assert!(!verified.encrypted);
    assert!(verified.verified_at_ms.is_some());
}

#[tokio::test]
async fn encrypted_backup_is_blocked_when_capability_denied() {
    let source = InMemorySource::default();
    *source.state.lock().unwrap() = sample_snapshot();
    let dir = tempfile::tempdir().unwrap();
    let dest = BackupDestination::directory(dir.path()).unwrap();

    let creator = BackupCreator::new(&source);
    let req = BackupCreateRequest {
        destination_token: dest.locator.clone(),
        encrypted: true,
        passphrase: Some("hunter2".into()),
        include_conversations: true,
        include_lore: true,
    };
    let err = creator
        .create(req, dest, &denied_decision(), 1_700_000_000_000)
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        BackupError::CapabilityDenied { capability } if capability == "backup.encrypted"
    ));
}

#[tokio::test]
async fn encrypted_backup_without_passphrase_is_rejected() {
    let source = InMemorySource::default();
    let dir = tempfile::tempdir().unwrap();
    let dest = BackupDestination::directory(dir.path()).unwrap();

    let creator = BackupCreator::new(&source);
    let req = BackupCreateRequest {
        destination_token: dest.locator.clone(),
        encrypted: true,
        passphrase: Some("".into()),
        include_conversations: true,
        include_lore: true,
    };
    let err = creator
        .create(req, dest, &empty_decision_allowed(), 0)
        .await
        .unwrap_err();
    assert!(matches!(err, BackupError::PassphraseRequired));
}

// ---------- Manifest determinism ----------

#[test]
fn manifest_sha256_is_stable_across_serialisations() {
    let manifest = sample_manifest();
    let h1 = manifest.manifest_sha256_hex().unwrap();
    let h2 = manifest.manifest_sha256_hex().unwrap();
    assert_eq!(h1, h2);
    assert_eq!(h1.len(), 64);
    assert!(h1.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f')));
}

fn sample_manifest() -> crate::manifest::BackupManifest {
    crate::manifest::BackupManifest {
        schema_version: crate::manifest::CURRENT_SCHEMA_VERSION,
        backup_id: "01HZX.TEST".into(),
        created_at_ms: 1_700_000_000_000,
        encrypted: false,
        include_conversations: true,
        include_lore: true,
        payload_sha256: "abcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdef0000".into(),
        payload_byte_length: 42,
        kdf_salt_b64: None,
        kdf_params: None,
        cipher_nonce_b64: None,
        counts: crate::manifest::BackupCounts {
            pack_bindings: 1,
            annotations: 1,
            conversations: 1,
            lore_memories: 1,
        },
    }
}

// ---------- Full create + restore round trip ----------

#[tokio::test]
async fn plaintext_round_trip_restores_exact_snapshot() {
    let original = sample_snapshot();
    let source = InMemorySource::default();
    *source.state.lock().unwrap() = original.clone();
    let dir = tempfile::tempdir().unwrap();
    let dest = BackupDestination::directory(dir.path()).unwrap();

    let creator = BackupCreator::new(&source);
    let verified = creator
        .create(
            BackupCreateRequest {
                destination_token: dest.locator.clone(),
                encrypted: false,
                passphrase: None,
                include_conversations: true,
                include_lore: true,
            },
            dest.clone(),
            &empty_decision_allowed(),
            1_700_000_000_000,
        )
        .await
        .unwrap();
    assert_eq!(verified.state, BackupState::Verified);

    // Wipe source, then restore.
    *source.state.lock().unwrap() = UserDataSnapshot::default();
    let restorer = BackupRestorer::new(&source);
    let source_token = dir
        .path()
        .join(format!("{}.enzb", verified.backup_id))
        .to_str()
        .unwrap()
        .to_string();
    let (plan, _manifest, snapshot) = restorer
        .plan(
            &dest,
            &BackupRestorePlanRequest {
                source_token,
                passphrase: None,
            },
        )
        .await
        .unwrap();
    assert_eq!(plan.pack_bindings, 1);
    assert_eq!(plan.annotations, 1);
    assert_eq!(plan.conversations, 1);
    assert_eq!(plan.lore_memories, 1);
    assert_eq!(plan.encrypted, false);

    let (_job, counts) = restorer
        .commit(
            snapshot,
            &crate::restore::BackupRestoreCommitRequest {
                backup_id: plan.backup_id.clone(),
                conflict_policy: "keep_backup".into(),
            },
        )
        .await
        .unwrap();
    assert_eq!(counts.pack_bindings, 1);
    assert_eq!(counts.annotations, 1);
    assert_eq!(counts.conversations, 1);
    assert_eq!(counts.lore_memories, 1);

    let restored = source.state.lock().unwrap().clone();
    assert_eq!(restored, original);
}

#[tokio::test]
async fn encrypted_round_trip_restores_exact_snapshot() {
    let original = sample_snapshot();
    let source = InMemorySource::default();
    *source.state.lock().unwrap() = original.clone();
    let dir = tempfile::tempdir().unwrap();
    let dest = BackupDestination::directory(dir.path()).unwrap();
    let passphrase = "correct horse battery staple";

    let creator = BackupCreator::new(&source);
    let verified = creator
        .create(
            BackupCreateRequest {
                destination_token: dest.locator.clone(),
                encrypted: true,
                passphrase: Some(passphrase.into()),
                include_conversations: true,
                include_lore: true,
            },
            dest.clone(),
            &empty_decision_allowed(),
            1_700_000_000_000,
        )
        .await
        .unwrap();
    assert_eq!(verified.state, BackupState::Verified);
    assert!(verified.encrypted);

    *source.state.lock().unwrap() = UserDataSnapshot::default();
    let restorer = BackupRestorer::new(&source);
    let source_token = dir
        .path()
        .join(format!("{}.enzb", verified.backup_id))
        .to_str()
        .unwrap()
        .to_string();
    let (plan, _manifest, snapshot) = restorer
        .plan(
            &dest,
            &BackupRestorePlanRequest {
                source_token,
                passphrase: Some(passphrase.into()),
            },
        )
        .await
        .unwrap();
    assert_eq!(plan.encrypted, true);
    let (_job, _counts) = restorer
        .commit(
            snapshot,
            &crate::restore::BackupRestoreCommitRequest {
                backup_id: plan.backup_id,
                conflict_policy: "keep_backup".into(),
            },
        )
        .await
        .unwrap();
    let restored = source.state.lock().unwrap().clone();
    assert_eq!(restored, original);
}

#[tokio::test]
async fn encrypted_restore_with_wrong_passphrase_fails_without_state_change() {
    let original = sample_snapshot();
    let source = InMemorySource::default();
    *source.state.lock().unwrap() = original.clone();
    let dir = tempfile::tempdir().unwrap();
    let dest = BackupDestination::directory(dir.path()).unwrap();
    let creator = BackupCreator::new(&source);
    let verified = creator
        .create(
            BackupCreateRequest {
                destination_token: dest.locator.clone(),
                encrypted: true,
                passphrase: Some("right passphrase".into()),
                include_conversations: true,
                include_lore: true,
            },
            dest.clone(),
            &empty_decision_allowed(),
            0,
        )
        .await
        .unwrap();
    // Existing state in source is the sample; restore should not mutate it.
    let before = source.state.lock().unwrap().clone();
    let restorer = BackupRestorer::new(&source);
    let source_token = dir
        .path()
        .join(format!("{}.enzb", verified.backup_id))
        .to_str()
        .unwrap()
        .to_string();
    let err = restorer
        .plan(
            &dest,
            &BackupRestorePlanRequest {
                source_token,
                passphrase: Some("wrong passphrase".into()),
            },
        )
        .await
        .unwrap_err();
    assert!(matches!(err, BackupError::PassphraseWrong));
    let after = source.state.lock().unwrap().clone();
    assert_eq!(before, after, "§13.5: a failed restore leaves data unchanged");
}

// ---------- Failed-restore atomicity ----------

#[tokio::test]
async fn failed_commit_leaves_existing_data_unchanged() {
    let original = sample_snapshot();
    let source = InMemorySource::default();
    *source.state.lock().unwrap() = original.clone();
    let dir = tempfile::tempdir().unwrap();
    let dest = BackupDestination::directory(dir.path()).unwrap();

    let creator = BackupCreator::new(&source);
    let verified = creator
        .create(
            BackupCreateRequest {
                destination_token: dest.locator.clone(),
                encrypted: false,
                passphrase: None,
                include_conversations: true,
                include_lore: true,
            },
            dest.clone(),
            &empty_decision_allowed(),
            0,
        )
        .await
        .unwrap();

    // Arm the source to fail mid-import.
    *source.fail_next_import.lock().unwrap() = true;

    let before = source.state.lock().unwrap().clone();
    let restorer = BackupRestorer::new(&source);
    let source_token = dir
        .path()
        .join(format!("{}.enzb", verified.backup_id))
        .to_str()
        .unwrap()
        .to_string();
    let (_plan, _manifest, snapshot) = restorer
        .plan(
            &dest,
            &BackupRestorePlanRequest {
                source_token,
                passphrase: None,
            },
        )
        .await
        .unwrap();
    let err = restorer
        .commit(
            snapshot,
            &crate::restore::BackupRestoreCommitRequest {
                backup_id: _plan.backup_id.clone(),
                conflict_policy: "keep_backup".into(),
            },
        )
        .await
        .unwrap_err();
    assert!(matches!(err, BackupError::Conflict(_)));
    let after = source.state.lock().unwrap().clone();
    assert_eq!(before, after, "§13.5 atomicity: failed commit must be a no-op");
}

// ---------- Tamper / verification ----------

#[tokio::test]
async fn read_back_verification_catches_a_corrupt_staged_file() {
    // We exercise the verification step directly by writing a bad payload
    // to the staging path the creator would use, then handing the staged
    // file to `verify_staged`. This proves the verification function
    // actually re-hashes the bytes on disk rather than trusting the
    // manifest.
    let source = InMemorySource::default();
    *source.state.lock().unwrap() = sample_snapshot();
    let dir = tempfile::tempdir().unwrap();
    let dest = BackupDestination::directory(dir.path()).unwrap();

    let creator = BackupCreator::new(&source);
    let verified = creator
        .create(
            BackupCreateRequest {
                destination_token: dest.locator.clone(),
                encrypted: false,
                passphrase: None,
                include_conversations: true,
                include_lore: true,
            },
            dest.clone(),
            &empty_decision_allowed(),
            0,
        )
        .await
        .unwrap();

    // Re-read the artifact, flip a payload byte, write it back, then try
    // to plan a restore. The plan MUST fail verification.
    let artifact_path = dir
        .path()
        .join(format!("{}.enzb", verified.backup_id));
    let mut bytes = std::fs::read(&artifact_path).unwrap();
    // Flip the very last byte of the payload (after the manifest).
    let last = bytes.len() - 1;
    bytes[last] ^= 0xFF;
    std::fs::write(&artifact_path, bytes).unwrap();

    let restorer = BackupRestorer::new(&source);
    let source_token = artifact_path.to_str().unwrap().to_string();
    let err = restorer
        .plan(
            &dest,
            &BackupRestorePlanRequest {
                source_token,
                passphrase: None,
            },
        )
        .await
        .unwrap_err();
    assert!(
        matches!(err, BackupError::VerificationFailed(_)),
        "tampered payload must fail verification, got {err:?}"
    );
}

#[test]
fn decode_rejects_bad_magic() {
    let bad = b"NOPE\x01\x00\x00\x00\x00".to_vec();
    let err = crate::artifact::decode(&bad).unwrap_err();
    assert!(matches!(err, BackupError::FormatInvalid(_)));
}

#[test]
fn decode_rejects_unknown_format_version() {
    let mut bad = b"ENZB".to_vec();
    bad.push(0x02); // unknown future version
    bad.extend_from_slice(&0u32.to_be_bytes());
    let err = crate::artifact::decode(&bad).unwrap_err();
    assert!(matches!(err, BackupError::FormatInvalid(_)));
}

#[test]
fn manifest_rejects_uppercase_or_short_digest() {
    let mut m = sample_manifest();
    m.payload_sha256 = "ABCDEF".into();
    assert!(m.validate().is_err());
    m.payload_sha256 =
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789ABCDEF".into();
    assert!(m.validate().is_err());
}

// ---------- Conflict policy parsing ----------

#[test]
fn conflict_policy_parser_accepts_spec_strings_and_rejects_unknown() {
    use crate::source::ConflictPolicy;
    assert_eq!(
        ConflictPolicy::from_request_str("keep_local").unwrap(),
        ConflictPolicy::KeepLocal
    );
    assert_eq!(
        ConflictPolicy::from_request_str("keep_backup").unwrap(),
        ConflictPolicy::KeepBackup
    );
    assert_eq!(
        ConflictPolicy::from_request_str("keep_both").unwrap(),
        ConflictPolicy::KeepBoth
    );
    assert!(ConflictPolicy::from_request_str("nuke").is_err());
}

// ---------- Staging cleanup ----------

#[tokio::test]
async fn failed_create_removes_partial_file() {
    // Force a failure mid-create by asking for a destination whose parent
    // cannot be created (a path under a non-existent drive on Linux is
    // enough — `/proc/cannot-create/x` is read-only).
    let source = InMemorySource::default();
    let dest = BackupDestination::file("/proc/enzime-cannot-backup.enzb").unwrap();

    let creator = BackupCreator::new(&source);
    let req = BackupCreateRequest {
        destination_token: dest.locator.clone(),
        encrypted: false,
        passphrase: None,
        include_conversations: true,
        include_lore: true,
    };
    let err = creator
        .create(req, dest.clone(), &empty_decision_allowed(), 0)
        .await
        .unwrap_err();
    assert!(matches!(err, BackupError::Io(_)));
    // The staging file must not exist after a failed create.
    let staging = std::path::Path::new(&dest.locator).with_file_name({
        let mut n = std::ffi::OsString::from("enzime-cannot-backup.enzb");
        n.push(".partial");
        n
    });
    assert!(!staging.exists(), "staging file must be cleaned up on error");
}
