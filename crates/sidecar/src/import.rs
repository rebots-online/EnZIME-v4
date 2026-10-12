//! Sidecar import behaviour — ARCHITECTURE.md §9.6.
//!
//! This module implements the import *policy*: given an incoming sidecar
//! artifact and the locally-known state, decide what to do with each piece.
//! The decision is purely functional — actual storage mutation is the
//! caller's job. The rules transcribe §9.6 verbatim:
//!
//!   - Import is idempotent by `artifact_sha256`.
//!   - Conflict key: `annotation_id + revision`.
//!   - identical revision bytes are a no-op;
//!   - higher imported revision creates an imported-layer revision;
//!   - divergent equal revisions create a conflict task;
//!   - imported private layers remain imported and are not merged into the
//!     local private layer automatically;
//!   - invalid signatures are quarantined;
//!   - unsigned artifacts require explicit user confirmation;
//!   - object digests are verified before object commit.
//!
//! "Identical revision bytes" is computed as canonical CBOR encoding of the
//! annotation map with the binding-required fields, so producers and
//! verifiers agree on equality regardless of source-code field order.

use std::collections::HashMap;

use ed25519_dalek::{Signature, Signer, Verifier, VerifyingKey};

use crate::cbor::{encode_to_vec, text_map, Value};use crate::codec::{
    artifact_id, encode_to_vec as encode_env_to_vec, encode_unsigned_signing_form, sha256_hex,
    validate_envelope_metadata,
};
use crate::error::SidecarError;
use crate::provider::domain_prefix;
use crate::types::{SidecarAnnotation, SidecarV1, ALGORITHM_ED25519, SIGNATURE_DOMAIN};

/// Index of sidecar artifacts already imported by the local device, used
/// only to detect duplicates via `artifact_sha256` per §9.6.
#[derive(Default)]
pub struct ImportIndex {
    seen_artifact_sha256: HashMap<String, ()>,
    /// Existing annotations on this device, keyed by `annotation_id` — the
    /// first half of the §9.6 conflict key. The value is the locally-stored
    /// canonical encoding of the annotation at its current revision.
    annotations: HashMap<String, IndexedAnnotation>,
}

#[derive(Clone)]
struct IndexedAnnotation {
    revision: u32,
    canonical_bytes: Vec<u8>,
}

impl ImportIndex {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an annotation as already imported at the given revision.
    /// Intended for callers to seed the index from local storage before
    /// running [`classify_import`].
    pub fn insert_annotation(&mut self, ann: &SidecarAnnotation) {
        let bytes = canonical_annotation_bytes(ann);
        self.annotations.insert(
            ann.annotation_id.clone(),
            IndexedAnnotation {
                revision: ann.revision,
                canonical_bytes: bytes,
            },
        );
    }

    /// Mark an `artifact_sha256` as already imported. Subsequent calls to
    /// [`classify_import`] with the same hash will return [`ImportVerdict::Duplicate`].
    pub fn mark_artifact_seen(&mut self, artifact_sha256: &str) {
        self.seen_artifact_sha256.insert(artifact_sha256.to_string(), ());
    }

    fn lookup_annotation(&self, id: &str) -> Option<&IndexedAnnotation> {
        self.annotations.get(id)
    }
}

/// Outcome of evaluating one incoming sidecar artifact against the index.
#[derive(Debug)]
pub enum ImportVerdict {
    /// Artifact was already imported (idempotent by `artifact_sha256`).
    Duplicate { artifact_sha256: String },
    /// Signature invalid, artifact_id mismatch, or embedded object digest
    /// mismatch — must NOT be applied to local state.
    Quarantined {
        artifact_id: String,
        reason: QuarantineReason,
    },
    /// Artifact carries no usable signature — caller must obtain explicit
    /// user confirmation before importing (§9.6: "unsigned artifacts
    /// require explicit user confirmation").
    RequiresConfirmation {
        artifact_id: String,
        reason: ConfirmationReason,
    },
    /// Artifact passed every gate; the caller may apply the per-annotation
    /// outcomes.
    Accepted {
        artifact_id: String,
        artifact_sha256: String,
        annotation_outcomes: Vec<AnnotationOutcome>,
        object_outcomes: Vec<ObjectOutcome>,
    },
}

