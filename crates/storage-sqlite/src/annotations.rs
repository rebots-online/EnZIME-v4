//! Annotation-domain reads and writes over the §6.2 tables.
//!
//! Covers `notebooks`, `tags`, `sidecar_layers`, `annotations`,
//! `annotation_revisions`, `annotation_assets`, `annotation_tags`, and
//! `sidecar_imports`. Conventions mirror [`crate::query`] and
//! [`crate::write`] exactly: plain row structs, borrowed `&Connection`,
//! `Result<_, rusqlite::Error>`, no serde and no clock calls — ids and
//! timestamps are caller-supplied parameters (A-02).
//!
//! Multi-statement invariants (annotation + revision + tag links, or the
//! sidecar import + layer + annotation set) are composed by the service
//! layer inside a transaction; every function here is a single bounded
//! statement. The CHECK constraints in §6.2 remain the final authority —
//! an illegal `annotation_type`, `visibility`, or revision surfaces as a
//! `rusqlite::Error`.

#![allow(clippy::needless_pass_by_value)]

use rusqlite::{params, Connection, OptionalExtension};

// ===========================================================================
// Row types
// ===========================================================================

/// One row of `notebooks` (§6.2) with its annotation count.
#[derive(Clone, Debug)]
pub struct NotebookRow {
    pub notebook_id: String,
    pub parent_notebook_id: Option<String>,
    pub name: String,
    pub description: String,
    pub sort_order: i64,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    pub annotation_count: i64,
}

/// One row of `tags` (§6.2).
#[derive(Clone, Debug)]
pub struct TagRow {
    pub tag_id: String,
    pub name: String,
    pub color_token: String,
    pub created_at_ms: i64,
}

/// One row of `sidecar_layers` (§6.2) with its annotation count.
#[derive(Clone, Debug)]
pub struct SidecarLayerRow {
    pub sidecar_layer_id: String,
    pub name: String,
    pub owner_device_id: Option<String>,
    pub visibility: String,
    pub trust_state: String,
    pub source_artifact_id: Option<String>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    pub annotation_count: i64,
}

/// One row of `annotations` (§6.2) — the identity/revision head, without
/// revision content. Content lives in [`AnnotationRevisionRow`].
#[derive(Clone, Debug)]
pub struct AnnotationRow {
    pub annotation_id: String,
    pub annotation_type: String,
    pub pack_id: String,
    pub article_key: String,
    pub notebook_id: Option<String>,
    pub sidecar_layer_id: String,
    pub current_revision: i64,
    pub resolution_state: String,
    pub pinned: bool,
    pub deleted_at_ms: Option<i64>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

/// One row of `annotation_revisions` (§6.2) — the content of one revision.
/// `selector_json` and `style_json` are serialized selector/style documents.
#[derive(Clone, Debug)]
pub struct AnnotationRevisionRow {
    pub annotation_id: String,
    pub revision: i64,
    pub body_text: Option<String>,
    pub selector_json: String,
    pub style_json: String,
    pub context_text: String,
    pub source_revision_digest: Option<String>,
    pub authored_by_device_id: Option<String>,
    pub created_at_ms: i64,
}

/// One row of `annotation_assets` (§6.2).
#[derive(Clone, Debug)]
pub struct AnnotationAssetRow {
    pub annotation_asset_id: String,
    pub annotation_id: String,
    pub asset_kind: String,
    pub object_sha256: String,
    pub mime_type: String,
    pub byte_length: i64,
    pub duration_ms: Option<i64>,
    pub created_at_ms: i64,
}

/// One row of `sidecar_imports` (§6.2).
#[derive(Clone, Debug)]
pub struct SidecarImportRow {
    pub sidecar_import_id: String,
    pub artifact_sha256: String,
    pub signature_state: String,
    pub imported_layer_id: Option<String>,
    pub record_count: i64,
    pub conflict_count: i64,
    pub imported_at_ms: i64,
}

// ===========================================================================
// §6.2 `sidecar_layers`
// ===========================================================================

/// Insert one `sidecar_layers` row. `visibility` must be one of
/// `private`, `shared`, `imported`; `trust_state` one of `local`,
/// `verified`, `unverified`, `rejected` (column CHECKs).
pub fn insert_sidecar_layer(
    conn: &Connection,
    row: &SidecarLayerRow,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO sidecar_layers (\n\
         \x20\x20        sidecar_layer_id, name, owner_device_id, visibility,\n\
         \x20\x20        trust_state, source_artifact_id, created_at_ms, updated_at_ms\n\
         \x20\x20      ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            row.sidecar_layer_id,
            row.name,
            row.owner_device_id,
            row.visibility,
            row.trust_state,
            row.source_artifact_id,
            row.created_at_ms,
            row.updated_at_ms,
        ],
    )?;
    Ok(())
}

