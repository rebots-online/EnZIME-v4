//! `AnZimmermanZimArchive` and its `ZimArchive` trait impl, plus the open-time
//! pipeline that turns a `ZimOpenRequest` into a fully indexed in-memory
//! archive handle.
//!
//! Design notes:
//!
//! * All on-disk reads go through one `std::fs::File` guarded by a `Mutex`.
//!   Async trait methods perform synchronous reads inline. For local page
//!   cache hits this is fast; for cold reads it is no worse than any
//!   `spawn_blocking` strategy and avoids the runtime dependency.
//! * `Inner` lives behind an `Arc` so the `entries` stream can capture a
//!   strong reference and yield without borrowing from `&self`.
//! * The most recently decompressed cluster is cached behind a separate
//!   `Mutex` so that batch iteration over entries in the same cluster
//!   avoids redundant decompression work.

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use bytes::Bytes;
use domain::identity::{ArticleKey, ContentDigest, DigestAlgorithm, EntryKey};
use ports::zim::{
    ArticleDocument, ResourceDocument, TitleSearchHit, TitleSearchQuery, ZimArchive,
    ZimEntryDescriptor, ZimEntryKind, ZimEntryStream, ZimError, ZimMetadata, ZimOpenRequest,
    ZimVerificationFinding, ZimVerificationReport,
};
use sha2::{Digest as _, Sha256};

use crate::cluster::{decompress_cluster, extract_blob};
use crate::directory::{parse_entry, ArticleEntry, RawEntry};
use crate::header::{validate_header_offsets, NO_PAGE, ZIM_MAGIC};
use crate::header::{parse_header, ZimHeader};
use crate::limits::{
    HASH_CHUNK_BYTES, MAX_COMPRESSED_CLUSTER_BYTES, MAX_ENTRY_READ_BYTES, MAX_INDEX_ALLOCATION_BYTES,
    MAX_MIME_TYPES, MIN_ZIM_FILE_BYTES,
};
use crate::sanitize;

pub(crate) fn entry_key_string(namespace: u8, url: &str) -> String {
    let ns = if namespace.is_ascii() {
        namespace as char
    } else {
        '?'
    };
    format!("{}/{}", ns, url)
}

/// Open a ZIM archive as the concrete [`AnZimmermanZimArchive`].
///
/// Exposed (rather than `pub(crate)`) so the integration layer can call
/// [`AnZimmermanZimArchive::layout_snapshot`] without going through the
/// `ZimArchive` trait object (which has no layout accessor). Callers that
/// only need the trait surface should still use
/// [`crate::AnZimmermanZimEngine::open`].
pub async fn open_archive(
    request: &ZimOpenRequest,
) -> Result<AnZimmermanZimArchive, ZimError> {
    let inner = Inner::open(request)?;
    Ok(AnZimmermanZimArchive {
        inner: Arc::new(inner),
    })
}

