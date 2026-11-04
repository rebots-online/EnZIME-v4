//! Billing + offline entitlement (ARCHITECTURE.md §13).
//!
//! ```text
//!  UI ──IPC──▶ billing.rs ──HTTPS──▶ EnZIME billing function (Netlify) ──▶ Chargebee
//!                 │   ▲                       │ signs enzime.entitlement.cbor.v1
//!                 │   └──── artifact (CBOR) ◀─┘
//!                 ▼
//!   verify (embedded Ed25519 public key) → SQLite entitlement_artifacts
//!                 ▼
//!   OfflineCapabilityPolicy → CapabilityDecision[] → UI
//! ```
//!
//! The app never talks to a commerce provider and holds no provider secret.
//! Decisions are derived offline from the last verified artifact, so paid
//! capabilities survive restarts without a network (PRD §8, A-09). Swapping
//! Chargebee for WooCommerce is a server-side change only.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use base64::Engine as _;
use ed25519_dalek::VerifyingKey;
use entitlement::{decode_artifact, verify_artifact, EntitlementArtifact, OfflineCapabilityPolicy};
use ports::entitlement::{CapabilityPolicy, EntitlementError};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sidecar::{Ed25519Keystore, Ed25519Provider};
use tauri::State;

use crate::settings;
use crate::state::AppState;

const DEVICE_ID_KEY: &str = "device.id";
const LICENSE_KEY: &str = "billing.license_id";
const HTTP_TIMEOUT: Duration = Duration::from_secs(20);

// ===========================================================================
// Configuration (build-time defaults, runtime env override)
// ===========================================================================

