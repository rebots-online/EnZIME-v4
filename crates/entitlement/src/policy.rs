//! §13.5 Capability gating — the offline `CapabilityPolicy`.
//!
//! `OfflineCapabilityPolicy` is the production policy. It is given the
//! currently verified `EntitlementArtifact` (decoded and verified elsewhere
//! via `crate::verify_artifact`) and a set of revoked issuer key ids, and
//! answers `decide(capability_key, at_ms)` purely from that local state.
//!
//! ## The binding rule (A-09)
//!
//! Commerce cannot revoke ownership. Concretely:
//!
//! - `reader.core` is **always** allowed, with `source = Core`. It does not
//!   consult the artifact, the revocation list, or the clock.
//! - A **perpetual** grant, once present in a verified artifact, is allowed
//!   forever from `source = SignedArtifact`, even past the artifact's
//!   `expires_at_ms`. The artifact's own expiry bounds non-perpetual grants
//!   only.
//! - Denied paid actions never destroy or hide existing user data, never
//!   block export of user-created data, and never block core reading. The
//!   policy itself does not touch user data; it only produces a decision.
//!   The explanation string on a denied decision states the capability and
//!   the restore/purchase path so the UI can show it.
//!
//! ## Grace window
//!
//! A non-perpetual grant whose validity window has elapsed is allowed for a
//! configurable grace period past expiry, with `source = Gracepolicy`. The
//! default is 72 hours, matching a typical offline-grace expectation in the
//! absence of an explicit number in the architecture. After the grace
//! window the decision is `Denied`. The grace window applies per-grant; the
//! effective expiry is the earlier of the grant's `valid_until_ms` and the
//! artifact's `expires_at_ms`.

use std::collections::HashSet;
use std::sync::RwLock;

use async_trait::async_trait;
use domain::entitlement::{CapabilityDecision, CapabilityDecisionSource, READER_CORE};
use ports::entitlement::{CapabilityPolicy, EntitlementError};

use crate::artifact::{EntitlementArtifact, GrantKind};

/// Default grace window past a non-perpetual grant's expiry: 72 hours.
/// The architecture does not specify a number; this is the crate's chosen
/// default and is configurable via `OfflineCapabilityPolicy::with_grace_window`.
pub const DEFAULT_GRACE_WINDOW_MS: i64 = 72 * 60 * 60 * 1000;

/// The eight capability keys defined by §13.1, in declared order. Used by
/// `all_decisions` to enumerate the full decision set. `reader.core` is
/// always allowed; the other seven are gated by the active artifact.
pub const KNOWN_CAPABILITY_KEYS: &[&str] = &[
    domain::entitlement::READER_CORE,
    domain::entitlement::READER_UNLIMITED_PACKS,
    domain::entitlement::ANNOTATIONS_RICH_MEDIA,
    domain::entitlement::ASK_LOCAL_INFERENCE,
    domain::entitlement::MEMORY_LORE,
    domain::entitlement::BACKUP_ENCRYPTED,
    domain::entitlement::SIDECAR_SIGNED_EXPORT,
    domain::entitlement::MODELS_PREMIUM_CATALOG,
];

struct Inner {
    active_artifact: Option<EntitlementArtifact>,
    revoked_issuers: HashSet<String>,
    grace_window_ms: i64,
}

/// Offline capability policy. Thread-safe; the trait methods take `&self`
/// and interior mutability is used only for the writer-side configuration
/// methods (`set_active_artifact`, `clear_active_artifact`,
/// `set_revoked_issuers`, `add_revoked_issuer`).
pub struct OfflineCapabilityPolicy {
    inner: RwLock<Inner>,
}

impl OfflineCapabilityPolicy {
    pub fn new() -> Self {
        Self::with_grace_window(DEFAULT_GRACE_WINDOW_MS)
    }

    pub fn with_grace_window(grace_window_ms: i64) -> Self {
        Self {
            inner: RwLock::new(Inner {
                active_artifact: None,
                revoked_issuers: HashSet::new(),
                grace_window_ms,
            }),
        }
    }

    /// Install the active entitlement artifact. The caller is responsible
    /// for having verified it (`crate::verify_artifact`) and checked the
    /// revocation list; this method does not re-verify.
    pub fn set_active_artifact(&self, artifact: EntitlementArtifact) {
        if let Ok(mut g) = self.inner.write() {
            g.active_artifact = Some(artifact);
        }
    }

