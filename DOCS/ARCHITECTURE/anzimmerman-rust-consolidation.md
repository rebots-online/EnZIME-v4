# AZR — One AnZimmermanLib, canonical Rust implementation

Date: 2026-10-12. Status: architecture and implementation contract; no implementation claimed.
Authority: operator clarification and master architecture AMD-5. Scope is consolidation only.
Existing master architecture already records library ownership; this requested companion contains the exact migration contract.

## Ownership and evidence

Canonical Rust source root (R below): `VENDOR/AnZIMmermanLib-src/AnZimmermanLIB/AnZimmermanLib/rust`.
Adapter root (A below): `crates/zim-anzimmerman`.
Both original donor `/home/robin/forgejo/EnZIM/AnZimmermanLIB/AnZimmermanLib` and tracked vendor contain no other Rust reader; their Rust code is only `TOOLS/zim-downloader`. Current A is the existing Rust port to subsume, not discard. Original Python/Go match vendor; vendor TS contains valuable MIME-width corrections. No donor checkout mutation or network publication belongs to this task.

Copy/migrate existing Rust logic into R, replace A implementation bodies with forwarding. Preserve tracked file paths as compatibility surfaces. Canonical library owns every ZIM byte interpretation, local archive I/O/index/cache logic and generic HTML projection. EnZIME owns port conversion, application identity, UI rendering sanitation, IPC, persistence policy and DynDon selection. The current `sanitize.rs` is a read-only text/link projection, NOT a security sanitizer; move that generic projection unchanged into R, preserve its adapter wrapper. Future remote/DynDon changes target this canonical implementation.

## Entity table and API contract

New file line anchors use `:1` as insertion anchor; existing modules retain public names during migration.

| Exact entity | Target | Role / signature |
|---|---|---|
| `anzimmerman` | R/Cargo.toml:1 | Canonical Rust library package; crate name `anzimmerman` |
| `model` | R/src/model.rs:1 | Exact library-owned types below |
| `error::ZimError` | R/src/error.rs:1 | Exact error variants below |
| `identity::{ArticleKey,EntryKey,ContentDigest,DigestAlgorithm,DigestError}` | R/src/identity.rs:1 | Exact value contracts below |
| `archive::AnZimmermanZimArchive` | R/src/archive.rs:1 | Move current concrete archive, `Inner` and all file/pointer/directory/cache implementation; implement library `ZimArchive` |
| `archive::open_archive` | R/src/archive.rs:1 | `pub async fn open_archive(request: &model::ZimOpenRequest) -> Result<AnZimmermanZimArchive, error::ZimError>` |
| `archive::{ZimLayoutEntry,ZimLayoutSnapshot}` | R/src/archive.rs:1 | Move exact existing fields, replacing domain/port types with library-owned equivalents; preserve `layout_snapshot(&self) -> Result<ZimLayoutSnapshot,ZimError>` |
| `header::{ZimHeader,Uuid,parse_header,validate_header_offsets}` | R/src/header.rs:1 | Move existing public structs/constants/functions with identical fields/signatures except library error |
| `directory::{RawEntry,ArticleEntry,RedirectEntry,LinktargetEntry,DeletedEntry,parse_entry}` | R/src/directory.rs:1 | Move current definitions/accessors and `parse_entry(&[u8])->Result<(RawEntry,usize),ZimError>` |
| `cluster::{decompress_cluster,extract_blob,decompress_cluster_blobs}` | R/src/cluster.rs:1 | Move all existing codec/blob implementations and private helpers; preserve public signatures except error type |
| `limits` | R/src/limits.rs:1 | Move all existing constants unchanged; resource-budget improvements are separate work |
| `sanitize::{ArticleProjection,project}` | R/src/sanitize.rs:1 | Move projection and tests; `project(&bytes::Bytes)->ArticleProjection`, library-owned links |
| `AnZimmermanZimEngine` | R/src/lib.rs:1 | Move existing engine constructor and implement library `ZimEngine`; preserve engine ID/version |
| `AnZimmermanZimEngine` | A/src/lib.rs:1 | Existing EnZIME `ports::zim::ZimEngine` adapter, delegates canonical engine/open path |
| `archive::AnZimmermanZimArchive` | A/src/archive.rs:1 | Wrapper with `inner: Arc<anzimmerman::archive::AnZimmermanZimArchive>` and cached EnZIME metadata for borrowed `metadata()` |
| `archive::{open_archive,ZimLayoutEntry,ZimLayoutSnapshot}` | A/src/archive.rs:1 | Preserve current public signatures/fields exactly; convert canonical layout values |
| `convert::adapter_error` | A/src/convert.rs:1 | `pub(crate) fn adapter_error(anzimmerman::error::ZimError)->ports::zim::ZimError`; exhaustive variants, retain payloads |
| `convert::open_request` | A/src/convert.rs:1 | `pub(crate) fn open_request(&ports::zim::ZimOpenRequest)->anzimmerman::model::ZimOpenRequest` |
| `convert::metadata` | A/src/convert.rs:1 | `pub(crate) fn metadata(anzimmerman::model::ZimMetadata)->ports::zim::ZimMetadata` |
| `convert::entry` | A/src/convert.rs:1 | `pub(crate) fn entry(anzimmerman::model::ZimEntryDescriptor)->ports::zim::ZimEntryDescriptor` |
| `convert::article` | A/src/convert.rs:1 | `pub(crate) fn article(anzimmerman::model::ArticleDocument)->ports::zim::ArticleDocument` |
| `convert::resource` | A/src/convert.rs:1 | `pub(crate) fn resource(anzimmerman::model::ResourceDocument)->ports::zim::ResourceDocument` |
| `convert::hit` | A/src/convert.rs:1 | `pub(crate) fn hit(anzimmerman::model::TitleSearchHit)->ports::zim::TitleSearchHit` |
| `convert::report` | A/src/convert.rs:1 | `pub(crate) fn report(anzimmerman::model::ZimVerificationReport)->ports::zim::ZimVerificationReport` |
| `convert::projection` | A/src/convert.rs:1 | `pub(crate) fn projection(anzimmerman::sanitize::ArticleProjection)->crate::sanitize::ArticleProjection` |
| `convert::layout` | A/src/convert.rs:1 | `pub(crate) fn layout(anzimmerman::archive::ZimLayoutSnapshot)->crate::archive::ZimLayoutSnapshot` |
| compatibility `header`, `directory`, `cluster`, `limits`, `sanitize` | A/src/{header,directory,cluster,limits,sanitize}.rs:1 | Public reexports for unchanged values; wrappers map library errors and projection types; no parsing/decompression bodies |
| canonical conformance | R/tests/conformance.rs:1 and R/tests/support/mod.rs:1 | Copy existing generic conformance suite/builders, change imports to canonical types; retain adapter suite |
| adapter parity | A/tests/library_adapter.rs:1 | Same fixture through canonical and adapter APIs; assert document/metadata/layout/error equality |
| dependency boundary | A/tests/library_boundary.rs:1 | Parse canonical manifest using `toml` dev dependency; assert absence of domain/ports/tauri dependencies and expected adapter path |

