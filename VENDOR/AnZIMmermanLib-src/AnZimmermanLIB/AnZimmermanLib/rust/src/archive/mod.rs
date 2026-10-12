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

use crate::identity::{ArticleKey, ContentDigest, DigestAlgorithm, EntryKey};
use crate::model::{
    ArticleDocument, ResourceDocument, TitleSearchHit, TitleSearchQuery, ZimArchive,
    ZimEntryDescriptor, ZimEntryKind, ZimEntryStream, ZimError, ZimMetadata, ZimOpenRequest,
    ZimVerificationFinding, ZimVerificationReport,
};
use async_trait::async_trait;
use bytes::Bytes;
use sha2::{Digest as _, Sha256};

use crate::cluster::{decompress_cluster, extract_blob};
use crate::directory::{parse_entry, ArticleEntry, RawEntry};
use crate::header::{parse_header, ZimHeader};
use crate::header::{validate_header_offsets, NO_PAGE, ZIM_MAGIC};
use crate::limits::{
    HASH_CHUNK_BYTES, MAX_COMPRESSED_CLUSTER_BYTES, MAX_ENTRY_READ_BYTES,
    MAX_INDEX_ALLOCATION_BYTES, MAX_MIME_TYPES, MIN_ZIM_FILE_BYTES,
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
pub async fn open_archive(request: &ZimOpenRequest) -> Result<AnZimmermanZimArchive, ZimError> {
    let inner = Inner::open(request)?;
    Ok(AnZimmermanZimArchive {
        inner: Arc::new(inner),
    })
}

pub struct AnZimmermanZimArchive {
    inner: Arc<Inner>,
}

pub(crate) fn classify_kind(namespace: u8, mime: &str) -> ZimEntryKind {
    if namespace == b'M' {
        ZimEntryKind::Metadata
    } else if namespace == b'A' || mime.starts_with("text/html") {
        ZimEntryKind::Article
    } else {
        ZimEntryKind::Resource
    }
}

mod inner;
mod io;
mod layout;
mod traits;
use inner::Inner;
use io::*;
pub use layout::{ZimLayoutEntry, ZimLayoutSnapshot};

pub(crate) use traits::{score_title, sha256_digest};
