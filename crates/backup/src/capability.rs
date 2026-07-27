//! §13.5 capability gating for the backup pipeline.
//!
//! Two capabilities can gate backup behaviour:
//!
//! - `backup.encrypted` (§13.1) — paid. **The encryption capability never
//!   blocks the export of user-created data.** If the caller asks for an
//!   encrypted backup and the capability is denied, the pipeline returns
//!   [`CapabilityDecision::denied`]; the caller can retry with
//!   `encrypted: false`, which always works. This is the §13.5 rule.
//!
//! - the core read path (`reader.core`) — never paid, never gates backup.
//!
//! The implementation here is the *decision*; the caller (Tauri command
//! handler) is responsible for surfacing `CapabilityDenied` to the UI as
//! the §14.1 `capability_denied` `AppErrorCode` with the
//! `restore_purchase` action.

use serde::{Deserialize, Serialize};

/// The §13.1 capability keys that the backup pipeline consults.
pub const CAPABILITY_BACKUP_ENCRYPTED: &str = "backup.encrypted";

/// Local copy of `domain::entitlement::CapabilityDecision`. We duplicate
/// only the minimum — `capability_key`, `allowed`, and a stable enum source
/// — because the `domain::entitlement` types depend on the (broken) ports
/// crate transitively in some downstream builds, and §13.5 says the UI
/// receives a `CapabilityDecision`, never raw RevenueCat state.
///
/// // I-10(b): swap for `domain::entitlement::CapabilityDecision` when the
/// ports crate compiles end-to-end. Field names match §13.2 verbatim.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CapabilityDecision {
    pub capability_key: String,
    pub allowed: bool,
    pub source: CapabilityDecisionSource,
    pub valid_until_ms: Option<i64>,
    pub explanation: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityDecisionSource {
    Core,
    SignedArtifact,
    Gracepolicy,
    Denied,
}

impl CapabilityDecision {
    /// Build the canonical "core, always-allowed" decision for a capability.
    pub fn core_allowed(key: &'static str) -> Self {
        Self {
            capability_key: key.to_string(),
            allowed: true,
            source: CapabilityDecisionSource::Core,
            valid_until_ms: None,
            explanation: format!("core entitlement: {key}"),
        }
    }
    /// Build the canonical "denied" decision.
    pub fn denied(key: &'static str, explanation: impl Into<String>) -> Self {
        Self {
            capability_key: key.to_string(),
            allowed: false,
            source: CapabilityDecisionSource::Denied,
            valid_until_ms: None,
            explanation: explanation.into(),
        }
    }

    /// Resolve the effective capability for a create request.
    ///
    /// Per §13.5:
    /// - `encrypted == false` is **always** allowed — the export of
    ///   user-created data is never blocked.
    /// - `encrypted == true` requires `backup.encrypted`. If denied, the
    ///   caller can fall back to plaintext (caller's choice; we surface the
    ///   denial).
    pub fn resolve_for_create(
        encrypted: bool,
        backup_encrypted_decision: &CapabilityDecision,
    ) -> Self {
        if !encrypted {
            return Self::core_allowed(CAPABILITY_BACKUP_ENCRYPTED);
        }
        if backup_encrypted_decision.allowed {
            backup_encrypted_decision.clone()
        } else {
            backup_encrypted_decision.clone()
        }
    }
}

/// Returns `Ok(())` if the request is allowed under the supplied capability
/// decision, else `Err(CapabilityDenied)`. Use this from the create path
/// before doing any work.
pub fn require_encryption_capability(
    encrypted: bool,
    decision: &CapabilityDecision,
) -> Result<(), crate::error::BackupError> {
    if encrypted && !decision.allowed {
        return Err(crate::error::BackupError::CapabilityDenied {
            capability: CAPABILITY_BACKUP_ENCRYPTED,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plaintext_backup_is_always_allowed_even_when_encryption_denied() {
        let denied = CapabilityDecision::denied(
            CAPABILITY_BACKUP_ENCRYPTED,
            "no revenuecat entitlement for backup.encrypted",
        );
        require_encryption_capability(false, &denied).expect(
            "plaintext backup is not gated by backup.encrypted — §13.5",
        );
    }

    #[test]
    fn encrypted_backup_is_blocked_when_capability_denied() {
        let denied = CapabilityDecision::denied(
            CAPABILITY_BACKUP_ENCRYPTED,
            "no revenuecat entitlement for backup.encrypted",
        );
        let err = require_encryption_capability(true, &denied).unwrap_err();
        match err {
            crate::error::BackupError::CapabilityDenied { capability } => {
                assert_eq!(capability, CAPABILITY_BACKUP_ENCRYPTED);
            }
            other => panic!("expected CapabilityDenied, got {other:?}"),
        }
    }

    #[test]
    fn encrypted_backup_proceeds_when_capability_allowed() {
        let allowed = CapabilityDecision::core_allowed(CAPABILITY_BACKUP_ENCRYPTED);
        require_encryption_capability(true, &allowed).expect("allowed capability must proceed");
    }
}