#[derive(Debug)]
pub enum QuarantineReason {
    ArtifactIdMismatch { reported: String, recomputed: String },
    InvalidSignature { detail: String },
    ObjectDigestMismatch { object_sha256: String, expected: String, actual: String },
    /// `signature.key_id` is not in the trusted set. The envelope's own
    /// embedded creator key is deliberately NOT used as a fallback: any
    /// attacker can self-sign with their own key, so an unknown key must
    /// quarantine exactly like an invalid signature (§9.6).
    UnknownKey { key_id: String },
}

#[derive(Debug)]
pub enum ConfirmationReason {
    /// No signature value was present (the value field is all-zero, which
    /// the codec uses as the canonical "no signature" marker).
    Unsigned,
}

/// Per-annotation outcome per the conflict-key rules in §9.6.
#[derive(Debug, PartialEq, Eq)]
pub enum AnnotationOutcome {
    /// Existing revision has identical canonical bytes — nothing to do.
    Noop { annotation_id: String, revision: u32 },
    /// No prior entry, or incoming revision is strictly higher — create
    /// the annotation on the imported layer (§9.6: "higher imported
    /// revision creates an imported-layer revision").
    Create {
        annotation_id: String,
        revision: u32,
        supersedes: Option<u32>,
    },
    /// Incoming revision is strictly lower than the existing one — keep
    /// the local revision; the import is a no-op for this annotation.
    Stale {
        annotation_id: String,
        incoming_revision: u32,
        existing_revision: u32,
    },
    /// Equal revisions with divergent bytes — caller must create a conflict
    /// task (§9.6: "divergent equal revisions create a conflict task").
    Conflict {
        annotation_id: String,
        revision: u32,
        existing_bytes: Vec<u8>,
        incoming_bytes: Vec<u8>,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum ObjectOutcome {
    /// Embedded bytes hash to the declared `object_sha256` — ready to commit.
    Verified {
        object_sha256: String,
        byte_length: u64,
    },
    /// No embedded bytes — caller must fetch externally before commit.
    External { object_sha256: String },
}

/// Decision returned for a sidecar that already passed every gate except
/// user confirmation (used by tests to assert the unsigned path without
/// re-running the whole classifier).
#[derive(Debug)]
pub struct UnsignedDecision {
    pub artifact_id: String,
}

/// Classify an incoming sidecar artifact against `index`, verifying the
/// signature against the trusted verifying key registered for
/// `signature.key_id`. Returns the verdict the caller must act on; performs
/// no storage mutation.
pub fn classify_import(
    env: &SidecarV1,
    index: &ImportIndex,
    trusted_keys: &HashMap<String, VerifyingKey>,
) -> Result<ImportVerdict, SidecarError> {
    // 1. Compute the idempotency key (canonical-encoding SHA-256) up front.
    let canonical_env_bytes = encode_env_to_vec(env);
    let artifact_sha256 = sha256_hex(&canonical_env_bytes);
    if index.seen_artifact_sha256.contains_key(&artifact_sha256) {
        return Ok(ImportVerdict::Duplicate { artifact_sha256 });
    }

    // 2. Validate envelope metadata (algorithm, signed_domain, public key
    //    length) — produces the verifying key bytes for signature check.
    let local_pubkey = validate_envelope_metadata(env)?;
    debug_assert_eq!(local_pubkey.len(), 32);

    // 3. Artifact_id must match SHA-256(unsigned signing form).
    let recomputed_artifact_id = artifact_id(env);
    if env.artifact_id != recomputed_artifact_id {
        return Ok(ImportVerdict::Quarantined {
            artifact_id: env.artifact_id.clone(),
            reason: QuarantineReason::ArtifactIdMismatch {
                reported: env.artifact_id.clone(),
                recomputed: recomputed_artifact_id,
            },
        });
    }

    // 4. Unsigned artifacts require explicit confirmation. Detect "no
    //    signature" by an all-zero `value` field.
    let is_unsigned = env.signature.value.iter().all(|b| *b == 0);
    if is_unsigned {
        return Ok(ImportVerdict::RequiresConfirmation {
            artifact_id: env.artifact_id.clone(),
            reason: ConfirmationReason::Unsigned,
        });
    }

    // 5. The signature is checked against the public key registered for
    //    `signature.key_id`. An unknown key quarantines — trusting the
    //    envelope's own embedded creator key would let self-signed
    //    artifacts pass as authentic.
    let pubkey_bytes = match trusted_keys.get(&env.signature.key_id) {
        Some(vk) => vk.to_bytes(),
        None => {
            return Ok(ImportVerdict::Quarantined {
                artifact_id: env.artifact_id.clone(),
                reason: QuarantineReason::UnknownKey {
                    key_id: env.signature.key_id.clone(),
                },
            });
        }
    };
    let verifying_key = VerifyingKey::from_bytes(&pubkey_bytes)
        .map_err(|e| SidecarError::InvalidSignature(e.to_string()))?;
    let signing_form = encode_unsigned_signing_form(env);
    let mut signed = domain_prefix(SIGNATURE_DOMAIN);
    signed.extend_from_slice(&signing_form);
    let mut sig_arr = [0u8; 64];
    sig_arr.copy_from_slice(&env.signature.value);
    let signature = Signature::from_bytes(&sig_arr);
    if verifying_key.verify(&signed, &signature).is_err() {
        return Ok(ImportVerdict::Quarantined {
            artifact_id: env.artifact_id.clone(),
            reason: QuarantineReason::InvalidSignature {
                detail: "signature did not validate against the trusted public key"
                    .to_string(),
            },
        });
    }

    // 6. Object digests verified before object commit. Mismatch → quarantine.
    let mut object_outcomes = Vec::with_capacity(env.objects.len());
    for o in &env.objects {
        match &o.embedded_bytes {
            Some(b) => {
                if b.len() as u64 != o.byte_length {
                    return Ok(ImportVerdict::Quarantined {
                        artifact_id: env.artifact_id.clone(),
                        reason: QuarantineReason::ObjectDigestMismatch {
                            object_sha256: o.object_sha256.clone(),
                            expected: o.object_sha256.clone(),
                            actual: format!(
                                "byte_length={} but embedded_bytes.len()={}",
                                o.byte_length,
                                b.len()
                            ),
                        },
                    });
                }
                let actual = sha256_hex(b);
                if actual != o.object_sha256 {
                    return Ok(ImportVerdict::Quarantined {
                        artifact_id: env.artifact_id.clone(),
                        reason: QuarantineReason::ObjectDigestMismatch {
                            object_sha256: o.object_sha256.clone(),
                            expected: o.object_sha256.clone(),
                            actual,
                        },
                    });
                }
                object_outcomes.push(ObjectOutcome::Verified {
                    object_sha256: o.object_sha256.clone(),
                    byte_length: o.byte_length,
                });
            }
            None => object_outcomes.push(ObjectOutcome::External {
                object_sha256: o.object_sha256.clone(),
            }),
        }
    }

    // 7. Per-annotation conflict resolution using `annotation_id + revision`.
    let mut annotation_outcomes = Vec::with_capacity(env.annotations.len());
    for incoming in &env.annotations {
        let outcome = match index.lookup_annotation(&incoming.annotation_id) {
            None => AnnotationOutcome::Create {
                annotation_id: incoming.annotation_id.clone(),
                revision: incoming.revision,
                supersedes: None,
            },
            Some(existing) => {
                use std::cmp::Ordering;
                match incoming.revision.cmp(&existing.revision) {
                    Ordering::Greater => AnnotationOutcome::Create {
                        annotation_id: incoming.annotation_id.clone(),
                        revision: incoming.revision,
                        supersedes: Some(existing.revision),
                    },
                    Ordering::Less => AnnotationOutcome::Stale {
                        annotation_id: incoming.annotation_id.clone(),
                        incoming_revision: incoming.revision,
                        existing_revision: existing.revision,
                    },
                    Ordering::Equal => {
                        let incoming_bytes = canonical_annotation_bytes(incoming);
                        if incoming_bytes == existing.canonical_bytes {
                            AnnotationOutcome::Noop {
                                annotation_id: incoming.annotation_id.clone(),
                                revision: incoming.revision,
                            }
                        } else {
                            AnnotationOutcome::Conflict {
                                annotation_id: incoming.annotation_id.clone(),
                                revision: incoming.revision,
                                existing_bytes: existing.canonical_bytes.clone(),
                                incoming_bytes,
                            }
                        }
                    }
                }
            }
        };
        annotation_outcomes.push(outcome);
    }

    Ok(ImportVerdict::Accepted {
        artifact_id: env.artifact_id.clone(),
        artifact_sha256,
        annotation_outcomes,
        object_outcomes,
    })
}

/// Encode a single annotation in the same canonical form the encoder uses
/// inside the envelope. Used to decide `AnnotationOutcome` equality
/// ("identical revision bytes are a no-op").
pub fn canonical_annotation_bytes(ann: &SidecarAnnotation) -> Vec<u8> {
    let v = text_map(&[
        (
            "annotation_id",
            Value::Text(ann.annotation_id.clone()),
        ),
        (
            "annotation_type",
            Value::Text(
                match ann.annotation_type {
                    crate::types::AnnotationType::Highlight => "highlight",
                    crate::types::AnnotationType::Comment => "comment",
                    crate::types::AnnotationType::Ink => "ink",
                    crate::types::AnnotationType::Voice => "voice",
                    crate::types::AnnotationType::BookmarkNote => "bookmark_note",
                }
                .to_string(),
            ),
        ),
        ("revision", Value::Unsigned(ann.revision as u64)),
        ("selector", ann.selector.clone()),
        (
            "body_text",
            match &ann.body_text {
                Some(s) => Value::Text(s.clone()),
                None => Value::Null,
            },
        ),
        ("style", ann.style.clone()),
        (
            "notebook_id",
            match &ann.notebook_id {
                Some(s) => Value::Text(s.clone()),
                None => Value::Null,
            },
        ),
        (
            "tag_ids",
            Value::Array(ann.tag_ids.iter().map(|s| Value::Text(s.clone())).collect()),
        ),
        (
            "asset_ids",
            Value::Array(ann.asset_ids.iter().map(|s| Value::Text(s.clone())).collect()),
        ),
        (
            "created_at_ms",
            if ann.created_at_ms >= 0 {
                Value::Unsigned(ann.created_at_ms as u64)
            } else {
                Value::Negative(ann.created_at_ms)
            },
        ),
        (
            "updated_at_ms",
            if ann.updated_at_ms >= 0 {
                Value::Unsigned(ann.updated_at_ms as u64)
            } else {
                Value::Negative(ann.updated_at_ms)
            },
        ),
    ]);
    encode_to_vec(&v)
}

/// Produce a freshly signed sidecar envelope from a populated but unsigned
/// one. Sets `artifact_id` and `signature.value` canonically so the result
/// is ready to write to disk.
///
/// Pure (no I/O): used internally and by tests; production callers should
/// prefer the async `SignatureProvider::sign` path so they can integrate
/// with a real keystore.
pub fn finalize_signed_envelope(
    env: &mut SidecarV1,
    key_id: &str,
    signing_key: &ed25519_dalek::SigningKey,
) -> Result<(), SidecarError> {
    // The artifact_id is SHA-256 of the unsigned signing form, which
    // already blanks the artifact_id field — so compute it first against
    // the current (possibly-uninitialized) artifact_id, then write the
    // result back.
    let id = artifact_id(env);
    env.artifact_id = id;
    let signing_form = encode_unsigned_signing_form(env);
    let mut message = domain_prefix(SIGNATURE_DOMAIN);
    message.extend_from_slice(&signing_form);
    let sig = signing_key.sign(&message);
    env.signature = crate::types::SidecarSignature {
        algorithm: ALGORITHM_ED25519.to_string(),
        key_id: key_id.to_string(),
        signed_domain: SIGNATURE_DOMAIN.to_string(),
        value: sig.to_bytes().to_vec(),
    };
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::decode;
    use crate::types::{
        AnnotationType, DeviceRef, Layer, LayerVisibility, Notebook, ObjectRef, SidecarAnnotation,
        SidecarSignature, SourceBinding, Tag, ALGORITHM_ED25519, SCHEMA_ID,
    };
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;

    fn empty_annotation(id: &str, revision: u32) -> SidecarAnnotation {
        SidecarAnnotation {
            annotation_id: id.to_string(),
            annotation_type: AnnotationType::Highlight,
            revision,
            selector: text_map(&[("exact", Value::Text("hello".to_string()))]),
            body_text: None,
            style: text_map(&[("color_token", Value::Text("yellow".to_string()))]),
            notebook_id: None,
            tag_ids: vec![],
            asset_ids: vec![],
            created_at_ms: 0,
            updated_at_ms: 0,
        }
    }

    fn unsigned_envelope(signing_key: &SigningKey) -> SidecarV1 {
        let pubkey = signing_key.verifying_key().to_bytes();
        SidecarV1 {
            schema: SCHEMA_ID.to_string(),
            artifact_id: String::new(),
            created_at_ms: 1_700_000_000_000,
            creator: DeviceRef {
                device_id: "device-1".to_string(),
                public_key: pubkey.to_vec(),
            },
            layer: Layer {
                layer_id: "layer-1".to_string(),
                name: "Layer one".to_string(),
                visibility: LayerVisibility::Imported,
            },
            source_bindings: vec![SourceBinding {
                pack_id: "pack-1".to_string(),
                corpus_id: "corpus-1".to_string(),
                artifact_sha256: "deadbeef".to_string(),
            }],
            annotations: vec![empty_annotation("a1", 1)],
            notebooks: vec![Notebook {
                notebook_id: "n1".to_string(),
                parent_notebook_id: None,
                name: "Notes".to_string(),
                description: String::new(),
            }],
            tags: vec![Tag {
                tag_id: "t1".to_string(),
                name: "tag".to_string(),
                color_token: "blue".to_string(),
            }],
            objects: vec![ObjectRef {
                object_sha256: sha256_hex(b"payload bytes"),
                mime_type: "text/plain".to_string(),
                byte_length: b"payload bytes".len() as u64,
                embedded_bytes: Some(b"payload bytes".to_vec()),
            }],
            signature: SidecarSignature {
                algorithm: ALGORITHM_ED25519.to_string(),
                key_id: "device-1".to_string(),
                signed_domain: SIGNATURE_DOMAIN.to_string(),
                value: vec![0u8; 64],
            },
        }
    }

    #[test]
    fn signed_envelope_classifies_as_accepted() {
        let mut rng = OsRng;
        let signing = SigningKey::generate(&mut rng);
        let mut env = unsigned_envelope(&signing);
        finalize_signed_envelope(&mut env, "device-1", &signing).unwrap();

        // Sanity: the artifact_id is non-empty and matches recomputation.
        assert!(!env.artifact_id.is_empty());

        // Sanity: encode/decode round-trip preserves artifact_id and signature.
        let bytes = encode_env_to_vec(&env);
        let decoded = decode(&bytes).unwrap();
        assert_eq!(decoded.artifact_id, env.artifact_id);
        assert_eq!(decoded.signature.value, env.signature.value);

        let index = ImportIndex::new();
        let mut trusted = HashMap::new();
        trusted.insert(
            "device-1".to_string(),
            signing.verifying_key(),
        );
        let verdict = classify_import(&env, &index, &trusted).unwrap();
        match verdict {
            ImportVerdict::Accepted {
                annotation_outcomes,
                object_outcomes,
                ..
            } => {
                assert_eq!(
                    annotation_outcomes,
                    vec![AnnotationOutcome::Create {
                        annotation_id: "a1".to_string(),
                        revision: 1,
                        supersedes: None,
                    }]
                );
                assert_eq!(
                    object_outcomes,
                    vec![ObjectOutcome::Verified {
                        object_sha256: sha256_hex(b"payload bytes"),
                        byte_length: b"payload bytes".len() as u64,
                    }]
                );
            }
            other => panic!("expected Accepted, got {:?}", other),
        }
    }

    #[test]
    fn duplicate_artifact_sha256_is_idempotent() {
        let mut rng = OsRng;
        let signing = SigningKey::generate(&mut rng);
        let mut env = unsigned_envelope(&signing);
        finalize_signed_envelope(&mut env, "device-1", &signing).unwrap();

        let mut index = ImportIndex::new();
        let canonical = encode_env_to_vec(&env);
        let hash = sha256_hex(&canonical);
        index.mark_artifact_seen(&hash);

        let trusted = HashMap::new();
        match classify_import(&env, &index, &trusted).unwrap() {
            ImportVerdict::Duplicate { artifact_sha256 } => {
                assert_eq!(artifact_sha256, hash);
            }
            other => panic!("expected Duplicate, got {:?}", other),
        }
    }

    #[test]
    fn unsigned_artifact_requires_confirmation() {
        let mut rng = OsRng;
        let signing = SigningKey::generate(&mut rng);
        let mut env = unsigned_envelope(&signing);
        // Compute artifact_id without signing so the unsigned gate fires.
        env.artifact_id = artifact_id(&env);

        let index = ImportIndex::new();
        let trusted = HashMap::new();
        match classify_import(&env, &index, &trusted).unwrap() {
            ImportVerdict::RequiresConfirmation { reason, .. } => {
                assert!(matches!(reason, ConfirmationReason::Unsigned));
            }
            other => panic!("expected RequiresConfirmation, got {:?}", other),
        }
    }

    #[test]
    fn corrupted_signature_is_quarantined() {
        let mut rng = OsRng;
        let signing = SigningKey::generate(&mut rng);
        let mut env = unsigned_envelope(&signing);
        finalize_signed_envelope(&mut env, "device-1", &signing).unwrap();
        // Flip a bit in the signature value — should fail verification.
        env.signature.value[0] ^= 0x01;

        let index = ImportIndex::new();
        let mut trusted = HashMap::new();
        trusted.insert("device-1".to_string(), signing.verifying_key());
        match classify_import(&env, &index, &trusted).unwrap() {
            ImportVerdict::Quarantined {
                reason: QuarantineReason::InvalidSignature { .. },
                ..
            } => {}
            other => panic!("expected Quarantined(InvalidSignature), got {:?}", other),
        }
    }

    #[test]
    fn unknown_key_id_is_quarantined_not_accepted() {
        // The envelope is validly signed by its own creator, but that key
        // is not in the trust set. The embedded creator key must NOT be
        // used as a fallback: a self-signed hostile artifact quarantines.
        let mut rng = OsRng;
        let signing = SigningKey::generate(&mut rng);
        let mut env = unsigned_envelope(&signing);
        finalize_signed_envelope(&mut env, "device-1", &signing).unwrap();

        let index = ImportIndex::new();
        let trusted = HashMap::new();
        match classify_import(&env, &index, &trusted).unwrap() {
            ImportVerdict::Quarantined {
                reason: QuarantineReason::UnknownKey { key_id },
                ..
            } => assert_eq!(key_id, "device-1"),
            other => panic!("expected Quarantined(UnknownKey), got {:?}", other),
        }
    }

    #[test]
    fn corrupted_artifact_id_is_quarantined() {
        let mut rng = OsRng;
        let signing = SigningKey::generate(&mut rng);
        let mut env = unsigned_envelope(&signing);
        finalize_signed_envelope(&mut env, "device-1", &signing).unwrap();
        env.artifact_id = format!("{}{}", env.artifact_id, "0");

        let index = ImportIndex::new();
        let trusted = HashMap::new();
        match classify_import(&env, &index, &trusted).unwrap() {
            ImportVerdict::Quarantined {
                reason: QuarantineReason::ArtifactIdMismatch { .. },
                ..
            } => {}
            other => panic!("expected Quarantined(ArtifactIdMismatch), got {:?}", other),
        }
    }

    #[test]
    fn embedded_object_digest_mismatch_is_quarantined() {
        let mut rng = OsRng;
        let signing = SigningKey::generate(&mut rng);
        let mut env = unsigned_envelope(&signing);
        // Replace embedded bytes with something whose hash differs.
        env.objects[0].embedded_bytes = Some(b"totally different content".to_vec());
        env.objects[0].byte_length = b"totally different content".len() as u64;
        finalize_signed_envelope(&mut env, "device-1", &signing).unwrap();

        let index = ImportIndex::new();
        let mut trusted = HashMap::new();
        trusted.insert("device-1".to_string(), signing.verifying_key());
        match classify_import(&env, &index, &trusted).unwrap() {
            ImportVerdict::Quarantined {
                reason: QuarantineReason::ObjectDigestMismatch { .. },
                ..
            } => {}
            other => panic!("expected Quarantined(ObjectDigestMismatch), got {:?}", other),
        }
    }

    #[test]
    fn identical_revision_bytes_are_noop() {
        let mut rng = OsRng;
        let signing = SigningKey::generate(&mut rng);
        let mut env = unsigned_envelope(&signing);
        finalize_signed_envelope(&mut env, "device-1", &signing).unwrap();

        let mut index = ImportIndex::new();
        // Seed with the identical annotation.
        index.insert_annotation(&env.annotations[0]);

        let mut trusted = HashMap::new();
        trusted.insert("device-1".to_string(), signing.verifying_key());
        let verdict = classify_import(&env, &index, &trusted).unwrap();
        match verdict {
            ImportVerdict::Accepted {
                annotation_outcomes, ..
            } => {
                assert_eq!(
                    annotation_outcomes,
                    vec![AnnotationOutcome::Noop {
                        annotation_id: "a1".to_string(),
                        revision: 1,
                    }]
                );
            }
            other => panic!("expected Accepted, got {:?}", other),
        }
    }

    #[test]
    fn higher_revision_supersedes() {
        let mut rng = OsRng;
        let signing = SigningKey::generate(&mut rng);
        let mut env = unsigned_envelope(&signing);
        finalize_signed_envelope(&mut env, "device-1", &signing).unwrap();

        let mut index = ImportIndex::new();
        // Existing local annotation is at revision 1 with the same body —
        // the incoming revision is 1 in the envelope, so bump the existing
        // to 0 to test the "higher incoming" path.
        let mut existing = env.annotations[0].clone();
        existing.revision = 0;
        index.insert_annotation(&existing);

        let mut trusted = HashMap::new();
        trusted.insert("device-1".to_string(), signing.verifying_key());
        let verdict = classify_import(&env, &index, &trusted).unwrap();
        match verdict {
            ImportVerdict::Accepted {
                annotation_outcomes, ..
            } => {
                assert_eq!(
                    annotation_outcomes,
                    vec![AnnotationOutcome::Create {
                        annotation_id: "a1".to_string(),
                        revision: 1,
                        supersedes: Some(0),
                    }]
                );
            }
            other => panic!("expected Accepted, got {:?}", other),
        }
    }

    #[test]
    fn lower_revision_is_stale_noop() {
        let mut rng = OsRng;
        let signing = SigningKey::generate(&mut rng);
        let mut env = unsigned_envelope(&signing);
        finalize_signed_envelope(&mut env, "device-1", &signing).unwrap();

        let mut index = ImportIndex::new();
        let mut existing = env.annotations[0].clone();
        existing.revision = 5;
        index.insert_annotation(&existing);

        let mut trusted = HashMap::new();
        trusted.insert("device-1".to_string(), signing.verifying_key());
        let verdict = classify_import(&env, &index, &trusted).unwrap();
        match verdict {
            ImportVerdict::Accepted {
                annotation_outcomes, ..
            } => {
                assert_eq!(
                    annotation_outcomes,
                    vec![AnnotationOutcome::Stale {
                        annotation_id: "a1".to_string(),
                        incoming_revision: 1,
                        existing_revision: 5,
                    }]
                );
            }
            other => panic!("expected Accepted, got {:?}", other),
        }
    }

    #[test]
    fn divergent_equal_revisions_create_conflict() {
        let mut rng = OsRng;
        let signing = SigningKey::generate(&mut rng);
        let mut env = unsigned_envelope(&signing);
        finalize_signed_envelope(&mut env, "device-1", &signing).unwrap();

        let mut index = ImportIndex::new();
        // Same annotation_id + revision, but different selector body.
        let mut divergent = env.annotations[0].clone();
        divergent.selector = text_map(&[("exact", Value::Text("DIFFERENT".to_string()))]);
        index.insert_annotation(&divergent);

        let mut trusted = HashMap::new();
        trusted.insert("device-1".to_string(), signing.verifying_key());
        let verdict = classify_import(&env, &index, &trusted).unwrap();
        match verdict {
            ImportVerdict::Accepted {
                annotation_outcomes, ..
            } => {
                assert_eq!(annotation_outcomes.len(), 1);
                assert!(matches!(
                    annotation_outcomes[0],
                    AnnotationOutcome::Conflict { revision: 1, .. }
                ));
            }
            other => panic!("expected Accepted with Conflict, got {:?}", other),
        }
    }
}
