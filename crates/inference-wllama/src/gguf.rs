//! Real verification of a GGUF model artifact against a
//! [`ports::inference::ModelManifest`].
//!
//! GGUF file format reference:
//! <https://github.com/ggerganov/ggml/blob/master/docs/gguf.md>
//!
//! The 4-byte magic at offset 0 is `0x46554747` (little-endian `G G U F`,
//! i.e. bytes `0x47 0x47 0x55 0x46`). Verification reads the entire file
//! once, hashing as it goes, and confirms:
//!
//! 1. the manifest declares `ModelFormat::Gguf`;
//! 2. the manifest declares support for the `LocalLm` backend;
//! 3. the manifest's `artifact_sha256` field is well-formed (64 lowercase
//!    hex characters);
//! 4. the path resolves to a readable file;
//! 5. the file's first 4 bytes are the GGUF magic;
//! 6. the file's total size matches `artifact_size_bytes`;
//! 7. the file's SHA-256 matches `artifact_sha256`.
//!
//! No step is skipped, no value is trusted from the caller.

use ports::inference::{BackendKind, ModelFormat, ModelManifest};
use sha2::{Digest, Sha256};
use std::path::Path;
use tokio::io::AsyncReadExt;

/// GGUF magic, little-endian: bytes `G G U F` = `0x47 0x47 0x55 0x46`.
/// Read as a `u32` from disk via `from_le_bytes`, this equals `0x46554747`.
pub const GGUF_MAGIC: u32 = 0x46554747;

#[derive(Debug, thiserror::Error)]
pub enum VerifyError {
    #[error("artifact path is not a readable file: {0}")]
    Path(String),
    #[error("file is too small to contain a GGUF header ({0} bytes)")]
    Truncated(u64),
    #[error("magic bytes do not match GGUF (read {read:#010x}, expected {expected:#010x})")]
    BadMagic { read: u32, expected: u32 },
    #[error("manifest declares format {declared:?}, artifact is GGUF")]
    FormatMismatch { declared: ModelFormat },
    #[error("manifest does not declare support for the LocalLm backend")]
    BackendNotSupported,
    #[error("file size {actual} does not match manifest size {expected}")]
    SizeMismatch { actual: u64, expected: u64 },
    #[error("sha256 mismatch: manifest expected {expected}, file computed {actual}")]
    DigestMismatch { expected: String, actual: String },
    #[error("manifest sha256 field is not 64 lowercase hex characters: {0}")]
    MalformedManifestDigest(String),
    #[error("io failure: {0}")]
    Io(String),
}

impl From<VerifyError> for ports::inference::InferenceError {
    fn from(value: VerifyError) -> Self {
        match value {
            VerifyError::Io(message) => {
                ports::inference::InferenceError::Runtime(message)
            }
            other => ports::inference::InferenceError::ModelIncompatible(other.to_string()),
        }
    }
}

