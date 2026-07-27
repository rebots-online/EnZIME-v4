//! Canonical encoder and decoder for `SidecarV1` and the `artifact_id`
//! derivation that ARCHITECTURE.md §9.4 mandates.
//!
//! ## The signing form
//!
//! Two canonicalization rules from §9.4 work together:
//!   - "signature value omitted while signing"
//!   - "`artifact_id` is SHA-256 of canonical unsigned envelope"
//!
//! The "unsigned envelope" is the envelope with both of these fields blanked:
//!   - `artifact_id` is set to the empty string, so the SHA-256 input does
//!     not depend on the hash itself (no chicken-and-egg);
//!   - the `signature` sub-map is emitted WITHOUT the `value` entry, so the
//!     signing payload excludes the bytes the signer is about to produce.
//!
//! Both the producer and the verifier reconstruct these same bytes; the
//! producer hashes-and-signs them, the verifier hashes-and-checks-against
//! `envelope.artifact_id` and verifies the signature over them. Round-trip
//! determinism (see `cbor::tests::deterministic_encoding_is_byte_identical`)
//! is what makes the two sides agree.

use sha2::{Digest, Sha256};

use crate::cbor::{decode as decode_cbor, encode, text_map, Value};
use crate::error::SidecarError;
use crate::types::{
    AnnotationType, DeviceRef, Layer, LayerVisibility, Notebook, ObjectRef, SidecarAnnotation,
    SidecarSignature, SidecarV1, SourceBinding, Tag, ALGORITHM_ED25519, SCHEMA_ID,
    SIGNATURE_DOMAIN, SIGNATURE_LEN,
};

/// Encode a full envelope (signature value included) as canonical CBOR.
pub fn encode_to_vec(env: &SidecarV1) -> Vec<u8> {
    let mut out = Vec::new();
    encode_into(&mut out, env);
    out
}

/// Encode a full envelope (signature value included) as canonical CBOR into `out`.
pub fn encode_into(out: &mut Vec<u8>, env: &SidecarV1) {
    encode(&encode_top_level(env, /* include_sig_value */ true), out);
}

/// Encode the canonical *unsigned* form that gets hashed (for `artifact_id`)
/// and signed (after the domain prefix is prepended).
///
/// See the module docs for the exact rules.
pub fn encode_unsigned_signing_form(env: &SidecarV1) -> Vec<u8> {
    // The signing form is the canonical envelope with `artifact_id` blanked
    // (so the SHA-256 input does not depend on the hash itself) and the
    // signature sub-map emitted WITHOUT its `value` entry (per
    // "signature value omitted while signing"). The producer hashes these
    // bytes before the artifact_id is known; the verifier must reconstruct
    // the same bytes, so the transformation is purely functional.
    let mut blanked = env.clone();
    blanked.artifact_id.clear();
    let mut out = Vec::new();
    encode(
        &encode_top_level(&blanked, /* include_sig_value */ false),
        &mut out,
    );
    out
}

/// Compute `artifact_id` (lowercase hex SHA-256 of the canonical unsigned
/// envelope) per ARCHITECTURE.md §9.4.
pub fn artifact_id(env: &SidecarV1) -> String {
    let bytes = encode_unsigned_signing_form(env);
    hex::encode(Sha256::digest(&bytes))
}

/// Decode canonical CBOR bytes into a `SidecarV1`. Rejects non-canonical
/// encoding (preferred serialization, sorted map keys, no floats) and
/// schema violations (`schema` field value, structural shape).
pub fn decode(input: &[u8]) -> Result<SidecarV1, SidecarError> {
    let value = decode_cbor(input)?;
    decode_top_level(&value)
}

/// SHA-256 of canonical bytes, returned as lowercase hex. Used both for
/// `artifact_id` derivation and for the idempotency key `artifact_sha256`
/// that §9.6 prescribes for import deduplication.
pub fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn encode_top_level(env: &SidecarV1, include_sig_value: bool) -> Value {
    text_map(&[
        ("schema", Value::Text(env.schema.clone())),
        ("artifact_id", Value::Text(env.artifact_id.clone())),
        ("created_at_ms", signed_int(env.created_at_ms)),
        ("creator", encode_device_ref(&env.creator)),
        ("layer", encode_layer(&env.layer)),
        (
            "source_bindings",
            Value::Array(env.source_bindings.iter().map(encode_source_binding).collect()),
        ),
        (
            "annotations",
            Value::Array(env.annotations.iter().map(encode_annotation).collect()),
        ),
        (
            "notebooks",
            Value::Array(env.notebooks.iter().map(encode_notebook).collect()),
        ),
        ("tags", Value::Array(env.tags.iter().map(encode_tag).collect())),
        (
            "objects",
            Value::Array(env.objects.iter().map(encode_object_ref).collect()),
        ),
        ("signature", encode_signature(&env.signature, include_sig_value)),
    ])
}

