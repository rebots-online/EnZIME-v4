//! EnZIME port adapter for canonical AnZimmermanLib Rust archives.
use crate::convert;
use anzimmerman::model::ZimArchive as _;
use async_trait::async_trait;
use domain::identity::{ArticleKey, EntryKey};
use futures_util::StreamExt;
use ports::zim::*;
use std::sync::Arc;
pub struct AnZimmermanZimArchive {
    inner: Arc<anzimmerman::archive::AnZimmermanZimArchive>,
    metadata: ZimMetadata,
}
pub async fn open_archive(request: &ZimOpenRequest) -> Result<AnZimmermanZimArchive, ZimError> {
    let inner = Arc::new(
        anzimmerman::archive::open_archive(&convert::open_request(request))
            .await
            .map_err(convert::adapter_error)?,
    );
    let metadata = convert::metadata(inner.metadata().clone());
    Ok(AnZimmermanZimArchive { inner, metadata })
}
/// A per-entry location record captured by [`AnZimmermanZimArchive::layout_snapshot`].
///
/// Mirrors [`ZimEntryDescriptor`] but adds the on-disk location fields
/// (`cluster_index`, `blob_index`) that the async trait surface intentionally
/// omits. Sufficient for the integration layer to build a cluster-cache
/// directory without re-reading the file.
#[derive(Clone, Debug)]
pub struct ZimLayoutEntry {
    /// `<namespace>/<url>` entry key, identical to `ZimEntryDescriptor::entry_key`.
    pub entry_key: String,
    /// Namespace character (e.g. `'A'`, `'-'`, `'M'`).
    pub namespace: char,
    /// URL tail under the namespace.
    pub path: String,
    /// Human-readable title (may differ from `path`).
    pub title: String,
    /// MIME type, resolved from the file's MIME list. Empty for redirects.
    pub mime_type: String,
    /// Article / Resource / Redirect / Metadata.
    pub kind: ZimEntryKind,
    /// If `kind == Redirect`, the canonical destination key. `None` otherwise.
    pub redirect_target: Option<EntryKey>,
    /// Which cluster holds this entry's blob. Only meaningful for
    /// `Article`/`Resource` kinds.
    pub cluster_index: u32,
    /// Blob index inside that cluster. Only meaningful for
    /// `Article`/`Resource` kinds.
    pub blob_index: u32,
}

/// A snapshot of the on-disk ZIM layout: one byte range per cluster (in
/// source order) plus one [`ZimLayoutEntry`] per directory entry.
///
/// Exposed so the integration layer can build both a DynDon `ClusterManifest`
/// (per-cluster byte ranges, driving ranged download) and a cluster-cache
/// `ClusterDirectory` (per-article cluster/blob indices, driving partial-pack
/// reads) in a single pass — without re-reading the file or going through the
/// async trait surface (which does not carry cluster/blob indices on
/// `ZimEntryDescriptor`).
///
/// Producing this snapshot touches directory pointers and the MIME list only;
/// it does NOT read or decompress any cluster payload.
#[derive(Clone, Debug)]
pub struct ZimLayoutSnapshot {
    /// `(byte_offset, byte_len)` for each cluster, indexed by cluster index.
    /// `byte_len` is the raw on-disk length including the compression-prefix
    /// byte, exactly the bytes a ranged GET must fetch.
    pub cluster_byte_ranges: Vec<(u64, u64)>,
    /// One entry per directory slot, in source-order.
    pub entries: Vec<ZimLayoutEntry>,
}

impl AnZimmermanZimArchive {
    pub fn layout_snapshot(&self) -> Result<ZimLayoutSnapshot, ZimError> {
        self.inner
            .layout_snapshot()
            .map(convert::layout)
            .map_err(convert::adapter_error)
    }
}
#[async_trait]
impl ZimArchive for AnZimmermanZimArchive {
    fn metadata(&self) -> &ZimMetadata {
        &self.metadata
    }
    async fn main_page(&self) -> Result<Option<ArticleKey>, ZimError> {
        self.inner
            .main_page()
            .await
            .map(|v| v.map(|k| ArticleKey(k.0)))
            .map_err(convert::adapter_error)
    }
    async fn entry(&self, key: &EntryKey) -> Result<ZimEntryDescriptor, ZimError> {
        self.inner
            .entry(&anzimmerman::EntryKey(key.0.clone()))
            .await
            .map(convert::entry)
            .map_err(convert::adapter_error)
    }
    async fn article(&self, key: &ArticleKey) -> Result<ArticleDocument, ZimError> {
        self.inner
            .article(&anzimmerman::ArticleKey(key.0.clone()))
            .await
            .map(convert::article)
            .map_err(convert::adapter_error)
    }
    async fn resource(&self, key: &EntryKey) -> Result<ResourceDocument, ZimError> {
        self.inner
            .resource(&anzimmerman::EntryKey(key.0.clone()))
            .await
            .map(convert::resource)
            .map_err(convert::adapter_error)
    }
    async fn resolve_redirect(&self, key: &EntryKey, max_hops: u8) -> Result<EntryKey, ZimError> {
        self.inner
            .resolve_redirect(&anzimmerman::EntryKey(key.0.clone()), max_hops)
            .await
            .map(|k| EntryKey(k.0))
            .map_err(convert::adapter_error)
    }
    async fn search_titles(&self, q: TitleSearchQuery) -> Result<Vec<TitleSearchHit>, ZimError> {
        self.inner
            .search_titles(anzimmerman::TitleSearchQuery {
                text: q.text,
                offset: q.offset,
                limit: q.limit,
            })
            .await
            .map(|v| v.into_iter().map(convert::hit).collect())
            .map_err(convert::adapter_error)
    }
    async fn entries(&self) -> Result<ZimEntryStream, ZimError> {
        let stream = self.inner.entries().await.map_err(convert::adapter_error)?;
        Ok(Box::pin(stream.map(|v| {
            v.map(convert::entry).map_err(convert::adapter_error)
        })))
    }
    async fn verify_structure(&self) -> Result<ZimVerificationReport, ZimError> {
        self.inner
            .verify_structure()
            .await
            .map(convert::report)
            .map_err(convert::adapter_error)
    }
}
