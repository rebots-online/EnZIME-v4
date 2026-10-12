use super::{ClusterBlobSource, IndexedEntry, RangeReadLimits, RemoteZimIndex, ZimEntryIndex};
use crate::{
    archive::{classify_kind, score_title, sha256_digest},
    directory::RawEntry,
    identity::*,
    model::*,
    sanitize,
};
use async_trait::async_trait;
use futures_util::StreamExt;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
pub struct RangeZimArchive {
    index: Arc<dyn ZimEntryIndex>,
    blobs: Arc<dyn ClusterBlobSource>,
    metadata: ZimMetadata,
    limits: RangeReadLimits,
    checked: Arc<AtomicU64>,
}
impl RangeZimArchive {
    pub async fn open(
        index: Arc<RemoteZimIndex>,
        blobs: Arc<dyn ClusterBlobSource>,
        metadata: ZimMetadata,
        limits: RangeReadLimits,
    ) -> Result<Self, ZimError> {
        Self::open_index(index, blobs, metadata, limits).await
    }
    pub async fn open_index(
        index: Arc<dyn ZimEntryIndex>,
        blobs: Arc<dyn ClusterBlobSource>,
        mut metadata: ZimMetadata,
        limits: RangeReadLimits,
    ) -> Result<Self, ZimError> {
        limits.validate()?;
        metadata.uuid = hex::encode(index.header().uuid);
        metadata.main_page = index.main_entry().await?.map(|e| ArticleKey(e.key.0));
        Ok(Self {
            index,
            blobs,
            metadata,
            limits,
            checked: Arc::new(AtomicU64::new(0)),
        })
    }
    pub fn index(&self) -> &Arc<dyn ZimEntryIndex> {
        &self.index
    }
    async fn descriptor(
        index: &dyn ZimEntryIndex,
        e: IndexedEntry,
    ) -> Result<ZimEntryDescriptor, ZimError> {
        let (kind, redirect_target) = match &e.raw {
            RawEntry::Article(a) => (
                classify_kind(a.namespace, e.mime_type.as_deref().unwrap_or("")),
                None,
            ),
            RawEntry::Redirect(r) => (
                ZimEntryKind::Redirect,
                Some(index.entry_at(r.redirect_index).await?.key),
            ),
            _ => (ZimEntryKind::Metadata, None),
        };
        Ok(ZimEntryDescriptor {
            entry_key: e.key,
            namespace: e.raw.namespace() as char,
            path: e.raw.url().into(),
            title: e.raw.title().into(),
            mime_type: e.mime_type.unwrap_or_default(),
            kind,
            redirect_target,
            byte_length: 0,
        })
    }
    async fn content(
        &self,
        key: &EntryKey,
        article_only: bool,
    ) -> Result<(IndexedEntry, bytes::Bytes), ZimError> {
        let e = self
            .index
            .resolve_entry(key, self.limits.max_redirect_hops)
            .await?;
        let RawEntry::Article(a) = &e.raw else {
            return Err(ZimError::EntryNotFound(key.0.clone()));
        };
        if article_only
            && classify_kind(a.namespace, e.mime_type.as_deref().unwrap_or(""))
                != ZimEntryKind::Article
        {
            return Err(ZimError::EntryNotFound(format!(
                "entry {} is not a readable article",
                key.0
            )));
        }
        let range = self.index.cluster_range(a.cluster_number).await?;
        let bytes = self
            .blobs
            .read_blob(
                a.cluster_number,
                range,
                a.blob_number,
                self.limits.max_blob_bytes,
            )
            .await?;
        if bytes.len() as u64 > self.limits.max_blob_bytes {
            return Err(ZimError::AllocationLimit);
        }
        self.checked.fetch_add(1, Ordering::Relaxed);
        Ok((e, bytes))
    }
}
#[async_trait]
impl ZimArchive for RangeZimArchive {
    fn metadata(&self) -> &ZimMetadata {
        &self.metadata
    }
    async fn main_page(&self) -> Result<Option<ArticleKey>, ZimError> {
        Ok(self.metadata.main_page.clone())
    }
    async fn entry(&self, key: &EntryKey) -> Result<ZimEntryDescriptor, ZimError> {
        let e = self
            .index
            .find_entry(key)
            .await?
            .ok_or_else(|| ZimError::EntryNotFound(key.0.clone()))?;
        let result = Self::descriptor(self.index.as_ref(), e).await?;
        self.checked.fetch_add(1, Ordering::Relaxed);
        Ok(result)
    }
    async fn article(&self, key: &ArticleKey) -> Result<ArticleDocument, ZimError> {
        let (e, raw_html) = self.content(&EntryKey(key.0.clone()), true).await?;
        let mime_type = e.mime_type.unwrap_or_default();
        let projection = sanitize::project(&raw_html);
        let content_digest = sha256_digest(&raw_html);
        Ok(ArticleDocument {
            article_key: ArticleKey(e.key.0),
            title: e.raw.title().into(),
            mime_type,
            raw_html,
            normalized_text: projection.normalized_text,
            content_digest,
            internal_links: projection.internal_links,
        })
    }
    async fn resource(&self, key: &EntryKey) -> Result<ResourceDocument, ZimError> {
        let (e, bytes) = self.content(key, false).await?;
        let content_digest = sha256_digest(&bytes);
        Ok(ResourceDocument {
            entry_key: e.key,
            mime_type: e.mime_type.unwrap_or_default(),
            bytes,
            content_digest,
        })
    }
    async fn resolve_redirect(&self, key: &EntryKey, max_hops: u8) -> Result<EntryKey, ZimError> {
        Ok(self.index.resolve_entry(key, max_hops).await?.key)
    }
    async fn search_titles(
        &self,
        query: TitleSearchQuery,
    ) -> Result<Vec<TitleSearchHit>, ZimError> {
        let limit = query.limit.min(200) as usize;
        let offset = query.offset.min(10000) as usize;
        if limit == 0 || query.text.trim().is_empty() {
            return Ok(Vec::new());
        }
        let capacity = offset + limit;
        let needle = query.text.trim().to_lowercase();
        let mut hits = Vec::new();
        let mut entries = self.index.clone().entries().await?;
        let mut i = 0u64;
        while let Some(entry) = entries.next().await {
            // Even an immediately-ready memory source must permit cancellation.
            if i % 128 == 0 {
                let mut yielded = false;
                futures_util::future::poll_fn(|cx| {
                    if yielded {
                        std::task::Poll::Ready(())
                    } else {
                        yielded = true;
                        cx.waker().wake_by_ref();
                        std::task::Poll::Pending
                    }
                })
                .await;
            }
            i += 1;
            let e = entry?;
            self.checked.fetch_add(1, Ordering::Relaxed);
            if !matches!(e.raw, RawEntry::Article(_))
                || classify_kind(e.raw.namespace(), e.mime_type.as_deref().unwrap_or(""))
                    != ZimEntryKind::Article
            {
                continue;
            }
            let score = score_title(&needle, &e.raw.title().to_lowercase());
            if score <= 0.0 {
                continue;
            }
            let hit = TitleSearchHit {
                article_key: ArticleKey(e.key.0),
                title: e.raw.title().into(),
                score,
            };
            let position = hits.partition_point(|h: &TitleSearchHit| {
                h.score > score || (h.score == score && h.title <= hit.title)
            });
            if position < capacity {
                hits.insert(position, hit);
                if hits.len() > capacity {
                    hits.pop();
                }
            }
        }
        Ok(hits.into_iter().skip(offset).take(limit).collect())
    }
    async fn entries(&self) -> Result<ZimEntryStream, ZimError> {
        let index = self.index.clone();
        let checked = self.checked.clone();
        Ok(Box::pin(async_stream::try_stream! {
            let mut entries = index.clone().entries().await?;
            while let Some(entry) = entries.next().await {
                let descriptor = Self::descriptor(index.as_ref(), entry?).await?;
                checked.fetch_add(1, Ordering::Relaxed);
                yield descriptor;
            }
        }))
    }
    async fn verify_structure(&self) -> Result<ZimVerificationReport, ZimError> {
        Ok(ZimVerificationReport {valid:true,engine_id:crate::ENGINE_ID.into(),engine_version:crate::ENGINE_VERSION.into(),checked_entries:self.checked.load(Ordering::Relaxed),warnings:vec![ZimVerificationFinding {code:"partial_metadata_validation".into(),entry_key:None,detail:"Only accessed metadata validated; checked_entries counts successful entry accesses, not distinct entries. Unread entries and payloads are not verified.".into()}],errors:vec![]})
    }
}
