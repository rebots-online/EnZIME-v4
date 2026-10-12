//! Canonical CBOR primitives for the sidecar codec.
//!
//! Implements RFC 8949 §4.2.1 (Core Deterministic Encoding) for the subset
//! of CBOR the sidecar CDDL uses.
//!
//! Rules enforced here mirror ARCHITECTURE.md §9.4 "Canonicalization":
//!   - preferred integer encoding (shortest form for arguments);
//!   - map keys sorted by encoded key bytes;
//!   - no indefinite-length items (AI 31 is rejected);
//!   - floating-point values are forbidden (major type 7 with AI 25/26/27);
//!   - definite-length arrays, maps, strings and byte strings only.
//!
//! `Value` is a recursive CBOR data model restricted to the types the
//! sidecar CDDL references (no tags, no floats, no undefined). It is used
//! as the Rust representation of CDDL `any` for `annotation.selector` and
//! `annotation.style`.

/// Major type codes (RFC 8949 §3.1).
const MT_UNSIGNED: u8 = 0;
const MT_NEGATIVE: u8 = 1;
const MT_BYTES: u8 = 2;
const MT_TEXT: u8 = 3;
const MT_ARRAY: u8 = 4;
const MT_MAP: u8 = 5;
const MT_TAG: u8 = 6;
const MT_OTHER: u8 = 7;

/// Additional-information value marking indefinite-length items (RFC 8949 §3.2).
const AI_INDEFINITE: u8 = 31;

/// Subset of the CBOR data model that the sidecar CDDL uses.
///
/// Floats, tags, and `undefined` are deliberately absent: the
/// canonicalization rules in ARCHITECTURE.md §9.4 forbid them, and the
/// CDDL never references any of them.
///
/// Serde is derived so that parent structs (e.g. `SidecarAnnotation`) can
/// also derive Serde for diagnostic JSON output. The Serde representation
/// is a tagged enum — not canonical CBOR — and is never used by the sidecar
/// codec itself; the codec always goes through [`encode`] / [`decode`].
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Value {
    /// Major type 0 — non-negative integer.
    Unsigned(u64),
    /// Major type 1 — negative integer, stored as the decoded `i64`.
    Negative(i64),
    /// Major type 2 — definite-length byte string.
    Bytes(Vec<u8>),
    /// Major type 3 — definite-length UTF-8 text string.
    Text(String),
    /// Major type 4 — definite-length array.
    Array(Vec<Value>),
    /// Major type 5 — definite-length map. The encoder sorts entries by
    /// encoded key bytes before emission; callers may hand in unsorted
    /// entries from human-readable source code without breaking canonicity.
    Map(Vec<(Value, Value)>),
    /// Major type 7, AI 21 — `true`.
    Bool(bool),
    /// Major type 7, AI 22 — `null`.
    Null,
}

/// Error returned by [`decode`] for any CBOR rule violation, including
/// every canonicalization rule that ARCHITECTURE.md §9.4 makes binding.
#[derive(Debug, thiserror::Error)]
pub enum CborError {
    #[error("input ended before a complete value was read")]
    Truncated,
    #[error("indefinite-length items are forbidden in canonical sidecar CBOR")]
    IndefiniteLength,
    #[error("reserved additional-information value {0}")]
    ReservedAi(u8),
    #[error("tagged values (major type 6) are forbidden in canonical sidecar CBOR")]
    TagForbidden,
    #[error("floating-point values are forbidden in canonical sidecar CBOR")]
    FloatForbidden,
    #[error("undefined (0x1f) is forbidden in canonical sidecar CBOR")]
    Undefined,
    #[error("invalid UTF-8 in text string: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
    #[error("integer argument {0} was not encoded using preferred serialization")]
    NonPreferredInteger(u64),
    #[error("map keys are not in canonical order, or a key is duplicated")]
    UnsortedMap,
    #[error("trailing bytes after top-level value")]
    Trailing,
    #[error("CBOR nesting exceeds the maximum depth of {0}")]
    DepthLimit(usize),
}

/// Maximum nesting depth the decoder will traverse for arrays and maps. The
/// sidecar CDDL defines structures at most a handful of levels deep; the cap
/// exists so hostile input (e.g. thousands of nested `0x81` array headers)
/// cannot overflow the stack on import.
pub const MAX_DEPTH: usize = 64;

