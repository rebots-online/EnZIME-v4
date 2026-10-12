Copyright (C) 2025–2026 Robin L. M. Cheung, MBA
All rights reserved.
Unauthorized use without prior written consent is strictly prohibited.

# AnZimmerman Architecture

**Copyright (C) 2025 Robin L. M. Cheung, MBA. All rights reserved.**

## Overview

AnZimmerman is a clean-room implementation of ZIM (Zeno IMproved) file format libraries across multiple programming languages. The architecture ensures no licensing contamination from existing ZIM implementations while providing full read/write capabilities.

## System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    AnZimmerman Project                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────┐│
│  │  zimlib.py  │  │ zimlib.ts   │  │  zimlib.go  │  │zimlib.php││
│  │   Python    │  │ TypeScript  │  │     Go      │  │   PHP   ││
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └────┬────┘│
│         │                │                │               │     │
│         └────────────────┴────────────────┴───────────────┘     │
│                              │                                  │
│                    ┌─────────▼─────────┐                       │
│                    │  ZIM File Format  │                       │
│                    │   Specification   │                       │
│                    └───────────────────┘                       │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    TOOLS                                │   │
│  │  ┌─────────────────────────────────────────────────┐   │   │
│  │  │  zim-downloader (Rust/Tauri)                    │   │   │
│  │  │  - TUI mode (ratatui)                           │   │   │
│  │  │  - GUI mode (Tauri tray app)                    │   │   │
│  │  └─────────────────────────────────────────────────┘   │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                     APIs                                │   │
│  │  Python (FastAPI) │ TypeScript (Express) │ PHP │ Go    │   │
│  │  All with Swagger UI documentation                      │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

## ZIM File Format Structure

### Header (80 bytes)

