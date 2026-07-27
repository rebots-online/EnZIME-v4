use domain::identity::{
    ArticleKey, ContentDigest, DigestAlgorithm, EntryKey, IdentityError, PackId, PackVersionId,
};
use serde_json::json;

#[test]
fn pack_id_round_trips_as_bare_string() {
    let id = PackId::parse("wikipedia_en_all_maxi").unwrap();
    assert_eq!(id.as_str(), "wikipedia_en_all_maxi");
    let ser = serde_json::to_string(&id).unwrap();
    assert_eq!(ser, json!("wikipedia_en_all_maxi").to_string());
    let de: PackId = serde_json::from_str(&ser).unwrap();
    assert_eq!(de, id);
}

#[test]
fn pack_version_id_round_trips_as_bare_string() {
    let id = PackVersionId::parse("wikipedia_en_all_maxi_2024-10").unwrap();
    assert_eq!(id.as_str(), "wikipedia_en_all_maxi_2024-10");
    let ser = serde_json::to_string(&id).unwrap();
    assert_eq!(ser, json!("wikipedia_en_all_maxi_2024-10").to_string());
    let de: PackVersionId = serde_json::from_str(&ser).unwrap();
    assert_eq!(de, id);
}

#[test]
fn article_key_round_trips_as_bare_string() {
    let key = ArticleKey::parse("article_abc").unwrap();
    assert_eq!(key.as_str(), "article_abc");
    let ser = serde_json::to_string(&key).unwrap();
    assert_eq!(ser, json!("article_abc").to_string());
    let de: ArticleKey = serde_json::from_str(&ser).unwrap();
    assert_eq!(de, key);
}

#[test]
fn entry_key_round_trips_as_bare_string() {
    let key = EntryKey::parse("entry_xyz").unwrap();
    assert_eq!(key.as_str(), "entry_xyz");
    let ser = serde_json::to_string(&key).unwrap();
    assert_eq!(ser, json!("entry_xyz").to_string());
    let de: EntryKey = serde_json::from_str(&ser).unwrap();
    assert_eq!(de, key);
}

#[test]
fn content_digest_round_trips() {
    let hex = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let digest = ContentDigest::sha256_from_hex(hex).unwrap();
    assert_eq!(digest.algorithm(), DigestAlgorithm::Sha256);
    assert_eq!(digest.hex(), hex);
    let ser = serde_json::to_string(&digest).unwrap();
    let expected = json!({
        "algorithm": "sha256",
        "hex": hex,
    })
    .to_string();
    assert_eq!(ser, expected);
    let de: ContentDigest = serde_json::from_str(&ser).unwrap();
    assert_eq!(de, digest);
}

#[test]
fn digest_algorithm_serializes_snake_case() {
    let ser = serde_json::to_string(&DigestAlgorithm::Sha256).unwrap();
    assert_eq!(ser, json!("sha256").to_string());
    let de: DigestAlgorithm = serde_json::from_str("\"sha256\"").unwrap();
    assert_eq!(de, DigestAlgorithm::Sha256);
    assert_eq!(DigestAlgorithm::Sha256.hex_len(), 64);
}

#[test]
fn parse_accepts_plain_identifier() {
    PackId::parse("wikipedia_en_all_maxi").unwrap();
    PackVersionId::parse("wikipedia_en_all_maxi_2024-10").unwrap();
    ArticleKey::parse("article_abc").unwrap();
    EntryKey::parse("entry_xyz").unwrap();
}

#[test]
fn parse_rejects_location_bearing_forms() {
    let bad = ["a/b", "a\\b", "http://x", ".hidden", "../escape", "C:x"];
    for raw in bad {
        let err = PackId::parse(raw).unwrap_err();
        assert_eq!(
            err,
            IdentityError::LocationBearing(raw.to_string()),
            "raw={raw:?}"
        );
    }
    let with_nul = "ab\0cd";
    let err = PackId::parse(with_nul).unwrap_err();
    assert_eq!(
        err,
        IdentityError::LocationBearing(with_nul.to_string())
    );
}

#[test]
fn parse_rejects_empty_and_whitespace_only() {
    let bad = ["", "   ", "\t", "\n", " \t\n"];
    for raw in bad {
        assert_eq!(
            PackId::parse(raw).unwrap_err(),
            IdentityError::Empty,
            "raw={raw:?}"
        );
    }
}

#[test]
fn parse_rejects_too_long_and_accepts_max() {
    let max_id = "a".repeat(256);
    PackId::parse(&max_id).unwrap();
    let too_long = "a".repeat(257);
    assert_eq!(
        PackId::parse(&too_long).unwrap_err(),
        IdentityError::TooLong {
            max: 256,
            actual: 257
        }
    );
}

#[test]
fn sha256_from_hex_accepts_64_lowercase() {
    let hex = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let digest = ContentDigest::sha256_from_hex(hex).unwrap();
    assert_eq!(digest.hex(), hex);
    assert_eq!(digest.algorithm(), DigestAlgorithm::Sha256);
}

#[test]
fn sha256_from_hex_rejects_wrong_length_and_bad_chars() {
    assert_eq!(
        ContentDigest::sha256_from_hex("0".repeat(63)).unwrap_err(),
        IdentityError::MalformedDigest { expected: 64 }
    );
    assert_eq!(
        ContentDigest::sha256_from_hex("0".repeat(65)).unwrap_err(),
        IdentityError::MalformedDigest { expected: 64 }
    );
    let upper = format!("A{}", "0".repeat(63));
    assert_eq!(
        ContentDigest::sha256_from_hex(upper).unwrap_err(),
        IdentityError::MalformedDigest { expected: 64 }
    );
    let bad = format!("g{}", "0".repeat(63));
    assert_eq!(
        ContentDigest::sha256_from_hex(bad).unwrap_err(),
        IdentityError::MalformedDigest { expected: 64 }
    );
}
