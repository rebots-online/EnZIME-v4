//! Sidecar import planning and commit (§9.6) over local storage.
//!
//! The application layer (`application::sidecar`) classifies artifacts
//! cryptographically; this module performs the *storage* half of the same
//! protocol: given a decoded envelope, produce the plan the UI shows
//! ([`plan_import`]) and, on user confirmation, materialize the imported
//! layer ([`commit_import`]).
//!
//! M1 trust posture: the device has no sidecar trust registry yet, so a
//! *signed* envelope from an unregistered key cannot be verified and is
//! refused (`unknown_key`) — consistent with the §9.6 quarantine policy the
//! codec implements. Unsigned envelopes import only with explicit
//! `accept_unsigned` confirmation and land on an `unverified` layer. When
//! the trust-registry milestone lands, `plan_import`/`commit_import` gain a
//! trusted-key set and route verification through the SignatureProvider
//! port.

use std::collections::HashMap;

use domain::annotation::{AnnotationError, TagSummary};
use serde::{Deserialize, Serialize};
use sidecar::cbor::Value as CborValue;
use sidecar::codec::{decode, encode_to_vec, sha256_hex};
use sidecar::types::{AnnotationType as SidecarAnnotationType, SidecarV1};
use storage_sqlite::annotations as sa;
use storage_sqlite::annotations::{SidecarImportRow, SidecarLayerRow, TagRow};

use crate::DbResultExt;
use ulid::Ulid;

use crate::{AnnotationsService, SidecarLayerView};

/// One user-visible conflict or skip recorded while planning/committing.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConflictItem {
    pub entity_kind: String,
    pub entity_id: String,
    pub explanation: String,
}

/// What [`AnnotationsService::plan_import`] reports to the confirm dialog.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPlan {
    pub artifact_sha256: String,
    pub signature_state: String,
    pub signer: Option<String>,
    pub annotation_count: usize,
    pub notebook_count: usize,
    pub tag_count: usize,
    pub object_bytes: u64,
    pub conflicts: Vec<ConflictItem>,
}

/// What [`AnnotationsService::commit_import`] reports.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportCommitReport {
    pub layer: SidecarLayerView,
    pub imported: usize,
    pub skipped: usize,
    pub conflicts: usize,
}

impl AnnotationsService {
    /// Decode and evaluate a sidecar artifact without mutating storage.
    pub fn plan_import(&self, bytes: &[u8]) -> Result<ImportPlan, AnnotationError> {
        let env = decode_envelope(bytes)?;
        let artifact_sha256 = artifact_sha256(&env);
        let unsigned = is_unsigned(&env);
        let mut conflicts: Vec<ConflictItem> = Vec::new();

        let signature_state = if unsigned {
            "unsigned"
        } else {
            // No M1 trust registry: a registered-key check is impossible,
            // and unregistered keys quarantine (§9.6).
            "unknown_key"
        };
        let signer = if unsigned {
            None
        } else {
            Some(hex_short(&env.creator.public_key))
        };

        let conn = self.lock()?;
        if sa::sidecar_import_by_sha256(&conn, &artifact_sha256).db()?.is_some() {
            conflicts.push(ConflictItem {
                entity_kind: "artifact".to_string(),
                entity_id: artifact_sha256.clone(),
                explanation: "artifact was already imported".to_string(),
            });
        }
        // Equal-or-newer local revisions of the same annotation id conflict.
        for ann in &env.annotations {
            if let Some(local) = sa::annotation_row(&conn, &ann.annotation_id).db()? {
                if i64::from(ann.revision) <= local.current_revision {
                    conflicts.push(ConflictItem {
                        entity_kind: "annotation".to_string(),
                        entity_id: ann.annotation_id.clone(),
                        explanation: format!(
                            "local revision {} is at or above incoming revision {}",
                            local.current_revision, ann.revision
                        ),
                    });
                }
            }
        }

        Ok(ImportPlan {
            artifact_sha256,
            signature_state: signature_state.to_string(),
            signer,
            annotation_count: env.annotations.len(),
            notebook_count: env.notebooks.len(),
            tag_count: env.tags.len(),
            object_bytes: env.objects.iter().map(|o| o.byte_length).sum(),
            conflicts,
        })
    }

