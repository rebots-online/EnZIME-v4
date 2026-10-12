//! Application-port adapter for injected canonical range archives.
use crate::convert;
use async_trait::async_trait;
use domain::identity::{ArticleKey, EntryKey};
use futures_util::StreamExt;
use ports::zim::*;
use std::sync::Arc;
pub struct RangeArchiveAdapter {
    inner: Arc<dyn anzimmerman::model::ZimArchive>,
    metadata: ZimMetadata,
}
impl RangeArchiveAdapter {
    pub fn new(inner: Arc<dyn anzimmerman::model::ZimArchive>) -> Self {
        let metadata = convert::metadata(inner.metadata().clone());
        Self { inner, metadata }
    }
}
#[async_trait]
impl ZimArchive for RangeArchiveAdapter {
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
