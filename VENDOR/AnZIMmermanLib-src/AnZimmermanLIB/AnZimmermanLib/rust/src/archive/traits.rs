use super::*;
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
        if !matches!(classify_kind(entry.namespace, &mime), ZimEntryKind::Article) {
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

    async fn resolve_redirect(&self, key: &EntryKey, max_hops: u8) -> Result<EntryKey, ZimError> {
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
                    current = entry_key_string(target.namespace(), target.url());
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
                    article_key: ArticleKey(entry_key_string(raw.namespace(), raw.url())),
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
        let end = start.saturating_add(query.limit as usize).min(hits.len());
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
                    detail: format!("path pointer {} >= file size {}", ptr, self.inner.size),
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
                                entry_key: Some(EntryKey(entry_key_string(a.namespace, &a.url))),
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
                                entry_key: Some(EntryKey(entry_key_string(a.namespace, &a.url))),
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
                    detail: format!("cluster pointer {} >= file size {}", ptr, self.inner.size),
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

pub(crate) fn score_title(needle: &str, haystack_lower: &str) -> f32 {
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

pub(crate) fn sha256_digest(bytes: &Bytes) -> ContentDigest {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let hash = hasher.finalize();
    ContentDigest {
        algorithm: DigestAlgorithm::Sha256,
        hex: hex::encode(hash),
    }
}