fn config_value(runtime: &str, build: Option<&'static str>) -> Option<String> {
    std::env::var(runtime)
        .ok()
        .or_else(|| build.map(str::to_string))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

#[derive(Clone)]
pub struct BillingConfig {
    pub base_url: Option<String>,
    pub trusted_key: Option<(String, VerifyingKey)>,
}

impl BillingConfig {
    pub fn from_env() -> Self {
        let base_url = config_value("ENZIME_BILLING_URL", option_env!("ENZIME_BILLING_URL"))
            .map(|u| u.trim_end_matches('/').to_string())
            .filter(|u| u.starts_with("https://") || u.starts_with("http://127.0.0.1"));
        let key_id = config_value(
            "ENZIME_ENTITLEMENT_KEY_ID",
            option_env!("ENZIME_ENTITLEMENT_KEY_ID"),
        );
        let pubkey = config_value(
            "ENZIME_ENTITLEMENT_PUBKEY",
            option_env!("ENZIME_ENTITLEMENT_PUBKEY"),
        );
        let trusted_key = match (key_id, pubkey) {
            (Some(id), Some(hex_key)) => hex::decode(hex_key)
                .ok()
                .and_then(|b| <[u8; 32]>::try_from(b).ok())
                .and_then(|b| VerifyingKey::from_bytes(&b).ok())
                .map(|vk| (id, vk)),
            _ => None,
        };
        Self { base_url, trusted_key }
    }

    /// Purchasing needs both a server and a trusted key; verifying stored
    /// artifacts needs only the key.
    pub fn purchasing_enabled(&self) -> bool {
        self.base_url.is_some() && self.trusted_key.is_some()
    }
}

// ===========================================================================
// Service
// ===========================================================================

pub struct Billing {
    pub config: BillingConfig,
    pub policy: Arc<OfflineCapabilityPolicy>,
    verifier: Ed25519Provider,
    http: reqwest::Client,
}

#[derive(Debug, thiserror::Error)]
pub enum BillingFailure {
    #[error("Purchasing is not configured in this build")]
    NotConfigured,
    #[error("Can't reach the billing service. Check your connection and try again.")]
    Offline,
    #[error("The billing service is unavailable right now. Try again shortly.")]
    Unavailable,
    #[error("No active purchase was found for this license key")]
    NoPurchases,
    #[error("{0}")]
    Invalid(String),
    #[error("The entitlement from the billing service failed verification")]
    Signature,
    #[error("internal error: {0}")]
    Internal(String),
}

impl Billing {
    pub fn new(config: BillingConfig) -> Result<Self, String> {
        let mut ks = Ed25519Keystore::default();
        if let Some((id, vk)) = &config.trusted_key {
            ks.add_verifying(id.clone(), *vk, Some("EnZIME entitlement issuer".into()));
        }
        let http = reqwest::Client::builder()
            .timeout(HTTP_TIMEOUT)
            .user_agent(concat!("EnZIME-Reader/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|e| e.to_string())?;
        Ok(Self {
            config,
            policy: Arc::new(OfflineCapabilityPolicy::new()),
            verifier: Ed25519Provider::new(ks),
            http,
        })
    }

    async fn post<T: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        body: serde_json::Value,
    ) -> Result<T, BillingFailure> {
        let base = self.config.base_url.as_ref().ok_or(BillingFailure::NotConfigured)?;
        let res = self
            .http
            .post(format!("{base}{path}"))
            .header("Content-Type", "application/json")
            .body(serde_json::to_vec(&body).map_err(|e| BillingFailure::Internal(e.to_string()))?)
            .send()
            .await
            .map_err(|_| BillingFailure::Offline)?;
        Self::read(res).await
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T, BillingFailure> {
        let base = self.config.base_url.as_ref().ok_or(BillingFailure::NotConfigured)?;
        let res = self
            .http
            .get(format!("{base}{path}"))
            .send()
            .await
            .map_err(|_| BillingFailure::Offline)?;
        Self::read(res).await
    }

    async fn read<T: for<'de> Deserialize<'de>>(res: reqwest::Response) -> Result<T, BillingFailure> {
        let status = res.status().as_u16();
        let text = res.text().await.map_err(|_| BillingFailure::Offline)?;
        if (200..300).contains(&status) {
            return serde_json::from_str(&text).map_err(|e| BillingFailure::Internal(e.to_string()));
        }
        let code = serde_json::from_str::<ServerError>(&text)
            .map(|e| e.error.code)
            .unwrap_or_default();
        Err(match (status, code.as_str()) {
            (404, "no_purchases") | (404, "not_found") => BillingFailure::NoPurchases,
            (400, _) => BillingFailure::Invalid("The billing service rejected the request".into()),
            _ => BillingFailure::Unavailable,
        })
    }

    /// Verify bytes against the trusted key and the device binding.
    async fn verify(&self, bytes: &[u8], device_id: &str) -> Result<EntitlementArtifact, BillingFailure> {
        let artifact = decode_artifact(bytes).map_err(|_| BillingFailure::Signature)?;
        verify_artifact(&artifact, &self.verifier, &HashSet::new())
            .await
            .map_err(|_| BillingFailure::Signature)?;
        if artifact.subject.subject_id != device_id {
            return Err(BillingFailure::Signature);
        }
        Ok(artifact)
    }
}

#[derive(Deserialize, Default)]
struct ServerErrorBody {
    #[serde(default)]
    code: String,
}
#[derive(Deserialize)]
struct ServerError {
    error: ServerErrorBody,
}

// ===========================================================================
// Persistence
// ===========================================================================

pub fn device_id(conn: &Connection) -> rusqlite::Result<String> {
    if let Some(id) = settings::get_string(conn, DEVICE_ID_KEY)? {
        return Ok(id);
    }
    let id = ulid::Ulid::new().to_string();
    settings::set_string(conn, DEVICE_ID_KEY, &id)?;
    Ok(id)
}

fn store_artifact(
    conn: &Connection,
    artifact: &EntitlementArtifact,
    bytes: &[u8],
    public_key: &VerifyingKey,
    now_ms: i64,
) -> rusqlite::Result<()> {
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO trust_keys (trust_key_id, key_kind, label, ed25519_public_key, trust_state,
                                 valid_from_ms, valid_until_ms, created_at_ms)
         VALUES (?1, 'entitlement', 'EnZIME entitlement issuer', ?2, 'trusted', 0, NULL, ?3)
         ON CONFLICT(trust_key_id) DO NOTHING",
        params![artifact.issuer_key_id, public_key.as_bytes().to_vec(), now_ms],
    )?;
    let sha = hex::encode(Sha256::digest(bytes));
    let inserted = tx.execute(
        "INSERT INTO entitlement_artifacts (entitlement_artifact_id, subject_device_id, artifact_cbor,
             artifact_sha256, issuer_key_id, issued_at_ms, not_before_ms, expires_at_ms,
             verification_state, imported_at_ms)
         VALUES (?1, NULL, ?2, ?3, ?4, ?5, ?6, ?7, 'verified', ?8)
         ON CONFLICT DO NOTHING",
        params![
            artifact.artifact_id,
            bytes,
            sha,
            artifact.issuer_key_id,
            artifact.issued_at_ms,
            artifact.not_before_ms,
            artifact.expires_at_ms,
            now_ms
        ],
    )?;
    if inserted == 1 {
        for g in &artifact.grants {
            let kind = match g.grant_kind {
                entitlement::GrantKind::Perpetual => "perpetual",
                entitlement::GrantKind::Term => "term",
                entitlement::GrantKind::Quantity => "quantity",
            };
            tx.execute(
                "INSERT INTO entitlement_grants (entitlement_artifact_id, capability_key, grant_kind,
                                                 quantity, valid_until_ms)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    artifact.artifact_id,
                    g.capability_key,
                    kind,
                    g.quantity.map(|q| q as i64),
                    g.valid_until_ms
                ],
            )?;
        }
    }
    tx.commit()
}

