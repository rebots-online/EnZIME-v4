//! Parsing of the fixed-size 80-byte ZIM file header (openZIM specification).
//!
//! The header layout matches the layout verified against production ZIM files
//! by the vendored AnZIMmermanLib (see `VENDOR/AnZIMmermanLib-src/.../ARCHITECTURE.md`,
//! 2026-02-08 fix entry). The earlier 4-byte-misaligned layout used by the
//! original vendored implementations is rejected.

use crate::model::ZimError;

/// ZIM magic number, little-endian byte sequence `5A 49 4D 04`.
pub const ZIM_MAGIC: u32 = 0x044D_495A;

/// Sentinel value of the `main_page` field meaning "no main page".
pub const NO_PAGE: u32 = 0xFFFF_FFFF;

/// ZIM major versions this implementation accepts.
pub const SUPPORTED_MAJOR_VERSIONS: &[u16] = &[5, 6];

/// 16-byte archive UUID (raw bytes, layout per openZIM spec).
pub type Uuid = [u8; 16];

#[derive(Clone, Debug)]
pub struct ZimHeader {
    pub magic: u32,
    pub major_version: u16,
    pub minor_version: u16,
    pub uuid: Uuid,
    pub entry_count: u32,
    pub cluster_count: u32,
    pub path_ptr_pos: u64,
    pub title_ptr_pos: u64,
    pub cluster_ptr_pos: u64,
    pub mime_list_pos: u64,
    pub main_page: u32,
    pub layout_page: u32,
    pub checksum_pos: u64,
}

/// Parses the 80-byte ZIM header. The buffer must be at least 80 bytes long;
/// any excess is ignored. Every field is validated for range and version.
pub fn parse_header(buf: &[u8]) -> Result<ZimHeader, ZimError> {
    if buf.len() < 80 {
        return Err(ZimError::InvalidHeader(format!(
            "header is {} bytes; need at least 80",
            buf.len()
        )));
    }
    let magic = le_u32(buf, 0);
    if magic != ZIM_MAGIC {
        return Err(ZimError::InvalidHeader(format!(
            "magic 0x{:08X} does not match ZIM magic 0x{:08X}",
            magic, ZIM_MAGIC
        )));
    }
    let major_version = le_u16(buf, 4);
    let minor_version = le_u16(buf, 6);
    if !SUPPORTED_MAJOR_VERSIONS.contains(&major_version) {
        return Err(ZimError::UnsupportedFeature(format!(
            "ZIM major version {} is not supported (accepted: {:?})",
            major_version, SUPPORTED_MAJOR_VERSIONS
        )));
    }
    let mut uuid = [0u8; 16];
    uuid.copy_from_slice(&buf[8..24]);
    let entry_count = le_u32(buf, 24);
    let cluster_count = le_u32(buf, 28);
    let path_ptr_pos = le_u64(buf, 32);
    let title_ptr_pos = le_u64(buf, 40);
    let cluster_ptr_pos = le_u64(buf, 48);
    let mime_list_pos = le_u64(buf, 56);
    let main_page = le_u32(buf, 64);
    let layout_page = le_u32(buf, 68);
    let checksum_pos = le_u64(buf, 72);

    Ok(ZimHeader {
        magic,
        major_version,
        minor_version,
        uuid,
        entry_count,
        cluster_count,
        path_ptr_pos,
        title_ptr_pos,
        cluster_ptr_pos,
        mime_list_pos,
        main_page,
        layout_page,
        checksum_pos,
    })
}

/// Checks that all header-derived offsets and indices lie inside the file
/// (or are the documented sentinel values). Called once during `open`.
pub fn validate_header_offsets(header: &ZimHeader, file_size: u64) -> Result<(), ZimError> {
    if header.mime_list_pos < 80 {
        return Err(ZimError::InvalidHeader(format!(
            "mime_list_pos {} is smaller than the 80-byte header",
            header.mime_list_pos
        )));
    }
    if header.mime_list_pos >= file_size {
        return Err(ZimError::InvalidHeader(format!(
            "mime_list_pos {} >= file size {}",
            header.mime_list_pos, file_size
        )));
    }
    for (name, off) in [
        ("path_ptr_pos", header.path_ptr_pos),
        ("cluster_ptr_pos", header.cluster_ptr_pos),
    ] {
        if off >= file_size {
            return Err(ZimError::InvalidHeader(format!(
                "{} {} >= file size {}",
                name, off, file_size
            )));
        }
    }
    // title_ptr_pos may be 0 in v6+ (obsolete field).
    if header.title_ptr_pos != 0 && header.title_ptr_pos >= file_size {
        return Err(ZimError::InvalidHeader(format!(
            "title_ptr_pos {} >= file size {}",
            header.title_ptr_pos, file_size
        )));
    }
    if header.checksum_pos != 0 && header.checksum_pos >= file_size {
        return Err(ZimError::InvalidHeader(format!(
            "checksum_pos {} >= file size {}",
            header.checksum_pos, file_size
        )));
    }
    if header.main_page != NO_PAGE && header.main_page as u64 >= header.entry_count as u64 {
        return Err(ZimError::InvalidHeader(format!(
            "main_page {} >= entry_count {}",
            header.main_page, header.entry_count
        )));
    }
    Ok(())
}

#[inline]
fn le_u16(buf: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([buf[at], buf[at + 1]])
}

#[inline]
fn le_u32(buf: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([buf[at], buf[at + 1], buf[at + 2], buf[at + 3]])
}

#[inline]
fn le_u64(buf: &[u8], at: usize) -> u64 {
    u64::from_le_bytes([
        buf[at],
        buf[at + 1],
        buf[at + 2],
        buf[at + 3],
        buf[at + 4],
        buf[at + 5],
        buf[at + 6],
        buf[at + 7],
    ])
}

/// Conservative cluster end shared by local and random-access readers.
pub fn cluster_end(
    header: &ZimHeader,
    start: u64,
    next_cluster: Option<u64>,
    source_len: u64,
) -> Result<u64, ZimError> {
    if start >= source_len || next_cluster.is_some_and(|n| n <= start || n > source_len) {
        return Err(ZimError::InvalidHeader("invalid cluster boundary".into()));
    }
    let mut end = next_cluster.unwrap_or(source_len);
    for boundary in [
        header.path_ptr_pos,
        header.title_ptr_pos,
        header.cluster_ptr_pos,
        header.mime_list_pos,
        header.checksum_pos,
        source_len,
    ] {
        if boundary > source_len {
            return Err(ZimError::InvalidHeader(
                "metadata boundary beyond source".into(),
            ));
        }
        if boundary > start {
            end = end.min(boundary);
        }
    }
    if end <= start {
        return Err(ZimError::InvalidHeader("empty cluster range".into()));
    }
    Ok(end)
}
