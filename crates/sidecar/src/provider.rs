//! Ed25519 implementation of `ports::signature::SignatureProvider`.
//!
//! The provider mixes the caller-supplied `domain` into the signed bytes
//! using the exact prefix the architecture mandates for sidecar artifacts:
//! UTF-8 `domain || NUL`. A signature produced under one `domain` therefore
//! does not verify under any other, which is the property §9.4 ("signature
//! domain prefix is UTF-8 `enzime.sidecar.v1\0`") relies on.
//!
//! The provider stores a single local signing key and a registry of known
//! verifying keys keyed by `key_id`. Revocation flips a key into the
//! `RevokedKey` state without deleting it, so revoked signatures are
//! reported distinctly from unknown-key signatures (per the
//! `SignatureVerificationState` enum in §9.5).

use std::collections::{HashMap, HashSet};
use std::sync::RwLock;

use async_trait::async_trait;
use domain::trust::{SignatureEnvelope, SignatureVerification, SignatureVerificationState};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use ports::signature::{SignatureError, SignatureProvider};

/// In-memory Ed25519 keystore backing [`Ed25519Provider`].
///
/// One signing identity (the local device) plus a registry of verifying keys
/// for peers. The provider does not persist keys; persistence is the
/// platform layer's job. Keys are addressed by `key_id`, the same string
/// that appears in `SignatureEnvelope.key_id`.
#[derive(Default)]
pub struct Ed25519Keystore {
    local: Option<(String, SigningKey)>,
    verifying: HashMap<String, VerifyingKey>,
    revoked: HashSet<String>,
    labels: HashMap<String, String>,
}

impl Ed25519Keystore {
    /// Install or replace the local signing identity.
    pub fn set_local(&mut self, key_id: impl Into<String>, signing_key: SigningKey) {
        self.local = Some((key_id.into(), signing_key));
    }

    /// Register a verifying key, optionally with a human-readable label.
    pub fn add_verifying(
        &mut self,
        key_id: impl Into<String>,
        verifying_key: VerifyingKey,
        label: Option<String>,
    ) {
        let key_id = key_id.into();
        if let Some(label) = label {
            self.labels.insert(key_id.clone(), label);
        }
        self.verifying.insert(key_id, verifying_key);
    }

    /// Mark a `key_id` as revoked. Subsequent verifications against it
    /// return `RevokedKey` instead of `Verified`.
    pub fn revoke(&mut self, key_id: impl Into<String>) {
        self.revoked.insert(key_id.into());
    }
}

/// Ed25519 `SignatureProvider` with an in-memory keystore.
pub struct Ed25519Provider {
    keystore: RwLock<Ed25519Keystore>,
}

impl Ed25519Provider {
    pub fn new(keystore: Ed25519Keystore) -> Self {
        Self {
            keystore: RwLock::new(keystore),
        }
    }

    /// Empty provider — no local identity, no known peers.
    pub fn empty() -> Self {
        Self::new(Ed25519Keystore::default())
    }
}

/// Construct the canonical signed-payload prefix for the given domain:
/// `domain` followed by a single NUL byte. ARCHITECTURE.md §9.4 fixes the
/// sidecar domain prefix to UTF-8 `enzime.sidecar.v1\0`.
///
/// Public so callers can compute the same bytes the provider signs without
/// going through the trait (used by the sidecar import path, which signs
/// canonical envelope bytes directly).
pub fn domain_prefix(domain: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(domain.len() + 1);
    out.extend_from_slice(domain.as_bytes());
    out.push(0u8);
    out
}

#[async_trait]
impl SignatureProvider for Ed25519Provider {
    async fn sign(
        &self,
        domain: &str,
        canonical_payload: &[u8],
    ) -> Result<SignatureEnvelope, SignatureError> {
        let guard = self.keystore.read().unwrap();
        let (key_id, signing_key) = guard
            .local
            .as_ref()
            .ok_or(SignatureError::KeyUnavailable)?;
        if guard.revoked.contains(key_id) {
            return Err(SignatureError::KeyRevoked);
        }
        let mut message = domain_prefix(domain);
        message.extend_from_slice(canonical_payload);
        let sig: Signature = signing_key.sign(&message);
        Ok(SignatureEnvelope {
            algorithm: "ed25519".to_string(),
            key_id: key_id.clone(),
            signed_domain: domain.to_string(),
            value: sig.to_bytes().to_vec(),
        })
    }