    /// Materialize an accepted import: imported layer + notebooks + tags +
    /// annotations (on the imported layer) + the idempotency record.
    ///
    /// `accept_unsigned` is required for unsigned artifacts; signed
    /// artifacts from unregistered keys are refused until the trust
    /// registry exists.
    pub fn commit_import(
        &self,
        bytes: &[u8],
        accept_unsigned: bool,
        layer_name: &str,
    ) -> Result<ImportCommitReport, AnnotationError> {
        let env = decode_envelope(bytes)?;
        let artifact_sha256 = artifact_sha256(&env);
        let unsigned = is_unsigned(&env);

        let signature_state: String;
        let layer_trust: String;
        if unsigned {
            if !accept_unsigned {
                return Err(AnnotationError::Storage(
                    "unsigned_sidecar: explicit user confirmation is required".to_string(),
                ));
            }
            signature_state = "unsigned".to_string();
            layer_trust = "unverified".to_string();
        } else {
            // §9.6 quarantine posture for unregistered keys (see module docs).
            return Err(AnnotationError::Storage(format!(
                "unknown_key: no trusted key for key_id {} — artifact quarantined",
                env.signature.key_id
            )));
        }

        let mut conn = self.lock()?;
        if sa::sidecar_import_by_sha256(&conn, &artifact_sha256).db()?.is_some() {
            return Err(AnnotationError::Storage(format!(
                "duplicate_artifact: {artifact_sha256} was already imported"
            )));
        }

        let now = crate::now_ms();
        let layer_id = format!("layer-import-{}", Ulid::new());
        let mut imported: usize = 0;
        let mut skipped: usize = 0;
        let mut conflicts: usize = 0;

        let tx = conn
            .transaction()
            .map_err(|e| AnnotationError::Storage(e.to_string()))?;

        sa::insert_sidecar_layer(
            &tx,
            &SidecarLayerRow {
                sidecar_layer_id: layer_id.clone(),
                name: layer_name.to_string(),
                owner_device_id: None,
                visibility: "imported".to_string(),
                trust_state: layer_trust,
                source_artifact_id: Some(env.artifact_id.clone()),
                created_at_ms: now,
                updated_at_ms: now,
                annotation_count: 0,
            },
        )
        .db()?;

        // Notebooks: parents before children; a broken parent chain skips
        // the orphan rather than failing the import.
        let mut notebook_ok: HashMap<String, ()> = HashMap::new();
        for pass in 0..2 {
            for nb in &env.notebooks {
                if notebook_ok.contains_key(&nb.notebook_id) {
                    continue;
                }
                let parent_ready = nb
                    .parent_notebook_id
                    .as_ref()
                    .map(|p| notebook_ok.contains_key(p) || sa::notebook_exists(&tx, p).unwrap_or(false))
                    .unwrap_or(true);
                if !parent_ready {
                    continue;
                }
                let result = sa::insert_notebook(
                    &tx,
                    &sa::NotebookRow {
                        notebook_id: nb.notebook_id.clone(),
                        parent_notebook_id: nb.parent_notebook_id.clone(),
                        name: nb.name.clone(),
                        description: nb.description.clone(),
                        sort_order: now,
                        created_at_ms: now,
                        updated_at_ms: now,
                        annotation_count: 0,
                    },
                );
                match result {
                    Ok(()) => {
                        notebook_ok.insert(nb.notebook_id.clone(), ());
                    }
                    Err(_) if pass == 1 => {
                        conflicts += 1;
                    }
                    Err(_) => {}
                }
            }
        }

        // Tags: envelope tag ids are preserved so annotation tag links
        // survive the round trip.
        let mut tag_ok: HashMap<String, ()> = HashMap::new();
        for tag in &env.tags {
            let result = sa::insert_tag(
                &tx,
                &TagRow {
                    tag_id: tag.tag_id.clone(),
                    name: tag.name.clone(),
                    color_token: tag.color_token.clone(),
                    created_at_ms: now,
                },
            );
            match result {
                Ok(()) => {
                    tag_ok.insert(tag.tag_id.clone(), ());
                }
                Err(_) => {
                    // Name collision with a local tag: keep the local one.
                    if let Ok(Some(existing)) = sa::tag_by_name(&tx, &tag.name) {
                        tag_ok.insert(existing.tag_id, ());
                    } else {
                        conflicts += 1;
                    }
                }
            }
        }

        // Pack existence map: annotations bound to uninstalled packs are
        // skipped (their source is not locally readable).
        let bindings: Vec<String> = env
            .source_bindings
            .iter()
            .map(|b| b.pack_id.clone())
            .collect();
        for pack_id in &bindings {
            let exists: bool = tx
                .query_row(
                    "SELECT 1 FROM packs WHERE pack_id = ?1",
                    rusqlite::params![pack_id],
                    |_| Ok(true),
                )
                .unwrap_or(false);
            if !exists {
                conflicts += 1;
            }
        }

        for ann in &env.annotations {
            let pack_id = bindings.first().cloned().unwrap_or_default();
            let pack_installed = bindings.iter().any(|p| {
                tx.query_row(
                    "SELECT 1 FROM packs WHERE pack_id = ?1",
                    rusqlite::params![p],
                    |_| Ok(true),
                )
                .unwrap_or(false)
            });
            let local = sa::annotation_row(&tx, &ann.annotation_id).unwrap_or(None);
            if local.is_some() {
                skipped += 1;
                continue;
            }
            if !pack_installed {
                skipped += 1;
                continue;
            }
            let type_str = sidecar_type_str(ann.annotation_type);
            let selector_json = match serde_json::to_string(&cbor_to_json(&ann.selector)) {
                Ok(s) => s,
                Err(_) => {
                    skipped += 1;
                    continue;
                }
            };
            let style_json = match serde_json::to_string(&cbor_to_json(&ann.style)) {
                Ok(s) => s,
                Err(_) => "null".to_string(),
            };
            let article_key = selector_article_key(&ann.selector);
            let resolution = if article_key == "orphaned" {
                "orphaned"
            } else {
                "ambiguous"
            };
            let result = sa::insert_annotation(
                &tx,
                &sa::AnnotationRow {
                    annotation_id: ann.annotation_id.clone(),
                    annotation_type: type_str.to_string(),
                    pack_id,
                    article_key,
                    notebook_id: ann
                        .notebook_id
                        .clone()
                        .filter(|id| notebook_ok.contains_key(id)),
                    sidecar_layer_id: layer_id.clone(),
                    current_revision: i64::from(ann.revision),
                    resolution_state: resolution.to_string(),
                    pinned: false,
                    deleted_at_ms: None,
                    created_at_ms: ann.created_at_ms.max(now),
                    updated_at_ms: ann.updated_at_ms.max(now),
                },
            )
            .and_then(|()| {
                sa::insert_annotation_revision(
                    &tx,
                    &sa::AnnotationRevisionRow {
                        annotation_id: ann.annotation_id.clone(),
                        revision: i64::from(ann.revision),
                        body_text: ann.body_text.clone(),
                        selector_json,
                        style_json,
                        context_text: selector_context_text(&ann.selector),
                        source_revision_digest: None,
                        authored_by_device_id: None,
                        created_at_ms: ann.created_at_ms.max(now),
                    },
                )
            })
            .and_then(|()| {
                for tag_id in &ann.tag_ids {
                    if tag_ok.contains_key(tag_id) {
                        sa::link_annotation_tag(&tx, &ann.annotation_id, tag_id)?;
                    }
                }
                Ok(())
            });
            match result {
                Ok(()) => imported += 1,
                Err(_) => {
                    skipped += 1;
                    conflicts += 1;
                }
            }
        }

        sa::insert_sidecar_import(
            &tx,
            &SidecarImportRow {
                sidecar_import_id: Ulid::new().to_string(),
                artifact_sha256: artifact_sha256.clone(),
                signature_state: signature_state.clone(),
                imported_layer_id: Some(layer_id.clone()),
                record_count: imported as i64,
                conflict_count: conflicts as i64,
                imported_at_ms: now,
            },
        )
        .db()?;

        tx.commit()
            .map_err(|e| AnnotationError::Storage(e.to_string()))?;

        let layer = sa::sidecar_layer(&conn, &layer_id).db()?
            .ok_or_else(|| AnnotationError::Storage("layer vanished after commit".to_string()))?;
        Ok(ImportCommitReport {
            layer: SidecarLayerView::from_row(layer),
            imported,
            skipped,
            conflicts,
        })
    }

