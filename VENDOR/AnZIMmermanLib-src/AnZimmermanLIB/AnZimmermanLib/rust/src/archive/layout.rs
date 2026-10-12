use super::*;
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
