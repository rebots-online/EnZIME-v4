#![forbid(unsafe_code)]

//! §10.5 model lifecycle — application-layer orchestration for the model
//! install state machine and the §10.5 model-removal policy.
//!
//! This module depends on `domain` and `ports` only (A-02). The inference,
//! signature and embedding providers are injected as trait objects — never
//! imported from a concrete provider crate.
//!
//! Per §26 AMD-2 the sole inference engine is wllama3; GGUF is the file
//! format it consumes. This module references `ModelFormat::Gguf` and
//! `BackendKind::LocalLm` from the inference port — it never names a
//! "litert"- or "gguf-engine"-flavoured entity.
//!
//! Per §10.5: no automatic model download, ever. Installation is always an
//! explicit user act — every entry into the `Downloading` state requires the
//! caller to have invoked [`ModelInstallation::begin_download`].

use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use domain::model::ModelLifecycleState;
use ports::catalog::CatalogEntry;
use ports::inference::{InferenceBackend, InferenceError, ModelKind, ModelManifest};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

// ===========================================================================
// §10.5 state machine — pure
// ===========================================================================

/// §10.5 transitions, encoded as a predicate. Returns `true` iff the
/// `from → to` edge exists in the state diagram.
///
/// Diagram edges (the only legal moves):
///   `Available   → Downloading`
///   `Downloading → Verifying`
///   `Verifying   → Installed | Quarantined`
///   `Installed   → Active | Removing`
///   `Active      → Installed`
///   `Removing    → Available`
pub fn allows(from: ModelLifecycleState, to: ModelLifecycleState) -> bool {
    use ModelLifecycleState::*;
    matches!(
        (from, to),
        (Available, Downloading)
            | (Downloading, Verifying)
            | (Verifying, Installed)
            | (Verifying, Quarantined)
            | (Installed, Active)
            | (Active, Installed)
            | (Installed, Removing)
            | (Removing, Available)
    )
}

/// Apply a §10.5 transition. Returns the new state on success, or
/// [`ModelLifecycleError::IllegalTransition`] if `(from, to)` is not an edge
/// of the diagram.
pub fn apply(
    from: ModelLifecycleState,
    to: ModelLifecycleState,
) -> Result<ModelLifecycleState, ModelLifecycleError> {
    if allows(from, to) {
        Ok(to)
    } else {
        Err(ModelLifecycleError::IllegalTransition { from, to })
    }
}

// ===========================================================================
// Records
// ===========================================================================

/// A model the user is managing locally. `state` is the source of truth —
/// every mutation goes through a transition method that delegates to
/// [`apply`], so an illegal move is rejected rather than silently corrupting
/// the record.
#[derive(Clone, Debug)]
pub struct ModelInstallation {
    pub model_id: String,
    pub manifest: ModelManifest,
    pub state: ModelLifecycleState,
    /// Absolute path of the downloaded artifact. Populated on
    /// [`begin_download`](Self::begin_download), retained through
    /// `Verifying` / `Installed` / `Active` / `Removing`, and cleared on
    /// [`complete_removal`](Self::complete_removal) — §10.5: "removes the
    /// installation location".
    pub artifact_path: Option<PathBuf>,
    /// Epoch-millis at which the record first reached `Installed`. Retained
    /// across removal per §10.5 ("retains manifest, run metadata and
    /// conversation content").
    pub installed_at_ms: Option<i64>,
    pub last_used_at_ms: Option<i64>,
    /// Catalog entry the artifact was installed from. Retained across
    /// removal for the same reason as `installed_at_ms`.
    pub source: CatalogEntry,
}

impl ModelInstallation {
    /// Create a fresh record in the `Available` state. No download is
    /// performed — §10.5 forbids automatic download.
    pub fn available(model_id: String, manifest: ModelManifest, source: CatalogEntry) -> Self {
        ModelInstallation {
            model_id,
            manifest,
            state: ModelLifecycleState::Available,
            artifact_path: None,
            installed_at_ms: None,
            last_used_at_ms: None,
            source,
        }
    }

    /// Advance `Available → Downloading` and record where the artifact will
    /// be staged. The caller is responsible for the byte transfer itself
    /// (via a `TransferSource`); this method only moves the state machine.
    pub fn begin_download(&mut self, artifact_path: PathBuf) -> Result<(), ModelLifecycleError> {
        apply(self.state, ModelLifecycleState::Downloading)?;
        self.artifact_path = Some(artifact_path);
        self.state = ModelLifecycleState::Downloading;
        Ok(())
    }