/// Return the `sidecar_layers` row with the given id, if present.
pub fn sidecar_layer(
    conn: &Connection,
    sidecar_layer_id: &str,
) -> Result<Option<SidecarLayerRow>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT l.sidecar_layer_id, l.name, l.owner_device_id, l.visibility,\n\
         \x20\x20       l.trust_state, l.source_artifact_id, l.created_at_ms, l.updated_at_ms,\n\
         \x20\x20       (SELECT COUNT(*) FROM annotations a\n\
         \x20\x20         WHERE a.sidecar_layer_id = l.sidecar_layer_id\n\
         \x20\x20           AND a.deleted_at_ms IS NULL)\n\
         \x20\x20FROM sidecar_layers l WHERE l.sidecar_layer_id = ?1",
    )?;
    stmt.query_row(params![sidecar_layer_id], map_sidecar_layer_row)
        .optional()
}

/// Return every `sidecar_layers` row, local-first then oldest-first, each
/// with its live-annotation count.
pub fn list_sidecar_layers(conn: &Connection) -> Result<Vec<SidecarLayerRow>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT l.sidecar_layer_id, l.name, l.owner_device_id, l.visibility,\n\
         \x20\x20       l.trust_state, l.source_artifact_id, l.created_at_ms, l.updated_at_ms,\n\
         \x20\x20       (SELECT COUNT(*) FROM annotations a\n\
         \x20\x20         WHERE a.sidecar_layer_id = l.sidecar_layer_id\n\
         \x20\x20           AND a.deleted_at_ms IS NULL)\n\
         \x20\x20FROM sidecar_layers l\n\
         \x20\x20ORDER BY (l.visibility = 'private') DESC, l.created_at_ms ASC",
    )?;
    let rows = stmt.query_map([], map_sidecar_layer_row)?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

fn map_sidecar_layer_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<SidecarLayerRow> {
    Ok(SidecarLayerRow {
        sidecar_layer_id: row.get(0)?,
        name: row.get(1)?,
        owner_device_id: row.get(2)?,
        visibility: row.get(3)?,
        trust_state: row.get(4)?,
        source_artifact_id: row.get(5)?,
        created_at_ms: row.get(6)?,
        updated_at_ms: row.get(7)?,
        annotation_count: row.get(8)?,
    })
}

// ===========================================================================
// §6.2 `notebooks`
// ===========================================================================

/// Insert one `notebooks` row. `parent_notebook_id` must reference an
/// existing notebook when set (FK); the `UNIQUE(parent_notebook_id, name)`
/// constraint rejects sibling name collisions.
pub fn insert_notebook(conn: &Connection, row: &NotebookRow) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO notebooks (\n\
         \x20\x20        notebook_id, parent_notebook_id, name, description,\n\
         \x20\x20        sort_order, created_at_ms, updated_at_ms\n\
         \x20\x20      ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            row.notebook_id,
            row.parent_notebook_id,
            row.name,
            row.description,
            row.sort_order,
            row.created_at_ms,
            row.updated_at_ms,
        ],
    )?;
    Ok(())
}

/// Update an existing notebook's name, description, and parent.
pub fn update_notebook(
    conn: &Connection,
    notebook_id: &str,
    parent_notebook_id: Option<&str>,
    name: &str,
    description: &str,
    updated_at_ms: i64,
) -> Result<usize, rusqlite::Error> {
    conn.execute(
        "UPDATE notebooks\n\
         \x20\x20SET parent_notebook_id = ?2, name = ?3, description = ?4, updated_at_ms = ?5\n\
         \x20\x20WHERE notebook_id = ?1",
        params![notebook_id, parent_notebook_id, name, description, updated_at_ms],
    )
}

