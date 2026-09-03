//! App bootstrap use case (§14.4 `app_bootstrap` → `AppBootstrap`).
//!
//! Owns the §14.5 wire DTO and its assembly from primitive inputs. The IPC
//! layer supplies `platform`, `app_version`, `first_launch`, and a built
//! [`SystemReadiness`]; this module composes the §16.7 route list and the
//! (M1-empty) capabilities array.

use serde::Serialize;

use crate::readiness::SystemReadiness;

/// §14.5 `AppBootstrap`.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppBootstrap {
    /// Always `1` for the §6 schema family.
    pub schema_version: i32,
    pub app_version: String,
    /// `"android"` | `"linux"` | `"windows"`.
    pub platform: String,
    pub first_launch: bool,
    pub routes: Vec<RouteDescriptor>,
    /// M1 ships with no entitlement decisions surfaced here.
    pub capabilities: Vec<serde_json::Value>,
    pub readiness: SystemReadiness,
}

/// §14.5 `RouteDescriptor`.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteDescriptor {
    pub path: String,
    pub enabled: bool,
}

/// §16.7 route identifiers paired with their M1 enabled flag. The list matches
/// the `RoutePath` union in §14.5 verbatim (the `/ask/:conversationId` variant
/// is a parameterization of `/ask`, not a separate §14.5 union member).
const M1_ROUTES: &[(&str, bool)] = &[
    ("/", true),
    ("/library", true),
    ("/reader/:packId/:articleKey", true),
    ("/ask", true),
    ("/notes", true),
    ("/memory", true),
    ("/settings", true),
    ("/setup", true),
];

/// Compose the bootstrap DTO from primitive inputs.
pub fn build(
    platform: &str,
    app_version: &str,
    first_launch: bool,
    readiness: SystemReadiness,
) -> AppBootstrap {
    AppBootstrap {
        schema_version: 1,
        app_version: app_version.to_string(),
        platform: platform.to_string(),
        first_launch,
        routes: M1_ROUTES
            .iter()
            .map(|(path, enabled)| RouteDescriptor {
                path: (*path).to_string(),
                enabled: *enabled,
            })
            .collect(),
        // §14.5 types this as `CapabilityDecision[]`; M1 ships an empty array
        // and uses `serde_json::Value` so the application crate need not pull
        // the full `domain::entitlement` shape into the bootstrap path.
        capabilities: Vec::new(),
        readiness,
    }
}
