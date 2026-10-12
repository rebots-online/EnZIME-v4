//! Bounded budgets that protect the reader from malformed or hostile ZIM data.
//!
//! Every cap here is a hard guard: when an input would require an allocation
//! or decompression product exceeding the cap, the reader returns a declared
//! `ZimError` variant instead of allocating.

/// Maximum decompressed size of a single cluster.
///
/// 2 GiB is large enough for the biggest clusters in production Wikipedia
/// archives (typically <500 MiB uncompressed) and small enough that a
/// decompression bomb is contained on commodity hardware.
pub const MAX_DECOMPRESSED_CLUSTER_BYTES: u64 = 2 * 1024 * 1024 * 1024;

/// Maximum compressed size we are willing to read for a single cluster
/// before treating the cluster pointer table as corrupt.
pub const MAX_COMPRESSED_CLUSTER_BYTES: u64 = 4 * 1024 * 1024 * 1024;

/// Maximum number of bytes for any single blob extracted from a cluster.
pub const MAX_BLOB_BYTES: u64 = 512 * 1024 * 1024;

/// Maximum number of MIME types we will accept from the MIME list. The
/// ZIM format reserves indices 0xFFFD / 0xFFFE / 0xFFFF for non-blob
/// entries, so the largest legal MIME index is 0xFFFC = 65_532.
pub const MAX_MIME_TYPES: u32 = 65_530;

/// Maximum byte length of a single directory entry's `url` field.
pub const MAX_URL_BYTES: usize = 2_048;

/// Maximum byte length of a single directory entry's `title` field.
pub const MAX_TITLE_BYTES: usize = 2_048;

/// Upper bound on the number of bytes read at one directory-entry offset.
/// Sums the fixed header (16 bytes) with the two capped NUL-terminated
/// strings; entries that exceed this are unparseable.
pub const MAX_ENTRY_READ_BYTES: u64 = (16 + MAX_URL_BYTES + MAX_TITLE_BYTES + 2) as u64;

/// Maximum in-memory allocation permitted when reading any of the indexed
/// tables (path pointer table, cluster pointer table, MIME list).
pub const MAX_INDEX_ALLOCATION_BYTES: u64 = 256 * 1024 * 1024;

/// Minimum size of any valid ZIM file: the 80-byte header alone.
pub const MIN_ZIM_FILE_BYTES: u64 = 80;

/// Streaming SHA-256 verification buffer size.
pub const HASH_CHUNK_BYTES: usize = 64 * 1024;

/// Streaming buffer size used for chunked decompression.
pub const DECOMPRESS_CHUNK_BYTES: usize = 64 * 1024;