Per the [official ZIM specification](https://wiki.openzim.org/wiki/ZIM_file_format):

| Offset | Size | Field | Description |
|--------|------|-------|-------------|
| 0 | 4 | magicNumber | `0x044D495A` (72173914 decimal) |
| 4 | 2 | majorVersion | Major format version (5 or 6) |
| 6 | 2 | minorVersion | Minor format version |
| 8 | 16 | uuid | Unique archive identifier (16 bytes) |
| 24 | 4 | entryCount | Total number of entries |
| 28 | 4 | clusterCount | Total number of clusters |
| 32 | 8 | pathPtrPos | Position of directory pointers ordered by path |
| 40 | 8 | titlePtrPos | Position of directory pointers ordered by title (obsolete) |
| 48 | 8 | clusterPtrPos | Position of the cluster pointer list |
| 56 | 8 | mimeListPos | Position of the MIME type list (also header size) |
| 64 | 4 | mainPage | Main page index, or `0xFFFFFFFF` if none |
| 68 | 4 | layoutPage | Layout page index (deprecated, always `0xFFFFFFFF`) |
| 72 | 8 | checksumPos | Position of the MD5 checksum (16 bytes before EOF) |

> **Note (2026-02-08):** Previous versions of this document and all four language
> implementations contained an incorrect header layout that omitted the 16-byte UUID
> field at offset 8 and listed fabricated fields (`articleCount`, `redirectCount`,
> `clusterCountPos`) that do not exist in the ZIM specification. The magic number was
> also wrong (`0x4D495A5A` then `0x004D495A`). All implementations have been corrected
> to match the spec above. See the Changelog section below for details.

### Directory Entry Types

```
┌──────────────────────────────────────┐
│         Directory Entry              │
├──────────────────────────────────────┤
│ mimetype_index (4 bytes)             │
│ namespace (1 byte)                   │
│ revision (4 bytes)                   │
│ cluster_number (4 bytes)             │
│ blob_number (4 bytes)                │
│ url (null-terminated string)         │
│ title (null-terminated string)       │
└──────────────────────────────────────┘

┌──────────────────────────────────────┐
│         Redirect Entry               │
├──────────────────────────────────────┤
│ mimetype_index = 0xFFFF (4 bytes)    │
│ namespace (1 byte)                   │
│ revision (4 bytes)                   │
│ redirect_index (4 bytes)             │
│ url (null-terminated string)         │
│ title (null-terminated string)       │
└──────────────────────────────────────┘
```

### Namespaces

| Code | Description |
|------|-------------|
| `A` | Main articles |
| `I` | Images |
| `M` | Metadata |
| `-` | Raw data |
| `S` | CSS stylesheets |
| `J` | JavaScript |
| `T` | Fonts |

### Compression Types

| Code | Type |
|------|------|
| 0 | Default (none) |
| 1 | None |
| 2 | ZLIB |
| 3 | BZIP2 |
| 4 | LZMA |
| 5 | Zstandard |

## Component Details

### Core Libraries

Each library implements:

1. **BinaryReader/BinaryWriter** - Low-level binary I/O utilities
2. **ZIMHeader** - Header parsing and serialization
3. **DirectoryEntry/RedirectEntry** - Entry type handling
4. **ZIMReader** - Read operations with streaming support
5. **ZIMWriter** - Write operations with cluster management
6. **ZIMReaderBrowser** (TypeScript only) - ArrayBuffer-based browser support

### Data Flow

```
Read Flow:
┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
│  Open    │───▶│  Parse   │───▶│  Read    │───▶│ Decomp.  │
│  File    │    │  Header  │    │  Entry   │    │  Content │
└──────────┘    └──────────┘    └──────────┘    └──────────┘

Write Flow:
┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
│  Create  │───▶│   Add    │───▶│  Build   │───▶│ Finalize │
│  Writer  │    │ Articles │    │ Clusters │    │  Header  │
└──────────┘    └──────────┘    └──────────┘    └──────────┘
```

## Integration Points

### Chrome Extension "Zimmer"

- Uses `ZIMReaderBrowser` class from TypeScript library
- ArrayBuffer-based for in-memory ZIM handling
- Service Worker support for offline access

### Kiwix-Scale Backend

- REST APIs with Swagger documentation
- Microservice-ready architecture
- Large file streaming support

## Clean-Room Methodology

1. **Specification Only**: Implementations derived solely from openZIM format spec
2. **No Source Review**: No examination of existing ZIM library source code
3. **Independent Testing**: Test cases created from format documentation
4. **Audit Trail**: All design decisions documented

## Directory Structure

```
AnZimmerman/
├── zimlib.py          # Python library
├── zimlib.ts          # TypeScript library
├── zimlib.go          # Go library
├── zimlib.php         # PHP library
├── LICENSE            # All rights reserved
├── version.json       # Version/build info
├── api/               # REST API implementations
│   ├── python/
│   ├── typescript/
│   ├── php/
│   └── go/
├── docs/              # Per-language architecture docs
├── examples/          # Usage examples
├── scripts/           # Build scripts
└── TOOLS/
    └── zim-downloader/  # Rust/Tauri download tool
```

## Version Information

Version and build numbers follow the format: `MAJOR.MINOR.PATCH+BUILD`

- **BUILD**: Last 5 digits of epoch minutes (auto-generated)
- Run `scripts/build.sh` to update version across all components

---

*For detailed per-language implementation notes, see the individual architecture documents in `docs/`.*

---

## Changelog

### 2026-02-08 — Critical: ZIM Header Spec Compliance Fix

**Severity:** Critical (all ZIM files failed to load)

**Root Cause:** The ZIM header parser in all four language implementations was based on an
incorrect header layout that:

1. Used the wrong magic number (`0x4D495A5A`, later incorrectly "fixed" to `0x004D495A`)
2. Omitted the 16-byte UUID field at offset 8, shifting every subsequent field read by 16 bytes
3. Included fabricated fields (`articleCount`, `redirectCount`, `clusterCountPos`) that do not exist in the ZIM specification
4. Hardcoded byte offsets (e.g., offset 80 for directory pointers) instead of using header field values

**Verification:** The correct layout was verified by:
- Hex-dumping the header of `freecodecamp_en_all_2025-11.zim` (magic bytes: `5A 49 4D 04`)
- Cross-referencing with the [official ZIM specification](https://wiki.openzim.org/wiki/ZIM_file_format)

**Changes (TypeScript `zimlib.ts`):**

| Component | Before | After |
|-----------|--------|-------|
| `ZIM_MAGIC_NUMBER` | `0x4D495A5A` → `0x004D495A` | `0x044D495A` (72173914) |
| `ZIMHeader.uuid` | missing | `Buffer` (16 bytes at offset 8) |
| `ZIMHeader.articleCount` | present | removed (not in spec) |
| `ZIMHeader.redirectCount` | present | removed (not in spec) |
| `ZIMHeader.mimeTypeListPos` | at wrong offset | renamed `mimeListPos`, correct offset 56 |
| `ZIMHeader.urlPtrPos` | at wrong offset | renamed `pathPtrPos`, correct offset 32 |
| `ZIMHeader.titleIndexPos` | at wrong offset | renamed `titlePtrPos`, correct offset 40 |
| `ZIMHeader.clusterCountPos` | present | removed (not in spec) |
| `ZIMHeader.mainPageIndex` | at wrong offset | renamed `mainPage`, correct offset 64 |
| `ZIMHeader.layoutPageIndex` | at wrong offset | renamed `layoutPage`, correct offset 68 |
| `ZIMReader.readHeader()` | skipped UUID | reads 16-byte UUID at offset 8 |
| `ZIMReader.readDirectory()` | scanned past MIME list | uses `header.pathPtrPos` directly |
| `ZIMWriter.create()` | wrong magic, no UUID slot | correct magic, 16-byte UUID placeholder |
| `ZIMWriter.finalize()` | wrong magic, wrong layout | correct spec-compliant 80-byte header |
| `ZIMReaderBrowser.readHeader()` | wrong magic, no UUID | matches spec |
| `ZIMReaderBrowser.readDirectory()` | hardcoded offset 80 | uses `header.pathPtrPos` |
| `getMainPage()` | no sentinel check | checks `0xFFFFFFFF` (no main page) |

**Affected files:**
- `zimlib.ts` — ZIMReader, ZIMWriter, ZIMReaderBrowser classes
- `src/services/zimService.ts` — EnZIM application service (same fixes)

**Status:** TypeScript implementation fixed. Python, Go, and PHP implementations still need the same fix applied.

## Canonical Rust implementation (2026-10-12)

`rust/` is the maintained Rust implementation of this multilingual AnZimmermanLib.
It incorporates the existing EnZIME Rust implementation into the library rather
than maintaining a second ZIM library. `crates/zim-anzimmerman` in EnZIME is only
an application-port and identity adapter. Byte parsing, archive I/O, codecs and
read-only HTML text/link projection live here. That projection is not a security
sanitizer. This is a local maintained addition; the original vendor lock has not
been rewritten to claim upstream provenance.

Run `cargo test -p anzimmerman` from the EnZIME workspace. The portable fixture
`tests/fixtures/rust-consolidation.zim` and adjacent expected JSON preserve MIME
width, Unicode paths, redirect indices, payloads and SHA256. Regenerate explicitly
with `cargo run -p anzimmerman --example generate_shared_fixture`; tests never
rewrite the fixture. Existing TypeScript MIME-width corrections are preserved.

Cross-language follow-up: apply this fixture to Python, Go and PHP before changing
those readers; verify magic/header offsets, cluster-level decompression and
redirect bounds against the same bytes. Extended-offset flags and codec framing
need independent real-archive verification. In particular, historical raw-DEFLATE
comments are not sufficient evidence to propagate codec assumptions. No parity
claim for those languages is made by this Rust consolidation.