pub struct AnZimmermanZimArchive {
    inner: Arc<Inner>,
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
    /// Build a layout snapshot by walking the path-pointer table and the
    /// cluster-pointer table once each. No cluster payloads are read.
    ///
    /// This is the single integration point the reader uses to materialize a
    /// DynDon-subset pack: it yields everything both the planner (cluster
    /// byte ranges) and the cluster-cache reader (per-article cluster/blob
    /// indices) need.
    pub fn layout_snapshot(&self) -> Result<ZimLayoutSnapshot, ZimError> {
        let cluster_count = self.inner.cluster_ptrs.len() as u32;
        let mut cluster_byte_ranges = Vec::with_capacity(cluster_count as usize);
        for idx in 0..cluster_count {
            let (start, end) = self.inner.cluster_byte_range(idx)?;
            let len = end.checked_sub(start).ok_or_else(|| {
                ZimError::InvalidHeader(format!(
                    "cluster {} byte range end {} precedes start {}",
                    idx, end, start
                ))
            })?;
            cluster_byte_ranges.push((start, len));
        }

        let entry_count = self.inner.path_ptrs.len() as u32;
        let mut entries = Vec::with_capacity(entry_count as usize);
        for idx in 0..entry_count {
            let raw = self.inner.entry_at(idx)?;
            let namespace_byte = raw.namespace();
            let ns_char = if namespace_byte.is_ascii() {
                namespace_byte as char
            } else {
                '?'
            };
            let entry_key = entry_key_string(namespace_byte, raw.url());
            let path = raw.url().to_string();
            let title = raw.title().to_string();
            match raw {
                RawEntry::Article(a) => {
                    let mime = self.inner.mime_for(a.mimetype_index)?.to_string();
                    let kind = classify_kind(a.namespace, &mime);
                    entries.push(ZimLayoutEntry {
                        entry_key,
                        namespace: ns_char,
                        path,
                        title,
                        mime_type: mime,
                        kind,
                        redirect_target: None,
                        cluster_index: a.cluster_number,
                        blob_index: a.blob_number,
                    });
                }
                RawEntry::Redirect(r) => {
                    let redirect_target = self.inner.redirect_target_key(r.redirect_index)?;
                    entries.push(ZimLayoutEntry {
                        entry_key,
                        namespace: ns_char,
                        path,
                        title,
                        mime_type: String::new(),
                        kind: ZimEntryKind::Redirect,
                        redirect_target,
                        // Redirects have no cluster payload of their own.
                        cluster_index: 0,
                        blob_index: 0,
                    });
                }
                RawEntry::Linktarget(_) => {
                    entries.push(ZimLayoutEntry {
                        entry_key,
                        namespace: ns_char,
                        path,
                        title,
                        mime_type: String::new(),
                        kind: ZimEntryKind::Metadata,
                        redirect_target: None,
                        cluster_index: 0,
                        blob_index: 0,
                    });
                }
                RawEntry::Deleted(_) => {
                    entries.push(ZimLayoutEntry {
                        entry_key,
                        namespace: ns_char,
                        path,
                        title,
                        mime_type: String::new(),
                        kind: ZimEntryKind::Metadata,
                        redirect_target: None,
                        cluster_index: 0,
                        blob_index: 0,
                    });
                }
            }
        }

        Ok(ZimLayoutSnapshot {
            cluster_byte_ranges,
            entries,
        })
    }
}

struct Inner {
    file: Mutex<File>,
    size: u64,
    header: ZimHeader,
    mime_types: Vec<String>,
    path_ptrs: Vec<u64>,
    cluster_ptrs: Vec<u64>,
    metadata: ZimMetadata,
    path_index: HashMap<String, u32>,
    cluster_cache: Mutex<ClusterCache>,
}

#[derive(Default)]
struct ClusterCache {
    index: Option<u32>,
    decompressed: Option<Arc<Vec<u8>>>,
}

impl Inner {
    fn open(request: &ZimOpenRequest) -> Result<Self, ZimError> {
        let path = Path::new(&request.path);
        let mut file =
            File::open(path).map_err(|e| map_open_err(e, &request.path))?;
        let metadata = file.metadata().map_err(io_err)?;
        let size = metadata.len();
        if size < MIN_ZIM_FILE_BYTES {
            return Err(ZimError::InvalidHeader(format!(
                "file is {} bytes, need at least {}",
                size, MIN_ZIM_FILE_BYTES
            )));
        }
        if !metadata.is_file() {
            return Err(ZimError::UnreadablePath);
        }
        if let Some(expected_size) = request.expected_size_bytes {
            if size != expected_size {
                return Err(ZimError::LengthMismatch);
            }
        }
        if let Some(expected_hex) = request.expected_sha256.as_ref() {
            verify_sha256(&mut file, size, expected_hex)?;
        }
        file.seek(SeekFrom::Start(0)).map_err(io_err)?;

        let mut header_buf = [0u8; 80];
        file.read_exact(&mut header_buf).map_err(io_err)?;
        let header = parse_header(&header_buf)?;
        validate_header_offsets(&header, size)?;

        let mime_types = read_mime_list(&mut file, header.mime_list_pos, size)?;
        let path_ptrs = read_u64_table(
            &mut file,
            header.path_ptr_pos,
            header.entry_count,
            "path pointer",
        )?;
        let cluster_ptrs = read_u64_table(
            &mut file,
            header.cluster_ptr_pos,
            header.cluster_count,
            "cluster pointer",
        )?;
        let path_index = build_path_index(&mut file, &path_ptrs, size)?;

        let mut inner = Inner {
            file: Mutex::new(file),
            size,
            header,
            mime_types,
            path_ptrs,
            cluster_ptrs,
            metadata: ZimMetadata {
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
            },
            path_index,
            cluster_cache: Mutex::new(ClusterCache::default()),
        };
        inner.populate_metadata()?;
        Ok(inner)
    }