    /// Advance `Downloading → Verifying` once the byte transfer is complete.
    pub fn begin_verify(&mut self) -> Result<(), ModelLifecycleError> {
        self.state = apply(self.state, ModelLifecycleState::Verifying)?;
        Ok(())
    }

    /// Advance `Verifying → Installed` after successful verification,
    /// stamping the install timestamp.
    pub fn complete_verify(&mut self, at_ms: i64) -> Result<(), ModelLifecycleError> {
        let next = apply(self.state, ModelLifecycleState::Installed)?;
        self.installed_at_ms.get_or_insert(at_ms);
        self.state = next;
        Ok(())
    }

    /// Advance `Verifying → Quarantined` after failed verification. The
    /// artifact path is retained so the bad bytes can be inspected by
    /// diagnostics. §10.5 draws no edge out of `Quarantined` — recovery is
    /// an explicit user decision (typically: drop the record and start a
    /// fresh install attempt).
    pub fn quarantine(&mut self) -> Result<(), ModelLifecycleError> {
        self.state = apply(self.state, ModelLifecycleState::Quarantined)?;
        Ok(())
    }

    /// Advance `Installed → Active` and stamp `last_used_at_ms`.
    pub fn activate(&mut self, at_ms: i64) -> Result<(), ModelLifecycleError> {
        let next = apply(self.state, ModelLifecycleState::Active)?;
        self.last_used_at_ms = Some(at_ms);
        self.state = next;
        Ok(())
    }

    /// Advance `Active → Installed`.
    pub fn deactivate(&mut self) -> Result<(), ModelLifecycleError> {
        self.state = apply(self.state, ModelLifecycleState::Installed)?;
        Ok(())
    }

    /// Refresh `last_used_at_ms`. Permitted only when the model is
    /// `Installed` or `Active` — touching a model that has not finished
    /// installing (or is being removed) is rejected.
    pub fn touch(&mut self, at_ms: i64) -> Result<(), ModelLifecycleError> {
        match self.state {
            ModelLifecycleState::Installed | ModelLifecycleState::Active => {
                self.last_used_at_ms = Some(at_ms);
                Ok(())
            }
            other => Err(ModelLifecycleError::NotInService { state: other }),
        }
    }

    /// Advance `Installed → Removing`. The artifact path is left in place so
    /// the remover knows what to delete; it is cleared by `complete_removal`.
    pub fn begin_removal(&mut self) -> Result<(), ModelLifecycleError> {
        self.state = apply(self.state, ModelLifecycleState::Removing)?;
        Ok(())
    }

    /// Advance `Removing → Available`. Drops the local artifact path but
    /// retains the manifest, source catalog entry and timestamps per §10.5.
    /// The caller receives a [`ModelRemovalOutcome`] describing what was
    /// done and what is now pending.
    pub fn complete_removal(&mut self) -> Result<ModelRemovalOutcome, ModelLifecycleError> {
        let next = apply(self.state, ModelLifecycleState::Available)?;
        let artifact_removed = self.artifact_path.take().is_some();
        let embeddings_pending_rebuild = self.manifest.model_kind == ModelKind::Embedding;
        self.state = next;
        Ok(ModelRemovalOutcome {
            model_id: self.model_id.clone(),
            artifact_removed,
            manifest_retained: true,
            embeddings_pending_rebuild,
        })
    }
}

/// Result of a successful removal, surfaced to the caller so it can honour
/// the §10.5 rule "schedules embeddings from that model for deletion/rebuild
/// only after another embedding space is selected". The caller must NOT drop
/// embedding vectors until the user picks an alternative embedding space.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModelRemovalOutcome {
    pub model_id: String,
    pub artifact_removed: bool,
    pub manifest_retained: bool,
    pub embeddings_pending_rebuild: bool,
}

// ===========================================================================
// Errors
// ===========================================================================

