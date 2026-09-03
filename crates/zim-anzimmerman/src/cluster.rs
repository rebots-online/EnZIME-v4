//! Cluster reading: codec dispatch, bounded decompression, blob extraction.
//!
//! ZIM clusters are laid out as a 1-byte compression discriminator followed
//! by a stream of compressed bytes. The decompressed payload is itself a
//! `[u32 offset_table] || [blob_data]` region, where the first offset marks
//! the start of blob data and therefore implicitly encodes the table length
//! (`offset_count = first_offset / 4`, blob_count is `offset_count - 1`).

use std::io::Read;

use ports::zim::ZimError;

use crate::limits::{MAX_BLOB_BYTES, MAX_DECOMPRESSED_CLUSTER_BYTES, DECOMPRESS_CHUNK_BYTES};

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
    match compression {
        COMPRESSION_DEFAULT | COMPRESSION_NONE => {
            enforce_allocation(payload.len() as u64)?;
            Ok(payload.to_vec())
        }
        COMPRESSION_ZLIB => decode_zlib(payload),
        COMPRESSION_ZSTD => decode_zstd(payload),
        COMPRESSION_LZMA => Err(ZimError::UnsupportedFeature(
            "LZMA cluster codec not wired in this build".into(),
        )),
        COMPRESSION_BZIP2 => Err(ZimError::UnsupportedFeature(
            "bzip2 cluster codec is deprecated by ZIM and not supported".into(),
        )),
        COMPRESSION_EXTENDED => Err(ZimError::UnsupportedFeature(
            "extended 4-byte cluster compression tags are not supported".into(),
        )),
        other => Err(ZimError::UnsupportedFeature(format!(
            "unknown cluster compression byte 0x{:02X}",
            other
        ))),
    }
}

#[inline]
fn enforce_allocation(byte_count: u64) -> Result<(), ZimError> {
    if byte_count > MAX_DECOMPRESSED_CLUSTER_BYTES {
        return Err(ZimError::DecompressionLimit);
    }
    Ok(())
}

fn decode_zlib(payload: &[u8]) -> Result<Vec<u8>, ZimError> {
    // ZIM ZLIB clusters store raw DEFLATE bytes (no zlib header); this is
    // confirmed against the vendored AnZimmermanLib reference and against
    // libzim's zstd/zlib writer. DeflateDecoder matches that exactly.
    use flate2::read::DeflateDecoder;
    let mut out = Vec::new();
    let mut dec = DeflateDecoder::new(payload);
    let mut chunk = [0u8; DECOMPRESS_CHUNK_BYTES];
    loop {
        match dec.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => {
                out.extend_from_slice(&chunk[..n]);
                enforce_allocation(out.len() as u64)?;
            }
            Err(e) => {
                return Err(ZimError::InvalidHeader(format!(
                    "zlib decompression failed: {}",
                    e
                )));
            }
        }
    }
    Ok(out)
}

fn decode_zstd(payload: &[u8]) -> Result<Vec<u8>, ZimError> {
    let dec = zstd::stream::Decoder::new(payload)
        .map_err(|e| ZimError::InvalidHeader(format!("zstd decoder init: {}", e)))?;
    let dec = dec.single_frame();
    let mut reader = dec;
    let mut out = Vec::new();
    let mut chunk = [0u8; DECOMPRESS_CHUNK_BYTES];
    loop {
        match reader.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => {
                out.extend_from_slice(&chunk[..n]);
                enforce_allocation(out.len() as u64)?;
            }
            Err(e) => {
                return Err(ZimError::InvalidHeader(format!(
                    "zstd decompression failed: {}",
                    e
                )));
            }
        }
    }
    Ok(out)
}

/// Extracts one blob from a decompressed cluster buffer.
pub fn extract_blob(decompressed: &[u8], blob_number: u32) -> Result<&[u8], ZimError> {
    if decompressed.len() < 4 {
        return Err(ZimError::InvalidHeader(
            "decompressed cluster too short for blob offset table".into(),
        ));
    }
    let first_offset = le_u32(decompressed, 0) as usize;
    if first_offset == 0 || first_offset % 4 != 0 {
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
    if start > end {
        return Err(ZimError::InvalidHeader(format!(
            "cluster blob {} has inverted offsets [{}, {})",
            blob_idx, start, end
        )));
    }
    let len = end - start;
    if len as u64 > MAX_BLOB_BYTES {
        return Err(ZimError::AllocationLimit);
    }
    if end > decompressed.len() {
        return Err(ZimError::InvalidHeader(format!(
            "cluster blob {} extends past decompressed buffer [{}, {} > {})",
            blob_idx,
            start,
            end,
            decompressed.len()
        )));
    }
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
    let compression = raw[0];
    let payload = &raw[1..];
    let decompressed = decompress_cluster(compression, payload)?;

    // The decompressed buffer begins with a `u32` offset table; the first
    // offset both marks the start of blob data and encodes the table length
    // (see module header). We re-derive blob_count exactly as `extract_blob`
    // does, then walk every blob.
    if decompressed.len() < 4 {
        return Err(ZimError::InvalidHeader(
            "decompressed cluster too short for blob offset table".into(),
        ));
    }
    let first_offset = le_u32(&decompressed, 0) as usize;
    if first_offset == 0 || first_offset % 4 != 0 {
        return Err(ZimError::InvalidHeader(format!(
            "cluster blob table is mis-aligned: first offset {} is not a positive multiple of 4",
            first_offset
        )));
    }
    let offset_count = first_offset / 4;
    let blob_count = offset_count - 1;

    let mut out = Vec::with_capacity(blob_count);
    for idx in 0..blob_count {
        // `extract_blob` already enforces every per-blob invariant (inverted
        // offsets, out-of-range, MAX_BLOB_BYTES, etc.) — re-use it verbatim
        // by indexing through the same offset-table semantics.
        let blob = extract_blob(&decompressed, idx as u32)?;
        out.push(blob.to_vec());
    }
    Ok(out)
}
