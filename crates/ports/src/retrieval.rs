use async_trait::async_trait;
use domain::identity::{ArticleKey, PackId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RetrievalQuery {
    pub text: String,
    pub scope: RetrievalScope,
    pub limit: u32,
    pub context_budget_tokens: u32,
    pub include_source_kinds: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RetrievalScope {
    Passage {
        retrieval_chunk_id: String,
    },
    Article {
        pack_id: PackId,
        article_key: ArticleKey,
    },
    Packs {
        pack_ids: Vec<PackId>,
    },
    Notebook {
        notebook_id: String,
    },
    AllLocal,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RetrievalHit {
    pub retrieval_chunk_id: String,
    pub source_kind: String,
    pub source_id: String,
    pub title: String,
    pub text: String,
    pub locator: CitationLocator,
    pub lexical_score: Option<f32>,
    pub vector_score: Option<f32>,
    pub rerank_score: f32,
    pub token_count: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CitationLocator {
    pub pack_id: Option<PackId>,
    pub article_key: Option<ArticleKey>,
    pub annotation_id: Option<String>,
    pub start_char: u32,
    pub end_char: u32,
    pub quote_digest: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContextEvidence {
    pub citation_key: String,
    pub retrieval_chunk_id: String,
    pub title: String,
    pub text: String,
    pub source_kind: String,
    pub source_id: String,
    pub locator: CitationLocator,
    pub tokens: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContextSet {
    pub evidence: Vec<ContextEvidence>,
    pub used_tokens: u32,
    pub budget_tokens: u32,
    pub omitted_count: u32,
}

#[async_trait]
pub trait Retriever: Send + Sync {
    async fn retrieve(
        &self,
        query: RetrievalQuery,
    ) -> Result<Vec<RetrievalHit>, RetrievalError>;
}

#[async_trait]
pub trait ContextBuilder: Send + Sync {
    async fn build(
        &self,
        query: &RetrievalQuery,
        hits: Vec<RetrievalHit>,
    ) -> Result<ContextSet, RetrievalError>;
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum RetrievalError {
    #[error("scope invalid: {0}")]
    InvalidScope(String),
    #[error("source unavailable")]
    SourceUnavailable,
    #[error("index unavailable")]
    IndexUnavailable,
    #[error("embedding space unavailable")]
    EmbeddingUnavailable,
    #[error("storage error: {0}")]
    Storage(String),
}
