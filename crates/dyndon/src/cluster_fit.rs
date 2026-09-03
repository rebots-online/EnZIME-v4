//! The storage-fit planner — DynDon's core USP.
//!
//! Given a pack's [`ClusterManifest`] and a device's [`SelectionIntent`],
//! [`plan_fit`] selects the highest-value subset of clusters that fits the
//! device's free-space budget, dynamically narrowing breadth (domains /
//! topics) and depth (priority tiers) so that "all Wikipedia" can land as a
//! coherent tailored subset even when only a few GB are free.
//!
//! The planner is a pure function: deterministic, no I/O, no global state.

use std::collections::{HashMap, HashSet};

use crate::manifest::{ClusterManifest, ClusterFitPlan, SelectionIntent};

/// Plan a cluster selection that fits `intent.budget_bytes`.
///
/// Algorithm (the "dynamic depth & breadth to fit storage" USP):
///
/// 1. **Domain filter** — if `intent.domains` is non-empty, keep only entries
///    whose `domain` is in that set; otherwise keep all entries.
/// 2. **Breadth cap** — if `intent.breadth_limit` is set, keep only the
///    top-N distinct `(domain, topic)` slots ranked by aggregate priority
///    (sum of priorities of candidate entries in that slot).
/// 3. **Depth cap** — if `intent.depth_limit` is set, keep only entries in
///    the top-N distinct priority tiers (highest priorities win; lower
///    tiers drop first).
/// 4. **Rank** — sort surviving candidates by `priority` DESC, breaking ties
///    by `article_count` DESC. The sort is stable, so earlier source order
///    wins remaining ties (deterministic).
/// 5. **Greedy fit** — walk the whole ranked list, accumulating `byte_len`;
///    keep a cluster iff `running_total + byte_len <= budget_bytes`. The
///    scan continues past skipped clusters so a later, smaller cluster can
///    still fit.
/// 6. **Report** — `truncated` / `skipped_count` count candidates that
///    survived steps 1-3 but couldn't fit the budget; `domains_covered`
///    lists distinct selected domains; `rationale` summarizes the result.
pub fn plan_fit(manifest: &ClusterManifest, intent: &SelectionIntent) -> ClusterFitPlan {
    // Step 1: domain filter -> candidate indices into manifest.entries.
    let domain_filter: Option<HashSet<&str>> = if intent.domains.is_empty() {
        None
    } else {
        Some(intent.domains.iter().map(|s| s.as_str()).collect())
    };

    let mut cand_idx: Vec<usize> = (0..manifest.entries.len())
        .filter(|&i| match &domain_filter {
            None => true,
            Some(set) => manifest.entries[i]
                .domain
                .as_deref()
                .map(|d| set.contains(d))
                .unwrap_or(false),
        })
        .collect();

    // Step 2: breadth_limit -> keep top-N (domain, topic) slots by aggregate priority.
    if let Some(n) = intent.breadth_limit {
        if n == 0 {
            cand_idx.clear();
        } else {
            let n = n as usize;
            let mut agg: HashMap<(Option<&str>, Option<&str>), u64> = HashMap::new();
            for &i in &cand_idx {
                let e = &manifest.entries[i];
                *agg.entry((e.domain.as_deref(), e.topic.as_deref()))
                    .or_insert(0) += e.priority as u64;
            }
            // Rank slots by aggregate priority DESC; tie-break by the slot key
            // itself (ASC) so the result is fully deterministic across runs.
            let mut slots: Vec<((Option<&str>, Option<&str>), u64)> = agg.into_iter().collect();
            slots.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
            let allowed: HashSet<(Option<&str>, Option<&str>)> =
                slots.into_iter().take(n).map(|(k, _)| k).collect();
            cand_idx.retain(|i| {
                let e = &manifest.entries[*i];
                allowed.contains(&(e.domain.as_deref(), e.topic.as_deref()))
            });
        }
    }

    // Step 3: depth_limit -> keep entries in the top-N priority tiers.
    if let Some(limit) = intent.depth_limit {
        if limit == 0 {
            cand_idx.clear();
        } else {
            let limit = limit as usize;
            let mut tiers: Vec<u32> = cand_idx
                .iter()
                .map(|i| manifest.entries[*i].priority)
                .collect();
            tiers.sort_unstable_by(|a, b| b.cmp(a)); // DESC
            tiers.dedup();
            let allowed: HashSet<u32> = tiers.into_iter().take(limit).collect();
            cand_idx.retain(|i| allowed.contains(&manifest.entries[*i].priority));
        }
    }

    // Step 4: rank candidates (priority DESC, then article_count DESC). Stable
    // sort preserves source order for any remaining ties.
    cand_idx.sort_by(|a, b| {
        let ea = &manifest.entries[*a];
        let eb = &manifest.entries[*b];
        eb.priority
            .cmp(&ea.priority)
            .then_with(|| eb.article_count.cmp(&ea.article_count))
    });

    // Step 5: greedy fit across the whole ranked list.
    let budget = intent.budget_bytes;
    let mut sel_idx: Vec<usize> = Vec::new();
    let mut total_bytes: u64 = 0;
    for &i in &cand_idx {
        let e = &manifest.entries[i];
        match total_bytes.checked_add(e.byte_len) {
            Some(sum) if sum <= budget => {
                sel_idx.push(i);
                total_bytes = sum;
            }
            // Either the addition would overflow u64 (treat as "can't fit any
            // plausible budget") or it exceeds the declared budget: in both
            // cases the cluster is skipped due to the budget ceiling.
            _ => {}
        }
    }

    let candidate_count = cand_idx.len();
    let skipped_count = candidate_count.saturating_sub(sel_idx.len()) as u32;
    let truncated = skipped_count > 0;

    // domains_covered: distinct Some(domain) among selected, sorted alphabetically.
    let mut domains: HashSet<&str> = HashSet::new();
    for &i in &sel_idx {
        if let Some(d) = manifest.entries[i].domain.as_deref() {
            domains.insert(d);
        }
    }
    let mut domains_covered: Vec<String> = domains.into_iter().map(str::to_owned).collect();
    domains_covered.sort();

    let selected: Vec<u32> = sel_idx
        .iter()
        .map(|&i| manifest.entries[i].cluster_index)
        .collect();

    let free_bytes_after = intent.budget_bytes as i64 - total_bytes as i64;

    let rationale = build_rationale(
        selected.len(),
        candidate_count,
        domains_covered.len(),
        manifest.total_bytes,
        total_bytes,
        intent.budget_bytes,
        truncated,
    );

    ClusterFitPlan {
        selected,
        total_bytes,
        free_bytes_after,
        truncated,
        skipped_count,
        domains_covered,
        rationale,
    }
}