    fn populate_metadata(&mut self) -> Result<(), ZimError> {
        let uuid_hex: String = self
            .header
            .uuid
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect();
        let title = self.metadata_text("Title")?.unwrap_or_default();
        let description = self.metadata_text("Description")?.unwrap_or_default();
        let language = self.metadata_text("Language")?.unwrap_or_default();
        let creator = self.metadata_text("Creator")?.unwrap_or_default();
        let publisher = self.metadata_text("Publisher")?.unwrap_or_default();
        let date = self.metadata_text("Date")?.unwrap_or_default();
        let counter = self.metadata_text("Counter")?.unwrap_or_default();
        let (article_count, media_count) = parse_counter(&counter, self.header.entry_count);
        let main_page = if self.header.main_page == NO_PAGE {
            None
        } else {
            self.main_page_key(self.header.main_page)?
        };
        let checksum = self.read_checksum()?;
        self.metadata = ZimMetadata {
            uuid: uuid_hex,
            title,
            description,
            language_bcp47: language,
            creator,
            publisher,
            date,
            article_count,
            media_count,
            main_page,
            checksum,
        };
        Ok(())
    }

    fn metadata_text(&self, name: &str) -> Result<Option<String>, ZimError> {
        let key = format!("M/{}", name);
        let idx = match self.path_index.get(&key) {
            Some(i) => *i,
            None => return Ok(None),
        };
        let raw = self.entry_at(idx)?;
        let article = match raw {
            RawEntry::Article(a) => a,
            _ => return Ok(None),
        };
        let bytes = self.blob_bytes(&article)?;
        let text = String::from_utf8(bytes.to_vec()).map_err(|_| {
            ZimError::InvalidHeader(format!("metadata M/{} is not valid UTF-8", name))
        })?;
        Ok(Some(text.trim().to_string()))
    }

    fn main_page_key(&self, idx: u32) -> Result<Option<ArticleKey>, ZimError> {
        if idx as usize >= self.path_ptrs.len() {
            return Ok(None);
        }
        let raw = self.entry_at(idx)?;
        Ok(Some(ArticleKey(entry_key_string(
            raw.namespace(),
            raw.url(),
        ))))
    }

    fn read_checksum(&self) -> Result<Option<String>, ZimError> {
        let pos = self.header.checksum_pos;
        if pos == 0 || pos + 16 > self.size {
            return Ok(None);
        }
        let mut guard = self
            .file
            .lock()
            .map_err(|_| ZimError::Io("file mutex poisoned".into()))?;
        guard.seek(SeekFrom::Start(pos)).map_err(io_err)?;
        let mut buf = [0u8; 16];
        guard.read_exact(&mut buf).map_err(io_err)?;
        drop(guard);
        let hex: String = buf.iter().map(|b| format!("{:02x}", b)).collect();
        Ok(Some(hex))
    }

    fn entry_at(&self, idx: u32) -> Result<RawEntry, ZimError> {
        let ptr = *self.path_ptrs.get(idx as usize).ok_or_else(|| {
            ZimError::EntryNotFound(format!("entry index {} out of range", idx))
        })?;
        self.read_entry_at_ptr(ptr)
    }

    fn read_entry_at_ptr(&self, ptr: u64) -> Result<RawEntry, ZimError> {
        if ptr >= self.size {
            return Err(ZimError::InvalidHeader(format!(
                "directory entry pointer {} >= file size {}",
                ptr, self.size
            )));
        }
        let read_len = ((self.size - ptr).min(MAX_ENTRY_READ_BYTES)) as usize;
        let mut guard = self
            .file
            .lock()
            .map_err(|_| ZimError::Io("file mutex poisoned".into()))?;
        guard.seek(SeekFrom::Start(ptr)).map_err(io_err)?;
        let mut buf = vec![0u8; read_len];
        let n = guard.read(&mut buf).map_err(io_err)?;
        drop(guard);
        buf.truncate(n);
        let (entry, _consumed) = parse_entry(&buf)?;
        Ok(entry)
    }

    fn find_entry_idx(&self, key: &str) -> Result<u32, ZimError> {
        if let Some(&i) = self.path_index.get(key) {
            return Ok(i);
        }
        Err(ZimError::EntryNotFound(key.to_string()))
    }

