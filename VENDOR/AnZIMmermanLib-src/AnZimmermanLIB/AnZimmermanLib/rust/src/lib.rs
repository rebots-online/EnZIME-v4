#![forbid(unsafe_code)]

//! Canonical Rust implementation of the multilingual AnZimmermanLib.
//! Maintained here; EnZIME uses a separate application-port adapter.

pub mod archive;
pub mod cluster;
pub mod directory;
pub mod header;
pub mod limits;
pub mod sanitize;

pub use archive::{AnZimmermanZimArchive, ZimLayoutEntry, ZimLayoutSnapshot};

use async_trait::async_trait;

use std::sync::Arc;

/// The engine ID, exactly as named in ARCHITECTURE.md §7.2 release table.
pub const ENGINE_ID: &str = "zim.anzimmerman.v1";

/// Engine semantic version. Mirrors the vendored library `version.json`.
pub const ENGINE_VERSION: &str = "1.0.0";

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

pub mod error;
pub mod identity;
pub mod model;
pub use error::ZimError;
pub use identity::*;
pub use model::*;
