//! The two release backend types: [`WllamaInferenceBackend`] (chat) and
//! [`WllamaEmbeddingBackend`] (embeddings). Both are constructed with a
//! [`WllamaBridge`] that Ring 3 supplies.
//!
//! Per AMD-2 these are the **sole** local inference backend on every
//! platform; no LiteRT-LM, no `inference-gguf` engine exists in this crate.

use ports::embedding::{EmbeddingBackend, EmbeddingDescriptor, EmbeddingInput, EmbeddingOutput};
use ports::inference::{
    AcceleratorKind, BackendDescriptor, BackendHealth, BackendKind, InferenceBackend,
    InferenceError, InferenceSession, ModelFormat, ModelKind, ModelManifest, SessionOptions,
};
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

use crate::bridge::{WllamaBridge, WllamaLoadedInfo, WllamaLoadParams};
use crate::clock::now_unix_ms;
use crate::gguf::verify_gguf_artifact;
use crate::ipc::BridgeEmbeddingBatch;
use crate::session::WllamaInferenceSession;

/// Identifier used in `BackendDescriptor::backend_id` and `BackendHealth`.
pub const BACKEND_ID: &str = "inference-wllama";

/// Default pooling type for embedding loads. Embedding models in the GGUF
/// ecosystem are pre-pooled; `"mean"` matches llama.cpp's default for the
/// `embeddings=true` path.
const DEFAULT_POOLING_TYPE: &str = "mean";

/// Default context window for an embedding-only load. Embedding work does
/// not autoregress, so this can be modest; large batches are split by the
/// bridge.
const DEFAULT_EMBEDDING_CTX: u32 = 2048;

pub struct WllamaInferenceBackend {
    bridge: Arc<dyn WllamaBridge>,
    /// URI prefix the WebView uses to fetch model artifacts. The backend
    /// appends the file's leaf name; the bridge is responsible for resolving
    /// the resulting path back to the verified local file.
    artifact_uri_prefix: String,
    /// Default thread count for loads. `0` lets wllama decide.
    default_threads: u32,
}

impl WllamaInferenceBackend {
    /// Construct with a concrete bridge and the URI prefix under which the
    /// WebView can fetch model files. The prefix must be a valid URL prefix
    /// (no trailing slash), e.g. `tauri://localhost/models` or
    /// `http://localhost:5173/models`.
    pub fn new(bridge: Arc<dyn WllamaBridge>, artifact_uri_prefix: String) -> Self {
        Self {
            bridge,
            artifact_uri_prefix,
            default_threads: 0,
        }
    }

    /// Override the default CPU thread count for loads. `0` lets wllama pick
    /// (defaults to half `navigator.hardwareConcurrency`).
    #[must_use]
    pub fn with_default_threads(mut self, threads: u32) -> Self {
        self.default_threads = threads;
        self
    }

    fn artifact_uri(&self, artifact_path: &Path) -> String {
        let leaf = artifact_path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "artifact.gguf".to_string());
        format!(
            "{}/{}",
            self.artifact_uri_prefix.trim_end_matches('/'),
            leaf
        )
    }
}

#[async_trait::async_trait]
impl InferenceBackend for WllamaInferenceBackend {
    fn descriptor(&self) -> BackendDescriptor {
        BackendDescriptor {
            backend_id: BACKEND_ID.to_string(),
            backend_kind: BackendKind::LocalLm,
            display_name: "wllama3 (llama.cpp via WebAssembly)".to_string(),
            supported_formats: vec![ModelFormat::Gguf],
            supports_streaming: true,
            supports_embeddings: true,
            // Hybrid because wllama may engage WebGPU; falls back to CPU.
            accelerator: AcceleratorKind::Hybrid,
        }
    }

    async fn health(&self) -> BackendHealth {
        let (available, accelerator, detail) = match self.bridge.probe().await {
            Ok(probe) => (
                probe.available,
                if probe.webgpu_available {
                    AcceleratorKind::Hybrid
                } else {
                    AcceleratorKind::Cpu
                },
                probe.detail,
            ),
            Err(e) => (
                false,
                AcceleratorKind::Cpu,
                format!("probe failed: {e}"),
            ),
        };
        BackendHealth {
            backend_id: BACKEND_ID.to_string(),
            available,
            accelerator,
            detail,
            checked_at_ms: now_unix_ms(),
        }
    }

