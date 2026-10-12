//! Minimal ZIM writer used by the test suite to construct conformance
//! fixtures in memory. Implements only the openZIM v6 layout actually
//! exercised by `AnZimmermanZimArchive`: 80-byte header, MIME list,
//! path pointer table, cluster pointer table, one or more uncompressed
//! or ZLIB clusters of blobs, plus a trailing 16-byte zero checksum.
//!
//! This module is `cfg(test)`-only — production code reads real ZIM files
//! produced by Wikipedia / openZIM. The writer exists because every
//! §7.4 conformance fixture genuinely needs valid on-disk bytes for its
//! assertion to mean anything.

use std::io::Write;

use flate2::write::ZlibEncoder;
use flate2::Compression;

pub const ZIM_MAGIC: u32 = 0x044D_495A;
pub const NO_PAGE: u32 = 0xFFFF_FFFF;
pub const REDIRECT_MIMETYPE: u16 = 0xFFFF;
pub const COMPRESSION_DEFAULT: u8 = 0;
pub const COMPRESSION_ZLIB: u8 = 2;

#[derive(Clone, Debug)]
pub enum TestEntry {
    Article {
        namespace: u8,
        url: String,
        title: String,
        mimetype: String,
        blob: Vec<u8>,
    },
    Redirect {
        namespace: u8,
        url: String,
        title: String,
        target_index: u32,
    },
}

#[derive(Clone, Debug, Default)]
pub struct ZimBuilder {
    pub entries: Vec<TestEntry>,
    pub main_page_index: Option<u32>,
    pub uuid: [u8; 16],
    /// If set, every cluster is compressed with raw DEFLATE.
    pub zlib_clusters: bool,
    /// Pairs of `(metadata_name, metadata_value)` written verbatim as
    /// `M/<name>` text articles in namespace `M`.
    pub metadata: Vec<(String, String)>,
    /// If set, replaces the on-disk cluster payload for the (single) article
    /// cluster with these exact bytes (`[comp_byte] || [compressed]`).
    /// Used by the decompression-bomb conformance fixture.
    pub raw_cluster_override: Option<Vec<u8>>,
}

impl ZimBuilder {
    pub fn add_article(
        mut self,
        namespace: u8,
        url: impl Into<String>,
        title: impl Into<String>,
        mimetype: impl Into<String>,
        blob: Vec<u8>,
    ) -> Self {
        self.entries.push(TestEntry::Article {
            namespace,
            url: url.into(),
            title: title.into(),
            mimetype: mimetype.into(),
            blob,
        });
        self
    }

    pub fn add_redirect(
        mut self,
        namespace: u8,
        url: impl Into<String>,
        title: impl Into<String>,
        target_index: u32,
    ) -> Self {
        self.entries.push(TestEntry::Redirect {
            namespace,
            url: url.into(),
            title: title.into(),
            target_index,
        });
        self
    }

