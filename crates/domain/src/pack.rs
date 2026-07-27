//! Pack identities, kind and verification state — durable pack entities from
//! §8.1. These live in the domain crate because they are durable value objects
//! independent of any replaceable engine.

use crate::identity::{PackId, PackVersionId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PackManifest {
    pub pack_id: PackId,
    pub corpus_id: String,
    pub display_name: String,
    pub description: String,
    pub language_bcp47: String,
    pub pack_kind: PackKind,
    pub versions: Vec<PackVersionSummary>,
    pub active_version_id: Option<PackVersionId>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackKind {
    FullZim,
    TailoredZim,
    EnzimePack,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PackVersionSummary {
    pub pack_version_id: PackVersionId,
    pub upstream_version: String,
    pub artifact_sha256: String,
    pub artifact_size_bytes: u64,
    pub verification_state: VerificationState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationState {
    Unverified,
    Verified,
    Quarantined,
}
