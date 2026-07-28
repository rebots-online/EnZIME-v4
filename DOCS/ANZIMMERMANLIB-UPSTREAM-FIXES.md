# AnZIMmermanLib — Upstream Fixes Found by P2-01

This file records defects in the vendored clean-room ZIM library
(`VENDOR/AnZIMmermanLib-src/AnZimmermanLIB/AnZimmermanLib/`) discovered
while implementing the Rust port (`crates/zim-anzimmerman`). They are
documented so they can be propagated upstream into the TypeScript, Python,
Go and PHP sister implementations.

The Rust port does NOT depend on the upstream implementations at runtime —
it is a fresh Rust translation. The findings below were observed during the
translation work.

## Finding 1 — Cluster blob-offset-table terminator is wrong in TS reader

**File:** `zimlib.ts`, `ZIMReader.getArticleContent` (around line 437).

**Upstream behaviour:** the cluster blob-offset loop terminates when it
reads a u32 offset equal to `0`:

```ts
const blobOffsets: number[] = [];
while (true) {
    const offset = clusterReader.readUInt32LE();
    blobOffsets.push(offset);
    if (offset === 0) break; // Last offset
}
```

**Spec-correct behaviour:** per the openZIM cluster format, a cluster
contains `N+1` offsets for `N` blobs — the last offset is the byte length
of the decompressed cluster payload, not `0`. Offset `0` never appears in
a valid cluster (offset[0] is the start of blob 0, which is always past
the offset table itself).

**Consequence for upstream:** loading any cluster where no offset
happens to equal `0` would loop until it hit EOF or random zero bytes in
compressed data, then slice the blob with a meaningless `blobNumber`
index. Real-world ZIMs sometimes work because their first blob's offset
table happens to contain a `0` from later garbage bytes — pure luck.

**Fix applied in the Rust port** (`crates/zim-anzimmerman/src/cluster.rs`,
`extract_blob`):

```rust
let first_offset = le_u32(decompressed, 0) as usize;
if first_offset == 0 || first_offset % 4 != 0 {
    return Err(ZimError::InvalidHeader(/* ... */));
}
let offset_count = first_offset / 4;
let blob_count = offset_count - 1;
```

The first offset encodes the offset-table byte length; dividing by four
yields the offset count, and `blob_count = offset_count - 1`. The same
fix should be applied to the TS/Python/Go/PHP readers.

## Finding 2 — `usize.to_le_bytes()` produces platform-dependent-width bytes

**Not an upstream defect per se** — a hazard that bit the Rust test
fixtures during P2-01 development. Calling `to_le_bytes()` on `usize`
emits 8 bytes on 64-bit targets and 4 on 32-bit, silently misaligning
any on-disk structure that expects `u32`. The Rust port casts explicitly
to `u32` before serialising any little-endian ZIM integer. Upstream
implementations in dynamic languages don't have this footgun, but a
future Rust port should know.

## Finding 3 — MIME list terminator handling for empty lists

**Upstream behaviour:** the MIME-type-list writer in `zimlib.ts`
(`ZIMWriter.finalize`) emits one terminating `\0` after the last MIME
string. For the (uncommon but spec-legal) empty MIME list this yields
just `\0`, but the reader searches for the required `\0\0` double-NUL.
On a file with no MIME types the reader therefore over-runs the list
into the directory entries.

**Fix applied in the Rust port's test builder:**
`crates/zim-anzimmerman/tests/support/mod.rs` always emits at least
`\0\0` for the MIME list, even when no MIME types are declared. The
reader (`crates/zim-anzimmerman/src/archive.rs`, `read_mime_list`)
correctly searches for `\0\0` and rejects the file if the terminator
is missing.

## Finding 4 — LZMA codec left unimplemented

**Upstream behaviour:** `zimlib.ts` `getArticleContent` throws
`'LZMA compression not implemented in this clean-room version'` for
LZMA clusters. Python/Go/PHP are believed to behave similarly.

**Rust port behaviour:** returns
`ZimError::UnsupportedFeature("LZMA cluster codec not wired in this build")`.
Same outcome, different idiom. A future task should add `xz2` (or
equivalent) raw-LZMA1 decoding so archives produced by older libzim
versions are readable; modern ZIMs default to ZSTD so this is not
blocking.

---

Documented by P2-01 on 2026-07-27.
