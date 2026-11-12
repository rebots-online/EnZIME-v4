use super::*;

async fn selected_clusters(subject: Option<&str>) -> (Vec<u32>, PlanSummary) {
    let mut builder = support::ZimBuilder::default();
    for i in 0..8 {
        builder = builder.add_article(
            b'A', format!("topic_{i}"), format!("Topic {i}"), "text/html",
            vec![b'x'; 3 * 1024 * 1024],
        );
    }
    builder.main_page_index = Some(0);
    let mut bytes = Vec::new();
    builder.write(&mut bytes);
    let len = bytes.len() as u64;
    let (root, _, store, index) = opened(bytes, len).await;
    let summary = plan_disk_fit_with_preference(
        store.clone(), index.clone(), dyndon::SelectionIntent {
            domains: vec![], breadth_limit: None, depth_limit: None,
            budget_bytes: 10 * 1024 * 1024,
        }, &[0], subject,
    ).await.unwrap();
    assert!(summary.selected_bytes + summary.metadata_reserved_bytes + summary.workspace_reserved_bytes <= summary.disk_budget_bytes);
    discover_offline_entries(store.clone(), index).await.unwrap();
    install_selected(store).await.unwrap();
    let mut selected: Vec<u32> = std::fs::read_dir(root.join("clusters")).unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .filter_map(|name| name.strip_suffix(".bin").and_then(|n| n.parse().ok()))
        .collect();
    selected.sort_unstable();
    (selected, summary)
}

#[tokio::test]
async fn cuts_retain_evenly_spaced_clusters_and_real_subject_priority_changes_them() {
    let (even, summary) = selected_clusters(None).await;
    assert_eq!(even, vec![0, 2, 4]);
    assert_eq!(summary.selected_count, 3);
    let (preferred, _) = selected_clusters(Some("TOPIC 7")).await;
    assert_eq!(preferred, vec![0, 4, 7]);
    assert_ne!(even, preferred);
}

#[tokio::test]
async fn unmatched_subjects_fail_without_claiming_the_even_plan_is_prioritized() {
    let bytes = fixture();
    let len = bytes.len() as u64;
    let (_, _, store, index) = opened(bytes, len).await;
    let error = plan_disk_fit_with_preference(
        store, index, dyndon::SelectionIntent {
            domains: vec![], breadth_limit: None, depth_limit: None,
            budget_bytes: 1024 * 1024,
        }, &[0], Some("a-subject-absent-from-this-archive"),
    ).await.unwrap_err();
    assert!(error.contains("No source domain labels or article titles match"));
}