/// Compose the one-line human summary, mirroring the contract's example:
/// `"selected 142 of 880 clusters across 6 domains; full pack 93.4 GB, plan 4.7 GB fitting 5.0 GB budget (truncated)"`.
fn build_rationale(
    selected_count: usize,
    candidate_count: usize,
    domains_count: usize,
    full_pack_bytes: u64,
    plan_bytes: u64,
    budget_bytes: u64,
    truncated: bool,
) -> String {
    let status = if truncated { "truncated" } else { "complete" };
    format!(
        "selected {} of {} clusters across {} domains; full pack {}, plan {} fitting {} budget ({})",
        selected_count,
        candidate_count,
        domains_count,
        format_bytes(full_pack_bytes),
        format_bytes(plan_bytes),
        format_bytes(budget_bytes),
        status,
    )
}

/// Render a byte count with an SI suffix (B / KB / MB / GB / TB / PB), one
/// decimal place for units above B. Used only for human display in
/// `rationale`; never participates in planning arithmetic.
fn format_bytes(b: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    if b == 0 {
        return "0 B".to_string();
    }
    let mut val = b as f64;
    let mut unit_idx = 0usize;
    while val >= 1000.0 && unit_idx < UNITS.len() - 1 {
        val /= 1000.0;
        unit_idx += 1;
    }
    if unit_idx == 0 {
        format!("{} {}", b, UNITS[0])
    } else {
        format!("{:.1} {}", val, UNITS[unit_idx])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{ClusterManifest, ClusterManifestEntry};

    fn entry(
        idx: u32,
        byte_len: u64,
        domain: Option<&str>,
        topic: Option<&str>,
        priority: u32,
        article_count: u32,
    ) -> ClusterManifestEntry {
        ClusterManifestEntry {
            cluster_index: idx,
            byte_offset: 0,
            byte_len,
            domain: domain.map(str::to_owned),
            topic: topic.map(str::to_owned),
            priority,
            article_count,
        }
    }

    fn manifest(entries: Vec<ClusterManifestEntry>) -> ClusterManifest {
        let total_bytes = entries.iter().map(|e| e.byte_len).sum();
        let cluster_count = entries.len() as u32;
        ClusterManifest {
            pack_version_id: "test-v1".to_owned(),
            cluster_count,
            total_bytes,
            entries,
        }
    }

    fn intent(budget: u64) -> SelectionIntent {
        SelectionIntent {
            domains: vec![],
            breadth_limit: None,
            depth_limit: None,
            budget_bytes: budget,
        }
    }

    // (a) everything fits -> nothing truncated, all selected, ranked.
    #[test]
    fn everything_fits_not_truncated() {
        let m = manifest(vec![
            entry(0, 100, Some("Med"), Some("Cardio"), 10, 50),
            entry(1, 200, Some("Med"), Some("Neuro"), 5, 30),
            entry(2, 150, Some("CS"), Some("AI"), 8, 40),
        ]);
        let plan = plan_fit(&m, &intent(1_000_000));

        assert!(!plan.truncated);
        assert_eq!(plan.skipped_count, 0);
        assert_eq!(plan.selected.len(), 3);
        assert_eq!(plan.total_bytes, 450);
        assert_eq!(plan.free_bytes_after, 1_000_000 - 450);
        // Ranked: prio 10 (idx 0), prio 8 (idx 2), prio 5 (idx 1).
        assert_eq!(plan.selected, vec![0, 2, 1]);
        // Domains sorted alphabetically.
        assert_eq!(plan.domains_covered, vec!["CS".to_owned(), "Med".to_owned()]);
        assert!(plan.rationale.contains("(complete)"));
    }

    // (b) budget smaller than total -> truncated, respects budget, free >= 0,
    //     and later smaller clusters still fit after a skip.
    #[test]
    fn budget_smaller_than_total_truncates() {
        // Ranked by priority DESC: 0 (100B, prio10), 1 (200B, prio5), 2 (50B, prio1).
        // Budget 220: take 0 (total=100); 1 would make 300 > 220 -> skip;
        //              2 makes 150 <= 220 -> take. Demonstrates "don't stop at first overflow".
        let m = manifest(vec![
            entry(0, 100, Some("Med"), None, 10, 0),
            entry(1, 200, Some("Med"), None, 5, 0),
            entry(2, 50, Some("CS"), None, 1, 0),
        ]);
        let plan = plan_fit(&m, &intent(220));

        assert!(plan.truncated);
        assert!(plan.total_bytes <= 220);
        assert!(plan.free_bytes_after >= 0);
        assert_eq!(plan.total_bytes, 150);
        assert_eq!(plan.free_bytes_after, 70);
        assert_eq!(plan.skipped_count, 1);
        assert_eq!(plan.selected, vec![0, 2]);
        assert!(plan.rationale.contains("(truncated)"));
    }

    // (c) domain filter respected; None-domain entries are excluded when filter active.
    #[test]
    fn domain_filter_respected() {
        let m = manifest(vec![
            entry(0, 100, Some("Med"), None, 5, 0),
            entry(1, 100, Some("CS"), None, 5, 0),
            entry(2, 100, Some("Med"), None, 5, 0),
            entry(3, 100, None, None, 5, 0),
        ]);
        let mut it = intent(1_000_000);
        it.domains = vec!["Med".to_owned()];
        let plan = plan_fit(&m, &it);

        assert_eq!(plan.selected.len(), 2);
        assert!(plan.selected.contains(&0));
        assert!(plan.selected.contains(&2));
        assert!(!plan.selected.contains(&1));
        assert!(!plan.selected.contains(&3));
        assert_eq!(plan.domains_covered, vec!["Med".to_owned()]);
    }

    // (d) empty manifest -> empty plan.
    #[test]
    fn empty_manifest_yields_empty_plan() {
        let m = manifest(vec![]);
        let plan = plan_fit(&m, &intent(1000));

        assert!(plan.selected.is_empty());
        assert_eq!(plan.total_bytes, 0);
        assert_eq!(plan.free_bytes_after, 1000);
        assert!(!plan.truncated);
        assert_eq!(plan.skipped_count, 0);
        assert!(plan.domains_covered.is_empty());
        assert!(plan.rationale.contains("selected 0 of 0"));
    }

    // (e) breadth_limit caps distinct (domain, topic) slots by aggregate priority.
    #[test]
    fn breadth_limit_keeps_top_n_slots() {
        // Slots: (A,x) agg=5+5=10 ; (B,y) agg=7 ; (C,z) agg=3.
        let m = manifest(vec![
            entry(0, 10, Some("A"), Some("x"), 5, 0),
            entry(1, 10, Some("A"), Some("x"), 5, 0),
            entry(2, 10, Some("B"), Some("y"), 7, 0),
            entry(3, 10, Some("C"), Some("z"), 3, 0),
        ]);
        let mut it = intent(1_000_000);
        it.breadth_limit = Some(2);
        let plan = plan_fit(&m, &it);

        // Top-2 slots are (A,x) and (B,y); (C,z) dropped entirely.
        assert_eq!(plan.selected.len(), 3);
        assert!(!plan.selected.contains(&3));
        assert!(plan.selected.contains(&0));
        assert!(plan.selected.contains(&1));
        assert!(plan.selected.contains(&2));
    }

    // (f) depth_limit keeps only the top-N priority tiers.
    #[test]
    fn depth_limit_keeps_top_n_tiers() {
        let m = manifest(vec![
            entry(0, 10, Some("A"), None, 10, 0),
            entry(1, 10, Some("A"), None, 8, 0),
            entry(2, 10, Some("A"), None, 5, 0),
            entry(3, 10, Some("A"), None, 1, 0),
        ]);
        let mut it = intent(1_000_000);
        it.depth_limit = Some(2);
        let plan = plan_fit(&m, &it);

        // Top-2 tiers are {10, 8} -> entries 0 and 1.
        assert_eq!(plan.selected.len(), 2);
        assert!(plan.selected.contains(&0));
        assert!(plan.selected.contains(&1));
        assert!(!plan.selected.contains(&2));
        assert!(!plan.selected.contains(&3));
    }

    // (g) zero-byte clusters always fit, even at budget 0.
    #[test]
    fn zero_byte_clusters_always_fit() {
        let m = manifest(vec![
            entry(0, 0, Some("A"), None, 5, 0),
            entry(1, 0, Some("A"), None, 5, 0),
        ]);
        let plan = plan_fit(&m, &intent(0));
        assert_eq!(plan.selected.len(), 2);
        assert_eq!(plan.total_bytes, 0);
        assert_eq!(plan.free_bytes_after, 0);
        assert!(!plan.truncated);
    }

    // (h) tie-break: equal priority ranks by article_count DESC.
    #[test]
    fn ties_break_by_article_count_desc() {
        let m = manifest(vec![
            entry(0, 10, Some("A"), None, 5, 10),
            entry(1, 10, Some("A"), None, 5, 99),
            entry(2, 10, Some("A"), None, 5, 50),
        ]);
        let plan = plan_fit(&m, &intent(1_000_000));
        // Same priority -> article_count DESC: 1 (99), 2 (50), 0 (10).
        assert_eq!(plan.selected, vec![1, 2, 0]);
    }
}
