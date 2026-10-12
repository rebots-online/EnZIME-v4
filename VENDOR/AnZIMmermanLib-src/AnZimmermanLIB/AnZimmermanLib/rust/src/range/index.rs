use super::{PagedRangeReader, RandomAccessSource, RangeReadLimits};
use crate::{
    directory::{parse_entry, RawEntry},
    header::{cluster_end, parse_header, validate_header_offsets, ZimHeader, NO_PAGE},
    EntryKey, ZimError,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, sync::Arc};
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct ClusterByteRange {
    pub start: u64,
    pub length: u64,
}
#[derive(Clone, Debug)]
pub struct IndexedEntry {
    pub index: u32,
    pub key: EntryKey,
    pub raw: RawEntry,
    pub mime_type: Option<String>,
}
#[derive(Clone, Debug)]
pub struct CanonicalEntryRecord {
    pub entry: IndexedEntry,
    pub raw_bytes: bytes::Bytes,
}
pub struct RemoteZimIndex {
    reader: PagedRangeReader,
    header: ZimHeader,
    mimes: Vec<String>,
    limits: RangeReadLimits,
}
fn invalid(s: &str) -> ZimError {
    ZimError::InvalidHeader(s.into())
}
impl RemoteZimIndex {
    pub async fn open(
        source: Arc<dyn RandomAccessSource>,
        limits: RangeReadLimits,
    ) -> Result<Self, ZimError> {
        let reader = PagedRangeReader::new(source, limits.clone())?;
        let header = parse_header(&reader.read_exact(0, 80).await?)?;
        validate_header_offsets(&header, reader.len())?;
        for (offset, count, width) in [
            (header.path_ptr_pos, header.entry_count, 8u64),
            (header.cluster_ptr_pos, header.cluster_count, 8),
            (
                header.title_index_pos().unwrap_or(0),
                if header.title_index_pos().is_none() {
                    0
                } else {
                    header.entry_count
                },
                4,
            ),
        ] {
            if offset
                .checked_add(
                    (count as u64)
                        .checked_mul(width)
                        .ok_or_else(|| invalid("pointer size overflow"))?,
                )
                .is_none_or(|e| e > reader.len())
            {
                return Err(invalid("pointer table outside source"));
            }
        }
        if header.checksum_pos != 0
            && header
                .checksum_pos
                .checked_add(16)
                .is_none_or(|e| e > reader.len())
        {
            return Err(invalid("checksum outside source"));
        }
        let mut mimes = Vec::new();
        let mut value = Vec::new();
        let mut pos = header.mime_list_pos;
        let mut consumed = 0;
        let boundary = [
            header.path_ptr_pos,
            header.title_index_pos().unwrap_or(0),
            header.cluster_ptr_pos,
            header.checksum_pos,
            reader.len(),
        ]
        .into_iter()
        .filter(|n| *n > pos)
        .min()
        .unwrap_or(reader.len());
        // Read in bounded increments; parse only through the terminating empty MIME string.
        'mime: loop {
            let length = (boundary - pos)
                .min(256)
                .min((limits.max_mime_bytes - consumed) as u64) as usize;
            if length == 0 {
                return Err(invalid("unterminated MIME list"));
            }
            let bytes = reader.read_exact(pos, length).await?;
            for byte in bytes {
                consumed += 1;
                pos += 1;
                if byte == 0 {
                    if value.is_empty() {
                        break 'mime;
                    }
                    mimes.push(
                        String::from_utf8(std::mem::take(&mut value))
                            .map_err(|_| invalid("MIME is not UTF-8"))?,
                    );
                    if mimes.len() > crate::limits::MAX_MIME_TYPES as usize {
                        return Err(ZimError::AllocationLimit);
                    }
                } else {
                    value.push(byte);
                }
            }
        }
        Ok(Self {
            reader,
            header,
            mimes,
            limits,
        })
    }
    pub fn header(&self) -> &ZimHeader {
        &self.header
    }
    pub async fn cache_counts(&self) -> (usize, usize) {
        self.reader.cache_counts().await
    }
    async fn pointer(&self, offset: u64, index: u32) -> Result<u64, ZimError> {
        let pos = offset
            .checked_add(index as u64 * 8)
            .ok_or_else(|| invalid("pointer overflow"))?;
        let bytes = self.reader.read_exact(pos, 8).await?;
        Ok(u64::from_le_bytes(bytes[..].try_into().unwrap()))
    }
    pub async fn entry_at(&self, index: u32) -> Result<IndexedEntry, ZimError> {
        Ok(self.entry_record(index).await?.entry)
    }
    pub async fn entry_record(&self, index: u32) -> Result<CanonicalEntryRecord, ZimError> {
        if index >= self.header.entry_count {
            return Err(ZimError::EntryNotFound(format!("entry index {index}")));
        }
        let ptr = self.pointer(self.header.path_ptr_pos, index).await?;
        if ptr < 80 || ptr >= self.reader.len() {
            return Err(invalid("directory pointer outside source"));
        }
        let length = (self.reader.len() - ptr).min(self.limits.max_entry_bytes as u64) as usize;
        let bytes = self.reader.read_exact(ptr, length).await?;
        let (raw, consumed) = parse_entry(&bytes)?;
        let mime_type = match &raw {
            RawEntry::Article(a) => {
                if a.cluster_number >= self.header.cluster_count {
                    return Err(invalid("entry cluster index outside table"));
                }
                Some(
                    self.mimes
                        .get(a.mimetype_index as usize)
                        .ok_or_else(|| invalid("entry MIME index outside table"))?
                        .clone(),
                )
            }
            _ => None,
        };
        let key = EntryKey(crate::archive::entry_key_string(raw.namespace(), raw.url()));
        Ok(CanonicalEntryRecord {
            entry: IndexedEntry {
                index,
                key,
                raw,
                mime_type,
            },
            raw_bytes: bytes::Bytes::copy_from_slice(&bytes[..consumed]),
        })
    }
    pub async fn find_entry(&self, key: &EntryKey) -> Result<Option<IndexedEntry>, ZimError> {
        let (mut lo, mut hi) = (0, self.header.entry_count);
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            let e = self.entry_at(mid).await?;
            match e.key.0.as_bytes().cmp(key.0.as_bytes()) {
                std::cmp::Ordering::Less => lo = mid + 1,
                std::cmp::Ordering::Greater => hi = mid,
                std::cmp::Ordering::Equal => return Ok(Some(e)),
            }
        }
        Ok(None)
    }
    pub async fn cluster_range(&self, index: u32) -> Result<ClusterByteRange, ZimError> {
        if index >= self.header.cluster_count {
            return Err(ZimError::EntryNotFound(format!("cluster index {index}")));
        }
        let start = self.pointer(self.header.cluster_ptr_pos, index).await?;
        if start < 80 {
            return Err(invalid("cluster overlaps header"));
        }
        let next = if index + 1 < self.header.cluster_count {
            Some(self.pointer(self.header.cluster_ptr_pos, index + 1).await?)
        } else {
            None
        };
        let end = cluster_end(&self.header, start, next, self.reader.len())?;
        Ok(ClusterByteRange {
            start,
            length: end - start,
        })
    }
    pub async fn resolve_records(
        &self,
        index: u32,
        max_hops: u8,
    ) -> Result<Vec<CanonicalEntryRecord>, ZimError> {
        let max_hops = max_hops.min(self.limits.max_redirect_hops).min(32);
        let mut records = Vec::with_capacity(max_hops as usize + 1);
        let mut visited = HashSet::new();
        let mut index = index;
        for hop in 0..=max_hops {
            if !visited.insert(index) {
                return Err(ZimError::RedirectCycle);
            }
            let record = self.entry_record(index).await?;
            let target = match &record.entry.raw {
                RawEntry::Redirect(r) => Some(r.redirect_index),
                _ => None,
            };
            records.push(record);
            match target {
                None => return Ok(records),
                Some(_) if hop == max_hops => {
                    return Err(ZimError::UnsupportedFeature(
                        "redirect hop limit exceeded".into(),
                    ))
                }
                Some(next) => index = next,
            }
        }
        unreachable!()
    }
    async fn resolve_index(
        &self,
        entry: IndexedEntry,
        max_hops: u8,
    ) -> Result<IndexedEntry, ZimError> {
        Ok(self
            .resolve_records(entry.index, max_hops)
            .await?
            .pop()
            .unwrap()
            .entry)
    }
    pub async fn resolve_entry(
        &self,
        key: &EntryKey,
        max_hops: u8,
    ) -> Result<IndexedEntry, ZimError> {
        let entry = self
            .find_entry(key)
            .await?
            .ok_or_else(|| ZimError::EntryNotFound(key.0.clone()))?;
        self.resolve_index(entry, max_hops).await
    }
    pub async fn main_entry(&self) -> Result<Option<IndexedEntry>, ZimError> {
        if self.header.main_page == NO_PAGE {
            return Ok(None);
        }
        let entry = self.entry_at(self.header.main_page).await?;
        self.resolve_index(entry, self.limits.max_redirect_hops)
            .await
            .map(Some)
    }
}