/// Newest stored artifact candidates, most recent first.
fn stored_artifacts(conn: &Connection) -> rusqlite::Result<Vec<Vec<u8>>> {
    let mut stmt = conn.prepare(
        "SELECT artifact_cbor FROM entitlement_artifacts
         WHERE verification_state = 'verified'
         ORDER BY issued_at_ms DESC LIMIT 5",
    )?;
    let rows = stmt.query_map([], |r| r.get::<_, Vec<u8>>(0))?;
    rows.collect()
}

/// Boot path: re-verify the newest stored artifact (never trust the DB flag
/// alone) and install it in the offline policy.
pub async fn load_active(state: &AppState) -> Result<(), String> {
    let conn = state.conn.clone();
    let (device, blobs) = tokio::task::spawn_blocking(move || {
        let g = conn.lock().map_err(|e| e.to_string())?;
        Ok::<_, String>((
            device_id(&g).map_err(|e| e.to_string())?,
            stored_artifacts(&g).map_err(|e| e.to_string())?,
        ))
    })
    .await
    .map_err(|e| e.to_string())??;
    for bytes in blobs {
        if let Ok(artifact) = state.billing.verify(&bytes, &device).await {
            state.billing.policy.set_active_artifact(artifact);
            break;
        }
    }
    Ok(())
}

