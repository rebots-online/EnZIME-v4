use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::inference::InferenceError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmbeddingDescriptor {
    pub embedding_space_id: String,
    pub model_id: String,
    pub dimensions: u32,
    pub distance_metric: String,
    pub normalized: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmbeddingInput {
    pub input_id: String,
    pub text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmbeddingOutput {
    pub input_id: String,
    pub vector: Vec<f32>,
}

#[async_trait]
pub trait EmbeddingBackend: Send + Sync {
    fn descriptor(&self) -> EmbeddingDescriptor;
    async fn embed(
        &self,
        inputs: Vec<EmbeddingInput>,
    ) -> Result<Vec<EmbeddingOutput>, InferenceError>;
}
