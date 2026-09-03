//! Cluster-cache directory types — the always-local manifest that lets a
//! reader locate an article's bytes inside a pack whose clusters are only
//! PARTIALLY present on device.
//!
//! The directory owns three things:
//!   * the article map: `ArticleKey` -> `(cluster_index, blob_index)` plus
//!     the static metadata needed to build an `ArticleDocument` /
//!     `ZimEntryDescriptor` without touching the cluster bytes
//!     (title, mime_type, namespace, path, kind, redirect target).
//!   * the cluster byte-range map: `cluster_index` -> `(start_byte,
//!     end_byte_exclusive)` — where the cluster lives in the source pack,
//!     used to drive `TransferSource::open_range` on demand.
//!   * the pack-level `ZimMetadata` + main-page key + ordered title list
//!     for `search_titles`.
//!
//! M1 (offline) builds this directory once from a manifest produced by the
//! DynDon planner; production (online) wires a `TransferSource` so missing
//! clusters are fetched via HTTP byte-range and cached.

use bytes::Bytes;
use domain::identity::{ArticleKey, EntryKey};
use ports::zim::{ZimEntryKind, ZimMetadata};
use serde::{Deserialize, Serialize};

/// A single article's static directory record. Everything here can be
/// served without reading any cluster payload — only `article()` /
/// `resource()` actually pull bytes.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArticleRecord {
    /// The article's namespace character (e.g. 'A' for articles, '-' for
    /// metadata). Stored as `char` so it round-trips through `ZimEntryDescriptor::namespace`.
    pub namespace: char,
    /// Path component under the namespace (the URL tail).
    pub path: String,
    /// Human-readable title (may differ from `path`).
    pub title: String,
    /// MIME type, copied from the manifest. M1 does not interpret it.
    pub mime_type: String,
    /// Article vs. Resource vs. Redirect vs. Metadata.
    pub kind: ZimEntryKind,
    /// If `kind == Redirect`, the canonical destination key. `None`
    /// otherwise.
    pub redirect_target: Option<EntryKey>,
    /// The cluster that holds this article's blob.
    pub cluster_index: u32,
    /// The blob index inside that cluster.
    pub blob_index: u32,
}

/// Byte range of one cluster inside the source pack URI. The range is
/// `[start_byte, end_byte_exclusive)` — the on-the-wire form expected by
/// `ports::transfer::TransferRange`.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ClusterRange {
    pub start_byte: u64,
    pub end_byte_exclusive: u64,
}

/// The full in-memory directory. Built once at archive open time (from a
/// DynDon manifest or by scanning a complete ZIM header); treated as
/// immutable thereafter. Cluster payloads live in a separate LRU.
///
/// Not `Default` — `ZimMetadata` deliberately has no default, so callers
/// must build a directory explicitly via [`ClusterDirectory::new`] and
/// then assign `.metadata`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClusterDirectory {
    /// Article / entry key -> static record. M1 FLAG: this is a flat
    /// `HashMap`, not a sorted path-index; `search_titles` walks the
    /// value list rather than a ZIM-style title index. Sufficient for
    /// the small DynDon-subset pack sizes targeted in M1.
    pub articles: std::collections::HashMap<ArticleKey, ArticleRecord>,

    /// Per-cluster byte range, used to drive on-demand fetch. A cluster
    /// index that is missing here is "not present in this pack" — the
    /// reader cannot satisfy it and returns `EntryNotFound`.
    pub cluster_ranges: std::collections::HashMap<u32, ClusterRange>,

    /// Pack-level metadata, surfaced verbatim through `ZimArchive::metadata`.
    pub metadata: ZimMetadata,

    /// The article key for `main_page()`, if the manifest named one.
    pub main_page: Option<ArticleKey>,
}

impl ClusterDirectory {
    /// Construct an empty directory; fields are then populated by the
    /// caller (the DynDon manifest loader).
    pub fn new() -> Self {
        Self {
            articles: std::collections::HashMap::new(),
            cluster_ranges: std::collections::HashMap::new(),
            metadata: empty_metadata(),
            main_page: None,
        }
    }

    /// Look up the static record for `key`. Cheap; touches no cluster bytes.
    pub fn article(&self, key: &ArticleKey) -> Option<&ArticleRecord> {
        self.articles.get(key)
    }

    /// Look up by `EntryKey`. Returns the same record as `article()` but
    /// keyed on the entry-key form (which the trait's `entry()` /
    /// `resource()` paths take).
    pub fn entry(&self, key: &EntryKey) -> Option<&ArticleRecord> {
        self.articles
            .iter()
            .find(|(ak, _)| ak.0 == key.0)
            .map(|(_, r)| r)
    }

    /// The byte range for a given cluster index, if this pack knows it.
    pub fn cluster_range(&self, cluster_index: u32) -> Option<ClusterRange> {
        self.cluster_ranges.get(&cluster_index).copied()
    }

    /// Borrow the pack metadata.
    pub fn metadata(&self) -> &ZimMetadata {
        &self.metadata
    }

    /// Number of articles / entries in the directory — the value reported
    /// by `verify_structure`'s `checked_entries`.
    pub fn len(&self) -> usize {
        self.articles.len()
    }

    /// Iterator over all `(ArticleKey, &ArticleRecord)` pairs — used by
    /// `entries()` and `search_titles()` which can't borrow the HashMap
    /// directly without exposing the underlying type.
    pub fn iter_articles(
        &self,
    ) -> impl Iterator<Item = (&ArticleKey, &ArticleRecord)> {
        self.articles.iter()
    }
}

/// A tiny owned blob + MIME pair returned by the cache layer once a cluster
/// has been fetched and split into its member blobs. Kept as `Bytes` so the
/// `ZimArchive` impl can hand the buffer straight to `ArticleDocument` /
/// `ResourceDocument` without another copy.
#[derive(Clone, Debug)]
pub struct CachedBlob {
    pub bytes: Bytes,
    pub mime_type: String,
}

/// Build an all-empty `ZimMetadata` for the directory default. Callers
/// (`ClusterDirectory::new`) get a directory whose `.metadata` is empty
/// until the manifest loader overwrites it with the real pack metadata.
fn empty_metadata() -> ZimMetadata {
    ZimMetadata {
        uuid: String::new(),
        title: String::new(),
        description: String::new(),
        language_bcp47: String::new(),
        creator: String::new(),
        publisher: String::new(),
        date: String::new(),
        article_count: 0,
        media_count: 0,
        main_page: None,
        checksum: None,
    }
}