fn encode_device_ref(d: &DeviceRef) -> Value {
    text_map(&[
        ("device_id", Value::Text(d.device_id.clone())),
        ("public_key", Value::Bytes(d.public_key.clone())),
    ])
}

fn encode_layer(l: &Layer) -> Value {
    text_map(&[
        ("layer_id", Value::Text(l.layer_id.clone())),
        ("name", Value::Text(l.name.clone())),
        ("visibility", encode_visibility(l.visibility)),
    ])
}

fn encode_visibility(v: LayerVisibility) -> Value {
    Value::Text(
        match v {
            LayerVisibility::Private => "private",
            LayerVisibility::Shared => "shared",
            LayerVisibility::Imported => "imported",
        }
        .to_string(),
    )
}

fn encode_source_binding(s: &SourceBinding) -> Value {
    text_map(&[
        ("pack_id", Value::Text(s.pack_id.clone())),
        ("corpus_id", Value::Text(s.corpus_id.clone())),
        ("artifact_sha256", Value::Text(s.artifact_sha256.clone())),
    ])
}

fn encode_annotation(a: &SidecarAnnotation) -> Value {
    text_map(&[
        ("annotation_id", Value::Text(a.annotation_id.clone())),
        (
            "annotation_type",
            Value::Text(encode_annotation_type(a.annotation_type).to_string()),
        ),
        ("revision", Value::Unsigned(a.revision as u64)),
        ("selector", a.selector.clone()),
        (
            "body_text",
            match &a.body_text {
                Some(s) => Value::Text(s.clone()),
                None => Value::Null,
            },
        ),
        ("style", a.style.clone()),
        (
            "notebook_id",
            match &a.notebook_id {
                Some(s) => Value::Text(s.clone()),
                None => Value::Null,
            },
        ),
        (
            "tag_ids",
            Value::Array(a.tag_ids.iter().map(|s| Value::Text(s.clone())).collect()),
        ),
        (
            "asset_ids",
            Value::Array(a.asset_ids.iter().map(|s| Value::Text(s.clone())).collect()),
        ),
        ("created_at_ms", signed_int(a.created_at_ms)),
        ("updated_at_ms", signed_int(a.updated_at_ms)),
    ])
}

fn encode_annotation_type(t: AnnotationType) -> &'static str {
    match t {
        AnnotationType::Highlight => "highlight",
        AnnotationType::Comment => "comment",
        AnnotationType::Ink => "ink",
        AnnotationType::Voice => "voice",
        AnnotationType::BookmarkNote => "bookmark_note",
    }
}

fn encode_notebook(n: &Notebook) -> Value {
    text_map(&[
        ("notebook_id", Value::Text(n.notebook_id.clone())),
        (
            "parent_notebook_id",
            match &n.parent_notebook_id {
                Some(s) => Value::Text(s.clone()),
                None => Value::Null,
            },
        ),
        ("name", Value::Text(n.name.clone())),
        ("description", Value::Text(n.description.clone())),
    ])
}

fn encode_tag(t: &Tag) -> Value {
    text_map(&[
        ("tag_id", Value::Text(t.tag_id.clone())),
        ("name", Value::Text(t.name.clone())),
        ("color_token", Value::Text(t.color_token.clone())),
    ])
}

fn encode_object_ref(o: &ObjectRef) -> Value {
    text_map(&[
        ("object_sha256", Value::Text(o.object_sha256.clone())),
        ("mime_type", Value::Text(o.mime_type.clone())),
        ("byte_length", Value::Unsigned(o.byte_length)),
        (
            "embedded_bytes",
            match &o.embedded_bytes {
                Some(b) => Value::Bytes(b.clone()),
                None => Value::Null,
            },
        ),
    ])
}

