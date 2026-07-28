//! Sidecar import and export orchestration — ARCHITECTURE.md §9.6.
//!
//! This module is the application-layer policy for portable annotation
//! bundles. It consumes an already-decoded sidecar artifact, drives the
//! [`SignatureProvider`](ports::signature::SignatureProvider) port for
//! signature verification and signing, applies the §9.6 conflict-key
//! rules, and emits outcomes. It performs no storage mutation directly:
//! the caller dispatches on the returned verdict (e.g. calling
//! `ports::annotation::AnnotationService` for accepted `Create` outcomes,
//! surfacing `Conflict` outcomes to the user, persisting `Quarantined`
//! artifacts for forensics).
//!
//! ## Why this lives here, not in the `sidecar` provider crate
//!
//! The `sidecar` provider owns the CDDL data model, the canonical CBOR
//! codec, and a reference Ed25519 keystore. But §9.5 fixes signature
//! verification behind a *port* precisely so that an HSM-backed or
//! remote keystore can be substituted without rewriting the import
//! policy. A-02 ("ports before providers") forbids `application` from
//! naming a provider crate, so the §9.6 rules are transcribed here a
//! second time, driven entirely through `SignatureProvider`. The two
//! implementations agree because both transcribe the same authority.
//!
//! ## The §9.6 rules, transcribed verbatim
//!
//! - Import is idempotent by `artifact_sha256`.
//! - Conflict key: `annotation_id + revision`.
//! - identical revision bytes are a no-op;
//! - higher imported revision creates an imported-layer revision;
//! - divergent equal revisions create a conflict task;
//! - imported private layers remain imported and are not merged into the
//!   local private layer automatically;
//! - invalid signatures are quarantined;
//! - unsigned artifacts require explicit user confirmation;
//! - object digests are verified before object commit.
//!
//! The seam between this module and the provider is the
//! [`SidecarArtifact`] DTO: primitive fields and domain types only, so
//! no provider type ever crosses into `application`.

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use domain::trust::{SignatureEnvelope, SignatureVerificationState};
use ports::signature::{SignatureError, SignatureProvider};

/// Domain separation string used while signing sidecar envelopes.
///
/// ARCHITECTURE.md §9.4 fixes the sidecar domain prefix to UTF-8
/// `enzime.sidecar.v1\0`. The `SignatureProvider` port prepends the NUL
/// terminator internally (see the `sidecar` provider's `domain_prefix`),
/// so callers pass this bare string to [`SignatureProvider::sign`] /
/// [`SignatureProvider::verify`].
pub const SIGNATURE_DOMAIN: &str = "enzime.sidecar.v1";

/// Decoded sidecar artifact — the only thing that crosses the seam from
/// the `sidecar` provider into this module.
///
/// Every field is either a primitive or a `domain` type, so this DTO
/// carries no provider dependency. The caller's codec is responsible for
/// canonicalization, the `artifact_id` and `artifact_sha256` digests,
/// and the per-annotation `canonical_bytes` computations.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SidecarArtifact {
    /// SHA-256 of the canonical unsigned envelope. ARCHITECTURE.md §9.4:
    /// "`artifact_id` is SHA-256 of canonical unsigned envelope". Used by
    /// the import path to detect a corrupted/replaced envelope header.
    pub artifact_id: String,

    /// SHA-256 of the canonical signed envelope bytes. §9.6: "Import is
    /// idempotent by `artifact_sha256`" — this is that key.
    pub artifact_sha256: String,

    /// Canonical-encoded envelope with `artifact_id` blanked and the
    /// signature `value` entry omitted, exactly as ARCHITECTURE.md §9.4
    /// ("signature value omitted while signing") requires. These are the
    /// bytes the signer signed, and what
    /// [`SignatureProvider::verify`](ports::signature::SignatureProvider::verify)
    /// must reconstruct.
    pub canonical_unsigned_bytes: Vec<u8>,

    /// Envelope signature. An all-zero `value` is the codec's canonical
    /// "no signature" marker and triggers the unsigned-confirmation gate
    /// of §9.6 before any cryptographic verification runs.
    pub signature: SignatureEnvelope,

    /// Layer visibility transcribed from the envelope `layer.visibility`.
    /// §9.6: "imported private layers remain imported and are not merged
    /// into the local private layer automatically" — the importer never
    /// flips an `Imported` or `Private` layer into the local private
    /// layer; that is a separate, user-initiated action.
    pub layer_visibility: SidecarLayerVisibility,

    /// Annotations carried by the artifact, each with its canonical
    /// bytes pre-computed by the caller's codec. Equality on those bytes
    /// is what §9.6 calls "identical revision bytes".
    pub annotations: Vec<SidecarArtifactAnnotation>,

    /// Object references with optional embedded bytes. §9.6: "object
    /// digests are verified before object commit" — embedded bytes are
    /// hashed here and any mismatch quarantines the whole artifact
    /// before any object reaches storage.
    pub objects: Vec<SidecarArtifactObject>,
}

/// Visibility of the layer an imported artifact belongs to
/// (ARCHITECTURE.md §9.4 `layer.visibility`).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SidecarLayerVisibility {
    Private,
    Shared,
    Imported,
}

/// One annotation entry inside a [`SidecarArtifact`].
///
/// `canonical_bytes` is the encoding the caller's codec produces for the
/// annotation map — the same bytes the producer hashed for equality.
/// Producers and verifiers agree on equality regardless of source-code
/// field order because both use the canonical encoder.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SidecarArtifactAnnotation {
    pub annotation_id: String,
    pub revision: u32,
    pub canonical_bytes: Vec<u8>,
}

/// One object reference inside a [`SidecarArtifact`]
/// (ARCHITECTURE.md §9.4 `object-ref`).
///
/// When `embedded_bytes` is `Some(b)`, `b` is the literal object content
/// carried inside the sidecar so an offline recipient can verify and
/// store it without an external fetch. The importer hashes `b` and
/// compares the digest to `object_sha256` before declaring the object
/// ready to commit (§9.6: "object digests are verified before object
/// commit").
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SidecarArtifactObject {
    pub object_sha256: String,
    pub mime_type: String,
    pub byte_length: u64,
    pub embedded_bytes: Option<Vec<u8>>,
}