    /// Drop the active artifact. Future non-core decisions will be `Denied`
    /// until a new artifact is installed.
    pub fn clear_active_artifact(&self) {
        if let Ok(mut g) = self.inner.write() {
            g.active_artifact = None;
        }
    }

    /// Replace the issuer revocation set wholesale.
    pub fn set_revoked_issuers(&self, issuers: HashSet<String>) {
        if let Ok(mut g) = self.inner.write() {
            g.revoked_issuers = issuers;
        }
    }

    /// Add a single revoked issuer key id.
    pub fn add_revoked_issuer(&self, issuer: String) {
        if let Ok(mut g) = self.inner.write() {
            g.revoked_issuers.insert(issuer);
        }
    }

    /// Decide for a single capability. Pure function of (active_artifact,
    /// revoked_issuers, grace_window_ms, at_ms, capability_key). See the
    /// module docs for the decision table.
    async fn decide_inner(
        &self,
        capability_key: &str,
        at_ms: i64,
    ) -> Result<CapabilityDecision, EntitlementError> {
        // A-09: core reading is always available. This decision does not
        // consult the artifact, the clock, or the revocation list.
        if capability_key == READER_CORE {
            return Ok(CapabilityDecision {
                capability_key: capability_key.to_string(),
                allowed: true,
                source: CapabilityDecisionSource::Core,
                valid_until_ms: None,
                explanation: "Core reading is always available; commerce cannot revoke ownership (A-09)."
                    .to_string(),
            });
        }

        let g = self
            .inner
            .read()
            .map_err(|_| EntitlementError::Policy("policy lock poisoned".to_string()))?;

        let Some(artifact) = g.active_artifact.as_ref() else {
            return Ok(denied_no_artifact(capability_key));
        };

        if g.revoked_issuers.contains(&artifact.issuer_key_id) {
            return Ok(CapabilityDecision {
                capability_key: capability_key.to_string(),
                allowed: false,
                source: CapabilityDecisionSource::Denied,
                valid_until_ms: None,
                explanation: format!(
                    "Issuer '{}' has been revoked. Restore entitlement or contact support to retain your data.",
                    artifact.issuer_key_id
                ),
            });
        }

        let Some(grant) = artifact
            .grants
            .iter()
            .find(|gr| gr.capability_key == capability_key)
        else {
            return Ok(CapabilityDecision {
                capability_key: capability_key.to_string(),
                allowed: false,
                source: CapabilityDecisionSource::Denied,
                valid_until_ms: None,
                explanation: format!(
                    "Capability '{capability_key}' is not present in the active entitlement. Restore or purchase is available from settings; existing data is preserved."
                ),
            });
        };

        // A-09: a perpetual grant is forever, even past artifact expiry.
        if grant.grant_kind == GrantKind::Perpetual {
            return Ok(CapabilityDecision {
                capability_key: capability_key.to_string(),
                allowed: true,
                source: CapabilityDecisionSource::SignedArtifact,
                valid_until_ms: None,
                explanation: format!(
                    "Perpetual grant from artifact '{}'; commerce cannot revoke ownership (A-09).",
                    artifact.artifact_id
                ),
            });
        }

        // Term or quantity grant. Effective expiry is the earlier of the
        // grant's own expiry and the artifact's overall expiry.
        let effective_expiry = match (grant.valid_until_ms, artifact.expires_at_ms) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        };

        match effective_expiry {
            None => {
                // No expiry recorded on a non-perpetual grant. Defensive
                // allow with `SignedArtifact` so the user is not punished
                // for an issuer omission; the grant is "currently active".
                Ok(CapabilityDecision {
                    capability_key: capability_key.to_string(),
                    allowed: true,
                    source: CapabilityDecisionSource::SignedArtifact,
                    valid_until_ms: None,
                    explanation: format!(
                        "Grant from artifact '{}' is currently active (no expiry recorded).",
                        artifact.artifact_id
                    ),
                })
            }
            Some(expiry) if at_ms <= expiry => Ok(CapabilityDecision {
                capability_key: capability_key.to_string(),
                allowed: true,
                source: CapabilityDecisionSource::SignedArtifact,
                valid_until_ms: Some(expiry),
                explanation: format!(
                    "Grant from artifact '{}' is active until {}.",
                    artifact.artifact_id, expiry
                ),
            }),
            Some(expiry) => {
                let grace_until = expiry.saturating_add(g.grace_window_ms);
                if at_ms <= grace_until {
                    Ok(CapabilityDecision {
                        capability_key: capability_key.to_string(),
                        allowed: true,
                        source: CapabilityDecisionSource::GracePolicy,
                        valid_until_ms: Some(grace_until),
                        explanation: format!(
                            "Grant expired at {} but within offline grace window until {}. Existing data is preserved. Restore entitlement when online.",
                            expiry, grace_until
                        ),
                    })
                } else {
                    Ok(CapabilityDecision {
                        capability_key: capability_key.to_string(),
                        allowed: false,
                        source: CapabilityDecisionSource::Denied,
                        valid_until_ms: None,
                        explanation: format!(
                            "Grant expired at {} and the offline grace window ended at {}. Restore or purchase is available from settings; existing data is preserved.",
                            expiry, grace_until
                        ),
                    })
                }
            }
        }
    }
}

