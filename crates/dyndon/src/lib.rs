//! DynDon — EnZIME's storage-fit cluster planner (the product USP).
//!
//! "Load all Wikipedia, no matter how little app storage you can spare right
//! now." Given a pack's cluster manifest and the device's free-space budget,
//! DynDon selects the cluster subset (by domain/breadth and priority/depth)
//! that fits — so a 90 GB Wikipedia can install into 5 GB as a tailored
//! subset. This crate is the pure planning brain; I/O (transfer, caching)
//! lives elsewhere. Implementation filled in by coder B.

pub mod cluster_fit;
pub mod manifest;

pub use cluster_fit::plan_fit;
pub use manifest::{
    ClusterFitPlan, ClusterManifest, ClusterManifestEntry, SelectionIntent,
};
