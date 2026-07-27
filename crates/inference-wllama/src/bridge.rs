//! IPC transport abstraction for reaching the wllama3 instance that lives
//! inside the platform WebView. Ring 3 (`apps/reader/src-tauri`) is
//! responsible for implementing this trait against Tauri's WebView eval /
//! event channels (or whatever native equivalent the platform ships).
//!
//! No member of this trait performs business logic — they only carry typed
//! payloads across the language boundary. All policy (prompt formatting,
//! evidence marshalling, citation emission, usage accounting, embedding
//! normalization) lives in [`crate::backend`], [`crate::session`] and
//! [`crate::prompt`].
//!
//! The trait is intentionally concrete about cancellation (`abort` is a
//! first-class method) and streaming (`chat_start` returns a tokio
//! `mpsc::Receiver` so the implementer can fan in WebView events at its own
//! pace). It is intentionally vague about the URI scheme used to fetch
//! model bytes — that is platform-specific (Tauri uses `tauri://` and a
//! custom protocol; Android WebView may use `http://localhost`). What
//! matters is that the URI resolves, on the WebView side, to the same
//! file whose SHA-256 was just verified by [`crate::gguf`].

use ports::inference::InferenceError;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

/// Parameters that mirror the subset of wllama's `LoadModelParams` that this
/// crate actually needs. Every other field is left to the TypeScript side's
/// defaults.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WllamaLoadParams {
    /// Context size in tokens. Always sent because the manifest's
    /// `context_tokens` is the source of truth.
    pub n_ctx: u32,
    /// Whether this load is for an embedding-only session.
    pub embeddings: bool,
    /// Pooling type when `embeddings` is `true`. One of `"none"`, `"mean"`,
    /// `"cls"`, `"last"`, `"rank"`. `None` means let wllama decide from GGUF
    /// metadata.
    pub pooling_type: Option<String>,
    /// Random seed.
    pub seed: u64,
    /// Number of CPU threads. `0` means let wllama pick (defaults to half
    /// of `navigator.hardwareConcurrency`).
    pub n_threads: u32,
}

/// Information returned by wllama after a successful model load.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WllamaLoadedInfo {
    /// Effective context size, may be clamped by the runtime below `n_ctx`
    /// if memory is constrained.
    pub n_ctx: u32,
    /// Token ID for BOS, -1 if absent.
    pub token_bos: i32,
    /// Token ID for EOS, -1 if absent.
    pub token_eos: i32,
    /// Whether the multi-thread build was used.
    pub use_multi_thread: bool,
    /// Whether WebGPU is active for this load.
    pub use_webgpu: bool,
    /// Embedding dimensionality (0 when not loaded for embeddings).
    pub n_embd: u32,
}

/// Result of probing the runtime.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WllamaProbe {
    /// `true` when the WebView has a wllama module reachable for IPC.
    pub available: bool,
    /// Human-readable detail (libllama version, multi-thread support, ...).
    pub detail: String,
    /// `true` when WebGPU is usable in this WebView.
    pub webgpu_available: bool,
}

/// One streaming chat completion request. Field names mirror wllama's
/// `ChatCompletionParams` (snake_case) so the JSON payload can be forwarded
/// to `Wllama.createChatCompletion` with minimal transformation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WllamaChatParams {
    /// Pre-formatted chat messages from [`crate::prompt::format_chat_prompt`].
    pub messages: Vec<crate::ipc::BridgeChatMessage>,
    /// Sampling temperature.
    pub temperature: f32,
    /// Nucleus sampling probability.
    pub top_p: f32,
    /// Sampling seed.
    pub seed: u64,
    /// Maximum tokens to generate.
    pub max_tokens: u32,
    /// Stop sequences (in addition to EOS).
    pub stop: Vec<String>,
}

/// Stream events delivered by the bridge for one chat request. The bridge
/// must emit exactly one terminal event (`Done` or `Failed`) per
/// `request_id`, after which it closes the channel.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WllamaStreamEvent {
    /// One or more UTF-8 text deltas produced by the model.
    Delta {
        text: String,
    },
    /// Token usage reported by the runtime near completion.
    Usage {
        input_tokens: u32,
        output_tokens: u32,
    },
    /// Generation finished naturally (EOS, stop sequence, or length limit).
    Done {
        finish_reason: String,
    },
    /// Runtime reported an error.
    Failed {
        message: String,
    },
}

/// Transport into the WebView wllama instance. All methods are async because
/// every call crosses a process or language boundary.
///
/// # Error contract
///
/// Implementations map any transport-level failure to
/// [`InferenceError::BackendUnavailable`] or [`InferenceError::Runtime`].
/// `ModelIncompatible` is reserved for load failures that the runtime can
/// attribute to the model itself (bad metadata, missing tensors). `Cancelled`
/// is reserved for the abort path.
#[async_trait::async_trait]
pub trait WllamaBridge: Send + Sync {
    /// Low-cost liveness probe, used by
    /// [`crate::backend::WllamaInferenceBackend::health`]. Should complete in
    /// well under a second on a healthy WebView.
    async fn probe(&self) -> Result<WllamaProbe, InferenceError>;

    /// Load the model into a new wllama session. `model_uri` is whatever URI
    /// the WebView can fetch (e.g. a `tauri://` or `http://localhost` URL
    /// served by the host). The bridge must not retry on format errors.
    ///
    /// The caller has already verified the file's SHA-256 against the
    /// manifest; the bridge may trust that and skip its own integrity check.
    async fn load_model(
        &self,
        session_id: &str,
        model_uri: &str,
        params: WllamaLoadParams,
    ) -> Result<WllamaLoadedInfo, InferenceError>;

    /// Begin a streaming chat completion. The returned receiver delivers
    /// [`WllamaStreamEvent`]s tagged to `request_id` until the bridge emits
    /// `Done` or `Failed`, after which the channel is closed.
    ///
    /// The receiver buffer is implementation-defined; the caller pulls from
    /// it as fast as it can render.
    async fn chat_start(
        &self,
        session_id: &str,
        request_id: &str,
        params: WllamaChatParams,
    ) -> Result<mpsc::Receiver<WllamaStreamEvent>, InferenceError>;

    /// Abort an in-flight request. Returns `Ok(())` once the runtime has
    /// confirmed the abort (or confirms the request was already finished).
    /// Cancelling a finished or unknown request is **not** an error.
    async fn abort(
        &self,
        session_id: &str,
        request_id: &str,
    ) -> Result<(), InferenceError>;

    /// Compute embeddings for a batch of texts.
    async fn embed(
        &self,
        session_id: &str,
        batch: crate::ipc::BridgeEmbeddingBatch,
    ) -> Result<crate::ipc::BridgeEmbeddingResult, InferenceError>;

    /// Unload the model and free all runtime memory for this session. Must
    /// be idempotent (unloading an unknown session id is not an error).
    async fn unload(&self, session_id: &str) -> Result<(), InferenceError>;
}