/// Encode `value` into `out` using canonical RFC 8949 deterministic rules.
///
/// Map entries are sorted by encoded key bytes before emission, so equal
/// `Value`s always produce byte-identical output — the property the sidecar
/// codec relies on for `artifact_id` derivation and signing.
pub fn encode(value: &Value, out: &mut Vec<u8>) {
    match value {
        Value::Unsigned(n) => encode_arg(MT_UNSIGNED, *n as u128, out),
        Value::Negative(v) => {
            // CBOR major type 1 stores `(-1 - value)` as the argument.
            // Compute via i128 so `i64::MIN` does not overflow.
            let arg = (-1_i128 - (*v as i128)) as u128;
            encode_arg(MT_NEGATIVE, arg, out);
        }
        Value::Bytes(b) => {
            encode_arg(MT_BYTES, b.len() as u128, out);
            out.extend_from_slice(b);
        }
        Value::Text(s) => {
            let bytes = s.as_bytes();
            encode_arg(MT_TEXT, bytes.len() as u128, out);
            out.extend_from_slice(bytes);
        }
        Value::Array(items) => {
            encode_arg(MT_ARRAY, items.len() as u128, out);
            for item in items {
                encode(item, out);
            }
        }
        Value::Map(entries) => {
            // Compute the encoded form of each key once, then sort entries
            // by those bytes — the RFC 8949 deterministic ordering rule.
            let mut keyed: Vec<(Vec<u8>, &Value, &Value)> = entries
                .iter()
                .map(|(k, v)| {
                    let mut kb = Vec::new();
                    encode(k, &mut kb);
                    (kb, k, v)
                })
                .collect();
            keyed.sort_by(|a, b| a.0.cmp(&b.0));
            // Canonical CBOR forbids duplicate keys; a duplicate here is a
            // programmer error (the CDDL gives every map distinct keys).
            if keyed.windows(2).any(|w| w[0].0 == w[1].0) {
                panic!("duplicate map key in canonical sidecar encoder");
            }
            encode_arg(MT_MAP, keyed.len() as u128, out);
            for (_, k, v) in keyed {
                encode(k, out);
                encode(v, out);
            }
        }
        Value::Bool(true) => out.push((MT_OTHER << 5) | 21),
        Value::Bool(false) => out.push((MT_OTHER << 5) | 20),
        Value::Null => out.push((MT_OTHER << 5) | 22),
    }
}

/// Encode a value into a fresh `Vec<u8>`.
pub fn encode_to_vec(value: &Value) -> Vec<u8> {
    let mut out = Vec::new();
    encode(value, &mut out);
    out
}

/// Build a canonical CBOR map from text-keyed entries. Convenience for the
/// sidecar encoder, where every map in the CDDL uses text keys; the result
/// is a [`Value::Map`] sorted by encoded key bytes (the encoder will sort
/// again at emit time, but constructing pre-sorted keeps equality checks
/// structural rather than only-bytes).
pub fn text_map(entries: &[(&str, Value)]) -> Value {
    let mut keyed: Vec<(&str, Value)> = entries.iter().map(|(k, v)| (*k, v.clone())).collect();
    keyed.sort_by(|a, b| {
        let mut ka = Vec::new();
        let mut kb = Vec::new();
        encode(&Value::Text(a.0.to_string()), &mut ka);
        encode(&Value::Text(b.0.to_string()), &mut kb);
        ka.cmp(&kb)
    });
    if keyed.windows(2).any(|w| w[0].0 == w[1].0) {
        panic!("duplicate text key passed to text_map");
    }
    Value::Map(keyed.into_iter().map(|(k, v)| (Value::Text(k.to_string()), v)).collect())
}

/// Decode a single top-level CBOR value from `input`, requiring the entire
/// input to be consumed and the encoding to be canonical (preferred integer
/// serialization, no floats, no indefinite-length, sorted map keys, no
/// duplicate keys).
pub fn decode(input: &[u8]) -> Result<Value, CborError> {
    let mut pos = 0usize;
    let v = decode_one(input, &mut pos, 0)?;
    if pos != input.len() {
        return Err(CborError::Trailing);
    }
    Ok(v)
}

