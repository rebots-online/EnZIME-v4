//! Cluster-cache `ZimArchive` — lets a reader render articles from a pack
//! whose clusters are only PARTIALLY present on device (the DynDon subset).
//!
//! The ZIM directory (article -> cluster/blob map) is always local; cluster
//! PAYLOADS are either cached locally (downloaded by the DynDon plan) or, in
//! production, fetched on demand via `ports::transfer::TransferSource`
//! (byte-range) and LRU-evicted. NO ZIM writer is required — this reads the
//! directory + decompresses individual clusters, never assembling a full
//! monolithic archive.
//!
//! ## M1 scope (offline, locally-cached clusters only)
//! The `TransferSource` + `base_uri` channels exist on the constructor so a
//! production caller can wire them, but for M1 they are `None`: a cluster
//! that is not in `local_clusters` simply yields `EntryNotFound`. The
//! on-demand fetch path is implemented (and is exercised by `cargo check`),
//! but it is not smoke-tested against a live remote in M1.
//!
//! ## Honest-minimal trait surface
//! `ArticleDocument.normalized_text` and `internal_links` are returned EMPTY
//! in M1 — the cluster-cache crate intentionally does not pull in the
//! HTML-projection dependency that `zim-anzimmerman` has (`scraper`). The
//! raw HTML + a computed `sha256` digest are always populated; consumers
//! that need normalized text/internal links should re-project downstream or
//! wait for a later milestone that lifts `sanitize::project` into a shared
//! crate. This simplification is FLAGGED in `cache_archive::article`.

#![forbid(unsafe_code)]

pub mod cache_archive;
pub mod manifest;

pub use cache_archive::ClusterCacheArchive;
pub use manifest::{ArticleRecord, CachedBlob, ClusterDirectory, ClusterRange};

pub mod disk;
