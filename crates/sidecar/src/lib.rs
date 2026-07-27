//! Sidecar CBOR codec, signing and import — ARCHITECTURE.md §9.4–§9.6.
//!
//! Three layers:
//!   - [`cbor`]: minimal canonical RFC 8949 deterministic encoder/decoder
//!     restricted to the subset the sidecar CDDL uses.
//!   - [`codec`]: `SidecarV1` (de)serialization plus `artifact_id` derivation.
//!   - [`provider`]: `Ed25519Provider` implementing
//!     `ports::signature::SignatureProvider` with the sidecar domain prefix.
//!   - [`import`]: declarative §9.6 conflict resolution.
//!
//! All four are pure (no I/O); the only state mutation in this crate happens
//! inside `Ed25519Provider`'s in-memory keystore.

#![forbid(unsafe_code)]

pub mod cbor;
pub mod codec;
pub mod error;
pub mod import;
pub mod provider;
pub mod types;

pub use cbor::{decode as decode_cbor_value, encode as encode_cbor_value, Value};
pub use codec::{
    artifact_id, decode, encode_into, encode_to_vec, encode_unsigned_signing_form, sha256_hex,
};
pub use error::SidecarError;
pub use import::{
    canonical_annotation_bytes, classify_import, finalize_signed_envelope, AnnotationOutcome,
    ConfirmationReason, ImportIndex, ImportVerdict, ObjectOutcome, QuarantineReason,
};
pub use provider::{domain_prefix, Ed25519Keystore, Ed25519Provider};
pub use types::{
    AnnotationType, DeviceRef, Layer, LayerVisibility, Notebook, ObjectRef, SidecarAnnotation,
    SidecarSignature, SidecarV1, SourceBinding, Tag, ALGORITHM_ED25519, FILE_EXTENSION,
    MEDIA_TYPE, PUBLIC_KEY_LEN, SCHEMA_ID, SIGNATURE_DOMAIN, SIGNATURE_LEN,
};
