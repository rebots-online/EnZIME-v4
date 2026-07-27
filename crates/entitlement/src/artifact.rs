//! §13.3 Signed offline entitlement artifact.
//!
//! Wire format: CBOR, media type `application/vnd.enzime.entitlement+cbor`.
//! Canonical CDDL: `schemas/entitlement/enzime.entitlement.cddl`.
//!
//! Signing canonical bytes are the CBOR encoding of the artifact with the
//! `signature` field omitted (see `UnsignedArtifact` below). The signing
//! domain string is `enzime.entitlement.v1\0`. The `issuer_key_id` at the
//! top of the artifact must match the `key_id` inside the signature envelope,
//! and the envelope's `signed_domain` must equal `ENTITLEMENT_SIGNING_DOMAIN`.
//!
//! This module is purely offline: it never queries a provider, never imports a
//! provider type. It calls the `ports::signature::SignatureProvider` port for
//! the actual cryptographic operation; the port impl is supplied by the
//! caller (release build wires the real verifier in via a separate adapter).

use std::collections::HashSet;

use domain::trust::{SignatureEnvelope, SignatureVerificationState};
use ports::entitlement::EntitlementError;
use ports::signature::SignatureProvider;
use serde::{Deserialize, Serialize};

/// Schema discriminator carried in every artifact. Checked on encode, sign
/// and verify — any other value is rejected as an invalid signature (the
/// payload cannot be trusted to mean what we think it means).
pub const ARTIFACT_SCHEMA: &str = "enzime.entitlement.cbor.v1";

/// Signing domain string for entitlement artifacts. The trailing NUL is part
/// of the domain and is checked on verify.
pub const ENTITLEMENT_SIGNING_DOMAIN: &str = "enzime.entitlement.v1\0";

/// Media type carried on the wire for an encoded artifact.
pub const MEDIA_TYPE: &str = "application/vnd.enzime.entitlement+cbor";