fn encode_arg(major: u8, arg: u128, out: &mut Vec<u8>) {
    debug_assert!(major < 8);
    let mt = major << 5;
    if arg <= 23 {
        out.push(mt | (arg as u8));
    } else if arg <= 0xff {
        out.push(mt | 24);
        out.push(arg as u8);
    } else if arg <= 0xffff {
        out.push(mt | 25);
        out.extend_from_slice(&(arg as u16).to_be_bytes());
    } else if arg <= 0xffff_ffff {
        out.push(mt | 26);
        out.extend_from_slice(&(arg as u32).to_be_bytes());
    } else {
        out.push(mt | 27);
        out.extend_from_slice(&(arg as u64).to_be_bytes());
    }
}

fn decode_one(input: &[u8], pos: &mut usize, depth: usize) -> Result<Value, CborError> {
    if depth > MAX_DEPTH {
        return Err(CborError::DepthLimit(MAX_DEPTH));
    }
    if *pos >= input.len() {
        return Err(CborError::Truncated);
    }
    let initial = input[*pos];
    *pos += 1;
    let major = initial >> 5;
    let ai = initial & 0x1f;

    if major == MT_OTHER {
        return decode_other(ai, input, pos);
    }
    if major == MT_TAG {
        return Err(CborError::TagForbidden);
    }

    let arg = read_arg(major, ai, input, pos)?;
    match major {
        MT_UNSIGNED => Ok(Value::Unsigned(arg)),
        MT_NEGATIVE => {
            // Reconstruct i64 from `arg = -1 - value`. arg may be up to
            // 2^63 (when value == i64::MIN); compute via i128 to avoid overflow.
            let v = (-1_i128 - arg as i128) as i64;
            Ok(Value::Negative(v))
        }
        MT_BYTES => {
            let bytes = take(arg, input, pos)?;
            Ok(Value::Bytes(bytes))
        }
        MT_TEXT => {
            let bytes = take(arg, input, pos)?;
            let s = String::from_utf8(bytes)?;
            Ok(Value::Text(s))
        }
        MT_ARRAY => {
            let n = usize::try_from(arg).map_err(|_| CborError::Truncated)?;
            let mut items = Vec::with_capacity(n.min(1024));
            for _ in 0..n {
                items.push(decode_one(input, pos, depth + 1)?);
            }
            Ok(Value::Array(items))
        }
        MT_MAP => {
            let n = usize::try_from(arg).map_err(|_| CborError::Truncated)?;
            let mut entries: Vec<(Value, Value)> = Vec::with_capacity(n.min(1024));
            for _ in 0..n {
                let k = decode_one(input, pos, depth + 1)?;
                let v = decode_one(input, pos, depth + 1)?;
                entries.push((k, v));
            }
            // Reject non-canonical ordering and duplicate keys at decode time.
            verify_sorted_unique(&entries)?;
            Ok(Value::Map(entries))
        }
        _ => unreachable!("major type {} is rejected above", major),
    }
}

fn decode_other(ai: u8, input: &[u8], pos: &mut usize) -> Result<Value, CborError> {
    match ai {
        20 => Ok(Value::Bool(false)),
        21 => Ok(Value::Bool(true)),
        22 => Ok(Value::Null),
        23 => Err(CborError::Undefined),
        // 25..=27 carry float16/float32/float64 — forbidden by §9.4.
        25 | 26 | 27 => {
            // Advance past the float bytes so error reporting is consistent.
            let len = match ai {
                25 => 2,
                26 => 4,
                _ => 8,
            };
            take(len, input, pos)?;
            Err(CborError::FloatForbidden)
        }
        AI_INDEFINITE => Err(CborError::IndefiniteLength),
        other => Err(CborError::ReservedAi(other)),
    }
}

fn read_arg(major: u8, ai: u8, input: &[u8], pos: &mut usize) -> Result<u64, CborError> {
    if ai < 24 {
        return Ok(ai as u64);
    }
    if ai == AI_INDEFINITE {
        return Err(CborError::IndefiniteLength);
    }
    if matches!(ai, 28 | 29 | 30) {
        return Err(CborError::ReservedAi(ai));
    }
    let len = match ai {
        24 => 1,
        25 => 2,
        26 => 4,
        27 => 8,
        _ => unreachable!(),
    };
    let bytes = take(len as u64, input, pos)?;
    let arg = bytes.iter().fold(0u64, |acc, b| (acc << 8) | (*b as u64));
    // Preferred serialization: the AI width must be the smallest that fits.
    // Major type 7 (handled by decode_other) is exempt — its AI selects a
    // value type rather than encoding an integer width.
    if major != MT_OTHER {
        verify_preferred(ai, arg)?;
    }
    Ok(arg)
}

