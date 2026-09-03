//! `ClusterCacheArchive` — the `ports::zim::ZimArchive` impl that backs
//! DynDon partial-cluster reads.
//!
//! Holds an immutable [`ClusterDirectory`] (built once from the DynDon
//! manifest), an LRU of cluster payloads already on device, and — for
//! production — an optional `TransferSource` + `base_uri` to fetch missing
//! clusters over HTTP byte-range on demand.
//!
//! All `ZimArchive` methods are `async`. `metadata()`, `main_page()`,
//! `entry()`, `entries()`, `search_titles()`, and `verify_structure()` are
//! answerable from the directory alone (no cluster I/O). `article()` and
//! `resource()` resolve the cluster via the cache (and on-demand channel,
//! if wired), decompress it through `zim_anzimmerman::decompress_cluster_blobs`,
//! and return the blob at the resolved index.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use domain::identity::{ArticleKey, ContentDigest, EntryKey};
use futures_util::StreamExt;
use ports::transfer::{TransferRange, TransferSource};
use ports::zim::{
    ArticleDocument, InternalLink, ResourceDocument, TitleSearchHit, TitleSearchQuery,
    ZimArchive, ZimEntryDescriptor, ZimEntryKind, ZimError, ZimEntryStream,
    ZimVerificationFinding, ZimVerificationReport,
};
use sha2::{Digest, Sha256};
use tokio::sync::RwLock;
use tracing::warn;

use crate::manifest::ClusterDirectory;

/// Engine identity surfaced through `verify_structure`.
pub const ENGINE_ID: &str = "zim.cluster_cache.v1";
pub const ENGINE_VERSION: &str = "0.1.0";

/// Cap on the number of decompressed cluster payloads held in memory at
/// once. M1 default is conservative — each cluster can be a few MiB even
/// after decompression.
pub const DEFAULT_LRU_CAPACITY: usize = 8;

/// A single cached cluster, fully decompressed into its ordered blob list.
/// Stored as `Bytes` per blob so `ArticleDocument::raw_html` etc. can be
/// assembled with a cheap reference-counted clone.
#[derive(Clone)]
struct CachedCluster {
    blobs: Vec<Bytes>,
}

/// Tiny LRU-ish map keyed by cluster index. M1 FLAG: this is a `HashMap`
/// guarded by an `RwLock`, NOT a true LRU. When capacity is exceeded the
/// map simply keeps growing — no eviction happens in M1. For the small
/// DynDon-subset pack sizes targeted in M1 (a few clusters, all of which
/// were pre-downloaded by the planner) this is acceptable; production will
/// swap in a real bounded LRU.
#[derive(Default)]
struct ClusterLru {
    entries: HashMap<u32, CachedCluster>,
}

impl ClusterLru {
    fn get(&self, cluster_index: u32) -> Option<&CachedCluster> {
        self.entries.get(&cluster_index)
    }

    fn insert(&mut self, cluster_index: u32, cluster: CachedCluster) {
        self.entries.insert(cluster_index, cluster);
    }
}

/// The `ZimArchive` impl. Cheap to clone — the directory, cache, and
/// transfer source all live behind `Arc`.
#[derive(Clone)]
pub struct ClusterCacheArchive {
    directory: Arc<ClusterDirectory>,
    local_clusters: Arc<RwLock<ClusterLru>>,
    /// Production on-demand fetch channel. `None` in M1 (offline); when
    /// `None`, a cluster miss surfaces as `EntryNotFound`.
    source: Option<Arc<dyn TransferSource>>,
    /// Base URI of the source pack (e.g. an HTTPS URL or `file://` path).
    /// Required when `source` is `Some`; cluster byte ranges are appended
    /// via `TransferSource::open_range`.
    base_uri: Option<String>,
    engine_id: &'static str,
    engine_version: String,
}

