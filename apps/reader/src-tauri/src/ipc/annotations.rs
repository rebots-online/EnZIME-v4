//! Annotation-vertical IPC commands (§14.4 notes contract).
//!
//! Sixteen commands covering the multimodal-notes USP: annotation CRUD +
//! article listing, chunked ink/voice/video asset upload, notebook and tag
//! taxonomy, sidecar layer listing, and sidecar import planning/commit.
//! File *selection* for import is frontend-driven (`<input type="file">`);
//! `sidecar_import_plan` / `sidecar_import_commit` receive the artifact
//! bytes base64-encoded.
//!
//! # Typed error envelope
//! Unlike the M1 `Result<T, String>` commands in the parent module, every
//! command here returns the `CommandResult<T>` envelope the frontend
//! `annotationService` already parses — a structured
//! `{ ok, value?, error? }` with a machine-readable `code`. This vertical is
//! the pilot for the typed-error seam; the M1 commands migrate when they are
//! re-pointed at the application layer.

use annotations::{
    import_flow::{ImportCommitReport, ImportPlan},
    ListQuery, NotebookNode, SidecarLayerView,
};
use application::annotation_assets::{AssetUploadInit, UploadState};
use base64::Engine;
use domain::annotation::{
    AnnotationDraft, AnnotationError, AnnotationRecord, TagSummary,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use tauri::State;

use crate::state::{AppState, AssetUpload};

// ===========================================================================
// Envelope
// ===========================================================================

/// Machine-readable error code (the §14.4 `AppErrorCode` union subset the
/// annotation seam can produce).
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AppErrorCode {
    InvalidRequest,
    NotFound,
    RevisionConflict,
    FormatInvalid,
    SignatureInvalid,
    SourceChanged,
    Busy,
    Internal,
}

/// Structured IPC error (§14.4 `AppErrorDto`).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: AppErrorCode,
    pub message: String,
    pub correlation_id: String,
}

/// The envelope every annotation command returns.
#[derive(Debug, Serialize)]
pub struct CommandResult<T> {
    pub ok: bool,
    pub value: Option<T>,
    pub error: Option<AppError>,
}

impl<T> CommandResult<T> {
    fn ok(value: T) -> Self {
        Self {
            ok: true,
            value: Some(value),
            error: None,
        }
    }
}

fn fail<T>(code: AppErrorCode, message: String) -> CommandResult<T> {
    CommandResult {
        ok: false,
        value: None,
        error: Some(AppError {
            code,
            message,
            correlation_id: ulid::Ulid::new().to_string(),
        }),
    }
}

fn fail_from<T>(e: AnnotationError) -> CommandResult<T> {
    let (code, message) = match &e {
        AnnotationError::NotFound => (AppErrorCode::NotFound, e.to_string()),
        AnnotationError::RevisionConflict => (AppErrorCode::RevisionConflict, e.to_string()),
        AnnotationError::InvalidSelector(_) | AnnotationError::InvalidAsset(_) => {
            (AppErrorCode::InvalidRequest, e.to_string())
        }
        AnnotationError::Storage(detail) => {
            // Stable error-kind prefixes from the annotations crate's import
            // flow; see `import_flow::commit_import`.
            let code = if detail.starts_with("unsigned_sidecar:")
                || detail.starts_with("unknown_key:")
            {
                AppErrorCode::SignatureInvalid
            } else if detail.starts_with("duplicate_artifact:") {
                AppErrorCode::SourceChanged
            } else if detail.starts_with("invalid sidecar artifact:") {
                AppErrorCode::FormatInvalid
            } else {
                AppErrorCode::Internal
            };
            (code, e.to_string())
        }
        AnnotationError::SourceUnavailable => (AppErrorCode::Internal, e.to_string()),
    };
    fail(code, message)
}

/// Page envelope for `annotation_list` (§14.4 `PageResult`).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnotationPage {
    pub items: Vec<AnnotationRecord>,
    pub total: i64,
}

// ===========================================================================
// Annotation CRUD + listing
// ===========================================================================

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn annotation_list(
    state: State<'_, AppState>,
    pack_ids: Option<Vec<String>>,
    notebook_ids: Option<Vec<String>>,
    tag_ids: Option<Vec<String>>,
    sidecar_layer_ids: Option<Vec<String>>,
    resolution_states: Option<Vec<String>>,
    text: Option<String>,
    offset: i64,
    limit: i64,
) -> Result<CommandResult<AnnotationPage>, String> {
    let query = ListQuery {
        pack_ids: pack_ids.unwrap_or_default(),
        notebook_ids: notebook_ids.unwrap_or_default(),
        tag_ids: tag_ids.unwrap_or_default(),
        sidecar_layer_ids: sidecar_layer_ids.unwrap_or_default(),
        resolution_states: resolution_states.unwrap_or_default(),
        text,
        offset,
        limit: limit.clamp(1, 500),
    };
    let svc = state.annotations.clone();
    let result = tokio::task::spawn_blocking(move || svc.list(query))
        .await
        .map_err(|e: tokio::task::JoinError| e.to_string())?;
    match result {
        Ok((items, total)) => Ok(CommandResult::ok(AnnotationPage { items, total })),
        Err(e) => Ok(fail_from(e)),
    }
}