/// In-memory index of sidecar artifacts already imported by the local
/// device and annotations already known.
///
/// Used by [`SidecarImporter::classify`] to evaluate the §9.6 idempotency
/// and conflict-key rules. The index is purely advisory state; persistent
/// backing (so a restart preserves idempotency) is the caller's job —
/// the application layer does not name a storage port here, because
/// §9.6 itself does not constrain where the index lives, only how it is
/// consulted.
#[derive(Clone, Debug, Default)]
pub struct ImportIndex {
    seen_artifact_sha256: HashMap<String, ()>,
    annotations: HashMap<String, IndexedAnnotation>,
}

#[derive(Clone, Debug)]
struct IndexedAnnotation {
    revision: u32,
    canonical_bytes: Vec<u8>,
}

impl ImportIndex {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record that `artifact_sha256` has been imported. Subsequent calls
    /// to [`SidecarImporter::classify`] with the same hash return
    /// [`ImportOutcome::Duplicate`].
    pub fn mark_artifact_seen(&mut self, artifact_sha256: &str) {
        self.seen_artifact_sha256
            .insert(artifact_sha256.to_string(), ());
    }

    /// Returns true if an artifact with this SHA-256 has already been
    /// imported (§9.6 idempotency key).
    pub fn has_artifact(&self, artifact_sha256: &str) -> bool {
        self.seen_artifact_sha256.contains_key(artifact_sha256)
    }

    /// Seed the per-annotation index with an annotation that already
    /// exists locally, so the §9.6 conflict-key rules can compare
    /// against it.
    pub fn index_annotation(
        &mut self,
        annotation_id: &str,
        revision: u32,
        canonical_bytes: Vec<u8>,
    ) {
        self.annotations.insert(
            annotation_id.to_string(),
            IndexedAnnotation {
                revision,
                canonical_bytes,
            },
        );
    }

    fn lookup_annotation(&self, annotation_id: &str) -> Option<&IndexedAnnotation> {
        self.annotations.get(annotation_id)
    }
}

/// Verdict returned by [`SidecarImporter::classify`] for one incoming
/// sidecar artifact.
#[derive(Clone, Debug)]
pub enum ImportOutcome {
    /// Artifact was already imported — §9.6 idempotency by
    /// `artifact_sha256`. The caller performs no further action.
    Duplicate { artifact_sha256: String },

    /// Artifact failed an integrity gate (signature invalid, signed
    /// domain wrong, artifact_id mismatch, or embedded object digest
    /// mismatch). §9.6: "invalid signatures are quarantined". The
    /// caller MUST NOT activate any bytes from this artifact; it should
    /// be persisted to a quarantine store for forensics.
    Quarantined {
        artifact_id: String,
        reason: QuarantineReason,
    },

    /// Artifact carries no usable signature. §9.6: "unsigned artifacts
    /// require explicit user confirmation". The caller surfaces this to
    /// the user; only on explicit confirmation may the caller re-run the
    /// import via [`SidecarImporter::classify_confirmed`].
    RequiresConfirmation {
        artifact_id: String,
        artifact_sha256: String,
        reason: ConfirmationReason,
    },

    /// Artifact passed every gate. The caller dispatches on the
    /// per-annotation outcomes (creating imported-layer revisions for
    /// each `Create`, surfacing `Conflict` outcomes as conflict tasks,
    /// and committing verified objects).
    Accepted {
        artifact_id: String,
        artifact_sha256: String,
        signature_key_id: String,
        annotation_outcomes: Vec<AnnotationOutcome>,
        object_outcomes: Vec<ObjectOutcome>,
    },
}

/// Why an artifact was quarantined.
#[derive(Clone, Debug)]
pub enum QuarantineReason {
    /// The envelope-reported `artifact_id` did not match the SHA-256 of
    /// the canonical unsigned bytes — the envelope has been edited or
    /// corrupted in transit.
    ArtifactIdMismatch {
        reported: String,
        recomputed: String,
    },

    /// The signature failed cryptographic verification, or the
    /// `SignatureProvider` reported the key as unknown or revoked. The
    /// `state` field carries the exact verification result so the
    /// caller can distinguish "wrong key" from "tampered bytes" in
    /// user-facing messaging.
    SignatureUnverifiable {
        state: SignatureVerificationState,
        key_id: String,
        explanation: String,
    },

    /// An embedded object's claimed `object_sha256` or `byte_length`
    /// does not match the actual embedded bytes. §9.6: "object digests
    /// are verified before object commit" — mismatch quarantines the
    /// whole artifact; partial object commit is not permitted.
    ObjectDigestMismatch {
        object_sha256: String,
        expected: String,
        actual: String,
    },
}

/// Why an artifact requires user confirmation before import.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfirmationReason {
    /// The signature value is the codec's canonical all-zero "no
    /// signature" marker — the artifact is intentionally unsigned.
    Unsigned,
}

/// Per-annotation outcome under the §9.6 conflict-key rules.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AnnotationOutcome {
    /// Existing revision has identical canonical bytes — nothing to do.
    /// §9.6: "identical revision bytes are a no-op".
    Noop {
        annotation_id: String,
        revision: u32,
    },

    /// No prior entry for this `annotation_id`, or the incoming revision
    /// is strictly higher. §9.6: "higher imported revision creates an
    /// imported-layer revision". When `supersedes` is `Some(r)`, the
    /// caller should mark the prior revision `r` as superseded while
    /// creating the new revision on the imported layer.
    Create {
        annotation_id: String,
        revision: u32,
        supersedes: Option<u32>,
    },

    /// Incoming revision is strictly lower than the locally-known one.
    /// The import is a no-op for this annotation — the local revision
    /// wins. (Listed explicitly so the caller can record it for
    /// auditability; §9.6 does not require a conflict task here.)
    Stale {
        annotation_id: String,
        incoming_revision: u32,
        existing_revision: u32,
    },

    /// Equal revisions with divergent canonical bytes. §9.6: "divergent
    /// equal revisions create a conflict task". The caller surfaces this
    /// to the user as a manual-resolve conflict; neither side is
    /// applied automatically.
    Conflict {
        annotation_id: String,
        revision: u32,
        existing_bytes: Vec<u8>,
        incoming_bytes: Vec<u8>,
    },
}

