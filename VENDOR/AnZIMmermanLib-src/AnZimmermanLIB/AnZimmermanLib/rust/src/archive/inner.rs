use super::*;
pub(super) struct Inner {
    pub(super) file: Mutex<File>,
    pub(super) size: u64,
    pub(super) header: ZimHeader,
    pub(super) mime_types: Vec<String>,
    pub(super) path_ptrs: Vec<u64>,
    pub(super) cluster_ptrs: Vec<u64>,
    pub(super) metadata: ZimMetadata,
    pub(super) path_index: HashMap<String, u32>,
    cluster_cache: Mutex<ClusterCache>,
}

#[derive(Default)]
struct ClusterCache {
    index: Option<u32>,
    decompressed: Option<(Arc<Vec<u8>>, u8)>,
}

impl Inner {
    pub(super) fn open(request: &ZimOpenRequest) -> Result<Self, ZimError> {
        let path = Path::new(&request.path);
        let mut file = File::open(path).map_err(|e| map_open_err(e, &request.path))?;
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

    pub(super) fn populate_metadata(&mut self) -> Result<(), ZimError> {
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

    pub(super) fn metadata_text(&self, name: &str) -> Result<Option<String>, ZimError> {
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

    pub(super) fn main_page_key(&self, idx: u32) -> Result<Option<ArticleKey>, ZimError> {
        if idx as usize >= self.path_ptrs.len() {
            return Ok(None);
        }
        let raw = self.entry_at(idx)?;
        Ok(Some(ArticleKey(entry_key_string(
            raw.namespace(),
            raw.url(),
        ))))
    }

    pub(super) fn read_checksum(&self) -> Result<Option<String>, ZimError> {
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

    pub(super) fn entry_at(&self, idx: u32) -> Result<RawEntry, ZimError> {
        let ptr = *self
            .path_ptrs
            .get(idx as usize)
            .ok_or_else(|| ZimError::EntryNotFound(format!("entry index {} out of range", idx)))?;
        self.read_entry_at_ptr(ptr)
    }

    pub(super) fn read_entry_at_ptr(&self, ptr: u64) -> Result<RawEntry, ZimError> {
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

    pub(super) fn find_entry_idx(&self, key: &str) -> Result<u32, ZimError> {
        if let Some(&i) = self.path_index.get(key) {
            return Ok(i);
        }
        Err(ZimError::EntryNotFound(key.to_string()))
    }

    pub(super) fn find_article_idx(&self, key: &str) -> Result<u32, ZimError> {
        if key.starts_with("A/") {
            return self.find_entry_idx(key);
        }
        let prefixed = format!("A/{}", key);
        if let Ok(i) = self.find_entry_idx(&prefixed) {
            return Ok(i);
        }
        self.find_entry_idx(key)
    }

    pub(super) fn mime_for(&self, idx: u16) -> Result<&str, ZimError> {
        self.mime_types
            .get(idx as usize)
            .map(|s| s.as_str())
            .ok_or_else(|| ZimError::InvalidHeader(format!("mimetype index {} out of range", idx)))
    }

    pub(super) fn cluster_byte_range(&self, cluster_idx: u32) -> Result<(u64, u64), ZimError> {
        let start = *self.cluster_ptrs.get(cluster_idx as usize).ok_or_else(|| {
            ZimError::EntryNotFound(format!("cluster index {} out of range", cluster_idx))
        })?;
        let end = crate::header::cluster_end(
            &self.header,
            start,
            self.cluster_ptrs.get(cluster_idx as usize + 1).copied(),
            self.size,
        )?;
        let span = end - start;
        if span > MAX_COMPRESSED_CLUSTER_BYTES {
            return Err(ZimError::AllocationLimit);
        }
        Ok((start, end))
    }

    pub(super) fn read_cluster_payload(&self, cluster_idx: u32) -> Result<(u8, Vec<u8>), ZimError> {
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

    pub(super) fn get_or_decompress_cluster(
        &self,
        cluster_idx: u32,
    ) -> Result<(Arc<Vec<u8>>, u8), ZimError> {
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
        let width = crate::cluster::cluster_encoding(compression)?.1;
        guard.decompressed = Some((decompressed.clone(), width));
        Ok((decompressed, width))
    }

    pub(super) fn cluster_cache_lock_match(&self, cluster_idx: u32) -> Option<(Arc<Vec<u8>>, u8)> {
        let guard = self.cluster_cache.lock().ok()?;
        if guard.index == Some(cluster_idx) {
            guard.decompressed.clone()
        } else {
            None
        }
    }

    pub(super) fn blob_bytes(&self, entry: &ArticleEntry) -> Result<Bytes, ZimError> {
        let (cluster, width) = self.get_or_decompress_cluster(entry.cluster_number)?;
        if width == 4 {
            return Ok(Bytes::copy_from_slice(extract_blob(
                &cluster,
                entry.blob_number,
            )?));
        }
        crate::cluster_stream::read_cluster_blob(
            &mut std::io::Cursor::new(cluster.as_slice()),
            crate::cluster_stream::DecodedCluster {
                length: cluster.len() as u64,
                offset_width: width,
            },
            entry.blob_number,
            crate::limits::MAX_BLOB_BYTES,
        )
    }

    pub(super) fn build_entry_descriptor(
        &self,
        _idx: u32,
        raw: &RawEntry,
    ) -> Result<ZimEntryDescriptor, ZimError> {
        let entry_key = EntryKey(entry_key_string(raw.namespace(), raw.url()));
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
                let redirect_target = self.redirect_target_key(r.redirect_index)?;
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

    pub(super) fn redirect_target_key(
        &self,
        target_idx: u32,
    ) -> Result<Option<EntryKey>, ZimError> {
        if (target_idx as usize) >= self.path_ptrs.len() {
            return Ok(None);
        }
        let raw = self.entry_at(target_idx)?;
        Ok(Some(EntryKey(entry_key_string(raw.namespace(), raw.url()))))
    }

    pub(super) fn blob_len(&self, entry: &ArticleEntry) -> Result<u64, ZimError> {
        let (cluster, width) = self.get_or_decompress_cluster(entry.cluster_number)?;
        if width == 4 {
            return Ok(extract_blob(&cluster, entry.blob_number)?.len() as u64);
        }
        Ok(self.blob_bytes(entry)?.len() as u64)
    }
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