fn encode_signature(s: &SidecarSignature, include_value: bool) -> Value {
    let mut entries: Vec<(&str, Value)> = vec![
        ("algorithm", Value::Text(s.algorithm.clone())),
        ("key_id", Value::Text(s.key_id.clone())),
        ("signed_domain", Value::Text(s.signed_domain.clone())),
    ];
    if include_value {
        entries.push(("value", Value::Bytes(s.value.clone())));
    }
    text_map(&entries)
}

/// Map a Rust `i64` into the canonical CBOR integer: non-negative values
/// become major type 0, negative values become major type 1. The encoder
/// handles preferred serialization (smallest AI width) for both.
fn signed_int(v: i64) -> Value {
    if v >= 0 {
        Value::Unsigned(v as u64)
    } else {
        Value::Negative(v)
    }
}

fn decode_top_level(v: &Value) -> Result<SidecarV1, SidecarError> {
    let entries = expect_map(v)?;
    let schema = take_text_field(entries, "schema")?;
    if schema != SCHEMA_ID {
        return Err(SidecarError::SchemaMismatch {
            expected: SCHEMA_ID.to_string(),
            actual: schema,
        });
    }
    let artifact_id = take_text_field(entries, "artifact_id")?;
    let created_at_ms = take_int_field(entries, "created_at_ms")?;
    let creator = decode_device_ref(take_field(entries, "creator")?)?;
    let layer = decode_layer(take_field(entries, "layer")?)?;
    let source_bindings = decode_array(take_field(entries, "source_bindings")?)?
        .iter()
        .map(decode_source_binding)
        .collect::<Result<Vec<_>, _>>()?;
    let annotations = decode_array(take_field(entries, "annotations")?)?
        .iter()
        .map(decode_annotation)
        .collect::<Result<Vec<_>, _>>()?;
    let notebooks = decode_array(take_field(entries, "notebooks")?)?
        .iter()
        .map(decode_notebook)
        .collect::<Result<Vec<_>, _>>()?;
    let tags = decode_array(take_field(entries, "tags")?)?
        .iter()
        .map(decode_tag)
        .collect::<Result<Vec<_>, _>>()?;
    let objects = decode_array(take_field(entries, "objects")?)?
        .iter()
        .map(decode_object_ref)
        .collect::<Result<Vec<_>, _>>()?;
    let signature = decode_signature(take_field(entries, "signature")?)?;
    Ok(SidecarV1 {
        schema,
        artifact_id,
        created_at_ms,
        creator,
        layer,
        source_bindings,
        annotations,
        notebooks,
        tags,
        objects,
        signature,
    })
}

fn expect_map(v: &Value) -> Result<&[(Value, Value)], SidecarError> {
    match v {
        Value::Map(entries) => Ok(entries.as_slice()),
        _ => Err(SidecarError::MissingKey { key: "<top-level map>" }),
    }
}

fn take_field<'a>(entries: &'a [(Value, Value)], key: &'static str) -> Result<&'a Value, SidecarError> {
    entries
        .iter()
        .find(|(k, _)| matches!(k, Value::Text(s) if s == key))
        .map(|(_, v)| v)
        .ok_or(SidecarError::MissingKey { key })
}

fn take_text_field(entries: &[(Value, Value)], key: &'static str) -> Result<String, SidecarError> {
    match take_field(entries, key)? {
        Value::Text(s) => Ok(s.clone()),
        _ => Err(SidecarError::MissingKey { key }),
    }
}

fn take_int_field(entries: &[(Value, Value)], key: &'static str) -> Result<i64, SidecarError> {
    match take_field(entries, key)? {
        Value::Unsigned(n) => Ok(*n as i64),
        Value::Negative(n) => Ok(*n),
        _ => Err(SidecarError::MissingKey { key }),
    }
}

fn decode_array(v: &Value) -> Result<&[Value], SidecarError> {
    match v {
        Value::Array(items) => Ok(items.as_slice()),
        _ => Err(SidecarError::MissingKey { key: "<array>" }),
    }
}

fn decode_device_ref(v: &Value) -> Result<DeviceRef, SidecarError> {
    let entries = expect_map(v)?;
    let device_id = take_text_field(entries, "device_id")?;
    let public_key_bytes = match take_field(entries, "public_key")? {
        Value::Bytes(b) => b.clone(),
        _ => return Err(SidecarError::MissingKey { key: "public_key" }),
    };
    if public_key_bytes.len() != crate::types::PUBLIC_KEY_LEN {
        return Err(SidecarError::ByteStringSize {
            field: "public_key",
            expected: crate::types::PUBLIC_KEY_LEN,
            actual: public_key_bytes.len(),
        });
    }
    Ok(DeviceRef {
        device_id,
        public_key: public_key_bytes,
    })
}

