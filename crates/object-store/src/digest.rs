//! SHA-256 streaming and helpers.
//!
//! The §6.3 commit algorithm requires that the digest be recomputed from the
//! staged bytes (step 4), not carried over from the in-memory stream. This
//! module provides the streaming hasher used at that step, plus a re-read
//! helper that hashes an already-flushed staging file.

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::error::ObjectStoreError;

/// Streaming SHA-256 hasher. Thin newtype wrapper so call sites read as the
/// §6.3 step ("calculate SHA-256 from staged bytes") rather than as a generic
/// crypto primitive.
pub struct StreamingSha256 {
    inner: Sha256,
    bytes_in: u64,
}

impl StreamingSha256 {
    pub fn new() -> Self {
        StreamingSha256 {
            inner: Sha256::new(),
            bytes_in: 0,
        }
    }

    pub fn update(&mut self, bytes: &[u8]) {
        self.bytes_in += bytes.len() as u64;
        self.inner.update(bytes);
    }

    pub fn finalize(self) -> (String, u64) {
        let hex = hex::encode(self.inner.finalize());
        (hex, self.bytes_in)
    }
}

impl Default for StreamingSha256 {
    fn default() -> Self {
        Self::new()
    }
}

/// Re-open a flushed file and hash it, returning `(lowercase_sha256_hex,
/// bytes_read)`. Used at §6.3 step 4 to compute the digest from the bytes
/// that actually landed on disk rather than from the in-memory stream — a
/// defence-in-depth against in-memory corruption between stream and disk.
pub fn hash_file(path: &Path) -> Result<(String, u64), ObjectStoreError> {
    let file = File::open(path)?;
    let mut reader = BufReader::with_capacity(64 * 1024, file);
    let mut hasher = StreamingSha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn known_answer(input: &[u8], expected_hex: &str) {
        let mut h = StreamingSha256::new();
        h.update(input);
        let (hex, n) = h.finalize();
        assert_eq!(hex, expected_hex);
        assert_eq!(n, input.len() as u64);
    }

    #[test]
    fn empty_input_matches_known_answer() {
        // sha256("") = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        known_answer(
            b"",
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        );
    }

    #[test]
    fn abc_matches_known_answer() {
        // sha256("abc") = ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad
        known_answer(
            b"abc",
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        );
    }

    #[test]
    fn hashing_file_re_reads_disk_bytes() {
        let dir = std::env::var_os("CARGO_TARGET_TMPDIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        let path = dir.join("object-store-digest-test.bin");
        {
            let mut f = std::fs::File::create(&path).unwrap();
            f.write_all(b"abc").unwrap();
            f.sync_all().unwrap();
        }
        let (hex, n) = hash_file(&path).unwrap();
        assert_eq!(n, 3);
        assert_eq!(
            hex,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        let _ = std::fs::remove_file(&path);
    }
}
