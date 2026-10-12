use super::*;
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