/// Per-object outcome — embedded objects only; external objects are
/// reported as [`ObjectOutcome::External`] and the caller must fetch
/// them before commit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ObjectOutcome {
    /// Embedded bytes hash to the declared `object_sha256` and match the
    /// declared `byte_length` — ready to commit to object storage.
    Verified {
        object_sha256: String,
        byte_length: u64,
    },

    /// No embedded bytes — the object is referenced but not carried.
    /// Before committing the artifact, the caller must fetch the object
    /// by `object_sha256` (e.g. from a transfer port) and re-verify its
    /// digest.
    External { object_sha256: String },
}

/// Application-layer sidecar importer.
///
/// Holds an injected [`SignatureProvider`] (the port) and applies the
/// §9.6 rules to incoming artifacts. Stateless apart from the provider
/// reference — the per-device import index is passed in by the caller
/// so it can be backed by persistent storage.
pub struct SidecarImporter {
    signature: Arc<dyn SignatureProvider>,
}

impl SidecarImporter {
    pub fn new(signature: Arc<dyn SignatureProvider>) -> Self {
        Self { signature }
    }

    /// Classify an incoming sidecar artifact against `index` using the
    /// §9.6 rules. Pure with respect to local state — performs no
    /// storage mutation, only signals what the caller should do.
    ///
    /// Order of gates (each can short-circuit):
    ///   1. idempotency by `artifact_sha256`;
    ///   2. `artifact_id` matches SHA-256 of the canonical unsigned
    ///      bytes (envelope-integrity check, independent of cryptography);
    ///   3. unsigned artifacts require explicit user confirmation;
    ///   4. signature verification via the `SignatureProvider` port —
    ///      invalid / unknown-key / revoked-key signatures quarantine;
    ///   5. embedded object digests verified before object commit;
    ///   6. per-annotation conflict resolution.
    pub async fn classify(
        &self,
        artifact: &SidecarArtifact,
        index: &ImportIndex,
    ) -> Result<ImportOutcome, SidecarImportError> {
        // 1. §9.6: "Import is idempotent by `artifact_sha256`".
        if index.has_artifact(&artifact.artifact_sha256) {
            return Ok(ImportOutcome::Duplicate {
                artifact_sha256: artifact.artifact_sha256.clone(),
            });
        }

        // 2. Envelope integrity: the reported `artifact_id` must equal
        //    SHA-256 of the canonical unsigned bytes. A mismatch means
        //    the envelope was edited after signing or corrupted in
        //    transit — quarantine without even consulting the signer.
        let recomputed_artifact_id = sha256_hex(&artifact.canonical_unsigned_bytes);
        if artifact.artifact_id != recomputed_artifact_id {
            return Ok(ImportOutcome::Quarantined {
                artifact_id: artifact.artifact_id.clone(),
                reason: QuarantineReason::ArtifactIdMismatch {
                    reported: artifact.artifact_id.clone(),
                    recomputed: recomputed_artifact_id,
                },
            });
        }

        // 3. §9.6: "unsigned artifacts require explicit user
        //    confirmation". The codec's canonical "no signature" marker
        //    is an all-zero signature value; detect it before crypto.
        if is_unsigned_marker(&artifact.signature) {
            return Ok(ImportOutcome::RequiresConfirmation {
                artifact_id: artifact.artifact_id.clone(),
                artifact_sha256: artifact.artifact_sha256.clone(),
                reason: ConfirmationReason::Unsigned,
            });
        }

        // 4. §9.6: "invalid signatures are quarantined". Delegate the
        //    cryptography to the port so any SignatureProvider backend
        //    (in-process Ed25519, HSM, remote) can be substituted.
        let verification = self
            .signature
            .verify(
                SIGNATURE_DOMAIN,
                &artifact.canonical_unsigned_bytes,
                &artifact.signature,
            )
            .await
            .map_err(SidecarImportError::SignatureBackend)?;
        match verification.state {
            SignatureVerificationState::Verified => { /* continue */ }
            SignatureVerificationState::Unsigned => {
                // The provider can report Unsigned if it knows the key
                // is absent; treat it as the confirmation path.
                return Ok(ImportOutcome::RequiresConfirmation {
                    artifact_id: artifact.artifact_id.clone(),
                    artifact_sha256: artifact.artifact_sha256.clone(),
                    reason: ConfirmationReason::Unsigned,
                });
            }
            state @ (SignatureVerificationState::Invalid
            | SignatureVerificationState::UnknownKey
            | SignatureVerificationState::RevokedKey) => {
                return Ok(ImportOutcome::Quarantined {
                    artifact_id: artifact.artifact_id.clone(),
                    reason: QuarantineReason::SignatureUnverifiable {
                        state,
                        key_id: verification.key_id,
                        explanation: verification.explanation,
                    },
                });
            }
        }

        // 5. §9.6: "object digests are verified before object commit".
        //    Any embedded object whose claimed digest or length does
        //    not match the actual bytes quarantines the whole artifact;
        //    partial commit is not permitted.
        let mut object_outcomes = Vec::with_capacity(artifact.objects.len());
        for object in &artifact.objects {
            match &object.embedded_bytes {
                Some(bytes) => {
                    if bytes.len() as u64 != object.byte_length {
                        return Ok(ImportOutcome::Quarantined {
                            artifact_id: artifact.artifact_id.clone(),
                            reason: QuarantineReason::ObjectDigestMismatch {
                                object_sha256: object.object_sha256.clone(),
                                expected: object.object_sha256.clone(),
                                actual: format!(
                                    "byte_length={} but embedded_bytes.len()={}",
                                    object.byte_length,
                                    bytes.len()
                                ),
                            },
                        });
                    }
                    let actual = sha256_hex(bytes);
                    if actual != object.object_sha256 {
                        return Ok(ImportOutcome::Quarantined {
                            artifact_id: artifact.artifact_id.clone(),
                            reason: QuarantineReason::ObjectDigestMismatch {
                                object_sha256: object.object_sha256.clone(),
                                expected: object.object_sha256.clone(),
                                actual,
                            },
                        });
                    }
                    object_outcomes.push(ObjectOutcome::Verified {
                        object_sha256: object.object_sha256.clone(),
                        byte_length: object.byte_length,
                    });
                }
                None => object_outcomes.push(ObjectOutcome::External {
                    object_sha256: object.object_sha256.clone(),
                }),
            }
        }

        // 6. Per-annotation conflict resolution under §9.6 conflict key
        //    `annotation_id + revision`.
        let mut annotation_outcomes = Vec::with_capacity(artifact.annotations.len());
        for incoming in &artifact.annotations {
            let outcome = match index.lookup_annotation(&incoming.annotation_id) {
                None => AnnotationOutcome::Create {
                    annotation_id: incoming.annotation_id.clone(),
                    revision: incoming.revision,
                    supersedes: None,
                },
                Some(existing) => match incoming.revision.cmp(&existing.revision) {
                    std::cmp::Ordering::Greater => AnnotationOutcome::Create {
                        annotation_id: incoming.annotation_id.clone(),
                        revision: incoming.revision,
                        supersedes: Some(existing.revision),
                    },
                    std::cmp::Ordering::Less => AnnotationOutcome::Stale {
                        annotation_id: incoming.annotation_id.clone(),
                        incoming_revision: incoming.revision,
                        existing_revision: existing.revision,
                    },
                    std::cmp::Ordering::Equal => {
                        if incoming.canonical_bytes == existing.canonical_bytes {
                            AnnotationOutcome::Noop {
                                annotation_id: incoming.annotation_id.clone(),
                                revision: incoming.revision,
                            }
                        } else {
                            AnnotationOutcome::Conflict {
                                annotation_id: incoming.annotation_id.clone(),
                                revision: incoming.revision,
                                existing_bytes: existing.canonical_bytes.clone(),
                                incoming_bytes: incoming.canonical_bytes.clone(),
                            }
                        }
                    }
                },
            };
            annotation_outcomes.push(outcome);
        }

        Ok(ImportOutcome::Accepted {
            artifact_id: artifact.artifact_id.clone(),
            artifact_sha256: artifact.artifact_sha256.clone(),
            signature_key_id: verification.key_id,
            annotation_outcomes,
            object_outcomes,
        })
    }