### Library identity contracts

`ArticleKey(pub String)` and `EntryKey(pub String)` derive Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize; `as_str(&self)->&str` returns the string. They are archive paths, NOT opaque app IDs: never call EnZIME `ArticleKey::parse`/`EntryKey::parse` during conversion because those reject slashes. Convert string newtypes directly and preserve Unicode.
`DigestAlgorithm { Sha256 }` derives Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize with snake_case serde; `hex_len(self)->usize` returns 64.
`ContentDigest { pub algorithm: DigestAlgorithm, pub hex: String }` derives Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize. Methods `sha256_from_hex(hex: impl Into<String>)->Result<Self,DigestError>`, `hex(&self)->&str`, `algorithm(&self)->DigestAlgorithm`. Validate exactly 64 lowercase hex characters; `DigestError::MalformedDigest { expected: usize }`, deriving Clone, Debug, thiserror::Error, Eq, PartialEq. This retains useful digest logic without importing unrelated app identity validation.

### Exact library model, error and trait definitions

Use imports `async_trait::async_trait`, `bytes::Bytes`, `crate::identity::{ArticleKey,ContentDigest,EntryKey}`, `crate::error::ZimError`, `futures_core::Stream`, `serde::{Deserialize,Serialize}`, `std::{pin::Pin,sync::Arc}` as applicable. Reexport `model::*`, `error::ZimError`, and identity values from lib.rs. Keep these signatures to minimize migration and preserve async stream behavior; no new app semantics.

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ZimOpenRequest {
    pub path: String,
    pub expected_sha256: Option<String>,
    pub expected_size_bytes: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ZimMetadata {
    pub uuid: String,
    pub title: String,
    pub description: String,
    pub language_bcp47: String,
    pub creator: String,
    pub publisher: String,
    pub date: String,
    pub article_count: u64,
    pub media_count: u64,
    pub main_page: Option<ArticleKey>,
    pub checksum: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ZimEntryDescriptor {
    pub entry_key: EntryKey,
    pub namespace: char,
    pub path: String,
    pub title: String,
    pub mime_type: String,
    pub kind: ZimEntryKind,
    pub redirect_target: Option<EntryKey>,
    pub byte_length: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZimEntryKind {
    Article,
    Resource,
    Redirect,
    Metadata,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArticleDocument {
    pub article_key: ArticleKey,
    pub title: String,
    pub mime_type: String,
    pub raw_html: Bytes,
    pub normalized_text: String,
    pub content_digest: ContentDigest,
    pub internal_links: Vec<InternalLink>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InternalLink {
    pub href: String,
    pub target: InternalLinkTarget,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum InternalLinkTarget {
    Article { article_key: ArticleKey },
    Resource { entry_key: EntryKey },
    Unsupported { reason: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceDocument {
    pub entry_key: EntryKey,
    pub mime_type: String,
    pub bytes: Bytes,
    pub content_digest: ContentDigest,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TitleSearchQuery {
    pub text: String,
    pub offset: u32,
    pub limit: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TitleSearchHit {
    pub article_key: ArticleKey,
    pub title: String,
    pub score: f32,
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum ZimError {
    #[error("path is not readable")]
    UnreadablePath,
    #[error("archive length mismatch")]
    LengthMismatch,
    #[error("archive digest mismatch")]
    DigestMismatch,
    #[error("invalid ZIM header: {0}")]
    InvalidHeader(String),
    #[error("unsupported ZIM feature: {0}")]
    UnsupportedFeature(String),
    #[error("entry not found: {0}")]
    EntryNotFound(String),
    #[error("redirect cycle")]
    RedirectCycle,
    #[error("allocation limit exceeded")]
    AllocationLimit,
    #[error("decompression limit exceeded")]
    DecompressionLimit,
    #[error("I/O error: {0}")]
    Io(String),
}

pub type ZimEntryStream =
    Pin<Box<dyn Stream<Item = Result<ZimEntryDescriptor, ZimError>> + Send>>;

#[async_trait]
pub trait ZimEngine: Send + Sync {
    fn engine_id(&self) -> &'static str;
    fn engine_version(&self) -> String;
    async fn probe(&self, request: &ZimOpenRequest) -> Result<ZimMetadata, ZimError>;
    async fn open(
        &self,
        request: ZimOpenRequest,
    ) -> Result<Arc<dyn ZimArchive>, ZimError>;
}

#[async_trait]
pub trait ZimArchive: Send + Sync {
    fn metadata(&self) -> &ZimMetadata;
    async fn main_page(&self) -> Result<Option<ArticleKey>, ZimError>;
    async fn entry(&self, key: &EntryKey) -> Result<ZimEntryDescriptor, ZimError>;
    async fn article(&self, key: &ArticleKey) -> Result<ArticleDocument, ZimError>;
    async fn resource(&self, key: &EntryKey) -> Result<ResourceDocument, ZimError>;
    async fn resolve_redirect(
        &self,
        key: &EntryKey,
        max_hops: u8,
    ) -> Result<EntryKey, ZimError>;
    async fn search_titles(
        &self,
        query: TitleSearchQuery,
    ) -> Result<Vec<TitleSearchHit>, ZimError>;
    async fn entries(&self) -> Result<ZimEntryStream, ZimError>;
    async fn verify_structure(&self) -> Result<ZimVerificationReport, ZimError>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ZimVerificationReport {
    pub valid: bool,
    pub engine_id: String,
    pub engine_version: String,
    pub checked_entries: u64,
    pub warnings: Vec<ZimVerificationFinding>,
    pub errors: Vec<ZimVerificationFinding>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ZimVerificationFinding {
    pub code: String,
    pub entry_key: Option<EntryKey>,
    pub detail: String,
}

```

Place the error enum in error.rs, other above declarations in model.rs. Imports above are split accordingly. Existing `ZimLayoutEntry`/`ZimLayoutSnapshot` definitions migrate verbatim from A archive.rs with library identities/types.

## Adapter conversion rules

All fields are mapped explicitly; no JSON serialization round trips. Convert key strings losslessly, digest enum/hex unchanged, raw Bytes by move/cheap clone. Exhaustively match entry kinds, link targets and errors. Preserve order, redirect limits and offsets. Map query fields directly. `entries()` maps the canonical stream one item at a time, without collecting it. `metadata()` returns adapter-cached converted metadata because the port returns a borrowed reference. `main_page` and `resolve_redirect` convert string wrappers directly. `layout_snapshot()` converts each existing result; removing full-layout eagerness is explicitly a successor, not silently introduced here. Compatibility `header`/`directory` functions delegate and map errors, retaining public type identities through reexports. `cluster` functions delegate and map errors; `extract_blob` retains its borrowed output lifetime. `sanitize::project` delegates generic projection and converts links. The adapter contains no byte-offset parsing, codec dispatch or filesystem reader.

## Cargo and provenance

Add R to root `[workspace].members`. R uses explicit package fields: name `anzimmerman`, version `1.0.0`, edition `2021`, rust-version `1.82`, license-file `../LICENSE`, authors `["Robin L. M. Cheung, MBA"]`, publish=false. Do not inherit EnZIME application version. Move required third-party dependencies from A with existing versions (`async-trait`, `async-stream`, `bytes` serde feature, `futures-core`, `futures-util`, `ego-tree`, `scraper`, `serde` derive, `sha2`, `hex`, `flate2`, `zstd`), add `thiserror = "1"` for library errors. Preserve tempfile/tokio test dependencies. A adds `anzimmerman = { path = "../../VENDOR/AnZIMmermanLib-src/AnZimmermanLIB/AnZimmermanLib/rust" }`; remove now-unused codec/parser dependencies; retain domain/ports and adapters' direct dependencies. Add `toml = "0.8"` to A dev-dependencies for dependency-boundary test. Let Cargo update lockfile locally.

Preserve original-library copyright and license. Register Rust in original README and ARCHITECTURE, explaining port lineage and single canonical location. Do not mutate the upstream vendor lock to pretend it already contained the new port: document the local maintained addition. Do not edit original donor checkout. Source version/transport publication is outside this task.

## Cross-language applicability matrix

| Behavior | Current evidence | Consolidation action / later parity |
|---|---|---|
| 80-byte header, magic and UUID offsets | Rust and corrected TS agree; Python reader still checks obsolete magic | Preserve Rust; record Python/Go/PHP verification requirement |
| MIME uint16 plus parameter byte | Vendor TS corrected relative to donor | Preserve vendor correction; shared fixture below exercises it |
| Decompress cluster then interpret offsets | Rust implements cluster-level extraction; original TS/Python/Go methods differ | Preserve Rust behavior; later repair other languages with same fixture |
| Extended-offset/compression flags | Rust currently rejects some forms; other readers differ | No new codec behavior here; explicit successor coverage |
| Redirect numeric indices and bounds | Existing Rust suite covers redirects | Preserve exact outputs; apply same fixture to other languages later |
| Codec framing | Existing Rust raw-DEFLATE comment is not independent verification | Do not propagate an assumption; verify against real archives before changing languages |
| App opaque-ID validation | App parser rejects archive path syntax | Keep library paths independent; direct wrapper conversion only |

Shared portable regression data lives at original-library `tests/fixtures/rust-consolidation.zim` plus `rust-consolidation.expected.json`. Generate deterministically from existing Rust fixture builder: uncompressed article `A/Café` (title `Café`, MIME `text/html`), HTML `<p>Library fixture</p>`, redirect `A/Home` (title `Home`) targeting it, resource `I/pixel.txt` (title `pixel.txt`, MIME `text/plain`) with bytes `pixel`, valid MIME table and header; main page points to `A/Home`, UUID is sixteen zero bytes. JSON names entry keys, title, MIME, redirect destination and raw UTF-8 payloads; fixture SHA256 recorded in JSON. Add canonical test `shared_fixture_matches_expected` reading checked-in bytes/JSON (serde_json dev dependency). Generation is preparation, not test-time mutation. Both canonical and adapter tests consume these bytes; later language parity reuses them. Preserve existing codec/malformed-input fixtures in canonical conformance tests.

## Verification and scope exit

Run `cargo test -p anzimmerman`, `cargo test -p zim-anzimmerman`, `cargo test -p zim-cluster-cache`, then `cargo check -p reader`. Expected: all tests pass and reader check succeeds. Adapter parity asserts Unicode article/resource bytes, digest, metadata, main-page redirect, layout mappings and malformed-header errors. Boundary test ensures library depends on no application crate and adapter targets R. Inspect A modules to confirm only forwarding/conversion remains. This closes ownership consolidation only: no claim of remote Kiwix acquisition, memory bounds, restart durability or reader annotations follows from it.