    fn find_article_idx(&self, key: &str) -> Result<u32, ZimError> {
        if key.starts_with("A/") {
            return self.find_entry_idx(key);
        }
        let prefixed = format!("A/{}", key);
        if let Ok(i) = self.find_entry_idx(&prefixed) {
            return Ok(i);
        }
        self.find_entry_idx(key)
    }

    fn mime_for(&self, idx: u16) -> Result<&str, ZimError> {
        self.mime_types
            .get(idx as usize)
            .map(|s| s.as_str())
            .ok_or_else(|| {
                ZimError::InvalidHeader(format!("mimetype index {} out of range", idx))
            })
    }

    fn cluster_byte_range(&self, cluster_idx: u32) -> Result<(u64, u64), ZimError> {
        let start = *self.cluster_ptrs.get(cluster_idx as usize).ok_or_else(|| {
            ZimError::EntryNotFound(format!("cluster index {} out of range", cluster_idx))
        })?;
        let end = if (cluster_idx as usize + 1) < self.cluster_ptrs.len() {
            self.cluster_ptrs[cluster_idx as usize + 1]
        } else if self.header.checksum_pos > start && self.header.checksum_pos <= self.size {
            self.header.checksum_pos
        } else {
            self.size
        };
        if start >= end || end > self.size {
            return Err(ZimError::InvalidHeader(format!(
                "cluster {} has invalid byte range [{}, {} > {})",
                cluster_idx, start, end, self.size
            )));
        }
        let span = end - start;
        if span > MAX_COMPRESSED_CLUSTER_BYTES {
            return Err(ZimError::AllocationLimit);
        }
        Ok((start, end))
    }

    fn read_cluster_payload(&self, cluster_idx: u32) -> Result<(u8, Vec<u8>), ZimError> {
        let (start, end) = self.cluster_byte_range(cluster_idx)?;
        let span = (end - start) as usize;
        if span < 1 {
            return Err(ZimError::InvalidHeader(format!(
                "cluster {} is too short to contain a compression byte",
                cluster_idx
            )));
        }
        let mut guard = self
            .file
            .lock()
            .map_err(|_| ZimError::Io("file mutex poisoned".into()))?;
        guard.seek(SeekFrom::Start(start)).map_err(io_err)?;
        let mut buf = vec![0u8; span];
        guard.read_exact(&mut buf).map_err(io_err)?;
        drop(guard);
        let compression = buf[0];
        let payload = buf[1..].to_vec();
        Ok((compression, payload))
    }

    fn get_or_decompress_cluster(&self, cluster_idx: u32) -> Result<Arc<Vec<u8>>, ZimError> {
        if let Some(cached) = self.cluster_cache_lock_match(cluster_idx) {
            return Ok(cached);
        }
        let (compression, payload) = self.read_cluster_payload(cluster_idx)?;
        let decompressed = Arc::new(decompress_cluster(compression, &payload)?);
        let mut guard = self
            .cluster_cache
            .lock()
            .map_err(|_| ZimError::Io("cluster cache mutex poisoned".into()))?;
        guard.index = Some(cluster_idx);
        guard.decompressed = Some(decompressed.clone());
        Ok(decompressed)
    }

    fn cluster_cache_lock_match(&self, cluster_idx: u32) -> Option<Arc<Vec<u8>>> {
        let guard = self.cluster_cache.lock().ok()?;
        if guard.index == Some(cluster_idx) {
            guard.decompressed.clone()
        } else {
            None
        }
    }

    fn blob_bytes(&self, entry: &ArticleEntry) -> Result<Bytes, ZimError> {
        let cluster = self.get_or_decompress_cluster(entry.cluster_number)?;
        let blob = extract_blob(&cluster, entry.blob_number)?;
        Ok(Bytes::copy_from_slice(blob))
    }

