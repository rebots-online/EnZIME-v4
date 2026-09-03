//! Data types for DynDon's cluster manifest and storage-fit planning.
//!
//! These are the on-disk / over-the-wire shapes that describe a ZIM pack's
//! cluster layout ([`ClusterManifest`]) and the device-side request to fit
//! that pack into bounded free space ([`SelectionIntent`]). The planner's
//! result is a [`ClusterFitPlan`].
//!
//! All types are plain data: pure, deterministic, no I/O. They are
//! `Serialize`/`Deserialize` so manifests and plans can be persisted or
//! streamed alongside pack transfers.

use serde::{Deserialize, Serialize};

/// A single cluster's metadata within a pack manifest.
///
/// A "cluster" is the unit of transferable, decompressible payload inside a
/// ZIM pack (a blob of compressed article bodies). Each cluster carries a
/// breadth tag (`domain`/`topic`, e.g. "Medicine" / "Cardio") and a depth
/// tag (`priority` — higher means more core to the pack).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClusterManifestEntry {
    /// Logical index of this cluster within the source ZIM.
    pub cluster_index: u32,
    /// Absolute byte offset of the cluster payload in the source ZIM.
    pub byte_offset: u64,
    /// Compressed length (in bytes) of the cluster payload.
    pub byte_len: u64,
    /// Breadth axis — the domain this cluster belongs to (e.g. "Medicine").
    /// `None` means the cluster has no domain classification.
    pub domain: Option<String>,
    /// Finer breadth axis — the topic within the domain (e.g. "Cardio").
    pub topic: Option<String>,
    /// Importance / depth tier. Higher = more core to the pack; lower-priority
    /// tiers are dropped first when storage is tight.
    pub priority: u32,
    /// Number of article blobs inside the cluster. Used as a secondary
    /// tie-breaker when ranking clusters of equal priority.
    pub article_count: u32,
}

/// The full manifest describing every cluster in a DynDon pack.
///
/// `total_bytes` is the sum of every entry's `byte_len` (the full pack size);
/// callers use it to compare against a device budget before attempting a
/// full install.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClusterManifest {
    /// Identifier of the pack version this manifest describes.
    pub pack_version_id: String,
    /// Number of entries in `entries` (kept as a denormalized field for
    /// quick sanity checks without scanning the vec).
    pub cluster_count: u32,
    /// Sum of every entry's `byte_len`. The uncompressed-then-recompressed
    /// full pack footprint.
    pub total_bytes: u64,
    /// One entry per cluster, in source-ZIM order.
    pub entries: Vec<ClusterManifestEntry>,
}

/// The device-side intent: which slices of a pack the caller wants and how
/// much storage they can spare right now.
///
/// DynDon's USP ("download all Wikipedia even when you don't have enough
/// disc space") is realized by combining the breadth filters (`domains`,
/// `breadth_limit`) with the depth filter (`depth_limit`) and a hard
/// `budget_bytes` ceiling: the planner drops the least-valuable clusters
/// first so the most important breadth-and-depth slice always fits.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SelectionIntent {
    /// Restrict selection to these domains. Empty = all domains. Entries
    /// whose `domain` is `None` are excluded when this list is non-empty.
    pub domains: Vec<String>,
    /// Cap the number of distinct `(domain, topic)` breadth slots included,
    /// keeping the slots with the highest aggregate priority. `None` = no
    /// breadth cap.
    pub breadth_limit: Option<u32>,
    /// Cap the number of distinct priority tiers included, keeping the
    /// highest-priority tiers (lower-priority tiers are dropped first).
    /// `None` = no depth cap.
    pub depth_limit: Option<u32>,
    /// The device's free-space budget, in bytes, for this pack. The plan
    /// never exceeds this.
    pub budget_bytes: u64,
}

/// The planner's output: which clusters to transfer, in what order, and why.
///
/// Produced by [`crate::cluster_fit::plan_fit`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClusterFitPlan {
    /// `cluster_index` of each selected cluster, in the order the planner
    /// recommends transferring them (highest priority first).
    pub selected: Vec<u32>,
    /// Sum of every selected cluster's `byte_len`. Always `<= budget_bytes`.
    pub total_bytes: u64,
    /// `budget_bytes - total_bytes`. Always `>= 0`.
    pub free_bytes_after: i64,
    /// `true` iff any candidate was dropped SOLELY because of the budget
    /// (i.e. more clusters were desired than would fit).
    pub truncated: bool,
    /// Number of candidates not selected due to the budget ceiling.
    pub skipped_count: u32,
    /// Distinct domains among the selected clusters, sorted alphabetically.
    pub domains_covered: Vec<String>,
    /// One-line human-readable summary of the plan.
    pub rationale: String,
}