/// Stream the entire file once, hashing as it goes, and verify against the
/// manifest. Returns the computed `(size_bytes, sha256_hex)` on success.
pub async fn verify_gguf_artifact(
    manifest: &ModelManifest,
    artifact_path: &Path,
) -> Result<(u64, String), VerifyError> {
    // (1) Static manifest checks.
    if manifest.format != ModelFormat::Gguf {
        return Err(VerifyError::FormatMismatch {
            declared: manifest.format,
        });
    }
    if !manifest
        .supported_backends
        .iter()
        .any(|b| *b == BackendKind::LocalLm)
    {
        return Err(VerifyError::BackendNotSupported);
    }
    let expected_hex = manifest.artifact_sha256.trim().to_ascii_lowercase();
    if expected_hex.len() != 64 || !expected_hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(VerifyError::MalformedManifestDigest(expected_hex));
    }

    // (2) Path sanity.
    if !artifact_path.is_file() {
        return Err(VerifyError::Path(artifact_path.display().to_string()));
    }

    // (3) Stream-read the entire file, hashing as we go. The first 4 bytes
    //     are inspected for the GGUF magic before we accept the rest.
    let mut file = tokio::fs::File::open(artifact_path)
        .await
        .map_err(|e| VerifyError::Io(e.to_string()))?;

    let mut hasher = Sha256::new();
    let mut size: u64 = 0;
    let mut magic_buf = [0u8; 4];
    let mut header_read = false;
    let mut buf = [0u8; 64 * 1024];

    loop {
        let n = file
            .read(&mut buf)
            .await
            .map_err(|e| VerifyError::Io(e.to_string()))?;
        if n == 0 {
            break;
        }
        if !header_read {
            if n < 4 {
                size += n as u64;
                return Err(VerifyError::Truncated(size));
            }
            magic_buf.copy_from_slice(&buf[..4]);
            let magic = u32::from_le_bytes(magic_buf);
            if magic != GGUF_MAGIC {
                return Err(VerifyError::BadMagic {
                    read: magic,
                    expected: GGUF_MAGIC,
                });
            }
            header_read = true;
        }
        hasher.update(&buf[..n]);
        size += n as u64;
    }

    if !header_read {
        return Err(VerifyError::Truncated(size));
    }

    let digest_hex = hex::encode(hasher.finalize());

    // (4) Compare size and digest.
    if size != manifest.artifact_size_bytes {
        return Err(VerifyError::SizeMismatch {
            actual: size,
            expected: manifest.artifact_size_bytes,
        });
    }
    if digest_hex != expected_hex {
        return Err(VerifyError::DigestMismatch {
            expected: expected_hex,
            actual: digest_hex,
        });
    }
    Ok((size, digest_hex))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ports::inference::{BackendKind, ModelFormat, ModelKind, ModelManifest};
    use sha2::{Digest, Sha256};

    fn sample_manifest(sha: &str, size: u64, format: ModelFormat) -> ModelManifest {
        ModelManifest {
            model_id: "test.model".to_string(),
            display_name: "Test".to_string(),
            family: "test".to_string(),
            model_kind: ModelKind::Chat,
            format,
            artifact_sha256: sha.to_string(),
            artifact_size_bytes: size,
            context_tokens: 2048,
            recommended_ram_bytes: 0,
            supported_backends: vec![BackendKind::LocalLm],
            prompt_template_id: "chat".to_string(),
            license_spdx: "MIT".to_string(),
        }
    }

    fn write_temp(name: &str, bytes: &[u8]) -> std::path::PathBuf {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join(name);
        std::fs::write(&path, bytes).expect("write");
        // Leak the dir so the file stays around for the test; OS cleanup will
        // reap it. tempfile::TempDir would remove on drop.
        std::mem::forget(dir);
        path
    }

    #[tokio::test]
    async fn verifies_a_real_gguf_file() {
        // Minimal in-memory GGUF: 4 magic bytes + arbitrary payload.
        let mut payload = Vec::new();
        payload.extend_from_slice(&0x46554747u32.to_le_bytes());
        payload.extend_from_slice(b"\x03\x00\x00\x00\x00\x00\x00\x00");
        payload.extend_from_slice(b"\x01\x00\x00\x00\x00\x00\x00\x00");
        let path = write_temp("good.gguf", &payload);

        let mut h = Sha256::new();
        h.update(&payload);
        let sha = hex::encode(h.finalize());
        let manifest = sample_manifest(&sha, payload.len() as u64, ModelFormat::Gguf);

        let (size, digest) = verify_gguf_artifact(&manifest, &path).await.unwrap();
        assert_eq!(size, payload.len() as u64);
        assert_eq!(digest, sha);
    }

    #[tokio::test]
    async fn rejects_non_gguf_magic() {
        let payload = b"JUNK extra data here that is more than four bytes";
        let path = write_temp("bad.gguf", payload);

        let mut h = Sha256::new();
        h.update(payload);
        let sha = hex::encode(h.finalize());
        let manifest = sample_manifest(&sha, payload.len() as u64, ModelFormat::Gguf);

        let err = verify_gguf_artifact(&manifest, &path).await.unwrap_err();
        assert!(matches!(err, VerifyError::BadMagic { .. }));
    }

    #[tokio::test]
    async fn rejects_size_mismatch() {
        let mut payload = Vec::new();
        payload.extend_from_slice(&0x46554747u32.to_le_bytes());
        payload.extend_from_slice(b"body");
        let path = write_temp("sized.gguf", &payload);

        let mut h = Sha256::new();
        h.update(&payload);
        let sha = hex::encode(h.finalize());

        // lie about the size by one byte
        let manifest = sample_manifest(&sha, payload.len() as u64 + 1, ModelFormat::Gguf);
        let err = verify_gguf_artifact(&manifest, &path).await.unwrap_err();
        assert!(matches!(err, VerifyError::SizeMismatch { .. }));
    }

    #[tokio::test]
    async fn rejects_digest_mismatch() {
        let mut payload = Vec::new();
        payload.extend_from_slice(&0x46554747u32.to_le_bytes());
        payload.extend_from_slice(b"body");
        let path = write_temp("digest.gguf", &payload);

        let wrong_sha = "0".repeat(64);
        let manifest = sample_manifest(&wrong_sha, payload.len() as u64, ModelFormat::Gguf);
        let err = verify_gguf_artifact(&manifest, &path).await.unwrap_err();
        assert!(matches!(err, VerifyError::DigestMismatch { .. }));
    }

    #[tokio::test]
    async fn rejects_unknown_path_with_valid_manifest_sha() {
        // ModelFormat only has one variant (Gguf), so we cannot exercise
        // FormatMismatch from outside ports. Instead, confirm that a
        // structurally valid manifest with a path that doesn't exist trips
        // the Path check.
        let manifest = sample_manifest(&"0".repeat(64), 1, ModelFormat::Gguf);
        let err = verify_gguf_artifact(&manifest, std::path::Path::new("/nonexistent/path.gguf"))
            .await
            .unwrap_err();
        assert!(matches!(err, VerifyError::Path(_)));
    }
}
