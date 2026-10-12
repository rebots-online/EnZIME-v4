//! Cluster reading: codec dispatch, bounded decompression, blob extraction.
//!
//! ZIM clusters are laid out as a 1-byte compression discriminator followed
//! by a stream of compressed bytes. The decompressed payload is itself a
//! `[u32 offset_table] || [blob_data]` region, where the first offset marks
//! the start of blob data and therefore implicitly encodes the table length
//! (`offset_count = first_offset / 4`, blob_count is `offset_count - 1`).

use std::io::Read;

use crate::model::ZimError;

use crate::limits::MAX_BLOB_BYTES;

/// Compression codes from the openZIM specification.
pub const COMPRESSION_DEFAULT: u8 = 0;
pub const COMPRESSION_NONE: u8 = 1;
pub const COMPRESSION_ZLIB: u8 = 2;
pub const COMPRESSION_BZIP2: u8 = 3;
pub const COMPRESSION_LZMA: u8 = 4;
pub const COMPRESSION_ZSTD: u8 = 5;

/// Extended tag occupying 4 bytes — used by some experimental archives,
/// not supported by this reader.
pub const COMPRESSION_EXTENDED: u8 = 0xFF;

/// Decompress (or copy) a cluster payload into a single owned buffer.
/// `compression` is the first byte of the cluster; `payload` is the cluster
/// body that follows the compression byte.
pub fn decompress_cluster(compression: u8, payload: &[u8]) -> Result<Vec<u8>, ZimError> {
    let mut input = std::io::Cursor::new([compression]).chain(payload);
    let mut output = Vec::new();
    crate::cluster_stream::decode_cluster_to(
        &mut input,
        &mut output,
        crate::cluster_stream::ClusterDecodeLimits::default(),
    )?;
    Ok(output)
}
pub(crate) fn cluster_encoding(tag: u8) -> Result<(u8, u8), ZimError> {
    if tag & 0xe0 != 0 {
        return Err(ZimError::UnsupportedFeature(format!(
            "unknown cluster flag {tag:#x}"
        )));
    }
    let codec = tag & 15;
    if !matches!(codec, 0 | 1 | 2 | 4 | 5) {
        return Err(ZimError::UnsupportedFeature(format!(
            "cluster codec {codec}"
        )));
    }
    Ok((codec, if tag & 0x10 != 0 { 8 } else { 4 }))
}
pub(crate) fn validate_blob_offsets(
    start: u64,
    end: u64,
    table_bytes: u64,
    total_bytes: u64,
    maximum_bytes: u64,
) -> Result<(), ZimError> {
    if start < table_bytes || start > end || end > total_bytes {
        return Err(ZimError::InvalidHeader("invalid blob endpoints".into()));
    }
    if end - start > maximum_bytes {
        return Err(ZimError::AllocationLimit);
    }
    Ok(())
}

/// Extracts one blob from a decompressed cluster buffer.
pub fn extract_blob(decompressed: &[u8], blob_number: u32) -> Result<&[u8], ZimError> {
    if decompressed.len() < 4 {
        return Err(ZimError::InvalidHeader(
            "decompressed cluster too short for blob offset table".into(),
        ));
    }
    let first_offset = le_u32(decompressed, 0) as usize;
    if first_offset < 8 || first_offset % 4 != 0 || first_offset > decompressed.len() {
        return Err(ZimError::InvalidHeader(format!(
            "cluster blob table is mis-aligned: first offset {} is not a positive multiple of 4",
            first_offset
        )));
    }
    let offset_count = first_offset / 4;
    let blob_count = offset_count - 1;
    let blob_idx = blob_number as usize;
    if blob_idx >= blob_count {
        return Err(ZimError::EntryNotFound(format!(
            "blob index {} out of range (cluster has {} blobs)",
            blob_idx, blob_count
        )));
    }
    let start = read_offset(decompressed, blob_idx)?;
    let end = read_offset(decompressed, blob_idx + 1)?;
    validate_blob_offsets(
        start as u64,
        end as u64,
        first_offset as u64,
        decompressed.len() as u64,
        MAX_BLOB_BYTES,
    )?;
    Ok(&decompressed[start..end])
}

fn read_offset(buf: &[u8], idx: usize) -> Result<usize, ZimError> {
    let byte_idx = idx * 4;
    if byte_idx + 4 > buf.len() {
        return Err(ZimError::InvalidHeader(format!(
            "blob offset {} out of range",
            idx
        )));
    }
    Ok(le_u32(buf, byte_idx) as usize)
}

#[inline]
fn le_u32(buf: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([buf[at], buf[at + 1], buf[at + 2], buf[at + 3]])
}

/// Decompress a raw ZIM cluster payload (bytes read straight from the file —
/// the first byte is the compression discriminator, the rest is the codec
/// payload) into its ordered list of member blobs.
///
/// This is the public re-entry point used by `zim-cluster-cache` (and any
/// other reader that obtains raw cluster bytes through a non-`File` channel
/// such as `TransferSource::open_range`). It wraps the same internal
/// `decompress_cluster` + `extract_blob` path that
/// `AnZimmermanZimArchive::get_or_decompress_cluster` uses once it has the
/// raw bytes — no new decompression logic.
///
/// Empty input is rejected (a real cluster always has at least the
/// 1-byte compression tag).
pub fn decompress_cluster_blobs(raw: &[u8]) -> Result<Vec<Vec<u8>>, ZimError> {
    if raw.is_empty() {
        return Err(ZimError::InvalidHeader(
            "cluster payload is empty (missing compression byte)".into(),
        ));
    }
    let (_, width) = cluster_encoding(raw[0])?;
    let decompressed = decompress_cluster(raw[0], &raw[1..])?;
    let mut input = std::io::Cursor::new(&decompressed);
    let mut first = [0; 8];
    input
        .read_exact(&mut first[..width as usize])
        .map_err(|e| ZimError::InvalidHeader(e.to_string()))?;
    let table = u64::from_le_bytes(first);
    if table < 2 * width as u64 || table % width as u64 != 0 || table > decompressed.len() as u64 {
        return Err(ZimError::InvalidHeader("invalid blob table".into()));
    }
    let mut out = Vec::new();
    for idx in 0..table / width as u64 - 1 {
        if width == 4 {
            out.push(extract_blob(&decompressed, idx as u32)?.to_vec());
            continue;
        }
        out.push(
            crate::cluster_stream::read_cluster_blob(
                &mut input,
                crate::cluster_stream::DecodedCluster {
                    length: decompressed.len() as u64,
                    offset_width: width,
                },
                idx as u32,
                MAX_BLOB_BYTES,
            )?
            .to_vec(),
        );
    }
    Ok(out)
}
