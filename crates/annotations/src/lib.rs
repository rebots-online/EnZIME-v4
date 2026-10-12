#![forbid(unsafe_code)]

//! The `AnnotationService` adapter: the persistence-facing implementation of
//! [`ports::AnnotationService`] over `storage-sqlite` + the object store.
//!
//! This crate fills the ring-3 adapter seat the hexagonal layout reserves for
//! annotations (ARCHITECTURE.md §3/§9): it owns SQLite transactions, id
//! allocation, and clock reads, and nothing above the storage layer does. The
//! Tauri shell constructs it and calls it under `spawn_blocking`.
//!
//! Wire-shape note: `domain::annotation` types serialize camelCase, matching
//! the frontend contract (§14.4) byte-for-byte, so records returned from this
//! crate cross IPC without a DTO translation layer.
//!
//! M1 scoping decisions (deliberate, documented):
//! - Notebook/tag/layer taxonomy operations are inherent methods on
//!   [`AnnotationsService`] rather than new `ports` traits; the port surface
//!   stays at the §9.3 contract until a second consumer justifies generalizing.
//! - Selector→article resolution is heuristic (`resolve_selector`): the
//!   backend has no article DOM at write time, so text-quote selectors are
//!   marked `ambiguous` and structural selectors `resolved`. The
//!   `ports::AnnotationResolver` port remains the place for a verifying
//!   resolver when the reader integration lands.
//! - Video ("film") notes are `comment` annotations carrying a `file` asset
//!   with a video MIME — the §6.2 CHECK constraints predate the modality and
//!   a table-rebuild migration is not warranted for M1.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use application::annotation_assets::CommittedAssetSink;
use async_trait::async_trait;
use bytes::Bytes;
use domain::annotation::{
    AnnotationAssetInput, AnnotationAssetRecord, AnnotationDraft, AnnotationError,
    AnnotationRecord, AnnotationResolution, AnnotationResolutionState, AnnotationStyle,
    AnnotationType, SelectorBundle, TagSummary,
};
use domain::identity::{ArticleKey, ContentDigest, DigestAlgorithm, PackId};
use object_store::{store::ObjectStore, AcceptRawBytes, CommitRequest};
use ports::annotation::AnnotationService as AnnotationServicePort;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use storage_sqlite::annotations as sa;
use storage_sqlite::annotations::{
    AnnotationFilter, AnnotationRow, AnnotationRevisionRow, NotebookRow, SidecarLayerRow,
    TagRow,
};
use ulid::Ulid;

pub mod import_flow;

/// Fixed id of the device's own private layer (created lazily).
pub const LOCAL_LAYER_ID: &str = "layer-self-local";

/// Facade over the annotation-domain tables. Cheap to clone via `Arc`.
pub struct AnnotationsService {
    conn: Arc<Mutex<Connection>>,
    objects: Arc<ObjectStore>,
    sink: Arc<AssetSink>,
}

impl AnnotationsService {
    pub fn new(conn: Arc<Mutex<Connection>>, objects: Arc<ObjectStore>) -> Self {
        let sink = Arc::new(AssetSink {
            conn: conn.clone(),
            objects: objects.clone(),
        });
        Self {
            conn,
            objects,
            sink,
        }
    }

    /// The injected [`CommittedAssetSink`] shared with the application-layer
    /// chunked-upload service.
    pub fn asset_sink(&self) -> Arc<AssetSink> {
        self.sink.clone()
    }

    /// Read access to the object store (asset bytes are content-addressed
    /// objects; playback/retrieval resolves them by digest).
    pub fn object_store(&self) -> &Arc<ObjectStore> {
        &self.objects
    }

