# Native Kiwix catalog acquisition

2026-10-12. Scope: `crates/catalog-openzim/` only. The parent integrates these APIs into IPC and durable stores. Catalog browsing downloads bounded XML metadata, never a ZIM archive. Existing signed JSON/library adapters retain their contracts.

## Decision

Use the real Kiwix OPDS feed, then resolve the selected acquisition Metalink. OPDS does not publish a SHA-256; its displayed length can be rounded. Metalink exact size and optional published digests are separate facts. Published digests are not signatures and have not verified downloaded bytes. The existing mandatory-digest CatalogEntry cannot truthfully represent unresolved OPDS; separate typed models are intentional until the parent adapts the application boundary.

## Entity contract

All new entities are in `crates/catalog-openzim/src/kiwix.rs:1` (new module insertion), exported from `src/lib.rs:26`.

| Entity | Exact contract / role |
|---|---|
| KiwixQuery | text: Option<String>, language: Option<String>, offset: u32, limit: u32; Default 20 results; max 100, reject zero or >100; URL query encoding |
| KiwixEntry | id, title, description, language, name, updated, publisher, category, acquisition_url: String; advertised_size_bytes: u64; browse_url: Option<String>; no invented digest |
| KiwixPage | entries: Vec<KiwixEntry>, total: Option<u64>, offset: u32, next_offset: Option<u32> |
| PublishedDigest | algorithm, hex: String; supported sha-256, sha-1, md5; valid exact hex widths |
| KiwixAcquisition | file_name: String, size_bytes: u64, mirrors: Vec<String>, digests: Vec<PublishedDigest>; sha256() -> Option<&str> |
| KiwixCatalogClient | new(Arc<dyn TransferSource>) -> Self; with_endpoint(Arc<dyn TransferSource>, &str) -> Result<Self,CatalogError>; search(&KiwixQuery) async -> Result<KiwixPage,CatalogError>; resolve(&KiwixEntry) async -> Result<KiwixAcquisition,CatalogError> |
| parse_kiwix_opds | (&[u8], &str) -> Result<KiwixPage,CatalogError>; namespace-aware feed/parser, bounded depth/bytes/nodes, mandatory identifiers/size/link, no DTD |
| parse_kiwix_metalink | (&[u8], &str) -> Result<KiwixAcquisition,CatalogError>; exactly one file, HTTPS mirrors stable priority order/dedup, exact positive size, validate published digests; no full transfer |
| XmlNode / parse_xml | private bounded namespace-aware XML tree using existing quick-xml; named child helpers, text and attributes; reject unknown prefixes, DTD, multiple roots and unsupported encoding |
| https_url / required / number | private validators shared by OPDS and Metalink parsing |

The default endpoint is `https://library.kiwix.org/catalog/v2/entries`. Query `q`, `lang`, `start`, `count` use url::Url query_pairs_mut. Document ceiling 4 MiB OPDS, 8 MiB Metalink, depth 32, nodes 100000. HTTPS URL validators reject credentials/fragments. Resolve fetches only .meta4 documents; direct HTTPS ZIM links resolve metadata without downloading the file, carrying no digest. HTTP mirrors are skipped; no usable HTTPS mirror is an error. Native TransferSource owns redirect/transport policy. Application pins source ETag/size independently before DynDon ranges.

## Checklist and acceptance rubric

- ✅ KC1. Implement all entity rows in new `src/kiwix.rs`, export from `src/lib.rs`. Reuse existing fetch_bytes stream ceiling; do not modify CatalogSource/TransferSource or old adapters. Add unit tests in `src/kiwix/tests.rs` for namespaces (including prefixed Atom), entity decoding, pagination, encoded query, exact Metalink size independent of OPDS rounded size, digest validation/absence, mirror priority/HTTPS filtering, malicious DTD/depth/oversize, and asynchronous transfer bound with mock transport. Verify `cargo test -p catalog-openzim`. Accept all old and new tests pass, exported APIs compile.
- [X] KC2. Add maintained `examples/kiwix_catalog.rs` using an injected native HTTP adapter as dev dependency if already workspace member; supports query and resolves first selection, prints metadata only. Verify live example yields actual Kiwix results and real HTTPS mirrors/digest, no ZIM bytes fetched. This is an operator verification protocol, not idempotent checklist acceptance.

## Evidence

Live OPDS Wikipedia100: advertised 333430784 bytes; Metalink exact 333429922 bytes; published SHA-256 2eb7fd115bfb8644a08a9482f01f52b507428b1b8a30129fead92f7b6715440b. Observed via bounded native HTTPS on 2026-10-12. This is metadata evidence, not archive download/verification.

CodeGraph covers Rust source. Its explicit no-index match for Cargo.toml allowed the targeted manifest read. No Forgejo operations; parent owns GitHub publication and root checklist integration.

Verified 2026-10-12: `cargo test -p catalog-openzim` passed 21 unit tests (6 new Kiwix cases) and 13 existing adapter integration tests. `cargo run -p catalog-openzim --example kiwix_catalog -- wikipedia` exited 0 against live HTTPS, returned total=65, next_offset=2 and two entries, resolved Wikipedia100 into five HTTPS mirrors, exact size 333429922 and the SHA-256 above. No archive transfer was requested. Existing transfer-http unused-import warning remains outside this task. Parent still must integrate IPC, persistence, source identity pinning and DynDon; these results do not certify an installed reader artifact.