#[derive(Clone, Debug, Error)]
pub enum ModelLifecycleError {
    #[error("illegal lifecycle transition: {from:?} -> {to:?}")]
    IllegalTransition {
        from: ModelLifecycleState,
        to: ModelLifecycleState,
    },
    #[error("model not in service (state: {state:?})")]
    NotInService { state: ModelLifecycleState },
    #[error("verification failed: {0}")]
    VerificationFailed(String),
    #[error("io error: {0}")]
    Io(String),
    #[error("inference backend error: {0}")]
    Inference(#[from] InferenceError),
    #[error("active-session drain failed: {0}")]
    SessionDrain(String),
    #[error("embedding rebuild scheduling failed: {0}")]
    EmbeddingSchedule(String),
    #[error("removal cancelled by user")]
    RemovalCancelled,
}

// ===========================================================================
// Artifact verification — the `Verifying` state's payload
// ===========================================================================

/// Performs the §10.5 verification step that gates `Verifying → Installed`.
///
/// Verification is the conjunction of:
///   1. Streaming SHA-256 of the artifact bytes equals
///      `manifest.artifact_sha256`.
///   2. The injected inference backend accepts the artifact via
///      `validate_model` — the port-side check that the bytes form a loadable
///      GGUF model for wllama3 (§26 AMD-2).
///
/// I-10(b): signature verification against a fetched `SignatureEnvelope` is
/// not performed here. The canonical-payload bytes for a model signature are
/// not nailed down by §10.5, and `SignatureProvider::verify` requires a
/// pre-fetched envelope. That wiring will land with the signature-domain
/// task; until then this verifier covers the two checks the spec actually
/// defines for the `Verifying` state.
pub struct ModelArtifactVerifier {
    inference: Arc<dyn InferenceBackend>,
}

impl ModelArtifactVerifier {
    pub fn new(inference: Arc<dyn InferenceBackend>) -> Self {
        ModelArtifactVerifier { inference }
    }