fn decode_layer(v: &Value) -> Result<Layer, SidecarError> {
    let entries = expect_map(v)?;
    Ok(Layer {
        layer_id: take_text_field(entries, "layer_id")?,
        name: take_text_field(entries, "name")?,
        visibility: decode_visibility(take_field(entries, "visibility")?)?,
    })
}

fn decode_visibility(v: &Value) -> Result<LayerVisibility, SidecarError> {
    match v {
        Value::Text(s) if s == "private" => Ok(LayerVisibility::Private),
        Value::Text(s) if s == "shared" => Ok(LayerVisibility::Shared),
        Value::Text(s) if s == "imported" => Ok(LayerVisibility::Imported),
        _ => Err(SidecarError::MissingKey { key: "visibility" }),
    }
}

fn decode_source_binding(v: &Value) -> Result<SourceBinding, SidecarError> {
    let entries = expect_map(v)?;
    Ok(SourceBinding {
        pack_id: take_text_field(entries, "pack_id")?,
        corpus_id: take_text_field(entries, "corpus_id")?,
        artifact_sha256: take_text_field(entries, "artifact_sha256")?,
    })
}

fn decode_annotation(v: &Value) -> Result<SidecarAnnotation, SidecarError> {
    let entries = expect_map(v)?;
    let annotation_type =
        decode_annotation_type(take_field(entries, "annotation_type")?)?;
    let revision = match take_field(entries, "revision")? {
        Value::Unsigned(n) => Ok(*n as u32),
        Value::Negative(_) => Err(SidecarError::NegativeUint {
            field: "revision",
            value: 0,
        }),
        _ => Err(SidecarError::MissingKey { key: "revision" }),
    }?;
    let body_text = match take_field(entries, "body_text")? {
        Value::Text(s) => Some(s.clone()),
        Value::Null => None,
        _ => return Err(SidecarError::MissingKey { key: "body_text" }),
    };
    let notebook_id = match take_field(entries, "notebook_id")? {
        Value::Text(s) => Some(s.clone()),
        Value::Null => None,
        _ => return Err(SidecarError::MissingKey { key: "notebook_id" }),
    };
    let tag_ids = decode_string_array(take_field(entries, "tag_ids")?)?;
    let asset_ids = decode_string_array(take_field(entries, "asset_ids")?)?;
    Ok(SidecarAnnotation {
        annotation_id: take_text_field(entries, "annotation_id")?,
        annotation_type,
        revision,
        selector: take_field(entries, "selector")?.clone(),
        body_text,
        style: take_field(entries, "style")?.clone(),
        notebook_id,
        tag_ids,
        asset_ids,
        created_at_ms: take_int_field(entries, "created_at_ms")?,
        updated_at_ms: take_int_field(entries, "updated_at_ms")?,
    })
}

fn decode_annotation_type(v: &Value) -> Result<AnnotationType, SidecarError> {
    match v {
        Value::Text(s) if s == "highlight" => Ok(AnnotationType::Highlight),
        Value::Text(s) if s == "comment" => Ok(AnnotationType::Comment),
        Value::Text(s) if s == "ink" => Ok(AnnotationType::Ink),
        Value::Text(s) if s == "voice" => Ok(AnnotationType::Voice),
        Value::Text(s) if s == "bookmark_note" => Ok(AnnotationType::BookmarkNote),
        _ => Err(SidecarError::MissingKey { key: "annotation_type" }),
    }
}

fn decode_string_array(v: &Value) -> Result<Vec<String>, SidecarError> {
    let arr = decode_array(v)?;
    arr.iter()
        .map(|x| match x {
            Value::Text(s) => Ok(s.clone()),
            _ => Err(SidecarError::MissingKey { key: "<string element>" }),
        })
        .collect()
}

fn decode_notebook(v: &Value) -> Result<Notebook, SidecarError> {
    let entries = expect_map(v)?;
    let parent_notebook_id = match take_field(entries, "parent_notebook_id")? {
        Value::Text(s) => Some(s.clone()),
        Value::Null => None,
        _ => return Err(SidecarError::MissingKey { key: "parent_notebook_id" }),
    };
    Ok(Notebook {
        notebook_id: take_text_field(entries, "notebook_id")?,
        parent_notebook_id,
        name: take_text_field(entries, "name")?,
        description: take_text_field(entries, "description")?,
    })
}