    /// Re-run classification for an artifact the user has explicitly
    /// confirmed despite being unsigned (§9.6: "unsigned artifacts
    /// require explicit user confirmation").
    ///
    /// Behaviour is identical to [`Self::classify`] except the
    /// unsigned-confirmation gate (step 3) is skipped. The caller MUST
    /// only invoke this after obtaining explicit user confirmation for
    /// the supplied `artifact_sha256`; the gate exists because an
    /// unsigned artifact carries no integrity guarantee.
    pub async fn classify_confirmed(
        &self,
        artifact: &SidecarArtifact,
        index: &ImportIndex,
    ) -> Result<ImportOutcome, SidecarImportError> {
        // The unsigned gate (step 3 in `classify`) is skipped here. To
        // avoid duplicating the rest of the pipeline, we forward to
        // `classify` for the still-signed case and only short-circuit
        // when the artifact is actually unsigned.
        if !is_unsigned_marker(&artifact.signature) {
            return self.classify(artifact, index).await;
        }

        // 1. idempotency still applies.
        if index.has_artifact(&artifact.artifact_sha256) {
            return Ok(ImportOutcome::Duplicate {
                artifact_sha256: artifact.artifact_sha256.clone(),
            });
        }

        // 2. envelope integrity still applies.
        let recomputed_artifact_id = sha256_hex(&artifact.canonical_unsigned_bytes);
        if artifact.artifact_id != recomputed_artifact_id {
            return Ok(ImportOutcome::Quarantined {
                artifact_id: artifact.artifact_id.clone(),
                reason: QuarantineReason::ArtifactIdMismatch {
                    reported: artifact.artifact_id.clone(),
                    recomputed: recomputed_artifact_id,
                },
            });
        }

        // 3. (skipped — user has confirmed)

        // 4. signature verification is skipped because there is no
        //    signature to verify; the user's confirmation is the trust
        //    root for this artifact.

        // 5. object digests still must verify before commit.
        let mut object_outcomes = Vec::with_capacity(artifact.objects.len());
        for object in &artifact.objects {
            match &object.embedded_bytes {
                Some(bytes) => {
                    if bytes.len() as u64 != object.byte_length {
                        return Ok(ImportOutcome::Quarantined {
                            artifact_id: artifact.artifact_id.clone(),
                            reason: QuarantineReason::ObjectDigestMismatch {
                                object_sha256: object.object_sha256.clone(),
                                expected: object.object_sha256.clone(),
                                actual: format!(
                                    "byte_length={} but embedded_bytes.len()={}",
                                    object.byte_length,
                                    bytes.len()
                                ),
                            },
                        });
                    }
                    let actual = sha256_hex(bytes);
                    if actual != object.object_sha256 {
                        return Ok(ImportOutcome::Quarantined {
                            artifact_id: artifact.artifact_id.clone(),
                            reason: QuarantineReason::ObjectDigestMismatch {
                                object_sha256: object.object_sha256.clone(),
                                expected: object.object_sha256.clone(),
                                actual,
                            },
                        });
                    }
                    object_outcomes.push(ObjectOutcome::Verified {
                        object_sha256: object.object_sha256.clone(),
                        byte_length: object.byte_length,
                    });
                }
                None => object_outcomes.push(ObjectOutcome::External {
                    object_sha256: object.object_sha256.clone(),
                }),
            }
        }

        // 6. conflict-key resolution still applies.
        let mut annotation_outcomes = Vec::with_capacity(artifact.annotations.len());
        for incoming in &artifact.annotations {
            let outcome = match index.lookup_annotation(&incoming.annotation_id) {
                None => AnnotationOutcome::Create {
                    annotation_id: incoming.annotation_id.clone(),
                    revision: incoming.revision,
                    supersedes: None,
                },
                Some(existing) => match incoming.revision.cmp(&existing.revision) {
                    std::cmp::Ordering::Greater => AnnotationOutcome::Create {
                        annotation_id: incoming.annotation_id.clone(),
                        revision: incoming.revision,
                        supersedes: Some(existing.revision),
                    },
                    std::cmp::Ordering::Less => AnnotationOutcome::Stale {
                        annotation_id: incoming.annotation_id.clone(),
                        incoming_revision: incoming.revision,
                        existing_revision: existing.revision,
                    },
                    std::cmp::Ordering::Equal => {
                        if incoming.canonical_bytes == existing.canonical_bytes {
                            AnnotationOutcome::Noop {
                                annotation_id: incoming.annotation_id.clone(),
                                revision: incoming.revision,
                            }
                        } else {
                            AnnotationOutcome::Conflict {
                                annotation_id: incoming.annotation_id.clone(),
                                revision: incoming.revision,
                                existing_bytes: existing.canonical_bytes.clone(),
                                incoming_bytes: incoming.canonical_bytes.clone(),
                            }
                        }
                    }
                },
            };
            annotation_outcomes.push(outcome);
        }

        Ok(ImportOutcome::Accepted {
            artifact_id: artifact.artifact_id.clone(),
            artifact_sha256: artifact.artifact_sha256.clone(),
            // Unsigned artifacts carry no key_id; record the empty
            // string so the outcome type stays consistent.
            signature_key_id: String::new(),
            annotation_outcomes,
            object_outcomes,
        })
    }
}