    pub async fn verify(
        &self,
        manifest: &ModelManifest,
        artifact_path: &Path,
    ) -> Result<(), ModelLifecycleError> {
        let computed = sha256_hex_of(artifact_path).await?;
        if computed != manifest.artifact_sha256 {
            return Err(ModelLifecycleError::VerificationFailed(format!(
                "sha256 mismatch: manifest={} computed={}",
                manifest.artifact_sha256, computed
            )));
        }
        self.inference
            .validate_model(manifest, artifact_path)
            .await?;
        Ok(())
    }
}

async fn sha256_hex_of(path: &Path) -> Result<String, ModelLifecycleError> {
    let mut file = File::open(path)
        .await
        .map_err(|e| ModelLifecycleError::Io(e.to_string()))?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = file
            .read(&mut buf)
            .await
            .map_err(|e| ModelLifecycleError::Io(e.to_string()))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

// ===========================================================================
// §10.5 removal policy
// ===========================================================================

/// I-10(b): swap when the active-session-registry task lands. The trait is
/// exactly what §10.5 ("waits for active sessions to close or user
/// cancellation") requires — a way to count and to await the drain of
/// inference sessions bound to a model — but no concrete implementation
/// exists in `ports` yet.
#[async_trait]
pub trait ActiveSessionRegistry: Send + Sync {
    /// Number of currently-open inference sessions bound to `model_id`.
    async fn active_session_count(&self, model_id: &str) -> Result<usize, ModelLifecycleError>;

    /// Resolves once the active-session count for `model_id` reaches zero.
    /// The registry chooses its own polling/notify strategy. The caller is
    /// responsible for racing this future against a user-cancellation
    /// signal — §10.5 permits either outcome.
    async fn await_drained(&self, model_id: &str) -> Result<(), ModelLifecycleError>;
}

/// I-10(b): swap when the embedding-rebuild-planner task lands. The trait
/// encodes the §10.5 rule "schedules embeddings from that model for
/// deletion/rebuild only after another embedding space is selected":
/// schedule does not delete, and deletion is performed only after a
/// replacement space has been chosen.
#[async_trait]
pub trait EmbeddingRebuildPlanner: Send + Sync {
    /// Mark embedding vectors originating from `removed_model_id` as
    /// pending rebuild. Vectors remain queryable until
    /// [`rebuild_with_alternative`](Self::rebuild_with_alternative) is
    /// invoked — §10.5: deletion happens "only after another embedding
    /// space is selected".
    async fn schedule_rebuild(
        &self,
        removed_model_id: &str,
    ) -> Result<(), ModelLifecycleError>;

    /// Drop the vectors belonging to `removed_model_id` and rebuild them
    /// under `replacement_model_id`. Caller must have selected
    /// `replacement_model_id` as the active embedding space first.
    async fn rebuild_with_alternative(
        &self,
        removed_model_id: &str,
        replacement_model_id: &str,
    ) -> Result<(), ModelLifecycleError>;
}

/// §10.5 removal policy. Composes the two injected sentinels with the
/// state-machine record so the full set of removal rules is enforced in one
/// place.
///
/// The gate does NOT own a clock or a cancellation token — both are caller
/// responsibilities, which keeps the gate deterministic in tests.
pub struct RemovalGate {
    sessions: Arc<dyn ActiveSessionRegistry>,
    embeddings: Arc<dyn EmbeddingRebuildPlanner>,
}

impl RemovalGate {
    pub fn new(
        sessions: Arc<dyn ActiveSessionRegistry>,
        embeddings: Arc<dyn EmbeddingRebuildPlanner>,
    ) -> Self {
        RemovalGate { sessions, embeddings }
    }

    /// §10.5: "waits for active sessions to close or user cancellation". The
    /// caller races this future against a user-cancellation signal; whichever
    /// wins determines whether the removal proceeds.
    pub async fn await_sessions_closed(
        &self,
        model_id: &str,
    ) -> Result<(), ModelLifecycleError> {
        self.sessions.await_drained(model_id).await
    }

    /// §10.5 removal finalization. Requires the record to already be in the
    /// `Removing` state (caller invoked
    /// [`ModelInstallation::begin_removal`] and raced
    /// [`await_sessions_closed`](Self::await_sessions_closed) against user
    /// cancellation). Performs:
    ///   - filesystem removal of the local artifact (rule: "removes the
    ///     installation location");
    ///   - when the model was an embedding model, scheduling of the
    ///     embedding vectors for rebuild (rule: "schedules embeddings ...
    ///     only after another embedding space is selected");
    ///   - state transition `Removing → Available`.
    ///
    /// The rule "retains manifest, run metadata and conversation content" is
    /// honoured by the state-machine layer: [`ModelInstallation`] keeps
    /// `manifest`, `source`, `installed_at_ms` and `last_used_at_ms` across
    /// the `Removing → Available` transition. The rule "invalidates no
    /// citation" is a consequence of the same retention — citations live in
    /// the annotation store, which this gate never touches.
    pub async fn finalize(
        &self,
        installation: &mut ModelInstallation,
    ) -> Result<ModelRemovalOutcome, ModelLifecycleError> {
        if installation.state != ModelLifecycleState::Removing {
            return Err(ModelLifecycleError::IllegalTransition {
                from: installation.state,
                to: ModelLifecycleState::Available,
            });
        }

        if let Some(path) = installation.artifact_path.as_ref() {
            tokio::fs::remove_file(path)
                .await
                .map_err(|e| ModelLifecycleError::Io(e.to_string()))?;
        }

        if installation.manifest.model_kind == ModelKind::Embedding {
            self.embeddings
                .schedule_rebuild(&installation.model_id)
                .await?;
        }

        installation.complete_removal()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ----- pure state machine -------------------------------------------

    #[test]
    fn allows_only_diagram_edges() {
        use ModelLifecycleState::*;
        let cases: Vec<(ModelLifecycleState, ModelLifecycleState, bool)> = vec![
            (Available, Downloading, true),
            (Downloading, Verifying, true),
            (Verifying, Installed, true),
            (Verifying, Quarantined, true),
            (Installed, Active, true),
            (Active, Installed, true),
            (Installed, Removing, true),
            (Removing, Available, true),
            // non-edges
            (Available, Installed, false),
            (Available, Verifying, false),
            (Available, Active, false),
            (Downloading, Installed, false),
            (Downloading, Available, false),
            (Installed, Available, false),
            (Active, Available, false),
            (Active, Removing, false),
            (Quarantined, Available, false),
            (Quarantined, Verifying, false),
            (Quarantined, Installed, false),
            (Removing, Installed, false),
            (Removing, Quarantined, false),
        ];
        for (from, to, expected) in cases {
            assert_eq!(
                allows(from, to),
                expected,
                "allows({from:?}, {to:?}) should be {expected}"
            );
        }
    }

    #[test]
    fn apply_returns_illegal_transition_for_non_edges() {
        use ModelLifecycleState::*;
        let err = apply(Available, Installed).unwrap_err();
        match err {
            ModelLifecycleError::IllegalTransition { from, to } => {
                assert_eq!(from, Available);
                assert_eq!(to, Installed);
            }
            other => panic!("expected IllegalTransition, got {other:?}"),
        }
    }

    // ----- record state-machine methods ---------------------------------

    fn fixture_record() -> ModelInstallation {
        ModelInstallation::available(
            "tinyllama-1b".into(),
            ModelManifest {
                model_id: "tinyllama-1b".into(),
                display_name: "TinyLlama 1B".into(),
                family: "llama".into(),
                model_kind: ModelKind::Chat,
                format: ports::inference::ModelFormat::Gguf,
                artifact_sha256: "ab".repeat(32),
                artifact_size_bytes: 1024,
                context_tokens: 2048,
                recommended_ram_bytes: 2048 * 1024 * 1024,
                supported_backends: vec![ports::inference::BackendKind::LocalLm],
                prompt_template_id: "llama3".into(),
                license_spdx: "Apache-2.0".into(),
            },
            CatalogEntry {
                catalog_entry_id: "ce-1".into(),
                corpus_id: "models".into(),
                upstream_version: "1".into(),
                title: "TinyLlama 1B".into(),
                description: "small chat model".into(),
                language_bcp47: "en".into(),
                category: "chat".into(),
                publisher: "test".into(),
                artifact_url: "https://example.invalid/tinyllama.gguf".into(),
                artifact_size_bytes: 1024,
                artifact_sha256: "ab".repeat(32),
                signature_url: None,
                trust: ports::catalog::TrustPresentation {
                    state: "unsigned".into(),
                    signer: None,
                    explanation: "test fixture".into(),
                },
            },
        )
    }

    #[test]
    fn record_starts_available() {
        let r = fixture_record();
        assert_eq!(r.state, ModelLifecycleState::Available);
        assert!(r.artifact_path.is_none());
        assert!(r.installed_at_ms.is_none());
    }

    #[test]
    fn record_happy_path_install_activate_remove() {
        let mut r = fixture_record();
        r.begin_download("/tmp/dummy".into()).unwrap();
        assert_eq!(r.state, ModelLifecycleState::Downloading);
        assert_eq!(
            r.artifact_path.as_ref().map(|p| p.to_string_lossy().to_string()),
            Some("/tmp/dummy".into())
        );

        r.begin_verify().unwrap();
        assert_eq!(r.state, ModelLifecycleState::Verifying);

        r.complete_verify(1700_000_000).unwrap();
        assert_eq!(r.state, ModelLifecycleState::Installed);
        assert_eq!(r.installed_at_ms, Some(1700_000_000));

        r.activate(1700_000_500).unwrap();
        assert_eq!(r.state, ModelLifecycleState::Active);
        assert_eq!(r.last_used_at_ms, Some(1700_000_500));

        r.touch(1700_000_900).unwrap();
        assert_eq!(r.last_used_at_ms, Some(1700_000_900));

        r.deactivate().unwrap();
        assert_eq!(r.state, ModelLifecycleState::Installed);

        r.begin_removal().unwrap();
        assert_eq!(r.state, ModelLifecycleState::Removing);
        assert!(r.artifact_path.is_some());

        let outcome = r.complete_removal().unwrap();
        assert_eq!(r.state, ModelLifecycleState::Available);
        assert!(r.artifact_path.is_none());
        assert_eq!(
            outcome,
            ModelRemovalOutcome {
                model_id: "tinyllama-1b".into(),
                artifact_removed: true,
                manifest_retained: true,
                embeddings_pending_rebuild: false,
            }
        );
        // §10.5: manifest / timestamps are retained across removal.
        assert_eq!(r.manifest.model_id, "tinyllama-1b");
        assert_eq!(r.installed_at_ms, Some(1700_000_000));
    }

    #[test]
    fn record_quarantine_then_no_direct_recovery() {
        let mut r = fixture_record();
        r.begin_download("/tmp/dummy".into()).unwrap();
        r.begin_verify().unwrap();
        r.quarantine().unwrap();
        assert_eq!(r.state, ModelLifecycleState::Quarantined);
        // §10.5 draws no edge out of Quarantined — every transition is rejected.
        assert!(matches!(
            apply(ModelLifecycleState::Quarantined, ModelLifecycleState::Installed),
            Err(ModelLifecycleError::IllegalTransition { .. })
        ));
        assert!(matches!(
            apply(ModelLifecycleState::Quarantined, ModelLifecycleState::Available),
            Err(ModelLifecycleError::IllegalTransition { .. })
        ));
    }

    #[test]
    fn record_rejects_out_of_order_moves() {
        let mut r = fixture_record();
        // cannot verify before download
        assert!(matches!(
            r.begin_verify(),
            Err(ModelLifecycleError::IllegalTransition { .. })
        ));
        // cannot activate from Available
        assert!(matches!(
            r.activate(0),
            Err(ModelLifecycleError::IllegalTransition { .. })
        ));
        // touch fails from Available
        assert!(matches!(
            r.touch(0),
            Err(ModelLifecycleError::NotInService { .. })
        ));
    }
}