    /// The tag set currently linked to one annotation (IPC convenience).
    pub fn tags_for(&self, annotation_id: &str) -> Result<Vec<TagSummary>, AnnotationError> {
        let conn = self.lock()?;
        Ok(sa::annotation_tags(&conn, annotation_id)
            .db()?
            .into_iter()
            .map(|t| TagSummary {
                tag_id: t.tag_id,
                name: t.name,
                color_token: t.color_token,
            })
            .collect())
    }
}

fn decode_envelope(bytes: &[u8]) -> Result<SidecarV1, AnnotationError> {
    decode(bytes)
        .map_err(|e| AnnotationError::Storage(format!("invalid sidecar artifact: {e}")))
}

fn artifact_sha256(env: &SidecarV1) -> String {
    sha256_hex(&encode_to_vec(env))
}

fn is_unsigned(env: &SidecarV1) -> bool {
    env.signature.value.iter().all(|b| *b == 0)
}

fn hex_short(public_key: &[u8]) -> String {
    let head: Vec<String> = public_key
        .iter()
        .take(8)
        .map(|b| format!("{b:02x}"))
        .collect();
    if head.is_empty() {
        "unknown".to_string()
    } else {
        format!("{}…", head.join(":"))
    }
}

/// Convert the canonical-CBOR `Value` tree into a serde JSON tree so
/// selectors and styles can live in the §6.2 `*_json` columns.
fn cbor_to_json(value: &sidecar::cbor::Value) -> serde_json::Value {
    use sidecar::cbor::Value as C;
    match value {
        C::Unsigned(n) => serde_json::json!(*n),
        C::Negative(n) => serde_json::json!(*n),
        C::Bytes(b) => serde_json::json!(b64(b)),
        C::Text(s) => serde_json::json!(s),
        C::Array(items) => serde_json::Value::Array(items.iter().map(cbor_to_json).collect()),
        C::Map(entries) => {
            let mut map = serde_json::Map::new();
            for (k, v) in entries {
                let key = match k {
                    C::Text(s) => s.clone(),
                    other => format!("{other:?}"),
                };
                map.insert(key, cbor_to_json(v));
            }
            serde_json::Value::Object(map)
        }
        C::Bool(b) => serde_json::json!(b),
        C::Null => serde_json::Value::Null,
    }
}

