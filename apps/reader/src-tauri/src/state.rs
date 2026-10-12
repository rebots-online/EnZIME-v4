//! Process-wide application state for the EnZIME Reader Tauri shell.
//!
//! Holds the single SQLite connection (§6.1 pragmas already applied at open
//! time) and the table of currently-open ZIM archives keyed by pack id. Both
//! the connection and the archive table are behind an `Arc<Mutex<…>>` so
//! command handlers can move a clone into `tokio::task::spawn_blocking`
//! without borrowing `tauri::State` across an await.
//!
//! DynDon plans and cluster metadata are durable; active archives are restored
//! from registered storage roots before the shell becomes ready.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use annotations::AnnotationsService;
use application::annotation_assets::{AnnotationAssetService, AssetUploadSession};
use object_store::ObjectStore;
use ports::zim::ZimArchive;
use rusqlite::Connection;

use ports::transfer::TransferSource;
use zim_cluster_cache::disk::DiskPackStore;

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
    /// Native transport and process-local handles over durable pack stores.
    pub transfer_source: Arc<dyn TransferSource>,
    pub dyndon_stores: Arc<Mutex<HashMap<PathBuf, Arc<DiskPackStore>>>>,
    pub acquisition_gate: tokio::sync::Mutex<()>,
    pub restore_errors: Arc<Mutex<Vec<String>>>,
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
        let source = Arc::new(transfer_http::HttpTransferSource::new().map_err(|e| e.to_string())?);
        Self::new_with_transfer(conn, app_data_dir, source)
    }
    pub fn new_with_transfer(
        conn: Connection,
        app_data_dir: PathBuf,
        transfer_source: Arc<dyn TransferSource>,
    ) -> Result<Self, String> {
        let objects = ObjectStore::open(&app_data_dir).map_err(|e| e.to_string())?;
        let objects = Arc::new(objects);
        let conn = Arc::new(Mutex::new(conn));
        let annotations = Arc::new(AnnotationsService::new(conn.clone(), objects));
        let asset_service = Arc::new(AnnotationAssetService::new(annotations.asset_sink()));
        Ok(Self {
            conn,
            archives: Arc::new(Mutex::new(HashMap::new())),
            app_data_dir,
            transfer_source,
            dyndon_stores: Arc::new(Mutex::new(HashMap::new())),
            acquisition_gate: tokio::sync::Mutex::new(()),
            restore_errors: Arc::new(Mutex::new(Vec::new())),
            annotations,
            asset_service,
            asset_uploads: Arc::new(Mutex::new(HashMap::new())),
        })
    }
}
