//! wllama3 inference backend — the sole local inference engine on every
//! platform (AMENDMENT §26 AMD-2).
//!
//! wllama3 is a WebAssembly binding for llama.cpp that runs inside the
//! platform WebView. This crate provides the Rust-side implementation of the
//! [`ports::inference::InferenceBackend`],
//! [`ports::inference::InferenceSession`] and
//! [`ports::embedding::EmbeddingBackend`] traits. The Rust side owns:
//!
//! - GGUF artifact verification (magic + size + SHA-256) — see [`gguf`].
//! - Prompt and evidence marshalling — see [`prompt`].
//! - Request lifecycle, streaming, cancellation — see [`session`].
//! - Backend descriptor + health reporting — see [`backend`].
//!
//! The TypeScript side (vendored at `VENDOR/wllama3/`) owns llama.cpp
//! invocation; this crate reaches it through the [`bridge::WllamaBridge`]
//! trait that Ring 3 wires to its concrete WebView IPC transport.
//!
//! No LiteRT-LM or `inference-gguf` engine exists in this crate — AMD-2
//! retired both. GGUF survives only as the file format
//! ([`ports::inference::ModelFormat::Gguf`]) that wllama3 consumes.

#![forbid(unsafe_code)]

pub mod backend;
pub mod bridge;
pub mod clock;
pub mod gguf;
pub mod ipc;
pub mod prompt;
pub mod session;

pub use backend::{WllamaEmbeddingBackend, WllamaInferenceBackend, BACKEND_ID};
pub use bridge::{
    WllamaBridge, WllamaChatParams, WllamaLoadedInfo, WllamaLoadParams, WllamaProbe,
    WllamaStreamEvent,
};
pub use gguf::{verify_gguf_artifact, GGUF_MAGIC, VerifyError};
pub use ipc::{BridgeChatMessage, BridgeEmbeddingBatch, BridgeEmbeddingResult};
pub use prompt::format_chat_prompt;
pub use session::WllamaInferenceSession;