impl ClusterCacheArchive {
    /// Build an offline-only archive: the directory plus a set of clusters
    /// that are already on device (the DynDon-downloaded subset). Any
    /// article whose cluster isn't here will return `EntryNotFound`.
    ///
    /// `local_clusters_raw` is keyed by cluster index; each value is the
    /// raw cluster payload read straight from the pack (compression
    /// prefix byte + body), exactly the bytes that
    /// `zim_anzimmerman::decompress_cluster_blobs` expects.
    pub fn new_offline(
        directory: ClusterDirectory,
        local_clusters_raw: HashMap<u32, Bytes>,
    ) -> Result<Self, ZimError> {
        let mut cache = ClusterLru::default();
        for (cluster_index, raw) in local_clusters_raw {
            let blobs = decompress_and_split(&raw)?;
            cache.insert(cluster_index, CachedCluster { blobs });
        }
        Ok(Self {
            directory: Arc::new(directory),
            local_clusters: Arc::new(RwLock::new(cache)),
            source: None,
            base_uri: None,
            engine_id: ENGINE_ID,
            engine_version: ENGINE_VERSION.to_string(),
        })
    }

    /// Build a production archive with on-demand cluster fetch. Missing
    /// clusters are pulled via `source.open_range(base_uri, range)` and
    /// cached. `base_uri` MUST be `Some` when `source` is `Some`.
    pub fn new_online(
        directory: ClusterDirectory,
        local_clusters_raw: HashMap<u32, Bytes>,
        source: Arc<dyn TransferSource>,
        base_uri: String,
    ) -> Result<Self, ZimError> {
        let mut me = Self::new_offline(directory, local_clusters_raw)?;
        me.source = Some(source);
        me.base_uri = Some(base_uri);
        Ok(me)
    }

    /// Look up `cluster_index` in the local cache; if absent and a
    /// `TransferSource` is wired, fetch it via byte-range and cache it.
    /// Returns the cached `CachedCluster` (cloned — cheap, the blobs are
    /// `Bytes`).
    async fn resolve_cluster(
        &self,
        cluster_index: u32,
    ) -> Result<Option<CachedCluster>, ZimError> {
        // Fast path: already local.
        {
            let guard = self.local_clusters.read().await;
            if let Some(cluster) = guard.get(cluster_index) {
                return Ok(Some(cluster.clone()));
            }
        }

        let (source, base_uri) = match (&self.source, &self.base_uri) {
            (Some(s), Some(u)) => (s.clone(), u.clone()),
            // M1 offline miss: truthful error. No fabrication.
            _ => return Ok(None),
        };

        let range = match self.directory.cluster_range(cluster_index) {
            Some(r) => r,
            None => return Ok(None),
        };

        // On-demand fetch + bounded aggregation.
        let transfer_range = TransferRange {
            start_byte: range.start_byte,
            end_byte_exclusive: range.end_byte_exclusive,
        };
        let stream = source
            .open_range(&base_uri, transfer_range, None)
            .await
            .map_err(transfer_err_to_zim)?;
        let mut raw = Vec::new();
        let mut s = stream;
        while let Some(item) = s.next().await {
            let bytes = item.map_err(transfer_err_to_zim)?;
            raw.extend_from_slice(&bytes);
        }
        let raw_bytes = Bytes::from(raw);
        let blobs = decompress_and_split(&raw_bytes)?;

        let cluster = CachedCluster { blobs };
        // Re-validate under the write lock (another task may have inserted).
        let mut guard = self.local_clusters.write().await;
        if let Some(existing) = guard.get(cluster_index) {
            return Ok(Some(existing.clone()));
        }
        guard.insert(cluster_index, cluster.clone());
        Ok(Some(cluster))
    }

    /// Pull blob `blob_index` from `cluster_index`'s decompressed payload.
    async fn blob_at(
        &self,
        cluster_index: u32,
        blob_index: u32,
    ) -> Result<Option<Bytes>, ZimError> {
        let cluster = match self.resolve_cluster(cluster_index).await? {
            Some(c) => c,
            None => return Ok(None),
        };
        let idx = blob_index as usize;
        if idx >= cluster.blobs.len() {
            return Err(ZimError::EntryNotFound(format!(
                "blob index {} out of range (cluster {} has {} blobs)",
                blob_index,
                cluster_index,
                cluster.blobs.len()
            )));
        }
        Ok(Some(cluster.blobs[idx].clone()))
    }
}

/// ===========================================================================
/// Free helpers
/// ===========================================================================