fn verify_preferred(ai: u8, arg: u64) -> Result<(), CborError> {
    let min_ai = if arg <= 23 {
        0
    } else if arg <= 0xff {
        24
    } else if arg <= 0xffff {
        25
    } else if arg <= 0xffff_ffff {
        26
    } else {
        27
    };
    if min_ai == 0 {
        if ai <= 23 {
            return Ok(());
        }
        return Err(CborError::NonPreferredInteger(arg));
    }
    if ai == min_ai {
        Ok(())
    } else {
        Err(CborError::NonPreferredInteger(arg))
    }
}

fn take(n: u64, input: &[u8], pos: &mut usize) -> Result<Vec<u8>, CborError> {
    let n = usize::try_from(n).map_err(|_| CborError::Truncated)?;
    let end = pos.checked_add(n).ok_or(CborError::Truncated)?;
    if end > input.len() {
        return Err(CborError::Truncated);
    }
    let out = input[*pos..end].to_vec();
    *pos = end;
    Ok(out)
}

fn verify_sorted_unique(entries: &[(Value, Value)]) -> Result<(), CborError> {
    let mut prev: Option<Vec<u8>> = None;
    for (k, _) in entries {
        let mut kb = Vec::new();
        encode(k, &mut kb);
        if let Some(p) = prev.as_ref() {
            if kb.as_slice() <= p.as_slice() {
                // Less-than means unsorted; equal means duplicate.
                return Err(CborError::UnsortedMap);
            }
        }
        prev = Some(kb);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsigned_preferred_serialization() {
        // 0..=23 fit in the initial byte.
        assert_eq!(encode_to_vec(&Value::Unsigned(0)), vec![0x00]);
        assert_eq!(encode_to_vec(&Value::Unsigned(23)), vec![0x17]);
        // 24 is the smallest value requiring AI 24.
        assert_eq!(encode_to_vec(&Value::Unsigned(24)), vec![0x18, 0x18]);
        assert_eq!(encode_to_vec(&Value::Unsigned(0xff)), vec![0x18, 0xff]);
        // 256 forces AI 25.
        assert_eq!(encode_to_vec(&Value::Unsigned(0x100)), vec![0x19, 0x01, 0x00]);
        assert_eq!(
            encode_to_vec(&Value::Unsigned(0xffff)),
            vec![0x19, 0xff, 0xff]
        );
        assert_eq!(
            encode_to_vec(&Value::Unsigned(0x1_0000)),
            vec![0x1a, 0x00, 0x01, 0x00, 0x00]
        );
        assert_eq!(
            encode_to_vec(&Value::Unsigned(0x1_0000_0000)),
            vec![0x1b, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00]
        );
    }

    #[test]
    fn negative_preferred_serialization() {
        // -1 stores argument 0.
        assert_eq!(encode_to_vec(&Value::Negative(-1)), vec![0x20]);
        assert_eq!(encode_to_vec(&Value::Negative(-24)), vec![0x37]);
        // -25 stores argument 24 → AI 24.
        assert_eq!(encode_to_vec(&Value::Negative(-25)), vec![0x38, 0x18]);
        // i64::MIN must round-trip without overflow.
        let v = Value::Negative(i64::MIN);
        let bytes = encode_to_vec(&v);
        let back = decode(&bytes).unwrap();
        assert_eq!(back, v);
    }

    #[test]
    fn map_keys_sorted_by_encoded_bytes() {
        // Names chosen so length-first ordering differs from bytewise ASCII
        // ordering of the raw strings: "a" (shorter) must come first by
        // encoded-bytes ordering because the length prefix sorts before any
        // payload byte comparison.
        let v = text_map(&[
            ("bbb", Value::Unsigned(1)),
            ("aaa", Value::Unsigned(2)),
            ("a", Value::Unsigned(3)),
        ]);
        let out = encode_to_vec(&v);
        // Encoded keys: "a" -> 0x61 0x61 ; "aaa" -> 0x63 'a' 'a' 'a' ; "bbb" -> 0x63 'b' 'b' 'b'
        // Length-first ordering: "a" (2 bytes encoded) < "aaa"/"bbb" (4 bytes encoded).
        assert_eq!(
            out,
            vec![
                0xa3, // map(3)
                0x61, b'a', 0x03, // "a" -> 3
                0x63, b'a', b'a', b'a', 0x02, // "aaa" -> 2
                0x63, b'b', b'b', b'b', 0x01, // "bbb" -> 1
            ]
        );
    }

    #[test]
    fn rejects_floats_and_indefinite() {
        // float16 1.0 = 0xf9 0x3c 0x00
        assert!(matches!(decode(&[0xf9, 0x3c, 0x00]), Err(CborError::FloatForbidden)));
        // float32 1.0 = 0xfa 0x3f 0x80 0x00 0x00
        assert!(matches!(
            decode(&[0xfa, 0x3f, 0x80, 0x00, 0x00]),
            Err(CborError::FloatForbidden)
        ));
        // indefinite-length array start
        assert!(matches!(decode(&[0x9f]), Err(CborError::IndefiniteLength)));
    }

    #[test]
    fn rejects_non_preferred_integer_encoding() {
        // 0x18 0x00 = AI 24 with argument 0, which fits inline.
        assert!(matches!(decode(&[0x18, 0x00]), Err(CborError::NonPreferredInteger(0))));
        // 0x19 0x00 0xff = AI 25 with argument 0xff, which fits in AI 24.
        assert!(matches!(
            decode(&[0x19, 0x00, 0xff]),
            Err(CborError::NonPreferredInteger(255))
        ));
    }

    #[test]
    fn rejects_unsorted_or_duplicate_map_keys() {
        // Two text keys "a" and "aa" in canonical order.
        let ok = vec![0xa2, 0x61, b'a', 0x01, 0x62, b'a', b'a', 0x02];
        assert!(decode(&ok).is_ok());

        // Reversed: "aa" before "a" — encoded bytes 0x62.. > 0x61.., wrong order.
        let bad = vec![0xa2, 0x62, b'a', b'a', 0x02, 0x61, b'a', 0x01];
        assert!(matches!(decode(&bad), Err(CborError::UnsortedMap)));

        // Duplicate key
        let dup = vec![0xa2, 0x61, b'a', 0x01, 0x61, b'a', 0x02];
        assert!(matches!(decode(&dup), Err(CborError::UnsortedMap)));
    }

    #[test]
    fn deterministic_encoding_is_byte_identical() {
        // The same logical map handed in via different source-code order
        // must encode to identical bytes — this is the contract artifact_id
        // and the signature depend on.
        let entries_a: Vec<(&str, Value)> = vec![
            ("z", Value::Unsigned(7)),
            ("a longer key", Value::Array(vec![])),
        ];
        let entries_b: Vec<(&str, Value)> = vec![
            ("a longer key", Value::Array(vec![])),
            ("z", Value::Unsigned(7)),
        ];
        assert_eq!(encode_to_vec(&text_map(&entries_a)), encode_to_vec(&text_map(&entries_b)));
    }

    #[test]
    fn round_trip_nested_map_selector_any() {
        // CDDL `selector: any` exercises arbitrary nested maps. Make sure
        // encode/decode are inverse and that floats are still rejected.
        let nested = Value::Array(vec![
            text_map(&[
                ("exact", Value::Text("hi".to_string())),
                ("prefix", Value::Text("p".to_string())),
            ]),
            Value::Unsigned(42),
        ]);
        let bytes = encode_to_vec(&nested);
        let back = decode(&bytes).unwrap();
        assert_eq!(back, nested);
    }

    #[test]
    fn nesting_up_to_max_depth_round_trips() {
        // A chain of exactly MAX_DEPTH nested arrays (levels 0..MAX_DEPTH)
        // must still decode — the cap accommodates legal CDDL structures.
        let mut v = Value::Unsigned(1);
        for _ in 0..MAX_DEPTH {
            v = Value::Array(vec![v]);
        }
        let bytes = encode_to_vec(&v);
        assert_eq!(decode(&bytes).unwrap(), v);
    }

    #[test]
    fn nesting_beyond_max_depth_is_rejected() {
        // One level past the cap must fail without recursing further.
        let mut v = Value::Unsigned(1);
        for _ in 0..(MAX_DEPTH + 1) {
            v = Value::Array(vec![v]);
        }
        let bytes = encode_to_vec(&v);
        assert!(matches!(decode(&bytes), Err(CborError::DepthLimit(_))));
    }

    #[test]
    fn hostile_deeply_nested_input_hits_depth_limit() {
        // ~100k nested array headers: pre-fix this overflowed the stack;
        // now the depth budget rejects it long before.
        let hostile = vec![0x81u8; 100_000];
        assert!(matches!(decode(&hostile), Err(CborError::DepthLimit(_))));
    }
}