/// Result of signing a sidecar artifact for export.
///
/// The caller's codec uses these three fields to finalize the envelope:
/// write `artifact_id` into the envelope header, write `signature` into
/// the signature sub-map, then re-encode the canonical signed form and
/// compute its SHA-256 to populate `artifact_sha256` on the producing
/// side. (The application layer does not own the codec, so it stops
/// here.)
#[derive(Clone, Debug)]
pub struct SignedExport {
    /// SHA-256 of the canonical unsigned bytes — written into the
    /// envelope header before the final signed re-encode.
    pub artifact_id: String,
    /// Signature produced by the injected `SignatureProvider` over the
    /// canonical unsigned bytes under [`SIGNATURE_DOMAIN`].
    pub signature: SignatureEnvelope,
}

/// Application-layer sidecar exporter.
///
/// Holds an injected [`SignatureProvider`] (the port) and produces the
/// signed envelope header for an outgoing sidecar artifact. As with
/// [`SidecarImporter`], no provider crate is named: the caller hands
/// the canonical-unsigned bytes from their codec, the exporter signs
/// via the port and returns the signature plus the `artifact_id`
/// digest.
pub struct SidecarExporter {
    signature: Arc<dyn SignatureProvider>,
}

impl SidecarExporter {
    pub fn new(signature: Arc<dyn SignatureProvider>) -> Self {
        Self { signature }
    }

    /// Sign an outgoing sidecar artifact.
    ///
    /// `canonical_unsigned_bytes` MUST be the canonical-encoded envelope
    /// with `artifact_id` blanked and the signature `value` entry
    /// omitted (exactly the form §9.4 specifies for signing). The
    /// caller's codec is responsible for producing these bytes; this
    /// method only signs them and derives the `artifact_id` digest.
    pub async fn sign_export(
        &self,
        canonical_unsigned_bytes: &[u8],
    ) -> Result<SignedExport, SidecarExportError> {
        let signature = self
            .signature
            .sign(SIGNATURE_DOMAIN, canonical_unsigned_bytes)
            .await
            .map_err(SidecarExportError::SignatureBackend)?;
        let artifact_id = sha256_hex(canonical_unsigned_bytes);
        Ok(SignedExport {
            artifact_id,
            signature,
        })
    }
}

