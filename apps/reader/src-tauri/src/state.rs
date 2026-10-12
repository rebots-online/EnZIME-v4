//! Process-wide application state for the EnZIME Reader Tauri shell.
//!
//! Holds the single SQLite connection (§6.1 pragmas already applied at open
//! time) and the table of currently-open ZIM archives keyed by pack id. Both
//! the connection and the archive table are behind an `Arc<Mutex<…>>` so
//! command handlers can move a clone into `tokio::task::spawn_blocking`
//! without borrowing `tauri::State` across an await.
//!
//! ## M1 DynDon additions
//! `app_data_dir` is the resolved Tauri app-data directory — used by
//! `storage_roots_list` to synthesize the M1 default destination when the
//! `storage_roots` table is empty. `install_plans` is a volatile plan store:
//! `install_plan` stashes the computed plan keyed by its id; `install_start`
//! retrieves it. Persisting the *selection* (which clusters to fetch) is
//! handled through the durable `selection_profile_revisions` rows; the plan
//! envelope itself is process-local because a plan is only ever consumed by
//! the immediately-following `install_start` call.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use annotations::AnnotationsService;
use application::annotation_assets::{AnnotationAssetService, AssetUploadSession};
use object_store::ObjectStore;
use ports::zim::ZimArchive;
use rusqlite::Connection;

use crate::ipc::PersistedInstallPlan;

/// One in-flight annotation-asset upload keyed by its upload token. The
/// session value is owned by the host (the shell) per the §9.3 contract —
/// the application service transforms it deterministically.
pub struct AssetUpload {
    pub session: AssetUploadSession,
    pub annotation_id: String,
    pub last_sequence_no: u64,
}

/// Managed Tauri state. `Send + Sync` so it can be registered with
/// `tauri::App::manage` and borrowed from async command handlers.
pub struct AppState {
    /// The single read/write SQLite connection. M1 serializes all DB access
    /// through this mutex; no command holds the lock across an `await`.
    pub conn: Arc<Mutex<Connection>>,
    /// Open ZIM archives, keyed by pack id. For installed DynDon packs the
    /// entry is a `ClusterCacheArchive` (registered by `install_start`); for
    /// any directly-opened archive it is the underlying `ZimArchive` impl.
    pub archives: Arc<Mutex<HashMap<String, Arc<dyn ZimArchive>>>>,
    /// Resolved platform app-data directory (`app.path().app_data_dir()`).
    /// Used as the M1 default storage root when the DB has none.
    pub app_data_dir: PathBuf,
    /// Volatile install-plan store keyed by `install_plan_id`. Plans are
    /// consumed by the immediately-following `install_start` call; the
    /// selected clusters themselves are persisted in
    /// `selection_profile_revisions` so they survive a restart even if the
    /// plan envelope does not.
    pub install_plans: Arc<Mutex<HashMap<String, PersistedInstallPlan>>>,
    /// The annotation-domain service over the shared connection + object
    /// store (the `crates/annotations` adapter).
    pub annotations: Arc<AnnotationsService>,
    /// §9.3 chunked-asset upload orchestrator (application layer).
    pub asset_service: Arc<AnnotationAssetService>,
    /// In-flight asset uploads keyed by upload token.
    pub asset_uploads: Arc<Mutex<HashMap<String, AssetUpload>>>,
}

impl AppState {
    /// Construct from an already-migrated connection and the resolved
    /// app-data directory. The object store is rooted under the same
    /// directory; the annotation service shares the connection.
    pub fn new(conn: Connection, app_data_dir: PathBuf) -> Result<Self, String> {
        let objects = ObjectStore::open(&app_data_dir).map_err(|e| e.to_string())?;
        let objects = Arc::new(objects);
        let conn = Arc::new(Mutex::new(conn));
        let annotations = Arc::new(AnnotationsService::new(conn.clone(), objects));
        let asset_service = Arc::new(AnnotationAssetService::new(annotations.asset_sink()));
        Ok(Self {
            conn,
            archives: Arc::new(Mutex::new(HashMap::new())),
            app_data_dir,
            install_plans: Arc::new(Mutex::new(HashMap::new())),
            annotations,
            asset_service,
            asset_uploads: Arc::new(Mutex::new(HashMap::new())),
        })
    }
}
