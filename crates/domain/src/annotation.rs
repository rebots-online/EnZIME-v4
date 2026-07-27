use crate::identity::{ArticleKey, EntryKey, PackId};
use async_trait::async_trait;
use bytes::Bytes;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SelectorBundle {
    pub article: ArticleSelector,
    pub text_quote: Option<TextQuoteSelector>,
    pub text_position: Option<TextPositionSelector>,
    pub dom_path: Option<DomPathSelector>,
    pub image_region: Option<ImageRegionSelector>,
    pub captured_context: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArticleSelector {
    pub pack_id: PackId,
    pub article_key: ArticleKey,
    pub source_revision_digest: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextQuoteSelector {
    pub exact: String,
    pub prefix: String,
    pub suffix: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextPositionSelector {
    pub normalized_start: u32,
    pub normalized_end: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomPathSelector {
    pub path: String,
    pub node_text_digest: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImageRegionSelector {
    pub resource_entry_key: EntryKey,
    pub x_fraction: f32,
    pub y_fraction: f32,
    pub width_fraction: f32,
    pub height_fraction: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnnotationType {
    Highlight,
    Comment,
    Ink,
    Voice,
    BookmarkNote,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnnotationDraft {
    pub annotation_type: AnnotationType,
    pub selector: SelectorBundle,
    pub body_text: Option<String>,
    pub style: AnnotationStyle,
    pub notebook_id: Option<String>,
    pub tag_ids: Vec<String>,
    pub sidecar_layer_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnnotationStyle {
    pub color_token: String,
    pub emphasis: String,
    pub stroke_width: Option<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnnotationRecord {
    pub annotation_id: String,
    pub annotation_type: AnnotationType,
    pub selector: SelectorBundle,
    pub body_text: Option<String>,
    pub style: AnnotationStyle,
    pub notebook_id: Option<String>,
    pub tags: Vec<TagSummary>,
    pub sidecar_layer_id: String,
    pub resolution: AnnotationResolution,
    pub pinned: bool,
    pub revision: u32,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TagSummary {
    pub tag_id: String,
    pub name: String,
    pub color_token: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnnotationResolution {
    pub state: AnnotationResolutionState,
    pub confidence: f32,
    pub matched_start: Option<u32>,
    pub matched_end: Option<u32>,
    pub explanation: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnnotationResolutionState {
    Resolved,
    Ambiguous,
    SourceUnavailable,
    Orphaned,
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum AnnotationError {
    #[error("annotation not found")]
    NotFound,
    #[error("annotation revision conflict")]
    RevisionConflict,
    #[error("source unavailable")]
    SourceUnavailable,
    #[error("selector invalid: {0}")]
    InvalidSelector(String),
    #[error("asset invalid: {0}")]
    InvalidAsset(String),
    #[error("storage error: {0}")]
    Storage(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnnotationAssetInput {
    pub asset_kind: String,
    pub mime_type: String,
    pub bytes: Bytes,
    pub duration_ms: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnnotationAssetRecord {
    pub annotation_asset_id: String,
    pub annotation_id: String,
    pub asset_kind: String,
    pub object_sha256: String,
    pub mime_type: String,
    pub byte_length: u64,
    pub duration_ms: Option<u64>,
}