fn decode_tag(v: &Value) -> Result<Tag, SidecarError> {
    let entries = expect_map(v)?;
    Ok(Tag {
        tag_id: take_text_field(entries, "tag_id")?,
        name: take_text_field(entries, "name")?,
        color_token: take_text_field(entries, "color_token")?,
    })
}

fn decode_object_ref(v: &Value) -> Result<ObjectRef, SidecarError> {
    let entries = expect_map(v)?;
    let byte_length = match take_field(entries, "byte_length")? {
        Value::Unsigned(n) => *n,
        Value::Negative(_) => {
            return Err(SidecarError::NegativeUint {
                field: "byte_length",
                value: 0,
            })
        }
        _ => return Err(SidecarError::MissingKey { key: "byte_length" }),
    };
    let embedded_bytes = match take_field(entries, "embedded_bytes")? {
        Value::Bytes(b) => Some(b.clone()),
        Value::Null => None,
        _ => return Err(SidecarError::MissingKey { key: "embedded_bytes" }),
    };
    Ok(ObjectRef {
        object_sha256: take_text_field(entries, "object_sha256")?,
        mime_type: take_text_field(entries, "mime_type")?,
        byte_length,
        embedded_bytes,
    })
}

fn decode_signature(v: &Value) -> Result<SidecarSignature, SidecarError> {
    let entries = expect_map(v)?;
    let algorithm = take_text_field(entries, "algorithm")?;
    let key_id = take_text_field(entries, "key_id")?;
    let signed_domain = take_text_field(entries, "signed_domain")?;
    let value_bytes = match take_field(entries, "value")? {
        Value::Bytes(b) => b.clone(),
        _ => return Err(SidecarError::MissingKey { key: "value" }),
    };
    if value_bytes.len() != SIGNATURE_LEN {
        return Err(SidecarError::SignatureLength {
            expected: SIGNATURE_LEN,
            actual: value_bytes.len(),
        });
    }
    Ok(SidecarSignature {
        algorithm,
        key_id,
        signed_domain,
        value: value_bytes,
    })
}