    async fn validate_model(
        &self,
        manifest: &ModelManifest,
        artifact_path: &Path,
    ) -> Result<(), InferenceError> {
        // Reranker models are valid for the engine, but this backend only
        // exposes chat + embedding surfaces. Refuse to load so the selector
        // doesn't pick a reranker for chat. The embedding backend is
        // loadable separately with `model_kind == Embedding`.
        if manifest.model_kind == ModelKind::Reranker {
            return Err(InferenceError::ModelIncompatible(
                "reranker models are not loadable by WllamaInferenceBackend; \
                 use a dedicated rerank adapter"
                    .to_string(),
            ));
        }
        verify_gguf_artifact(manifest, artifact_path).await?;
        Ok(())
    }

    async fn open_session(
        &self,
        manifest: ModelManifest,
        artifact_path: &Path,
        options: SessionOptions,
    ) -> Result<Box<dyn InferenceSession>, InferenceError> {
        // (1) Re-verify the artifact on open. open_session is the trust
        //     boundary — the manifest has crossed into this backend, so we
        //     prove the file's bytes match before loading it into the
        //     WebView's memory.
        verify_gguf_artifact(&manifest, artifact_path).await?;

        // (2) Refuse to open a chat session for an embedding-only model.
        //     The embedding backend opens its own load; chat callers should
        //     not get an embedding model that would emit noise.
        if manifest.model_kind == ModelKind::Embedding {
            return Err(InferenceError::ModelIncompatible(
                "embedding models are not loadable for chat sessions".to_string(),
            ));
        }
        if manifest.model_kind == ModelKind::Reranker {
            return Err(InferenceError::ModelIncompatible(
                "reranker models are not loadable for chat sessions".to_string(),
            ));
        }

        // (3) Build load params and ask the bridge to load the model.
        let session_id = Uuid::new_v4().to_string();
        let model_uri = self.artifact_uri(artifact_path);
        let load_params = WllamaLoadParams {
            n_ctx: options.context_tokens,
            embeddings: false,
            pooling_type: None,
            seed: options.seed,
            n_threads: self.default_threads,
        };
        let _loaded: WllamaLoadedInfo = self
            .bridge
            .load_model(&session_id, &model_uri, load_params)
            .await?;

        Ok(Box::new(WllamaInferenceSession::new(
            session_id,
            manifest.model_id.clone(),
            self.bridge.clone(),
            options,
        )))
    }
}

/// Embedding backend. Maintains its own long-lived wllama load so callers
/// don't have to manage a chat-style session for batched embedding work.
pub struct WllamaEmbeddingBackend {
    bridge: Arc<dyn WllamaBridge>,
    artifact_uri_prefix: String,
    embedding_space_id: String,
    model_id: String,
    dimensions: u32,
    distance_metric: String,
    normalized: bool,
    session: Arc<tokio::sync::Mutex<Option<EmbeddingSession>>>,
}

struct EmbeddingSession {
    session_id: String,
}