    fn lock(&self) -> Result<MutexGuard<'_, Connection>, AnnotationError> {
        self.conn.lock().map_err(|e: PoisonError<MutexGuard<'_, Connection>>| {
            AnnotationError::Storage(format!("connection lock poisoned: {e}"))
        })
    }

    /// Create the device's local private layer if it does not exist and
    /// return its id.
    pub fn ensure_local_layer(&self) -> Result<String, AnnotationError> {
        let conn = self.lock()?;
        if sa::sidecar_layer(&conn, LOCAL_LAYER_ID).db()?.is_some() {
            return Ok(LOCAL_LAYER_ID.to_string());
        }
        let now = now_ms();
        sa::insert_sidecar_layer(
            &conn,
            &SidecarLayerRow {
                sidecar_layer_id: LOCAL_LAYER_ID.to_string(),
                name: "My notes".to_string(),
                owner_device_id: None,
                visibility: "private".to_string(),
                trust_state: "local".to_string(),
                source_artifact_id: None,
                created_at_ms: now,
                updated_at_ms: now,
                annotation_count: 0,
            },
        )
        .db()?;
        Ok(LOCAL_LAYER_ID.to_string())
    }

    // =======================================================================
    // Taxonomy: notebooks, tags, layers
    // =======================================================================

    pub fn notebook_create(
        &self,
        parent_notebook_id: Option<&str>,
        name: &str,
        description: &str,
    ) -> Result<NotebookNode, AnnotationError> {
        if name.trim().is_empty() {
            return Err(AnnotationError::InvalidSelector(
                "notebook name must not be empty".to_string(),
            ));
        }
        let conn = self.lock()?;
        if let Some(parent) = parent_notebook_id {
            if !sa::notebook_exists(&conn, parent).db()? {
                return Err(AnnotationError::NotFound);
            }
        }
        let now = now_ms();
        let row = NotebookRow {
            notebook_id: Ulid::new().to_string(),
            parent_notebook_id: parent_notebook_id.map(str::to_string),
            name: name.trim().to_string(),
            description: description.to_string(),
            sort_order: now,
            created_at_ms: now,
            updated_at_ms: now,
            annotation_count: 0,
        };
        sa::insert_notebook(&conn, &row).db()?;
        Ok(NotebookNode::from_row(&row, Vec::new()))
    }

    pub fn notebook_update(
        &self,
        notebook_id: &str,
        parent_notebook_id: Option<&str>,
        name: &str,
        description: &str,
    ) -> Result<NotebookNode, AnnotationError> {
        if name.trim().is_empty() {
            return Err(AnnotationError::InvalidSelector(
                "notebook name must not be empty".to_string(),
            ));
        }
        let conn = self.lock()?;
        if parent_notebook_id == Some(notebook_id) {
            return Err(AnnotationError::InvalidSelector(
                "a notebook cannot be its own parent".to_string(),
            ));
        }
        if let Some(parent) = parent_notebook_id {
            if !sa::notebook_exists(&conn, parent).db()? {
                return Err(AnnotationError::NotFound);
            }
        }
        let updated = sa::update_notebook(
            &conn,
            notebook_id,
            parent_notebook_id,
            name.trim(),
            description,
            now_ms(),
        )
        .db()?;
        if updated == 0 {
            return Err(AnnotationError::NotFound);
        }
        let notebooks = sa::list_notebooks(&conn).db()?;
        let row = notebooks
            .into_iter()
            .find(|n| n.notebook_id == notebook_id)
            .ok_or(AnnotationError::NotFound)?;
        Ok(NotebookNode::from_row(&row, Vec::new()))
    }

    /// The full notebook forest with per-node live annotation counts.
    pub fn notebook_tree(&self) -> Result<Vec<NotebookNode>, AnnotationError> {
        let conn = self.lock()?;
        let rows = sa::list_notebooks(&conn).db()?;
        Ok(build_notebook_tree(rows))
    }

    /// Create a tag; idempotent by name (case-insensitive) — an existing
    /// tag with the same name is returned unchanged rather than erroring.
    pub fn tag_create(
        &self,
        name: &str,
        color_token: &str,
    ) -> Result<TagSummary, AnnotationError> {
        if name.trim().is_empty() {
            return Err(AnnotationError::InvalidSelector(
                "tag name must not be empty".to_string(),
            ));
        }
        let conn = self.lock()?;
        if let Some(existing) = sa::tag_by_name(&conn, name.trim()).db()? {
            return Ok(TagSummary {
                tag_id: existing.tag_id,
                name: existing.name,
                color_token: existing.color_token,
            });
        }
        let row = TagRow {
            tag_id: Ulid::new().to_string(),
            name: name.trim().to_string(),
            color_token: color_token.to_string(),
            created_at_ms: now_ms(),
        };
        sa::insert_tag(&conn, &row).db()?;
        Ok(TagSummary {
            tag_id: row.tag_id,
            name: row.name,
            color_token: row.color_token,
        })
    }

    pub fn tag_list(&self) -> Result<Vec<TagSummary>, AnnotationError> {
        let conn = self.lock()?;
        Ok(sa::list_tags(&conn).db()?
            .into_iter()
            .map(|t| TagSummary {
                tag_id: t.tag_id,
                name: t.name,
                color_token: t.color_token,
            })
            .collect())
    }

    /// Every sidecar layer with its live annotation count, local layer
    /// first.
    pub fn sidecar_layers(&self) -> Result<Vec<SidecarLayerView>, AnnotationError> {
        let conn = self.lock()?;
        Ok(sa::list_sidecar_layers(&conn).db()?
            .into_iter()
            .map(SidecarLayerView::from_row)
            .collect())
    }

    // =======================================================================
    // Listing
    // =======================================================================

    /// Filtered, paginated listing for the notes browser. Returns the page
    /// and the total match count.
    pub fn list(
        &self,
        query: ListQuery,
    ) -> Result<(Vec<AnnotationRecord>, i64), AnnotationError> {
        let conn = self.lock()?;
        let filter = AnnotationFilter {
            pack_ids: query.pack_ids,
            notebook_ids: query.notebook_ids,
            tag_ids: query.tag_ids,
            sidecar_layer_ids: query.sidecar_layer_ids,
            resolution_states: query.resolution_states,
            article_key: None,
            text: query.text,
            include_deleted: false,
        };
        let (rows, total) = sa::list_annotations(&conn, &filter, query.offset, query.limit).db()?;
        let mut records = Vec::with_capacity(rows.len());
        for row in rows {
            records.push(Self::assemble(&conn, &row)?);
        }
        Ok((records, total))
    }

    /// Every live annotation attached to one article (the reader overlay's
    /// data source).
    pub fn list_for_article_records(
        &self,
        pack_id: &str,
        article_key: &str,
    ) -> Result<Vec<AnnotationRecord>, AnnotationError> {
        let conn = self.lock()?;
        let filter = AnnotationFilter {
            pack_ids: vec![pack_id.to_string()],
            article_key: Some(article_key.to_string()),
            ..Default::default()
        };
        let (rows, _) = sa::list_annotations(&conn, &filter, 0, i64::MAX).db()?;
        let mut records = Vec::with_capacity(rows.len());
        for row in rows {
            records.push(Self::assemble(&conn, &row)?);
        }
        Ok(records)
    }

    // =======================================================================
    // Record assembly + shared helpers
    // =======================================================================

    fn assemble(
        conn: &Connection,
        head: &AnnotationRow,
    ) -> Result<AnnotationRecord, AnnotationError> {
        let revision = sa::latest_revision(conn, &head.annotation_id)
            .db()?
            .ok_or(AnnotationError::NotFound)?;
        let selector: SelectorBundle = serde_json::from_str(&revision.selector_json)
            .map_err(|e| AnnotationError::Storage(format!("stored selector unreadable: {e}")))?;
        let style: AnnotationStyle = serde_json::from_str(&revision.style_json)
            .map_err(|e| AnnotationError::Storage(format!("stored style unreadable: {e}")))?;
        let tags = sa::annotation_tags(conn, &head.annotation_id).db()?
            .into_iter()
            .map(|t| TagSummary {
                tag_id: t.tag_id,
                name: t.name,
                color_token: t.color_token,
            })
            .collect();
        Ok(AnnotationRecord {
            annotation_id: head.annotation_id.clone(),
            annotation_type: annotation_type_from_str(&head.annotation_type),
            selector,
            body_text: revision.body_text,
            style,
            notebook_id: head.notebook_id.clone(),
            tags,
            sidecar_layer_id: head.sidecar_layer_id.clone(),
            resolution: AnnotationResolution {
                state: resolution_state_from_str(&head.resolution_state),
                confidence: 1.0,
                matched_start: None,
                matched_end: None,
                explanation: String::new(),
            },
            pinned: head.pinned,
            revision: u32::try_from(head.current_revision).unwrap_or(u32::MAX),
            created_at_ms: head.created_at_ms,
            updated_at_ms: head.updated_at_ms,
        })
    }

    pub fn create_impl(&self, draft: AnnotationDraft) -> Result<AnnotationRecord, AnnotationError> {
        let now = now_ms();
        let id = Ulid::new().to_string();
        let resolution = resolve_selector(&draft.selector);
        let selector_json = serde_json::to_string(&draft.selector)
            .map_err(|e| AnnotationError::Storage(e.to_string()))?;
        let style_json = serde_json::to_string(&draft.style)
            .map_err(|e| AnnotationError::Storage(e.to_string()))?;

        let mut conn = self.lock()?;
        let tx = conn
            .transaction()
            .map_err(|e| AnnotationError::Storage(e.to_string()))?;
        sa::insert_annotation(
            &tx,
            &AnnotationRow {
                annotation_id: id.clone(),
                annotation_type: annotation_type_str(draft.annotation_type).to_string(),
                pack_id: draft.selector.article.pack_id.0.clone(),
                article_key: draft.selector.article.article_key.0.clone(),
                notebook_id: draft.notebook_id.clone(),
                sidecar_layer_id: draft.sidecar_layer_id.clone(),
                current_revision: 1,
                resolution_state: resolution_state_str(resolution.state).to_string(),
                pinned: false,
                deleted_at_ms: None,
                created_at_ms: now,
                updated_at_ms: now,
            },
        )
        .db()?;
        sa::insert_annotation_revision(
            &tx,
            &AnnotationRevisionRow {
                annotation_id: id.clone(),
                revision: 1,
                body_text: draft.body_text.clone(),
                selector_json,
                style_json,
                context_text: draft.selector.captured_context.clone(),
                source_revision_digest: draft.selector.article.source_revision_digest.clone(),
                authored_by_device_id: None,
                created_at_ms: now,
            },
        )
        .db()?;
        for tag_id in &draft.tag_ids {
            sa::link_annotation_tag(&tx, &id, tag_id).db()?;
        }
        tx.commit()
            .map_err(|e| AnnotationError::Storage(e.to_string()))?;
        drop(conn);
        self.get_impl(&id)
    }

    pub fn get_impl(&self, annotation_id: &str) -> Result<AnnotationRecord, AnnotationError> {
        let conn = self.lock()?;
        let head = sa::annotation_row(&conn, annotation_id)
            .db()?
            .ok_or(AnnotationError::NotFound)?;
        if head.deleted_at_ms.is_some() {
            return Err(AnnotationError::NotFound);
        }
        Self::assemble(&conn, &head)
    }

    pub fn update_impl(
        &self,
        annotation_id: &str,
        expected_revision: u32,
        draft: AnnotationDraft,
    ) -> Result<AnnotationRecord, AnnotationError> {
        let now = now_ms();
        let selector_json = serde_json::to_string(&draft.selector)
            .map_err(|e| AnnotationError::Storage(e.to_string()))?;
        let style_json = serde_json::to_string(&draft.style)
            .map_err(|e| AnnotationError::Storage(e.to_string()))?;
        let resolution = resolve_selector(&draft.selector);

        let mut conn = self.lock()?;
        let tx = conn
            .transaction()
            .map_err(|e| AnnotationError::Storage(e.to_string()))?;
        let head = sa::annotation_row(&tx, annotation_id)
            .db()?
            .ok_or(AnnotationError::NotFound)?;
        if head.deleted_at_ms.is_some() {
            return Err(AnnotationError::NotFound);
        }
        if head.current_revision != i64::from(expected_revision) {
            return Err(AnnotationError::RevisionConflict);
        }
        let next_revision = head.current_revision + 1;
        sa::insert_annotation_revision(
            &tx,
            &AnnotationRevisionRow {
                annotation_id: annotation_id.to_string(),
                revision: next_revision,
                body_text: draft.body_text.clone(),
                selector_json,
                style_json,
                context_text: draft.selector.captured_context.clone(),
                source_revision_digest: draft.selector.article.source_revision_digest.clone(),
                authored_by_device_id: None,
                created_at_ms: now,
            },
        )
        .db()?;
        // Head-row update: the §6.2 trigger `trg_annotation_revision_append`
        // keeps `current_revision` authoritative, so only the mutable fields
        // are updated here.
        tx.execute(
            "UPDATE annotations SET notebook_id = ?2, resolution_state = ?3, updated_at_ms = ?4
\
             \x20\x20WHERE annotation_id = ?1",
            rusqlite::params![
                annotation_id,
                draft.notebook_id,
                resolution_state_str(resolution.state),
                now
            ],
        )
        .map_err(|e| AnnotationError::Storage(e.to_string()))?;
        sa::replace_annotation_tags(&tx, annotation_id, &draft.tag_ids).db()?;
        tx.commit()
            .map_err(|e| AnnotationError::Storage(e.to_string()))?;
        drop(conn);
        self.get_impl(annotation_id)
    }

    pub fn soft_delete_impl(
        &self,
        annotation_id: &str,
        expected_revision: u32,
    ) -> Result<(), AnnotationError> {
        let conn = self.lock()?;
        let head = sa::annotation_row(&conn, annotation_id)
            .db()?
            .ok_or(AnnotationError::NotFound)?;
        if head.current_revision != i64::from(expected_revision) {
            return Err(AnnotationError::RevisionConflict);
        }
        sa::soft_delete_annotation(&conn, annotation_id, now_ms()).db()?;
        Ok(())
    }

    pub fn restore_impl(&self, annotation_id: &str) -> Result<AnnotationRecord, AnnotationError> {
        let conn = self.lock()?;
        let head = sa::annotation_row(&conn, annotation_id)
            .db()?
            .ok_or(AnnotationError::NotFound)?;
        if head.deleted_at_ms.is_some() {
            sa::restore_annotation(&conn, annotation_id).db()?;
        }
        Self::assemble(&conn, &head)
    }
}