#[tauri::command]
pub async fn annotation_create(
    state: State<'_, AppState>,
    draft: AnnotationDraft,
) -> Result<CommandResult<AnnotationRecord>, String> {
    let svc = state.annotations.clone();
    let result = tokio::task::spawn_blocking(move || svc.create_impl(draft))
        .await
        .map_err(|e: tokio::task::JoinError| e.to_string())?;
    Ok(result.map(CommandResult::ok).unwrap_or_else(fail_from))
}

#[tauri::command]
pub async fn annotation_update(
    state: State<'_, AppState>,
    annotation_id: String,
    expected_revision: u32,
    draft: AnnotationDraft,
) -> Result<CommandResult<AnnotationRecord>, String> {
    let svc = state.annotations.clone();
    let result = tokio::task::spawn_blocking(move || {
        svc.update_impl(&annotation_id, expected_revision, draft)
    })
    .await
    .map_err(|e: tokio::task::JoinError| e.to_string())?;
    Ok(result.map(CommandResult::ok).unwrap_or_else(fail_from))
}

#[tauri::command]
pub async fn annotation_delete(
    state: State<'_, AppState>,
    annotation_id: String,
    expected_revision: u32,
) -> Result<CommandResult<()>, String> {
    let svc = state.annotations.clone();
    let result = tokio::task::spawn_blocking(move || {
        svc.soft_delete_impl(&annotation_id, expected_revision)
    })
    .await
    .map_err(|e: tokio::task::JoinError| e.to_string())?;
    Ok(result.map(|()| CommandResult::ok(())).unwrap_or_else(fail_from))
}

#[tauri::command]
pub async fn annotation_restore(
    state: State<'_, AppState>,
    annotation_id: String,
) -> Result<CommandResult<AnnotationRecord>, String> {
    let svc = state.annotations.clone();
    let result = tokio::task::spawn_blocking(move || svc.restore_impl(&annotation_id))
        .await
        .map_err(|e: tokio::task::JoinError| e.to_string())?;
    Ok(result.map(CommandResult::ok).unwrap_or_else(fail_from))
}

// ===========================================================================
// Chunked asset upload (§9.3) — ink, voice, video, file
// ===========================================================================

/// Per-kind byte ceilings for M1 uploads. Video clips dominate; ink strokes
/// are tiny. 256 KiB chunks keep each IPC payload well under WebView
/// message limits.
const MAX_ASSET_BYTES: &[(&str, u64)] = &[
    ("ink_svg", 4 * 1024 * 1024),
    ("voice_opus", 32 * 1024 * 1024),
    ("image", 32 * 1024 * 1024),
    ("file", 128 * 1024 * 1024),
];
const CHUNK_BYTES: u64 = 256 * 1024;

fn max_bytes_for(kind: &str) -> u64 {
    MAX_ASSET_BYTES
        .iter()
        .find(|(k, _)| *k == kind)
        .map(|(_, v)| *v)
        .unwrap_or(4 * 1024 * 1024)
}

/// Wire shape of a successful `annotation_asset_begin`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadBegin {
    pub upload_token: String,
    pub maximum_chunk_bytes: u64,
}

#[tauri::command]
pub async fn annotation_asset_begin(
    state: State<'_, AppState>,
    annotation_id: String,
    asset_kind: String,
    mime_type: String,
    expected_bytes: u64,
    duration_ms: Option<u64>,
) -> Result<CommandResult<UploadBegin>, String> {
    let max_byte_length = max_bytes_for(&asset_kind);
    if expected_bytes > max_byte_length {
        return Ok(fail(
            AppErrorCode::InvalidRequest,
            format!(
                "asset of {expected_bytes} bytes exceeds the {asset_kind} limit of {max_byte_length}"
            ),
        ));
    }
    let token = ulid::Ulid::new().to_string();
    let init = AssetUploadInit {
        upload_id: token.clone(),
        asset_kind,
        mime_type,
        duration_ms,
        max_byte_length,
    };
    let service = state.asset_service.clone();
    let begun = tokio::task::spawn_blocking(move || service.begin(init))
        .await
        .map_err(|e: tokio::task::JoinError| e.to_string())?;
    match begun {
        Ok(session) => {
            state.asset_uploads.lock().map_err(|e| e.to_string())?.insert(
                token.clone(),
                AssetUpload {
                    session,
                    annotation_id,
                    last_sequence_no: 0,
                },
            );
            Ok(CommandResult::ok(UploadBegin {
                upload_token: token,
                maximum_chunk_bytes: CHUNK_BYTES,
            }))
        }
        Err(e) => Ok(fail_from(e)),
    }
}

