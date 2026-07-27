use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PackId(pub String);

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PackVersionId(pub String);

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ArticleKey(pub String);

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct EntryKey(pub String);

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ContentDigest {
    pub algorithm: DigestAlgorithm,
    pub hex: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DigestAlgorithm {
    Sha256,
}

#[derive(Clone, Debug, thiserror::Error, Eq, PartialEq)]
pub enum IdentityError {
    #[error("identifier is empty")]
    Empty,
    #[error("identifier contains a path or URL component: {0}")]
    LocationBearing(String),
    #[error("identifier exceeds {max} bytes (was {actual})")]
    TooLong { max: usize, actual: usize },
    #[error("digest hex is not {expected} lowercase hex characters")]
    MalformedDigest { expected: usize },
}

fn validate_identifier(raw: &str, max: usize) -> Result<(), IdentityError> {
    if raw.trim().is_empty() {
        return Err(IdentityError::Empty);
    }
    if raw.len() > max {
        return Err(IdentityError::TooLong {
            max,
            actual: raw.len(),
        });
    }
    if raw.contains('/')
        || raw.contains('\\')
        || raw.contains("://")
        || raw.starts_with('.')
        || raw.contains("..")
        || raw.contains('\0')
        || is_windows_drive_prefix(raw)
    {
        return Err(IdentityError::LocationBearing(raw.to_string()));
    }
    Ok(())
}

fn is_windows_drive_prefix(s: &str) -> bool {
    let mut chars = s.chars();
    match (chars.next(), chars.next()) {
        (Some(letter), Some(':')) => letter.is_ascii_alphabetic(),
        _ => false,
    }
}

impl PackId {
    pub const MAX_BYTES: usize = 256;

    pub fn parse(raw: impl Into<String>) -> Result<Self, IdentityError> {
        let raw = raw.into();
        validate_identifier(&raw, Self::MAX_BYTES)?;
        Ok(PackId(raw))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl PackVersionId {
    pub const MAX_BYTES: usize = 256;

    pub fn parse(raw: impl Into<String>) -> Result<Self, IdentityError> {
        let raw = raw.into();
        validate_identifier(&raw, Self::MAX_BYTES)?;
        Ok(PackVersionId(raw))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ArticleKey {
    pub const MAX_BYTES: usize = 256;

    pub fn parse(raw: impl Into<String>) -> Result<Self, IdentityError> {
        let raw = raw.into();
        validate_identifier(&raw, Self::MAX_BYTES)?;
        Ok(ArticleKey(raw))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl EntryKey {
    pub const MAX_BYTES: usize = 256;

    pub fn parse(raw: impl Into<String>) -> Result<Self, IdentityError> {
        let raw = raw.into();
        validate_identifier(&raw, Self::MAX_BYTES)?;
        Ok(EntryKey(raw))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ContentDigest {
    pub fn sha256_from_hex(hex: impl Into<String>) -> Result<Self, IdentityError> {
        let hex = hex.into();
        if hex.len() != 64 || !hex.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f')) {
            return Err(IdentityError::MalformedDigest { expected: 64 });
        }
        Ok(ContentDigest {
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

impl DigestAlgorithm {
    pub fn hex_len(self) -> usize {
        match self {
            DigestAlgorithm::Sha256 => 64,
        }
    }
}