/// Return every notebook, ordered parent-then-name for tree building.
pub fn list_notebooks(conn: &Connection) -> Result<Vec<NotebookRow>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT n.notebook_id, n.parent_notebook_id, n.name, n.description,\n\
         \x20\x20       n.sort_order, n.created_at_ms, n.updated_at_ms,\n\
         \x20\x20       (SELECT COUNT(*) FROM annotations a\n\
         \x20\x20         WHERE a.notebook_id = n.notebook_id\n\
         \x20\x20           AND a.deleted_at_ms IS NULL)\n\
         \x20\x20FROM notebooks n\n\
         \x20\x20ORDER BY n.parent_notebook_id IS NULL DESC, n.sort_order ASC, n.name ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(NotebookRow {
            notebook_id: row.get(0)?,
            parent_notebook_id: row.get(1)?,
            name: row.get(2)?,
            description: row.get(3)?,
            sort_order: row.get(4)?,
            created_at_ms: row.get(5)?,
            updated_at_ms: row.get(6)?,
            annotation_count: row.get(7)?,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

/// Whether a notebook with the given id exists.
pub fn notebook_exists(conn: &Connection, notebook_id: &str) -> Result<bool, rusqlite::Error> {
    Ok(conn
        .query_row(
            "SELECT 1 FROM notebooks WHERE notebook_id = ?1",
            params![notebook_id],
            |_| Ok(()),
        )
        .optional()?
        .is_some())
}

// ===========================================================================
// §6.2 `tags`
// ===========================================================================

/// Insert one `tags` row. `name` is `COLLATE NOCASE UNIQUE` — a duplicate
/// name (any casing) surfaces as a constraint error.
pub fn insert_tag(conn: &Connection, row: &TagRow) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO tags (tag_id, name, color_token, created_at_ms)\n\
         \x20\x20VALUES (?1, ?2, ?3, ?4)",
        params![row.tag_id, row.name, row.color_token, row.created_at_ms],
    )?;
    Ok(())
}

/// Return every tag, name-ordered (the column collation is NOCASE).
pub fn list_tags(conn: &Connection) -> Result<Vec<TagRow>, rusqlite::Error> {
    let mut stmt = conn
        .prepare("SELECT tag_id, name, color_token, created_at_ms FROM tags ORDER BY name ASC")?;
    let rows = stmt.query_map([], |row| {
        Ok(TagRow {
            tag_id: row.get(0)?,
            name: row.get(1)?,
            color_token: row.get(2)?,
            created_at_ms: row.get(3)?,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

/// Return the tag whose name matches `name` case-insensitively, if any.
pub fn tag_by_name(conn: &Connection, name: &str) -> Result<Option<TagRow>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT tag_id, name, color_token, created_at_ms FROM tags WHERE name = ?1",
    )?;
    stmt.query_row(params![name], |row| {
        Ok(TagRow {
            tag_id: row.get(0)?,
            name: row.get(1)?,
            color_token: row.get(2)?,
            created_at_ms: row.get(3)?,
        })
    })
    .optional()
}

// ===========================================================================
// §6.2 `annotations` + `annotation_revisions` + `annotation_tags`
// ===========================================================================

/// Insert one `annotations` head row.
pub fn insert_annotation(conn: &Connection, row: &AnnotationRow) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO annotations (\n\
         \x20\x20        annotation_id, annotation_type, pack_id, article_key,\n\
         \x20\x20        notebook_id, sidecar_layer_id, current_revision,\n\
         \x20\x20        resolution_state, pinned, deleted_at_ms, created_at_ms, updated_at_ms\n\
         \x20\x20      ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            row.annotation_id,
            row.annotation_type,
            row.pack_id,
            row.article_key,
            row.notebook_id,
            row.sidecar_layer_id,
            row.current_revision,
            row.resolution_state,
            i64::from(row.pinned),
            row.deleted_at_ms,
            row.created_at_ms,
            row.updated_at_ms,
        ],
    )?;
    Ok(())
}

/// Insert one `annotation_revisions` content row.
pub fn insert_annotation_revision(
    conn: &Connection,
    row: &AnnotationRevisionRow,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO annotation_revisions (\n\
         \x20\x20        annotation_id, revision, body_text, selector_json, style_json,\n\
         \x20\x20        context_text, source_revision_digest, authored_by_device_id,\n\
         \x20\x20        created_at_ms\n\
         \x20\x20      ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            row.annotation_id,
            row.revision,
            row.body_text,
            row.selector_json,
            row.style_json,
            row.context_text,
            row.source_revision_digest,
            row.authored_by_device_id,
            row.created_at_ms,
        ],
    )?;
    Ok(())
}

/// Link one tag to one annotation (`annotation_tags`).
pub fn link_annotation_tag(
    conn: &Connection,
    annotation_id: &str,
    tag_id: &str,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT OR IGNORE INTO annotation_tags (annotation_id, tag_id) VALUES (?1, ?2)",
        params![annotation_id, tag_id],
    )?;
    Ok(())
}

/// Replace the tag link set for one annotation (delete-then-link; the
/// caller wraps in a transaction with the head-row update).
pub fn replace_annotation_tags(
    conn: &Connection,
    annotation_id: &str,
    tag_ids: &[String],
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "DELETE FROM annotation_tags WHERE annotation_id = ?1",
        params![annotation_id],
    )?;
    for tag_id in tag_ids {
        conn.execute(
            "INSERT OR IGNORE INTO annotation_tags (annotation_id, tag_id) VALUES (?1, ?2)",
            params![annotation_id, tag_id],
        )?;
    }
    Ok(())
}