    /// Produces a fully serialized ZIM file in `out`. Returns the byte length
    /// written. The output is a valid v6 ZIM with one cluster per article
    /// (keeps the fixture builder trivial and avoids cross-blob packing).
    pub fn write(&self, out: &mut Vec<u8>) -> usize {
        // Metadata entries are stored as text articles in namespace M.
        let metadata_entries: Vec<TestEntry> = self
            .metadata
            .iter()
            .map(|(name, value)| TestEntry::Article {
                namespace: b'M',
                url: format!("M/{}", name)
                    .strip_prefix("M/")
                    .unwrap_or(name)
                    .to_string(),
                title: name.clone(),
                mimetype: "text/plain;charset=UTF-8".to_string(),
                blob: value.as_bytes().to_vec(),
            })
            .collect();
        let all_entries: Vec<&TestEntry> =
            self.entries.iter().chain(metadata_entries.iter()).collect();
        let article_entries: Vec<&TestEntry> = all_entries
            .iter()
            .copied()
            .filter(|e| matches!(e, TestEntry::Article { .. }))
            .collect();

        // Assign MIME indices and build the MIME list.
        let mut mimes: Vec<String> = Vec::new();
        let mime_index_for = |s: &str, mimes: &mut Vec<String>| -> u16 {
            for (i, m) in mimes.iter().enumerate() {
                if m == s {
                    return i as u16;
                }
            }
            mimes.push(s.to_string());
            (mimes.len() - 1) as u16
        };
        for entry in &article_entries {
            let m = match entry {
                TestEntry::Article { mimetype, .. } => mimetype,
                _ => unreachable!(),
            };
            let _ = mime_index_for(m, &mut mimes);
        }
        let _ = mime_index_for;

        // MIME list — starts at offset 80. Always at least `\0\0`.
        let mut mime_list_bytes: Vec<u8> = Vec::new();
        for m in &mimes {
            mime_list_bytes.extend_from_slice(m.as_bytes());
            mime_list_bytes.push(0);
        }
        mime_list_bytes.push(0); // terminating second NUL
        if mime_list_bytes.len() < 2 {
            // Empty MIME list — still need the full double-NUL terminator.
            mime_list_bytes.extend_from_slice(b"\0\0");
        }
        let mime_list_pos: u64 = 80;
        let mime_list_end: u64 = mime_list_pos + mime_list_bytes.len() as u64;

        // One cluster per article — cluster K contains the blob of the K-th
        // article entry. The on-disk form is
        //   `[compression_byte] || [compressed(offset_table || blob)]`.
        // We pre-serialise every cluster to its final bytes here so the
        // offset table below can use actual compressed sizes.
        let cluster_count: u32 = article_entries.len() as u32;
        let mut cluster_payloads: Vec<Vec<u8>> = Vec::with_capacity(article_entries.len());
        for entry in &article_entries {
            let blob = match entry {
                TestEntry::Article { blob, .. } => blob.clone(),
                _ => unreachable!(),
            };
            let table_bytes = 8usize; // two u32 offsets
            let start = table_bytes as u32;
            let end = (table_bytes + blob.len()) as u32;
            let mut inner = Vec::with_capacity(table_bytes + blob.len());
            inner.extend_from_slice(&start.to_le_bytes());
            inner.extend_from_slice(&end.to_le_bytes());
            inner.extend_from_slice(&blob);
            let compression_byte = if self.zlib_clusters {
                COMPRESSION_ZLIB
            } else {
                COMPRESSION_DEFAULT
            };
            let mut on_disk = Vec::with_capacity(1 + inner.len());
            on_disk.push(compression_byte);
            if self.zlib_clusters {
                let mut enc = ZlibEncoder::new(Vec::new(), Compression::default());
                enc.write_all(&inner).unwrap();
                let mut z = enc.finish().unwrap();
                // flate2's ZlibEncoder wraps raw DEFLATE in a 2-byte zlib
                // header and a 4-byte adler32 trailer; ZIM clusters store
                // raw DEFLATE alone, so strip them.
                z.drain(0..2);
                z.truncate(z.len().saturating_sub(4));
                on_disk.extend_from_slice(&z);
            } else {
                on_disk.extend_from_slice(&inner);
            }
            cluster_payloads.push(on_disk);
        }
        // Optional override: replace cluster 0 with hand-built bytes
        // (used by the decompression-bomb fixture).
        if let Some(raw) = self.raw_cluster_override.as_ref() {
            if !cluster_payloads.is_empty() {
                cluster_payloads[0] = raw.clone();
            }
        }

        // Plan the layout: header | mime | directory | path_ptr_table | cluster_ptr_table | clusters | checksum
        let entry_count = all_entries.len() as u32;

        let directory_size: u64 = all_entries
            .iter()
            .map(|e| match e {
                TestEntry::Article { url, title, .. } => 16 + url.len() + 1 + title.len() + 1,
                TestEntry::Redirect { url, title, .. } => 12 + url.len() + 1 + title.len() + 1,
            })
            .map(|n| n as u64)
            .sum();

        let path_ptr_table_size = (entry_count as u64) * 8;
        let cluster_ptr_table_size = (cluster_count as u64) * 8;

        let directory_pos: u64 = mime_list_end;
        let path_ptr_table_pos: u64 = directory_pos + directory_size;
        let cluster_ptr_table_pos: u64 = path_ptr_table_pos + path_ptr_table_size;
        let clusters_pos: u64 = cluster_ptr_table_pos + cluster_ptr_table_size;

        // Compute directory entry offsets (absolute) for the path pointer table.
        let mut entry_offsets: Vec<u64> = Vec::with_capacity(entry_count as usize);
        {
            let mut cursor = directory_pos;
            for e in &all_entries {
                entry_offsets.push(cursor);
                cursor += match e {
                    TestEntry::Article { url, title, .. } => 16 + url.len() + 1 + title.len() + 1,
                    TestEntry::Redirect { url, title, .. } => 12 + url.len() + 1 + title.len() + 1,
                } as u64;
            }
        }

        // Compute cluster offsets.
        let mut cluster_offsets: Vec<u64> = Vec::with_capacity(cluster_count as usize);
        {
            let mut cursor = clusters_pos;
            for c in &cluster_payloads {
                cluster_offsets.push(cursor);
                cursor += c.len() as u64;
            }
        }
        let checksum_pos: u64 = cluster_offsets.last().copied().unwrap_or(clusters_pos)
            + cluster_payloads.last().map(|c| c.len() as u64).unwrap_or(0);

        // --- Serialise ------------------------------------------------------
        out.clear();

        // Header (80 bytes)
        out.extend_from_slice(&ZIM_MAGIC.to_le_bytes());
        out.extend_from_slice(&6u16.to_le_bytes()); // major version
        out.extend_from_slice(&0u16.to_le_bytes()); // minor version
        out.extend_from_slice(&self.uuid); // 16-byte uuid
        out.extend_from_slice(&entry_count.to_le_bytes());
        out.extend_from_slice(&cluster_count.to_le_bytes());
        out.extend_from_slice(&path_ptr_table_pos.to_le_bytes());
        out.extend_from_slice(&0u64.to_le_bytes()); // title ptr table (obsolete)
        out.extend_from_slice(&cluster_ptr_table_pos.to_le_bytes());
        out.extend_from_slice(&mime_list_pos.to_le_bytes());
        let main_page = self.main_page_index.unwrap_or(NO_PAGE);
        out.extend_from_slice(&main_page.to_le_bytes());
        out.extend_from_slice(&NO_PAGE.to_le_bytes()); // layout page
        out.extend_from_slice(&checksum_pos.to_le_bytes());
        assert_eq!(out.len(), 80);

        // MIME list
        out.extend_from_slice(&mime_list_bytes);

        // Directory entries
        let mut cluster_idx_for_article: u32 = 0;
        for e in &all_entries {
            match e {
                TestEntry::Article {
                    namespace,
                    url,
                    title,
                    mimetype,
                    blob: _,
                } => {
                    // Look up the precomputed MIME index without re-borrowing `mimes`.
                    let mut mt: u16 = 0;
                    for (i, m) in mimes.iter().enumerate() {
                        if m == mimetype {
                            mt = i as u16;
                            break;
                        }
                    }
                    out.extend_from_slice(&mt.to_le_bytes());
                    out.push(0); // parameterLen
                    out.push(*namespace);
                    out.extend_from_slice(&0u32.to_le_bytes()); // revision
                    out.extend_from_slice(&cluster_idx_for_article.to_le_bytes());
                    out.extend_from_slice(&0u32.to_le_bytes()); // blob 0 in cluster
                    out.extend_from_slice(url.as_bytes());
                    out.push(0);
                    out.extend_from_slice(title.as_bytes());
                    out.push(0);
                    cluster_idx_for_article += 1;
                }
                TestEntry::Redirect {
                    namespace,
                    url,
                    title,
                    target_index,
                } => {
                    out.extend_from_slice(&REDIRECT_MIMETYPE.to_le_bytes());
                    out.push(0);
                    out.push(*namespace);
                    out.extend_from_slice(&0u32.to_le_bytes());
                    out.extend_from_slice(&target_index.to_le_bytes());
                    out.extend_from_slice(url.as_bytes());
                    out.push(0);
                    out.extend_from_slice(title.as_bytes());
                    out.push(0);
                }
            }
        }
        assert_eq!(out.len() as u64, path_ptr_table_pos);

        // Path pointer table
        for off in &entry_offsets {
            out.extend_from_slice(&off.to_le_bytes());
        }
        assert_eq!(out.len() as u64, cluster_ptr_table_pos);

        // Cluster pointer table
        for off in &cluster_offsets {
            out.extend_from_slice(&off.to_le_bytes());
        }
        assert_eq!(out.len() as u64, clusters_pos);

        // Clusters — already in final on-disk form (post-compression).
        for raw in &cluster_payloads {
            out.extend_from_slice(raw);
        }
        assert_eq!(out.len() as u64, checksum_pos);

        // 16-byte zero checksum (not validated by this engine)
        out.extend_from_slice(&[0u8; 16]);

        out.len()
    }
}

/// Convenience: write to a `tempfile::NamedTempFile` and return its path.
pub fn write_temp(builder: &ZimBuilder) -> std::path::PathBuf {
    let mut buf = Vec::new();
    builder.write(&mut buf);
    let mut tmp = tempfile::NamedTempFile::new().expect("tempfile");
    tmp.write_all(&buf).expect("write");
    tmp.into_temp_path().keep().expect("keep")
}