/// Errors raised by the sidecar import path.
///
/// Storage-layer errors (e.g. a failure to persist an accepted
/// annotation) are not modelled here because the importer performs no
/// storage mutation; the caller surfaces those through the storage
/// port's own error type when dispatching on [`ImportOutcome::Accepted`].
#[derive(Debug, thiserror::Error)]
pub enum SidecarImportError {
    /// The injected `SignatureProvider` returned an error before it
    /// could produce a verification verdict (e.g. keystore unreachable).
    /// Distinct from a verified-Invalid verdict, which is reported as
    /// [`ImportOutcome::Quarantined`].
    #[error("signature backend failure during import: {0}")]
    SignatureBackend(#[from] SignatureError),
}

/// Errors raised by the sidecar export path.
#[derive(Debug, thiserror::Error)]
pub enum SidecarExportError {
    /// The injected `SignatureProvider` could not sign — no local
    /// identity, key revoked, or keystore error.
    #[error("signature backend failure during export: {0}")]
    SignatureBackend(#[from] SignatureError),
}

/// SHA-256 of `bytes`, lowercase hex-encoded — the digest encoding used
/// for `artifact_id`, `artifact_sha256`, and `object_sha256` throughout
/// §9.4 and §9.6.
fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

/// Returns true if `envelope.value` is the codec's canonical "no
/// signature" marker — 64 zero bytes (Ed25519 signature length).
///
/// ARCHITECTURE.md §9.4 fixes signature value length at `bstr .size 64`;
/// the codec uses the all-zero value of that length as the canonical
/// unsigned marker so that "unsigned" has a single deterministic
/// encoding. An empty `value` is treated as unsigned too — defensive
/// against malformed input that never carried a signature.
fn is_unsigned_marker(envelope: &SignatureEnvelope) -> bool {
    envelope.value.is_empty() || envelope.value.iter().all(|b| *b == 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use domain::trust::{SignatureEnvelope, SignatureVerification};

    /// In-memory test-only SignatureProvider that returns canned
    /// verification verdicts and records the bytes it was asked to
    /// sign. NEVER used outside this module's tests; the production
    /// path injects a real provider.
    struct CannedProvider {
        verify_state: SignatureVerificationState,
        last_signed: std::sync::Mutex<Option<(String, Vec<u8>)>>,
    }

    impl CannedProvider {
        fn with_state(state: SignatureVerificationState) -> Self {
            Self {
                verify_state: state,
                last_signed: std::sync::Mutex::new(None),
            }
        }
    }

    #[async_trait]
    impl SignatureProvider for CannedProvider {
        async fn sign(
            &self,
            domain: &str,
            canonical_payload: &[u8],
        ) -> Result<SignatureEnvelope, SignatureError> {
            *self.last_signed.lock().unwrap() =
                Some((domain.to_string(), canonical_payload.to_vec()));
            Ok(SignatureEnvelope {
                algorithm: "ed25519".to_string(),
                key_id: "test-key".to_string(),
                signed_domain: domain.to_string(),
                value: vec![0u8; 64],
            })
        }

        async fn verify(
            &self,
            _domain: &str,
            _canonical_payload: &[u8],
            envelope: &SignatureEnvelope,
        ) -> Result<SignatureVerification, SignatureError> {
            Ok(SignatureVerification {
                state: self.verify_state,
                key_id: envelope.key_id.clone(),
                signer_label: None,
                explanation: "canned".to_string(),
            })
        }
    }

    fn artifact_id_for(payload: &[u8]) -> String {
        sha256_hex(payload)
    }

    fn signed_artifact(
        canonical_unsigned_bytes: &[u8],
        annotations: Vec<SidecarArtifactAnnotation>,
        objects: Vec<SidecarArtifactObject>,
    ) -> SidecarArtifact {
        SidecarArtifact {
            artifact_id: artifact_id_for(canonical_unsigned_bytes),
            artifact_sha256: sha256_hex(b"signed envelope bytes"),
            canonical_unsigned_bytes: canonical_unsigned_bytes.to_vec(),
            signature: SignatureEnvelope {
                algorithm: "ed25519".to_string(),
                key_id: "device-1".to_string(),
                signed_domain: SIGNATURE_DOMAIN.to_string(),
                value: vec![0xABu8; 64],
            },
            layer_visibility: SidecarLayerVisibility::Imported,
            annotations,
            objects,
        }
    }

    fn ann(id: &str, revision: u32, body: &[u8]) -> SidecarArtifactAnnotation {
        SidecarArtifactAnnotation {
            annotation_id: id.to_string(),
            revision,
            canonical_bytes: body.to_vec(),
        }
    }

    #[tokio::test]
    async fn duplicate_artifact_sha256_is_idempotent() {
        let importer = SidecarImporter::new(Arc::new(CannedProvider::with_state(
            SignatureVerificationState::Verified,
        )));
        let artifact = signed_artifact(b"payload", vec![], vec![]);
        let mut index = ImportIndex::new();
        index.mark_artifact_seen(&artifact.artifact_sha256);

        match importer.classify(&artifact, &index).await.unwrap() {
            ImportOutcome::Duplicate { artifact_sha256 } => {
                assert_eq!(artifact_sha256, artifact.artifact_sha256);
            }
            other => panic!("expected Duplicate, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn corrupted_artifact_id_is_quarantined_before_signing() {
        let importer = SidecarImporter::new(Arc::new(CannedProvider::with_state(
            SignatureVerificationState::Verified,
        )));
        let mut artifact = signed_artifact(b"payload", vec![], vec![]);
        artifact.artifact_id = format!("{}-tampered", artifact.artifact_id);

        match importer.classify(&artifact, &ImportIndex::new()).await.unwrap() {
            ImportOutcome::Quarantined {
                reason: QuarantineReason::ArtifactIdMismatch { reported, recomputed },
                ..
            } => {
                assert_eq!(reported, artifact.artifact_id);
                assert_eq!(recomputed, artifact_id_for(b"payload"));
            }
            other => panic!("expected ArtifactIdMismatch, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn unsigned_artifact_requires_confirmation() {
        let importer = SidecarImporter::new(Arc::new(CannedProvider::with_state(
            SignatureVerificationState::Verified,
        )));
        let mut artifact = signed_artifact(b"payload", vec![], vec![]);
        artifact.signature.value = vec![0u8; 64];

        match importer.classify(&artifact, &ImportIndex::new()).await.unwrap() {
            ImportOutcome::RequiresConfirmation { reason, .. } => {
                assert_eq!(reason, ConfirmationReason::Unsigned);
            }
            other => panic!("expected RequiresConfirmation, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn confirmed_unsigned_artifact_is_accepted_without_signature() {
        let importer = SidecarImporter::new(Arc::new(CannedProvider::with_state(
            SignatureVerificationState::Invalid,
        )));
        let mut artifact = signed_artifact(b"payload", vec![ann("a1", 1, b"x")], vec![]);
        artifact.signature.value = vec![0u8; 64];

        match importer.classify_confirmed(&artifact, &ImportIndex::new()).await.unwrap() {
            ImportOutcome::Accepted {
                annotation_outcomes,
                signature_key_id,
                ..
            } => {
                assert!(signature_key_id.is_empty());
                assert_eq!(
                    annotation_outcomes,
                    vec![AnnotationOutcome::Create {
                        annotation_id: "a1".to_string(),
                        revision: 1,
                        supersedes: None,
                    }]
                );
            }
            other => panic!("expected Accepted, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn invalid_signature_is_quarantined() {
        let importer = SidecarImporter::new(Arc::new(CannedProvider::with_state(
            SignatureVerificationState::Invalid,
        )));
        let artifact = signed_artifact(b"payload", vec![], vec![]);

        match importer.classify(&artifact, &ImportIndex::new()).await.unwrap() {
            ImportOutcome::Quarantined {
                reason: QuarantineReason::SignatureUnverifiable { state, .. },
                ..
            } => {
                assert_eq!(state, SignatureVerificationState::Invalid);
            }
            other => panic!("expected SignatureUnverifiable, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn revoked_key_is_quarantined() {
        let importer = SidecarImporter::new(Arc::new(CannedProvider::with_state(
            SignatureVerificationState::RevokedKey,
        )));
        let artifact = signed_artifact(b"payload", vec![], vec![]);

        match importer.classify(&artifact, &ImportIndex::new()).await.unwrap() {
            ImportOutcome::Quarantined {
                reason: QuarantineReason::SignatureUnverifiable { state, .. },
                ..
            } => assert_eq!(state, SignatureVerificationState::RevokedKey),
            other => panic!("expected SignatureUnverifiable, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn unknown_key_is_quarantined() {
        let importer = SidecarImporter::new(Arc::new(CannedProvider::with_state(
            SignatureVerificationState::UnknownKey,
        )));
        let artifact = signed_artifact(b"payload", vec![], vec![]);

        match importer.classify(&artifact, &ImportIndex::new()).await.unwrap() {
            ImportOutcome::Quarantined {
                reason: QuarantineReason::SignatureUnverifiable { state, .. },
                ..
            } => assert_eq!(state, SignatureVerificationState::UnknownKey),
            other => panic!("expected SignatureUnverifiable, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn embedded_object_digest_mismatch_is_quarantined() {
        let importer = SidecarImporter::new(Arc::new(CannedProvider::with_state(
            SignatureVerificationState::Verified,
        )));
        let object = SidecarArtifactObject {
            object_sha256: sha256_hex(b"declared"),
            mime_type: "text/plain".to_string(),
            byte_length: b"declared".len() as u64,
            embedded_bytes: Some(b"different content".to_vec()),
        };
        let artifact = signed_artifact(b"payload", vec![], vec![object]);

        match importer.classify(&artifact, &ImportIndex::new()).await.unwrap() {
            ImportOutcome::Quarantined {
                reason: QuarantineReason::ObjectDigestMismatch { .. },
                ..
            } => {}
            other => panic!("expected ObjectDigestMismatch, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn embedded_object_byte_length_mismatch_is_quarantined() {
        let importer = SidecarImporter::new(Arc::new(CannedProvider::with_state(
            SignatureVerificationState::Verified,
        )));
        let object = SidecarArtifactObject {
            object_sha256: sha256_hex(b"payload bytes"),
            mime_type: "text/plain".to_string(),
            byte_length: 999,
            embedded_bytes: Some(b"payload bytes".to_vec()),
        };
        let artifact = signed_artifact(b"payload", vec![], vec![object]);

        match importer.classify(&artifact, &ImportIndex::new()).await.unwrap() {
            ImportOutcome::Quarantined {
                reason: QuarantineReason::ObjectDigestMismatch { .. },
                ..
            } => {}
            other => panic!("expected ObjectDigestMismatch, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn external_object_is_reported_not_verified() {
        let importer = SidecarImporter::new(Arc::new(CannedProvider::with_state(
            SignatureVerificationState::Verified,
        )));
        let object = SidecarArtifactObject {
            object_sha256: "abc".to_string(),
            mime_type: "text/plain".to_string(),
            byte_length: 0,
            embedded_bytes: None,
        };
        let artifact = signed_artifact(b"payload", vec![], vec![object]);

        match importer.classify(&artifact, &ImportIndex::new()).await.unwrap() {
            ImportOutcome::Accepted { object_outcomes, .. } => {
                assert_eq!(
                    object_outcomes,
                    vec![ObjectOutcome::External {
                        object_sha256: "abc".to_string()
                    }]
                );
            }
            other => panic!("expected Accepted, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn identical_revision_bytes_are_noop() {
        let importer = SidecarImporter::new(Arc::new(CannedProvider::with_state(
            SignatureVerificationState::Verified,
        )));
        let artifact = signed_artifact(
            b"payload",
            vec![ann("a1", 1, b"hello")],
            vec![],
        );
        let mut index = ImportIndex::new();
        index.index_annotation("a1", 1, b"hello".to_vec());

        match importer.classify(&artifact, &index).await.unwrap() {
            ImportOutcome::Accepted { annotation_outcomes, .. } => {
                assert_eq!(
                    annotation_outcomes,
                    vec![AnnotationOutcome::Noop {
                        annotation_id: "a1".to_string(),
                        revision: 1
                    }]
                );
            }
            other => panic!("expected Accepted, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn higher_revision_supersedes() {
        let importer = SidecarImporter::new(Arc::new(CannedProvider::with_state(
            SignatureVerificationState::Verified,
        )));
        let artifact = signed_artifact(
            b"payload",
            vec![ann("a1", 5, b"v5")],
            vec![],
        );
        let mut index = ImportIndex::new();
        index.index_annotation("a1", 3, b"v3".to_vec());

        match importer.classify(&artifact, &index).await.unwrap() {
            ImportOutcome::Accepted { annotation_outcomes, .. } => {
                assert_eq!(
                    annotation_outcomes,
                    vec![AnnotationOutcome::Create {
                        annotation_id: "a1".to_string(),
                        revision: 5,
                        supersedes: Some(3)
                    }]
                );
            }
            other => panic!("expected Accepted, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn lower_revision_is_stale() {
        let importer = SidecarImporter::new(Arc::new(CannedProvider::with_state(
            SignatureVerificationState::Verified,
        )));
        let artifact = signed_artifact(
            b"payload",
            vec![ann("a1", 1, b"v1")],
            vec![],
        );
        let mut index = ImportIndex::new();
        index.index_annotation("a1", 9, b"v9".to_vec());

        match importer.classify(&artifact, &index).await.unwrap() {
            ImportOutcome::Accepted { annotation_outcomes, .. } => {
                assert_eq!(
                    annotation_outcomes,
                    vec![AnnotationOutcome::Stale {
                        annotation_id: "a1".to_string(),
                        incoming_revision: 1,
                        existing_revision: 9
                    }]
                );
            }
            other => panic!("expected Accepted, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn divergent_equal_revisions_create_conflict() {
        let importer = SidecarImporter::new(Arc::new(CannedProvider::with_state(
            SignatureVerificationState::Verified,
        )));
        let artifact = signed_artifact(
            b"payload",
            vec![ann("a1", 1, b"incoming")],
            vec![],
        );
        let mut index = ImportIndex::new();
        index.index_annotation("a1", 1, b"existing".to_vec());

        match importer.classify(&artifact, &index).await.unwrap() {
            ImportOutcome::Accepted { annotation_outcomes, .. } => {
                assert_eq!(annotation_outcomes.len(), 1);
                match &annotation_outcomes[0] {
                    AnnotationOutcome::Conflict {
                        annotation_id,
                        revision,
                        existing_bytes,
                        incoming_bytes,
                    } => {
                        assert_eq!(annotation_id, "a1");
                        assert_eq!(*revision, 1);
                        assert_eq!(existing_bytes, b"existing");
                        assert_eq!(incoming_bytes, b"incoming");
                    }
                    other => panic!("expected Conflict, got {:?}", other),
                }
            }
            other => panic!("expected Accepted, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn importer_uses_signature_domain_prefix_when_verifying() {
        // Verify the importer forwards SIGNATURE_DOMAIN to the port
        // (the port is responsible for appending the NUL terminator per
        // §9.4 "signature domain prefix is UTF-8 `enzime.sidecar.v1\0`").
        struct CaptureProvider {
            captured: std::sync::Mutex<Option<String>>,
        }
        #[async_trait]
        impl SignatureProvider for CaptureProvider {
            async fn sign(
                &self,
                _domain: &str,
                _canonical_payload: &[u8],
            ) -> Result<SignatureEnvelope, SignatureError> {
                unreachable!()
            }
            async fn verify(
                &self,
                domain: &str,
                _canonical_payload: &[u8],
                _envelope: &SignatureEnvelope,
            ) -> Result<SignatureVerification, SignatureError> {
                *self.captured.lock().unwrap() = Some(domain.to_string());
                Ok(SignatureVerification {
                    state: SignatureVerificationState::Verified,
                    key_id: "device-1".to_string(),
                    signer_label: None,
                    explanation: "captured".to_string(),
                })
            }
        }
        let provider = Arc::new(CaptureProvider {
            captured: std::sync::Mutex::new(None),
        });
        let importer = SidecarImporter::new(provider.clone());
        let artifact = signed_artifact(b"payload", vec![], vec![]);
        let _ = importer.classify(&artifact, &ImportIndex::new()).await;

        assert_eq!(
            provider.captured.lock().unwrap().as_deref(),
            Some(SIGNATURE_DOMAIN)
        );
    }

    #[tokio::test]
    async fn exporter_signs_canonical_unsigned_bytes_and_derives_artifact_id() {
        struct CaptureProvider {
            captured: std::sync::Mutex<Option<(String, Vec<u8>)>>,
        }
        #[async_trait]
        impl SignatureProvider for CaptureProvider {
            async fn sign(
                &self,
                domain: &str,
                canonical_payload: &[u8],
            ) -> Result<SignatureEnvelope, SignatureError> {
                *self.captured.lock().unwrap() =
                    Some((domain.to_string(), canonical_payload.to_vec()));
                Ok(SignatureEnvelope {
                    algorithm: "ed25519".to_string(),
                    key_id: "device-1".to_string(),
                    signed_domain: domain.to_string(),
                    value: vec![0xCDu8; 64],
                })
            }
            async fn verify(
                &self,
                _: &str,
                _: &[u8],
                _: &SignatureEnvelope,
            ) -> Result<SignatureVerification, SignatureError> {
                unreachable!()
            }
        }
        let provider = Arc::new(CaptureProvider {
            captured: std::sync::Mutex::new(None),
        });
        let exporter = SidecarExporter::new(provider.clone());

        let payload = b"canonical unsigned envelope";
        let result = exporter.sign_export(payload).await.unwrap();
        assert_eq!(result.artifact_id, sha256_hex(payload));
        assert_eq!(result.signature.value, vec![0xCDu8; 64]);

        let captured = provider.captured.lock().unwrap().clone();
        assert_eq!(
            captured,
            Some((SIGNATURE_DOMAIN.to_string(), payload.to_vec()))
        );
    }

    #[tokio::test]
    async fn exporter_surfaces_signature_backend_errors() {
        struct FailingProvider;
        #[async_trait]
        impl SignatureProvider for FailingProvider {
            async fn sign(
                &self,
                _: &str,
                _: &[u8],
            ) -> Result<SignatureEnvelope, SignatureError> {
                Err(SignatureError::KeyUnavailable)
            }
            async fn verify(
                &self,
                _: &str,
                _: &[u8],
                _: &SignatureEnvelope,
            ) -> Result<SignatureVerification, SignatureError> {
                unreachable!()
            }
        }
        let exporter = SidecarExporter::new(Arc::new(FailingProvider));
        let err = exporter.sign_export(b"payload").await.unwrap_err();
        assert!(matches!(err, SidecarExportError::SignatureBackend(SignatureError::KeyUnavailable)));
    }

    #[test]
    fn import_index_tracks_seen_artifacts() {
        let mut idx = ImportIndex::new();
        assert!(!idx.has_artifact("abc"));
        idx.mark_artifact_seen("abc");
        assert!(idx.has_artifact("abc"));
    }

    #[test]
    fn sha256_hex_matches_known_vector() {
        // SHA-256 of the empty string — RFC 6234 test vector.
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