/// Return the `annotations` head row with the given id, if present
/// (soft-deleted rows included — callers filter).
pub fn annotation_row(
    conn: &Connection,
    annotation_id: &str,
) -> Result<Option<AnnotationRow>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT annotation_id, annotation_type, pack_id, article_key, notebook_id,\n\
         \x20\x20       sidecar_layer_id, current_revision, resolution_state, pinned,\n\
         \x20\x20       deleted_at_ms, created_at_ms, updated_at_ms\n\
         \x20\x20FROM annotations WHERE annotation_id = ?1",
    )?;
    stmt.query_row(params![annotation_id], map_annotation_row)
        .optional()
}

fn map_annotation_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<AnnotationRow> {
    Ok(AnnotationRow {
        annotation_id: row.get(0)?,
        annotation_type: row.get(1)?,
        pack_id: row.get(2)?,
        article_key: row.get(3)?,
        notebook_id: row.get(4)?,
        sidecar_layer_id: row.get(5)?,
        current_revision: row.get(6)?,
        resolution_state: row.get(7)?,
        pinned: row.get::<_, i64>(8)? != 0,
        deleted_at_ms: row.get(9)?,
        created_at_ms: row.get(10)?,
        updated_at_ms: row.get(11)?,
    })
}

/// Return the latest revision content for one annotation, if present.
pub fn latest_revision(
    conn: &Connection,
    annotation_id: &str,
) -> Result<Option<AnnotationRevisionRow>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT annotation_id, revision, body_text, selector_json, style_json,\n\
         \x20\x20       context_text, source_revision_digest, authored_by_device_id, created_at_ms\n\
         \x20\x20FROM annotation_revisions\n\
         \x20\x20WHERE annotation_id = ?1\n\
         \x20\x20ORDER BY revision DESC LIMIT 1",
    )?;
    stmt.query_row(params![annotation_id], |row| {
        Ok(AnnotationRevisionRow {
            annotation_id: row.get(0)?,
            revision: row.get(1)?,
            body_text: row.get(2)?,
            selector_json: row.get(3)?,
            style_json: row.get(4)?,
            context_text: row.get(5)?,
            source_revision_digest: row.get(6)?,
            authored_by_device_id: row.get(7)?,
            created_at_ms: row.get(8)?,
        })
    })
    .optional()
}

