use async_trait::async_trait;
use domain::lore::{LoreCandidate, LoreCorrection, LoreMemory, LoreQuery};

#[async_trait]
pub trait LoreService: Send + Sync {
    async fn extract_candidates(
        &self,
        conversation_id: &str,
        inference_run_id: &str,
    ) -> Result<Vec<LoreCandidate>, LoreError>;
    async fn list_inbox(
        &self,
        offset: u32,
        limit: u32,
    ) -> Result<Vec<LoreCandidate>, LoreError>;
    async fn accept(
        &self,
        lore_candidate_id: &str,
    ) -> Result<LoreMemory, LoreError>;
    async fn reject(
        &self,
        lore_candidate_id: &str,
    ) -> Result<(), LoreError>;
    async fn correct(
        &self,
        memory_id: &str,
        expected_revision: u32,
        correction: LoreCorrection,
    ) -> Result<LoreMemory, LoreError>;
    async fn set_pinned(
        &self,
        memory_id: &str,
        pinned: bool,
    ) -> Result<LoreMemory, LoreError>;
    async fn delete(
        &self,
        memory_id: &str,
        expected_revision: u32,
    ) -> Result<(), LoreError>;
    async fn query(
        &self,
        query: LoreQuery,
    ) -> Result<Vec<LoreMemory>, LoreError>;
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum LoreError {
    #[error("candidate not found")]
    CandidateNotFound,
    #[error("memory not found")]
    MemoryNotFound,
    #[error("revision conflict")]
    RevisionConflict,
    #[error("evidence missing")]
    EvidenceMissing,
    #[error("policy rejected proposal: {0}")]
    PolicyRejected(String),
    #[error("storage error: {0}")]
    Storage(String),
}