#[tauri::command]
pub async fn annotation_asset_chunk(
    state: State<'_, AppState>,
    upload_token: String,
    sequence_no: u64,
    bytes_base64: String,
) -> Result<CommandResult<()>, String> {
    let bytes = match base64::engine::general_purpose::STANDARD.decode(bytes_base64.as_bytes()) {
        Ok(b) => b,
        Err(e) => {
            return Ok(fail(
                AppErrorCode::InvalidRequest,
                format!("chunk is not valid base64: {e}"),
            ))
        }
    };
    let mut uploads = state.asset_uploads.lock().map_err(|e| e.to_string())?;
    let Some(upload) = uploads.get_mut(&upload_token) else {
        return Ok(fail(
            AppErrorCode::NotFound,
            format!("unknown upload token {upload_token}"),
        ));
    };
    if sequence_no != upload.last_sequence_no + 1 {
        return Ok(fail(
            AppErrorCode::InvalidRequest,
            format!(
                "out-of-order chunk: expected {}, got {sequence_no}",
                upload.last_sequence_no + 1
            ),
        ));
    }
    let service = state.asset_service.clone();
    let next = service.append_chunk(upload.session.clone(), bytes.into());
    match next {
        Ok(session) => {
            upload.session = session;
            upload.last_sequence_no = sequence_no;
            Ok(CommandResult::ok(()))
        }
        Err(e) => {
            // Size breach or non-open state: the session is consumed and
            // dead — drop it so the token cannot be reused.
            uploads.remove(&upload_token);
            Ok(fail_from(e))
        }
    }
}

#[tauri::command]
pub async fn annotation_asset_commit(
    state: State<'_, AppState>,
    upload_token: String,
    expected_sha256: String,
) -> Result<CommandResult<AnnotationRecord>, String> {
    let Some(upload) = state
        .asset_uploads
        .lock()
        .map_err(|e| e.to_string())?
        .remove(&upload_token)
    else {
        return Ok(fail(
            AppErrorCode::NotFound,
            format!("unknown upload token {upload_token}"),
        ));
    };
    if upload.session.state != UploadState::Open {
        return Ok(fail(
            AppErrorCode::Busy,
            "upload session is not open".to_string(),
        ));
    }
    // The frontend's content-hash gate: compare the assembled bytes to the
    // caller-declared digest before handing off to the §9.3 commit.
    let mut hasher = Sha256::new();
    hasher.update(&upload.session.accumulated);
    let actual = hex::encode(hasher.finalize());
    if !actual.eq_ignore_ascii_case(&expected_sha256) {
        return Ok(fail(
            AppErrorCode::InvalidRequest,
            format!("digest mismatch: expected {expected_sha256}, assembled {actual}"),
        ));
    }
    let annotation_id = upload.annotation_id.clone();
    let service = state.asset_service.clone();
    let committed = service.commit(upload.session, &annotation_id).await;
    match committed {
        Ok(_record) => {
            let svc = state.annotations.clone();
            let result =
                tokio::task::spawn_blocking(move || svc.get_impl(&annotation_id))
                    .await
                    .map_err(|e: tokio::task::JoinError| e.to_string())?;
            Ok(result.map(CommandResult::ok).unwrap_or_else(fail_from))
        }
        Err(e) => Ok(fail_from(e)),
    }
}

// ===========================================================================
// Taxonomy: notebooks, tags, layers
// ===========================================================================

#[tauri::command]
pub async fn notebook_tree(
    state: State<'_, AppState>,
) -> Result<CommandResult<Vec<NotebookNode>>, String> {
    let svc = state.annotations.clone();
    let result = tokio::task::spawn_blocking(move || svc.notebook_tree())
        .await
        .map_err(|e: tokio::task::JoinError| e.to_string())?;
    Ok(result.map(CommandResult::ok).unwrap_or_else(fail_from))
}

#[tauri::command]
pub async fn notebook_create(
    state: State<'_, AppState>,
    parent_notebook_id: Option<String>,
    name: String,
    description: String,
) -> Result<CommandResult<NotebookNode>, String> {
    let svc = state.annotations.clone();
    let result = tokio::task::spawn_blocking(move || {
        svc.notebook_create(parent_notebook_id.as_deref(), &name, &description)
    })
    .await
    .map_err(|e: tokio::task::JoinError| e.to_string())?;
    Ok(result.map(CommandResult::ok).unwrap_or_else(fail_from))
}