    async fn verify(
        &self,
        domain: &str,
        canonical_payload: &[u8],
        envelope: &SignatureEnvelope,
    ) -> Result<SignatureVerification, SignatureError> {
        if envelope.algorithm != "ed25519" {
            return Err(SignatureError::UnsupportedAlgorithm);
        }
        let guard = self.keystore.read().unwrap();
        let key_id = &envelope.key_id;
        let label = guard.labels.get(key_id).cloned();
        if let Some(verifying_key) = guard.verifying.get(key_id) {
            let state = if guard.revoked.contains(key_id) {
                SignatureVerificationState::RevokedKey
            } else {
                let mut message = domain_prefix(domain);
                message.extend_from_slice(canonical_payload);
                let sig_arr: [u8; 64] = envelope
                    .value
                    .as_slice()
                    .try_into()
                    .map_err(|_| SignatureError::Invalid)?;
                let signature = Signature::from_bytes(&sig_arr);
                match verifying_key.verify(&message, &signature) {
                    Ok(()) => SignatureVerificationState::Verified,
                    Err(_) => SignatureVerificationState::Invalid,
                }
            };
            let explanation = match state {
                SignatureVerificationState::Verified => "signature verified".to_string(),
                SignatureVerificationState::Invalid => "signature did not validate".to_string(),
                SignatureVerificationState::UnknownKey => "no such key".to_string(),
                SignatureVerificationState::RevokedKey => "key is revoked".to_string(),
                SignatureVerificationState::Unsigned => "no signature present".to_string(),
            };
            Ok(SignatureVerification {
                state,
                key_id: key_id.clone(),
                signer_label: label,
                explanation,
            })
        } else {
            Ok(SignatureVerification {
                state: SignatureVerificationState::UnknownKey,
                key_id: key_id.clone(),
                signer_label: None,
                explanation: "no verifying key registered for key_id".to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;

    fn make_provider() -> (Ed25519Provider, VerifyingKey, String) {
        let mut rng = OsRng;
        let signing = SigningKey::generate(&mut rng);
        let verifying = signing.verifying_key();
        let key_id = "local".to_string();
        let mut store = Ed25519Keystore::default();
        store.set_local(key_id.clone(), signing);
        store.add_verifying(key_id.clone(), verifying, Some("test label".to_string()));
        (Ed25519Provider::new(store), verifying, key_id)
    }

    #[tokio::test]
    async fn sign_then_verify_round_trips() {
        let (provider, _vk, key_id) = make_provider();
        let payload = b"some canonical bytes";
        let env = provider
            .sign("enzime.sidecar.v1", payload)
            .await
            .expect("signing succeeds");
        assert_eq!(env.algorithm, "ed25519");
        assert_eq!(env.key_id, key_id);
        assert_eq!(env.signed_domain, "enzime.sidecar.v1");
        assert_eq!(env.value.len(), 64);
        let verification = provider
            .verify("enzime.sidecar.v1", payload, &env)
            .await
            .expect("verify call succeeds");
        assert_eq!(verification.state, SignatureVerificationState::Verified);
        assert_eq!(verification.key_id, key_id);
        assert_eq!(verification.signer_label.as_deref(), Some("test label"));
    }

    #[tokio::test]
    async fn signature_under_one_domain_does_not_verify_under_another() {
        let (provider, _vk, _key_id) = make_provider();
        let payload = b"canonical bytes that should be bound to a domain";
        let signed = provider
            .sign("enzime.sidecar.v1", payload)
            .await
            .expect("signing succeeds");
        // Same payload, different domain — must NOT verify.
        let cross = provider
            .verify("enzime.notebook.v1", payload, &signed)
            .await
            .expect("verify call still returns a result");
        assert_eq!(cross.state, SignatureVerificationState::Invalid);
        // Sanity: original domain still verifies.
        let ok = provider
            .verify("enzime.sidecar.v1", payload, &signed)
            .await
            .unwrap();
        assert_eq!(ok.state, SignatureVerificationState::Verified);
    }

    #[tokio::test]
    async fn unknown_key_reports_unknown() {
        let provider = Ed25519Provider::empty();
        let env = SignatureEnvelope {
            algorithm: "ed25519".to_string(),
            key_id: "ghost".to_string(),
            signed_domain: "enzime.sidecar.v1".to_string(),
            value: vec![0u8; 64],
        };
        let v = provider
            .verify("enzime.sidecar.v1", b"payload", &env)
            .await
            .unwrap();
        assert_eq!(v.state, SignatureVerificationState::UnknownKey);
    }

    #[tokio::test]
    async fn revoked_key_reports_revoked_even_if_signature_valid() {
        let (provider, _vk, key_id) = make_provider();
        let env = provider
            .sign("enzime.sidecar.v1", b"payload")
            .await
            .unwrap();
        {
            let mut store = provider.keystore.write().unwrap();
            store.revoke(&key_id);
        }
        let v = provider
            .verify("enzime.sidecar.v1", b"payload", &env)
            .await
            .unwrap();
        assert_eq!(v.state, SignatureVerificationState::RevokedKey);
    }

    #[tokio::test]
    async fn unsupported_algorithm_is_rejected() {
        let (provider, _vk, _key_id) = make_provider();
        let env = SignatureEnvelope {
            algorithm: "rsa-2048".to_string(),
            key_id: "local".to_string(),
            signed_domain: "enzime.sidecar.v1".to_string(),
            value: vec![0u8; 64],
        };
        let err = provider
            .verify("enzime.sidecar.v1", b"payload", &env)
            .await
            .unwrap_err();
        assert!(matches!(err, SignatureError::UnsupportedAlgorithm));
    }

    #[test]
    fn domain_prefix_appends_single_nul() {
        let p = domain_prefix("enzime.sidecar.v1");
        assert_eq!(p, b"enzime.sidecar.v1\0".to_vec());
        assert_eq!(p.last(), Some(&0u8));
        assert_eq!(p.iter().filter(|b| **b == 0).count(), 1);
    }
}
