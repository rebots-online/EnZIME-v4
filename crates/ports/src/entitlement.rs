use async_trait::async_trait;
use domain::entitlement::CapabilityDecision;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderEntitlementEvent {
    pub provider: String,
    pub provider_event_id: String,
    pub subject_reference: String,
    pub product_id: String,
    pub event_kind: String,
    pub effective_at_ms: i64,
    pub expires_at_ms: Option<i64>,
    pub raw_event_digest: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntitlementRefreshRequest {
    pub device_id: String,
    pub provider_user_reference: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntitlementRestoreRequest {
    pub device_id: String,
    pub restore_token: String,
}

#[async_trait]
pub trait EntitlementSource: Send + Sync {
    fn provider_id(&self) -> &'static str;
    async fn refresh(
        &self,
        request: EntitlementRefreshRequest,
    ) -> Result<Vec<ProviderEntitlementEvent>, EntitlementError>;
    async fn restore(
        &self,
        request: EntitlementRestoreRequest,
    ) -> Result<Vec<ProviderEntitlementEvent>, EntitlementError>;
}

#[async_trait]
pub trait CapabilityPolicy: Send + Sync {
    async fn decide(
        &self,
        capability_key: &str,
        at_ms: i64,
    ) -> Result<CapabilityDecision, EntitlementError>;
    async fn all_decisions(
        &self,
        at_ms: i64,
    ) -> Result<Vec<CapabilityDecision>, EntitlementError>;
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum EntitlementError {
    #[error("offline")]
    Offline,
    #[error("provider unavailable")]
    ProviderUnavailable,
    #[error("restore failed")]
    RestoreFailed,
    #[error("artifact signature invalid")]
    InvalidArtifactSignature,
    #[error("artifact expired")]
    ArtifactExpired,
    #[error("issuer revoked")]
    IssuerRevoked,
    #[error("coupon already redeemed")]
    CouponAlreadyRedeemed,
    #[error("policy error: {0}")]
    Policy(String),
}