#[tauri::command]
pub async fn notebook_update(
    state: State<'_, AppState>,
    notebook_id: String,
    name: String,
    description: String,
    parent_notebook_id: Option<String>,
) -> Result<CommandResult<NotebookNode>, String> {
    let svc = state.annotations.clone();
    let result = tokio::task::spawn_blocking(move || {
        svc.notebook_update(
            &notebook_id,
            parent_notebook_id.as_deref(),
            &name,
            &description,
        )
    })
    .await
    .map_err(|e: tokio::task::JoinError| e.to_string())?;
    Ok(result.map(CommandResult::ok).unwrap_or_else(fail_from))
}

#[tauri::command]
pub async fn tag_list(
    state: State<'_, AppState>,
) -> Result<CommandResult<Vec<TagSummary>>, String> {
    let svc = state.annotations.clone();
    let result = tokio::task::spawn_blocking(move || svc.tag_list())
        .await
        .map_err(|e: tokio::task::JoinError| e.to_string())?;
    Ok(result.map(CommandResult::ok).unwrap_or_else(fail_from))
}

#[tauri::command]
pub async fn tag_create(
    state: State<'_, AppState>,
    name: String,
    color_token: String,
) -> Result<CommandResult<TagSummary>, String> {
    let svc = state.annotations.clone();
    let result = tokio::task::spawn_blocking(move || svc.tag_create(&name, &color_token))
        .await
        .map_err(|e: tokio::task::JoinError| e.to_string())?;
    Ok(result.map(CommandResult::ok).unwrap_or_else(fail_from))
}

#[tauri::command]
pub async fn sidecar_layers(
    state: State<'_, AppState>,
) -> Result<CommandResult<Vec<SidecarLayerView>>, String> {
    let svc = state.annotations.clone();
    let result = tokio::task::spawn_blocking(move || {
        // Ensure the local private layer exists so the toggle always has a
        // destination for fresh notes.
        svc.ensure_local_layer()?;
        svc.sidecar_layers()
    })
    .await
    .map_err(|e: tokio::task::JoinError| e.to_string())?;
    Ok(result.map(CommandResult::ok).unwrap_or_else(fail_from))
}

// ===========================================================================
// Sidecar import (bytes come from the frontend file picker)
// ===========================================================================

const MAX_SIDECAR_BYTES: usize = 64 * 1024 * 1024;

fn decode_sidecar<T>(bytes_base64: &str) -> Result<Vec<u8>, CommandResult<T>> {
    let raw = base64::engine::general_purpose::STANDARD
        .decode(bytes_base64.as_bytes())
        .map_err(|e| fail::<T>(AppErrorCode::FormatInvalid, format!("not base64: {e}")))?;
    if raw.len() > MAX_SIDECAR_BYTES {
        return Err(fail(
            AppErrorCode::InvalidRequest,
            format!(
                "sidecar artifact of {} bytes exceeds the {}-byte import limit",
                raw.len(),
                MAX_SIDECAR_BYTES
            ),
        ));
    }
    Ok(raw)
}

#[tauri::command]
pub async fn sidecar_import_plan(
    state: State<'_, AppState>,
    bytes_base64: String,
) -> Result<CommandResult<ImportPlan>, String> {
    let bytes = match decode_sidecar(&bytes_base64) {
        Ok(b) => b,
        Err(envelope) => return Ok(envelope),
    };
    let svc = state.annotations.clone();
    let result = tokio::task::spawn_blocking(move || svc.plan_import(&bytes))
        .await
        .map_err(|e: tokio::task::JoinError| e.to_string())?;
    Ok(result.map(CommandResult::ok).unwrap_or_else(fail_from))
}

#[tauri::command]
pub async fn sidecar_import_commit(
    state: State<'_, AppState>,
    bytes_base64: String,
    accept_unsigned: bool,
    layer_name: String,
) -> Result<CommandResult<ImportCommitReport>, String> {
    let bytes = match decode_sidecar(&bytes_base64) {
        Ok(b) => b,
        Err(envelope) => return Ok(envelope),
    };
    if layer_name.trim().is_empty() {
        return Ok(fail(
            AppErrorCode::InvalidRequest,
            "layer name must not be empty".to_string(),
        ));
    }
    let svc = state.annotations.clone();
    let result = tokio::task::spawn_blocking(move || {
        svc.commit_import(&bytes, accept_unsigned, layer_name.trim())
    })
    .await
    .map_err(|e: tokio::task::JoinError| e.to_string())?;
    Ok(result.map(CommandResult::ok).unwrap_or_else(fail_from))
}