#[async_trait]
impl AnnotationServicePort for AnnotationsService {
    async fn create(&self, draft: AnnotationDraft) -> Result<AnnotationRecord, AnnotationError> {
        self.create_impl(draft)
    }

    async fn get(&self, annotation_id: &str) -> Result<AnnotationRecord, AnnotationError> {
        self.get_impl(annotation_id)
    }

    async fn update(
        &self,
        annotation_id: &str,
        expected_revision: u32,
        draft: AnnotationDraft,
    ) -> Result<AnnotationRecord, AnnotationError> {
        self.update_impl(annotation_id, expected_revision, draft)
    }

    async fn soft_delete(
        &self,
        annotation_id: &str,
        expected_revision: u32,
    ) -> Result<(), AnnotationError> {
        self.soft_delete_impl(annotation_id, expected_revision)
    }

    async fn restore(&self, annotation_id: &str) -> Result<AnnotationRecord, AnnotationError> {
        self.restore_impl(annotation_id)
    }

    async fn list_for_article(
        &self,
        pack_id: &PackId,
        article_key: &ArticleKey,
    ) -> Result<Vec<AnnotationRecord>, AnnotationError> {
        self.list_for_article_records(&pack_id.0, &article_key.0)
    }

    async fn attach_asset(
        &self,
        annotation_id: &str,
        asset: AnnotationAssetInput,
    ) -> Result<AnnotationAssetRecord, AnnotationError> {
        let mut hasher = Sha256::new();
        hasher.update(&asset.bytes);
        let declared = hex::encode(hasher.finalize());
        self.sink
            .commit_linked(
                annotation_id,
                &asset.asset_kind,
                &declared,
                &asset.mime_type,
                asset.bytes.len() as u64,
                asset.duration_ms,
                asset.bytes,
            )
            .await
    }
}