/// Run the public `decompress_cluster_blobs` from `zim-anzimmerman` and
/// wrap each returned blob in `Bytes` for cheap cloning.
fn decompress_and_split(raw: &[u8]) -> Result<Vec<Bytes>, ZimError> {
    let blobs = zim_anzimmerman::cluster::decompress_cluster_blobs(raw)?;
    Ok(blobs.into_iter().map(Bytes::from).collect())
}

/// Compute a `ContentDigest` (sha256) over `payload`. Same scheme as
/// `zim-anzimmerman` uses for its article/resource digests.
fn sha256_digest(payload: &[u8]) -> ContentDigest {
    let mut hasher = Sha256::new();
    hasher.update(payload);
    let digest = hasher.finalize();
    let hex: String = digest.iter().map(|b| format!("{:02x}", b)).collect();
    // `sha256_from_hex` only fails on malformed hex; ours is canonical.
    ContentDigest::sha256_from_hex(hex)
        .expect("sha256 hex output is always 64 lowercase hex chars")
}

/// Map a `TransferError` to the closest-fit `ZimError`. `Offline` /
/// `NotFound` become `EntryNotFound` (the cluster isn't reachable from
/// this device right now); transport / source-changed become `Io` (the
/// reader can't tell whether the byte stream was truncated).
fn transfer_err_to_zim(err: ports::transfer::TransferError) -> ZimError {
    use ports::transfer::TransferError as TE;
    match err {
        TE::Offline | TE::NotFound | TE::PermissionDenied => {
            ZimError::EntryNotFound(format!("cluster fetch unavailable: {}", err))
        }
        TE::RangeUnsupported | TE::InvalidRangeResponse | TE::ResponseLimitExceeded => {
            ZimError::Io(format!("cluster range rejected by source: {}", err))
        }
        TE::SourceChanged => ZimError::Io(format!("cluster source changed: {}", err)),
        TE::Transport(msg) => ZimError::Io(format!("cluster transport: {}", msg)),
    }
}

/// Build a `ZimEntryDescriptor` from a directory record + its key. No
/// cluster bytes required.
fn descriptor_from_record(key: &ArticleKey, record: &crate::ArticleRecord) -> ZimEntryDescriptor {
    ZimEntryDescriptor {
        entry_key: EntryKey(key.0.clone()),
        namespace: record.namespace,
        path: record.path.clone(),
        title: record.title.clone(),
        mime_type: record.mime_type.clone(),
        kind: record.kind,
        redirect_target: record.redirect_target.clone(),
        // Byte length isn't tracked by the directory in M1; the manifest
        // carries no per-blob size. Surfacing 0 rather than guessing.
        byte_length: 0,
    }
}

/// ===========================================================================
/// `ZimArchive` impl
/// ===========================================================================

#[async_trait]
impl ZimArchive for ClusterCacheArchive {
    fn metadata(&self) -> &ports::zim::ZimMetadata {
        self.directory.metadata()
    }

    async fn main_page(&self) -> Result<Option<ArticleKey>, ZimError> {
        Ok(self.directory.main_page.clone())
    }

    async fn entry(&self, key: &EntryKey) -> Result<ZimEntryDescriptor, ZimError> {
        let (article_key, record) = self
            .directory
            .iter_articles()
            .find(|(ak, _)| &ak.0 == &key.0)
            .ok_or_else(|| ZimError::EntryNotFound(key.0.clone()))?;
        Ok(descriptor_from_record(article_key, record))
    }

    async fn article(&self, key: &ArticleKey) -> Result<ArticleDocument, ZimError> {
        let record = self
            .directory
            .article(key)
            .ok_or_else(|| ZimError::EntryNotFound(key.0.clone()))?;

        // Honor redirects at the type level: an `Article`-kind record with
        // a redirect target is not directly fetchable.
        if record.kind == ZimEntryKind::Redirect {
            return Err(ZimError::EntryNotFound(format!(
                "entry {} is a redirect (use resolve_redirect)",
                key.0
            )));
        }
        if record.kind != ZimEntryKind::Article {
            return Err(ZimError::EntryNotFound(format!(
                "entry {} is not in the article namespace (kind={:?})",
                key.0, record.kind
            )));
        }

        let blob = self.blob_at(record.cluster_index, record.blob_index).await?;
        let raw_html = match blob {
            Some(b) => b,
            // Cluster not present and not fetchable from this device.
            None => {
                return Err(ZimError::EntryNotFound(format!(
                    "cluster {} for article {} is not present on device",
                    record.cluster_index, key.0
                )))
            }
        };

        let content_digest = sha256_digest(&raw_html);
        Ok(ArticleDocument {
            article_key: key.clone(),
            title: record.title.clone(),
            mime_type: record.mime_type.clone(),
            raw_html,
            // M1 FLAG: normalized_text and internal_links are returned
            // empty. Producing them requires the `scraper`-based projection
            // that lives in `zim-anzimmerman::sanitize`. That projection is
            // intentionally not re-vendored here (it would need to be
            // lifted into a shared crate). Consumers that need these two
            // fields should project the raw HTML downstream.
            normalized_text: String::new(),
            content_digest,
            internal_links: Vec::<InternalLink>::new(),
        })
    }