fn b64(bytes: &[u8]) -> String {
    // Small dependency-free base64 (standard alphabet, padded) — selector
    // byte strings are rare and small in practice.
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        out.push(ALPHABET[(n >> 18) as usize & 63] as char);
        out.push(ALPHABET[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            ALPHABET[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            ALPHABET[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

/// Best-effort article-key extraction from a CBOR selector map: the
/// producer's `article.articleKey` (or `article_key`) text value. Anything
/// else resolves to `"orphaned"`.
fn selector_article_key(selector: &sidecar::cbor::Value) -> String {
    for key in ["articleKey", "article_key"] {
        if let Some(found) = selector
            .map_get("article")
            .and_then(|article| article.map_get(key))
            .and_then(|v| v.as_text())
        {
            return found.to_string();
        }
    }
    "orphaned".to_string()
}

fn selector_context_text(selector: &sidecar::cbor::Value) -> String {
    selector
        .map_get("capturedContext")
        .or_else(|| selector.map_get("captured_context"))
        .and_then(|v| v.as_text())
        .unwrap_or_default()
        .to_string()
}

/// The §6.2 CHECK vocabulary string for a sidecar annotation type.
fn sidecar_type_str(t: SidecarAnnotationType) -> &'static str {
    match t {
        SidecarAnnotationType::Highlight => "highlight",
        SidecarAnnotationType::Comment => "comment",
        SidecarAnnotationType::Ink => "ink",
        SidecarAnnotationType::Voice => "voice",
        SidecarAnnotationType::BookmarkNote => "bookmark_note",
    }
}

/// Helpers on the canonical-CBOR value tree, as an extension trait (the
/// type itself belongs to the `sidecar` crate).
trait CborValueExt {
    fn map_get(&self, key: &str) -> Option<&CborValue>;
    fn as_text(&self) -> Option<&str>;
}

impl CborValueExt for CborValue {
    fn map_get(&self, key: &str) -> Option<&CborValue> {
        if let CborValue::Map(entries) = self {
            entries
                .iter()
                .find(|(k, _)| matches!(k, CborValue::Text(t) if t == key))
                .map(|(_, v)| v)
        } else {
            None
        }
    }

    fn as_text(&self) -> Option<&str> {
        if let CborValue::Text(t) = self {
            Some(t)
        } else {
            None
        }
    }
}
