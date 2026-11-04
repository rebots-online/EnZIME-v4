//! Google Analytics 4, opt-in, via the Measurement Protocol.
//!
//! Why server-side MP instead of gtag.js in the webview: no third-party
//! script runs next to the article sandbox (§7.3), no cookies on a custom
//! scheme, and every payload passes through one allow-list here.
//!
//! Privacy contract (PRD §8.3 "No content telemetry"):
//! - nothing is sent until the user explicitly opts in; default is off;
//! - only allow-listed event names and parameter keys;
//! - parameter values must be short machine tokens (`[a-z0-9_:/.-]`), so
//!   article titles, search text, note bodies and file paths cannot pass;
//! - page paths are route *templates* (`/reader/:packId/:articleKey`);
//! - the client id is a random per-install ULID, never derived from the
//!   license key, device id or any account.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::settings;
use crate::state::AppState;

const CONSENT_KEY: &str = "analytics.consent";
const CLIENT_ID_KEY: &str = "analytics.client_id";

/// (event name, allowed parameter keys)
const EVENTS: &[(&str, &[&str])] = &[
    ("page_view", &["page_path"]),
    ("app_open", &[]),
    ("view_plans", &[]),
    ("select_plan", &["plan_id"]),
    ("begin_checkout", &["plan_id"]),
    ("purchase_refresh", &["result"]),
    ("restore_purchase", &["result"]),
    ("open_billing_portal", &[]),
    ("capability_gate", &["capability"]),
    ("pack_install_start", &[]),
    ("annotation_create", &["kind"]),
    ("title_search", &[]),
];

#[derive(Clone)]
pub struct AnalyticsConfig {
    pub measurement_id: Option<String>,
    pub api_secret: Option<String>,
    pub debug: bool,
}

impl AnalyticsConfig {
    pub fn from_env() -> Self {
        fn pick(runtime: &str, build: Option<&'static str>) -> Option<String> {
            std::env::var(runtime)
                .ok()
                .or_else(|| build.map(str::to_string))
                .filter(|s| !s.trim().is_empty())
        }
        let measurement_id = pick("ENZIME_GA_MEASUREMENT_ID", option_env!("ENZIME_GA_MEASUREMENT_ID"))
            .filter(|m| m.starts_with("G-") && m.len() <= 20 && m[2..].bytes().all(|b| b.is_ascii_alphanumeric()));
        Self {
            measurement_id,
            api_secret: pick("ENZIME_GA_API_SECRET", option_env!("ENZIME_GA_API_SECRET")),
            debug: std::env::var("ENZIME_GA_DEBUG").map(|v| v == "1").unwrap_or(false),
        }
    }

    pub fn configured(&self) -> bool {
        self.measurement_id.is_some() && self.api_secret.is_some()
    }
}

pub struct Analytics {
    pub config: AnalyticsConfig,
    http: reqwest::Client,
    session_id: String,
}

impl Analytics {
    pub fn new(config: AnalyticsConfig) -> Result<Self, String> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| e.to_string())?;
        let session_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_else(|_| "0".into());
        Ok(Self { config, http, session_id })
    }
}

fn token_ok(v: &str) -> bool {
    !v.is_empty()
        && v.len() <= 64
        && v.bytes().all(|b| {
            b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'_' | b':' | b'/' | b'.' | b'-')
        })
}

/// Validate and reduce an event to its allow-listed form. `None` = drop.
pub fn sanitize(name: &str, params: &BTreeMap<String, String>) -> Option<BTreeMap<String, String>> {
    let (_, allowed) = EVENTS.iter().find(|(n, _)| *n == name)?;
    let mut out = BTreeMap::new();
    for (k, v) in params {
        if !allowed.contains(&k.as_str()) {
            return None; // unknown key: drop the whole event rather than guess
        }
        if !token_ok(v) {
            return None;
        }
        if k == "page_path" && !v.starts_with('/') {
            return None;
        }
        out.insert(k.clone(), v.clone());
    }
    Some(out)
}

fn consent(conn: &Connection) -> rusqlite::Result<Option<bool>> {
    Ok(settings::get_string(conn, CONSENT_KEY)?.map(|v| v == "granted"))
}