    async fn resource(&self, key: &EntryKey) -> Result<ResourceDocument, ZimError> {
        let (article_key, record) = self
            .directory
            .iter_articles()
            .find(|(ak, _)| &ak.0 == &key.0)
            .ok_or_else(|| ZimError::EntryNotFound(key.0.clone()))?;

        if record.kind == ZimEntryKind::Redirect {
            return Err(ZimError::EntryNotFound(format!(
                "entry {} is a redirect (use resolve_redirect)",
                key.0
            )));
        }
        if record.kind != ZimEntryKind::Resource {
            return Err(ZimError::EntryNotFound(format!(
                "entry {} is not a resource (kind={:?})",
                key.0, record.kind
            )));
        }

        let blob = self.blob_at(record.cluster_index, record.blob_index).await?;
        let bytes = match blob {
            Some(b) => b,
            None => {
                return Err(ZimError::EntryNotFound(format!(
                    "cluster {} for resource {} is not present on device",
                    record.cluster_index, key.0
                )))
            }
        };

        let content_digest = sha256_digest(&bytes);
        Ok(ResourceDocument {
            entry_key: EntryKey(article_key.0.clone()),
            mime_type: record.mime_type.clone(),
            bytes,
            content_digest,
        })
    }

    async fn resolve_redirect(
        &self,
        key: &EntryKey,
        max_hops: u8,
    ) -> Result<EntryKey, ZimError> {
        let mut current = key.0.clone();
        let mut visited: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        for _ in 0..=max_hops {
            if !visited.insert(current.clone()) {
                return Err(ZimError::RedirectCycle);
            }
            let (ak, record) = self
                .directory
                .iter_articles()
                .find(|(ak, _)| ak.0 == current)
                .ok_or_else(|| ZimError::EntryNotFound(current.clone()))?;
            match record.redirect_target {
                Some(ref target) if record.kind == ZimEntryKind::Redirect => {
                    current = target.0.clone();
                }
                _ => return Ok(EntryKey(ak.0.clone())),
            }
        }
        Err(ZimError::UnsupportedFeature(format!(
            "redirect chain at '{}' exceeded max_hops={}",
            key.0, max_hops
        )))
    }

