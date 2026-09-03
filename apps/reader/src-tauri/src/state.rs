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

use ports::zim::ZimArchive;
use rusqlite::Connection;

use crate::ipc::PersistedInstallPlan;

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
}

impl AppState {
    /// Construct from an already-migrated connection and the resolved
    /// app-data directory.
    pub fn new(conn: Connection, app_data_dir: PathBuf) -> Self {
        Self {
            conn: Arc::new(Mutex::new(conn)),
            archives: Arc::new(Mutex::new(HashMap::new())),
            app_data_dir,
            install_plans: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