/// Verify that the envelope's `algorithm`, `signed_domain` and `creator`
/// device reference are the values §9.4 and §9.5 require. Returns the
/// resolved Ed25519 public key bytes for downstream signature checks.
pub(crate) fn validate_envelope_metadata(env: &SidecarV1) -> Result<Vec<u8>, SidecarError> {
    if env.signature.algorithm != ALGORITHM_ED25519 {
        return Err(SidecarError::UnsupportedAlgorithm(env.signature.algorithm.clone()));
    }
    if env.signature.signed_domain != SIGNATURE_DOMAIN {
        return Err(SidecarError::SignedDomainMismatch {
            expected: SIGNATURE_DOMAIN.to_string(),
            actual: env.signature.signed_domain.clone(),
        });
    }
    if env.creator.public_key.len() != crate::types::PUBLIC_KEY_LEN {
        return Err(SidecarError::ByteStringSize {
            field: "public_key",
            expected: crate::types::PUBLIC_KEY_LEN,
            actual: env.creator.public_key.len(),
        });
    }
    if env.signature.value.len() != crate::types::SIGNATURE_LEN {
        return Err(SidecarError::SignatureLength {
            expected: crate::types::SIGNATURE_LEN,
            actual: env.signature.value.len(),
        });
    }
    Ok(env.creator.public_key.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn toy_envelope() -> SidecarV1 {
        SidecarV1 {
            schema: SCHEMA_ID.to_string(),
            artifact_id: String::new(),
            created_at_ms: 1_700_000_000_000,
            creator: DeviceRef {
                device_id: "device-1".to_string(),
                public_key: vec![0u8; crate::types::PUBLIC_KEY_LEN],
            },
            layer: Layer {
                layer_id: "layer-1".to_string(),
                name: "My layer".to_string(),
                visibility: LayerVisibility::Private,
            },
            source_bindings: vec![SourceBinding {
                pack_id: "pack-1".to_string(),
                corpus_id: "corpus-1".to_string(),
                artifact_sha256: "deadbeef".to_string(),
            }],
            annotations: vec![SidecarAnnotation {
                annotation_id: "a1".to_string(),
                annotation_type: AnnotationType::Highlight,
                revision: 1,
                selector: text_map(&[("exact", Value::Text("hello".to_string()))]),
                body_text: Some("body".to_string()),
                style: text_map(&[("color_token", Value::Text("yellow".to_string()))]),
                notebook_id: None,
                tag_ids: vec!["t1".to_string()],
                asset_ids: vec![],
                created_at_ms: -1,
                updated_at_ms: 100,
            }],
            notebooks: vec![Notebook {
                notebook_id: "n1".to_string(),
                parent_notebook_id: None,
                name: "Notes".to_string(),
                description: String::new(),
            }],
            tags: vec![Tag {
                tag_id: "t1".to_string(),
                name: "tag".to_string(),
                color_token: "blue".to_string(),
            }],
            objects: vec![ObjectRef {
                object_sha256: "abc".to_string(),
                mime_type: "text/plain".to_string(),
                byte_length: 3,
                embedded_bytes: Some(vec![1, 2, 3]),
            }],
            signature: SidecarSignature {
                algorithm: ALGORITHM_ED25519.to_string(),
                key_id: "k1".to_string(),
                signed_domain: SIGNATURE_DOMAIN.to_string(),
                value: vec![0u8; SIGNATURE_LEN],
            },
        }
    }

    #[test]
    fn encode_then_decode_round_trips() {
        let env = toy_envelope();
        let bytes = encode_to_vec(&env);
        let back = decode(&bytes).expect("decode succeeds for canonically encoded envelope");
        assert_eq!(back.schema, env.schema);
        assert_eq!(back.artifact_id, env.artifact_id);
        assert_eq!(back.created_at_ms, env.created_at_ms);
        assert_eq!(back.creator, env.creator);
        assert_eq!(back.layer, env.layer);
        assert_eq!(back.source_bindings, env.source_bindings);
        assert_eq!(back.annotations, env.annotations);
        assert_eq!(back.notebooks, env.notebooks);
        assert_eq!(back.tags, env.tags);
        assert_eq!(back.objects, env.objects);
        assert_eq!(back.signature.algorithm, env.signature.algorithm);
    }

    #[test]
    fn deterministic_encoding_byte_identical_for_equal_envelopes() {
        let a = encode_to_vec(&toy_envelope());
        let b = encode_to_vec(&toy_envelope());
        assert_eq!(a, b);
        // Encoding the same envelope twice must produce the same length too.
        assert_eq!(a.len(), b.len());
    }

    #[test]
    fn artifact_id_stable_and_sixty_four_hex_chars() {
        let env = toy_envelope();
        let id1 = artifact_id(&env);
        let id2 = artifact_id(&env);
        assert_eq!(id1, id2);
        assert_eq!(id1.len(), 64);
        assert!(id1.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    }

    #[test]
    fn signing_form_excludes_signature_value_and_blanks_artifact_id() {
        let mut env = toy_envelope();
        env.artifact_id = "abcdef".to_string();
        env.signature.value = vec![0xff; SIGNATURE_LEN];
        let bytes = encode_unsigned_signing_form(&env);
        let value = crate::cbor::decode(&bytes).unwrap();
        let entries = match &value {
            Value::Map(e) => e,
            _ => unreachable!(),
        };
        // artifact_id must be empty in the signing form
        let aid = entries
            .iter()
            .find(|(k, _)| matches!(k, Value::Text(s) if s == "artifact_id"))
            .map(|(_, v)| v);
        assert_eq!(aid, Some(&Value::Text(String::new())));

        // signature sub-map must NOT contain a "value" entry
        let sig = entries
            .iter()
            .find(|(k, _)| matches!(k, Value::Text(s) if s == "signature"))
            .map(|(_, v)| v)
            .expect("signature entry present");
        let sig_entries = match sig {
            Value::Map(e) => e,
            _ => unreachable!(),
        };
        let has_value = sig_entries
            .iter()
            .any(|(k, _)| matches!(k, Value::Text(s) if s == "value"));
        assert!(
            !has_value,
            "signature sub-map in signing form must omit `value`"
        );
        // But algorithm, key_id, signed_domain must be present
        for required in ["algorithm", "key_id", "signed_domain"] {
            let present = sig_entries
                .iter()
                .any(|(k, _)| matches!(k, Value::Text(s) if s == required));
            assert!(present, "signing form must keep {}", required);
        }
    }
}
