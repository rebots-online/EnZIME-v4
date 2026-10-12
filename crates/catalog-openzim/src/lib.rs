#![forbid(unsafe_code)]

//! Ring-2 catalog adapter — `crates/catalog-openzim`.
//!
//! Implements the §8.2 `CatalogSource` release implementations:
//!
//! | Implementation | `source_id` | Format |
//! |---|---|---|
//! | `OpenZimXmlCatalogSource` | `catalog.openzim.xml.v1` | OpenZIM library XML |
//! | `EnzimeJsonCatalogSource` | `catalog.enzime.json.v1` | EnZIME catalog JSON |
//!
//! Both adapters consume the §8.2 `ports::catalog::CatalogSource` trait and
//! fetch catalog bytes through an injected `ports::transfer::TransferSource`
//! (typically `HttpTransferSource` from `transfer-http`, supplied by the
//! application layer at wiring time). Signature verification, when configured,
//! delegates to `ports::signature::SignatureProvider`. This keeps this crate
//! on the Ring-2 / Ring-1 boundary — it never depends on a concrete transfer
//! or signature provider implementation (invariant A-02).
//!
//! The third §8.2 catalog implementation, `LocalManifestCatalogSource`
//! (`catalog.local.manifest.v1`), is out of scope for this crate — it consumes
//! in-process `PackManifest` values rather than fetched catalog documents and
//! belongs in a separate adapter.

pub mod enzime_json;
pub mod openzim_xml;
pub mod kiwix;
pub use kiwix::{
    parse_kiwix_metalink, parse_kiwix_opds, KiwixAcquisition, KiwixCatalogClient,
    KiwixEntry, KiwixPage, KiwixQuery, PublishedDigest,
};
mod store;
mod validate;

pub use enzime_json::EnzimeJsonCatalogSource;
pub use openzim_xml::OpenZimXmlCatalogSource;

// Re-export the port types so consumers can `use catalog_openzim::*` without a
// second dependency on the ports crate just to spell a query.
pub use ports::catalog::{
    CatalogEntry, CatalogError, CatalogPage, CatalogQuery, CatalogRefreshReport,
    CatalogSource, TrustPresentation,
};