// ===========================================================================
// DTOs (§14 camelCase wire shape)
// ===========================================================================

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityDecisionDto {
    pub capability_key: String,
    pub allowed: bool,
    pub source: domain::entitlement::CapabilityDecisionSource,
    pub valid_until_ms: Option<i64>,
    pub explanation: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BillingStatusDto {
    pub purchasing_enabled: bool,
    pub license_id: Option<String>,
    pub decisions: Vec<CapabilityDecisionDto>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicPlanDto {
    pub id: String,
    pub name: String,
    pub price_label: String,
    pub kind: String,
    pub capabilities: Vec<String>,
    pub highlight: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckoutDto {
    pub url: String,
    pub license_id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BillingError {
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct BillingResult<T> {
    pub ok: bool,
    pub value: Option<T>,
    pub error: Option<BillingError>,
}

impl<T> From<Result<T, BillingFailure>> for BillingResult<T> {
    fn from(r: Result<T, BillingFailure>) -> Self {
        match r {
            Ok(v) => Self { ok: true, value: Some(v), error: None },
            Err(e) => {
                let code = match e {
                    BillingFailure::NotConfigured => "unsupported",
                    BillingFailure::Offline => "offline",
                    BillingFailure::Unavailable => "backend_unavailable",
                    BillingFailure::NoPurchases => "not_found",
                    BillingFailure::Invalid(_) => "invalid_request",
                    BillingFailure::Signature => "signature_invalid",
                    BillingFailure::Internal(_) => "internal",
                };
                Self {
                    ok: false,
                    value: None,
                    error: Some(BillingError { code, message: e.to_string() }),
                }
            }
        }
    }
}

// ===========================================================================
// Operations
// ===========================================================================

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn internal<E: std::fmt::Display>(e: E) -> BillingFailure {
    BillingFailure::Internal(e.to_string())
}

async fn with_conn<T: Send + 'static>(
    state: &AppState,
    f: impl FnOnce(&Connection) -> rusqlite::Result<T> + Send + 'static,
) -> Result<T, BillingFailure> {
    let conn = state.conn.clone();
    tokio::task::spawn_blocking(move || {
        let g = conn.lock().map_err(internal)?;
        f(&g).map_err(internal)
    })
    .await
    .map_err(internal)?
}

async fn decisions(state: &AppState) -> Result<Vec<CapabilityDecisionDto>, BillingFailure> {
    let all = state
        .billing
        .policy
        .all_decisions(now_ms())
        .await
        .map_err(|e: EntitlementError| internal(e))?;
    Ok(all
        .into_iter()
        .map(|d| CapabilityDecisionDto {
            capability_key: d.capability_key,
            allowed: d.allowed,
            source: d.source,
            valid_until_ms: d.valid_until_ms,
            explanation: d.explanation,
        })
        .collect())
}

async fn status(state: &AppState) -> Result<BillingStatusDto, BillingFailure> {
    let license_id = with_conn(state, |c| settings::get_string(c, LICENSE_KEY)).await?;
    Ok(BillingStatusDto {
        purchasing_enabled: state.billing.config.purchasing_enabled(),
        license_id,
        decisions: decisions(state).await?,
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct EntitlementResponse {
    artifact: String,
}

/// Fetch, verify, persist and activate the entitlement for `license_id`.
async fn refresh_with(state: &AppState, license_id: &str) -> Result<(), BillingFailure> {
    let (_, public_key) = state
        .billing
        .config
        .trusted_key
        .clone()
        .ok_or(BillingFailure::NotConfigured)?;
    let device = with_conn(state, device_id).await?;
    let resp: EntitlementResponse = state
        .billing
        .post(
            "/v1/entitlement",
            serde_json::json!({ "deviceId": device, "licenseId": license_id }),
        )
        .await?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(resp.artifact)
        .map_err(|_| BillingFailure::Signature)?;
    let artifact = state.billing.verify(&bytes, &device).await?;
    let to_store = artifact.clone();
    with_conn(state, move |c| store_artifact(c, &to_store, &bytes, &public_key, now_ms())).await?;
    state.billing.policy.set_active_artifact(artifact);
    Ok(())
}

pub fn valid_license_id(s: &str) -> bool {
    s.len() == 30
        && s.starts_with("enz_")
        && s[4..].bytes().all(|b| b.is_ascii_digit() || b.is_ascii_lowercase())
}

/// Background refresh at startup; failures are silent (offline is normal).
pub async fn refresh_in_background(state: &AppState) {
    if !state.billing.config.purchasing_enabled() {
        return;
    }
    if let Ok(Some(license)) = with_conn(state, |c| settings::get_string(c, LICENSE_KEY)).await {
        let _ = refresh_with(state, &license).await;
    }
}

// ===========================================================================
// IPC commands
// ===========================================================================

#[tauri::command]
pub async fn billing_status(state: State<'_, AppState>) -> Result<BillingResult<BillingStatusDto>, String> {
    Ok(status(&state).await.into())
}

#[tauri::command]
pub async fn entitlement_decisions(
    state: State<'_, AppState>,
) -> Result<BillingResult<Vec<CapabilityDecisionDto>>, String> {
    Ok(decisions(&state).await.into())
}

#[derive(Deserialize)]
struct PlansResponse {
    plans: Vec<PublicPlanDto>,
}

#[tauri::command]
pub async fn billing_plans(state: State<'_, AppState>) -> Result<BillingResult<Vec<PublicPlanDto>>, String> {
    let r: Result<PlansResponse, _> = state.billing.get("/v1/plans").await;
    Ok(r.map(|p| p.plans).into())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CheckoutResponse {
    license_id: String,
    url: String,
}

/// Start a hosted checkout. The license id is minted server-side on first
/// purchase and stored locally before the browser opens, so a purchase that
/// completes while the app is closed is still recoverable on next launch.
#[tauri::command]
pub async fn billing_checkout(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    plan_id: String,
) -> Result<BillingResult<CheckoutDto>, String> {
    let run = async {
        if !state.billing.config.purchasing_enabled() {
            return Err(BillingFailure::NotConfigured);
        }
        if plan_id.is_empty() || plan_id.len() > 64 {
            return Err(BillingFailure::Invalid("Unknown plan".into()));
        }
        let (device, license) = with_conn(&state, |c| {
            Ok((device_id(c)?, settings::get_string(c, LICENSE_KEY)?))
        })
        .await?;
        let resp: CheckoutResponse = state
            .billing
            .post(
                "/v1/checkout",
                serde_json::json!({ "deviceId": device, "planId": plan_id, "licenseId": license }),
            )
            .await?;
        if !valid_license_id(&resp.license_id) || !resp.url.starts_with("https://") {
            return Err(BillingFailure::Signature);
        }
        let lic = resp.license_id.clone();
        with_conn(&state, move |c| settings::set_string(c, LICENSE_KEY, &lic)).await?;
        open_external(&app, &resp.url)?;
        Ok(CheckoutDto { url: resp.url, license_id: resp.license_id })
    };
    Ok(run.await.into())
}

#[derive(Deserialize)]
struct UrlResponse {
    url: String,
}

#[tauri::command]
pub async fn billing_portal(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<BillingResult<String>, String> {
    let run = async {
        let license = with_conn(&state, |c| settings::get_string(c, LICENSE_KEY))
            .await?
            .ok_or(BillingFailure::NoPurchases)?;
        let resp: UrlResponse = state
            .billing
            .post("/v1/portal", serde_json::json!({ "licenseId": license }))
            .await?;
        if !resp.url.starts_with("https://") {
            return Err(BillingFailure::Signature);
        }
        open_external(&app, &resp.url)?;
        Ok(resp.url)
    };
    Ok(run.await.into())
}

#[tauri::command]
pub async fn entitlement_refresh(
    state: State<'_, AppState>,
) -> Result<BillingResult<BillingStatusDto>, String> {
    let run = async {
        let license = with_conn(&state, |c| settings::get_string(c, LICENSE_KEY))
            .await?
            .ok_or(BillingFailure::NoPurchases)?;
        match refresh_with(&state, &license).await {
            // A-09: an empty purchase list never revokes what is stored;
            // term grants simply lapse at their own end time.
            Ok(()) | Err(BillingFailure::NoPurchases) => status(&state).await,
            Err(e) => Err(e),
        }
    };
    Ok(run.await.into())
}

/// Restore on a new device with the license key from the original purchase.
#[tauri::command]
pub async fn entitlement_restore(
    state: State<'_, AppState>,
    license_id: String,
) -> Result<BillingResult<BillingStatusDto>, String> {
    let run = async {
        let license = license_id.trim().to_lowercase();
        if !valid_license_id(&license) {
            return Err(BillingFailure::Invalid(
                "That doesn't look like an EnZIME license key (enz_…)".into(),
            ));
        }
        refresh_with(&state, &license).await?;
        let lic = license.clone();
        with_conn(&state, move |c| settings::set_string(c, LICENSE_KEY, &lic)).await?;
        status(&state).await
    };
    Ok(run.await.into())
}

fn open_external(app: &tauri::AppHandle, url: &str) -> Result<(), BillingFailure> {
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| BillingFailure::Internal(format!("could not open browser: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::OptionalExtension;

    #[test]
    fn license_ids_validate() {
        assert!(valid_license_id("enz_0123456789abcdefghjkmnpqrs"));
        assert!(!valid_license_id("enz_0123456789ABCDEFGHJKMNPQRS"));
        assert!(!valid_license_id("enz_short"));
        assert!(!valid_license_id("xyz_0123456789abcdefghjkmnpqrs"));
    }

    #[test]
    fn unconfigured_build_disables_purchasing() {
        let cfg = BillingConfig { base_url: None, trusted_key: None };
        assert!(!cfg.purchasing_enabled());
    }

    #[test]
    fn device_id_is_stable_and_artifact_round_trips_through_sqlite() {
        let conn = storage_sqlite::open_with_pragmas(std::path::Path::new(":memory:")).unwrap();
        storage_sqlite::MigrationRunner::new(&conn).apply_initial().unwrap();
        let a = device_id(&conn).unwrap();
        assert_eq!(a, device_id(&conn).unwrap());

        let fixture = include_str!("../../../../crates/entitlement/tests/fixtures/ts_golden.txt");
        let mut lines = fixture.lines();
        let _key_id = lines.next().unwrap();
        let pk: [u8; 32] = hex::decode(lines.next().unwrap()).unwrap().try_into().unwrap();
        let _device = lines.next().unwrap();
        let bytes = hex::decode(lines.next().unwrap()).unwrap();
        let artifact = decode_artifact(&bytes).unwrap();
        let vk = VerifyingKey::from_bytes(&pk).unwrap();
        store_artifact(&conn, &artifact, &bytes, &vk, 1).unwrap();
        // Idempotent re-store.
        store_artifact(&conn, &artifact, &bytes, &vk, 2).unwrap();
        assert_eq!(stored_artifacts(&conn).unwrap(), vec![bytes]);
        let grants: i64 = conn
            .query_row("SELECT COUNT(*) FROM entitlement_grants", [], |r| r.get(0))
            .unwrap();
        assert_eq!(grants as usize, artifact.grants.len());
        let _ = conn
            .query_row("SELECT 1 FROM trust_keys WHERE key_kind='entitlement'", [], |r| r.get::<_, i64>(0))
            .optional()
            .unwrap()
            .expect("trust key stored");
    }

    /// End-to-end against the real TypeScript billing server
    /// (`node services/billing/scripts/dev-server.ts`, mock provider).
    /// Skips unless ENZIME_E2E_BILLING_URL + key env vars are set.
    #[tokio::test(flavor = "multi_thread")]
    async fn e2e_purchase_then_offline_restart() {
        let Ok(url) = std::env::var("ENZIME_E2E_BILLING_URL") else {
            eprintln!("skipping: ENZIME_E2E_BILLING_URL not set");
            return;
        };
        std::env::set_var("ENZIME_BILLING_URL", &url);
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("enzime.db");
        let open = || {
            let conn = storage_sqlite::open_with_pragmas(&db).unwrap();
            storage_sqlite::MigrationRunner::new(&conn).apply_initial().ok();
            AppState::new(conn, dir.path().to_path_buf()).unwrap()
        };
        let state = open();
        assert!(state.billing.config.purchasing_enabled());

        let device = with_conn(&state, device_id).await.unwrap();
        let checkout: CheckoutResponse = state
            .billing
            .post("/v1/checkout", serde_json::json!({ "deviceId": device, "planId": "lifetime" }))
            .await
            .unwrap();
        assert!(valid_license_id(&checkout.license_id));
        // Nothing purchased yet → NoPurchases, nothing unlocked.
        assert!(matches!(
            refresh_with(&state, &checkout.license_id).await,
            Err(BillingFailure::NoPurchases)
        ));
        // "Pay" on the mock hosted page, then refresh.
        reqwest::get(&checkout.url).await.unwrap().error_for_status().unwrap();
        refresh_with(&state, &checkout.license_id).await.unwrap();
        let allowed = |ds: Vec<CapabilityDecisionDto>, k: &str| {
            ds.into_iter().find(|d| d.capability_key == k).map(|d| d.allowed)
        };
        assert_eq!(allowed(decisions(&state).await.unwrap(), "annotations.rich_media"), Some(true));
        drop(state);

        // Restart with the billing server unreachable: entitlement holds offline.
        std::env::set_var("ENZIME_BILLING_URL", "http://127.0.0.1:9");
        let state = open();
        load_active(&state).await.unwrap();
        assert_eq!(allowed(decisions(&state).await.unwrap(), "annotations.rich_media"), Some(true));
        assert_eq!(allowed(decisions(&state).await.unwrap(), "models.premium_catalog"), Some(false));
    }
}