    fn build_entry_descriptor(
        &self,
        _idx: u32,
        raw: &RawEntry,
    ) -> Result<ZimEntryDescriptor, ZimError> {
        let entry_key =
            EntryKey(entry_key_string(raw.namespace(), raw.url()));
        let ns_char = if raw.namespace().is_ascii() {
            raw.namespace() as char
        } else {
            '?'
        };
        let path = raw.url().to_string();
        let title = raw.title().to_string();
        match raw {
            RawEntry::Article(a) => {
                let mime = self.mime_for(a.mimetype_index)?.to_string();
                let kind = classify_kind(a.namespace, &mime);
                let byte_length = self.blob_len(a).unwrap_or(0);
                Ok(ZimEntryDescriptor {
                    entry_key,
                    namespace: ns_char,
                    path,
                    title,
                    mime_type: mime,
                    kind,
                    redirect_target: None,
                    byte_length,
                })
            }
            RawEntry::Redirect(r) => {
                let redirect_target =
                    self.redirect_target_key(r.redirect_index)?;
                Ok(ZimEntryDescriptor {
                    entry_key,
                    namespace: ns_char,
                    path,
                    title,
                    mime_type: String::new(),
                    kind: ZimEntryKind::Redirect,
                    redirect_target,
                    byte_length: 0,
                })
            }
            RawEntry::Linktarget(_) => Ok(ZimEntryDescriptor {
                entry_key,
                namespace: ns_char,
                path,
                title,
                mime_type: String::new(),
                kind: ZimEntryKind::Metadata,
                redirect_target: None,
                byte_length: 0,
            }),
            RawEntry::Deleted(_) => Ok(ZimEntryDescriptor {
                entry_key,
                namespace: ns_char,
                path,
                title,
                mime_type: String::new(),
                kind: ZimEntryKind::Metadata,
                redirect_target: None,
                byte_length: 0,
            }),
        }
    }

    fn redirect_target_key(&self, target_idx: u32) -> Result<Option<EntryKey>, ZimError> {
        if (target_idx as usize) >= self.path_ptrs.len() {
            return Ok(None);
        }
        let raw = self.entry_at(target_idx)?;
        Ok(Some(EntryKey(entry_key_string(
            raw.namespace(),
            raw.url(),
        ))))
    }

    fn blob_len(&self, entry: &ArticleEntry) -> Result<u64, ZimError> {
        let cluster = self.get_or_decompress_cluster(entry.cluster_number)?;
        let blob = extract_blob(&cluster, entry.blob_number)?;
        Ok(blob.len() as u64)
    }
}

fn classify_kind(namespace: u8, mime: &str) -> ZimEntryKind {
    if namespace == b'M' {
        ZimEntryKind::Metadata
    } else if namespace == b'A' || mime.starts_with("text/html") {
        ZimEntryKind::Article
    } else {
        ZimEntryKind::Resource
    }
}

#[async_trait]
impl ZimArchive for AnZimmermanZimArchive {
    fn metadata(&self) -> &ZimMetadata {
        &self.inner.metadata
    }

    async fn main_page(&self) -> Result<Option<ArticleKey>, ZimError> {
        Ok(self.inner.metadata.main_page.clone())
    }

    async fn entry(&self, key: &EntryKey) -> Result<ZimEntryDescriptor, ZimError> {
        let idx = self.inner.find_entry_idx(&key.0)?;
        let raw = self.inner.entry_at(idx)?;
        self.inner.build_entry_descriptor(idx, &raw)
    }

    async fn article(&self, key: &ArticleKey) -> Result<ArticleDocument, ZimError> {
        let idx = self.inner.find_article_idx(&key.0)?;
        let raw = self.inner.entry_at(idx)?;
        let entry = match raw {
            RawEntry::Article(a) => a,
            _ => {
                return Err(ZimError::EntryNotFound(format!(
                    "entry under {} is not a readable article",
                    key.0
                )));
            }
        };
        let mime = self.inner.mime_for(entry.mimetype_index)?.to_string();
        if !matches!(
            classify_kind(entry.namespace, &mime),
            ZimEntryKind::Article
        ) {
            return Err(ZimError::EntryNotFound(format!(
                "entry {} is not in the article namespace",
                key.0
            )));
        }
        let raw_html = self.inner.blob_bytes(&entry)?;
        let content_digest = sha256_digest(&raw_html);
        let projection = sanitize::project(&raw_html);
        Ok(ArticleDocument {
            article_key: ArticleKey(entry_key_string(entry.namespace, &entry.url)),
            title: entry.title.clone(),
            mime_type: mime,
            raw_html,
            normalized_text: projection.normalized_text,
            content_digest,
            internal_links: projection.internal_links,
        })
    }

