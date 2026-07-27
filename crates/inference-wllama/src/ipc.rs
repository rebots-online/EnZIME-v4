//! JSON shapes that travel across the [`crate::bridge::WllamaBridge`]. Kept in
//! one place so the contract with the TypeScript side (vendored at
//! `VENDOR/wllama3/src/`) is reviewable in isolation. Field names mirror
//! wllama's snake_case naming so payloads can be forwarded to
//! `Wllama.createChatCompletion` / `Wllama.createEmbedding` with minimal
//! transformation.

use serde::{Deserialize, Serialize};

/// One chat message exchanged with wllama.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BridgeChatMessage {
    /// One of `"system"`, `"user"`, `"assistant"`. Unknown roles are
    /// coerced to `"user"` by the prompt formatter before they reach here.
    pub role: String,
    /// Plain-text content. The TS side is responsible for applying the
    /// model's jinja chat template.
    pub content: String,
}

/// A batch of plain-text inputs to embed. Order is preserved on the way back.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BridgeEmbeddingBatch {
    pub inputs: Vec<String>,
}

/// Result returned by the bridge after embedding a batch.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BridgeEmbeddingResult {
    /// One vector per input, in request order.
    pub vectors: Vec<Vec<f32>>,
    /// Dimensionality of each vector. Must be uniform.
    pub dimensions: u32,
}
