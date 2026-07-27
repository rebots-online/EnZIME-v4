//! Shared validation helpers used by both OpenZIM-XML and EnZIME-JSON catalog
//! adapters. Every input that fails one of these checks is rejected with the
//! declared `CatalogError::Malformed` variant — never silently coerced.

use crate::CatalogError;
use domain::identity::ContentDigest;
use url::Url;

/// A bare catalog-entry identifier is opaque to the architecture, but it must
/// not be empty, must not contain path/URL components, must not carry control
/// characters and must stay inside a sane length budget. These rules mirror the
/// spirit of `domain::identity` so that a downstream pack-id parser will not be
/// fed garbage.
pub(crate) const CATALOG_ENTRY_ID_MAX_BYTES: usize = 256;

pub(crate) fn validate_catalog_entry_id(raw: &str) -> Result<(), CatalogError> {
    if raw.trim().is_empty() {
        return Err(CatalogError::Malformed(
            "catalog_entry_id is empty".to_string(),
        ));
    }
    if raw.len() > CATALOG_ENTRY_ID_MAX_BYTES {
        return Err(CatalogError::Malformed(format!(
            "catalog_entry_id exceeds {CATALOG_ENTRY_ID_MAX_BYTES} bytes (was {})",
            raw.len()
        )));
    }
    if raw.contains('/')
        || raw.contains('\\')
        || raw.contains("://")
        || raw.starts_with('.')
        || raw.contains("..")
        || raw.contains('\0')
        || raw.chars().any(|c| c.is_control() || c.is_whitespace())
    {
        return Err(CatalogError::Malformed(format!(
            "catalog_entry_id is location-bearing or contains whitespace: {raw}"
        )));
    }
    Ok(())
}

pub(crate) fn validate_non_empty(
    field: &'static str,
    raw: &str,
) -> Result<(), CatalogError> {
    if raw.trim().is_empty() {
        return Err(CatalogError::Malformed(format!(
            "{field} is empty"
        )));
    }
    Ok(())
}

/// `language_bcp47` is stored as-is from the source but must be a syntactically
/// plausible BCP47 tag: ASCII letters, digits and hyphens only, non-empty.
pub(crate) fn validate_language_bcp47(raw: &str) -> Result<(), CatalogError> {
    if raw.is_empty() {
        return Err(CatalogError::Malformed(
            "language_bcp47 is empty".to_string(),
        ));
    }
    if !raw
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-')
    {
        return Err(CatalogError::Malformed(format!(
            "language_bcp47 contains non-tag characters: {raw}"
        )));
    }
    Ok(())
}

/// `artifact_url` must use one of the schemes a `TransferSource` is allowed to
/// fetch. Anything else is rejected — never silently coerced into http(s).
pub(crate) fn validate_artifact_url(raw: &str) -> Result<(), CatalogError> {
    let parsed = Url::parse(raw).map_err(|e| {
        CatalogError::Malformed(format!("artifact_url is not a valid URL: {e}"))
    })?;
    match parsed.scheme() {
        "http" | "https" | "file" => Ok(()),
        other => Err(CatalogError::Malformed(format!(
            "artifact_url uses unsupported scheme '{other}'"
        ))),
    }
}

pub(crate) fn validate_artifact_size_bytes(
    size: u64,
) -> Result<(), CatalogError> {
    if size == 0 {
        return Err(CatalogError::Malformed(
            "artifact_size_bytes is zero".to_string(),
        ));
    }
    Ok(())
}

/// The architecture reuses domain's lowercase-hex SHA-256 representation, so we
/// delegate to `ContentDigest::sha256_from_hex` for the exact 64-character
/// lowercase rule. Uppercase, short, long, or non-hex inputs are rejected.
pub(crate) fn validate_artifact_sha256(raw: &str) -> Result<(), CatalogError> {
    ContentDigest::sha256_from_hex(raw)
        .map(|_| ())
        .map_err(|_| {
            CatalogError::Malformed(
                "artifact_sha256 must be 64 lowercase hex characters"
                    .to_string(),
            )
        })
}

/// `signature_url`, when present, must itself be fetchable: http(s|file).
pub(crate) fn validate_signature_url(raw: &str) -> Result<(), CatalogError> {
    let parsed = Url::parse(raw).map_err(|e| {
        CatalogError::Malformed(format!(
            "signature_url is not a valid URL: {e}"
        ))
    })?;
    match parsed.scheme() {
        "http" | "https" | "file" => Ok(()),
        other => Err(CatalogError::Malformed(format!(
            "signature_url uses unsupported scheme '{other}'"
        ))),
    }
}