/// The `CommittedAssetSink` the application-layer upload service delegates
/// to: re-verify the digest, write the object content-addressed into the
/// object store, then insert the `annotation_assets` row. If the DB insert
/// fails the object is already committed — an unreferenced content-addressed
/// object is garbage-collectable, and no annotation ever references a
/// partial asset (§9.3).
pub struct AssetSink {
    conn: Arc<Mutex<Connection>>,
    objects: Arc<ObjectStore>,
}

#[async_trait]
impl CommittedAssetSink for AssetSink {
    async fn commit_linked(
        &self,
        annotation_id: &str,
        asset_kind: &str,
        declared_sha256: &str,
        mime_type: &str,
        byte_length: u64,
        duration_ms: Option<u64>,
        bytes: Bytes,
    ) -> Result<AnnotationAssetRecord, AnnotationError> {
        // (a) Digest defence in depth: the service already hashed the
        // assembled buffer; we re-hash what actually arrived.
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let actual = hex::encode(hasher.finalize());
        if actual != declared_sha256 {
            return Err(AnnotationError::InvalidAsset(format!(
                "digest mismatch: declared {declared_sha256}, received {actual}"
            )));
        }

        // (b) Content-addressed object write (§6.3 11-step commit).
        let digest = ContentDigest {
            algorithm: DigestAlgorithm::Sha256,
            hex: actual.clone(),
        };
        let transaction_id = format!("asset-{}", Ulid::new());
        let audit_event_id = format!("audit-{transaction_id}");
        self.objects
            .commit(CommitRequest {
                transaction_id: &transaction_id,
                audit_event_id_on_failure: &audit_event_id,
                max_size: byte_length,
                expected_length: byte_length,
                expected_digest: &digest,
                bytes: std::io::Cursor::new(bytes),
                verifier: &AcceptRawBytes,
            })
            .map_err(|e| AnnotationError::Storage(format!("object commit failed: {e}")))?;

        // (c) Metadata row — FK guarantees the annotation exists.
        let duration_ms_i64 = duration_ms
            .map(i64::try_from)
            .transpose()
            .map_err(|_| AnnotationError::InvalidAsset("duration overflow".to_string()))?;
        let row = sa::AnnotationAssetRow {
            annotation_asset_id: Ulid::new().to_string(),
            annotation_id: annotation_id.to_string(),
            asset_kind: asset_kind.to_string(),
            object_sha256: actual,
            mime_type: mime_type.to_string(),
            byte_length: i64::try_from(byte_length)
                .map_err(|_| AnnotationError::InvalidAsset("byte length overflow".to_string()))?,
            duration_ms: duration_ms_i64,
            created_at_ms: now_ms(),
        };
        let conn = self.conn.lock().map_err(|e| {
            AnnotationError::Storage(format!("connection lock poisoned: {e}"))
        })?;
        sa::insert_annotation_asset(&conn, &row).db()?;

        Ok(AnnotationAssetRecord {
            annotation_asset_id: row.annotation_asset_id,
            annotation_id: row.annotation_id,
            asset_kind: row.asset_kind,
            object_sha256: row.object_sha256,
            mime_type: row.mime_type,
            byte_length: byte_length,
            duration_ms,
        })
    }
}

