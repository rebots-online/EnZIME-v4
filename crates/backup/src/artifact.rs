//! On-disk artifact container.
//!
//! Layout (all integers big-endian, fixed-width):
//!
//! ```text
//!   offset 0..4    : magic bytes b"ENZB"
//!   offset 4       : format version byte (1)
//!   offset 5..9    : u32 manifest length (bytes of manifest JSON that follow)
//!   offset 9..N    : manifest JSON (UTF-8), exactly manifest_length bytes
//!   offset N..     : payload bytes (ciphertext + Poly1305 tag if encrypted,
//!                    else the cleartext payload bytes themselves)
//! ```
//!
//! The manifest is **not** length-prefixed by the payload; the payload is
//! whatever remains in the file after the manifest. This is deliberate —
//! the manifest records `payload_byte_length` (cleartext) and the cipher
//! implicitly defines the ciphertext length, so a reader can detect
//! truncation without an extra framing field.

use std::io;

use bytes::BytesMut;

use crate::error::BackupError;
use crate::manifest::{ARTIFACT_FORMAT_VERSION, ARTIFACT_MAGIC, BackupManifest};

/// Serialise a complete artifact (header + manifest + payload) into one
/// `Vec<u8>` for the creator to stream to the destination.
pub fn encode(manifest: &BackupManifest, payload: &[u8]) -> Result<Vec<u8>, BackupError> {
    let manifest_json = manifest.to_canonical_json()?;
    if manifest_json.len() > u32::MAX as usize {
        return Err(BackupError::FormatInvalid(format!(
            "manifest is {} bytes, exceeds u32",
            manifest_json.len()
        )));
    }
    let mut out = BytesMut::with_capacity(9 + manifest_json.len() + payload.len());
    out.extend_from_slice(ARTIFACT_MAGIC);
    out.extend_from_slice(&[ARTIFACT_FORMAT_VERSION]);
    out.extend_from_slice(&(manifest_json.len() as u32).to_be_bytes());
    out.extend_from_slice(&manifest_json);
    out.extend_from_slice(payload);
    Ok(out.freeze().to_vec())
}

/// The decoded artifact.
#[derive(Clone, Debug)]
pub struct DecodedArtifact {
    pub manifest: BackupManifest,
    /// Payload bytes exactly as they appeared in the file (ciphertext if
    /// the manifest says `encrypted == true`, cleartext otherwise).
    pub raw_payload: Vec<u8>,
    /// Manifest JSON bytes, kept around so callers can re-hash them for
    /// the §6.2 `backups.manifest_sha256` column.
    pub manifest_json: Vec<u8>,
}

/// Decode an artifact from a complete in-memory buffer.
pub fn decode(buf: &[u8]) -> Result<DecodedArtifact, BackupError> {
    let mut cursor = 0usize;
    let magic = take(&mut cursor, buf, 4)?;
    if magic != ARTIFACT_MAGIC {
        return Err(BackupError::FormatInvalid(format!(
            "artifact magic bytes are {:?}, expected {:?}",
            magic, ARTIFACT_MAGIC
        )));
    }
    let version = take(&mut cursor, buf, 1)?;
    if version[0] != ARTIFACT_FORMAT_VERSION {
        return Err(BackupError::FormatInvalid(format!(
            "artifact format version is {}, expected {}",
            version[0], ARTIFACT_FORMAT_VERSION
        )));
    }
    let length_bytes = take(&mut cursor, buf, 4)?;
    let manifest_len = u32::from_be_bytes([
        length_bytes[0],
        length_bytes[1],
        length_bytes[2],
        length_bytes[3],
    ]) as usize;
    let manifest_json = take(&mut cursor, buf, manifest_len)?;
    let manifest: BackupManifest = serde_json::from_slice(manifest_json)?;
    let raw_payload = buf[cursor..].to_vec();
    manifest.validate()?;
    Ok(DecodedArtifact {
        manifest,
        raw_payload,
        manifest_json: manifest_json.to_vec(),
    })
}

fn take<'a>(cursor: &mut usize, buf: &'a [u8], n: usize) -> Result<&'a [u8], BackupError> {
    if *cursor + n > buf.len() {
        return Err(BackupError::FormatInvalid(format!(
            "artifact truncated: need {} bytes at offset {}, only {} remain",
            n,
            *cursor,
            buf.len() - *cursor
        )));
    }
    let slice = &buf[*cursor..*cursor + n];
    *cursor += n;
    Ok(slice)
}

/// Convenience: SHA-256 of an entire encoded artifact file, lowercase hex.
/// Used to fill `BackupArtifactSummary.artifactSha256`.
pub fn artifact_sha256_hex(buf: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(buf);
    hex::encode(hasher.finalize())
}

/// Convert an [`io::Error`] that we ourselves raised during encode/decode
/// into a `FormatInvalid`. (Currently unused because we operate on
/// in-memory buffers, but kept for future streaming paths.)
#[allow(dead_code)]
pub(crate) fn io_to_format(e: io::Error) -> BackupError {
    BackupError::FormatInvalid(format!("io: {e}"))
}