fn denied_no_artifact(capability_key: &str) -> CapabilityDecision {
    CapabilityDecision {
        capability_key: capability_key.to_string(),
        allowed: false,
        source: CapabilityDecisionSource::Denied,
        valid_until_ms: None,
        explanation: format!(
            "No active entitlement artifact is installed. Capability '{capability_key}' requires a verified entitlement. Restore or purchase is available from settings; existing data is preserved."
        ),
    }
}

impl Default for OfflineCapabilityPolicy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CapabilityPolicy for OfflineCapabilityPolicy {
    async fn decide(
        &self,
        capability_key: &str,
        at_ms: i64,
    ) -> Result<CapabilityDecision, EntitlementError> {
        self.decide_inner(capability_key, at_ms).await
    }

    async fn all_decisions(
        &self,
        at_ms: i64,
    ) -> Result<Vec<CapabilityDecision>, EntitlementError> {
        let mut out = Vec::with_capacity(KNOWN_CAPABILITY_KEYS.len());
        for k in KNOWN_CAPABILITY_KEYS {
            out.push(self.decide_inner(k, at_ms).await?);
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifact::{
        ArtifactSignature, EntitlementArtifact, Grant, GrantKind, ProviderReceipt, Subject,
        SubjectKind, ARTIFACT_SCHEMA,
    };

    fn artifact_with(grant_kind: GrantKind, valid_until_ms: Option<i64>, expires_at_ms: Option<i64>) -> EntitlementArtifact {
        EntitlementArtifact {
            schema: ARTIFACT_SCHEMA.to_string(),
            artifact_id: "art-test".to_string(),
            issuer_key_id: "issuer-7".to_string(),
            subject: Subject {
                kind: SubjectKind::Device,
                subject_id: "device-test".to_string(),
            },
            issued_at_ms: 1_000,
            not_before_ms: 1_000,
            expires_at_ms,
            grants: vec![Grant {
                capability_key: domain::entitlement::MEMORY_LORE.to_string(),
                grant_kind,
                quantity: None,
                valid_until_ms,
            }],
            provider_receipts: vec![ProviderReceipt {
                provider: "revenuecat".to_string(),
                event_digest: "deadbeef".to_string(),
            }],
            nonce: vec![0u8; crate::artifact::NONCE_LEN],
            signature: ArtifactSignature {
                algorithm: "alg".to_string(),
                key_id: "issuer-7".to_string(),
                signed_domain: crate::ENTITLEMENT_SIGNING_DOMAIN.to_string(),
                value: vec![0u8; 32],
            },
        }
    }

    #[tokio::test]
    async fn reader_core_is_always_allowed_without_artifact() {
        let p = OfflineCapabilityPolicy::new();
        let d = p.decide(domain::entitlement::READER_CORE, 0).await.unwrap();
        assert!(d.allowed);
        assert_eq!(d.source, CapabilityDecisionSource::Core);
        assert_eq!(d.valid_until_ms, None);
    }

    #[tokio::test]
    async fn paid_capability_denied_without_artifact() {
        let p = OfflineCapabilityPolicy::new();
        let d = p
            .decide(domain::entitlement::MEMORY_LORE, 1_000)
            .await
            .unwrap();
        assert!(!d.allowed);
        assert_eq!(d.source, CapabilityDecisionSource::Denied);
    }

    #[tokio::test]
    async fn perpetual_grant_allowed_forever_past_artifact_expiry() {
        // Artifact itself expired at 10_000 but grant is perpetual.
        let art = artifact_with(GrantKind::Perpetual, None, Some(10_000));
        let p = OfflineCapabilityPolicy::new();
        p.set_active_artifact(art);
        let d = p
            .decide(domain::entitlement::MEMORY_LORE, 1_000_000_000)
            .await
            .unwrap();
        assert!(d.allowed);
        assert_eq!(d.source, CapabilityDecisionSource::SignedArtifact);
        assert_eq!(d.valid_until_ms, None);
    }

    #[tokio::test]
    async fn term_grant_within_window_allowed_as_signed_artifact() {
        let art = artifact_with(GrantKind::Term, Some(10_000), Some(20_000));
        let p = OfflineCapabilityPolicy::new();
        p.set_active_artifact(art);
        let d = p
            .decide(domain::entitlement::MEMORY_LORE, 5_000)
            .await
            .unwrap();
        assert!(d.allowed);
        assert_eq!(d.source, CapabilityDecisionSource::SignedArtifact);
        // Effective expiry is the earlier of grant (10_000) and artifact (20_000).
        assert_eq!(d.valid_until_ms, Some(10_000));
    }

    #[tokio::test]
    async fn term_grant_recently_expired_uses_grace_policy() {
        let art = artifact_with(GrantKind::Term, Some(10_000), Some(20_000));
        let p = OfflineCapabilityPolicy::with_grace_window(2_000);
        p.set_active_artifact(art);
        // at_ms is 1 past grant expiry, within the 2_000 grace window.
        let d = p
            .decide(domain::entitlement::MEMORY_LORE, 10_001)
            .await
            .unwrap();
        assert!(d.allowed);
        assert_eq!(d.source, CapabilityDecisionSource::GracePolicy);
        assert_eq!(d.valid_until_ms, Some(12_000));
    }

    #[tokio::test]
    async fn term_grant_past_grace_is_denied() {
        let art = artifact_with(GrantKind::Term, Some(10_000), Some(20_000));
        let p = OfflineCapabilityPolicy::with_grace_window(2_000);
        p.set_active_artifact(art);
        let d = p
            .decide(domain::entitlement::MEMORY_LORE, 100_000)
            .await
            .unwrap();
        assert!(!d.allowed);
        assert_eq!(d.source, CapabilityDecisionSource::Denied);
    }

    #[tokio::test]
    async fn revoked_issuer_denies_paid_keeps_core() {
        let art = artifact_with(GrantKind::Perpetual, None, None);
        let p = OfflineCapabilityPolicy::new();
        p.set_active_artifact(art);
        p.add_revoked_issuer("issuer-7".to_string());
        let paid = p
            .decide(domain::entitlement::MEMORY_LORE, 1_000)
            .await
            .unwrap();
        assert!(!paid.allowed);
        assert_eq!(paid.source, CapabilityDecisionSource::Denied);
        // Core reading is unaffected by revocation (A-09).
        let core = p.decide(domain::entitlement::READER_CORE, 1_000).await.unwrap();
        assert!(core.allowed);
        assert_eq!(core.source, CapabilityDecisionSource::Core);
    }

    #[tokio::test]
    async fn grant_not_in_artifact_is_denied() {
        // Artifact only carries memory.lore.
        let art = artifact_with(GrantKind::Perpetual, None, None);
        let p = OfflineCapabilityPolicy::new();
        p.set_active_artifact(art);
        let d = p
            .decide(domain::entitlement::ASK_LOCAL_INFERENCE, 1_000)
            .await
            .unwrap();
        assert!(!d.allowed);
        assert_eq!(d.source, CapabilityDecisionSource::Denied);
    }

    #[tokio::test]
    async fn all_decisions_returns_one_per_known_key() {
        let p = OfflineCapabilityPolicy::new();
        let all = p.all_decisions(1_000).await.unwrap();
        assert_eq!(all.len(), KNOWN_CAPABILITY_KEYS.len());
        // reader.core must be present and allowed; the others denied without
        // an artifact.
        let mut seen_core = false;
        for d in &all {
            if d.capability_key == domain::entitlement::READER_CORE {
                seen_core = true;
                assert!(d.allowed);
                assert_eq!(d.source, CapabilityDecisionSource::Core);
            } else {
                assert!(!d.allowed);
            }
        }
        assert!(seen_core);
    }

    #[tokio::test]
    async fn clear_active_artifact_returns_to_denied() {
        let art = artifact_with(GrantKind::Perpetual, None, None);
        let p = OfflineCapabilityPolicy::new();
        p.set_active_artifact(art);
        assert!(p
            .decide(domain::entitlement::MEMORY_LORE, 1_000)
            .await
            .unwrap()
            .allowed);
        p.clear_active_artifact();
        let d = p
            .decide(domain::entitlement::MEMORY_LORE, 1_000)
            .await
            .unwrap();
        assert!(!d.allowed);
    }
}
