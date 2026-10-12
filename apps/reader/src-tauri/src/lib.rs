pub mod ipc;
mod reader_resources;
pub mod state;

use std::path::PathBuf;

use state::AppState;
use tauri::Manager;

/// Resolve the M1 database location. Prefers the platform app-data directory
/// (via the Tauri path resolver); falls back to a directory next to the
/// current executable for dev runs where the resolver is unavailable.
fn resolve_db_path(app: &tauri::App) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let dir = match app.path().app_data_dir() {
        Ok(d) => d,
        Err(_) => {
            let exe = std::env::current_exe()?;
            exe.parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| PathBuf::from("."))
        }
    };
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join("enzime.db"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    reader_resources::register(tauri::Builder::default())
        .setup(|app| {
            let db_path = resolve_db_path(app)?;
            let conn = storage_sqlite::open_with_pragmas(&db_path)
                .map_err(|e| Box::<dyn std::error::Error>::from(e.to_string()))?;
            storage_sqlite::MigrationRunner::new(&conn)
                .apply_initial()
                .map_err(|e| Box::<dyn std::error::Error>::from(e.to_string()))?;
            storage_sqlite::apply_dyndon_schema(&conn)?;
            // The app-data directory is the M1 default storage root when the
            // `storage_roots` table has no user-chosen root yet. Resolve it
            // from the Tauri path API; fall back to the DB's parent dir for
            // dev runs where the resolver is unavailable.
            let app_data_dir = match app.path().app_data_dir() {
                Ok(d) => d,
                Err(_) => db_path
                    .parent()
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(|| PathBuf::from(".")),
            };
            let state = AppState::new(conn, app_data_dir)
                .map_err(|e| Box::<dyn std::error::Error>::from(e.to_string()))?;
            // The local private layer must exist before the first note is
            // taken; creating it at boot keeps every command path simple.
            state
                .annotations
                .ensure_local_layer()
                .map_err(|e| Box::<dyn std::error::Error>::from(e.to_string()))?;
            tauri::async_runtime::block_on(ipc::dyndon::restore_dyndon_archives(&state))?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::app_bootstrap,
            ipc::home_snapshot,
            ipc::readiness_get,
            ipc::reader_open,
            ipc::reader_article,
            // M1 Library / DynDon vertical (§14.4 wire contract).
            ipc::storage_roots_list,
            ipc::pack_list,
            ipc::catalog_search,
            ipc::install_plan,
            ipc::install_start,
            ipc::install_cancel,
            ipc::job_list,
            // Annotation vertical (§14.4 notes contract) — multimodal-notes USP.
            ipc::annotations::annotation_list,
            reader_resources::annotation_assets_list,
            ipc::annotations::annotation_create,
            ipc::annotations::annotation_update,
            ipc::annotations::annotation_delete,
            ipc::annotations::annotation_restore,
            ipc::annotations::annotation_asset_begin,
            ipc::annotations::annotation_asset_chunk,
            ipc::annotations::annotation_asset_commit,
            ipc::annotations::notebook_tree,
            ipc::annotations::notebook_create,
            ipc::annotations::notebook_update,
            ipc::annotations::tag_list,
            ipc::annotations::tag_create,
            ipc::annotations::sidecar_layers,
            ipc::annotations::sidecar_import_plan,
            ipc::annotations::sidecar_import_commit,
        ])
        .run(tauri::generate_context!())
        .expect("error while running EnZIME Reader");
}
