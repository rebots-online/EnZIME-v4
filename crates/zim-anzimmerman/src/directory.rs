//! Parsing of directory entries (article, redirect, linktarget, deleted).
//!
//! Layout per openZIM spec — the 2-byte `mimetype_index` discriminator is
//! read first; the values `0xFFFF`, `0xFFFE`, `0xFFFD` select the
//! redirect/linktarget/deleted variants respectively, everything else is an
//! article entry. All variants share the same `namespace`, `revision`,
//! `url`, `title` fields.

use ports::zim::ZimError;

use crate::limits::{MAX_TITLE_BYTES, MAX_URL_BYTES};

/// Discriminator value selecting the redirect variant.
pub const MIMETYPE_REDIRECT: u16 = 0xFFFF;
/// Discriminator value selecting the linktarget variant.
pub const MIMETYPE_LINKTARGET: u16 = 0xFFFE;
/// Discriminator value selecting the deleted variant.
pub const MIMETYPE_DELETED: u16 = 0xFFFD;

#[derive(Clone, Debug)]
pub enum RawEntry {
    Article(ArticleEntry),
    Redirect(RedirectEntry),
    Linktarget(LinktargetEntry),
    Deleted(DeletedEntry),
}

#[derive(Clone, Debug)]
pub struct ArticleEntry {
    pub mimetype_index: u16,
    pub namespace: u8,
    pub revision: u32,
    pub cluster_number: u32,
    pub blob_number: u32,
    pub url: String,
    pub title: String,
}

#[derive(Clone, Debug)]
pub struct RedirectEntry {
    pub namespace: u8,
    pub revision: u32,
    pub redirect_index: u32,
    pub url: String,
    pub title: String,
}

#[derive(Clone, Debug)]
pub struct LinktargetEntry {
    pub namespace: u8,
    pub revision: u32,
    pub url: String,
    pub title: String,
}

#[derive(Clone, Debug)]
pub struct DeletedEntry {
    pub namespace: u8,
    pub revision: u32,
    pub url: String,
    pub title: String,
}

impl RawEntry {
    pub fn namespace(&self) -> u8 {
        match self {
            RawEntry::Article(a) => a.namespace,
            RawEntry::Redirect(r) => r.namespace,
            RawEntry::Linktarget(l) => l.namespace,
            RawEntry::Deleted(d) => d.namespace,
        }
    }

    pub fn url(&self) -> &str {
        match self {
            RawEntry::Article(a) => &a.url,
            RawEntry::Redirect(r) => &r.url,
            RawEntry::Linktarget(l) => &l.url,
            RawEntry::Deleted(d) => &d.url,
        }
    }

    pub fn title(&self) -> &str {
        match self {
            RawEntry::Article(a) => &a.title,
            RawEntry::Redirect(r) => &r.title,
            RawEntry::Linktarget(l) => &l.title,
            RawEntry::Deleted(d) => &d.title,
        }
    }
}

/// Parses a single directory entry located at the start of `buf`. Returns the
/// decoded entry plus the number of bytes consumed, which lets the caller
/// advance if it ever needs to walk entries sequentially.
pub fn parse_entry(buf: &[u8]) -> Result<(RawEntry, usize), ZimError> {
    if buf.len() < 8 {
        return Err(ZimError::InvalidHeader(format!(
            "directory entry truncated: {} bytes available, need at least 8",
            buf.len()
        )));
    }
    let mimetype = le_u16(buf, 0);
    // offset 2: 1-byte parameterLen, currently unused by this reader.
    let namespace = buf[3];
    let revision = le_u32(buf, 4);

    match mimetype {
        MIMETYPE_REDIRECT => {
            if buf.len() < 12 {
                return Err(ZimError::InvalidHeader(
                    "redirect entry truncated before redirect_index".into(),
                ));
            }
            let redirect_index = le_u32(buf, 8);
            let (url, title, str_consumed) = parse_url_title(&buf[12..])?;
            Ok((
                RawEntry::Redirect(RedirectEntry {
                    namespace,
                    revision,
                    redirect_index,
                    url,
                    title,
                }),
                12 + str_consumed,
            ))
        }
        MIMETYPE_LINKTARGET => {
            let (url, title, str_consumed) = parse_url_title(&buf[8..])?;
            Ok((
                RawEntry::Linktarget(LinktargetEntry {
                    namespace,
                    revision,
                    url,
                    title,
                }),
                8 + str_consumed,
            ))
        }
        MIMETYPE_DELETED => {
            let (url, title, str_consumed) = parse_url_title(&buf[8..])?;
            Ok((
                RawEntry::Deleted(DeletedEntry {
                    namespace,
                    revision,
                    url,
                    title,
                }),
                8 + str_consumed,
            ))
        }
        _ => {
            if buf.len() < 16 {
                return Err(ZimError::InvalidHeader(
                    "article entry truncated before blob_number".into(),
                ));
            }
            let cluster_number = le_u32(buf, 8);
            let blob_number = le_u32(buf, 12);
            let (url, title, str_consumed) = parse_url_title(&buf[16..])?;
            Ok((
                RawEntry::Article(ArticleEntry {
                    mimetype_index: mimetype,
                    namespace,
                    revision,
                    cluster_number,
                    blob_number,
                    url,
                    title,
                }),
                16 + str_consumed,
            ))
        }
    }
}

fn parse_url_title(buf: &[u8]) -> Result<(String, String, usize), ZimError> {
    let (url, url_nul_pos) = read_cstr(buf, MAX_URL_BYTES, "url")?;
    let after_url = url_nul_pos + 1; // include NUL
    if after_url >= buf.len() {
        return Err(ZimError::InvalidHeader(
            "entry truncated after url field — missing title NUL".into(),
        ));
    }
    let (title, title_nul_pos) = read_cstr(&buf[after_url..], MAX_TITLE_BYTES, "title")?;
    Ok((url, title, after_url + title_nul_pos + 1))
}

fn read_cstr(buf: &[u8], max_bytes: usize, field: &'static str) -> Result<(String, usize), ZimError> {
    let nul_pos = buf.iter().position(|&b| b == 0).ok_or_else(|| {
        ZimError::InvalidHeader(format!("entry {} field missing NUL terminator", field))
    })?;
    if nul_pos > max_bytes {
        return Err(ZimError::AllocationLimit);
    }
    let raw = &buf[..nul_pos];
    let s = std::str::from_utf8(raw)
        .map_err(|_| ZimError::InvalidHeader(format!("entry {} is not valid UTF-8", field)))?;
    Ok((s.to_string(), nul_pos))
}

#[inline]
fn le_u16(buf: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([buf[at], buf[at + 1]])
}

#[inline]
fn le_u32(buf: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([buf[at], buf[at + 1], buf[at + 2], buf[at + 3]])
}
