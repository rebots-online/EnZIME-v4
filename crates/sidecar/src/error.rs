//! Errors raised by the sidecar codec and import path.

#[derive(Debug, thiserror::Error)]
pub enum SidecarError {
    #[error("CBOR canonicalization error: {0}")]
    Cbor(#[from] crate::cbor::CborError),

    #[error("schema mismatch: expected {expected}, got {actual}")]
    SchemaMismatch { expected: String, actual: String },

    #[error("signature algorithm unsupported: {0}")]
    UnsupportedAlgorithm(String),

    #[error("signature signed_domain mismatch: expected {expected}, got {actual}")]
    SignedDomainMismatch { expected: String, actual: String },

    #[error("signature value has wrong length: expected {expected}, got {actual}")]
    SignatureLength { expected: usize, actual: usize },

    #[error("public key has wrong length: expected {expected}, got {actual}")]
    PublicKeyLength { expected: usize, actual: usize },

    #[error("artifact_id mismatch: envelope reports {reported}, recomputed {recomputed}")]
    ArtifactIdMismatch {
        reported: String,
        recomputed: String,
    },

    #[error("object digest mismatch for {object_sha256}: envelope reports {expected}, recomputed {actual}")]
    ObjectDigestMismatch {
        object_sha256: String,
        expected: String,
        actual: String,
    },

    #[error("artifact is unsigned; explicit user confirmation required")]
    Unsigned,

    #[error("artifact signature is invalid: {0}")]
    InvalidSignature(String),

    #[error("sidecar artifact already imported (artifact_sha256={0})")]
    Duplicate(String),

    #[error("sidecar map did not contain the expected key {key}")]
    MissingKey { key: &'static str },

    #[error("sidecar map contained unexpected key {key}")]
    UnexpectedKey { key: String },

    #[error("CDDL `uint` field {field} held a negative value {value}")]
    NegativeUint { field: &'static str, value: i64 },

    #[error("CDDL `bstr .size {expected}` field {field} held {actual} bytes")]
    ByteStringSize {
        field: &'static str,
        expected: usize,
        actual: usize,
    },
}