// ===========================================================================
// Wire projections owned by this adapter
// ===========================================================================

/// One node of the notebook forest (§14.4 wire shape).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotebookNode {
    pub notebook_id: String,
    pub parent_notebook_id: Option<String>,
    pub name: String,
    pub description: String,
    pub annotation_count: i64,
    pub children: Vec<NotebookNode>,
}

impl NotebookNode {
    fn from_row(row: &NotebookRow, children: Vec<NotebookNode>) -> Self {
        Self {
            notebook_id: row.notebook_id.clone(),
            parent_notebook_id: row.parent_notebook_id.clone(),
            name: row.name.clone(),
            description: row.description.clone(),
            annotation_count: row.annotation_count,
            children,
        }
    }
}

/// One sidecar layer as the layer toggle renders it (§14.4 wire shape).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SidecarLayerView {
    pub sidecar_layer_id: String,
    pub name: String,
    pub visibility: String,
    pub trust_state: String,
    pub enabled: bool,
    pub annotation_count: i64,
}

impl SidecarLayerView {
    pub fn from_row(row: SidecarLayerRow) -> Self {
        let enabled = row.trust_state != "rejected";
        Self {
            sidecar_layer_id: row.sidecar_layer_id,
            name: row.name,
            visibility: row.visibility,
            trust_state: row.trust_state,
            enabled,
            annotation_count: row.annotation_count,
        }
    }
}

