//! Readiness projection (§14.2 `SystemReadiness`, M1 baseline).
//!
//! M1 scope note: §14.2 `SystemReadiness` is a per-subsystem record
//! (`reader`/`storage`/`inference`/`backup`/`entitlement`, each a
//! `ReadinessState`). The Milestone-1 contract in the dispatch spec asks for a
//! flat `overall`/`summary`/`flags`/`checkedAt` shape so the frontend can show
//! a single readiness indicator before per-subsystem probes exist. This module
//! therefore emits the M1 shape; the per-subsystem decomposition is deferred
//! to a later milestone (flagged for the architect).

use serde::Serialize;

/// Flat M1 readiness view.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemReadiness {
    /// `"ready"` | `"warning"` | `"blocked"`.
    pub overall: String,
    pub summary: String,
    pub flags: Vec<ReadinessFlag>,
    pub checked_at_ms: i64,
}

/// One contributing signal to [`SystemReadiness`].
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadinessFlag {
    /// Stable machine key, e.g. `"schema"`.
    pub key: String,
    /// `"ready"` | `"warning"` | `"blocked"`.
    pub state: String,
    pub detail: String,
}

/// Build the M1 baseline readiness.
///
/// `schema_applied` is the sole M1 signal: when the §6.2 initial migration is
/// recorded, overall readiness is `"ready"`; otherwise it is `"warning"`.
pub fn build(schema_applied: bool, checked_at_ms: i64) -> SystemReadiness {
    let state = if schema_applied { "ready" } else { "warning" };
    let detail = if schema_applied {
        "initial schema applied"
    } else {
        "initial schema not yet applied"
    };
    SystemReadiness {
        overall: state.to_string(),
        summary: if schema_applied {
            "M1 baseline met".to_string()
        } else {
            "schema pending".to_string()
        },
        flags: vec![ReadinessFlag {
            key: "schema".to_string(),
            state: state.to_string(),
            detail: detail.to_string(),
        }],
        checked_at_ms,
    }
}
