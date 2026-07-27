#![forbid(unsafe_code)]

//! AnZimmermanZimEngine — clean-room ZIM archive reader.
//!
//! Implements the `ports::zim::{ZimEngine, ZimArchive}` traits for the
//! ZIM (Zeno IMproved) file format per the openZIM specification. This is
//! the release-named implementation backing `engine_id = "zim.anzimmerman.v1"`
//! in ARCHITECTURE.md §7.2.
//!
//! Source-of-truth: the clean-room TypeScript / Python / Go / PHP reference
//! implementations in `VENDOR/AnZIMmermanLib-src/AnZimmermanLIB/AnZimmermanLib/`.
//! The Rust port follows their (corrected, post-2026-02-08) header layout.
//!
//! Hard guarantees (ARCHITECTURE.md §7.1):
//!
//!   * No panic on malformed input — every malformed-byte path returns a
//!     declared `ZimError` variant.
//!   * No unbounded allocation — blob, MIME list, and pointer-table sizes
//!     are capped by the constants in [`limits`].
//!   * No unbounded decompression — clusters decompress under a fixed
//!     byte budget; bombs return [`ZimError::DecompressionLimit`].
//!   * No silent "unsupported" success — unsupported codecs, versions,
//!     and features return [`ZimError::UnsupportedFeature`].

pub mod archive;
pub mod cluster;
pub mod directory;
pub mod header;
pub mod limits;
pub mod sanitize;

pub use archive::AnZimmermanZimArchive;

use async_trait::async_trait;
use ports::zim::{ZimArchive, ZimEngine, ZimError, ZimOpenRequest, ZimMetadata};
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
