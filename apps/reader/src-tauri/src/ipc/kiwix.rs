//! Native OPDS browsing with a durable, explicitly labelled offline cache.
use super::*;
use catalog_openzim::kiwix::{KiwixCatalogClient, KiwixEntry, KiwixQuery};
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogPage {
    pub items: Vec<LibraryCatalogEntry>,
    pub total: Option<u64>,
    pub next_offset: Option<u32>,
    pub source: String,
    pub notice: Option<String>,
}
pub fn language_code(language: Option<&str>) -> Option<String> {
    let value = language?.trim().to_ascii_lowercase();
    let mapped = match value.as_str() {
        "en" => "eng",
        "fr" => "fra",
        "de" => "deu",
        "es" => "spa",
        "pt" => "por",
        "zh" => "zho",
        "ja" => "jpn",
        "ar" => "ara",
        "ru" => "rus",
        other => other,
    };
    (mapped.len() == 3 && mapped.bytes().all(|b| b.is_ascii_lowercase()))
        .then(|| mapped.to_string())
}
pub fn row_id(entry: &KiwixEntry) -> String {
    format!("kiwix-{}", storage_sqlite::sha256_hex(entry.id.as_bytes()))
}
pub fn cached_catalog(
    state: &AppState,
    query: &str,
    language: Option<&str>,
    category: Option<&str>,
    offset: u32,
    limit: u32,
) -> Result<CatalogPage, String> {
    let conn = state.conn.lock().map_err(err_str)?;
    let rows = search_catalog_entries(
        &conn,
        query,
        language,
        category,
        i64::from(limit) + 1,
        offset.into(),
    )
    .map_err(err_str)?;
    let more = rows.len() > limit as usize;
    Ok(CatalogPage {
        items: rows
            .into_iter()
            .take(limit as usize)
            .map(map_catalog_entry)
            .collect(),
        total: None,
        next_offset: more.then_some(offset.saturating_add(limit)),
        source: "cache".into(),
        notice: Some(
            "Cached Kiwix entries; availability and advertised sizes may have changed.".into(),
        ),
    })
}
pub async fn search_kiwix_catalog(
    state: &AppState,
    query: &str,
    language: Option<&str>,
    offset: u32,
    limit: u32,
) -> Result<CatalogPage, String> {
    if query.len() > 4096 {
        return Err("catalog query is too long".into());
    }
    let limit = limit.clamp(1, 100);
    let language = language_code(language);
    let client = KiwixCatalogClient::new(state.transfer_source.clone());
    let result = client
        .search(&KiwixQuery {
            text: (!query.trim().is_empty()).then(|| query.trim().into()),
            language: language.clone(),
            offset,
            limit,
        })
        .await;
    let page = match result {
        Ok(page) => page,
        Err(error) => {
            let mut cached =
                cached_catalog(state, query, language.as_deref(), None, offset, limit)?;
            if cached.items.is_empty() {
                return Err(format!(
                    "Kiwix catalog unavailable and no matching cached entries: {error}"
                ));
            }
            cached.notice = Some(format!(
                "Kiwix unavailable ({error}); showing cached entries."
            ));
            return Ok(cached);
        }
    };
    let mut conn = state.conn.lock().map_err(err_str)?;
    let transaction = conn.transaction().map_err(err_str)?;
    let now = now_ms();
    transaction.execute("INSERT INTO catalog_sources(catalog_source_id,source_kind,display_name,endpoint,enabled,priority,last_refresh_at_ms,created_at_ms,updated_at_ms) VALUES('kiwix-opds-v2','kiwix_opds','Kiwix library','https://library.kiwix.org/catalog/v2/entries',1,0,?1,?1,?1) ON CONFLICT(catalog_source_id) DO UPDATE SET last_refresh_at_ms=excluded.last_refresh_at_ms,updated_at_ms=excluded.updated_at_ms",[now]).map_err(err_str)?;
    let mut items = Vec::new();
    for entry in page.entries {
        let id = row_id(&entry);
        let metadata = serde_json::to_string(&entry).map_err(err_str)?;
        let size = i64::try_from(entry.advertised_size_bytes).map_err(err_str)?;
        transaction.execute("INSERT INTO catalog_entries(catalog_entry_id,catalog_source_id,corpus_id,upstream_version,title,description,language_bcp47,category,publisher,artifact_url,artifact_size_bytes,artifact_sha256,metadata_json,discovered_at_ms) VALUES(?1,'kiwix-opds-v2',?2,?3,?4,?5,?6,?7,?8,?9,?10,NULL,?11,?12) ON CONFLICT(catalog_entry_id) DO UPDATE SET title=excluded.title,description=excluded.description,language_bcp47=excluded.language_bcp47,category=excluded.category,publisher=excluded.publisher,artifact_url=excluded.artifact_url,artifact_size_bytes=excluded.artifact_size_bytes,metadata_json=excluded.metadata_json,discovered_at_ms=excluded.discovered_at_ms",params![id,entry.name,entry.updated,entry.title,entry.description,entry.language,entry.category,entry.publisher,entry.acquisition_url,size,metadata,now]).map_err(err_str)?;
        items.push(map_catalog_entry(
            catalog_entry_by_id(&transaction, &id)
                .map_err(err_str)?
                .ok_or("cached catalog row disappeared")?,
        ));
    }
    transaction.commit().map_err(err_str)?;
    Ok(CatalogPage {
        items,
        total: page.total,
        next_offset: page.next_offset,
        source: "live".into(),
        notice: None,
    })
}
pub fn load_entry(state: &AppState, id: &str) -> Result<KiwixEntry, String> {
    let conn = state.conn.lock().map_err(err_str)?;
    let json:String=conn.query_row("SELECT metadata_json FROM catalog_entries WHERE catalog_entry_id=?1 AND catalog_source_id='kiwix-opds-v2'",[id],|r|r.get(0)).map_err(err_str)?;
    serde_json::from_str(&json).map_err(|e| format!("invalid cached Kiwix metadata: {e}"))
}
