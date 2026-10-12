use super::ClusterByteRange;
use crate::ZimError;
use async_trait::async_trait;
use bytes::Bytes;
use futures_util::lock::Mutex;
use std::{collections::VecDeque, sync::Arc};
#[async_trait]
pub trait RandomAccessSource: Send + Sync {
    fn len(&self) -> u64;
    async fn read_range(&self, offset: u64, length: u64) -> Result<Bytes, ZimError>;
}
#[derive(Clone, Debug)]
pub struct RangeReadLimits {
    pub page_bytes: usize,
    pub cached_pages: usize,
    pub max_entry_bytes: usize,
    pub max_mime_bytes: usize,
    pub max_blob_bytes: u64,
    pub max_redirect_hops: u8,
}
impl Default for RangeReadLimits {
    fn default() -> Self {
        Self {
            page_bytes: 65536,
            cached_pages: 8,
            max_entry_bytes: crate::limits::MAX_ENTRY_READ_BYTES as usize,
            max_mime_bytes: 1048576,
            max_blob_bytes: 33554432,
            max_redirect_hops: 16,
        }
    }
}
impl RangeReadLimits {
    pub(crate) fn validate(&self) -> Result<(), ZimError> {
        if self.page_bytes == 0
            || self.page_bytes > 65536
            || self.cached_pages == 0
            || self.cached_pages > 16
            || self.max_entry_bytes == 0
            || self.max_entry_bytes > 65536
            || self.max_mime_bytes == 0
            || self.max_mime_bytes > 1048576
            || self.max_blob_bytes == 0
            || self.max_blob_bytes > 33554432
            || self.max_redirect_hops == 0
            || self.max_redirect_hops > 32
        {
            return Err(ZimError::AllocationLimit);
        }
        Ok(())
    }
}
#[derive(Default)]
struct PageCache {
    pages: VecDeque<(u64, Bytes)>,
    peak_pages: usize,
}
pub struct PagedRangeReader {
    source: Arc<dyn RandomAccessSource>,
    limits: RangeReadLimits,
    cache: Mutex<PageCache>,
}
impl PagedRangeReader {
    pub fn new(
        source: Arc<dyn RandomAccessSource>,
        limits: RangeReadLimits,
    ) -> Result<Self, ZimError> {
        limits.validate()?;
        Ok(Self {
            source,
            limits,
            cache: Mutex::new(PageCache::default()),
        })
    }
    pub fn len(&self) -> u64 {
        self.source.len()
    }
    /// Actual retained page counts for bounded-cache diagnostics.
    pub async fn cache_counts(&self) -> (usize, usize) {
        let c = self.cache.lock().await;
        (c.pages.len(), c.peak_pages)
    }
    pub async fn read_exact(&self, offset: u64, length: usize) -> Result<Bytes, ZimError> {
        if length
            > self
                .limits
                .max_entry_bytes
                .max(self.limits.max_mime_bytes)
                .max(self.limits.page_bytes)
        {
            return Err(ZimError::AllocationLimit);
        }
        let end = offset
            .checked_add(length as u64)
            .filter(|e| *e <= self.len())
            .ok_or_else(|| ZimError::InvalidHeader("range outside source".into()))?;
        let mut out = Vec::with_capacity(length);
        let mut cursor = offset;
        while cursor < end {
            let page = cursor / self.limits.page_bytes as u64;
            let start = page * self.limits.page_bytes as u64;
            // Hold the async lock across a miss: concurrent callers cannot grow the cache or duplicate a fetch.
            let mut cache = self.cache.lock().await;
            let bytes = if let Some(pos) = cache.pages.iter().position(|(p, _)| *p == page) {
                cache.pages.remove(pos).unwrap().1
            } else {
                if cache.pages.len() == self.limits.cached_pages {
                    cache.pages.pop_front();
                }
                let size = (self.len() - start).min(self.limits.page_bytes as u64);
                let bytes = self.source.read_range(start, size).await?;
                if bytes.len() as u64 != size {
                    return Err(ZimError::Io("short or excess range response".into()));
                }
                bytes
            };
            let within = (cursor - start) as usize;
            let take = ((end - cursor) as usize).min(bytes.len() - within);
            out.extend_from_slice(&bytes[within..within + take]);
            cursor += take as u64;
            cache.pages.push_back((page, bytes));
            cache.peak_pages = cache.peak_pages.max(cache.pages.len());
        }
        Ok(Bytes::from(out))
    }
}
#[async_trait]
pub trait ClusterBlobSource: Send + Sync {
    async fn read_blob(
        &self,
        cluster_index: u32,
        range: ClusterByteRange,
        blob_index: u32,
        maximum_bytes: u64,
    ) -> Result<Bytes, ZimError>;
}
