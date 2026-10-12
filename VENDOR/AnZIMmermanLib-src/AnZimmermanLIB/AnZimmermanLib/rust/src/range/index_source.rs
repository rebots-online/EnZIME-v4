//! A canonical entry index may be complete and remote or a sparse offline catalog.
use super::{ClusterByteRange, IndexedEntry, RemoteZimIndex};
use crate::{
    directory::RawEntry,
    header::{ZimHeader, NO_PAGE},
    EntryKey, ZimError,
};
use async_trait::async_trait;
use futures_util::Stream;
use std::{collections::HashSet, pin::Pin, sync::Arc};

pub type IndexedEntryStream = Pin<Box<dyn Stream<Item = Result<IndexedEntry, ZimError>> + Send>>;

#[async_trait]
pub trait ZimEntryIndex: Send + Sync {
    fn header(&self) -> &ZimHeader;
    async fn entry_at(&self, index: u32) -> Result<IndexedEntry, ZimError>;
    async fn find_entry(&self, key: &EntryKey) -> Result<Option<IndexedEntry>, ZimError>;
    async fn cluster_range(&self, index: u32) -> Result<ClusterByteRange, ZimError>;
    async fn entries(self: Arc<Self>) -> Result<IndexedEntryStream, ZimError>;
    async fn resolve_entry(&self, key: &EntryKey, max_hops: u8) -> Result<IndexedEntry, ZimError> {
        let entry = self
            .find_entry(key)
            .await?
            .ok_or_else(|| ZimError::EntryNotFound(key.0.clone()))?;
        resolve_index(self, entry, max_hops).await
    }
    async fn main_entry(&self) -> Result<Option<IndexedEntry>, ZimError> {
        if self.header().main_page == NO_PAGE {
            return Ok(None);
        }
        let entry = self.entry_at(self.header().main_page).await?;
        resolve_index(self, entry, 16).await.map(Some)
    }
}

async fn resolve_index<T: ZimEntryIndex + ?Sized>(
    index: &T,
    mut entry: IndexedEntry,
    max_hops: u8,
) -> Result<IndexedEntry, ZimError> {
    let max_hops = max_hops.min(32);
    let mut visited = HashSet::new();
    for hop in 0..=max_hops {
        if entry.index >= index.header().entry_count {
            return Err(ZimError::EntryNotFound(format!(
                "entry index {}",
                entry.index
            )));
        }
        match &entry.raw {
            RawEntry::Redirect(redirect) => {
                if !visited.insert(entry.index) {
                    return Err(ZimError::RedirectCycle);
                }
                if hop == max_hops {
                    return Err(ZimError::UnsupportedFeature(
                        "redirect hop limit exceeded".into(),
                    ));
                }
                let target = redirect.redirect_index;
                if target >= index.header().entry_count {
                    return Err(ZimError::EntryNotFound(format!("entry index {target}")));
                }
                entry = index.entry_at(target).await?;
            }
            _ => return Ok(entry),
        }
    }
    unreachable!()
}

#[async_trait]
impl ZimEntryIndex for RemoteZimIndex {
    fn header(&self) -> &ZimHeader {
        RemoteZimIndex::header(self)
    }
    async fn entry_at(&self, index: u32) -> Result<IndexedEntry, ZimError> {
        RemoteZimIndex::entry_at(self, index).await
    }
    async fn find_entry(&self, key: &EntryKey) -> Result<Option<IndexedEntry>, ZimError> {
        RemoteZimIndex::find_entry(self, key).await
    }
    async fn cluster_range(&self, index: u32) -> Result<ClusterByteRange, ZimError> {
        RemoteZimIndex::cluster_range(self, index).await
    }
    async fn resolve_entry(&self, key: &EntryKey, max_hops: u8) -> Result<IndexedEntry, ZimError> {
        RemoteZimIndex::resolve_entry(self, key, max_hops).await
    }
    async fn main_entry(&self) -> Result<Option<IndexedEntry>, ZimError> {
        RemoteZimIndex::main_entry(self).await
    }
    async fn entries(self: Arc<Self>) -> Result<IndexedEntryStream, ZimError> {
        Ok(Box::pin(async_stream::try_stream! {
            for index in 0..self.header().entry_count { yield self.entry_at(index).await?; }
        }))
    }
}