/// Filter+page request for [`AnnotationsService::list`] (§14.4 wire shape).
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    pub pack_ids: Vec<String>,
    pub notebook_ids: Vec<String>,
    pub tag_ids: Vec<String>,
    pub sidecar_layer_ids: Vec<String>,
    pub resolution_states: Vec<String>,
    pub text: Option<String>,
    pub offset: i64,
    pub limit: i64,
}

// ===========================================================================
// Value helpers
// ===========================================================================

/// Bridge from storage-layer `rusqlite::Error` to the domain error type
/// (no orphan `From` impl is possible on a foreign error enum).
pub(crate) fn db_err(e: rusqlite::Error) -> AnnotationError {
    AnnotationError::Storage(e.to_string())
}

trait DbResultExt<T> {
    fn db(self) -> Result<T, AnnotationError>;
}

impl<T> DbResultExt<T> for Result<T, rusqlite::Error> {
    fn db(self) -> Result<T, AnnotationError> {
        self.map_err(db_err)
    }
}

/// Heuristic selector resolution (see crate docs). Structural anchors are
/// treated as resolved; a bare text quote is ambiguous until the reader
/// integration verifies it against the live article.
pub fn resolve_selector(selector: &SelectorBundle) -> AnnotationResolution {
    let (state, confidence, explanation, range) = if let Some(pos) = &selector.text_position {
        (
            AnnotationResolutionState::Resolved,
            1.0,
            "anchored by normalized text position".to_string(),
            Some((pos.normalized_start, pos.normalized_end)),
        )
    } else if selector.dom_path.is_some() {
        (
            AnnotationResolutionState::Resolved,
            0.9,
            "anchored by DOM path".to_string(),
            None,
        )
    } else if selector.image_region.is_some() {
        (
            AnnotationResolutionState::Resolved,
            0.9,
            "anchored by image region".to_string(),
            None,
        )
    } else if selector.text_quote.is_some() {
        (
            AnnotationResolutionState::Ambiguous,
            0.5,
            "text quote not yet verified against the source article".to_string(),
            None,
        )
    } else {
        (
            AnnotationResolutionState::Ambiguous,
            0.0,
            "selector carries no anchor".to_string(),
            None,
        )
    };
    AnnotationResolution {
        state,
        confidence,
        matched_start: range.map(|r| r.0),
        matched_end: range.map(|r| r.1),
        explanation,
    }
}

