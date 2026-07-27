use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoreCandidate {
    pub lore_candidate_id: String,
    pub statement: String,
    pub subject_key: String,
    pub predicate_key: String,
    pub object: serde_json::Value,
    pub confidence: f32,
    pub evidence: Vec<LoreEvidence>,
    pub created_at_ms: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoreEvidence {
    pub lore_evidence_id: String,
    pub source_kind: String,
    pub source_id: String,
    pub excerpt: String,
    pub locator: serde_json::Value,
    pub content_digest: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoreMemory {
    pub memory_id: String,
    pub subject_key: String,
    pub predicate_key: String,
    pub statement: String,
    pub object: serde_json::Value,
    pub revision: u32,
    pub pinned: bool,
    pub status: LoreMemoryStatus,
    pub evidence: Vec<LoreEvidence>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoreMemoryStatus {
    Active,
    Superseded,
    Deleted,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoreCorrection {
    pub statement: String,
    pub object: serde_json::Value,
    pub correction_reason: String,
    pub evidence_ids: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoreQuery {
    pub text: String,
    pub subject_keys: Vec<String>,
    pub include_deleted: bool,
    pub limit: u32,
}
