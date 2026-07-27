use crate::trust::{SignatureEnvelope, SignatureVerification};
use async_trait::async_trait;

#[async_trait]
pub trait SignatureProvider: Send + Sync {
    async fn sign(
        &self,
        domain: &str,
        canonical_payload: &[u8],
    ) -> Result<SignatureEnvelope, SignatureError>;
    async fn verify(
        &self,
        domain: &str,
        canonical_payload: &[u8],
        envelope: &SignatureEnvelope,
    ) -> Result<SignatureVerification, SignatureError>;
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum SignatureError {
    #[error("key unavailable")]
    KeyUnavailable,
    #[error("key revoked")]
    KeyRevoked,
    #[error("signature invalid")]
    Invalid,
    #[error("unsupported algorithm")]
    UnsupportedAlgorithm,
    #[error("keystore error: {0}")]
    KeyStore(String),
}
