use async_trait::async_trait;
use futures_core::Stream;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::pin::Pin;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BackendDescriptor {
    pub backend_id: String,
    pub backend_kind: BackendKind,
    pub display_name: String,
    pub supported_formats: Vec<ModelFormat>,
    pub supports_streaming: bool,
    pub supports_embeddings: bool,
    pub accelerator: AcceleratorKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AcceleratorKind {
    Cpu,
    Gpu,
    Npu,
    Hybrid,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BackendHealth {
    pub backend_id: String,
    pub available: bool,
    pub accelerator: AcceleratorKind,
    pub detail: String,
    pub checked_at_ms: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelManifest {
    pub model_id: String,
    pub display_name: String,
    pub family: String,
    pub model_kind: ModelKind,
    pub format: ModelFormat,
    pub artifact_sha256: String,
    pub artifact_size_bytes: u64,
    pub context_tokens: u32,
    pub recommended_ram_bytes: u64,
    pub supported_backends: Vec<BackendKind>,
    pub prompt_template_id: String,
    pub license_spdx: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelKind {
    Chat,
    Embedding,
    Reranker,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionOptions {
    pub context_tokens: u32,
    pub max_output_tokens: u32,
    pub temperature: f32,
    pub top_p: f32,
    pub seed: u64,
    pub stop_sequences: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub request_id: String,
    pub system_prompt: String,
    pub messages: Vec<InferenceMessage>,
    pub evidence: Vec<InferenceEvidence>,
    pub options: SessionOptions,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InferenceMessage {
    pub role: String,
    pub body: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InferenceEvidence {
    pub citation_key: String,
    pub title: String,
    pub text: String,
    pub source_kind: String,
    pub source_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum InferenceChunk {
    TextDelta {
        text: String,
    },
    CitationMarker {
        citation_key: String,
    },
    Usage {
        input_tokens: u32,
        output_tokens: u32,
    },
    Completed {
        finish_reason: String,
    },
}

pub type InferenceStream =
    Pin<Box<dyn Stream<Item = Result<InferenceChunk, InferenceError>> + Send>>;

#[async_trait]
pub trait InferenceBackend: Send + Sync {
    fn descriptor(&self) -> BackendDescriptor;
    async fn health(&self) -> BackendHealth;
    async fn validate_model(
        &self,
        manifest: &ModelManifest,
        artifact_path: &Path,
    ) -> Result<(), InferenceError>;
    async fn open_session(
        &self,
        manifest: ModelManifest,
        artifact_path: &Path,
        options: SessionOptions,
    ) -> Result<Box<dyn InferenceSession>, InferenceError>;
}

#[async_trait]
pub trait InferenceSession: Send {
    fn session_id(&self) -> &str;
    fn model_id(&self) -> &str;
    async fn generate(
        &mut self,
        request: InferenceRequest,
    ) -> Result<InferenceStream, InferenceError>;
    async fn cancel(&mut self, request_id: &str) -> Result<(), InferenceError>;
    async fn close(self: Box<Self>) -> Result<(), InferenceError>;
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum InferenceError {
    #[error("backend unavailable")]
    BackendUnavailable,
    #[error("model not installed")]
    ModelNotInstalled,
    #[error("model incompatible: {0}")]
    ModelIncompatible(String),
    #[error("insufficient memory")]
    InsufficientMemory,
    #[error("context limit exceeded")]
    ContextLimitExceeded,
    #[error("request cancelled")]
    Cancelled,
    #[error("runtime failure: {0}")]
    Runtime(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BackendSelectionInput {
    pub platform: String,
    pub installed_models: Vec<ModelInstallationSummary>,
    pub backend_health: Vec<BackendHealth>,
    pub free_ram_bytes: u64,
    pub battery_saver: bool,
    pub thermal_state: String,
    pub user_preferred_backend_id: Option<String>,
    pub user_preferred_model_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelInstallationSummary {
    pub model_id: String,
    pub backend_id: String,
    pub active: bool,
    pub last_used_at_ms: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BackendSelection {
    pub backend_id: String,
    pub model_id: String,
    pub reason: String,
}
