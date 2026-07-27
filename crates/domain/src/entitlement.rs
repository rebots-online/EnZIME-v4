use serde::{Deserialize, Serialize};

pub const READER_CORE: &str = "reader.core";
pub const READER_UNLIMITED_PACKS: &str = "reader.unlimited_packs";
pub const ANNOTATIONS_RICH_MEDIA: &str = "annotations.rich_media";
pub const ASK_LOCAL_INFERENCE: &str = "ask.local_inference";
pub const MEMORY_LORE: &str = "memory.lore";
pub const BACKUP_ENCRYPTED: &str = "backup.encrypted";
pub const SIDECAR_SIGNED_EXPORT: &str = "sidecar.signed_export";
pub const MODELS_PREMIUM_CATALOG: &str = "models.premium_catalog";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CapabilityDecision {
    pub capability_key: String,
    pub allowed: bool,
    pub source: CapabilityDecisionSource,
    pub valid_until_ms: Option<i64>,
    pub explanation: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityDecisionSource {
    Core,
    SignedArtifact,
    GracePolicy,
    Denied,
}