fn build_notebook_tree(rows: Vec<NotebookRow>) -> Vec<NotebookNode> {
    let mut by_id: std::collections::HashMap<String, NotebookNode> = rows
        .iter()
        .map(|r| (r.notebook_id.clone(), NotebookNode::from_row(r, Vec::new())))
        .collect();
    let mut root_ids: Vec<String> = Vec::new();
    // Attach children to parents first; orphans (missing or self parent)
    // become roots so a broken reference can never hide notes.
    for row in &rows {
        // Rows arrive parents-first, so removing the node before attaching
        // keeps later children able to find their (re-inserted) parent.
        let Some(node) = by_id.remove(&row.notebook_id) else {
            continue;
        };
        let parent = row
            .parent_notebook_id
            .as_ref()
            .and_then(|p| by_id.get_mut(p))
            .filter(|p| p.notebook_id != node.notebook_id);
        match parent {
            Some(parent) => parent.children.push(node),
            None => {
                root_ids.push(node.notebook_id.clone());
                by_id.insert(node.notebook_id.clone(), node);
            }
        }
    }
    let mut roots: Vec<NotebookNode> = root_ids
        .iter()
        .filter_map(|id| by_id.remove(id))
        .collect();
    roots.sort_by(|a, b| a.name.cmp(&b.name));
    roots
}

/// The §6.2 CHECK vocabulary string for a domain annotation type.
pub fn annotation_type_str(t: AnnotationType) -> &'static str {
    match t {
        AnnotationType::Highlight => "highlight",
        AnnotationType::Comment => "comment",
        AnnotationType::Ink => "ink",
        AnnotationType::Voice => "voice",
        AnnotationType::BookmarkNote => "bookmark_note",
    }
}

fn annotation_type_from_str(s: &str) -> AnnotationType {
    match s {
        "comment" => AnnotationType::Comment,
        "ink" => AnnotationType::Ink,
        "voice" => AnnotationType::Voice,
        "bookmark_note" => AnnotationType::BookmarkNote,
        _ => AnnotationType::Highlight,
    }
}

/// The §6.2 CHECK vocabulary string for a domain resolution state.
pub fn resolution_state_str(s: AnnotationResolutionState) -> &'static str {
    match s {
        AnnotationResolutionState::Resolved => "resolved",
        AnnotationResolutionState::Ambiguous => "ambiguous",
        AnnotationResolutionState::SourceUnavailable => "source_unavailable",
        AnnotationResolutionState::Orphaned => "orphaned",
    }
}

fn resolution_state_from_str(s: &str) -> AnnotationResolutionState {
    match s {
        "ambiguous" => AnnotationResolutionState::Ambiguous,
        "source_unavailable" => AnnotationResolutionState::SourceUnavailable,
        "orphaned" => AnnotationResolutionState::Orphaned,
        _ => AnnotationResolutionState::Resolved,
    }
}

