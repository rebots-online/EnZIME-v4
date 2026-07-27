//! Rust representation of the sidecar CDDL data model (ARCHITECTURE.md §9.4).
//!
//! Every struct field name, enum variant name, and serde rename below
//! transcribes the CDDL exactly. Renaming, reordering, adding or dropping
//! a field here would diverge from the binding contract.
//!
//! `selector` and `style` are CDDL `any`; the canonical sidecar codec
//! rejects floats and tags, so they are typed as [`cbor::Value`] which
//! models exactly the subset the CDDL permits.

use crate::cbor::Value;

/// Media type for sidecar artifacts (ARCHITECTURE.md §9.4 "Media type").
pub const MEDIA_TYPE: &str = "application/vnd.enzime.sidecar+cbor";

/// File extension for sidecar artifacts (ARCHITECTURE.md §9.4 "File extension").
pub const FILE_EXTENSION: &str = ".ezsc";

/// Schema discriminator stored in every sidecar envelope.
pub const SCHEMA_ID: &str = "enzime.sidecar.cbor.v1";

/// Domain separation string used while signing sidecar envelopes
/// (ARCHITECTURE.md §9.4 "Canonicalization": "signature domain prefix is
/// UTF-8 `enzime.sidecar.v1\0`"). The NUL terminator is part of the prefix
/// and is included in the bytes that get signed.
pub const SIGNATURE_DOMAIN: &str = "enzime.sidecar.v1";

/// Fixed byte length of an Ed25519 verifying key — matches the CDDL
/// `bstr .size 32` constraint on `device-ref.public_key`.
pub const PUBLIC_KEY_LEN: usize = 32;

/// Fixed byte length of an Ed25519 raw signature — matches the CDDL
/// `bstr .size 64` constraint on `signature.value`.
pub const SIGNATURE_LEN: usize = 64;

/// Algorithm identifier that must appear in every signed sidecar envelope.
pub const ALGORITHM_ED25519: &str = "ed25519";

/// Visibility of a layer (ARCHITECTURE.md §9.4 `layer.visibility`).
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayerVisibility {
    Private,
    Shared,
    Imported,
}

/// Type of an annotation (ARCHITECTURE.md §9.4 `annotation.annotation_type`).
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnnotationType {
    Highlight,
    Comment,
    Ink,
    Voice,
    BookmarkNote,
}

/// `device-ref` (ARCHITECTURE.md §9.4).
///
/// `public_key` is a 32-byte Ed25519 verifying key. Stored as a `Vec<u8>`
/// rather than a `[u8; 32]` so that `SidecarV1` and friends can derive
/// Serde for diagnostic JSON output (Rust's built-in array `Deserialize`
/// impls stop at length 32). The codec still validates the length against
/// `PUBLIC_KEY_LEN` at encode and decode time.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DeviceRef {
    pub device_id: String,
    pub public_key: Vec<u8>,
}

/// `layer` (ARCHITECTURE.md §9.4).
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Layer {
    pub layer_id: String,
    pub name: String,
    pub visibility: LayerVisibility,
}

/// `source-binding` (ARCHITECTURE.md §9.4).
#[derive(Clone, Debug, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SourceBinding {
    pub pack_id: String,
    pub corpus_id: String,
    pub artifact_sha256: String,
}

/// `annotation` (ARCHITECTURE.md §9.4).
///
/// `selector` and `style` are CDDL `any` and are kept as [`Value`], which
/// models exactly the canonical subset (no floats, no tags).
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SidecarAnnotation {
    pub annotation_id: String,
    pub annotation_type: AnnotationType,
    pub revision: u32,
    pub selector: Value,
    pub body_text: Option<String>,
    pub style: Value,
    pub notebook_id: Option<String>,
    pub tag_ids: Vec<String>,
    pub asset_ids: Vec<String>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

/// `notebook` (ARCHITECTURE.md §9.4).
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Notebook {
    pub notebook_id: String,
    pub parent_notebook_id: Option<String>,
    pub name: String,
    pub description: String,
}

/// `tag` (ARCHITECTURE.md §9.4).
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Tag {
    pub tag_id: String,
    pub name: String,
    pub color_token: String,
}

/// `object-ref` (ARCHITECTURE.md §9.4).
///
/// When `embedded_bytes` is `Some(b)`, `b` is the literal object content
/// carried inside the sidecar so an offline recipient can verify and store
/// it without an external fetch. The import path hashes `b` and compares
/// the digest to `object_sha256` before committing.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ObjectRef {
    pub object_sha256: String,
    pub mime_type: String,
    pub byte_length: u64,
    pub embedded_bytes: Option<Vec<u8>>,
}

/// `signature` (ARCHITECTURE.md §9.4). Stored as `Vec<u8>` for the same
/// Serde-compatibility reason as `DeviceRef::public_key`; the codec
/// validates the length against `SIGNATURE_LEN` at encode and decode time.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SidecarSignature {
    pub algorithm: String,
    pub key_id: String,
    pub signed_domain: String,
    pub value: Vec<u8>,
}

/// Top-level `enzime-sidecar-v1` envelope (ARCHITECTURE.md §9.4).
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SidecarV1 {
    pub schema: String,
    pub artifact_id: String,
    pub created_at_ms: i64,
    pub creator: DeviceRef,
    pub layer: Layer,
    pub source_bindings: Vec<SourceBinding>,
    pub annotations: Vec<SidecarAnnotation>,
    pub notebooks: Vec<Notebook>,
    pub tags: Vec<Tag>,
    pub objects: Vec<ObjectRef>,
    pub signature: SidecarSignature,
}

impl PartialEq for SidecarV1 {
    /// Two envelopes are equal iff their canonical bytes are equal.
    ///
    /// Deriving from canonical bytes (not field-wise) keeps equality aligned
    /// with what gets hashed for `artifact_id` and what gets signed, so a
    /// byte-identical re-encoding is always equal to the original.
    fn eq(&self, other: &Self) -> bool {
        let mut a = Vec::new();
        let mut b = Vec::new();
        crate::codec::encode_into(&mut a, self);
        crate::codec::encode_into(&mut b, other);
        a == b
    }
}

impl Eq for SidecarV1 {}
