#![forbid(unsafe_code)]

//! EnZIME port adapter; canonical Rust implementation lives in AnZimmermanLib.

pub mod archive;
pub mod cluster;
mod convert;
pub mod directory;
pub mod header;
pub mod limits;
pub mod sanitize;

pub use archive::{AnZimmermanZimArchive, ZimLayoutEntry, ZimLayoutSnapshot};

use async_trait::async_trait;
use ports::zim::{ZimArchive, ZimEngine, ZimError, ZimMetadata, ZimOpenRequest};
use std::sync::Arc;

/// The engine ID, exactly as named in ARCHITECTURE.md §7.2 release table.
pub const ENGINE_ID: &str = anzimmerman::ENGINE_ID;

/// Engine semantic version. Mirrors the vendored library `version.json`.
pub const ENGINE_VERSION: &str = anzimmerman::ENGINE_VERSION;

/// Concrete ZIM engine implementation backed by the AnZimmermanLib format.
///
/// Cheap to construct; `Clone` so callers can stash a copy per worker. All
/// real per-archive state lives inside [`AnZimmermanZimArchive`].
#[derive(Debug, Clone, Default)]
pub struct AnZimmermanZimEngine;

impl AnZimmermanZimEngine {
    #[inline]
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ZimEngine for AnZimmermanZimEngine {
    fn engine_id(&self) -> &'static str {
        ENGINE_ID
    }

    fn engine_version(&self) -> String {
        ENGINE_VERSION.to_string()
    }

    async fn probe(&self, request: &ZimOpenRequest) -> Result<ZimMetadata, ZimError> {
        let archive = archive::open_archive(request).await?;
        Ok(archive.metadata().clone())
    }

    async fn open(&self, request: ZimOpenRequest) -> Result<Arc<dyn ZimArchive>, ZimError> {
        let archive = archive::open_archive(&request).await?;
        Ok(Arc::new(archive))
    }
}

pub mod range_adapter;
pub use range_adapter::RangeArchiveAdapter;