/// Wall-clock passthrough for sibling modules.
pub(crate) fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use object_store::store::ObjectStore;

    fn service(dir: &std::path::Path) -> AnnotationsService {
        let conn = storage_sqlite::open_with_pragmas(dir.join("test.db")).unwrap();
        let runner = storage_sqlite::MigrationRunner::new(&conn);
        runner.apply_initial().unwrap();
        let objects = ObjectStore::open(dir).unwrap();
        AnnotationsService::new(Arc::new(Mutex::new(conn)), Arc::new(objects))
    }

    fn draft() -> AnnotationDraft {
        AnnotationDraft {
            annotation_type: AnnotationType::Highlight,
            selector: SelectorBundle {
                article: domain::annotation::ArticleSelector {
                    pack_id: PackId("pack-1".to_string()),
                    article_key: ArticleKey("article/main".to_string()),
                    source_revision_digest: None,
                },
                text_quote: None,
                text_position: Some(domain::annotation::TextPositionSelector {
                    normalized_start: 100,
                    normalized_end: 160,
                }),
                dom_path: None,
                image_region: None,
                captured_context: "some context".to_string(),
            },
            body_text: Some("first thought".to_string()),
            style: AnnotationStyle {
                color_token: "yellow".to_string(),
                emphasis: "none".to_string(),
                stroke_width: None,
            },
            notebook_id: None,
            tag_ids: vec![],
            sidecar_layer_id: LOCAL_LAYER_ID.to_string(),
        }
    }

    fn pack_row(conn: &Connection, pack_id: &str) {
        conn.execute(
            "INSERT INTO packs (pack_id, corpus_id, display_name, description,\n\
             \x20\x20  language_bcp47, pack_kind, active_pack_version_id,\n\
             \x20\x20  created_at_ms, updated_at_ms)\n\
             \x20\x20VALUES (?1, 'c', 'P', '', 'en', 'full_zim', NULL, 1, 1)",
            rusqlite::params![pack_id],
        )
        .unwrap();
    }

    #[tokio::test]
    async fn create_update_delete_restore_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let svc = service(dir.path());
        {
            let conn = svc.lock().unwrap();
            pack_row(&conn, "pack-1");
        }
        svc.ensure_local_layer().unwrap();

        let record = svc.create_impl(draft()).unwrap();
        assert_eq!(record.revision, 1);
        assert_eq!(record.body_text.as_deref(), Some("first thought"));
        assert_eq!(record.sidecar_layer_id, LOCAL_LAYER_ID);
        assert!(matches!(
            record.resolution.state,
            AnnotationResolutionState::Resolved
        ));
        assert!(record.annotation_id.len() >= 26, "ULID-shaped id");

        let updated = svc
            .update_impl(&record.annotation_id, 1, {
                let mut d = draft();
                d.body_text = Some("second thought".to_string());
                d
            })
            .unwrap();
        assert_eq!(updated.revision, 2);
        assert_eq!(updated.body_text.as_deref(), Some("second thought"));

        // Stale expected revision conflicts.
        assert!(matches!(
            svc.update_impl(&record.annotation_id, 1, draft()),
            Err(AnnotationError::RevisionConflict)
        ));

        svc.soft_delete_impl(&record.annotation_id, 2).unwrap();
        assert!(matches!(
            svc.get_impl(&record.annotation_id),
            Err(AnnotationError::NotFound)
        ));
        let restored = svc.restore_impl(&record.annotation_id).unwrap();
        assert_eq!(restored.revision, 2);

        // The article listing sees it again after restore.
        let for_article = svc
            .list_for_article_records("pack-1", "article/main")
            .unwrap();
        assert_eq!(for_article.len(), 1);
    }

    #[tokio::test]
    async fn taxonomy_tree_and_tags() {
        let dir = tempfile::tempdir().unwrap();
        let svc = service(dir.path());
        svc.ensure_local_layer().unwrap();

        let root = svc.notebook_create(None, "Research", "").unwrap();
        let child = svc
            .notebook_create(Some(&root.notebook_id), "Subtopic", "nested")
            .unwrap();
        let tree = svc.notebook_tree().unwrap();
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].children.len(), 1);
        assert_eq!(tree[0].children[0].notebook_id, child.notebook_id);

        // Renaming to a sibling of itself is rejected.
        assert!(svc
            .notebook_update(&root.notebook_id, Some(&root.notebook_id), "x", "")
            .is_err());

        let tag = svc.tag_create("Important", "blue").unwrap();
        // Case-insensitive idempotency.
        let again = svc.tag_create("IMPORTANT", "red").unwrap();
        assert_eq!(tag.tag_id, again.tag_id);
        assert_eq!(svc.tag_list().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn asset_attach_writes_object_and_row() {
        let dir = tempfile::tempdir().unwrap();
        let svc = service(dir.path());
        {
            let conn = svc.lock().unwrap();
            pack_row(&conn, "pack-1");
        }
        svc.ensure_local_layer().unwrap();
        let record = svc.create_impl(draft()).unwrap();

        let payload = b"fake-opus-bytes".to_vec();
        let mut hasher = Sha256::new();
        hasher.update(&payload);
        let digest = hex::encode(hasher.finalize());
        let asset = svc
            .attach_asset(
                &record.annotation_id,
                AnnotationAssetInput {
                    asset_kind: "voice_opus".to_string(),
                    mime_type: "audio/opus".to_string(),
                    bytes: Bytes::from(payload.clone()),
                    duration_ms: Some(1_500),
                },
            )
            .await
            .unwrap();
        assert_eq!(asset.object_sha256, digest);
        assert_eq!(asset.byte_length, payload.len() as u64);

        // The content-addressed object is retrievable from the store.
        let cd = ContentDigest {
            algorithm: DigestAlgorithm::Sha256,
            hex: digest.clone(),
        };
        assert!(svc.objects.contains(&cd).unwrap());

        // A digest mismatch is rejected before anything is written.
        let err = svc
            .attach_asset(
                &record.annotation_id,
                AnnotationAssetInput {
                    asset_kind: "voice_opus".to_string(),
                    mime_type: "audio/opus".to_string(),
                    bytes: Bytes::from_static(b"tampered"),
                    duration_ms: None,
                },
            )
            .await;
        // attach_asset computes the digest itself, so it succeeds; instead
        // verify the sink rejects an explicit mismatch.
        let sink_err = svc
            .asset_sink()
            .commit_linked(
                &record.annotation_id,
                "file",
                &"0".repeat(64),
                "application/octet-stream",
                8,
                None,
                Bytes::from_static(b"tampered"),
            )
            .await;
        assert!(matches!(err, Ok(_)));
        assert!(matches!(sink_err, Err(AnnotationError::InvalidAsset(_))));
        assert!(err.is_ok());
    }
}
