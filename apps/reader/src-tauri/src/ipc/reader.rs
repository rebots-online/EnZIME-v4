use super::*;
use ports::zim::TitleSearchQuery;

/// Search the installed archive, preserving its title ranking and retained scope.
#[tauri::command]
pub async fn reader_search_titles(
    state: tauri::State<'_, AppState>,
    pack_id: String,
    query: String,
    offset: Option<u32>,
    limit: Option<u32>,
) -> Result<ReaderTitleSearchPage, String> {
    if pack_id.trim().is_empty() {
        return Err("invalid_request: packId must not be empty".into());
    }
    let query = query.trim().to_string();
    if query.chars().count() > 256 {
        return Err("invalid_request: title query must contain at most 256 characters".into());
    }
    let offset = offset.unwrap_or(0);
    let limit = limit.unwrap_or(25).clamp(1, 100);
    if offset > 10_000 {
        return Err("invalid_request: title search supports offsets up to 10000; refine your search to see more matches".into());
    }
    let archive = {
        let archives = state.archives.lock().map_err(err_str)?;
        archives.get(&pack_id).cloned().ok_or_else(|| {
            format!("not_found: no open archive for packId {pack_id}; open an installed pack first")
        })?
    };
    let mut hits = Vec::new();
    if !query.is_empty() {
        // The retained range archive caps offsets at 10,000 and requests at
        // 200. At most 101 obtains real lookahead even at that last offset.
        // Other adapters may cap a page at 100; handle that below.
        hits = archive
            .search_titles(TitleSearchQuery {
                text: query.clone(),
                offset,
                limit: limit + 1,
            })
            .await
            .map_err(err_str)?;
    }
    let has_more = if hits.len() > limit as usize {
        true
    } else if hits.len() == limit as usize && offset + limit <= 10_000 {
        !archive
            .search_titles(TitleSearchQuery {
                text: query.clone(),
                offset: offset + limit,
                limit: 1,
            })
            .await
            .map_err(err_str)?
            .is_empty()
    } else {
        false
    };
    hits.truncate(limit as usize);
    let items = hits
        .into_iter()
        .map(|hit| {
            if !hit.score.is_finite() {
                return Err("invalid_archive: title search returned a non-finite score".to_string());
            }
            Ok(ReaderTitleSearchItem {
                article_key: hit.article_key.0,
                title: hit.title,
                score: hit.score,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(ReaderTitleSearchPage {
        pack_id,
        query,
        offset,
        limit,
        has_more,
        items,
    })
}

/// §14.4 `reader_open({ packId })`. Opens an INSTALLED pack: looks up the
/// `ClusterCacheArchive` registered in [`AppState::archives`] at install time
/// and returns its main page key. Any `path` argument the frontend may still
/// send is ignored — the packId is the only key.
#[tauri::command]
pub async fn reader_open(
    state: tauri::State<'_, AppState>,
    pack_id: String,
    #[allow(unused_variables)] path: Option<String>,
) -> Result<ReaderOpenResponse, String> {
    if pack_id.trim().is_empty() {
        return Err("invalid_request: packId must not be empty".to_string());
    }

    let archive = {
        let archives = state.archives.lock().map_err(err_str)?;
        archives.get(&pack_id).cloned().ok_or_else(|| {
            format!(
                "not_found: no installed pack for packId {} — call install_start first",
                pack_id
            )
        })?
    };

    let title = archive.metadata().title.clone();
    let main_page_key = archive.main_page().await.map_err(err_str)?.map(|k| k.0);

    Ok(ReaderOpenResponse {
        pack_id,
        title,
        main_page_key,
    })
}

/// §14.4 `reader_article({ packId, articleKey })`. Reads one article from
/// the registered archive. Article keys may contain `/` (e.g. `A/...`) — the
/// newtype is constructed directly, matching the engine's convention.
#[tauri::command]
pub async fn reader_article(
    state: tauri::State<'_, AppState>,
    pack_id: String,
    article_key: String,
) -> Result<ArticleResponse, String> {
    if pack_id.trim().is_empty() || article_key.trim().is_empty() {
        return Err("invalid_request: packId and articleKey must not be empty".to_string());
    }

    let archive = {
        let archives = state.archives.lock().map_err(err_str)?;
        archives
            .get(&pack_id)
            .cloned()
            .ok_or_else(|| format!("not_found: no open archive for packId {}", pack_id))?
    };

    let key = domain::identity::ArticleKey(article_key.clone());
    let document = archive.article(&key).await.map_err(err_str)?;

    // Best-effort visit history (§6.2): a failed write must never fail the read.
    let conn = state.conn.clone();
    let pack_id_copy = pack_id.clone();
    let article_key_copy = document.article_key.0.clone();
    let title_copy = document.title.clone();
    let _ = tokio::task::spawn_blocking(move || {
        if let Ok(g) = conn.lock() {
            let _ = storage_sqlite::query::record_reader_visit(
                &g,
                &ulid::Ulid::new().to_string(),
                &pack_id_copy,
                &article_key_copy,
                &title_copy,
                now_ms(),
            );
        }
    })
    .await;

    Ok(ArticleResponse {
        pack_id,
        article_key: document.article_key.0,
        title: document.title,
        raw_html: String::from_utf8_lossy(&document.raw_html).into_owned(),
        mime_type: document.mime_type,
        internal_links: document
            .internal_links
            .into_iter()
            .map(map_internal_link)
            .collect(),
    })
}
