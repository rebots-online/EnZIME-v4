//! Scoped passive archive resources and durable annotation playback.
mod routes;
mod stylesheet;
use routes::component;
pub use routes::{byte_range, parse_resource_path, passive_mime, ResourceRoute};
pub use stylesheet::rewrite_stylesheet;
#[cfg(test)]
mod tests;

use crate::state::AppState;
use domain::identity::{ContentDigest, EntryKey};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde::Serialize;
use std::{
    io::{Read, Seek, SeekFrom},
    sync::Arc,
};
use storage_sqlite::annotations::{annotation_assets, annotation_row, AnnotationAssetRow};
use tauri::{
    http::{Request, Response, StatusCode},
    Manager,
};
use tokio::sync::Semaphore;

const MAX_RESPONSE: u64 = 128 * 1024 * 1024;
fn encode(value: &str) -> String {
    utf8_percent_encode(value, NON_ALPHANUMERIC).to_string()
}
fn asset_url(annotation: &str, asset: &str) -> String {
    let origin = if cfg!(any(target_os = "windows", target_os = "android")) {
        "http://enzime-resource.localhost"
    } else {
        "enzime-resource://localhost"
    };
    format!(
        "{origin}/annotation/{}/{}",
        encode(annotation),
        encode(asset)
    )
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReaderAnnotationAsset {
    annotation_asset_id: String,
    annotation_id: String,
    asset_kind: String,
    mime_type: String,
    byte_length: u64,
    duration_ms: Option<u64>,
    resource_url: String,
}
fn live_assets(state: &AppState, annotation_id: &str) -> Result<Vec<AnnotationAssetRow>, String> {
    let conn = state
        .conn
        .lock()
        .map_err(|_| "annotation store unavailable")?;
    let head = annotation_row(&conn, annotation_id)
        .map_err(|_| "annotation store unavailable")?
        .ok_or("annotation not found")?;
    if head.deleted_at_ms.is_some() {
        return Err("annotation not found".into());
    }
    annotation_assets(&conn, annotation_id).map_err(|_| "annotation store unavailable".into())
}
fn list_assets(
    state: &AppState,
    annotation_id: &str,
) -> Result<Vec<ReaderAnnotationAsset>, String> {
    live_assets(state, annotation_id)?
        .into_iter()
        .filter(|r| passive_mime(&r.mime_type))
        .map(|r| {
            Ok(ReaderAnnotationAsset {
                resource_url: asset_url(&r.annotation_id, &r.annotation_asset_id),
                annotation_asset_id: r.annotation_asset_id,
                annotation_id: r.annotation_id,
                asset_kind: r.asset_kind,
                mime_type: r.mime_type,
                byte_length: u64::try_from(r.byte_length).map_err(|_| "invalid asset length")?,
                duration_ms: r
                    .duration_ms
                    .map(u64::try_from)
                    .transpose()
                    .map_err(|_| "invalid asset duration")?,
            })
        })
        .collect()
}
#[tauri::command]
pub async fn annotation_assets_list(
    state: tauri::State<'_, AppState>,
    annotation_id: String,
) -> Result<Vec<ReaderAnnotationAsset>, String> {
    list_assets(&state, &annotation_id)
}
fn response(
    status: StatusCode,
    mime: &str,
    body: Vec<u8>,
    length: u64,
    range: Option<String>,
) -> Response<Vec<u8>> {
    let mut builder = Response::builder()
        .status(status)
        .header("Content-Type", mime)
        .header("Content-Length", length.to_string())
        .header("X-Content-Type-Options", "nosniff")
        .header("Accept-Ranges", "bytes")
        .header("Content-Security-Policy", "default-src 'none'; sandbox")
        .header("Cache-Control", "no-store")
        .header("Access-Control-Allow-Origin", "null");
    if let Some(range) = range {
        builder = builder.header("Content-Range", range);
    }
    builder
        .body(body)
        .unwrap_or_else(|_| Response::new(Vec::new()))
}
fn error(status: StatusCode, message: &str, head: bool) -> Response<Vec<u8>> {
    response(
        status,
        "text/plain",
        if head {
            vec![]
        } else {
            message.as_bytes().to_vec()
        },
        message.len() as u64,
        None,
    )
}
fn selected_range(
    request: &Request<Vec<u8>>,
    length: u64,
) -> Result<(u64, u64, bool), Response<Vec<u8>>> {
    if request.headers().get_all("Range").iter().count() > 1 {
        return Err(response(
            StatusCode::RANGE_NOT_SATISFIABLE,
            "text/plain",
            vec![],
            0,
            Some(format!("bytes */{length}")),
        ));
    }
    let header = request.headers().get("Range");
    let range = header
        .map(|h| h.to_str())
        .transpose()
        .map_err(|_| ())
        .and_then(|h| byte_range(h, length).map_err(|_| ()));
    range.map_err(|_| {
        response(
            StatusCode::RANGE_NOT_SATISFIABLE,
            "text/plain",
            vec![],
            0,
            Some(format!("bytes */{length}")),
        )
    })
}
fn success(
    request: &Request<Vec<u8>>,
    mime: &str,
    bytes: Vec<u8>,
    start: u64,
    end: u64,
    length: u64,
    partial: bool,
) -> Response<Vec<u8>> {
    response(
        if partial {
            StatusCode::PARTIAL_CONTENT
        } else {
            StatusCode::OK
        },
        mime,
        if request.method() == "HEAD" {
            vec![]
        } else {
            bytes
        },
        end - start,
        partial.then(|| format!("bytes {start}-{}/{length}", end - 1)),
    )
}
async fn handle(state: &AppState, request: Request<Vec<u8>>) -> Response<Vec<u8>> {
    let head = request.method() == "HEAD";
    if request.method() != "GET" && !head {
        return error(
            StatusCode::METHOD_NOT_ALLOWED,
            "GET or HEAD required",
            false,
        );
    }
    let route = match parse_resource_path(request.uri().path()) {
        Ok(r) => r,
        Err(_) => return error(StatusCode::BAD_REQUEST, "Invalid resource path", head),
    };
    match route {
        ResourceRoute::Pack { pack_id, entry_key } => {
            let archive = state
                .archives
                .lock()
                .ok()
                .and_then(|m| m.get(&pack_id).cloned());
            let Some(archive) = archive else {
                return error(StatusCode::NOT_FOUND, "Pack unavailable", head);
            };
            // Check declared size before asking the archive to materialize a resource.
            match archive.entry(&EntryKey(entry_key.clone())).await {
                Ok(entry) if entry.byte_length <= MAX_RESPONSE => {}
                Ok(_) => {
                    return error(
                        StatusCode::PAYLOAD_TOO_LARGE,
                        "Resource exceeds response budget",
                        head,
                    )
                }
                Err(_) => {
                    return error(StatusCode::NOT_FOUND, "Resource unavailable offline", head)
                }
            }
            let resource = match archive.resource(&EntryKey(entry_key)).await {
                Ok(r) => r,
                Err(_) => {
                    return error(StatusCode::NOT_FOUND, "Resource unavailable offline", head)
                }
            };
            if !passive_mime(&resource.mime_type) {
                return error(
                    StatusCode::UNSUPPORTED_MEDIA_TYPE,
                    "Active resource blocked",
                    head,
                );
            }
            if resource.bytes.len() as u64 > MAX_RESPONSE {
                return error(
                    StatusCode::PAYLOAD_TOO_LARGE,
                    "Resource exceeds response budget",
                    head,
                );
            }
            let data = if resource
                .mime_type
                .split(';')
                .next()
                .unwrap_or("")
                .trim()
                .eq_ignore_ascii_case("text/css")
            {
                match std::str::from_utf8(&resource.bytes)
                    .ok()
                    .and_then(|css| rewrite_stylesheet(css, &pack_id).ok())
                {
                    Some(css) => bytes::Bytes::from(css),
                    None => {
                        return error(
                            StatusCode::UNSUPPORTED_MEDIA_TYPE,
                            "Unsafe stylesheet",
                            head,
                        )
                    }
                }
            } else {
                resource.bytes
            };
            let length = data.len() as u64;
            let (start, end, partial) = match selected_range(&request, length) {
                Ok(r) => r,
                Err(r) => return r,
            };
            success(
                &request,
                &resource.mime_type,
                if head {
                    vec![]
                } else {
                    data[start as usize..end as usize].to_vec()
                },
                start,
                end,
                length,
                partial,
            )
        }
        ResourceRoute::Annotation {
            annotation_id,
            asset_id,
        } => {
            let row = match live_assets(state, &annotation_id)
                .ok()
                .and_then(|rows| rows.into_iter().find(|r| r.annotation_asset_id == asset_id))
            {
                Some(r) => r,
                None => return error(StatusCode::NOT_FOUND, "Annotation asset unavailable", head),
            };
            if !passive_mime(&row.mime_type) {
                return error(
                    StatusCode::UNSUPPORTED_MEDIA_TYPE,
                    "Active resource blocked",
                    head,
                );
            }
            let length = match u64::try_from(row.byte_length) {
                Ok(n) => n,
                Err(_) => return error(StatusCode::NOT_FOUND, "Invalid asset metadata", head),
            };
            let digest = match ContentDigest::sha256_from_hex(row.object_sha256) {
                Ok(d) => d,
                Err(_) => return error(StatusCode::NOT_FOUND, "Invalid asset metadata", head),
            };
            let mut file = match state.annotations.object_store().read(&digest) {
                Ok(f) => f,
                Err(_) => {
                    return error(StatusCode::NOT_FOUND, "Annotation asset unavailable", head)
                }
            };
            if file.metadata().map(|m| m.len()).ok() != Some(length) {
                return error(StatusCode::CONFLICT, "Asset length mismatch", head);
            }
            let (start, end, partial) = match selected_range(&request, length) {
                Ok(r) => r,
                Err(r) => return r,
            };
            if end - start > MAX_RESPONSE {
                return error(
                    StatusCode::PAYLOAD_TOO_LARGE,
                    "Resource exceeds response budget",
                    head,
                );
            }
            let mut data = Vec::new();
            if !head {
                if file.seek(SeekFrom::Start(start)).is_err()
                    || file.take(end - start).read_to_end(&mut data).is_err()
                    || data.len() as u64 != end - start
                {
                    return error(StatusCode::INTERNAL_SERVER_ERROR, "Asset read failed", head);
                }
            }
            success(&request, &row.mime_type, data, start, end, length, partial)
        }
    }
}
pub fn register<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    let permits = Arc::new(Semaphore::new(2));
    builder.register_asynchronous_uri_scheme_protocol(
        "enzime-resource",
        move |context, request, responder| {
            let app = context.app_handle().clone();
            let permits = permits.clone();
            tauri::async_runtime::spawn(async move {
                let Ok(_permit) = permits.acquire_owned().await else {
                    responder.respond(error(
                        StatusCode::SERVICE_UNAVAILABLE,
                        "Resources unavailable",
                        false,
                    ));
                    return;
                };
                let state = app.state::<AppState>();
                responder.respond(handle(&state, request).await);
            });
        },
    )
}
