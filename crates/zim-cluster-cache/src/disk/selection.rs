use super::{store::err, DiskPackStore};
use anzimmerman::{directory::RawEntry, range::RemoteZimIndex};
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CutStrategy {
    #[default]
    Even,
    PreferDomain,
}

pub fn subject_matches(key: &str, terms: &[String]) -> bool {
    let normalized = key.to_lowercase().replace('_', " ");
    terms.iter().any(|term| normalized.contains(term))
}

pub async fn apply_selection_preference(
    store: &DiskPackStore,
    index: &RemoteZimIndex,
    preferred_domain: Option<&str>,
) -> Result<(), String> {
    let Some(subject) = preferred_domain else {
        return Ok(());
    };
    let terms: Vec<String> = subject
        .split(',')
        .map(|term| term.trim().to_lowercase())
        .filter(|term| !term.is_empty())
        .collect();
    if terms.is_empty() || subject.chars().count() > 120 {
        return Err("Enter a domain or topic of at most 120 characters to prioritize.".into());
    }
    {
        let conn = store.conn.lock().map_err(err)?;
        conn.execute("UPDATE clusters SET priority=0", [])
            .map_err(err)?;
        let mut stmt = conn
            .prepare("SELECT cluster_index,domain FROM clusters WHERE domain IS NOT NULL")
            .map_err(err)?;
        let mut rows = stmt.query([]).map_err(err)?;
        while let Some(row) = rows.next().map_err(err)? {
            if subject_matches(&row.get::<_, String>(1).map_err(err)?, &terms) {
                // Only bump rows still at 0: an already matched cluster is
                // never rewritten by a later matching domain/article.
                conn.execute(
                    "UPDATE clusters SET priority=1 WHERE cluster_index=?1 AND priority=0",
                    [row.get::<_, u32>(0).map_err(err)?],
                )
                .map_err(err)?;
            }
        }
    }
    // Read directory metadata; install_selected acquires content payloads later.
    // The scan runs inside the 180s install_plan timeout, so it yields to the
    // runtime every 64 entries — including before each non-matching `continue`
    // — to stay cooperative with other tasks on the same worker.
    for cursor in 0..index.header().entry_count {
        if cursor % 64 == 0 {
            tokio::task::yield_now().await;
        }
        let records = index.resolve_records(cursor, 16).await.map_err(err)?;
        if !records
            .iter()
            .any(|record| subject_matches(&record.entry.key.0, &terms))
        {
            continue;
        }
        if let Some(RawEntry::Article(article)) = records.last().map(|record| &record.entry.raw) {
            store
                .conn
                .lock()
                .map_err(err)?
                .execute(
                    "UPDATE clusters SET priority=1 WHERE cluster_index=?1 AND priority=0",
                    params![article.cluster_number],
                )
                .map_err(err)?;
        }
    }
    let matches: u64 = store
        .conn
        .lock()
        .map_err(err)?
        .query_row(
            "SELECT COUNT(*) FROM clusters WHERE priority=1",
            [],
            |row| row.get(0),
        )
        .map_err(err)?;
    if matches == 0 {
        return Err(format!("No source domain labels or article titles match '{subject}'. Choose another topic or spread cuts evenly."));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn subjects_use_real_case_insensitive_article_keys() {
        let terms = vec!["heart disease".into(), "cardiology".into()];
        assert!(subject_matches("A/Heart_disease", &terms));
        assert!(subject_matches("A/CARDIOLOGY", &terms));
        assert!(!subject_matches("A/Architecture", &terms));
        assert!(!subject_matches("A/Anything", &[]));
    }
    #[test]
    fn strategies_have_stable_wire_names_and_reject_unknown_modes() {
        assert_eq!(
            serde_json::to_string(&CutStrategy::Even).unwrap(),
            "\"even\""
        );
        assert_eq!(
            serde_json::to_string(&CutStrategy::PreferDomain).unwrap(),
            "\"prefer_domain\""
        );
        assert!(serde_json::from_str::<CutStrategy>("\"unknown\"").is_err());
    }
}