    async fn resource(&self, key: &EntryKey) -> Result<ResourceDocument, ZimError> {
        let idx = self.inner.find_entry_idx(&key.0)?;
        let raw = self.inner.entry_at(idx)?;
        let entry = match raw {
            RawEntry::Article(a) => a,
            _ => {
                return Err(ZimError::EntryNotFound(format!(
                    "entry {} is not a readable blob",
                    key.0
                )));
            }
        };
        let bytes = self.inner.blob_bytes(&entry)?;
        let mime = self.inner.mime_for(entry.mimetype_index)?.to_string();
        let content_digest = sha256_digest(&bytes);
        Ok(ResourceDocument {
            entry_key: EntryKey(entry_key_string(entry.namespace, &entry.url)),
            mime_type: mime,
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
        let mut visited: HashSet<String> = HashSet::new();
        for _ in 0..=max_hops {
            if !visited.insert(current.clone()) {
                return Err(ZimError::RedirectCycle);
            }
            let idx = self.inner.find_entry_idx(&current)?;
            let raw = self.inner.entry_at(idx)?;
            match raw {
                RawEntry::Redirect(r) => {
                    if (r.redirect_index as usize) >= self.inner.path_ptrs.len() {
                        return Err(ZimError::EntryNotFound(format!(
                            "redirect target index {} out of range",
                            r.redirect_index
                        )));
                    }
                    let target = self.inner.entry_at(r.redirect_index)?;
                    current =
                        entry_key_string(target.namespace(), target.url());
                }
                _ => return Ok(EntryKey(current)),
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
        for idx in 0..(self.inner.path_ptrs.len() as u32) {
            let raw = match self.inner.entry_at(idx) {
                Ok(r) => r,
                Err(_) => continue,
            };
            let is_article = matches!(
                raw,
                RawEntry::Article(ref a) if a.namespace == b'A'
            );
            if !is_article {
                continue;
            }
            let title = raw.title();
            if title.is_empty() {
                continue;
            }
            let score = score_title(&needle, &title.to_lowercase());
            if score > 0.0 {
                hits.push(TitleSearchHit {
                    article_key: ArticleKey(entry_key_string(
                        raw.namespace(),
                        raw.url(),
                    )),
                    title: title.to_string(),
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
        let inner = self.inner.clone();
        let total = inner.path_ptrs.len() as u32;
        let stream = async_stream::stream! {
            for idx in 0..total {
                let raw = match inner.entry_at(idx) {
                    Ok(r) => r,
                    Err(e) => {
                        yield Err(e);
                        return;
                    }
                };
                match inner.build_entry_descriptor(idx, &raw) {
                    Ok(desc) => yield Ok(desc),
                    Err(e) => {
                        yield Err(e);
                        return;
                    }
                }
            }
        };
        Ok(Box::pin(stream))
    }

    async fn verify_structure(&self) -> Result<ZimVerificationReport, ZimError> {
        let mut warnings: Vec<ZimVerificationFinding> = Vec::new();
        let mut errors: Vec<ZimVerificationFinding> = Vec::new();
        let mut checked: u64 = 0;
        for (idx, &ptr) in self.inner.path_ptrs.iter().enumerate() {
            if ptr >= self.inner.size {
                errors.push(ZimVerificationFinding {
                    code: "path_ptr_out_of_range".into(),
                    entry_key: Some(EntryKey(format!("index:{}", idx))),
                    detail: format!(
                        "path pointer {} >= file size {}",
                        ptr, self.inner.size
                    ),
                });
                continue;
            }
            match self.inner.entry_at(idx as u32) {
                Ok(raw) => {
                    checked += 1;
                    if let RawEntry::Article(a) = raw {
                        if (a.cluster_number as usize) >= self.inner.cluster_ptrs.len() {
                            errors.push(ZimVerificationFinding {
                                code: "cluster_index_out_of_range".into(),
                                entry_key: Some(EntryKey(entry_key_string(
                                    a.namespace,
                                    &a.url,
                                ))),
                                detail: format!(
                                    "entry points to cluster {} but only {} clusters exist",
                                    a.cluster_number,
                                    self.inner.cluster_ptrs.len()
                                ),
                            });
                        }
                        if let Ok(mime) = self.inner.mime_for(a.mimetype_index) {
                            // ok
                            let _ = mime;
                        } else {
                            warnings.push(ZimVerificationFinding {
                                code: "mimetype_index_out_of_range".into(),
                                entry_key: Some(EntryKey(entry_key_string(
                                    a.namespace,
                                    &a.url,
                                ))),
                                detail: format!(
                                    "entry has mimetype_index {} but only {} types exist",
                                    a.mimetype_index,
                                    self.inner.mime_types.len()
                                ),
                            });
                        }
                    }
                }
                Err(e) => {
                    errors.push(ZimVerificationFinding {
                        code: "entry_unreadable".into(),
                        entry_key: Some(EntryKey(format!("index:{}", idx))),
                        detail: format!("entry at index {} unreadable: {}", idx, e),
                    });
                }
            }
        }
        for (idx, &ptr) in self.inner.cluster_ptrs.iter().enumerate() {
            if ptr >= self.inner.size {
                errors.push(ZimVerificationFinding {
                    code: "cluster_ptr_out_of_range".into(),
                    entry_key: Some(EntryKey(format!("cluster:{}", idx))),
                    detail: format!(
                        "cluster pointer {} >= file size {}",
                        ptr, self.inner.size
                    ),
                });
            }
        }
        if self.inner.header.magic != ZIM_MAGIC {
            errors.push(ZimVerificationFinding {
                code: "magic_mismatch".into(),
                entry_key: None,
                detail: format!("magic 0x{:08X}", self.inner.header.magic),
            });
        }
        let valid = errors.is_empty();
        Ok(ZimVerificationReport {
            valid,
            engine_id: crate::ENGINE_ID.to_string(),
            engine_version: crate::ENGINE_VERSION.to_string(),
            checked_entries: checked,
            warnings,
            errors,
        })
    }
}

fn score_title(needle: &str, haystack_lower: &str) -> f32 {
    if haystack_lower == needle {
        return 1.0;
    }
    if haystack_lower.starts_with(needle) {
        return 0.9;
    }
    let mut prev = ' ';
    for (i, c) in haystack_lower.char_indices() {
        if !prev.is_alphanumeric()
            && haystack_lower[i..].starts_with(needle)
        {
            return 0.7;
        }
        prev = c;
    }
    if haystack_lower.contains(needle) {
        return 0.4;
    }
    0.0
}

fn parse_counter(counter_text: &str, header_entry_count: u32) -> (u64, u64) {
    let mut article: u64 = 0;
    let mut media: u64 = 0;
    let mut saw_pair = false;
    for raw_piece in counter_text.split(|c: char| c == ';' || c == ',') {
        let piece = raw_piece.trim();
        if piece.is_empty() {
            continue;
        }
        if let Some((k, v)) = piece.split_once('=') {
            let k = k.trim();
            let v = v.trim();
            if let Ok(n) = v.parse::<u64>() {
                saw_pair = true;
                if k.eq_ignore_ascii_case("article") {
                    article = article.saturating_add(n);
                } else {
                    media = media.saturating_add(n);
                }
            }
        } else if let Ok(n) = piece.parse::<u64>() {
            article = article.saturating_add(n);
        }
    }
    if !saw_pair && article == 0 {
        article = header_entry_count as u64;
    }
    (article, media)
}

fn sha256_digest(bytes: &Bytes) -> ContentDigest {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let hash = hasher.finalize();
    ContentDigest {
        algorithm: DigestAlgorithm::Sha256,
        hex: hex::encode(hash),
    }
}

fn verify_sha256(file: &mut File, size: u64, expected_hex: &str) -> Result<(), ZimError> {
    let expected = decode_sha256_hex(expected_hex)?;
    file.seek(SeekFrom::Start(0)).map_err(io_err)?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; HASH_CHUNK_BYTES];
    let mut remaining = size;
    while remaining > 0 {
        let to_read = remaining.min(buf.len() as u64) as usize;
        let n = file.read(&mut buf[..to_read]).map_err(io_err)?;
        if n == 0 {
            return Err(ZimError::Io(
                "unexpected EOF during SHA-256 verification".into(),
            ));
        }
        hasher.update(&buf[..n]);
        remaining -= n as u64;
    }
    let hash = hasher.finalize();
    if hash.as_slice() != &expected[..] {
        return Err(ZimError::DigestMismatch);
    }
    Ok(())
}

fn decode_sha256_hex(hex_in: &str) -> Result<[u8; 32], ZimError> {
    let hex = hex_in.trim().to_ascii_lowercase();
    if hex.len() != 64 {
        return Err(ZimError::InvalidHeader(format!(
            "expected_sha256 must be 64 hex characters, got {}",
            hex.len()
        )));
    }
    let bytes = hex::decode(&hex).map_err(|_| {
        ZimError::InvalidHeader("expected_sha256 is not valid hex".into())
    })?;
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

fn read_mime_list(file: &mut File, offset: u64, file_size: u64) -> Result<Vec<String>, ZimError> {
    file.seek(SeekFrom::Start(offset)).map_err(io_err)?;
    let mut bytes: Vec<u8> = Vec::new();
    let mut chunk = [0u8; 4096];
    let mut pos = offset;
    loop {
        if pos >= file_size {
            return Err(ZimError::InvalidHeader(
                "MIME list missing double-NUL terminator".into(),
            ));
        }
        let n = file.read(&mut chunk).map_err(io_err)?;
        if n == 0 {
            return Err(ZimError::InvalidHeader(
                "unexpected EOF in MIME list".into(),
            ));
        }
        bytes.extend_from_slice(&chunk[..n]);
        if let Some(idx) = find_subslice(&bytes, b"\0\0") {
            bytes.truncate(idx);
            break;
        }
        if bytes.len() as u64 > MAX_INDEX_ALLOCATION_BYTES {
            return Err(ZimError::AllocationLimit);
        }
        pos += n as u64;
    }
    let mut mimes = Vec::new();
    for piece in bytes.split(|&b| b == 0) {
        if piece.is_empty() {
            continue;
        }
        let s = std::str::from_utf8(piece).map_err(|_| {
            ZimError::InvalidHeader("MIME type is not valid UTF-8".into())
        })?;
        mimes.push(s.to_string());
    }
    if mimes.len() as u32 > MAX_MIME_TYPES {
        return Err(ZimError::InvalidHeader(format!(
            "MIME list declares {} entries, max {}",
            mimes.len(),
            MAX_MIME_TYPES
        )));
    }
    Ok(mimes)
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

fn read_u64_table(
    file: &mut File,
    offset: u64,
    count: u32,
    label: &str,
) -> Result<Vec<u64>, ZimError> {
    let byte_count = (count as u64)
        .checked_mul(8)
        .ok_or_else(|| ZimError::InvalidHeader(format!("{} table size overflows u64", label)))?;
    if byte_count > MAX_INDEX_ALLOCATION_BYTES {
        return Err(ZimError::AllocationLimit);
    }
    file.seek(SeekFrom::Start(offset)).map_err(io_err)?;
    let mut buf = vec![0u8; byte_count as usize];
    file.read_exact(&mut buf).map_err(io_err)?;
    let mut out = Vec::with_capacity(count as usize);
    for i in 0..count as usize {
        let off = i * 8;
        out.push(u64::from_le_bytes([
            buf[off],
            buf[off + 1],
            buf[off + 2],
            buf[off + 3],
            buf[off + 4],
            buf[off + 5],
            buf[off + 6],
            buf[off + 7],
        ]));
    }
    Ok(out)
}

fn build_path_index(
    file: &mut File,
    path_ptrs: &[u64],
    file_size: u64,
) -> Result<HashMap<String, u32>, ZimError> {
    let mut map = HashMap::with_capacity(path_ptrs.len());
    for (idx, &ptr) in path_ptrs.iter().enumerate() {
        if ptr >= file_size {
            return Err(ZimError::InvalidHeader(format!(
                "path pointer {} at index {} >= file size {}",
                ptr, idx, file_size
            )));
        }
        let read_len = ((file_size - ptr).min(MAX_ENTRY_READ_BYTES)) as usize;
        file.seek(SeekFrom::Start(ptr)).map_err(io_err)?;
        let mut buf = vec![0u8; read_len];
        let n = file.read(&mut buf).map_err(io_err)?;
        buf.truncate(n);
        let (raw, _consumed) = parse_entry(&buf)?;
        let key = entry_key_string(raw.namespace(), raw.url());
        map.insert(key, idx as u32);
    }
    Ok(map)
}

fn map_open_err(e: std::io::Error, path: &str) -> ZimError {
    match e.kind() {
        std::io::ErrorKind::NotFound
        | std::io::ErrorKind::PermissionDenied
        | std::io::ErrorKind::ConnectionRefused
        | std::io::ErrorKind::AddrInUse
        | std::io::ErrorKind::AddrNotAvailable => ZimError::UnreadablePath,
        _ => {
            if e.to_string().contains("No such file")
                || e.to_string().contains("permission")
            {
                ZimError::UnreadablePath
            } else {
                ZimError::Io(format!("cannot open {}: {}", path, e))
            }
        }
    }
}

fn io_err(e: std::io::Error) -> ZimError {
    ZimError::Io(e.to_string())
}