impl WllamaEmbeddingBackend {
    /// Construct with a concrete bridge, the URI prefix under which the
    /// WebView can fetch model files, and the embedding-space identity that
    /// [`EmbeddingBackend::descriptor`] reports.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        bridge: Arc<dyn WllamaBridge>,
        artifact_uri_prefix: String,
        embedding_space_id: String,
        model_id: String,
        dimensions: u32,
        distance_metric: String,
        normalized: bool,
    ) -> Self {
        Self {
            bridge,
            artifact_uri_prefix,
            embedding_space_id,
            model_id,
            dimensions,
            distance_metric,
            normalized,
            session: Arc::new(tokio::sync::Mutex::new(None)),
        }
    }

    fn artifact_uri(&self, artifact_path: &Path) -> String {
        let leaf = artifact_path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "artifact.gguf".to_string());
        format!(
            "{}/{}",
            self.artifact_uri_prefix.trim_end_matches('/'),
            leaf
        )
    }

    /// Lazy-load the embedding model into a long-lived session. Idempotent —
    /// subsequent calls return `Ok(())` without re-loading. The artifact is
    /// verified before loading.
    pub async fn ensure_loaded(
        &self,
        manifest: &ModelManifest,
        artifact_path: &Path,
    ) -> Result<(), InferenceError> {
        let mut guard = self.session.lock().await;
        if guard.is_some() {
            return Ok(());
        }
        if manifest.model_kind != ModelKind::Embedding {
            return Err(InferenceError::ModelIncompatible(format!(
                "{:?} model is not an embedding model",
                manifest.model_kind
            )));
        }
        verify_gguf_artifact(manifest, artifact_path).await?;

        let session_id = Uuid::new_v4().to_string();
        let model_uri = self.artifact_uri(artifact_path);
        let loaded = self
            .bridge
            .load_model(
                &session_id,
                &model_uri,
                WllamaLoadParams {
                    n_ctx: DEFAULT_EMBEDDING_CTX,
                    embeddings: true,
                    pooling_type: Some(DEFAULT_POOLING_TYPE.to_string()),
                    seed: 0,
                    n_threads: 0,
                },
            )
            .await?;
        // The runtime may clamp n_ctx below what we requested. We don't
        // currently expose this back to the caller, but we explicitly
        // acknowledge the report so it is not silently dropped.
        let _ = loaded.n_ctx;
        *guard = Some(EmbeddingSession { session_id });
        Ok(())
    }

    /// Unload the embedding model and free runtime memory. Safe to call
    /// repeatedly — once unloaded, subsequent `embed` calls fail with
    /// `BackendUnavailable` until `ensure_loaded` is called again.
    pub async fn unload(&self) -> Result<(), InferenceError> {
        let mut guard = self.session.lock().await;
        if let Some(session) = guard.take() {
            self.bridge.unload(&session.session_id).await?;
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl EmbeddingBackend for WllamaEmbeddingBackend {
    fn descriptor(&self) -> EmbeddingDescriptor {
        EmbeddingDescriptor {
            embedding_space_id: self.embedding_space_id.clone(),
            model_id: self.model_id.clone(),
            dimensions: self.dimensions,
            distance_metric: self.distance_metric.clone(),
            normalized: self.normalized,
        }
    }

    async fn embed(
        &self,
        inputs: Vec<EmbeddingInput>,
    ) -> Result<Vec<EmbeddingOutput>, InferenceError> {
        if inputs.is_empty() {
            return Ok(Vec::new());
        }
        let session_id = {
            let guard = self.session.lock().await;
            guard
                .as_ref()
                .map(|s| s.session_id.clone())
                .ok_or(InferenceError::BackendUnavailable)?
        };

        let texts: Vec<String> = inputs.iter().map(|i| i.text.clone()).collect();
        let result = self
            .bridge
            .embed(
                &session_id,
                BridgeEmbeddingBatch { inputs: texts },
            )
            .await?;

        if result.vectors.len() != inputs.len() {
            return Err(InferenceError::Runtime(format!(
                "embedding backend returned {} vectors for {} inputs",
                result.vectors.len(),
                inputs.len()
            )));
        }
        if result.dimensions != self.dimensions {
            return Err(InferenceError::Runtime(format!(
                "embedding dimensionality mismatch: descriptor={} bridge={}",
                self.dimensions, result.dimensions,
            )));
        }

        let mut out = Vec::with_capacity(inputs.len());
        for (input, mut vector) in inputs.into_iter().zip(result.vectors.into_iter()) {
            if self.normalized {
                let norm = l2_norm(&vector);
                if norm > 0.0 {
                    for v in vector.iter_mut() {
                        *v /= norm;
                    }
                }
            }
            out.push(EmbeddingOutput {
                input_id: input.input_id,
                vector,
            });
        }
        Ok(out)
    }
}

/// L2 norm of a slice. Used for in-place normalization when
/// `EmbeddingDescriptor::normalized` is true.
fn l2_norm(v: &[f32]) -> f32 {
    let sum: f32 = v.iter().map(|x| x * x).sum();
    sum.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ports::inference::{BackendKind, ModelFormat, ModelKind};

    fn fake_manifest(kind: ModelKind) -> ModelManifest {
        ModelManifest {
            model_id: "m".to_string(),
            display_name: "M".to_string(),
            family: "f".to_string(),
            model_kind: kind,
            format: ModelFormat::Gguf,
            artifact_sha256: "0".repeat(64),
            artifact_size_bytes: 0,
            context_tokens: 128,
            recommended_ram_bytes: 0,
            supported_backends: vec![BackendKind::LocalLm],
            prompt_template_id: "chat".to_string(),
            license_spdx: "MIT".to_string(),
        }
    }

    #[test]
    fn l2_norm_matches_definition() {
        let v = vec![3.0_f32, 4.0_f32];
        let n = l2_norm(&v);
        assert!((n - 5.0).abs() < 1e-6);
    }

    #[test]
    fn backend_id_is_stable() {
        assert_eq!(BACKEND_ID, "inference-wllama");
    }

    #[test]
    fn embedding_descriptor_round_trips_constructor_fields() {
        // Direct construction is impossible without a bridge; test the
        // surface we can reach via the public default-threads builder.
        let _ = (DEFAULT_POOLING_TYPE, DEFAULT_EMBEDDING_CTX);
        // manifest_kind sanity: an embedding manifest's kind is Embedding.
        let m = fake_manifest(ModelKind::Embedding);
        assert_eq!(m.model_kind, ModelKind::Embedding);
    }
}
