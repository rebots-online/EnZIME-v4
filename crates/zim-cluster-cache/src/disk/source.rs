use super::{DiskPackStore, SourceIdentity};
use anzimmerman::{model::ZimError, range::RandomAccessSource};
use async_trait::async_trait;
use bytes::Bytes;
use ports::transfer::*;
use std::sync::Arc;

pub struct PinnedTransferSource {
    inner: Arc<dyn TransferSource>,
    identity: TransferProbe,
}
impl PinnedTransferSource {
    pub fn new(inner: Arc<dyn TransferSource>, identity: &SourceIdentity) -> Result<Self, String> {
        if identity
            .etag
            .as_ref()
            .is_none_or(|e| !(e.len() >= 2 && e.starts_with('"') && e.ends_with('"')))
            && identity.last_modified.is_none()
        {
            return Err("source_identity_unavailable".into());
        }
        let url = url::Url::parse(&identity.uri).map_err(|_| "invalid_source_url")?;
        if !url.username().is_empty()
            || url.password().is_some()
            || !(url.scheme() == "https"
                || url.scheme() == "http" && url.host_str() == Some("127.0.0.1"))
        {
            return Err("insecure_source_url".into());
        }
        Ok(Self {
            inner,
            identity: TransferProbe {
                uri: identity.uri.clone(),
                size_bytes: identity.size_bytes,
                accepts_ranges: true,
                etag: identity.etag.clone(),
                last_modified: identity.last_modified.clone(),
                content_type: None,
            },
        })
    }
}
#[async_trait]
impl TransferSource for PinnedTransferSource {
    fn source_kind(&self) -> &'static str {
        "pinned.range.v1"
    }
    async fn probe(&self, request: TransferProbeRequest) -> Result<TransferProbe, TransferError> {
        let current = self.inner.probe(request).await?;
        let strong = self
            .identity
            .etag
            .as_deref()
            .filter(|e| e.len() >= 2 && e.starts_with('"') && e.ends_with('"'));
        if current.size_bytes != self.identity.size_bytes
            || !current.accepts_ranges
            || strong
                .map(|e| current.etag.as_deref() != Some(e))
                .unwrap_or_else(|| current.last_modified != self.identity.last_modified)
        {
            return Err(TransferError::SourceChanged);
        }
        Ok(current)
    }
    async fn open_range(
        &self,
        uri: &str,
        range: TransferRange,
        _: Option<&str>,
    ) -> Result<TransferByteStream, TransferError> {
        self.probe(TransferProbeRequest {
            uri: uri.into(),
            maximum_response_bytes: 80,
        })
        .await?;
        self.inner
            .open_pinned_range(uri, range, &self.identity)
            .await
    }
    async fn open_full(&self, _: &str, _: u64) -> Result<TransferByteStream, TransferError> {
        Err(TransferError::RangeUnsupported)
    }
}
pub struct DiskRangeSource {
    store: Arc<DiskPackStore>,
    pinned: bool,
    len: u64,
}
impl DiskRangeSource {
    pub fn new(store: Arc<DiskPackStore>) -> Self {
        let len = store.source_len;
        Self {
            store,
            pinned: true,
            len,
        }
    }
    pub fn new_for_planning(store: Arc<DiskPackStore>) -> Self {
        let len = store.source_len;
        Self {
            store,
            pinned: false,
            len,
        }
    }
}
#[async_trait]
impl RandomAccessSource for DiskRangeSource {
    fn len(&self) -> u64 {
        self.len
    }
    async fn read_range(&self, offset: u64, length: u64) -> Result<Bytes, ZimError> {
        if length > 1024 * 1024 || offset.checked_add(length).is_none_or(|n| n > self.len) {
            return Err(ZimError::Io("metadata range limit".into()));
        }
        let mut result = Vec::with_capacity(length as usize);
        let mut at = offset;
        while at < offset + length {
            let page = at / 65536;
            let bytes = self
                .store
                .read_page(page, self.pinned)
                .await
                .map_err(ZimError::Io)?;
            let start = (at % 65536) as usize;
            let count = (bytes.len() - start).min((offset + length - at) as usize);
            result.extend_from_slice(&bytes[start..start + count]);
            at += count as u64;
        }
        Ok(Bytes::from(result))
    }
}
