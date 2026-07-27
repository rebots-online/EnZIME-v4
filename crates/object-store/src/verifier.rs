//! Format verifiers for staged object bytes.
//!
//! §6.3 step 6 of the commit algorithm runs a "format verifier" against the
//! staged file before the file is renamed into the content-addressed object
//! store. The object store itself does not know what a valid pack, model,
//! sidecar or backup looks like — those are owned by the replaceable engine
//! crates — so the verifier is supplied per commit by the caller.
//!
//! The trait is synchronous and operates on a path so that verifiers may
//! memory-map, seek, or stream the staged file as their format requires.

use std::path::Path;

use crate::error::FormatRejection;

/// A format verifier inspects the staged bytes (still in the transaction's
/// staging directory) and either accepts them or returns a [`FormatRejection`]
/// whose `code` and `reason` will be persisted with the quarantined bytes.
///
/// Implementations MUST be deterministic: the same staged path must always
/// produce the same verdict. Non-determinism would make quarantine decisions
/// un-auditable.
pub trait ObjectFormatVerifier: Send + Sync {
    /// Inspect the staged file at `staged_path`. The file has already been
    /// fsynced to disk and its SHA-256 and length have already been
    /// confirmed against the caller's declared values; this method is the
    /// format-level check (magic bytes, container structure, etc.).
    fn verify(&self, staged_path: &Path) -> Result<(), FormatRejection>;
}

/// A verifier that accepts any bytes. Use for objects whose internal format
/// is opaque to the object store — for example, an already-encrypted backup
/// blob whose structure is only meaningful after decryption upstream.
#[derive(Debug, Default, Clone, Copy)]
pub struct AcceptRawBytes;

impl ObjectFormatVerifier for AcceptRawBytes {
    fn verify(&self, _staged_path: &Path) -> Result<(), FormatRejection> {
        Ok(())
    }
}

/// A verifier that checks the leading magic bytes against an expected
/// prefix. Useful for objects with a known file signature (ZIM, GGUF, etc.).
#[derive(Debug, Clone)]
pub struct MagicPrefixVerifier {
    /// The expected leading bytes, in order.
    pub expected: Vec<u8>,
}

impl MagicPrefixVerifier {
    pub fn new(expected: impl Into<Vec<u8>>) -> Self {
        MagicPrefixVerifier {
            expected: expected.into(),
        }
    }
}

impl ObjectFormatVerifier for MagicPrefixVerifier {
    fn verify(&self, staged_path: &Path) -> Result<(), FormatRejection> {
        use std::io::Read;
        let mut file = std::fs::File::open(staged_path).map_err(|e| FormatRejection {
            code: "open-failed".to_string(),
            reason: e.to_string(),
        })?;
        let mut buf = vec![0u8; self.expected.len()];
        let n = file.read(&mut buf).map_err(|e| FormatRejection {
            code: "read-failed".to_string(),
            reason: e.to_string(),
        })?;
        if n != self.expected.len() {
            return Err(FormatRejection::new(
                "too-short",
                format!(
                    "staged bytes are {} byte(s); expected at least {} for magic prefix",
                    n,
                    self.expected.len()
                ),
            ));
        }
        if buf != self.expected {
            return Err(FormatRejection::new(
                "bad-magic",
                format!(
                    "leading bytes {:02x?} did not match expected {:02x?}",
                    buf, self.expected
                ),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_tmp(name: &str, bytes: &[u8]) -> std::path::PathBuf {
        let dir = std::env::var_os("CARGO_TARGET_TMPDIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        let path = dir.join(name);
        {
            let mut f = std::fs::File::create(&path).unwrap();
            f.write_all(bytes).unwrap();
            f.sync_all().unwrap();
        }
        path
    }

    #[test]
    fn accept_raw_bytes_always_passes() {
        let p = write_tmp("verifier-empty.bin", b"");
        assert!(AcceptRawBytes.verify(&p).is_ok());
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn magic_verifier_accepts_match() {
        let p = write_tmp("verifier-zim.bin", &[0x5a, 0x49, 0x4d, 0x04, 0x00, 0x00]);
        let v = MagicPrefixVerifier::new([0x5a, 0x49, 0x4d, 0x04]);
        assert!(v.verify(&p).is_ok());
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn magic_verifier_rejects_short_file() {
        let p = write_tmp("verifier-short.bin", &[0x5a, 0x49]);
        let v = MagicPrefixVerifier::new([0x5a, 0x49, 0x4d, 0x04]);
        let err = v.verify(&p).unwrap_err();
        assert_eq!(err.code, "too-short");
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn magic_verifier_rejects_wrong_magic() {
        let p = write_tmp("verifier-wrong.bin", &[0xff, 0xff, 0x4d, 0x04]);
        let v = MagicPrefixVerifier::new([0x5a, 0x49, 0x4d, 0x04]);
        let err = v.verify(&p).unwrap_err();
        assert_eq!(err.code, "bad-magic");
        let _ = std::fs::remove_file(&p);
    }
}