/// Return the tag set linked to one annotation.
pub fn annotation_tags(
    conn: &Connection,
    annotation_id: &str,
) -> Result<Vec<TagRow>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT t.tag_id, t.name, t.color_token, t.created_at_ms\n\
         \x20\x20FROM annotation_tags at\n\
         \x20\x20JOIN tags t ON t.tag_id = at.tag_id\n\
         \x20\x20WHERE at.annotation_id = ?1\n\
         \x20\x20ORDER BY t.name ASC",
    )?;
    let rows = stmt.query_map(params![annotation_id], |row| {
        Ok(TagRow {
            tag_id: row.get(0)?,
            name: row.get(1)?,
            color_token: row.get(2)?,
            created_at_ms: row.get(3)?,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

/// Soft-delete: stamp `deleted_at_ms` on one annotation. Returns rows
/// affected (0 when the id does not exist).
pub fn soft_delete_annotation(
    conn: &Connection,
    annotation_id: &str,
    deleted_at_ms: i64,
) -> Result<usize, rusqlite::Error> {
    conn.execute(
        "UPDATE annotations SET deleted_at_ms = ?2 WHERE annotation_id = ?1",
        params![annotation_id, deleted_at_ms],
    )
}

/// Restore a soft-deleted annotation by clearing `deleted_at_ms`.
pub fn restore_annotation(
    conn: &Connection,
    annotation_id: &str,
) -> Result<usize, rusqlite::Error> {
    conn.execute(
        "UPDATE annotations SET deleted_at_ms = NULL WHERE annotation_id = ?1",
        params![annotation_id],
    )
}

/// Filter for [`list_annotations`]. Every set is an IN-filter; `text` is a
/// substring match against revision `body_text` and `context_text`
/// (LIKE-wildcards in user input are escaped, not interpreted).
#[derive(Clone, Debug, Default)]
pub struct AnnotationFilter {
    pub pack_ids: Vec<String>,
    pub notebook_ids: Vec<String>,
    pub tag_ids: Vec<String>,
    pub sidecar_layer_ids: Vec<String>,
    pub resolution_states: Vec<String>,
    pub article_key: Option<String>,
    pub text: Option<String>,
    /// When false (the default), soft-deleted rows are excluded.
    pub include_deleted: bool,
}

/// Escape SQL LIKE wildcards in user input; pair with `ESCAPE '\'` in the
/// query so `%`/`_` typed by a user match literally.
pub fn escape_like(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        if matches!(c, '%' | '_' | '\\') {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// List annotations matching `filter`, newest-`updated_at_ms` first, with
/// the total match count for pagination. Returns `(rows, total)`.
pub fn list_annotations(
    conn: &Connection,
    filter: &AnnotationFilter,
    offset: i64,
    limit: i64,
) -> Result<(Vec<AnnotationRow>, i64), rusqlite::Error> {
    let mut where_clauses: Vec<String> = Vec::new();
    let mut bind_values: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    if !filter.include_deleted {
        where_clauses.push("a.deleted_at_ms IS NULL".to_string());
    }
    append_in_filter(&mut where_clauses, &mut bind_values, "a.pack_id", &filter.pack_ids);
    append_in_filter(
        &mut where_clauses,
        &mut bind_values,
        "a.notebook_id",
        &filter.notebook_ids,
    );
    append_in_filter(
        &mut where_clauses,
        &mut bind_values,
        "a.sidecar_layer_id",
        &filter.sidecar_layer_ids,
    );
    append_in_filter(
        &mut where_clauses,
        &mut bind_values,
        "a.resolution_state",
        &filter.resolution_states,
    );
    if let Some(article_key) = &filter.article_key {
        where_clauses.push("a.article_key = ?".to_string());
        bind_values.push(Box::new(article_key.clone()));
    }
    if !filter.tag_ids.is_empty() {
        let placeholders = vec!["?"; filter.tag_ids.len()].join(", ");
        where_clauses.push(format!(
            "a.annotation_id IN (SELECT annotation_id FROM annotation_tags WHERE tag_id IN ({placeholders}))"
        ));
        for id in &filter.tag_ids {
            bind_values.push(Box::new(id.clone()));
        }
    }
    if let Some(text) = &filter.text {
        where_clauses.push(format!(
            "a.annotation_id IN (SELECT r.annotation_id FROM annotation_revisions r\n\
             \x20\x20  WHERE r.body_text LIKE ('%' || ? || '%') ESCAPE '\\'\n\
             \x20\x20     OR r.context_text LIKE ('%' || ? || '%') ESCAPE '\\')"
        ));
        let escaped = escape_like(text);
        bind_values.push(Box::new(escaped.clone()));
        bind_values.push(Box::new(escaped));
    }

    let where_sql = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    let count_sql =
        format!("SELECT COUNT(*) FROM annotations a {where_sql}");
    let total: i64 = {
        let mut stmt = conn.prepare(&count_sql)?;
        stmt.query_row(
            rusqlite::params_from_iter(bind_values.iter().map(|b| b.as_ref())),
            |row| row.get(0),
        )?
    };

    let list_sql = format!(
        "SELECT a.annotation_id, a.annotation_type, a.pack_id, a.article_key,\n\
         \x20\x20       a.notebook_id, a.sidecar_layer_id, a.current_revision,\n\
         \x20\x20       a.resolution_state, a.pinned, a.deleted_at_ms, a.created_at_ms,\n\
         \x20\x20       a.updated_at_ms\n\
         \x20\x20FROM annotations a {where_sql}\n\
         \x20\x20ORDER BY a.updated_at_ms DESC\n\
         \x20\x20LIMIT {} OFFSET {}",
        limit.max(0),
        offset.max(0)
    );
    let mut stmt = conn.prepare(&list_sql)?;
    let rows = stmt.query_map(
        rusqlite::params_from_iter(bind_values.iter().map(|b| b.as_ref())),
        map_annotation_row,
    )?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok((out, total))
}

fn append_in_filter(
    where_clauses: &mut Vec<String>,
    bind_values: &mut Vec<Box<dyn rusqlite::ToSql>>,
    column: &str,
    values: &[String],
) {
    if values.is_empty() {
        return;
    }
    let placeholders = vec!["?"; values.len()].join(", ");
    where_clauses.push(format!("{column} IN ({placeholders})"));
    for value in values {
        bind_values.push(Box::new(value.clone()));
    }
}

// ===========================================================================
// §6.2 `annotation_assets`
// ===========================================================================

/// Insert one `annotation_assets` row.
pub fn insert_annotation_asset(
    conn: &Connection,
    row: &AnnotationAssetRow,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO annotation_assets (\n\
         \x20\x20        annotation_asset_id, annotation_id, asset_kind, object_sha256,\n\
         \x20\x20        mime_type, byte_length, duration_ms, created_at_ms\n\
         \x20\x20      ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            row.annotation_asset_id,
            row.annotation_id,
            row.asset_kind,
            row.object_sha256,
            row.mime_type,
            row.byte_length,
            row.duration_ms,
            row.created_at_ms,
        ],
    )?;
    Ok(())
}

/// Return every asset linked to one annotation, oldest-first.
pub fn annotation_assets(
    conn: &Connection,
    annotation_id: &str,
) -> Result<Vec<AnnotationAssetRow>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT annotation_asset_id, annotation_id, asset_kind, object_sha256,\n\
         \x20\x20       mime_type, byte_length, duration_ms, created_at_ms\n\
         \x20\x20FROM annotation_assets WHERE annotation_id = ?1\n\
         \x20\x20ORDER BY created_at_ms ASC",
    )?;
    let rows = stmt.query_map(params![annotation_id], |row| {
        Ok(AnnotationAssetRow {
            annotation_asset_id: row.get(0)?,
            annotation_id: row.get(1)?,
            asset_kind: row.get(2)?,
            object_sha256: row.get(3)?,
            mime_type: row.get(4)?,
            byte_length: row.get(5)?,
            duration_ms: row.get(6)?,
            created_at_ms: row.get(7)?,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

// ===========================================================================
// §6.2 `sidecar_imports`
// ===========================================================================

/// Insert one `sidecar_imports` row. `signature_state` must be one of
/// `verified`, `unsigned`, `invalid`, `unknown_key` (column CHECK).
pub fn insert_sidecar_import(
    conn: &Connection,
    row: &SidecarImportRow,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO sidecar_imports (\n\
         \x20\x20        sidecar_import_id, artifact_sha256, signature_state,\n\
         \x20\x20        imported_layer_id, record_count, conflict_count, imported_at_ms\n\
         \x20\x20      ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            row.sidecar_import_id,
            row.artifact_sha256,
            row.signature_state,
            row.imported_layer_id,
            row.record_count,
            row.conflict_count,
            row.imported_at_ms,
        ],
    )?;
    Ok(())
}

/// Return the `sidecar_imports` row for an artifact hash, if present.
pub fn sidecar_import_by_sha256(
    conn: &Connection,
    artifact_sha256: &str,
) -> Result<Option<SidecarImportRow>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT sidecar_import_id, artifact_sha256, signature_state,\n\
         \x20\x20       imported_layer_id, record_count, conflict_count, imported_at_ms\n\
         \x20\x20FROM sidecar_imports WHERE artifact_sha256 = ?1",
    )?;
    stmt.query_row(params![artifact_sha256], |row| {
        Ok(SidecarImportRow {
            sidecar_import_id: row.get(0)?,
            artifact_sha256: row.get(1)?,
            signature_state: row.get(2)?,
            imported_layer_id: row.get(3)?,
            record_count: row.get(4)?,
            conflict_count: row.get(5)?,
            imported_at_ms: row.get(6)?,
        })
    })
    .optional()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migration::{apply_pragmas, MigrationRunner};

    fn memory_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        apply_pragmas(&conn).unwrap();
        let runner = MigrationRunner::new(&conn);
        runner.apply_initial().unwrap();
        conn
    }

    fn anchor_pack(conn: &Connection) -> String {
        // annotations.pack_id FK-targets packs; create the minimal pack.
        conn.execute(
            "INSERT INTO packs (pack_id, corpus_id, display_name, description,\n\
             \x20\x20  language_bcp47, pack_kind, active_pack_version_id,\n\
             \x20\x20  created_at_ms, updated_at_ms)\n\
             \x20\x20VALUES ('pack-1', 'corpus-1', 'P', '', 'en', 'full_zim', NULL, 1, 1)",
            [],
        )
        .unwrap();
        "pack-1".to_string()
    }

    fn local_layer(conn: &Connection) -> String {
        let row = SidecarLayerRow {
            sidecar_layer_id: "layer-local".to_string(),
            name: "Local".to_string(),
            owner_device_id: None,
            visibility: "private".to_string(),
            trust_state: "local".to_string(),
            source_artifact_id: None,
            created_at_ms: 1,
            updated_at_ms: 1,
            annotation_count: 0,
        };
        insert_sidecar_layer(conn, &row).unwrap();
        "layer-local".to_string()
    }

    fn sample_annotation(layer: &str) -> AnnotationRow {
        AnnotationRow {
            annotation_id: format!("ann-{}", layer),
            annotation_type: "highlight".to_string(),
            pack_id: "pack-1".to_string(),
            article_key: "article/main".to_string(),
            notebook_id: None,
            sidecar_layer_id: layer.to_string(),
            current_revision: 1,
            resolution_state: "resolved".to_string(),
            pinned: false,
            deleted_at_ms: None,
            created_at_ms: 10,
            updated_at_ms: 10,
        }
    }

    fn revision_row(id: &str) -> AnnotationRevisionRow {
        AnnotationRevisionRow {
            annotation_id: id.to_string(),
            revision: 1,
            body_text: Some("a thought".to_string()),
            selector_json: r#"{"textQuote":{"exact":"hello"}}"#.to_string(),
            style_json: r#"{"colorToken":"yellow","emphasis":"none"}"#.to_string(),
            context_text: "the hello world context".to_string(),
            source_revision_digest: None,
            authored_by_device_id: None,
            created_at_ms: 10,
        }
    }

    #[test]
    fn annotation_round_trip_with_revision_and_tags() {
        let conn = memory_conn();
        anchor_pack(&conn);
        let layer = local_layer(&conn);
        let row = sample_annotation(&layer);
        insert_annotation(&conn, &row).unwrap();
        insert_annotation_revision(&conn, &revision_row(&row.annotation_id)).unwrap();

        let tag = TagRow {
            tag_id: "tag-1".to_string(),
            name: "Important".to_string(),
            color_token: "blue".to_string(),
            created_at_ms: 1,
        };
        insert_tag(&conn, &tag).unwrap();
        link_annotation_tag(&conn, &row.annotation_id, &tag.tag_id).unwrap();

        let head = annotation_row(&conn, &row.annotation_id)
            .unwrap()
            .expect("row present");
        assert_eq!(head.current_revision, 1);
        assert!(!head.pinned);

        let rev = latest_revision(&conn, &row.annotation_id)
            .unwrap()
            .expect("revision present");
        assert_eq!(rev.body_text.as_deref(), Some("a thought"));

        let tags = annotation_tags(&conn, &row.annotation_id).unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "Important");
    }

    #[test]
    fn list_filters_exclude_deleted_and_match_text() {
        let conn = memory_conn();
        anchor_pack(&conn);
        let layer = local_layer(&conn);
        let mut row = sample_annotation(&layer);
        insert_annotation(&conn, &row).unwrap();
        insert_annotation_revision(&conn, &revision_row(&row.annotation_id)).unwrap();
        row.annotation_id = "ann-2".to_string();
        conn.execute(
            "INSERT INTO packs (pack_id, corpus_id, display_name, description,\n\
             \x20\x20  language_bcp47, pack_kind, active_pack_version_id,\n\
             \x20\x20  created_at_ms, updated_at_ms)\n\
             \x20\x20VALUES ('pack-other', 'corpus-2', 'Q', '', 'en', 'full_zim', NULL, 1, 1)",
            [],
        )
        .unwrap();
        row.pack_id = "pack-other".to_string();
        insert_annotation(&conn, &row).unwrap();
        let mut rev = revision_row("ann-2");
        rev.body_text = Some("unrelated".to_string());
        insert_annotation_revision(&conn, &rev).unwrap();

        // Text filter hits only the first annotation.
        let (rows, total) = list_annotations(
            &conn,
            &AnnotationFilter {
                text: Some("thought".to_string()),
                ..Default::default()
            },
            0,
            50,
        )
        .unwrap();
        assert_eq!((rows.len(), total), (1, 1));
        assert_eq!(rows[0].annotation_id, "ann-layer-local");

        // pack_ids filter.
        let (rows, _) = list_annotations(
            &conn,
            &AnnotationFilter {
                pack_ids: vec!["pack-other".to_string()],
                ..Default::default()
            },
            0,
            50,
        )
        .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].annotation_id, "ann-2");

        // Soft-delete removes it from the default listing…
        soft_delete_annotation(&conn, "ann-2", 99).unwrap();
        let (_, total) = list_annotations(&conn, &AnnotationFilter::default(), 0, 50).unwrap();
        assert_eq!(total, 1);
        // …and include_deleted brings it back.
        let (_, total) = list_annotations(
            &conn,
            &AnnotationFilter {
                include_deleted: true,
                ..Default::default()
            },
            0,
            50,
        )
        .unwrap();
        assert_eq!(total, 2);

        restore_annotation(&conn, "ann-2").unwrap();
        let (_, total) = list_annotations(&conn, &AnnotationFilter::default(), 0, 50).unwrap();
        assert_eq!(total, 2);
    }

    #[test]
    fn like_wildcards_in_text_are_literal() {
        let conn = memory_conn();
        anchor_pack(&conn);
        let layer = local_layer(&conn);
        let row = sample_annotation(&layer);
        insert_annotation(&conn, &row).unwrap();
        insert_annotation_revision(&conn, &revision_row(&row.annotation_id)).unwrap();

        // "%" must not match everything once escaped.
        let (rows, _) = list_annotations(
            &conn,
            &AnnotationFilter {
                text: Some("%".to_string()),
                ..Default::default()
            },
            0,
            50,
        )
        .unwrap();
        assert!(rows.is_empty());
    }

    #[test]
    fn layer_listing_counts_live_annotations() {
        let conn = memory_conn();
        anchor_pack(&conn);
        let layer = local_layer(&conn);
        let row = sample_annotation(&layer);
        insert_annotation(&conn, &row).unwrap();
        insert_annotation_revision(&conn, &revision_row(&row.annotation_id)).unwrap();

        let layers = list_sidecar_layers(&conn).unwrap();
        assert_eq!(layers.len(), 1);
        assert_eq!(layers[0].annotation_count, 1);
        assert_eq!(layers[0].visibility, "private");

        let fetched = sidecar_layer(&conn, &layer).unwrap().unwrap();
        assert_eq!(fetched.name, "Local");
    }

    #[test]
    fn notebook_and_tag_dedup_semantics() {
        let conn = memory_conn();
        let nb = NotebookRow {
            notebook_id: "nb-1".to_string(),
            parent_notebook_id: None,
            name: "Research".to_string(),
            description: String::new(),
            sort_order: 0,
            created_at_ms: 1,
            updated_at_ms: 1,
            annotation_count: 0,
        };
        insert_notebook(&conn, &nb).unwrap();
        assert!(notebook_exists(&conn, "nb-1").unwrap());
        assert!(!notebook_exists(&conn, "nb-2").unwrap());
        update_notebook(&conn, "nb-1", None, "Renamed", "d", 5).unwrap();
        let notebooks = list_notebooks(&conn).unwrap();
        assert_eq!(notebooks[0].name, "Renamed");

        let tag = TagRow {
            tag_id: "tag-1".to_string(),
            name: "todo".to_string(),
            color_token: "red".to_string(),
            created_at_ms: 1,
        };
        insert_tag(&conn, &tag).unwrap();
        // Case-insensitive lookup via the column's NOCASE collation.
        let found = tag_by_name(&conn, "TODO").unwrap().expect("nocase match");
        assert_eq!(found.tag_id, "tag-1");
        // Duplicate name (any casing) violates the UNIQUE constraint.
        let dup = TagRow {
            tag_id: "tag-2".to_string(),
            name: "TODO".to_string(),
            color_token: "red".to_string(),
            created_at_ms: 2,
        };
        assert!(insert_tag(&conn, &dup).is_err());
    }

    #[test]
    fn sidecar_import_row_round_trip() {
        let conn = memory_conn();
        anchor_pack(&conn);
        let layer = local_layer(&conn);
        let import = SidecarImportRow {
            sidecar_import_id: "imp-1".to_string(),
            artifact_sha256: "a".repeat(64),
            signature_state: "verified".to_string(),
            imported_layer_id: Some(layer),
            record_count: 3,
            conflict_count: 1,
            imported_at_ms: 7,
        };
        insert_sidecar_import(&conn, &import).unwrap();
        let fetched = sidecar_import_by_sha256(&conn, &import.artifact_sha256)
            .unwrap()
            .expect("import present");
        assert_eq!(fetched.record_count, 3);
        assert_eq!(fetched.conflict_count, 1);
        assert!(sidecar_import_by_sha256(&conn, &"b".repeat(64))
            .unwrap()
            .is_none());
    }

    #[test]
    fn asset_rows_round_trip() {
        let conn = memory_conn();
        anchor_pack(&conn);
        let layer = local_layer(&conn);
        let row = sample_annotation(&layer);
        insert_annotation(&conn, &row).unwrap();
        let asset = AnnotationAssetRow {
            annotation_asset_id: "asset-1".to_string(),
            annotation_id: row.annotation_id.clone(),
            asset_kind: "voice_opus".to_string(),
            object_sha256: "c".repeat(64),
            mime_type: "audio/opus".to_string(),
            byte_length: 2048,
            duration_ms: Some(4_200),
            created_at_ms: 11,
        };
        insert_annotation_asset(&conn, &asset).unwrap();
        let assets = annotation_assets(&conn, &row.annotation_id).unwrap();
        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].duration_ms, Some(4_200));
    }
}
