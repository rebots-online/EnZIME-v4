use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ArticleKey(pub String);
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct EntryKey(pub String);
impl ArticleKey {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl EntryKey {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DigestAlgorithm {
    Sha256,
}
impl DigestAlgorithm {
    pub fn hex_len(self) -> usize {
        64
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ContentDigest {
    pub algorithm: DigestAlgorithm,
    pub hex: String,
}
#[derive(Clone, Debug, thiserror::Error, Eq, PartialEq)]
pub enum DigestError {
    #[error("digest hex is not {expected} lowercase hex characters")]
    MalformedDigest { expected: usize },
}
impl ContentDigest {
    pub fn sha256_from_hex(hex: impl Into<String>) -> Result<Self, DigestError> {
        let hex = hex.into();
        if hex.len() != 64
            || !hex
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(DigestError::MalformedDigest { expected: 64 });
        }
        Ok(Self {
            algorithm: DigestAlgorithm::Sha256,
            hex,
        })
    }
    pub fn hex(&self) -> &str {
        &self.hex
    }
    pub fn algorithm(&self) -> DigestAlgorithm {
        self.algorithm
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn digest_is_strict_and_archive_keys_preserve_paths() {
        let valid = "0123456789abcdef".repeat(4);
        let digest = ContentDigest::sha256_from_hex(&valid).unwrap();
        assert_eq!(digest.hex(), valid);
        assert_eq!(digest.algorithm().hex_len(), 64);
        for invalid in [
            "a".repeat(63),
            "a".repeat(65),
            "A".repeat(64),
            "g".repeat(64),
        ] {
            assert_eq!(
                ContentDigest::sha256_from_hex(invalid),
                Err(DigestError::MalformedDigest { expected: 64 })
            );
        }
        assert_eq!(ArticleKey("A/Café".into()).as_str(), "A/Café");
        assert_eq!(
            EntryKey("I/nested/image.png".into()).as_str(),
            "I/nested/image.png"
        );
    }
}