/// Required length, in bytes, of the per-artifact nonce.
pub const NONCE_LEN: usize = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubjectKind {
    Device,
    Household,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Subject {
    pub kind: SubjectKind,
    pub subject_id: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GrantKind {
    Perpetual,
    Term,
    Quantity,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Grant {
    pub capability_key: String,
    pub grant_kind: GrantKind,
    pub quantity: Option<u64>,
    pub valid_until_ms: Option<i64>,
}

impl Grant {
    /// A perpetual grant has no end. A term or quantity grant is bounded by
    /// `valid_until_ms`; if it is `None`, the grant is treated as unbounded
    /// (defensive — issuers should set `valid_until_ms` for non-perpetual
    /// grants).
    pub fn is_in_effect_at(&self, at_ms: i64) -> bool {
        match self.grant_kind {
            GrantKind::Perpetual => true,
            GrantKind::Term | GrantKind::Quantity => {
                self.valid_until_ms.map_or(true, |t| at_ms <= t)
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProviderReceipt {
    pub provider: String,
    pub event_digest: String,
}

/// Flat signature envelope as carried inside an artifact. CDDL-faithful: the
/// `value` field is a CBOR byte string, not a sequence of integers.
///
/// This is a local alias of `domain::trust::SignatureEnvelope` whose `value`
/// is `serde_bytes::ByteBuf` so that the wire encoding matches the CDDL
/// signature term (`value: bstr`). Round-trips losslessly with the domain
/// type via the provided `From` impls.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ArtifactSignature {
    pub algorithm: String,
    pub key_id: String,
    pub signed_domain: String,
    #[serde(with = "serde_bytes")]
    pub value: Vec<u8>,
}

impl From<SignatureEnvelope> for ArtifactSignature {
    fn from(e: SignatureEnvelope) -> Self {
        Self {
            algorithm: e.algorithm,
            key_id: e.key_id,
            signed_domain: e.signed_domain,
            value: e.value,
        }
    }
}

impl From<ArtifactSignature> for SignatureEnvelope {
    fn from(s: ArtifactSignature) -> Self {
        Self {
            algorithm: s.algorithm,
            key_id: s.key_id,
            signed_domain: s.signed_domain,
            value: s.value,
        }
    }
}

/// Signed offline entitlement artifact — the full wire format. Stored and
/// transported as CBOR via `encode_artifact` / `decode_artifact`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EntitlementArtifact {
    pub schema: String,
    pub artifact_id: String,
    pub issuer_key_id: String,
    pub subject: Subject,
    pub issued_at_ms: i64,
    pub not_before_ms: i64,
    pub expires_at_ms: Option<i64>,
    pub grants: Vec<Grant>,
    pub provider_receipts: Vec<ProviderReceipt>,
    #[serde(with = "serde_bytes")]
    pub nonce: Vec<u8>,
    pub signature: ArtifactSignature,
}

/// Canonical form used for signing/verifying — every artifact field except
/// `signature`. The signing canonical bytes are the CBOR encoding of this
/// structure under the `ENTITLEMENT_SIGNING_DOMAIN`. Field order is fixed
/// by struct declaration order, so the same logical artifact always produces
/// the same canonical byte sequence (deterministic encoding is what matters
/// for signature replay, not RFC-8949 canonical key sorting).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct UnsignedArtifact {
    schema: String,
    artifact_id: String,
    issuer_key_id: String,
    subject: Subject,
    issued_at_ms: i64,
    not_before_ms: i64,
    expires_at_ms: Option<i64>,
    grants: Vec<Grant>,
    provider_receipts: Vec<ProviderReceipt>,
    #[serde(with = "serde_bytes")]
    nonce: Vec<u8>,
}

impl EntitlementArtifact {
    /// True iff `at_ms` falls inside `[not_before_ms, expires_at_ms]`. A
    /// perpetual artifact (`expires_at_ms == None`) is active forever once
    /// `not_before_ms` has passed.
    pub fn is_active_at(&self, at_ms: i64) -> bool {
        at_ms >= self.not_before_ms && self.expires_at_ms.map_or(true, |e| at_ms <= e)
    }

    /// Structural validation that does not depend on the signature. Used at
    /// sign and verify time to fail fast on malformed artifacts. The error
    /// variant is `Policy` because these are codec-level rejections, not
    /// signature failures.
    pub fn validate_shape(&self) -> Result<(), EntitlementError> {
        if self.schema != ARTIFACT_SCHEMA {
            return Err(EntitlementError::Policy(format!(
                "schema must be '{ARTIFACT_SCHEMA}', was '{}'",
                self.schema
            )));
        }
        if self.nonce.len() != NONCE_LEN {
            return Err(EntitlementError::Policy(format!(
                "nonce must be {NONCE_LEN} bytes, was {}",
                self.nonce.len()
            )));
        }
        if self.grants.is_empty() {
            return Err(EntitlementError::Policy(
                "artifact must carry at least one grant".to_string(),
            ));
        }
        if self.artifact_id.is_empty() {
            return Err(EntitlementError::Policy(
                "artifact_id must not be empty".to_string(),
            ));
        }
        if self.issuer_key_id.is_empty() {
            return Err(EntitlementError::Policy(
                "issuer_key_id must not be empty".to_string(),
            ));
        }
        Ok(())
    }
}

/// Compute the canonical signing bytes for `artifact`. Excludes the
/// `signature` field. Deterministic for equal inputs.
pub fn canonical_bytes_for_artifact(
    artifact: &EntitlementArtifact,
) -> Result<Vec<u8>, EntitlementError> {
    let unsigned = UnsignedArtifact {
        schema: artifact.schema.clone(),
        artifact_id: artifact.artifact_id.clone(),
        issuer_key_id: artifact.issuer_key_id.clone(),
        subject: artifact.subject.clone(),
        issued_at_ms: artifact.issued_at_ms,
        not_before_ms: artifact.not_before_ms,
        expires_at_ms: artifact.expires_at_ms,
        grants: artifact.grants.clone(),
        provider_receipts: artifact.provider_receipts.clone(),
        nonce: artifact.nonce.clone(),
    };
    let mut buf = Vec::new();
    ciborium::into_writer(&unsigned, &mut buf)
        .map_err(|e| EntitlementError::Policy(format!("cbor encode failed: {e}")))?;
    Ok(buf)
}

/// Encode the full signed artifact for storage or transport.
pub fn encode_artifact(artifact: &EntitlementArtifact) -> Result<Vec<u8>, EntitlementError> {
    let mut buf = Vec::new();
    ciborium::into_writer(artifact, &mut buf)
        .map_err(|e| EntitlementError::Policy(format!("cbor encode failed: {e}")))?;
    Ok(buf)
}

/// Decode a stored/transported artifact. Verifies the schema discriminator
/// and structural shape; does NOT verify the signature — call `verify_artifact`
/// before trusting the contents.
pub fn decode_artifact(bytes: &[u8]) -> Result<EntitlementArtifact, EntitlementError> {
    let artifact: EntitlementArtifact = ciborium::from_reader(bytes)
        .map_err(|e| EntitlementError::Policy(format!("cbor decode failed: {e}")))?;
    artifact.validate_shape()?;
    Ok(artifact)
}

/// Generate a fresh random 32-byte nonce using the platform RNG.
pub fn generate_nonce() -> Result<Vec<u8>, EntitlementError> {
    let mut buf = vec![0u8; NONCE_LEN];
    getrandom::getrandom(&mut buf)
        .map_err(|_| EntitlementError::Policy("nonce generation failed".to_string()))?;
    Ok(buf)
}

/// Sign `artifact` in place using `signer`. The artifact's `signature` field
/// is overwritten with the freshly produced envelope.
///
/// Pre-conditions enforced before signing:
/// - `schema == ARTIFACT_SCHEMA`
/// - `nonce.len() == NONCE_LEN`
/// - `grants` non-empty, `artifact_id`/`issuer_key_id` non-empty
/// - `issuer_key_id` equals the `key_id` reported by the signer (otherwise
///   the caller has mis-stated who is issuing).
pub async fn sign_artifact(
    artifact: &mut EntitlementArtifact,
    signer: &(impl SignatureProvider + Send + Sync),
) -> Result<(), EntitlementError> {
    artifact.validate_shape()?;
    let canonical = canonical_bytes_for_artifact(artifact)?;
    let envelope = signer
        .sign(ENTITLEMENT_SIGNING_DOMAIN, &canonical)
        .await
        .map_err(|e| EntitlementError::Policy(format!("signer error: {e}")))?;
    if envelope.signed_domain != ENTITLEMENT_SIGNING_DOMAIN {
        return Err(EntitlementError::Policy(format!(
            "signer did not honour the entitlement signing domain (got '{}')",
            envelope.signed_domain
        )));
    }
    if artifact.issuer_key_id != envelope.key_id {
        return Err(EntitlementError::Policy(format!(
            "issuer_key_id '{}' does not match signing key '{}'",
            artifact.issuer_key_id, envelope.key_id
        )));
    }
    artifact.signature = envelope.into();
    Ok(())
}

/// Verify `artifact` against `verifier`. This checks the signature, the
/// signing-domain string, the issuer/key-id binding, and the issuer
/// revocation set. It does NOT consult the wall-clock window — that is the
/// policy's job (`crate::policy`). Returning `Ok(())` here means "the bytes
/// are authentic and unrevoked"; whether they are *currently active* is a
/// separate question.
pub async fn verify_artifact(
    artifact: &EntitlementArtifact,
    verifier: &(impl SignatureProvider + Send + Sync),
    revoked_issuers: &HashSet<String>,
) -> Result<(), EntitlementError> {
    // Codec-shape rejections.
    if artifact.schema != ARTIFACT_SCHEMA
        || artifact.nonce.len() != NONCE_LEN
        || artifact.grants.is_empty()
        || artifact.artifact_id.is_empty()
    {
        return Err(EntitlementError::InvalidArtifactSignature);
    }
    // Top-level issuer key must equal signature key id.
    if artifact.issuer_key_id != artifact.signature.key_id {
        return Err(EntitlementError::InvalidArtifactSignature);
    }
    // Envelope signed-domain must match the entitlement domain.
    if artifact.signature.signed_domain != ENTITLEMENT_SIGNING_DOMAIN {
        return Err(EntitlementError::InvalidArtifactSignature);
    }
    // Revocation check.
    if revoked_issuers.contains(&artifact.issuer_key_id) {
        return Err(EntitlementError::IssuerRevoked);
    }
    // Re-canonicalise and verify.
    let canonical = canonical_bytes_for_artifact(artifact)?;
    let envelope: SignatureEnvelope = artifact.signature.clone().into();
    let v = verifier
        .verify(ENTITLEMENT_SIGNING_DOMAIN, &canonical, &envelope)
        .await
        .map_err(|_| EntitlementError::InvalidArtifactSignature)?;
    match v.state {
        SignatureVerificationState::Verified => Ok(()),
        SignatureVerificationState::RevokedKey => Err(EntitlementError::IssuerRevoked),
        SignatureVerificationState::Unsigned
        | SignatureVerificationState::Invalid
        | SignatureVerificationState::UnknownKey => Err(EntitlementError::InvalidArtifactSignature),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use ports::signature::{SignatureError, SignatureProvider};
    use sha2::{Digest, Sha256};

    /// Test-only HMAC-SHA256 signature provider. NOT a real signature — it
    /// is a MAC keyed by a single symmetric key. Sufficient for exercising
    /// the codec/policy wiring without pulling a public-key crate into the
    /// workspace; the cryptographic strength of the test signature is not
    /// what these tests verify.
    struct HmacTestSigner {
        key_id: String,
        key: Vec<u8>,
        trusted_key_ids: HashSet<String>,
    }

    impl HmacTestSigner {
        fn new(key_id: &str, key: &[u8]) -> Self {
            let mut trusted = HashSet::new();
            trusted.insert(key_id.to_string());
            Self {
                key_id: key_id.to_string(),
                key: key.to_vec(),
                trusted_key_ids: trusted,
            }
        }

        fn mac(&self, domain: &str, payload: &[u8]) -> [u8; 32] {
            // HMAC-SHA256, RFC 2104, block size 64.
            const BLOCK: usize = 64;
            let mut k = [0u8; BLOCK];
            if self.key.len() > BLOCK {
                let mut h = Sha256::new();
                h.update(&self.key);
                let d = h.finalize();
                k[..32].copy_from_slice(&d);
            } else {
                k[..self.key.len()].copy_from_slice(&self.key);
            }
            let mut ipad = [0u8; BLOCK];
            let mut opad = [0u8; BLOCK];
            for i in 0..BLOCK {
                ipad[i] = k[i] ^ 0x36;
                opad[i] = k[i] ^ 0x5c;
            }
            let mut inner = Sha256::new();
            inner.update(ipad);
            inner.update(domain.as_bytes());
            inner.update(payload);
            let inner_h = inner.finalize();
            let mut outer = Sha256::new();
            outer.update(opad);
            outer.update(&inner_h);
            let outer_h = outer.finalize();
            let mut out = [0u8; 32];
            out.copy_from_slice(&outer_h);
            out
        }
    }

    #[async_trait]
    impl SignatureProvider for HmacTestSigner {
        async fn sign(
            &self,
            domain: &str,
            canonical_payload: &[u8],
        ) -> Result<SignatureEnvelope, SignatureError> {
            Ok(SignatureEnvelope {
                algorithm: "test-hmac-sha256".to_string(),
                key_id: self.key_id.clone(),
                signed_domain: domain.to_string(),
                value: self.mac(domain, canonical_payload).to_vec(),
            })
        }
        async fn verify(
            &self,
            domain: &str,
            canonical_payload: &[u8],
            envelope: &SignatureEnvelope,
        ) -> Result<domain::trust::SignatureVerification, SignatureError> {
            use domain::trust::SignatureVerification;
            if envelope.signed_domain != domain {
                return Ok(SignatureVerification {
                    state: SignatureVerificationState::Invalid,
                    key_id: envelope.key_id.clone(),
                    signer_label: None,
                    explanation: "signed_domain mismatch".to_string(),
                });
            }
            if !self.trusted_key_ids.contains(&envelope.key_id) {
                return Ok(SignatureVerification {
                    state: SignatureVerificationState::UnknownKey,
                    key_id: envelope.key_id.clone(),
                    signer_label: None,
                    explanation: "unknown key id".to_string(),
                });
            }
            let expected = self.mac(domain, canonical_payload);
            if envelope.value.as_slice() == expected.as_slice() {
                Ok(SignatureVerification {
                    state: SignatureVerificationState::Verified,
                    key_id: envelope.key_id.clone(),
                    signer_label: None,
                    explanation: "verified".to_string(),
                })
            } else {
                Ok(SignatureVerification {
                    state: SignatureVerificationState::Invalid,
                    key_id: envelope.key_id.clone(),
                    signer_label: None,
                    explanation: "value mismatch".to_string(),
                })
            }
        }
    }

    fn fresh_artifact() -> EntitlementArtifact {
        EntitlementArtifact {
            schema: ARTIFACT_SCHEMA.to_string(),
            artifact_id: "art-001".to_string(),
            issuer_key_id: "issuer-7".to_string(),
            subject: Subject {
                kind: SubjectKind::Device,
                subject_id: "device-abc".to_string(),
            },
            issued_at_ms: 1_000,
            not_before_ms: 1_000,
            expires_at_ms: Some(10_000),
            grants: vec![Grant {
                capability_key: "memory.lore".to_string(),
                grant_kind: GrantKind::Perpetual,
                quantity: None,
                valid_until_ms: None,
            }],
            provider_receipts: vec![ProviderReceipt {
                provider: "revenuecat".to_string(),
                event_digest: "deadbeef".to_string(),
            }],
            nonce: generate_nonce().unwrap(),
            signature: ArtifactSignature {
                algorithm: String::new(),
                key_id: String::new(),
                signed_domain: String::new(),
                value: Vec::new(),
            },
        }
    }

    #[tokio::test]
    async fn sign_then_verify_roundtrip() {
        let signer = HmacTestSigner::new("issuer-7", b"secret-key-bytes");
        let mut art = fresh_artifact();
        sign_artifact(&mut art, &signer).await.unwrap();
        let bytes = encode_artifact(&art).unwrap();
        let decoded = decode_artifact(&bytes).unwrap();
        let revoked = HashSet::new();
        verify_artifact(&decoded, &signer, &revoked).await.unwrap();
    }

    #[tokio::test]
    async fn verify_rejects_tampered_grant() {
        let signer = HmacTestSigner::new("issuer-7", b"secret-key-bytes");
        let mut art = fresh_artifact();
        sign_artifact(&mut art, &signer).await.unwrap();
        // Tamper: flip capability_key after signing.
        art.grants[0].capability_key = "reader.unlimited_packs".to_string();
        let revoked = HashSet::new();
        let err = verify_artifact(&art, &signer, &revoked).await.unwrap_err();
        assert!(matches!(err, EntitlementError::InvalidArtifactSignature));
    }

    #[tokio::test]
    async fn verify_rejects_wrong_signing_key() {
        let signer = HmacTestSigner::new("issuer-7", b"secret-key-bytes");
        let mut art = fresh_artifact();
        // Claim issuer-7 in the artifact but actually sign with a different key.
        art.issuer_key_id = "issuer-7".to_string();
        let impostor = HmacTestSigner::new("issuer-9", b"different-key");
        let err = sign_artifact(&mut art, &impostor)
            .await
            .expect_err("signer key_id disagrees with issuer_key_id");
        assert!(matches!(err, EntitlementError::Policy(_)));
        let _ = signer;
    }

    #[tokio::test]
    async fn verify_rejects_revoked_issuer() {
        let signer = HmacTestSigner::new("issuer-7", b"secret-key-bytes");
        let mut art = fresh_artifact();
        sign_artifact(&mut art, &signer).await.unwrap();
        let mut revoked = HashSet::new();
        revoked.insert("issuer-7".to_string());
        let err = verify_artifact(&art, &signer, &revoked).await.unwrap_err();
        assert!(matches!(err, EntitlementError::IssuerRevoked));
    }

    #[tokio::test]
    async fn verify_rejects_unknown_signing_key_on_wire() {
        // Self-consistent on the wire, but the verifier does not trust the key.
        let impostor = HmacTestSigner::new("issuer-7", b"secret-key-bytes");
        let mut art = fresh_artifact();
        sign_artifact(&mut art, &impostor).await.unwrap();
        // A verifier that trusts no keys.
        let untrusting = UntrustingSigner;
        let revoked = HashSet::new();
        let err = verify_artifact(&art, &untrusting, &revoked)
            .await
            .unwrap_err();
        assert!(matches!(err, EntitlementError::InvalidArtifactSignature));
    }

    struct UntrustingSigner;
    #[async_trait]
    impl SignatureProvider for UntrustingSigner {
        async fn sign(
            &self,
            _: &str,
            _: &[u8],
        ) -> Result<SignatureEnvelope, SignatureError> {
            Err(SignatureError::KeyUnavailable)
        }
        async fn verify(
            &self,
            _: &str,
            _: &[u8],
            envelope: &SignatureEnvelope,
        ) -> Result<domain::trust::SignatureVerification, SignatureError> {
            Ok(domain::trust::SignatureVerification {
                state: SignatureVerificationState::UnknownKey,
                key_id: envelope.key_id.clone(),
                signer_label: None,
                explanation: "no keys trusted".to_string(),
            })
        }
    }

    #[tokio::test]
    async fn canonical_bytes_are_deterministic() {
        let signer = HmacTestSigner::new("issuer-7", b"secret-key-bytes");
        let mut art = fresh_artifact();
        sign_artifact(&mut art, &signer).await.unwrap();
        let a = canonical_bytes_for_artifact(&art).unwrap();
        let b = canonical_bytes_for_artifact(&art).unwrap();
        assert_eq!(a, b, "canonical bytes must be reproducible");
        // Mutate the signature field — canonical bytes must NOT change.
        let saved = art.signature.clone();
        art.signature.algorithm = "tampered".to_string();
        let c = canonical_bytes_for_artifact(&art).unwrap();
        assert_eq!(a, c, "canonical bytes must not depend on signature field");
        art.signature = saved;
    }

    #[test]
    fn nonce_generation_produces_32_random_bytes() {
        let a = generate_nonce().unwrap();
        let b = generate_nonce().unwrap();
        assert_eq!(a.len(), NONCE_LEN);
        assert_eq!(b.len(), NONCE_LEN);
        assert_ne!(a, b, "nonce must not repeat in practice");
    }

    #[test]
    fn validate_shape_rejects_bad_schema() {
        let mut art = fresh_artifact();
        art.schema = "wrong.schema.v0".to_string();
        assert!(art.validate_shape().is_err());
    }

    #[test]
    fn validate_shape_rejects_short_nonce() {
        let mut art = fresh_artifact();
        art.nonce = vec![0u8; 16];
        assert!(art.validate_shape().is_err());
    }
}