fn client_id(conn: &Connection) -> rusqlite::Result<String> {
    if let Some(id) = settings::get_string(conn, CLIENT_ID_KEY)? {
        return Ok(id);
    }
    let id = ulid::Ulid::new().to_string();
    settings::set_string(conn, CLIENT_ID_KEY, &id)?;
    Ok(id)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsStateDto {
    pub configured: bool,
    /// `"granted" | "denied" | "unset"`
    pub consent: &'static str,
}

async fn read_state(state: &AppState) -> Result<AnalyticsStateDto, String> {
    let conn = state.conn.clone();
    let c = tokio::task::spawn_blocking(move || {
        let g = conn.lock().map_err(|e| e.to_string())?;
        consent(&g).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??;
    Ok(AnalyticsStateDto {
        configured: state.analytics.config.configured(),
        consent: match c {
            Some(true) => "granted",
            Some(false) => "denied",
            None => "unset",
        },
    })
}

#[tauri::command]
pub async fn analytics_state(state: State<'_, AppState>) -> Result<AnalyticsStateDto, String> {
    read_state(&state).await
}

#[tauri::command]
pub async fn analytics_set_consent(
    state: State<'_, AppState>,
    granted: bool,
) -> Result<AnalyticsStateDto, String> {
    let conn = state.conn.clone();
    tokio::task::spawn_blocking(move || {
        let g = conn.lock().map_err(|e| e.to_string())?;
        settings::set_string(&g, CONSENT_KEY, if granted { "granted" } else { "denied" })
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??;
    read_state(&state).await
}

#[derive(Deserialize)]
pub struct TrackRequest {
    pub name: String,
    #[serde(default)]
    pub params: BTreeMap<String, String>,
}

/// Fire-and-forget. Always `Ok(())`: analytics must never surface an error
/// or block the UI. Returns before the network call completes.
#[tauri::command]
pub async fn analytics_track(state: State<'_, AppState>, event: TrackRequest) -> Result<(), String> {
    let analytics = state.analytics.clone();
    if !analytics.config.configured() {
        return Ok(());
    }
    let Some(params) = sanitize(&event.name, &event.params) else {
        return Ok(());
    };
    let conn = state.conn.clone();
    let cid = tokio::task::spawn_blocking(move || {
        let g = conn.lock().ok()?;
        match consent(&g).ok()? {
            Some(true) => client_id(&g).ok(),
            _ => None,
        }
    })
    .await
    .ok()
    .flatten();
    let Some(cid) = cid else { return Ok(()) };
    tauri::async_runtime::spawn(send(analytics, cid, event.name, params));
    Ok(())
}

async fn send(a: Arc<Analytics>, client_id: String, name: String, params: BTreeMap<String, String>) {
    let (Some(mid), Some(secret)) = (&a.config.measurement_id, &a.config.api_secret) else {
        return;
    };
    let mut p = serde_json::Map::new();
    for (k, v) in params {
        p.insert(k, serde_json::Value::String(v));
    }
    p.insert("session_id".into(), a.session_id.clone().into());
    p.insert("engagement_time_msec".into(), 100.into());
    p.insert("app_version".into(), env!("CARGO_PKG_VERSION").into());
    let body = serde_json::json!({
        "client_id": client_id,
        "non_personalized_ads": true,
        "events": [{ "name": name, "params": p }],
    });
    let path = if a.config.debug { "debug/mp/collect" } else { "mp/collect" };
    let url = format!(
        "https://www.google-analytics.com/{path}?measurement_id={mid}&api_secret={}",
        percent_encoding::utf8_percent_encode(secret, percent_encoding::NON_ALPHANUMERIC)
    );
    let Ok(bytes) = serde_json::to_vec(&body) else { return };
    let res = a
        .http
        .post(url)
        .header("Content-Type", "application/json")
        .body(bytes)
        .send()
        .await;
    if a.config.debug {
        if let Ok(r) = res {
            eprintln!("[analytics] {} → {}", name, r.text().await.unwrap_or_default());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    #[test]
    fn allows_listed_events_with_token_values() {
        assert!(sanitize("app_open", &p(&[])).is_some());
        assert!(sanitize("page_view", &p(&[("page_path", "/reader/:packid/:articlekey")])).is_some());
        assert!(sanitize("select_plan", &p(&[("plan_id", "pro_yearly")])).is_some());
    }

    #[test]
    fn blocks_content_and_unknowns() {
        // Unknown event.
        assert!(sanitize("article_read", &p(&[])).is_none());
        // Unknown param key.
        assert!(sanitize("title_search", &p(&[("query", "foo")])).is_none());
        // Free text (spaces, capitals) cannot pass as a value.
        assert!(sanitize("select_plan", &p(&[("plan_id", "Albert Einstein")])).is_none());
        // Page path must be a path.
        assert!(sanitize("page_view", &p(&[("page_path", "reader")])).is_none());
        // Overlong value.
        let long = "a".repeat(65);
        assert!(sanitize("select_plan", &p(&[("plan_id", &long)])).is_none());
    }

    #[test]
    fn measurement_id_shape_is_checked() {
        std::env::set_var("ENZIME_GA_MEASUREMENT_ID", "not-an-id");
        std::env::set_var("ENZIME_GA_API_SECRET", "s");
        assert!(!AnalyticsConfig::from_env().configured());
        std::env::set_var("ENZIME_GA_MEASUREMENT_ID", "G-ABC123XYZ");
        assert!(AnalyticsConfig::from_env().configured());
        std::env::remove_var("ENZIME_GA_MEASUREMENT_ID");
        std::env::remove_var("ENZIME_GA_API_SECRET");
    }
}