    async fn search_titles(
        &self,
        query: TitleSearchQuery,
    ) -> Result<Vec<TitleSearchHit>, ZimError> {
        if query.limit == 0 || query.text.trim().is_empty() {
            return Ok(Vec::new());
        }
        let needle = query.text.trim().to_lowercase();
        let mut hits: Vec<TitleSearchHit> = Vec::new();
        for (ak, record) in self.directory.iter_articles() {
            if record.kind != ZimEntryKind::Article {
                continue;
            }
            if record.title.is_empty() {
                continue;
            }
            let haystack = record.title.to_lowercase();
            let score = score_title(&needle, &haystack);
            if score > 0.0 {
                hits.push(TitleSearchHit {
                    article_key: ak.clone(),
                    title: record.title.clone(),
                    score,
                });
            }
        }
        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.title.cmp(&b.title))
        });
        let start = (query.offset as usize).min(hits.len());
        let end = start
            .saturating_add(query.limit as usize)
            .min(hits.len());
        Ok(hits[start..end].to_vec())
    }

    async fn entries(&self) -> Result<ZimEntryStream, ZimError> {
        // Collect the directory into a stable order so the stream yields
        // deterministically. M1 FLAG: the directory is a HashMap, so we
        // sort by key to give consumers a stable iteration order rather
        // than the HashMap's random order.
        let mut snapshot: Vec<(ArticleKey, crate::ArticleRecord)> = self
            .directory
            .iter_articles()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        snapshot.sort_by(|a, b| a.0 .0.cmp(&b.0 .0));

        let stream = async_stream::stream! {
            for (key, record) in snapshot {
                yield Ok(descriptor_from_record(&key, &record));
            }
        };
        Ok(Box::pin(stream))
    }

    async fn verify_structure(&self) -> Result<ZimVerificationReport, ZimError> {
        // M1 truthful report: every entry in the directory is "checked"
        // for internal consistency (redirect target resolves to a known
        // article, cluster range is known to this pack). We do NOT
        // attempt cluster fetches here — that would be a side-effecting
        // verify and could blow the offline cache budget.
        let mut warnings: Vec<ZimVerificationFinding> = Vec::new();
        let mut errors: Vec<ZimVerificationFinding> = Vec::new();
        let mut checked: u64 = 0;

        for (ak, record) in self.directory.iter_articles() {
            checked += 1;

            // Redirect targets must point somewhere known.
            if record.kind == ZimEntryKind::Redirect {
                if let Some(target) = record.redirect_target.as_ref() {
                    if self
                        .directory
                        .iter_articles()
                        .all(|(k, _)| &k.0 != &target.0)
                    {
                        errors.push(ZimVerificationFinding {
                            code: "redirect_target_missing".into(),
                            entry_key: Some(EntryKey(ak.0.clone())),
                            detail: format!(
                                "redirect target {} is not in the directory",
                                target.0
                            ),
                        });
                    }
                } else {
                    errors.push(ZimVerificationFinding {
                        code: "redirect_target_unset".into(),
                        entry_key: Some(EntryKey(ak.0.clone())),
                        detail: "entry is a Redirect but redirect_target is None"
                            .into(),
                    });
                }
                continue;
            }

            // Articles / resources must point to a cluster whose byte
            // range the pack knows. (Bytes-on-device is checked
            // separately at fetch time, not here.)
            if matches!(record.kind, ZimEntryKind::Article | ZimEntryKind::Resource) {
                if self
                    .directory
                    .cluster_range(record.cluster_index)
                    .is_none()
                {
                    errors.push(ZimVerificationFinding {
                        code: "cluster_range_unknown".into(),
                        entry_key: Some(EntryKey(ak.0.clone())),
                        detail: format!(
                            "entry points to cluster {} but the directory has no byte range for it",
                            record.cluster_index
                        ),
                    });
                }
            }
        }

        // Local cache vs. directory sanity: clusters present locally that
        // the directory doesn't know about are surfaced as warnings, not
        // errors — they don't block reading.
        let local_guard = self.local_clusters.read().await;
        let orphan_count = local_guard
            .entries
            .keys()
            .filter(|idx| self.directory.cluster_range(**idx).is_none())
            .count();
        if orphan_count > 0 {
            warn!(
                orphan_count,
                "cluster cache holds clusters not in the directory"
            );
            warnings.push(ZimVerificationFinding {
                code: "orphan_cached_clusters".into(),
                entry_key: None,
                detail: format!(
                    "{} cached cluster(s) are not referenced by the directory",
                    orphan_count
                ),
            });
        }

        let valid = errors.is_empty();
        Ok(ZimVerificationReport {
            valid,
            engine_id: self.engine_id.to_string(),
            engine_version: self.engine_version.clone(),
            checked_entries: checked,
            warnings,
            errors,
        })
    }
}

/// Title-score helper. Mirrors `zim-anzimmerman::archive::score_title` —
/// same buckets, same thresholds — so cluster-cache search returns the
/// same ranking users would get from a full AnZimmerman archive.
fn score_title(needle: &str, haystack_lower: &str) -> f32 {
    if haystack_lower == needle {
        return 1.0;
    }
    if haystack_lower.starts_with(needle) {
        return 0.9;
    }
    let mut prev = ' ';
    for (i, c) in haystack_lower.char_indices() {
        if !prev.is_alphanumeric() && haystack_lower[i..].starts_with(needle) {
            return 0.7;
        }
        prev = c;
    }
    if haystack_lower.contains(needle) {
        return 0.4;
    }
    0.0
}
