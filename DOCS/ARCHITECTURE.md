# EnZIME Reader — Complete Product Architecture

**Document ID:** `enzime-architecture-v4`
**Specification version:** `4.0.0`
**Date:** 2026-07-26
**Status:** Normative implementation contract
**Product:** EnZIME Reader
**Platforms:** Android, Linux, Windows
**Application shell:** Tauri 2
**Frontend:** React 19, TypeScript, Vite, Zustand
**Backend:** Rust, SQLite, content-addressed object storage
**Normative database schema:** `enzime.sqlite.v1`
**Normative sidecar schema:** `enzime.sidecar.cbor.v1`
**Normative IPC schema:** `enzime.ipc.v1`

> This document specifies a complete release product.
> Product code MUST NOT contain a stub, mock-only path, bypass gate, dead-end
> caption, disabled production control, or “implemented later” placeholder.
> Test doubles may exist only under test-only compilation and may never be
> selected by a release composition.

---

## 0. Normative reading rule

The words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT and MAY
have their RFC 2119 meanings.

The implementation authority order is:

1. this `ARCHITECTURE.md`;
2. machine-readable files included in this reboot pak;
3. the canonical `PRD.md` included in this reboot pak;
4. explicitly locked vendored source listed in `VENDOR-MANIFEST.md`;
5. fresh implementation decisions recorded as ADRs without changing a
   pre-committed public entity.

If two authorities conflict, implementation stops.
The conflict is resolved in the higher authority.
No coder may invent a second name for an entity already named here.

The reboot is closed-world.
A fresh implementation agent receives:

- this reboot pak;
- the exact vendored source tree declared by `VENDOR-MANIFEST.md`;
- ordinary language/runtime package registries;
- no previous EnZIME repository;
- no prior chat transcript;
- no operator memory as an undocumented dependency;
- no absolute workstation path;
- no hidden credential;
- no historical checklist completion state.

Passing `audits/cold-start-audit.md` is a release-blocking precondition.

---

# 1. Executive Summary

EnZIME Reader is an offline-first knowledge reader, annotator and local
conversation system for ZIM archives and EnZIME packs.

The product promise is:

> Acquire knowledge once, read and annotate it without a network, ask a local
> model questions with inspectable citations, and retain the user’s work even
> when an archive, model, vendor or device changes.

The release product provides:

- verified ZIM import and reading;
- catalog browsing and durable, resumable acquisition;
- storage planning before bytes are transferred;
- safe article rendering with internal navigation;
- bookmarks, highlights, comments, drawings and voice annotations;
- portable signed sidecars;
- local lexical and vector retrieval;
- cited local conversations;
- model discovery, installation, selection and lifecycle control;
- user-governed lore memory with correction, pinning and deletion;
- offline entitlement verification;
- backup, restore and diagnostic export;
- responsive desktop and Android interfaces from one typed contract.

The product remains useful without:

- an account;
- an internet connection;
- an installed inference model;
- a vector index;
- a paid entitlement;
- a sibling EnZIME application;
- access to a previous source repository.

The minimum release is not a shell.
Every visible control must complete its named action or be absent.

## 1.1 Platform posture

| Concern | Android | Linux | Windows |
|---|---|---|---|
| Shell | Tauri 2 Android | Tauri 2 desktop | Tauri 2 desktop |
| ZIM access | SAF + app storage | native paths | native paths |
| SQLite | bundled | bundled | bundled |
| Local inference | wllama3 | wllama3 | wllama3 |
| Entitlement online source | RevenueCat adapter | RevenueCat web restore adapter | RevenueCat web restore adapter |
| Offline entitlement | signed local artifact | signed local artifact | signed local artifact |
| Background transfer | foreground service | process job | process job |
| Sidecar share | Android share sheet/SAF | native file dialog | native file dialog |
| Auto-update | store/signed channel | signed channel | signed channel |

## 1.2 Data posture

Primary user data:

- annotations;
- annotation revisions;
- notebooks and tags;
- conversation messages authored by the user;
- accepted lore memory and its corrections;
- selection profiles;
- entitlement artifacts;
- trust keys;
- backup manifests.

Rebuildable data:

- title/full-text indexes;
- embeddings;
- retrieval chunk projections;
- thumbnails;
- capability probes;
- response caches.

Replaceable assets:

- ZIM archives with verified locators;
- model binaries with verified manifests;
- catalog snapshots;
- downloaded but uncommitted chunks.

Primary user data is never coupled to an evictable archive or model binary.

---

# 2. System Context

## 2.1 C4 Level 1

```mermaid
flowchart TD
    U["Reader / researcher"] --> E["EnZIME Reader"]
    C["Curator / pack manager"] --> E
    E --> Z["Local ZIM and pack files"]
    E --> S["Approved catalogs and mirrors"]
    E --> R["RevenueCat entitlement source"]
    E --> F["Local model runtime"]
    E --> O["User-selected backup or sidecar destination"]
```

No external system is required for local reading, annotation, search, local
inference over already installed assets, entitlement verification or export.

## 2.2 Personas

### `OfflineReaderPersona`

Needs fast local search, safe article navigation, bookmarks and durable notes.

### `KnowledgeWorkerPersona`

Needs cross-pack annotations, notebooks, cited conversation and memory review.

### `StorageConstrainedPersona`

Needs size visibility, tailored selection and transactional pack replacement.

### `PreparednessPersona`

Needs capability readiness, integrity status and predictable airplane-mode use.

### `AccessibilityPersona`

Needs keyboard completion, screen-reader semantics, scalable typography,
reduced motion and non-colour-only status.

## 2.3 External systems

| System ID | Role | Trust level | Required offline |
|---|---|---|---|
| `ext.openzim.catalog` | catalog metadata | signed/verified or visibly unverified | No |
| `ext.http.mirror` | range/full artifact transfer | bytes verified before activation | No |
| `ext.local.file` | import source | untrusted until verified | Yes |
| `ext.revenuecat` | online purchase report | vendor event source only | No |
| `ext.os.keystore` | device secret protection | platform trusted boundary | Yes |
| `ext.os.share` | explicit import/export | user-authorized boundary | Yes |
| `ext.wllama` | on-device inference runtime | replaceable engine | Yes when installed |

## 2.4 Trust boundaries

```mermaid
flowchart TD
    UI["React UI"] --> IPC["Validated Tauri IPC"]
    IPC --> APP["Application services"]
    APP --> DB["SQLite + object store"]
    APP --> ISO["Isolated article renderer"]
    APP --> ENGINES["Replaceable engines"]
    NET["Untrusted network/files"] --> VERIFY["Verifier + quarantine"]
    VERIFY --> DB
```

The article renderer has no Tauri command bridge.
The renderer communicates through a restricted parent mediation channel whose
message schema is `RendererMessageV1`.

---

# 3. Architectural Layers

The architecture has three rings.

## 3.1 Ring 1 — domain and durable data

Ring 1 contains:

- stable identities;
- domain value objects;
- state machines;
- repository ports;
- SQLite schema;
- object-store rules;
- canonical CBOR/JSON schemas;
- migration rules;
- capability policy;
- entitlement policy;
- retrieval/citation records;
- user-lore policy.

Ring 1 has no dependency on:

- Tauri;
- React;
- RevenueCat;
- libzim;
- wllama3;
- Android;
- Windows;
- Linux;
- HTTP clients.

## 3.2 Ring 2 — replaceable engines

Ring 2 contains adapters behind ports:

- `ZimEngine`;
- `CatalogSource`;
- `TransferSource`;
- `InferenceBackend`;
- `EmbeddingBackend`;
- `SignatureProvider`;
- `EntitlementSource`;
- `PlatformFileAccess`;
- `AudioCaptureAdapter`;
- `BackupTarget`.

Ring 2 depends inward on Ring 1 contracts.
Ring 1 never imports Ring 2 implementations.

## 3.3 Ring 3 — platform delivery

Ring 3 contains:

- Tauri runtime composition;
- command handlers;
- event publishing;
- React views;
- Zustand stores;
- native file/share/foreground-service adapters;
- packaging;
- signing;
- update channels.

Ring 3 selects complete adapters.
A release build fails when any required capability is unbound.

## 3.4 Dependency graph

```mermaid
flowchart TD
    FE["packages/app-ui"] --> CT["packages/contracts"]
    TA["apps/tauri"] --> AP["crates/application"]
    AP --> DO["crates/domain"]
    AD["crates/adapters-*"] --> PO["crates/ports"]
    AP --> PO
    TA --> AD
    ST["crates/storage-sqlite"] --> PO
```

---

# 4. Binding Invariants

## A-01 — Closed-world bootstrap

The reboot pak plus locked vendored source MUST be sufficient to begin.
No task may require an unspecified historical file, repository, conversation,
absolute path or remembered decision.

Verification:

```text
Create a clean directory.
Copy only the reboot pak and vendor tree.
Run the cold-start audit.
Resolve every referenced file and entity locally.
```

## A-02 — Ports before providers

Domain/application code imports stable ports.
Vendor and platform implementations import those ports.
No domain/application module imports a provider crate or SDK.

## A-03 — Platform graph isolation

Android, Linux and Windows ship one codebase under a single release
composition (§5.2).
There is no per-platform Cargo feature and no default feature that omits a
crate.
Every platform's dependency graph is identical; a graph difference is a build
failure.
Platform difference is expressed ONLY by `#[cfg(target_os = "...")]` inside
`crates/platform-adapter`, never by a feature that removes a user-visible
capability.

## A-04 — Last-known-good activation

Downloads, model installs, archive updates, database migrations and restores
stage and verify before one atomic activation step.
Failure leaves the last known good state usable.

## A-05 — Identity is not location

Pack, edition, article, annotation and citation identities do not contain a file
path, URL, SAF URI or mount point.
Locations are versioned hints attached to stable identities.

## A-06 — User work outlives sources and engines

Annotations, notebooks, accepted lore, entitlement evidence and exports survive
archive eviction, model removal, index rebuild, provider failure and app update.

## A-07 — Untrusted bytes never become active

Network, removable-media and imported bytes remain in staging or quarantine
until length, digest, signature policy and format validation pass.

## A-08 — AI is bounded and cited

Inference receives an explicit scope and bounded evidence set.
Every citation maps to stored retrieval evidence.
Model output cannot silently become accepted lore.

## A-09 — Commerce cannot revoke ownership

RevenueCat is an online event source.
The local capability decision is derived from a verified signed entitlement
artifact and policy.
Network/provider failure does not disable already granted perpetual capability.

## A-10 — UI follows the entity contract

Stitch, React and visual design do not invent backend semantics.
Every stateful UI component maps through the entity-to-UI binding table to:

- one TypeScript entity;
- one or more named Tauri commands;
- a declared SQLite or non-SQL source;
- one Zustand slice.

No screen generation result may rename a pre-committed entity.

---

# 5. Repository Structure

```text
EnZIME/
├── Cargo.toml
├── Cargo.lock
├── package.json
├── pnpm-lock.yaml
├── pnpm-workspace.yaml
├── rust-toolchain.toml
├── deny.toml
├── LICENSES/
├── VENDOR/
│   ├── LOCKFILE.md
│   ├── wllama3/
│   ├── tauri/
│   └── AnZIMmermanLib/
├── apps/
│   └── reader/
│       ├── package.json
│       ├── index.html
│       ├── vite.config.ts
│       ├── src/
│       │   ├── main.tsx
│       │   ├── App.tsx
│       │   ├── router.tsx
│       │   ├── routes/
│       │   ├── components/
│       │   ├── stores/
│       │   ├── services/
│       │   ├── styles/
│       │   └── test/
│       └── src-tauri/
│           ├── Cargo.toml
│           ├── build.rs
│           ├── tauri.conf.json
│           ├── capabilities/
│           ├── gen/android/
│           ├── icons/
│           └── src/
│               ├── main.rs
│               ├── lib.rs
│               ├── app_state.rs
│               ├── commands/
│               ├── events/
│               └── composition/
├── crates/
│   ├── domain/
│   ├── ports/
│   ├── application/
│   ├── storage-sqlite/
│   ├── object-store/
│   ├── zim-anzimmerman/
│   ├── catalog-openzim/
│   ├── transfer-http/
│   ├── transfer-file/
│   ├── annotations/
│   ├── sidecar/
│   ├── inference-wllama/
│   ├── retrieval/
│   ├── lore/
│   ├── entitlement/
│   ├── revenuecat-adapter/
│   ├── platform-adapter/
│   ├── backup/
│   └── diagnostics/
├── packages/
│   ├── contracts/
│   ├── design-tokens/
│   ├── ui-primitives/
│   └── test-fixtures/
├── schemas/
│   ├── sqlite/
│   │   └── 0001_initial.sql
│   ├── sidecar/
│   │   └── enzime.sidecar.cddl
│   ├── entitlement/
│   │   └── enzime.entitlement.cddl
│   └── ipc/
│       └── enzime.ipc.schema.json
├── fixtures/
│   ├── zim/
│   ├── catalogs/
│   ├── sidecars/
│   ├── retrieval/
│   ├── entitlement/
│   └── malicious/
├── tests/
│   ├── conformance/
│   ├── integration/
│   ├── e2e/
│   └── performance/
├── tools/
│   ├── contract-gen/
│   ├── fixture-builder/
│   ├── evidence-registry/
│   └── release/
└── DOCS/
    ├── PRD.md
    ├── ARCHITECTURE.md
    ├── CHECKLIST.md
    ├── TEST-RUBRIC.md
    ├── THREAT-MODEL.md
    ├── PRIVACY.md
    ├── THIRD-PARTY-NOTICES.md
    ├── adr/
    └── diagrams/
```

## 5.1 File purposes

| Path | Purpose |
|---|---|
| `crates/domain` | entities, value objects, policies, state enums |
| `crates/ports` | traits used by application services |
| `crates/application` | complete use cases and orchestration |
| `crates/storage-sqlite` | migrations and repository implementations |
| `crates/object-store` | staging, verification, atomic object activation |
| `crates/zim-anzimmerman` | `ZimEngine` implementation over vendored library |
| `crates/annotations` | selector capture, resolution and revision services |
| `crates/sidecar` | canonical CBOR, signing, import/export |
| `crates/retrieval` | lexical/vector retrieval, budgeting and citation map |
| `crates/lore` | extraction, review, consolidation and correction |
| `crates/entitlement` | signed artifact and capability policy |
| `apps/reader/src-tauri` | command/event delivery only |
| `packages/contracts` | generated and checked-in TypeScript contracts |
| `packages/design-tokens` | Aurora Glass tokens |

## 5.2 Cargo feature compositions

There is exactly one release composition, shared by Android, Linux and
Windows. No Cargo feature removes a user-visible capability from any platform.

```toml
[features]
default = [
  "dep:platform-adapter",
  "dep:inference-wllama",
  "dep:revenuecat-adapter"
]
```

`crates/platform-adapter` exposes the §17.2 platform port and selects
per-target behaviour with `#[cfg(target_os = "...")]` modules
(`android.rs`, `linux.rs`, `windows.rs`).

Release composition assertions:

- there is no per-platform feature; `default` is the only release composition;
- every platform's dependency graph is identical; a graph difference is a build
  failure;
- all platforms include `zim-anzimmerman`, `storage-sqlite`, `annotations`,
  `sidecar`, `inference-wllama`, `retrieval`, `lore`, `entitlement`,
  `revenuecat-adapter`, `platform-adapter`, `backup`, `diagnostics`.

---

# 6. Canonical Storage Model

## 6.1 SQLite rules

Database file:

```text
<app-data>/EnZIME/metadata/enzime.sqlite3
```

Required pragmas:

```sql
PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;
PRAGMA synchronous = FULL;
PRAGMA busy_timeout = 5000;
PRAGMA trusted_schema = OFF;
PRAGMA temp_store = MEMORY;
```

Time columns use Unix epoch milliseconds as `INTEGER`.
Boolean columns use `INTEGER NOT NULL CHECK(value IN (0,1))`.
IDs use lowercase canonical ULID text unless an entity explicitly uses a digest.
Digests use lowercase hexadecimal SHA-256 with
`CHECK(length(value) = 64)`.

## 6.2 Complete schema

```sql
BEGIN IMMEDIATE;

CREATE TABLE schema_migrations (
  version INTEGER PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  sha256 TEXT NOT NULL CHECK(length(sha256) = 64),
  applied_at_ms INTEGER NOT NULL
) STRICT;

CREATE TABLE app_meta (
  key TEXT PRIMARY KEY,
  value_json TEXT NOT NULL CHECK(json_valid(value_json)),
  updated_at_ms INTEGER NOT NULL
) STRICT;

CREATE TABLE devices (
  device_id TEXT PRIMARY KEY,
  display_name TEXT NOT NULL,
  platform TEXT NOT NULL CHECK(platform IN ('android','linux','windows')),
  installation_public_key BLOB NOT NULL,
  created_at_ms INTEGER NOT NULL,
  last_seen_at_ms INTEGER NOT NULL
) STRICT;

CREATE TABLE settings (
  key TEXT PRIMARY KEY,
  value_json TEXT NOT NULL CHECK(json_valid(value_json)),
  value_type TEXT NOT NULL CHECK(value_type IN ('boolean','integer','number','string','object','array')),
  updated_at_ms INTEGER NOT NULL
) STRICT;

CREATE TABLE storage_roots (
  storage_root_id TEXT PRIMARY KEY,
  kind TEXT NOT NULL CHECK(kind IN ('app_private','user_selected','removable')),
  platform_locator TEXT NOT NULL,
  display_name TEXT NOT NULL,
  writable INTEGER NOT NULL CHECK(writable IN (0,1)),
  capacity_bytes INTEGER CHECK(capacity_bytes IS NULL OR capacity_bytes >= 0),
  free_bytes INTEGER CHECK(free_bytes IS NULL OR free_bytes >= 0),
  last_probed_at_ms INTEGER,
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL
) STRICT;

CREATE TABLE capability_reports (
  capability_report_id TEXT PRIMARY KEY,
  device_id TEXT NOT NULL REFERENCES devices(device_id) ON DELETE CASCADE,
  report_json TEXT NOT NULL CHECK(json_valid(report_json)),
  app_version TEXT NOT NULL,
  created_at_ms INTEGER NOT NULL
) STRICT;

CREATE TABLE catalog_sources (
  catalog_source_id TEXT PRIMARY KEY,
  source_kind TEXT NOT NULL CHECK(source_kind IN ('openzim_xml','enzime_json','local_manifest')),
  display_name TEXT NOT NULL,
  endpoint TEXT NOT NULL,
  trust_key_id TEXT REFERENCES trust_keys(trust_key_id),
  enabled INTEGER NOT NULL CHECK(enabled IN (0,1)),
  priority INTEGER NOT NULL,
  last_refresh_at_ms INTEGER,
  etag TEXT,
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL
) STRICT;

CREATE TABLE catalog_entries (
  catalog_entry_id TEXT PRIMARY KEY,
  catalog_source_id TEXT NOT NULL REFERENCES catalog_sources(catalog_source_id) ON DELETE CASCADE,
  corpus_id TEXT NOT NULL,
  upstream_version TEXT NOT NULL,
  title TEXT NOT NULL,
  description TEXT NOT NULL,
  language_bcp47 TEXT NOT NULL,
  category TEXT NOT NULL,
  publisher TEXT NOT NULL,
  artifact_url TEXT NOT NULL,
  artifact_size_bytes INTEGER NOT NULL CHECK(artifact_size_bytes > 0),
  artifact_sha256 TEXT NOT NULL CHECK(length(artifact_sha256) = 64),
  signature_url TEXT,
  published_at_ms INTEGER,
  metadata_json TEXT NOT NULL CHECK(json_valid(metadata_json)),
  discovered_at_ms INTEGER NOT NULL,
  UNIQUE(catalog_source_id, corpus_id, upstream_version, artifact_sha256)
) STRICT;

CREATE TABLE packs (
  pack_id TEXT PRIMARY KEY,
  corpus_id TEXT NOT NULL,
  display_name TEXT NOT NULL,
  description TEXT NOT NULL,
  language_bcp47 TEXT NOT NULL,
  pack_kind TEXT NOT NULL CHECK(pack_kind IN ('full_zim','tailored_zim','enzime_pack')),
  active_pack_version_id TEXT,
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL
) STRICT;

CREATE TABLE pack_versions (
  pack_version_id TEXT PRIMARY KEY,
  pack_id TEXT NOT NULL REFERENCES packs(pack_id) ON DELETE CASCADE,
  upstream_version TEXT NOT NULL,
  edition_manifest_sha256 TEXT NOT NULL CHECK(length(edition_manifest_sha256) = 64),
  artifact_sha256 TEXT NOT NULL CHECK(length(artifact_sha256) = 64),
  artifact_size_bytes INTEGER NOT NULL CHECK(artifact_size_bytes > 0),
  format TEXT NOT NULL CHECK(format IN ('zim','enzime_pack')),
  verification_state TEXT NOT NULL CHECK(verification_state IN ('unverified','verified','quarantined')),
  installed_at_ms INTEGER,
  created_at_ms INTEGER NOT NULL,
  UNIQUE(pack_id, upstream_version, artifact_sha256)
) STRICT;

CREATE TABLE pack_locations (
  pack_location_id TEXT PRIMARY KEY,
  pack_version_id TEXT NOT NULL REFERENCES pack_versions(pack_version_id) ON DELETE CASCADE,
  storage_root_id TEXT NOT NULL REFERENCES storage_roots(storage_root_id) ON DELETE RESTRICT,
  relative_path TEXT NOT NULL,
  locator_kind TEXT NOT NULL CHECK(locator_kind IN ('active','retained','staging','quarantine')),
  byte_length INTEGER NOT NULL CHECK(byte_length >= 0),
  last_verified_at_ms INTEGER,
  created_at_ms INTEGER NOT NULL,
  UNIQUE(storage_root_id, relative_path)
) STRICT;

CREATE TABLE selection_profiles (
  profile_id TEXT PRIMARY KEY,
  pack_id TEXT NOT NULL REFERENCES packs(pack_id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  current_revision INTEGER NOT NULL CHECK(current_revision >= 1),
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL
) STRICT;

CREATE TABLE selection_profile_revisions (
  profile_id TEXT NOT NULL REFERENCES selection_profiles(profile_id) ON DELETE CASCADE,
  revision INTEGER NOT NULL CHECK(revision >= 1),
  canonical_json TEXT NOT NULL CHECK(json_valid(canonical_json)),
  profile_sha256 TEXT NOT NULL CHECK(length(profile_sha256) = 64),
  created_at_ms INTEGER NOT NULL,
  PRIMARY KEY(profile_id, revision)
) STRICT, WITHOUT ROWID;

CREATE TABLE download_jobs (
  download_job_id TEXT PRIMARY KEY,
  catalog_entry_id TEXT REFERENCES catalog_entries(catalog_entry_id) ON DELETE SET NULL,
  source_uri TEXT NOT NULL,
  destination_storage_root_id TEXT NOT NULL REFERENCES storage_roots(storage_root_id) ON DELETE RESTRICT,
  expected_size_bytes INTEGER NOT NULL CHECK(expected_size_bytes > 0),
  expected_sha256 TEXT NOT NULL CHECK(length(expected_sha256) = 64),
  state TEXT NOT NULL CHECK(state IN ('planned','fetching','paused','verifying','staged','activating','completed','failed','cancelled')),
  bytes_verified INTEGER NOT NULL DEFAULT 0 CHECK(bytes_verified >= 0),
  etag TEXT,
  last_modified TEXT,
  error_code TEXT,
  error_detail TEXT,
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL,
  completed_at_ms INTEGER
) STRICT;

CREATE TABLE download_chunks (
  download_job_id TEXT NOT NULL REFERENCES download_jobs(download_job_id) ON DELETE CASCADE,
  chunk_index INTEGER NOT NULL CHECK(chunk_index >= 0),
  start_byte INTEGER NOT NULL CHECK(start_byte >= 0),
  end_byte_exclusive INTEGER NOT NULL CHECK(end_byte_exclusive > start_byte),
  expected_sha256 TEXT CHECK(expected_sha256 IS NULL OR length(expected_sha256) = 64),
  actual_sha256 TEXT CHECK(actual_sha256 IS NULL OR length(actual_sha256) = 64),
  state TEXT NOT NULL CHECK(state IN ('pending','fetching','verified','failed')),
  attempts INTEGER NOT NULL DEFAULT 0 CHECK(attempts >= 0),
  updated_at_ms INTEGER NOT NULL,
  PRIMARY KEY(download_job_id, chunk_index)
) STRICT, WITHOUT ROWID;

CREATE TABLE install_transactions (
  install_transaction_id TEXT PRIMARY KEY,
  resource_kind TEXT NOT NULL CHECK(resource_kind IN ('pack','model','restore')),
  resource_id TEXT NOT NULL,
  previous_active_id TEXT,
  candidate_id TEXT NOT NULL,
  state TEXT NOT NULL CHECK(state IN ('planned','staging','verifying','ready','activating','committed','rolled_back','failed')),
  journal_json TEXT NOT NULL CHECK(json_valid(journal_json)),
  error_code TEXT,
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL,
  committed_at_ms INTEGER
) STRICT;

CREATE TABLE reader_positions (
  pack_id TEXT NOT NULL REFERENCES packs(pack_id) ON DELETE CASCADE,
  article_key TEXT NOT NULL,
  scroll_anchor TEXT NOT NULL,
  progress_fraction REAL NOT NULL CHECK(progress_fraction >= 0.0 AND progress_fraction <= 1.0),
  updated_at_ms INTEGER NOT NULL,
  PRIMARY KEY(pack_id, article_key)
) STRICT, WITHOUT ROWID;

CREATE TABLE reader_history (
  history_id TEXT PRIMARY KEY,
  pack_id TEXT NOT NULL REFERENCES packs(pack_id) ON DELETE CASCADE,
  article_key TEXT NOT NULL,
  title TEXT NOT NULL,
  visited_at_ms INTEGER NOT NULL
) STRICT;

CREATE TABLE bookmarks (
  bookmark_id TEXT PRIMARY KEY,
  pack_id TEXT NOT NULL REFERENCES packs(pack_id) ON DELETE CASCADE,
  article_key TEXT NOT NULL,
  title TEXT NOT NULL,
  created_at_ms INTEGER NOT NULL,
  UNIQUE(pack_id, article_key)
) STRICT;

CREATE TABLE notebooks (
  notebook_id TEXT PRIMARY KEY,
  parent_notebook_id TEXT REFERENCES notebooks(notebook_id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  description TEXT NOT NULL,
  sort_order INTEGER NOT NULL DEFAULT 0,
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL,
  UNIQUE(parent_notebook_id, name)
) STRICT;

CREATE TABLE tags (
  tag_id TEXT PRIMARY KEY,
  name TEXT NOT NULL COLLATE NOCASE UNIQUE,
  color_token TEXT NOT NULL,
  created_at_ms INTEGER NOT NULL
) STRICT;

CREATE TABLE sidecar_layers (
  sidecar_layer_id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  owner_device_id TEXT REFERENCES devices(device_id) ON DELETE SET NULL,
  visibility TEXT NOT NULL CHECK(visibility IN ('private','shared','imported')),
  trust_state TEXT NOT NULL CHECK(trust_state IN ('local','verified','unverified','rejected')),
  source_artifact_id TEXT,
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL
) STRICT;

CREATE TABLE annotations (
  annotation_id TEXT PRIMARY KEY,
  annotation_type TEXT NOT NULL CHECK(annotation_type IN ('highlight','comment','ink','voice','bookmark_note')),
  pack_id TEXT NOT NULL REFERENCES packs(pack_id) ON DELETE CASCADE,
  article_key TEXT NOT NULL,
  notebook_id TEXT REFERENCES notebooks(notebook_id) ON DELETE SET NULL,
  sidecar_layer_id TEXT NOT NULL REFERENCES sidecar_layers(sidecar_layer_id) ON DELETE RESTRICT,
  current_revision INTEGER NOT NULL CHECK(current_revision >= 1),
  resolution_state TEXT NOT NULL CHECK(resolution_state IN ('resolved','ambiguous','source_unavailable','orphaned')),
  pinned INTEGER NOT NULL DEFAULT 0 CHECK(pinned IN (0,1)),
  deleted_at_ms INTEGER,
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL
) STRICT;

CREATE TABLE annotation_revisions (
  annotation_id TEXT NOT NULL REFERENCES annotations(annotation_id) ON DELETE CASCADE,
  revision INTEGER NOT NULL CHECK(revision >= 1),
  body_text TEXT,
  selector_json TEXT NOT NULL CHECK(json_valid(selector_json)),
  style_json TEXT NOT NULL CHECK(json_valid(style_json)),
  context_text TEXT NOT NULL,
  source_revision_digest TEXT,
  authored_by_device_id TEXT REFERENCES devices(device_id) ON DELETE SET NULL,
  created_at_ms INTEGER NOT NULL,
  PRIMARY KEY(annotation_id, revision)
) STRICT, WITHOUT ROWID;

CREATE TABLE annotation_assets (
  annotation_asset_id TEXT PRIMARY KEY,
  annotation_id TEXT NOT NULL REFERENCES annotations(annotation_id) ON DELETE CASCADE,
  asset_kind TEXT NOT NULL CHECK(asset_kind IN ('ink_svg','voice_opus','image','file')),
  object_sha256 TEXT NOT NULL CHECK(length(object_sha256) = 64),
  mime_type TEXT NOT NULL,
  byte_length INTEGER NOT NULL CHECK(byte_length > 0),
  duration_ms INTEGER CHECK(duration_ms IS NULL OR duration_ms >= 0),
  created_at_ms INTEGER NOT NULL
) STRICT;

CREATE TABLE annotation_tags (
  annotation_id TEXT NOT NULL REFERENCES annotations(annotation_id) ON DELETE CASCADE,
  tag_id TEXT NOT NULL REFERENCES tags(tag_id) ON DELETE CASCADE,
  PRIMARY KEY(annotation_id, tag_id)
) STRICT, WITHOUT ROWID;

CREATE TABLE sidecar_imports (
  sidecar_import_id TEXT PRIMARY KEY,
  artifact_sha256 TEXT NOT NULL UNIQUE CHECK(length(artifact_sha256) = 64),
  signature_state TEXT NOT NULL CHECK(signature_state IN ('verified','unsigned','invalid','unknown_key')),
  imported_layer_id TEXT REFERENCES sidecar_layers(sidecar_layer_id) ON DELETE SET NULL,
  record_count INTEGER NOT NULL CHECK(record_count >= 0),
  conflict_count INTEGER NOT NULL CHECK(conflict_count >= 0),
  imported_at_ms INTEGER NOT NULL
) STRICT;

CREATE TABLE conversations (
  conversation_id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  scope_json TEXT NOT NULL CHECK(json_valid(scope_json)),
  backend_id TEXT,
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL,
  archived_at_ms INTEGER
) STRICT;

CREATE TABLE messages (
  message_id TEXT PRIMARY KEY,
  conversation_id TEXT NOT NULL REFERENCES conversations(conversation_id) ON DELETE CASCADE,
  role TEXT NOT NULL CHECK(role IN ('user','assistant','system_notice')),
  body_markdown TEXT NOT NULL,
  sequence_no INTEGER NOT NULL CHECK(sequence_no >= 0),
  inference_run_id TEXT,
  created_at_ms INTEGER NOT NULL,
  UNIQUE(conversation_id, sequence_no)
) STRICT;

CREATE TABLE retrieval_documents (
  retrieval_document_id TEXT PRIMARY KEY,
  source_kind TEXT NOT NULL CHECK(source_kind IN ('article','annotation','lore','message')),
  source_id TEXT NOT NULL,
  pack_id TEXT REFERENCES packs(pack_id) ON DELETE CASCADE,
  article_key TEXT,
  title TEXT NOT NULL,
  content_digest TEXT NOT NULL CHECK(length(content_digest) = 64),
  producer_version TEXT NOT NULL,
  updated_at_ms INTEGER NOT NULL,
  UNIQUE(source_kind, source_id, content_digest)
) STRICT;

CREATE TABLE retrieval_chunks (
  retrieval_chunk_id TEXT PRIMARY KEY,
  retrieval_document_id TEXT NOT NULL REFERENCES retrieval_documents(retrieval_document_id) ON DELETE CASCADE,
  ordinal INTEGER NOT NULL CHECK(ordinal >= 0),
  start_char INTEGER NOT NULL CHECK(start_char >= 0),
  end_char INTEGER NOT NULL CHECK(end_char > start_char),
  text_content TEXT NOT NULL,
  text_digest TEXT NOT NULL CHECK(length(text_digest) = 64),
  locator_json TEXT NOT NULL CHECK(json_valid(locator_json)),
  token_count INTEGER NOT NULL CHECK(token_count > 0),
  UNIQUE(retrieval_document_id, ordinal)
) STRICT;

CREATE VIRTUAL TABLE retrieval_chunks_fts USING fts5(
  title,
  text_content,
  content='',
  tokenize='unicode61 remove_diacritics 2'
);

CREATE TABLE embedding_spaces (
  embedding_space_id TEXT PRIMARY KEY,
  model_id TEXT NOT NULL,
  model_sha256 TEXT NOT NULL CHECK(length(model_sha256) = 64),
  dimensions INTEGER NOT NULL CHECK(dimensions > 0),
  distance_metric TEXT NOT NULL CHECK(distance_metric IN ('cosine','dot','l2')),
  normalized INTEGER NOT NULL CHECK(normalized IN (0,1)),
  created_at_ms INTEGER NOT NULL,
  UNIQUE(model_sha256, dimensions, distance_metric, normalized)
) STRICT;

CREATE TABLE chunk_embeddings (
  retrieval_chunk_id TEXT NOT NULL REFERENCES retrieval_chunks(retrieval_chunk_id) ON DELETE CASCADE,
  embedding_space_id TEXT NOT NULL REFERENCES embedding_spaces(embedding_space_id) ON DELETE CASCADE,
  vector_le_f32 BLOB NOT NULL,
  vector_sha256 TEXT NOT NULL CHECK(length(vector_sha256) = 64),
  created_at_ms INTEGER NOT NULL,
  PRIMARY KEY(retrieval_chunk_id, embedding_space_id)
) STRICT, WITHOUT ROWID;

CREATE TABLE inference_backends (
  backend_id TEXT PRIMARY KEY,
  backend_kind TEXT NOT NULL CHECK(backend_kind IN ('local_lm')),
  display_name TEXT NOT NULL,
  enabled INTEGER NOT NULL CHECK(enabled IN (0,1)),
  priority INTEGER NOT NULL,
  config_json TEXT NOT NULL CHECK(json_valid(config_json)),
  last_health_json TEXT CHECK(last_health_json IS NULL OR json_valid(last_health_json)),
  updated_at_ms INTEGER NOT NULL
) STRICT;

CREATE TABLE models (
  model_id TEXT PRIMARY KEY,
  display_name TEXT NOT NULL,
  family TEXT NOT NULL,
  model_kind TEXT NOT NULL CHECK(model_kind IN ('chat','embedding','reranker')),
  format TEXT NOT NULL CHECK(format IN ('gguf')),
  manifest_json TEXT NOT NULL CHECK(json_valid(manifest_json)),
  artifact_sha256 TEXT NOT NULL CHECK(length(artifact_sha256) = 64),
  artifact_size_bytes INTEGER NOT NULL CHECK(artifact_size_bytes > 0),
  created_at_ms INTEGER NOT NULL
) STRICT;

CREATE TABLE model_installations (
  model_installation_id TEXT PRIMARY KEY,
  model_id TEXT NOT NULL REFERENCES models(model_id) ON DELETE CASCADE,
  backend_id TEXT NOT NULL REFERENCES inference_backends(backend_id) ON DELETE RESTRICT,
  storage_root_id TEXT NOT NULL REFERENCES storage_roots(storage_root_id) ON DELETE RESTRICT,
  relative_path TEXT NOT NULL,
  state TEXT NOT NULL CHECK(state IN ('staged','verified','active','retained','quarantined')),
  installed_at_ms INTEGER NOT NULL,
  last_used_at_ms INTEGER,
  UNIQUE(model_id, backend_id, storage_root_id, relative_path)
) STRICT;

CREATE TABLE inference_runs (
  inference_run_id TEXT PRIMARY KEY,
  conversation_id TEXT REFERENCES conversations(conversation_id) ON DELETE SET NULL,
  backend_id TEXT NOT NULL REFERENCES inference_backends(backend_id) ON DELETE RESTRICT,
  model_id TEXT NOT NULL REFERENCES models(model_id) ON DELETE RESTRICT,
  scope_json TEXT NOT NULL CHECK(json_valid(scope_json)),
  prompt_digest TEXT NOT NULL CHECK(length(prompt_digest) = 64),
  status TEXT NOT NULL CHECK(status IN ('running','completed','cancelled','failed')),
  input_tokens INTEGER NOT NULL DEFAULT 0 CHECK(input_tokens >= 0),
  output_tokens INTEGER NOT NULL DEFAULT 0 CHECK(output_tokens >= 0),
  started_at_ms INTEGER NOT NULL,
  completed_at_ms INTEGER,
  error_code TEXT
) STRICT;

CREATE TABLE message_citations (
  message_id TEXT NOT NULL REFERENCES messages(message_id) ON DELETE CASCADE,
  citation_index INTEGER NOT NULL CHECK(citation_index >= 1),
  retrieval_chunk_id TEXT NOT NULL REFERENCES retrieval_chunks(retrieval_chunk_id) ON DELETE RESTRICT,
  quote_text TEXT NOT NULL,
  start_char INTEGER NOT NULL CHECK(start_char >= 0),
  end_char INTEGER NOT NULL CHECK(end_char > start_char),
  confidence REAL NOT NULL CHECK(confidence >= 0.0 AND confidence <= 1.0),
  PRIMARY KEY(message_id, citation_index)
) STRICT, WITHOUT ROWID;

CREATE TABLE lore_evidence (
  lore_evidence_id TEXT PRIMARY KEY,
  source_kind TEXT NOT NULL CHECK(source_kind IN ('message','annotation','article','user_assertion')),
  source_id TEXT NOT NULL,
  excerpt TEXT NOT NULL,
  locator_json TEXT NOT NULL CHECK(json_valid(locator_json)),
  content_digest TEXT NOT NULL CHECK(length(content_digest) = 64),
  created_at_ms INTEGER NOT NULL
) STRICT;

CREATE TABLE lore_candidates (
  lore_candidate_id TEXT PRIMARY KEY,
  statement TEXT NOT NULL,
  subject_key TEXT NOT NULL,
  predicate_key TEXT NOT NULL,
  object_json TEXT NOT NULL CHECK(json_valid(object_json)),
  confidence REAL NOT NULL CHECK(confidence >= 0.0 AND confidence <= 1.0),
  status TEXT NOT NULL CHECK(status IN ('inbox','accepted','rejected','expired')),
  proposed_by_run_id TEXT REFERENCES inference_runs(inference_run_id) ON DELETE SET NULL,
  created_at_ms INTEGER NOT NULL,
  reviewed_at_ms INTEGER
) STRICT;

CREATE TABLE lore_candidate_evidence (
  lore_candidate_id TEXT NOT NULL REFERENCES lore_candidates(lore_candidate_id) ON DELETE CASCADE,
  lore_evidence_id TEXT NOT NULL REFERENCES lore_evidence(lore_evidence_id) ON DELETE RESTRICT,
  PRIMARY KEY(lore_candidate_id, lore_evidence_id)
) STRICT, WITHOUT ROWID;

CREATE TABLE lore_memories (
  memory_id TEXT PRIMARY KEY,
  subject_key TEXT NOT NULL,
  predicate_key TEXT NOT NULL,
  current_revision INTEGER NOT NULL CHECK(current_revision >= 1),
  pinned INTEGER NOT NULL DEFAULT 0 CHECK(pinned IN (0,1)),
  status TEXT NOT NULL CHECK(status IN ('active','superseded','deleted')),
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL,
  deleted_at_ms INTEGER
) STRICT;

CREATE TABLE lore_memory_revisions (
  memory_id TEXT NOT NULL REFERENCES lore_memories(memory_id) ON DELETE CASCADE,
  revision INTEGER NOT NULL CHECK(revision >= 1),
  statement TEXT NOT NULL,
  object_json TEXT NOT NULL CHECK(json_valid(object_json)),
  correction_reason TEXT,
  authored_by TEXT NOT NULL CHECK(authored_by IN ('user','policy')),
  created_at_ms INTEGER NOT NULL,
  PRIMARY KEY(memory_id, revision)
) STRICT, WITHOUT ROWID;

CREATE TABLE lore_memory_evidence (
  memory_id TEXT NOT NULL REFERENCES lore_memories(memory_id) ON DELETE CASCADE,
  memory_revision INTEGER NOT NULL,
  lore_evidence_id TEXT NOT NULL REFERENCES lore_evidence(lore_evidence_id) ON DELETE RESTRICT,
  PRIMARY KEY(memory_id, memory_revision, lore_evidence_id),
  FOREIGN KEY(memory_id, memory_revision)
    REFERENCES lore_memory_revisions(memory_id, revision)
    ON DELETE CASCADE
) STRICT, WITHOUT ROWID;

CREATE TABLE trust_keys (
  trust_key_id TEXT PRIMARY KEY,
  key_kind TEXT NOT NULL CHECK(key_kind IN ('root','catalog','sidecar','entitlement','release')),
  label TEXT NOT NULL,
  ed25519_public_key BLOB NOT NULL CHECK(length(ed25519_public_key) = 32),
  trust_state TEXT NOT NULL CHECK(trust_state IN ('trusted','untrusted','revoked')),
  valid_from_ms INTEGER NOT NULL,
  valid_until_ms INTEGER,
  created_at_ms INTEGER NOT NULL
) STRICT;

CREATE TABLE entitlement_artifacts (
  entitlement_artifact_id TEXT PRIMARY KEY,
  subject_device_id TEXT REFERENCES devices(device_id) ON DELETE SET NULL,
  artifact_cbor BLOB NOT NULL,
  artifact_sha256 TEXT NOT NULL UNIQUE CHECK(length(artifact_sha256) = 64),
  issuer_key_id TEXT NOT NULL REFERENCES trust_keys(trust_key_id) ON DELETE RESTRICT,
  issued_at_ms INTEGER NOT NULL,
  not_before_ms INTEGER NOT NULL,
  expires_at_ms INTEGER,
  verification_state TEXT NOT NULL CHECK(verification_state IN ('verified','expired','invalid','revoked')),
  imported_at_ms INTEGER NOT NULL
) STRICT;

CREATE TABLE entitlement_grants (
  entitlement_artifact_id TEXT NOT NULL REFERENCES entitlement_artifacts(entitlement_artifact_id) ON DELETE CASCADE,
  capability_key TEXT NOT NULL,
  grant_kind TEXT NOT NULL CHECK(grant_kind IN ('perpetual','term','quantity')),
  quantity INTEGER,
  valid_until_ms INTEGER,
  PRIMARY KEY(entitlement_artifact_id, capability_key)
) STRICT, WITHOUT ROWID;

CREATE TABLE coupon_ledger (
  coupon_redemption_id TEXT PRIMARY KEY,
  coupon_id TEXT NOT NULL,
  entitlement_artifact_id TEXT REFERENCES entitlement_artifacts(entitlement_artifact_id) ON DELETE SET NULL,
  redeemed_on_device_id TEXT NOT NULL REFERENCES devices(device_id) ON DELETE RESTRICT,
  redeemed_at_ms INTEGER NOT NULL,
  redemption_nonce BLOB NOT NULL,
  receipt_sha256 TEXT NOT NULL UNIQUE CHECK(length(receipt_sha256) = 64),
  status TEXT NOT NULL CHECK(status IN ('accepted','reconciled','rejected')),
  UNIQUE(coupon_id, redeemed_on_device_id)
) STRICT;

CREATE TABLE backups (
  backup_id TEXT PRIMARY KEY,
  destination_kind TEXT NOT NULL CHECK(destination_kind IN ('file','directory','android_document')),
  destination_locator TEXT NOT NULL,
  manifest_sha256 TEXT NOT NULL CHECK(length(manifest_sha256) = 64),
  encrypted INTEGER NOT NULL CHECK(encrypted IN (0,1)),
  byte_length INTEGER NOT NULL CHECK(byte_length > 0),
  state TEXT NOT NULL CHECK(state IN ('creating','verified','failed')),
  created_at_ms INTEGER NOT NULL,
  verified_at_ms INTEGER
) STRICT;

CREATE TABLE user_tasks (
  task_id TEXT PRIMARY KEY,
  task_kind TEXT NOT NULL CHECK(task_kind IN ('download_attention','annotation_ambiguous','memory_review','backup_due','model_attention')),
  title TEXT NOT NULL,
  detail_json TEXT NOT NULL CHECK(json_valid(detail_json)),
  status TEXT NOT NULL CHECK(status IN ('open','resolved','dismissed')),
  priority INTEGER NOT NULL CHECK(priority BETWEEN 0 AND 100),
  created_at_ms INTEGER NOT NULL,
  resolved_at_ms INTEGER
) STRICT;

CREATE TABLE audit_events (
  audit_event_id TEXT PRIMARY KEY,
  event_type TEXT NOT NULL,
  entity_kind TEXT NOT NULL,
  entity_id TEXT NOT NULL,
  detail_json TEXT NOT NULL CHECK(json_valid(detail_json)),
  severity TEXT NOT NULL CHECK(severity IN ('info','warning','error','security')),
  created_at_ms INTEGER NOT NULL
) STRICT;

CREATE INDEX idx_catalog_entries_search
  ON catalog_entries(language_bcp47, category, title);
CREATE INDEX idx_catalog_entries_corpus_version
  ON catalog_entries(corpus_id, upstream_version);
CREATE INDEX idx_pack_versions_pack_created
  ON pack_versions(pack_id, created_at_ms DESC);
CREATE INDEX idx_pack_locations_version_kind
  ON pack_locations(pack_version_id, locator_kind);
CREATE INDEX idx_download_jobs_state_updated
  ON download_jobs(state, updated_at_ms DESC);
CREATE INDEX idx_download_chunks_state
  ON download_chunks(download_job_id, state, chunk_index);
CREATE INDEX idx_install_transactions_state
  ON install_transactions(state, updated_at_ms DESC);
CREATE INDEX idx_reader_history_recent
  ON reader_history(visited_at_ms DESC);
CREATE INDEX idx_annotations_article
  ON annotations(pack_id, article_key, deleted_at_ms);
CREATE INDEX idx_annotations_notebook
  ON annotations(notebook_id, updated_at_ms DESC);
CREATE INDEX idx_annotations_layer
  ON annotations(sidecar_layer_id, updated_at_ms DESC);
CREATE INDEX idx_annotation_revisions_created
  ON annotation_revisions(annotation_id, created_at_ms DESC);
CREATE INDEX idx_conversations_recent
  ON conversations(updated_at_ms DESC);
CREATE INDEX idx_messages_conversation_sequence
  ON messages(conversation_id, sequence_no);
CREATE INDEX idx_retrieval_documents_source
  ON retrieval_documents(source_kind, source_id);
CREATE INDEX idx_retrieval_chunks_document
  ON retrieval_chunks(retrieval_document_id, ordinal);
CREATE INDEX idx_inference_runs_status
  ON inference_runs(status, started_at_ms DESC);
CREATE INDEX idx_lore_candidates_inbox
  ON lore_candidates(status, created_at_ms DESC);
CREATE INDEX idx_lore_memories_subject
  ON lore_memories(subject_key, predicate_key, status);
CREATE INDEX idx_entitlement_state
  ON entitlement_artifacts(verification_state, expires_at_ms);
CREATE INDEX idx_user_tasks_open
  ON user_tasks(status, priority DESC, created_at_ms);
CREATE INDEX idx_audit_entity
  ON audit_events(entity_kind, entity_id, created_at_ms DESC);

CREATE TRIGGER trg_pack_active_version_same_pack
BEFORE UPDATE OF active_pack_version_id ON packs
WHEN NEW.active_pack_version_id IS NOT NULL
BEGIN
  SELECT CASE WHEN NOT EXISTS (
    SELECT 1
    FROM pack_versions pv
    WHERE pv.pack_version_id = NEW.active_pack_version_id
      AND pv.pack_id = NEW.pack_id
      AND pv.verification_state = 'verified'
  ) THEN RAISE(ABORT, 'active pack version must be verified and belong to pack') END;
END;

CREATE TRIGGER trg_pack_active_version_insert
BEFORE INSERT ON packs
WHEN NEW.active_pack_version_id IS NOT NULL
BEGIN
  SELECT RAISE(
    ABORT,
    'packs must be inserted without an active version; activate after verified version insert'
  );
END;

CREATE TRIGGER trg_annotation_revision_insert
AFTER INSERT ON annotation_revisions
BEGIN
  UPDATE annotations
  SET current_revision = NEW.revision,
      updated_at_ms = NEW.created_at_ms
  WHERE annotation_id = NEW.annotation_id
    AND NEW.revision > current_revision;
END;

CREATE TRIGGER trg_lore_revision_insert
AFTER INSERT ON lore_memory_revisions
BEGIN
  UPDATE lore_memories
  SET current_revision = NEW.revision,
      updated_at_ms = NEW.created_at_ms
  WHERE memory_id = NEW.memory_id
    AND NEW.revision > current_revision;
END;

CREATE TRIGGER trg_retrieval_chunk_fts_insert
AFTER INSERT ON retrieval_chunks
BEGIN
  INSERT INTO retrieval_chunks_fts(rowid, title, text_content)
  SELECT NEW.rowid, d.title, NEW.text_content
  FROM retrieval_documents d
  WHERE d.retrieval_document_id = NEW.retrieval_document_id;
END;

CREATE TRIGGER trg_retrieval_chunk_fts_delete
AFTER DELETE ON retrieval_chunks
BEGIN
  INSERT INTO retrieval_chunks_fts(retrieval_chunks_fts, rowid, title, text_content)
  VALUES('delete', OLD.rowid, '', OLD.text_content);
END;

CREATE TRIGGER trg_download_state_guard
BEFORE UPDATE OF state ON download_jobs
WHEN OLD.state <> NEW.state
BEGIN
  SELECT CASE WHEN NOT (
    (OLD.state = 'planned' AND NEW.state IN ('fetching','cancelled')) OR
    (OLD.state = 'fetching' AND NEW.state IN ('paused','verifying','failed','cancelled')) OR
    (OLD.state = 'paused' AND NEW.state IN ('fetching','cancelled')) OR
    (OLD.state = 'verifying' AND NEW.state IN ('staged','failed','cancelled')) OR
    (OLD.state = 'staged' AND NEW.state IN ('activating','cancelled')) OR
    (OLD.state = 'activating' AND NEW.state IN ('completed','failed')) OR
    (OLD.state = 'failed' AND NEW.state IN ('fetching','cancelled'))
  ) THEN RAISE(ABORT, 'invalid download state transition') END;
END;

CREATE TRIGGER trg_install_state_guard
BEFORE UPDATE OF state ON install_transactions
WHEN OLD.state <> NEW.state
BEGIN
  SELECT CASE WHEN NOT (
    (OLD.state = 'planned' AND NEW.state IN ('staging','failed')) OR
    (OLD.state = 'staging' AND NEW.state IN ('verifying','failed','rolled_back')) OR
    (OLD.state = 'verifying' AND NEW.state IN ('ready','failed','rolled_back')) OR
    (OLD.state = 'ready' AND NEW.state IN ('activating','rolled_back')) OR
    (OLD.state = 'activating' AND NEW.state IN ('committed','failed','rolled_back')) OR
    (OLD.state = 'failed' AND NEW.state = 'rolled_back')
  ) THEN RAISE(ABORT, 'invalid install state transition') END;
END;

COMMIT;
```

`packs.active_pack_version_id` is a cyclic relationship enforced by
`trg_pack_active_version_same_pack`; the trigger is the canonical SQLite
constraint and accepts only a verified version belonging to the same pack.

## 6.3 Object store

```text
<storage-root>/EnZIME/
├── objects/sha256/ab/<62 remaining hex characters>
├── staging/<install-transaction-id>/
├── quarantine/<audit-event-id>/
├── packs/<pack-id>/<pack-version-id>/
├── models/<model-id>/<artifact-sha256>/
├── sidecars/
├── backups/
└── locks/
```

Object commit algorithm:

1. create a transaction-scoped staging file with exclusive create;
2. stream bytes while enforcing declared maximum size;
3. flush file content;
4. calculate SHA-256 from staged bytes;
5. compare length and digest;
6. run format verifier;
7. create the content-addressed object directory;
8. atomically rename into the object store;
9. commit metadata transaction;
10. atomically update the active pointer;
11. remove the staging directory.

Object files are immutable.
Mutation creates a new digest.

## 6.4 Migrations

Every migration contains:

- integer version;
- stable name;
- SHA-256 registered in `schema_migrations`;
- forward SQL;
- preflight query;
- backup requirement;
- postflight query;
- fixture proving migration from every supported prior release.

A migration never silently repairs ambiguous primary data.
It emits a `user_tasks` row requiring review.

---

# 7. ZIM Subsystem

## 7.1 Rust identities

```rust
use async_trait::async_trait;
use bytes::Bytes;
use futures_core::Stream;
use serde::{Deserialize, Serialize};
use std::{path::Path, pin::Pin, sync::Arc};

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PackId(pub String);

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PackVersionId(pub String);

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ArticleKey(pub String);

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct EntryKey(pub String);

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ContentDigest {
    pub algorithm: DigestAlgorithm,
    pub hex: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DigestAlgorithm {
    Sha256,
}

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
```

## 7.2 Exact traits

```rust
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
    async fn entry(
        &self,
        key: &EntryKey,
    ) -> Result<ZimEntryDescriptor, ZimError>;
    async fn article(
        &self,
        key: &ArticleKey,
    ) -> Result<ArticleDocument, ZimError>;
    async fn resource(
        &self,
        key: &EntryKey,
    ) -> Result<ResourceDocument, ZimError>;
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

Release implementation:

```text
ZimEngine trait implementation: AnZimmermanZimEngine
ZimArchive trait implementation: AnZimmermanZimArchive
Engine ID: zim.anzimmerman.v1
```

No second parser is selected by policy in release 4.0.
The port permits replacement without changing application/UI contracts.

## 7.3 Rendering boundary

`ArticleDocument.raw_html` is never injected into the privileged React DOM.

Pipeline:

1. parse untrusted HTML with bounded parser settings;
2. remove scripts, event handlers, forms, frames and remote loads;
3. rewrite internal links to opaque renderer messages;
4. rewrite resource URLs to scoped `enzime-resource://` URLs;
5. apply strict CSP;
6. render in an isolated webview/origin without Tauri APIs;
7. send link/selection/scroll events to the parent through
   `RendererMessageV1`;
8. validate every message before dispatch.

```ts
export type RendererMessageV1 =
  | {
      schemaVersion: 1;
      kind: "open_article";
      articleKey: string;
    }
  | {
      schemaVersion: 1;
      kind: "selection_changed";
      selector: SelectorBundle;
    }
  | {
      schemaVersion: 1;
      kind: "scroll_position";
      scrollAnchor: string;
      progressFraction: number;
    }
  | {
      schemaVersion: 1;
      kind: "resource_failed";
      entryKey: string;
      reason: string;
    };
```

## 7.4 Conformance fixtures

Required fixtures:

| Fixture ID | Evidence |
|---|---|
| `zim.minimal.article.v1` | one HTML article |
| `zim.redirect.chain.v1` | valid three-hop redirect |
| `zim.redirect.cycle.v1` | cycle rejected |
| `zim.resource.matrix.v1` | CSS, image, SVG, audio |
| `zim.unicode.paths.v1` | non-ASCII titles and paths |
| `zim.large.cluster.guard.v1` | decompression budget |
| `zim.truncated.header.v1` | invalid header |
| `zim.out_of_bounds.pointer.v1` | allocation/bounds guard |
| `zim.scripted.article.v1` | sanitizer removes active content |
| `zim.internal.links.v1` | deterministic link rewriting |

Release evidence records:

- fixture digest;
- engine version;
- test version;
- platform;
- observed result;
- timestamp;
- CI run identifier.

---

# 8. Pack & Transfer Subsystem

## 8.1 Pack identities

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PackManifest {
    pub pack_id: PackId,
    pub corpus_id: String,
    pub display_name: String,
    pub description: String,
    pub language_bcp47: String,
    pub pack_kind: PackKind,
    pub versions: Vec<PackVersionSummary>,
    pub active_version_id: Option<PackVersionId>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackKind {
    FullZim,
    TailoredZim,
    EnzimePack,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PackVersionSummary {
    pub pack_version_id: PackVersionId,
    pub upstream_version: String,
    pub artifact_sha256: String,
    pub artifact_size_bytes: u64,
    pub verification_state: VerificationState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationState {
    Unverified,
    Verified,
    Quarantined,
}
```

## 8.2 Catalog and transfer traits

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CatalogQuery {
    pub text: Option<String>,
    pub languages: Vec<String>,
    pub categories: Vec<String>,
    pub max_size_bytes: Option<u64>,
    pub offset: u32,
    pub limit: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CatalogPage {
    pub entries: Vec<CatalogEntry>,
    pub total: Option<u64>,
    pub next_offset: Option<u32>,
    pub refreshed_at_ms: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CatalogEntry {
    pub catalog_entry_id: String,
    pub corpus_id: String,
    pub upstream_version: String,
    pub title: String,
    pub description: String,
    pub language_bcp47: String,
    pub category: String,
    pub publisher: String,
    pub artifact_url: String,
    pub artifact_size_bytes: u64,
    pub artifact_sha256: String,
    pub signature_url: Option<String>,
    pub trust: TrustPresentation,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrustPresentation {
    pub state: String,
    pub signer: Option<String>,
    pub explanation: String,
}

#[async_trait]
pub trait CatalogSource: Send + Sync {
    fn source_id(&self) -> &str;
    async fn refresh(&self) -> Result<CatalogRefreshReport, CatalogError>;
    async fn search(&self, query: CatalogQuery) -> Result<CatalogPage, CatalogError>;
    async fn entry(&self, catalog_entry_id: &str) -> Result<CatalogEntry, CatalogError>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CatalogRefreshReport {
    pub source_id: String,
    pub added: u64,
    pub updated: u64,
    pub removed: u64,
    pub signature_state: String,
    pub refreshed_at_ms: i64,
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum CatalogError {
    #[error("offline")]
    Offline,
    #[error("authentication failed")]
    AuthenticationFailed,
    #[error("catalog signature invalid")]
    InvalidSignature,
    #[error("catalog malformed: {0}")]
    Malformed(String),
    #[error("entry not found")]
    NotFound,
    #[error("transport error: {0}")]
    Transport(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransferProbeRequest {
    pub uri: String,
    pub maximum_response_bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransferProbe {
    pub uri: String,
    pub size_bytes: u64,
    pub accepts_ranges: bool,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub content_type: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransferRange {
    pub start_byte: u64,
    pub end_byte_exclusive: u64,
}

pub type TransferByteStream =
    Pin<Box<dyn Stream<Item = Result<Bytes, TransferError>> + Send>>;

#[async_trait]
pub trait TransferSource: Send + Sync {
    fn source_kind(&self) -> &'static str;
    async fn probe(
        &self,
        request: TransferProbeRequest,
    ) -> Result<TransferProbe, TransferError>;
    async fn open_range(
        &self,
        uri: &str,
        range: TransferRange,
        expected_etag: Option<&str>,
    ) -> Result<TransferByteStream, TransferError>;
    async fn open_full(
        &self,
        uri: &str,
        maximum_response_bytes: u64,
    ) -> Result<TransferByteStream, TransferError>;
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum TransferError {
    #[error("offline")]
    Offline,
    #[error("source changed")]
    SourceChanged,
    #[error("range unsupported")]
    RangeUnsupported,
    #[error("range response invalid")]
    InvalidRangeResponse,
    #[error("response exceeds limit")]
    ResponseLimitExceeded,
    #[error("source not found")]
    NotFound,
    #[error("permission denied")]
    PermissionDenied,
    #[error("transport error: {0}")]
    Transport(String),
}
```

Release implementations:

| Trait | Implementation | ID |
|---|---|---|
| `CatalogSource` | `OpenZimXmlCatalogSource` | `catalog.openzim.xml.v1` |
| `CatalogSource` | `EnzimeJsonCatalogSource` | `catalog.enzime.json.v1` |
| `CatalogSource` | `LocalManifestCatalogSource` | `catalog.local.manifest.v1` |
| `TransferSource` | `HttpTransferSource` | `transfer.http.v1` |
| `TransferSource` | `LocalFileTransferSource` | `transfer.file.v1` |

## 8.3 Install plan

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstallPlan {
    pub install_plan_id: String,
    pub catalog_entry: CatalogEntry,
    pub destination: StorageDestination,
    pub transfer_mode: TransferMode,
    pub download_bytes: u64,
    pub temporary_bytes: u64,
    pub final_bytes: u64,
    pub free_bytes_before: u64,
    pub free_bytes_after: u64,
    pub verification: VerificationPlan,
    pub replaces_pack_version_id: Option<PackVersionId>,
    pub retained_pack_version_id: Option<PackVersionId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StorageDestination {
    pub storage_root_id: String,
    pub display_name: String,
    pub writable: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferMode {
    Ranged,
    Full,
    LocalCopy,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerificationPlan {
    pub expected_size_bytes: u64,
    pub expected_sha256: String,
    pub signature_required: bool,
    pub zim_structure_check: bool,
}
```

Planning fails with an actionable result when:

- destination is not writable;
- final bytes do not fit;
- temporary plus retained last-known-good bytes do not fit;
- no verified digest is available;
- policy requires a signature and no trusted signature exists;
- source range behavior is incompatible with the selected strategy.

## 8.4 Durable install state machine

```mermaid
stateDiagram-v2
    [*] --> Planned
    Planned --> Fetching
    Fetching --> Paused
    Paused --> Fetching
    Fetching --> Verifying
    Verifying --> Staged
    Staged --> Activating
    Activating --> Completed
    Fetching --> Failed
    Verifying --> Failed
    Activating --> Failed
    Failed --> Fetching
```

State belongs to `download_jobs`.
Activation belongs to `install_transactions`.
The UI never guesses state from file existence.

## 8.5 Transfer limits

| Limit key | Release value |
|---|---:|
| `transfer.chunk_bytes` | 8 MiB |
| `transfer.max_parallel_chunks.android` | 2 |
| `transfer.max_parallel_chunks.desktop` | 4 |
| `transfer.max_redirects` | 5 |
| `transfer.probe_max_bytes` | 1 MiB |
| `transfer.catalog_max_bytes` | 32 MiB |
| `transfer.retry_count` | 5 |
| `transfer.retry_base_ms` | 500 |
| `transfer.retry_max_ms` | 30000 |

HTTP `200` to a range request is not accepted as a ranged response.
The user may explicitly approve conversion to a bounded full transfer.

---

# 9. Annotation & Sidecar Subsystem

## 9.1 Selector model

An annotation stores multiple selectors because no single selector survives every
edition, sanitizer and layout change.

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SelectorBundle {
    pub article: ArticleSelector,
    pub text_quote: Option<TextQuoteSelector>,
    pub text_position: Option<TextPositionSelector>,
    pub dom_path: Option<DomPathSelector>,
    pub image_region: Option<ImageRegionSelector>,
    pub captured_context: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArticleSelector {
    pub pack_id: PackId,
    pub article_key: ArticleKey,
    pub source_revision_digest: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextQuoteSelector {
    pub exact: String,
    pub prefix: String,
    pub suffix: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextPositionSelector {
    pub normalized_start: u32,
    pub normalized_end: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomPathSelector {
    pub path: String,
    pub node_text_digest: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImageRegionSelector {
    pub resource_entry_key: EntryKey,
    pub x_fraction: f32,
    pub y_fraction: f32,
    pub width_fraction: f32,
    pub height_fraction: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnnotationType {
    Highlight,
    Comment,
    Ink,
    Voice,
    BookmarkNote,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnnotationDraft {
    pub annotation_type: AnnotationType,
    pub selector: SelectorBundle,
    pub body_text: Option<String>,
    pub style: AnnotationStyle,
    pub notebook_id: Option<String>,
    pub tag_ids: Vec<String>,
    pub sidecar_layer_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnnotationStyle {
    pub color_token: String,
    pub emphasis: String,
    pub stroke_width: Option<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnnotationRecord {
    pub annotation_id: String,
    pub annotation_type: AnnotationType,
    pub selector: SelectorBundle,
    pub body_text: Option<String>,
    pub style: AnnotationStyle,
    pub notebook_id: Option<String>,
    pub tags: Vec<TagSummary>,
    pub sidecar_layer_id: String,
    pub resolution: AnnotationResolution,
    pub pinned: bool,
    pub revision: u32,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TagSummary {
    pub tag_id: String,
    pub name: String,
    pub color_token: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnnotationResolution {
    pub state: AnnotationResolutionState,
    pub confidence: f32,
    pub matched_start: Option<u32>,
    pub matched_end: Option<u32>,
    pub explanation: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnnotationResolutionState {
    Resolved,
    Ambiguous,
    SourceUnavailable,
    Orphaned,
}
```

## 9.2 Resolution order

The resolver tries:

1. exact source digest plus text position;
2. exact quote plus prefix/suffix;
3. exact quote with unique occurrence;
4. DOM path plus node digest;
5. bounded fuzzy quote match above `0.92`;
6. ambiguous result with candidates;
7. source unavailable;
8. orphaned.

It never selects a weak candidate silently.

```rust
#[async_trait]
pub trait AnnotationResolver: Send + Sync {
    async fn resolve(
        &self,
        selector: &SelectorBundle,
        article: &ArticleDocument,
    ) -> Result<AnnotationResolution, AnnotationError>;
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum AnnotationError {
    #[error("annotation not found")]
    NotFound,
    #[error("annotation revision conflict")]
    RevisionConflict,
    #[error("source unavailable")]
    SourceUnavailable,
    #[error("selector invalid: {0}")]
    InvalidSelector(String),
    #[error("asset invalid: {0}")]
    InvalidAsset(String),
    #[error("storage error: {0}")]
    Storage(String),
}
```

## 9.3 Annotation service

```rust
#[async_trait]
pub trait AnnotationService: Send + Sync {
    async fn create(
        &self,
        draft: AnnotationDraft,
    ) -> Result<AnnotationRecord, AnnotationError>;
    async fn get(
        &self,
        annotation_id: &str,
    ) -> Result<AnnotationRecord, AnnotationError>;
    async fn update(
        &self,
        annotation_id: &str,
        expected_revision: u32,
        draft: AnnotationDraft,
    ) -> Result<AnnotationRecord, AnnotationError>;
    async fn soft_delete(
        &self,
        annotation_id: &str,
        expected_revision: u32,
    ) -> Result<(), AnnotationError>;
    async fn restore(
        &self,
        annotation_id: &str,
    ) -> Result<AnnotationRecord, AnnotationError>;
    async fn list_for_article(
        &self,
        pack_id: &PackId,
        article_key: &ArticleKey,
    ) -> Result<Vec<AnnotationRecord>, AnnotationError>;
    async fn attach_asset(
        &self,
        annotation_id: &str,
        asset: AnnotationAssetInput,
    ) -> Result<AnnotationAssetRecord, AnnotationError>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnnotationAssetInput {
    pub asset_kind: String,
    pub mime_type: String,
    pub bytes: Bytes,
    pub duration_ms: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnnotationAssetRecord {
    pub annotation_asset_id: String,
    pub annotation_id: String,
    pub asset_kind: String,
    pub object_sha256: String,
    pub mime_type: String,
    pub byte_length: u64,
    pub duration_ms: Option<u64>,
}
```

## 9.4 Sidecar CBOR

File extension:

```text
.ezsc
```

Media type:

```text
application/vnd.enzime.sidecar+cbor
```

Canonical CDDL:

```cddl
enzime-sidecar-v1 = {
  "schema": "enzime.sidecar.cbor.v1",
  "artifact_id": tstr,
  "created_at_ms": int,
  "creator": device-ref,
  "layer": layer,
  "source_bindings": [* source-binding],
  "annotations": [* annotation],
  "notebooks": [* notebook],
  "tags": [* tag],
  "objects": [* object-ref],
  "signature": signature
}

device-ref = {
  "device_id": tstr,
  "public_key": bstr .size 32
}

layer = {
  "layer_id": tstr,
  "name": tstr,
  "visibility": "private" / "shared" / "imported"
}

source-binding = {
  "pack_id": tstr,
  "corpus_id": tstr,
  "artifact_sha256": tstr
}

annotation = {
  "annotation_id": tstr,
  "annotation_type": "highlight" / "comment" / "ink" / "voice" / "bookmark_note",
  "revision": uint,
  "selector": any,
  "body_text": tstr / null,
  "style": any,
  "notebook_id": tstr / null,
  "tag_ids": [* tstr],
  "asset_ids": [* tstr],
  "created_at_ms": int,
  "updated_at_ms": int
}

notebook = {
  "notebook_id": tstr,
  "parent_notebook_id": tstr / null,
  "name": tstr,
  "description": tstr
}

tag = {
  "tag_id": tstr,
  "name": tstr,
  "color_token": tstr
}

object-ref = {
  "object_sha256": tstr,
  "mime_type": tstr,
  "byte_length": uint,
  "embedded_bytes": bstr / null
}

signature = {
  "algorithm": "ed25519",
  "key_id": tstr,
  "signed_domain": "enzime.sidecar.v1",
  "value": bstr .size 64
}
```

Canonicalization:

- RFC 8949 deterministic CBOR;
- map keys sorted by encoded key bytes;
- preferred integer encoding;
- no indefinite-length items;
- floating values forbidden;
- signature value omitted while signing;
- `artifact_id` is SHA-256 of canonical unsigned envelope;
- signature domain prefix is UTF-8 `enzime.sidecar.v1\0`.

## 9.5 Signature traits

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignatureEnvelope {
    pub algorithm: String,
    pub key_id: String,
    pub signed_domain: String,
    pub value: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignatureVerification {
    pub state: SignatureVerificationState,
    pub key_id: String,
    pub signer_label: Option<String>,
    pub explanation: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignatureVerificationState {
    Verified,
    Unsigned,
    Invalid,
    UnknownKey,
    RevokedKey,
}

#[async_trait]
pub trait SignatureProvider: Send + Sync {
    async fn sign(
        &self,
        domain: &str,
        canonical_payload: &[u8],
    ) -> Result<SignatureEnvelope, SignatureError>;
    async fn verify(
        &self,
        domain: &str,
        canonical_payload: &[u8],
        envelope: &SignatureEnvelope,
    ) -> Result<SignatureVerification, SignatureError>;
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum SignatureError {
    #[error("key unavailable")]
    KeyUnavailable,
    #[error("key revoked")]
    KeyRevoked,
    #[error("signature invalid")]
    Invalid,
    #[error("unsupported algorithm")]
    UnsupportedAlgorithm,
    #[error("keystore error: {0}")]
    KeyStore(String),
}
```

## 9.6 Sidecar import

Import is idempotent by `artifact_sha256`.

Conflict key:

```text
annotation_id + revision
```

Rules:

- identical revision bytes are a no-op;
- higher imported revision creates an imported-layer revision;
- divergent equal revisions create a conflict task;
- imported private layers remain imported and are not merged into the local
  private layer automatically;
- invalid signatures are quarantined;
- unsigned artifacts require explicit user confirmation;
- object digests are verified before object commit.

---

# 10. Inference Subsystem

## 10.1 Exact traits

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BackendDescriptor {
    pub backend_id: String,
    pub backend_kind: BackendKind,
    pub display_name: String,
    pub supported_formats: Vec<ModelFormat>,
    pub supports_streaming: bool,
    pub supports_embeddings: bool,
    pub accelerator: AcceleratorKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackendKind {
    LocalLm,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelFormat {
    Gguf,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AcceleratorKind {
    Cpu,
    Gpu,
    Npu,
    Hybrid,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BackendHealth {
    pub backend_id: String,
    pub available: bool,
    pub accelerator: AcceleratorKind,
    pub detail: String,
    pub checked_at_ms: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelManifest {
    pub model_id: String,
    pub display_name: String,
    pub family: String,
    pub model_kind: ModelKind,
    pub format: ModelFormat,
    pub artifact_sha256: String,
    pub artifact_size_bytes: u64,
    pub context_tokens: u32,
    pub recommended_ram_bytes: u64,
    pub supported_backends: Vec<BackendKind>,
    pub prompt_template_id: String,
    pub license_spdx: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelKind {
    Chat,
    Embedding,
    Reranker,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionOptions {
    pub context_tokens: u32,
    pub max_output_tokens: u32,
    pub temperature: f32,
    pub top_p: f32,
    pub seed: u64,
    pub stop_sequences: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub request_id: String,
    pub system_prompt: String,
    pub messages: Vec<InferenceMessage>,
    pub evidence: Vec<InferenceEvidence>,
    pub options: SessionOptions,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InferenceMessage {
    pub role: String,
    pub body: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InferenceEvidence {
    pub citation_key: String,
    pub title: String,
    pub text: String,
    pub source_kind: String,
    pub source_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum InferenceChunk {
    TextDelta { text: String },
    CitationMarker { citation_key: String },
    Usage { input_tokens: u32, output_tokens: u32 },
    Completed { finish_reason: String },
}

pub type InferenceStream =
    Pin<Box<dyn Stream<Item = Result<InferenceChunk, InferenceError>> + Send>>;

#[async_trait]
pub trait InferenceBackend: Send + Sync {
    fn descriptor(&self) -> BackendDescriptor;
    async fn health(&self) -> BackendHealth;
    async fn validate_model(
        &self,
        manifest: &ModelManifest,
        artifact_path: &Path,
    ) -> Result<(), InferenceError>;
    async fn open_session(
        &self,
        manifest: ModelManifest,
        artifact_path: &Path,
        options: SessionOptions,
    ) -> Result<Box<dyn InferenceSession>, InferenceError>;
}

#[async_trait]
pub trait InferenceSession: Send {
    fn session_id(&self) -> &str;
    fn model_id(&self) -> &str;
    async fn generate(
        &mut self,
        request: InferenceRequest,
    ) -> Result<InferenceStream, InferenceError>;
    async fn cancel(&mut self, request_id: &str) -> Result<(), InferenceError>;
    async fn close(self: Box<Self>) -> Result<(), InferenceError>;
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum InferenceError {
    #[error("backend unavailable")]
    BackendUnavailable,
    #[error("model not installed")]
    ModelNotInstalled,
    #[error("model incompatible: {0}")]
    ModelIncompatible(String),
    #[error("insufficient memory")]
    InsufficientMemory,
    #[error("context limit exceeded")]
    ContextLimitExceeded,
    #[error("request cancelled")]
    Cancelled,
    #[error("runtime failure: {0}")]
    Runtime(String),
}
```

## 10.2 Embedding trait

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmbeddingDescriptor {
    pub embedding_space_id: String,
    pub model_id: String,
    pub dimensions: u32,
    pub distance_metric: String,
    pub normalized: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmbeddingInput {
    pub input_id: String,
    pub text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmbeddingOutput {
    pub input_id: String,
    pub vector: Vec<f32>,
}

#[async_trait]
pub trait EmbeddingBackend: Send + Sync {
    fn descriptor(&self) -> EmbeddingDescriptor;
    async fn embed(
        &self,
        inputs: Vec<EmbeddingInput>,
    ) -> Result<Vec<EmbeddingOutput>, InferenceError>;
}
```

## 10.3 Release backends

| Platform | Chat backend | Embedding backend |
|---|---|---|
| Android | `WllamaInferenceBackend` | `WllamaEmbeddingBackend` |
| Linux | `WllamaInferenceBackend` | `WllamaEmbeddingBackend` |
| Windows | `WllamaInferenceBackend` | `WllamaEmbeddingBackend` |

`crates/inference-wllama` provides the sole release backend on every platform.
No null backend exists in the release composition.
When no compatible installed model exists, Ask remains a complete non-AI state:

- scope selection works;
- model recommendation opens;
- existing conversations remain readable;
- the Send action is replaced by `Install a model`;
- Reader, Notes and Memory remain fully usable.

## 10.4 Selection policy

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BackendSelectionInput {
    pub platform: String,
    pub installed_models: Vec<ModelInstallationSummary>,
    pub backend_health: Vec<BackendHealth>,
    pub free_ram_bytes: u64,
    pub battery_saver: bool,
    pub thermal_state: String,
    pub user_preferred_backend_id: Option<String>,
    pub user_preferred_model_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelInstallationSummary {
    pub model_id: String,
    pub backend_id: String,
    pub active: bool,
    pub last_used_at_ms: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BackendSelection {
    pub backend_id: String,
    pub model_id: String,
    pub reason: String,
}
```

Selection order:

1. user-pinned compatible healthy backend/model;
2. the sole release backend (`inference-wllama`) when a compatible model is
   installed and RAM/thermal policy permits;
3. otherwise no backend is selected and Ask remains a complete non-AI state
   (§10.3).

With a single release backend there is no cross-backend arbitration; the
deterministic tie-breaker (lowest stable identifier) applies only between
multiple installed models on the one backend.

Policy never downloads a model automatically.

## 10.5 Model lifecycle

```mermaid
stateDiagram-v2
    [*] --> Available
    Available --> Downloading
    Downloading --> Verifying
    Verifying --> Installed
    Installed --> Active
    Active --> Installed
    Installed --> Removing
    Removing --> Available
    Verifying --> Quarantined
```

Model removal:

- waits for active sessions to close or user cancellation;
- removes the installation location;
- retains manifest, run metadata and conversation content;
- invalidates no citation;
- schedules embeddings from that model for deletion/rebuild only after another
  embedding space is selected.

---

# 11. Retrieval & Citation Subsystem

## 11.1 Staged pipeline

```mermaid
flowchart TD
    Q["Query + scope"] --> L["Lexical candidates"]
    Q --> V["Vector candidates"]
    L --> M["Merge"]
    V --> M
    M --> R["Rerank"]
    R --> B["Context budget"]
    B --> I["Inference"]
    I --> C["Citation validation"]
```

Stages:

1. normalize query without discarding user spelling;
2. resolve explicit scope;
3. retrieve lexical candidates;
4. retrieve vector candidates when an embedding space is ready;
5. reciprocal-rank merge;
6. deduplicate by retrieval chunk and overlapping locator;
7. optional local rerank;
8. enforce source diversity;
9. allocate context budget;
10. map evidence to stable citation keys;
11. generate;
12. validate emitted citation keys;
13. persist response and citations atomically.

## 11.2 Exact contracts

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RetrievalQuery {
    pub text: String,
    pub scope: RetrievalScope,
    pub limit: u32,
    pub context_budget_tokens: u32,
    pub include_source_kinds: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RetrievalScope {
    Passage {
        retrieval_chunk_id: String,
    },
    Article {
        pack_id: PackId,
        article_key: ArticleKey,
    },
    Packs {
        pack_ids: Vec<PackId>,
    },
    Notebook {
        notebook_id: String,
    },
    AllLocal,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RetrievalHit {
    pub retrieval_chunk_id: String,
    pub source_kind: String,
    pub source_id: String,
    pub title: String,
    pub text: String,
    pub locator: CitationLocator,
    pub lexical_score: Option<f32>,
    pub vector_score: Option<f32>,
    pub rerank_score: f32,
    pub token_count: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CitationLocator {
    pub pack_id: Option<PackId>,
    pub article_key: Option<ArticleKey>,
    pub annotation_id: Option<String>,
    pub start_char: u32,
    pub end_char: u32,
    pub quote_digest: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContextEvidence {
    pub citation_key: String,
    pub retrieval_chunk_id: String,
    pub title: String,
    pub text: String,
    pub source_kind: String,
    pub source_id: String,
    pub locator: CitationLocator,
    pub tokens: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContextSet {
    pub evidence: Vec<ContextEvidence>,
    pub used_tokens: u32,
    pub budget_tokens: u32,
    pub omitted_count: u32,
}

#[async_trait]
pub trait Retriever: Send + Sync {
    async fn retrieve(
        &self,
        query: RetrievalQuery,
    ) -> Result<Vec<RetrievalHit>, RetrievalError>;
}

#[async_trait]
pub trait ContextBuilder: Send + Sync {
    async fn build(
        &self,
        query: &RetrievalQuery,
        hits: Vec<RetrievalHit>,
    ) -> Result<ContextSet, RetrievalError>;
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum RetrievalError {
    #[error("scope invalid: {0}")]
    InvalidScope(String),
    #[error("source unavailable")]
    SourceUnavailable,
    #[error("index unavailable")]
    IndexUnavailable,
    #[error("embedding space unavailable")]
    EmbeddingUnavailable,
    #[error("storage error: {0}")]
    Storage(String),
}
```

## 11.3 Context budget

For a model context limit `C`:

```text
system reserve = min(2048, floor(C * 0.12))
conversation reserve = min(4096, floor(C * 0.25))
output reserve = configured max output tokens
evidence budget = C - system reserve - conversation reserve - output reserve
```

Evidence rules:

- maximum 12 chunks;
- maximum 3 adjacent chunks from one document;
- at least 2 source documents when the scope contains 2 or more;
- one complete sentence on both ends where possible;
- no chunk below 40 characters;
- overlapping chunks merge before budgeting;
- truncation never changes a citation locator silently.

## 11.4 Citation mapping

Citation keys are deterministic within a run:

```text
[S1], [S2], ... [S12]
```

Each key maps to one `ContextEvidence`.
The model is instructed to cite only known keys.
Unknown keys are rendered as invalid and excluded from
`message_citations`.
An assistant response with factual paragraphs and zero valid citations in a
grounded scope is stored with status `failed` and shown as an actionable retry,
not as a successful cited answer.

---

# 12. Lore Memory Subsystem

## 12.1 Memory lifecycle

```mermaid
stateDiagram-v2
    [*] --> Evidence
    Evidence --> Candidate
    Candidate --> Accepted
    Candidate --> Rejected
    Accepted --> Corrected
    Corrected --> Accepted
    Accepted --> Superseded
    Accepted --> Deleted
```

Layers:

1. `lore_evidence` — immutable evidence reference and excerpt;
2. `lore_candidates` — proposed statement awaiting policy/user review;
3. `lore_memories` — accepted identity and status;
4. `lore_memory_revisions` — append-only statement/correction history;
5. `lore_memory_evidence` — revision-specific evidence links.

## 12.2 Exact contracts

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoreCandidate {
    pub lore_candidate_id: String,
    pub statement: String,
    pub subject_key: String,
    pub predicate_key: String,
    pub object: serde_json::Value,
    pub confidence: f32,
    pub evidence: Vec<LoreEvidence>,
    pub created_at_ms: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoreEvidence {
    pub lore_evidence_id: String,
    pub source_kind: String,
    pub source_id: String,
    pub excerpt: String,
    pub locator: serde_json::Value,
    pub content_digest: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoreMemory {
    pub memory_id: String,
    pub subject_key: String,
    pub predicate_key: String,
    pub statement: String,
    pub object: serde_json::Value,
    pub revision: u32,
    pub pinned: bool,
    pub status: LoreMemoryStatus,
    pub evidence: Vec<LoreEvidence>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoreMemoryStatus {
    Active,
    Superseded,
    Deleted,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoreCorrection {
    pub statement: String,
    pub object: serde_json::Value,
    pub correction_reason: String,
    pub evidence_ids: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoreQuery {
    pub text: String,
    pub subject_keys: Vec<String>,
    pub include_deleted: bool,
    pub limit: u32,
}

#[async_trait]
pub trait LoreService: Send + Sync {
    async fn extract_candidates(
        &self,
        conversation_id: &str,
        inference_run_id: &str,
    ) -> Result<Vec<LoreCandidate>, LoreError>;
    async fn list_inbox(
        &self,
        offset: u32,
        limit: u32,
    ) -> Result<Vec<LoreCandidate>, LoreError>;
    async fn accept(
        &self,
        lore_candidate_id: &str,
    ) -> Result<LoreMemory, LoreError>;
    async fn reject(
        &self,
        lore_candidate_id: &str,
    ) -> Result<(), LoreError>;
    async fn correct(
        &self,
        memory_id: &str,
        expected_revision: u32,
        correction: LoreCorrection,
    ) -> Result<LoreMemory, LoreError>;
    async fn set_pinned(
        &self,
        memory_id: &str,
        pinned: bool,
    ) -> Result<LoreMemory, LoreError>;
    async fn delete(
        &self,
        memory_id: &str,
        expected_revision: u32,
    ) -> Result<(), LoreError>;
    async fn query(
        &self,
        query: LoreQuery,
    ) -> Result<Vec<LoreMemory>, LoreError>;
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum LoreError {
    #[error("candidate not found")]
    CandidateNotFound,
    #[error("memory not found")]
    MemoryNotFound,
    #[error("revision conflict")]
    RevisionConflict,
    #[error("evidence missing")]
    EvidenceMissing,
    #[error("policy rejected proposal: {0}")]
    PolicyRejected(String),
    #[error("storage error: {0}")]
    Storage(String),
}
```

## 12.3 Extraction policy

Candidate extraction is opt-in per conversation unless the user enables a
global review-inbox policy.

Allowed candidate classes:

- stable preference;
- explicit biographical assertion by the user;
- project constraint;
- named relationship;
- correction of an existing memory;
- recurring workflow preference.

Forbidden automatic candidates:

- inferred sensitive attribute;
- medical diagnosis;
- legal status;
- credential or secret;
- incidental one-time request;
- content quoted from a source as if it described the user;
- model speculation.

No candidate becomes accepted without:

- explicit user action; or
- a user-configured deterministic promotion policy whose rule and evidence are
  displayed in the Memory Centre.

## 12.4 Consolidation

Consolidation groups candidates by normalized
`subject_key + predicate_key`.

It may:

- merge duplicate evidence;
- propose a correction;
- propose supersession;
- shorten a statement without changing meaning.

It may not:

- delete evidence;
- rewrite a user correction;
- merge contradictory objects;
- promote a candidate.

## 12.5 Deletion

Deletion:

1. marks memory status `deleted`;
2. records `deleted_at_ms`;
3. removes memory content from retrieval documents;
4. deletes associated chunk embeddings;
5. excludes it from inference;
6. preserves revision metadata necessary to prevent sync resurrection;
7. excludes deleted text from diagnostics and ordinary export;
8. includes a tombstone without deleted text in signed sync export.

---

# 13. Entitlement Architecture

## 13.1 Capability keys

```text
reader.core
reader.unlimited_packs
annotations.rich_media
ask.local_inference
memory.lore
backup.encrypted
sidecar.signed_export
models.premium_catalog
```

`reader.core`, text highlights, text comments, bookmarks, local import and
human-readable export require no paid entitlement.

## 13.2 Provider boundary

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderEntitlementEvent {
    pub provider: String,
    pub provider_event_id: String,
    pub subject_reference: String,
    pub product_id: String,
    pub event_kind: String,
    pub effective_at_ms: i64,
    pub expires_at_ms: Option<i64>,
    pub raw_event_digest: String,
}

#[async_trait]
pub trait EntitlementSource: Send + Sync {
    fn provider_id(&self) -> &'static str;
    async fn refresh(
        &self,
        request: EntitlementRefreshRequest,
    ) -> Result<Vec<ProviderEntitlementEvent>, EntitlementError>;
    async fn restore(
        &self,
        request: EntitlementRestoreRequest,
    ) -> Result<Vec<ProviderEntitlementEvent>, EntitlementError>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntitlementRefreshRequest {
    pub device_id: String,
    pub provider_user_reference: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntitlementRestoreRequest {
    pub device_id: String,
    pub restore_token: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CapabilityDecision {
    pub capability_key: String,
    pub allowed: bool,
    pub source: CapabilityDecisionSource,
    pub valid_until_ms: Option<i64>,
    pub explanation: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityDecisionSource {
    Core,
    SignedArtifact,
    GracePolicy,
    Denied,
}

#[async_trait]
pub trait CapabilityPolicy: Send + Sync {
    async fn decide(
        &self,
        capability_key: &str,
        at_ms: i64,
    ) -> Result<CapabilityDecision, EntitlementError>;
    async fn all_decisions(
        &self,
        at_ms: i64,
    ) -> Result<Vec<CapabilityDecision>, EntitlementError>;
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum EntitlementError {
    #[error("offline")]
    Offline,
    #[error("provider unavailable")]
    ProviderUnavailable,
    #[error("restore failed")]
    RestoreFailed,
    #[error("artifact signature invalid")]
    InvalidArtifactSignature,
    #[error("artifact expired")]
    ArtifactExpired,
    #[error("issuer revoked")]
    IssuerRevoked,
    #[error("coupon already redeemed")]
    CouponAlreadyRedeemed,
    #[error("policy error: {0}")]
    Policy(String),
}
```

Release provider implementation:

```text
RevenueCatEntitlementSource
```

RevenueCat is never queried from domain or UI code directly.

## 13.3 Signed offline artifact

Media type:

```text
application/vnd.enzime.entitlement+cbor
```

Canonical CDDL:

```cddl
enzime-entitlement-v1 = {
  "schema": "enzime.entitlement.cbor.v1",
  "artifact_id": tstr,
  "issuer_key_id": tstr,
  "subject": subject,
  "issued_at_ms": int,
  "not_before_ms": int,
  "expires_at_ms": int / null,
  "grants": [1* grant],
  "provider_receipts": [* provider-receipt],
  "nonce": bstr .size 32,
  "signature": signature
}

subject = {
  "kind": "device" / "household",
  "subject_id": tstr
}

grant = {
  "capability_key": tstr,
  "grant_kind": "perpetual" / "term" / "quantity",
  "quantity": uint / null,
  "valid_until_ms": int / null
}

provider-receipt = {
  "provider": tstr,
  "event_digest": tstr
}
```

Signing domain:

```text
enzime.entitlement.v1\0
```

The artifact contains no provider secret.

## 13.4 Coupon ledger

An offline coupon is a signed single-redemption offer containing:

- `coupon_id`;
- capability grants;
- issuer key;
- validity window;
- random 256-bit coupon secret;
- signature.

Redemption derives:

```text
receipt_sha256 =
SHA256("enzime.coupon.receipt.v1\0" || coupon_id || device_id || coupon_secret)
```

The local ledger prevents repeat redemption on one device.
Reconciliation may detect cross-device reuse later.
Reconciliation cannot revoke unrelated perpetual grants or user data.

## 13.5 Capability gating

The UI receives `CapabilityDecision`.
It does not receive raw RevenueCat state.

Denied paid actions:

- remain visible when discoverability is useful;
- state the capability and purchase/restore path;
- never destroy or hide existing user data;
- never block export of user-created data;
- never block core reading.

---

# 14. Tauri IPC Bridge

## 14.1 IPC rules

Every command:

- uses the exact snake_case name in this section;
- receives one request object;
- returns `CommandResult<T>`;
- validates length, count, enum and path/URI constraints;
- performs no blocking work on the UI thread;
- returns a durable job ID for long work;
- emits only events declared in this section;
- maps internal errors to `AppErrorDto`;
- never returns a raw filesystem path unless the user explicitly selected it
  and the DTO field is named `displayPath`.

```ts
export interface CommandResult<T> {
  ok: boolean;
  value?: T;
  error?: AppErrorDto;
}

export interface AppErrorDto {
  code: AppErrorCode;
  message: string;
  action?: ErrorActionDto;
  correlationId: string;
  detail?: Record<string, unknown>;
}

export type AppErrorCode =
  | "invalid_request"
  | "not_found"
  | "permission_denied"
  | "offline"
  | "storage_full"
  | "source_changed"
  | "verification_failed"
  | "signature_invalid"
  | "format_invalid"
  | "unsupported"
  | "revision_conflict"
  | "capability_denied"
  | "backend_unavailable"
  | "model_unavailable"
  | "cancelled"
  | "busy"
  | "internal";

export interface ErrorActionDto {
  label: string;
  action:
    | "retry"
    | "open_settings"
    | "choose_storage"
    | "install_model"
    | "restore_purchase"
    | "view_quarantine"
    | "report_problem";
}
```

## 14.2 Shared TypeScript entities

```ts
export type Ulid = string;
export type Sha256Hex = string;
export type UnixTimeMs = number;

export interface PageRequest {
  offset: number;
  limit: number;
}

export interface PageResult<T> {
  items: T[];
  total?: number;
  nextOffset?: number;
}

export interface PackManifest {
  packId: Ulid;
  corpusId: string;
  displayName: string;
  description: string;
  languageBcp47: string;
  packKind: "full_zim" | "tailored_zim" | "enzime_pack";
  activeVersionId?: Ulid;
  versions: PackVersionSummary[];
}

export interface PackVersionSummary {
  packVersionId: Ulid;
  upstreamVersion: string;
  artifactSha256: Sha256Hex;
  artifactSizeBytes: number;
  verificationState: "unverified" | "verified" | "quarantined";
}

export interface PackLocationSummary {
  packLocationId: Ulid;
  packVersionId: Ulid;
  storageRootId: Ulid;
  storageName: string;
  locatorKind: "active" | "retained" | "staging" | "quarantine";
  byteLength: number;
  lastVerifiedAtMs?: UnixTimeMs;
}

export interface CatalogEntry {
  catalogEntryId: Ulid;
  corpusId: string;
  upstreamVersion: string;
  title: string;
  description: string;
  languageBcp47: string;
  category: string;
  publisher: string;
  artifactUrl: string;
  artifactSizeBytes: number;
  artifactSha256: Sha256Hex;
  signatureUrl?: string;
  trust: TrustPresentation;
}

export interface TrustPresentation {
  state: "verified" | "unsigned" | "invalid" | "unknown_key";
  signer?: string;
  explanation: string;
}

export interface DownloadJob {
  downloadJobId: Ulid;
  title: string;
  state:
    | "planned"
    | "fetching"
    | "paused"
    | "verifying"
    | "staged"
    | "activating"
    | "completed"
    | "failed"
    | "cancelled";
  bytesVerified: number;
  expectedSizeBytes: number;
  progressFraction: number;
  speedBytesPerSecond?: number;
  etaSeconds?: number;
  error?: AppErrorDto;
  createdAtMs: UnixTimeMs;
  updatedAtMs: UnixTimeMs;
}

export interface InstallPlan {
  installPlanId: Ulid;
  catalogEntry: CatalogEntry;
  destination: StorageDestination;
  transferMode: "ranged" | "full" | "local_copy";
  downloadBytes: number;
  temporaryBytes: number;
  finalBytes: number;
  freeBytesBefore: number;
  freeBytesAfter: number;
  verification: VerificationPlan;
  replacesPackVersionId?: Ulid;
  retainedPackVersionId?: Ulid;
}

export interface StorageDestination {
  storageRootId: Ulid;
  displayName: string;
  writable: boolean;
}

export interface VerificationPlan {
  expectedSizeBytes: number;
  expectedSha256: Sha256Hex;
  signatureRequired: boolean;
  zimStructureCheck: boolean;
}

export interface ArticleResponse {
  packId: Ulid;
  articleKey: string;
  title: string;
  sanitizedHtml: string;
  normalizedTextDigest: Sha256Hex;
  breadcrumb: BreadcrumbItem[];
  previousArticleKey?: string;
  nextArticleKey?: string;
  annotations: AnnotationRecord[];
}

export interface BreadcrumbItem {
  label: string;
  articleKey?: string;
}

export interface TitleSearchHit {
  packId: Ulid;
  articleKey: string;
  title: string;
  snippet?: string;
  score: number;
}

export interface SelectorBundle {
  article: {
    packId: Ulid;
    articleKey: string;
    sourceRevisionDigest?: Sha256Hex;
  };
  textQuote?: {
    exact: string;
    prefix: string;
    suffix: string;
  };
  textPosition?: {
    normalizedStart: number;
    normalizedEnd: number;
  };
  domPath?: {
    path: string;
    nodeTextDigest: Sha256Hex;
  };
  imageRegion?: {
    resourceEntryKey: string;
    xFraction: number;
    yFraction: number;
    widthFraction: number;
    heightFraction: number;
  };
  capturedContext: string;
}

export interface AnnotationStyle {
  colorToken: string;
  emphasis: string;
  strokeWidth?: number;
}

export interface AnnotationDraft {
  annotationType:
    | "highlight"
    | "comment"
    | "ink"
    | "voice"
    | "bookmark_note";
  selector: SelectorBundle;
  bodyText?: string;
  style: AnnotationStyle;
  notebookId?: Ulid;
  tagIds: Ulid[];
  sidecarLayerId: Ulid;
}

export interface AnnotationRecord {
  annotationId: Ulid;
  annotationType:
    | "highlight"
    | "comment"
    | "ink"
    | "voice"
    | "bookmark_note";
  selector: SelectorBundle;
  bodyText?: string;
  style: AnnotationStyle;
  notebookId?: Ulid;
  tags: TagSummary[];
  sidecarLayerId: Ulid;
  resolution: AnnotationResolution;
  pinned: boolean;
  revision: number;
  createdAtMs: UnixTimeMs;
  updatedAtMs: UnixTimeMs;
}

export interface AnnotationResolution {
  state: "resolved" | "ambiguous" | "source_unavailable" | "orphaned";
  confidence: number;
  matchedStart?: number;
  matchedEnd?: number;
  explanation: string;
}

export interface TagSummary {
  tagId: Ulid;
  name: string;
  colorToken: string;
}

export interface NotebookNode {
  notebookId: Ulid;
  parentNotebookId?: Ulid;
  name: string;
  description: string;
  annotationCount: number;
  children: NotebookNode[];
}

export interface SidecarLayer {
  sidecarLayerId: Ulid;
  name: string;
  visibility: "private" | "shared" | "imported";
  trustState: "local" | "verified" | "unverified" | "rejected";
  enabled: boolean;
  annotationCount: number;
}

export interface ConversationSummary {
  conversationId: Ulid;
  title: string;
  scope: RetrievalScope;
  backendId?: string;
  updatedAtMs: UnixTimeMs;
}

export interface ConversationMessage {
  messageId: Ulid;
  conversationId: Ulid;
  role: "user" | "assistant" | "system_notice";
  bodyMarkdown: string;
  sequenceNo: number;
  citations: CitationRecord[];
  createdAtMs: UnixTimeMs;
}

export interface CitationRecord {
  citationIndex: number;
  retrievalChunkId: Ulid;
  quoteText: string;
  locator: CitationLocator;
  confidence: number;
}

export interface CitationLocator {
  packId?: Ulid;
  articleKey?: string;
  annotationId?: Ulid;
  startChar: number;
  endChar: number;
  quoteDigest: Sha256Hex;
}

export type RetrievalScope =
  | {
      kind: "passage";
      retrievalChunkId: Ulid;
    }
  | {
      kind: "article";
      packId: Ulid;
      articleKey: string;
    }
  | {
      kind: "packs";
      packIds: Ulid[];
    }
  | {
      kind: "notebook";
      notebookId: Ulid;
    }
  | {
      kind: "all_local";
    };

export interface BackendStatus {
  backendId: string;
  displayName: string;
  available: boolean;
  accelerator: "cpu" | "gpu" | "npu" | "hybrid";
  activeModel?: ModelSummary;
  detail: string;
  checkedAtMs: UnixTimeMs;
}

export interface ModelSummary {
  modelId: string;
  displayName: string;
  family: string;
  modelKind: "chat" | "embedding" | "reranker";
  format: "gguf";
  artifactSizeBytes: number;
  installed: boolean;
  active: boolean;
  compatible: boolean;
  compatibilityExplanation: string;
}

export interface LoreCandidate {
  loreCandidateId: Ulid;
  statement: string;
  subjectKey: string;
  predicateKey: string;
  object: unknown;
  confidence: number;
  evidence: LoreEvidence[];
  createdAtMs: UnixTimeMs;
}

export interface LoreEvidence {
  loreEvidenceId: Ulid;
  sourceKind: "message" | "annotation" | "article" | "user_assertion";
  sourceId: string;
  excerpt: string;
  locator: Record<string, unknown>;
  contentDigest: Sha256Hex;
}

export interface LoreMemory {
  memoryId: Ulid;
  subjectKey: string;
  predicateKey: string;
  statement: string;
  object: unknown;
  revision: number;
  pinned: boolean;
  status: "active" | "superseded" | "deleted";
  evidence: LoreEvidence[];
  createdAtMs: UnixTimeMs;
  updatedAtMs: UnixTimeMs;
}

export interface CapabilityDecision {
  capabilityKey: CapabilityKey;
  allowed: boolean;
  source: "core" | "signed_artifact" | "grace_policy" | "denied";
  validUntilMs?: UnixTimeMs;
  explanation: string;
}

export type CapabilityKey =
  | "reader.core"
  | "reader.unlimited_packs"
  | "annotations.rich_media"
  | "ask.local_inference"
  | "memory.lore"
  | "backup.encrypted"
  | "sidecar.signed_export"
  | "models.premium_catalog";

export interface CapabilityReport {
  capabilityReportId: Ulid;
  platform: "android" | "linux" | "windows";
  appVersion: string;
  cpu: string;
  ramBytes: number;
  freeStorageBytes: number;
  accelerators: AcceleratorReport[];
  zimReady: boolean;
  inferenceReady: boolean;
  warnings: ReadinessFinding[];
  createdAtMs: UnixTimeMs;
}

export interface AcceleratorReport {
  kind: "cpu" | "gpu" | "npu";
  name: string;
  available: boolean;
  detail: string;
}

export interface ReadinessFinding {
  code: string;
  severity: "info" | "warning" | "error";
  title: string;
  detail: string;
  action?: ErrorActionDto;
}

export interface SystemReadiness {
  reader: ReadinessState;
  storage: ReadinessState;
  inference: ReadinessState;
  backup: ReadinessState;
  entitlement: ReadinessState;
}

export interface ReadinessState {
  state: "ready" | "limited" | "blocked";
  summary: string;
  findings: ReadinessFinding[];
}

export interface UserTask {
  taskId: Ulid;
  taskKind:
    | "download_attention"
    | "annotation_ambiguous"
    | "memory_review"
    | "backup_due"
    | "model_attention";
  title: string;
  detail: Record<string, unknown>;
  priority: number;
  createdAtMs: UnixTimeMs;
}

export interface HomeSnapshot {
  recentPacks: PackManifest[];
  continueReading?: ReaderHistoryItem;
  openTasks: UserTask[];
  memoryInboxCount: number;
  readiness: SystemReadiness;
}

export interface ReaderHistoryItem {
  packId: Ulid;
  articleKey: string;
  title: string;
  visitedAtMs: UnixTimeMs;
}

export interface SettingsSnapshot {
  networkPolicy: NetworkPolicy;
  computePolicy: ComputePolicy;
  accessibility: AccessibilitySettings;
  backup: BackupSettings;
  capabilities: CapabilityDecision[];
}

export interface NetworkPolicy {
  allowInternetCatalogRefresh: boolean;
  allowInternetDownloads: boolean;
  allowMeteredDownloads: boolean;
  wifiOnlyLargeTransfers: boolean;
}

export interface ComputePolicy {
  preferredBackendId?: string;
  preferredChatModelId?: string;
  preferredEmbeddingModelId?: string;
  batterySaverBehavior: "pause" | "cpu_only" | "continue";
  thermalBehavior: "pause" | "reduce" | "continue";
}

export interface AccessibilitySettings {
  fontScale: number;
  reduceMotion: boolean;
  highContrast: boolean;
  readerTypeface: "system" | "literata" | "atkinson";
  lineHeight: number;
}

export interface BackupSettings {
  reminderIntervalDays: number;
  includeConversations: boolean;
  includeLore: boolean;
  encryptByDefault: boolean;
}
```

## 14.3 Command DTOs

```ts
export interface EmptyRequest {}

export interface PackGetRequest {
  packId: Ulid;
}

export interface PackMetadataRequest {
  packVersionId: Ulid;
}

export interface CatalogSearchRequest {
  text?: string;
  languages: string[];
  categories: string[];
  maxSizeBytes?: number;
  offset: number;
  limit: number;
}

export interface CatalogRefreshRequest {
  catalogSourceIds?: Ulid[];
}

export interface InstallPlanRequest {
  catalogEntryId: Ulid;
  destinationStorageRootId: Ulid;
}

export interface InstallStartRequest {
  installPlanId: Ulid;
}

export interface JobRequest {
  jobId: Ulid;
}

export interface PackRemoveRequest {
  packVersionId: Ulid;
  removeUserData: boolean;
}

export interface PackImportRequest {
  sourceToken: string;
  destinationStorageRootId: Ulid;
}

export interface ReaderOpenRequest {
  packId: Ulid;
  articleKey?: string;
}

export interface ReaderArticleRequest {
  packId: Ulid;
  articleKey: string;
}

export interface ReaderSearchRequest {
  text: string;
  packIds: Ulid[];
  offset: number;
  limit: number;
}

export interface ReaderPositionSaveRequest {
  packId: Ulid;
  articleKey: string;
  scrollAnchor: string;
  progressFraction: number;
}

export interface BookmarkSetRequest {
  packId: Ulid;
  articleKey: string;
  title: string;
  bookmarked: boolean;
}

export interface AnnotationCreateRequest {
  draft: AnnotationDraft;
}

export interface AnnotationUpdateRequest {
  annotationId: Ulid;
  expectedRevision: number;
  draft: AnnotationDraft;
}

export interface AnnotationDeleteRequest {
  annotationId: Ulid;
  expectedRevision: number;
}

export interface AnnotationRestoreRequest {
  annotationId: Ulid;
}

export interface AnnotationListRequest {
  packIds?: Ulid[];
  notebookIds?: Ulid[];
  tagIds?: Ulid[];
  sidecarLayerIds?: Ulid[];
  resolutionStates?: AnnotationResolution["state"][];
  text?: string;
  offset: number;
  limit: number;
}

export interface AnnotationAssetBeginRequest {
  annotationId: Ulid;
  assetKind: "ink_svg" | "voice_opus" | "image" | "file";
  mimeType: string;
  expectedBytes: number;
  durationMs?: number;
}

export interface AnnotationAssetBeginResponse {
  uploadToken: string;
  maximumChunkBytes: number;
}

export interface AnnotationAssetChunkRequest {
  uploadToken: string;
  sequenceNo: number;
  bytesBase64: string;
}

export interface AnnotationAssetCommitRequest {
  uploadToken: string;
  expectedSha256: Sha256Hex;
}

export interface NotebookCreateRequest {
  parentNotebookId?: Ulid;
  name: string;
  description: string;
}

export interface NotebookUpdateRequest {
  notebookId: Ulid;
  name: string;
  description: string;
  parentNotebookId?: Ulid;
}

export interface TagCreateRequest {
  name: string;
  colorToken: string;
}

export interface SidecarExportRequest {
  sidecarLayerIds: Ulid[];
  includeObjects: boolean;
  destinationToken: string;
  sign: boolean;
}

export interface SidecarImportPlanRequest {
  sourceToken: string;
}

export interface SidecarImportPlan {
  artifactSha256: Sha256Hex;
  signatureState: "verified" | "unsigned" | "invalid" | "unknown_key";
  signer?: string;
  annotationCount: number;
  notebookCount: number;
  tagCount: number;
  objectBytes: number;
  conflicts: SidecarConflict[];
}

export interface SidecarConflict {
  entityKind: string;
  entityId: string;
  explanation: string;
}

export interface SidecarImportCommitRequest {
  artifactSha256: Sha256Hex;
  acceptUnsigned: boolean;
  layerName: string;
}

export interface ConversationCreateRequest {
  title: string;
  scope: RetrievalScope;
}

export interface ConversationGetRequest {
  conversationId: Ulid;
}

export interface ConversationListRequest {
  offset: number;
  limit: number;
}

export interface AskStartRequest {
  conversationId: Ulid;
  userMessage: string;
  scope: RetrievalScope;
  backendId?: string;
  modelId?: string;
}

export interface AskStartResponse {
  inferenceRunId: Ulid;
  userMessageId: Ulid;
}

export interface AskCancelRequest {
  inferenceRunId: Ulid;
}

export interface CitationOpenRequest {
  messageId: Ulid;
  citationIndex: number;
}

export interface LoreInboxRequest {
  offset: number;
  limit: number;
}

export interface LoreCandidateRequest {
  loreCandidateId: Ulid;
}

export interface LoreMemoryRequest {
  memoryId: Ulid;
}

export interface LoreCorrectRequest {
  memoryId: Ulid;
  expectedRevision: number;
  statement: string;
  object: unknown;
  correctionReason: string;
  evidenceIds: Ulid[];
}

export interface LorePinRequest {
  memoryId: Ulid;
  pinned: boolean;
}

export interface LoreDeleteRequest {
  memoryId: Ulid;
  expectedRevision: number;
}

export interface LoreSearchRequest {
  text: string;
  subjectKeys: string[];
  offset: number;
  limit: number;
}

export interface BackendSelectRequest {
  backendId: string;
  modelId: string;
}

export interface ModelInstallPlanRequest {
  modelId: string;
  destinationStorageRootId: Ulid;
}

export interface ModelRemoveRequest {
  modelInstallationId: Ulid;
}

export interface EntitlementRefreshRequest {
  providerUserReference?: string;
}

export interface EntitlementRestoreRequest {
  restoreToken: string;
}

export interface CouponRedeemRequest {
  couponEnvelopeBase64: string;
}

export interface SettingsUpdateRequest {
  settings: SettingsSnapshot;
}

export interface BackupCreateRequest {
  destinationToken: string;
  encrypted: boolean;
  passphrase?: string;
  includeConversations: boolean;
  includeLore: boolean;
}

export interface BackupRestorePlanRequest {
  sourceToken: string;
  passphrase?: string;
}

export interface BackupRestorePlan {
  backupId: Ulid;
  createdAtMs: UnixTimeMs;
  encrypted: boolean;
  schemaVersion: number;
  packBindings: number;
  annotations: number;
  conversations: number;
  loreMemories: number;
  conflicts: string[];
}

export interface BackupRestoreCommitRequest {
  backupId: Ulid;
  conflictPolicy: "keep_local" | "keep_backup" | "keep_both";
}

export interface DiagnosticExportRequest {
  destinationToken: string;
  includeContentSamples: boolean;
  selectedEntityIds: string[];
}

export interface TaskResolveRequest {
  taskId: Ulid;
  resolution: "resolved" | "dismissed";
}
```

## 14.4 Complete command registry

| Tauri command | Request DTO | Return DTO | Authorization |
|---|---|---|---|
| `app_bootstrap` | `EmptyRequest` | `AppBootstrap` | local |
| `home_snapshot` | `EmptyRequest` | `HomeSnapshot` | local |
| `capability_probe` | `EmptyRequest` | `CapabilityReport` | local |
| `readiness_get` | `EmptyRequest` | `SystemReadiness` | local |
| `storage_roots_list` | `EmptyRequest` | `StorageDestination[]` | local |
| `storage_root_choose` | `EmptyRequest` | `StorageDestination` | user gesture |
| `pack_list` | `PageRequest` | `PageResult<PackManifest>` | local |
| `pack_get` | `PackGetRequest` | `PackManifest` | local |
| `pack_metadata` | `PackMetadataRequest` | `PackVersionSummary` | local |
| `pack_locations` | `PackGetRequest` | `PackLocationSummary[]` | local |
| `pack_import_choose` | `EmptyRequest` | `string` | user gesture |
| `pack_import_plan` | `PackImportRequest` | `InstallPlan` | local |
| `pack_remove` | `PackRemoveRequest` | `void` | confirmed |
| `catalog_search` | `CatalogSearchRequest` | `PageResult<CatalogEntry>` | policy |
| `catalog_refresh` | `CatalogRefreshRequest` | `CatalogRefreshReport[]` | policy |
| `install_plan` | `InstallPlanRequest` | `InstallPlan` | local |
| `install_start` | `InstallStartRequest` | `DownloadJob` | confirmed |
| `job_list` | `PageRequest` | `PageResult<DownloadJob>` | local |
| `job_get` | `JobRequest` | `DownloadJob` | local |
| `job_pause` | `JobRequest` | `DownloadJob` | local |
| `job_resume` | `JobRequest` | `DownloadJob` | policy |
| `job_cancel` | `JobRequest` | `DownloadJob` | confirmed |
| `reader_open` | `ReaderOpenRequest` | `ArticleResponse` | local |
| `reader_article` | `ReaderArticleRequest` | `ArticleResponse` | local |
| `reader_search` | `ReaderSearchRequest` | `PageResult<TitleSearchHit>` | local |
| `reader_position_save` | `ReaderPositionSaveRequest` | `void` | local |
| `reader_history` | `PageRequest` | `PageResult<ReaderHistoryItem>` | local |
| `bookmark_set` | `BookmarkSetRequest` | `void` | local |
| `annotation_create` | `AnnotationCreateRequest` | `AnnotationRecord` | local |
| `annotation_update` | `AnnotationUpdateRequest` | `AnnotationRecord` | local |
| `annotation_delete` | `AnnotationDeleteRequest` | `void` | confirmed |
| `annotation_restore` | `AnnotationRestoreRequest` | `AnnotationRecord` | local |
| `annotation_list` | `AnnotationListRequest` | `PageResult<AnnotationRecord>` | local |
| `annotation_asset_begin` | `AnnotationAssetBeginRequest` | `AnnotationAssetBeginResponse` | capability |
| `annotation_asset_chunk` | `AnnotationAssetChunkRequest` | `void` | local |
| `annotation_asset_commit` | `AnnotationAssetCommitRequest` | `AnnotationRecord` | local |
| `notebook_tree` | `EmptyRequest` | `NotebookNode[]` | local |
| `notebook_create` | `NotebookCreateRequest` | `NotebookNode` | local |
| `notebook_update` | `NotebookUpdateRequest` | `NotebookNode` | local |
| `tag_list` | `EmptyRequest` | `TagSummary[]` | local |
| `tag_create` | `TagCreateRequest` | `TagSummary` | local |
| `sidecar_layers` | `EmptyRequest` | `SidecarLayer[]` | local |
| `sidecar_export_choose` | `EmptyRequest` | `string` | user gesture |
| `sidecar_export` | `SidecarExportRequest` | `BackupArtifactSummary` | capability |
| `sidecar_import_choose` | `EmptyRequest` | `string` | user gesture |
| `sidecar_import_plan` | `SidecarImportPlanRequest` | `SidecarImportPlan` | local |
| `sidecar_import_commit` | `SidecarImportCommitRequest` | `SidecarLayer` | confirmed |
| `conversation_list` | `ConversationListRequest` | `PageResult<ConversationSummary>` | local |
| `conversation_create` | `ConversationCreateRequest` | `ConversationSummary` | local |
| `conversation_get` | `ConversationGetRequest` | `ConversationDetail` | local |
| `ask_start` | `AskStartRequest` | `AskStartResponse` | capability |
| `ask_cancel` | `AskCancelRequest` | `void` | local |
| `citation_open` | `CitationOpenRequest` | `CitationOpenTarget` | local |
| `backend_status` | `EmptyRequest` | `BackendStatus[]` | local |
| `backend_select` | `BackendSelectRequest` | `BackendStatus` | local |
| `model_list` | `EmptyRequest` | `ModelSummary[]` | local |
| `model_install_plan` | `ModelInstallPlanRequest` | `InstallPlan` | capability |
| `model_remove` | `ModelRemoveRequest` | `void` | confirmed |
| `lore_inbox` | `LoreInboxRequest` | `PageResult<LoreCandidate>` | capability |
| `lore_accept` | `LoreCandidateRequest` | `LoreMemory` | capability |
| `lore_reject` | `LoreCandidateRequest` | `void` | capability |
| `lore_search` | `LoreSearchRequest` | `PageResult<LoreMemory>` | capability |
| `lore_correct` | `LoreCorrectRequest` | `LoreMemory` | capability |
| `lore_pin` | `LorePinRequest` | `LoreMemory` | capability |
| `lore_delete` | `LoreDeleteRequest` | `void` | confirmed |
| `entitlement_decisions` | `EmptyRequest` | `CapabilityDecision[]` | local |
| `entitlement_refresh` | `EntitlementRefreshRequest` | `CapabilityDecision[]` | policy |
| `entitlement_restore` | `EntitlementRestoreRequest` | `CapabilityDecision[]` | user gesture |
| `coupon_redeem` | `CouponRedeemRequest` | `CapabilityDecision[]` | confirmed |
| `settings_get` | `EmptyRequest` | `SettingsSnapshot` | local |
| `settings_update` | `SettingsUpdateRequest` | `SettingsSnapshot` | local |
| `backup_choose_destination` | `EmptyRequest` | `string` | user gesture |
| `backup_create` | `BackupCreateRequest` | `JobReference` | confirmed |
| `backup_restore_choose` | `EmptyRequest` | `string` | user gesture |
| `backup_restore_plan` | `BackupRestorePlanRequest` | `BackupRestorePlan` | local |
| `backup_restore_commit` | `BackupRestoreCommitRequest` | `JobReference` | confirmed |
| `diagnostic_export_choose` | `EmptyRequest` | `string` | user gesture |
| `diagnostic_export` | `DiagnosticExportRequest` | `BackupArtifactSummary` | confirmed |
| `task_list` | `PageRequest` | `PageResult<UserTask>` | local |
| `task_resolve` | `TaskResolveRequest` | `void` | local |

## 14.5 Remaining return DTOs

```ts
export interface AppBootstrap {
  schemaVersion: 1;
  appVersion: string;
  platform: "android" | "linux" | "windows";
  firstLaunch: boolean;
  routes: RouteDescriptor[];
  capabilities: CapabilityDecision[];
  readiness: SystemReadiness;
}

export interface RouteDescriptor {
  path: RoutePath;
  enabled: boolean;
}

export type RoutePath =
  | "/"
  | "/library"
  | "/reader/:packId/:articleKey"
  | "/ask"
  | "/notes"
  | "/memory"
  | "/settings"
  | "/setup";

export interface CatalogRefreshReport {
  sourceId: Ulid;
  added: number;
  updated: number;
  removed: number;
  signatureState: string;
  refreshedAtMs: UnixTimeMs;
}

export interface ConversationDetail {
  conversation: ConversationSummary;
  messages: ConversationMessage[];
}

export interface CitationOpenTarget {
  route: string;
  packId?: Ulid;
  articleKey?: string;
  annotationId?: Ulid;
  startChar: number;
  endChar: number;
}

export interface JobReference {
  jobId: Ulid;
  jobKind: "backup_create" | "backup_restore";
}

export interface BackupArtifactSummary {
  artifactSha256: Sha256Hex;
  displayPath: string;
  byteLength: number;
}
```

## 14.6 Event registry

All event names are prefixed `enzime://`.

```ts
export interface EventEnvelope<T> {
  schemaVersion: 1;
  eventId: Ulid;
  emittedAtMs: UnixTimeMs;
  payload: T;
}

export interface JobProgressEvent {
  job: DownloadJob;
}

export interface AskDeltaEvent {
  inferenceRunId: Ulid;
  conversationId: Ulid;
  kind: "text_delta" | "citation_marker" | "usage";
  text?: string;
  citationKey?: string;
  inputTokens?: number;
  outputTokens?: number;
}

export interface AskCompletedEvent {
  inferenceRunId: Ulid;
  conversationId: Ulid;
  assistantMessage: ConversationMessage;
}

export interface EntityChangedEvent {
  entityKind:
    | "pack"
    | "annotation"
    | "conversation"
    | "lore_candidate"
    | "lore_memory"
    | "entitlement"
    | "settings"
    | "task";
  entityId: string;
  change: "created" | "updated" | "deleted";
}

export interface ReadinessChangedEvent {
  readiness: SystemReadiness;
}

export interface RendererSelectionEvent {
  packId: Ulid;
  articleKey: string;
  selector?: SelectorBundle;
}
```

| Event name | Payload |
|---|---|
| `enzime://job/progress` | `EventEnvelope<JobProgressEvent>` |
| `enzime://job/completed` | `EventEnvelope<JobProgressEvent>` |
| `enzime://job/failed` | `EventEnvelope<JobProgressEvent>` |
| `enzime://ask/delta` | `EventEnvelope<AskDeltaEvent>` |
| `enzime://ask/completed` | `EventEnvelope<AskCompletedEvent>` |
| `enzime://ask/failed` | `EventEnvelope<AppErrorDto>` |
| `enzime://entity/changed` | `EventEnvelope<EntityChangedEvent>` |
| `enzime://readiness/changed` | `EventEnvelope<ReadinessChangedEvent>` |
| `enzime://renderer/selection` | `EventEnvelope<RendererSelectionEvent>` |

Event payloads are notifications, not a second source of truth.
Stores re-query the relevant read model after an entity-changed event.

---

# 15. Frontend Architecture

## 15.1 Component tree

```text
App
├── ErrorBoundary
├── ThemeProvider
├── IpcProvider
├── EventBridge
└── AppRouter
    ├── SetupRoute
    │   └── SetupWizard
    │       ├── CapabilityReport
    │       ├── SetupPathChooser
    │       └── ModelRecommendation
    └── AppShell
        ├── NavigationRail
        ├── GlobalSearchButton
        ├── JobStrip
        ├── ReadinessIndicator
        ├── RouteOutlet
        │   ├── HomeView
        │   │   ├── ContinueReadingCard
        │   │   ├── RecentPacksList
        │   │   │   └── PackCard
        │   │   ├── TaskInbox
        │   │   ├── MemoryInboxCard
        │   │   └── ReadinessPanel
        │   ├── LibraryView
        │   │   ├── LibraryToolbar
        │   │   ├── PackGrid
        │   │   │   └── PackCard
        │   │   ├── CatalogBrowser
        │   │   │   └── CatalogEntryCard
        │   │   ├── DownloadQueue
        │   │   │   └── DownloadJobRow
        │   │   ├── StoragePlanner
        │   │   └── DownloadPlanDialog
        │   │       ├── InstallPlanSummary
        │   │       └── StorageImpactBar
        │   ├── ReaderView
        │   │   ├── BreadcrumbNav
        │   │   ├── ArticleToolbar
        │   │   ├── ReaderPane
        │   │   │   └── ArticleContent
        │   │   ├── AnnotationWorkspace
        │   │   │   ├── AnnotationPanel
        │   │   │   ├── TextHighlighter
        │   │   │   ├── InkCanvas
        │   │   │   ├── VoiceRecorder
        │   │   │   └── SidecarImportDialog
        │   │   └── ReaderCompanionDrawer
        │   ├── AskView
        │   │   ├── ConversationSidebar
        │   │   ├── ScopeSelector
        │   │   ├── BackendStatusBadge
        │   │   ├── ConversationThread
        │   │   │   └── MessageBubble
        │   │   │       └── CitationChip
        │   │   └── Composer
        │   ├── NotesView
        │   │   ├── NotebookTree
        │   │   ├── NotesToolbar
        │   │   │   ├── TagFilter
        │   │   │   └── SidecarLayerToggle
        │   │   ├── AnnotationList
        │   │   │   └── AnnotationListItem
        │   │   └── AnnotationDetailPanel
        │   ├── MemoryView
        │   │   ├── MemoryInbox
        │   │   ├── MemoryWorkspace
        │   │   │   ├── EntityGraph
        │   │   │   └── EpisodeTimeline
        │   │   └── MemoryDetailPanel
        │   └── SettingsView
        │       ├── NetworkPolicyPanel
        │       ├── ComputePanel
        │       ├── AccessibilityPanel
        │       ├── EntitlementPanel
        │       └── BackupRestorePanel
        ├── ToastRegion
        ├── ConfirmDialogHost
        └── CommandPalette
```

## 15.2 Exact props

```ts
export interface AppShellProps {
  bootstrap: AppBootstrap;
}

export interface NavigationRailProps {
  routes: RouteDescriptor[];
  currentPath: RoutePath;
  onNavigate: (path: RoutePath) => void;
}

export interface JobStripProps {
  jobs: DownloadJob[];
  onOpenJob: (jobId: Ulid) => void;
}

export interface ReadinessIndicatorProps {
  readiness: SystemReadiness;
  onOpen: () => void;
}

export interface HomeViewProps {
  snapshot: HomeSnapshot;
  onContinueReading: (item: ReaderHistoryItem) => void;
  onOpenPack: (packId: Ulid) => void;
  onOpenTask: (taskId: Ulid) => void;
  onOpenMemoryInbox: () => void;
}

export interface ContinueReadingCardProps {
  item?: ReaderHistoryItem;
  onOpen: (item: ReaderHistoryItem) => void;
}

export interface RecentPacksListProps {
  packs: PackManifest[];
  onOpenPack: (packId: Ulid) => void;
}

export interface TaskInboxProps {
  tasks: UserTask[];
  onOpenTask: (taskId: Ulid) => void;
  onDismissTask: (taskId: Ulid) => void;
}

export interface ReadinessPanelProps {
  readiness: SystemReadiness;
  onAction: (action: ErrorActionDto["action"]) => void;
}

export interface LibraryViewProps {
  packs: PackManifest[];
  catalogEntries: CatalogEntry[];
  downloads: DownloadJob[];
  storageRoots: StorageDestination[];
  onSearchCatalog: (request: CatalogSearchRequest) => void;
  onPlanInstall: (entryId: Ulid, storageRootId: Ulid) => void;
  onImport: () => void;
  onOpenPack: (packId: Ulid) => void;
}

export interface PackGridProps {
  packs: PackManifest[];
  viewMode: "grid" | "list";
  onOpen: (packId: Ulid) => void;
  onMenu: (packId: Ulid, anchor: HTMLElement) => void;
}

export interface PackCardProps {
  pack: PackManifest;
  location?: PackLocationSummary;
  onOpen: (packId: Ulid) => void;
  onMenu: (packId: Ulid, anchor: HTMLElement) => void;
}

export interface CatalogBrowserProps {
  entries: CatalogEntry[];
  loading: boolean;
  query: CatalogSearchRequest;
  onQueryChange: (query: CatalogSearchRequest) => void;
  onPlanInstall: (catalogEntryId: Ulid) => void;
}

export interface CatalogEntryCardProps {
  entry: CatalogEntry;
  onPlanInstall: (catalogEntryId: Ulid) => void;
}

export interface DownloadQueueProps {
  jobs: DownloadJob[];
  onPause: (jobId: Ulid) => void;
  onResume: (jobId: Ulid) => void;
  onCancel: (jobId: Ulid) => void;
}

export interface DownloadJobRowProps {
  job: DownloadJob;
  onPause: () => void;
  onResume: () => void;
  onCancel: () => void;
}

export interface StoragePlannerProps {
  roots: StorageDestination[];
  packs: PackManifest[];
  selectedRootId?: Ulid;
  onSelectRoot: (storageRootId: Ulid) => void;
}

export interface DownloadPlanDialogProps {
  open: boolean;
  plan?: InstallPlan;
  busy: boolean;
  onConfirm: (installPlanId: Ulid) => void;
  onCancel: () => void;
}

export interface InstallPlanSummaryProps {
  plan: InstallPlan;
}

export interface StorageImpactBarProps {
  freeBytesBefore: number;
  temporaryBytes: number;
  finalBytes: number;
  freeBytesAfter: number;
}

export interface ReaderViewProps {
  article: ArticleResponse;
  annotationWorkspaceOpen: boolean;
  onNavigateArticle: (articleKey: string) => void;
  onToggleBookmark: () => void;
  onOpenAnnotations: () => void;
  onAskAboutArticle: () => void;
}

export interface BreadcrumbNavProps {
  items: BreadcrumbItem[];
  onNavigate: (articleKey: string) => void;
}

export interface ArticleToolbarProps {
  bookmarked: boolean;
  annotationCount: number;
  onBack: () => void;
  onForward: () => void;
  onToggleBookmark: () => void;
  onAnnotate: () => void;
  onAsk: () => void;
}

export interface ArticleContentProps {
  article: ArticleResponse;
  fontScale: number;
  readerTypeface: AccessibilitySettings["readerTypeface"];
  lineHeight: number;
  onRendererMessage: (message: RendererMessageV1) => void;
}

export interface AnnotationWorkspaceProps {
  article: ArticleResponse;
  selection?: SelectorBundle;
  annotations: AnnotationRecord[];
  activeTool: AnnotationTool;
  onToolChange: (tool: AnnotationTool) => void;
  onCreate: (draft: AnnotationDraft) => void;
  onUpdate: (
    annotationId: Ulid,
    expectedRevision: number,
    draft: AnnotationDraft
  ) => void;
  onClose: () => void;
}

export type AnnotationTool =
  | "select"
  | "highlight"
  | "comment"
  | "ink"
  | "voice";

export interface AnnotationPanelProps {
  annotations: AnnotationRecord[];
  selectedAnnotationId?: Ulid;
  onSelect: (annotationId: Ulid) => void;
  onDelete: (annotationId: Ulid, revision: number) => void;
}

export interface TextHighlighterProps {
  selection?: SelectorBundle;
  colors: string[];
  selectedColor: string;
  onColorChange: (colorToken: string) => void;
  onCreate: () => void;
}

export interface InkCanvasProps {
  enabled: boolean;
  colorToken: string;
  strokeWidth: number;
  onCommitSvg: (svg: string) => void;
  onCancel: () => void;
}

export interface VoiceRecorderProps {
  state: "idle" | "recording" | "review" | "uploading";
  elapsedMs: number;
  playbackUrl?: string;
  onStart: () => void;
  onStop: () => void;
  onCommit: () => void;
  onDiscard: () => void;
}

export interface SidecarImportDialogProps {
  open: boolean;
  plan?: SidecarImportPlan;
  onChooseFile: () => void;
  onImport: (acceptUnsigned: boolean, layerName: string) => void;
  onCancel: () => void;
}

export interface AskViewProps {
  conversations: ConversationSummary[];
  activeConversation?: ConversationDetail;
  scope: RetrievalScope;
  backendStatus: BackendStatus[];
  streamingRunId?: Ulid;
  onCreateConversation: (title: string, scope: RetrievalScope) => void;
  onSelectConversation: (conversationId: Ulid) => void;
  onScopeChange: (scope: RetrievalScope) => void;
  onSend: (message: string) => void;
  onCancel: () => void;
}

export interface ScopeSelectorProps {
  value: RetrievalScope;
  availablePacks: PackManifest[];
  availableNotebooks: NotebookNode[];
  onChange: (scope: RetrievalScope) => void;
}

export interface BackendStatusBadgeProps {
  status?: BackendStatus;
  onOpenSettings: () => void;
}

export interface ConversationThreadProps {
  messages: ConversationMessage[];
  streamingText?: string;
  onOpenCitation: (messageId: Ulid, citationIndex: number) => void;
}

export interface MessageBubbleProps {
  message: ConversationMessage;
  onOpenCitation: (citationIndex: number) => void;
}

export interface CitationChipProps {
  citation: CitationRecord;
  onOpen: () => void;
}

export interface ComposerProps {
  disabled: boolean;
  placeholder: string;
  onSend: (text: string) => void;
  onCancel?: () => void;
}

export interface NotesViewProps {
  annotations: AnnotationRecord[];
  notebooks: NotebookNode[];
  tags: TagSummary[];
  layers: SidecarLayer[];
  filter: AnnotationListRequest;
  selectedAnnotationId?: Ulid;
  onFilterChange: (filter: AnnotationListRequest) => void;
  onSelect: (annotationId: Ulid) => void;
  onOpenSource: (annotationId: Ulid) => void;
}

export interface AnnotationListProps {
  annotations: AnnotationRecord[];
  selectedAnnotationId?: Ulid;
  onSelect: (annotationId: Ulid) => void;
}

export interface NotebookTreeProps {
  nodes: NotebookNode[];
  selectedNotebookIds: Ulid[];
  onSelectionChange: (notebookIds: Ulid[]) => void;
  onCreate: (parentNotebookId?: Ulid) => void;
}

export interface TagFilterProps {
  tags: TagSummary[];
  selectedTagIds: Ulid[];
  onChange: (tagIds: Ulid[]) => void;
}

export interface SidecarLayerToggleProps {
  layers: SidecarLayer[];
  selectedLayerIds: Ulid[];
  onChange: (layerIds: Ulid[]) => void;
}

export interface MemoryViewProps {
  candidates: LoreCandidate[];
  memories: LoreMemory[];
  selectedMemoryId?: Ulid;
  onAccept: (candidateId: Ulid) => void;
  onReject: (candidateId: Ulid) => void;
  onSelectMemory: (memoryId: Ulid) => void;
  onCorrect: (request: LoreCorrectRequest) => void;
  onPin: (memoryId: Ulid, pinned: boolean) => void;
  onDelete: (memoryId: Ulid, revision: number) => void;
}

export interface MemoryInboxProps {
  candidates: LoreCandidate[];
  onAccept: (candidateId: Ulid) => void;
  onReject: (candidateId: Ulid) => void;
}

export interface EntityGraphProps {
  nodes: EntityGraphNode[];
  edges: EntityGraphEdge[];
  selectedNodeId?: string;
  onSelectNode: (nodeId: string) => void;
}

export interface EntityGraphNode {
  nodeId: string;
  label: string;
  kind: "subject" | "memory" | "evidence";
}

export interface EntityGraphEdge {
  edgeId: string;
  sourceNodeId: string;
  targetNodeId: string;
  label: string;
}

export interface EpisodeTimelineProps {
  memories: LoreMemory[];
  onSelectMemory: (memoryId: Ulid) => void;
}

export interface MemoryDetailPanelProps {
  memory?: LoreMemory;
  onCorrect: (request: LoreCorrectRequest) => void;
  onPin: (pinned: boolean) => void;
  onDelete: () => void;
}

export interface SettingsViewProps {
  settings: SettingsSnapshot;
  backends: BackendStatus[];
  models: ModelSummary[];
  onUpdate: (settings: SettingsSnapshot) => void;
  onRefreshEntitlement: () => void;
  onRestoreEntitlement: () => void;
  onCreateBackup: () => void;
  onRestoreBackup: () => void;
}

export interface ComputePanelProps {
  policy: ComputePolicy;
  backends: BackendStatus[];
  models: ModelSummary[];
  onChange: (policy: ComputePolicy) => void;
  onInstallModel: (modelId: string) => void;
  onRemoveModel: (modelId: string) => void;
}

export interface AccessibilityPanelProps {
  settings: AccessibilitySettings;
  onChange: (settings: AccessibilitySettings) => void;
}

export interface EntitlementPanelProps {
  decisions: CapabilityDecision[];
  onRefresh: () => void;
  onRestore: () => void;
  onRedeemCoupon: () => void;
}

export interface BackupRestorePanelProps {
  settings: BackupSettings;
  onChange: (settings: BackupSettings) => void;
  onCreateBackup: () => void;
  onRestoreBackup: () => void;
}

export interface SetupWizardProps {
  report: CapabilityReport;
  step: "capabilities" | "path" | "model" | "complete";
  onPathSelect: (path: SetupPath) => void;
  onModelSelect: (modelId?: string) => void;
  onComplete: () => void;
}

export type SetupPath =
  | "existing_files"
  | "choose_packs"
  | "explore_without_ai";

export interface CapabilityReportProps {
  report: CapabilityReport;
}

export interface SetupPathChooserProps {
  value?: SetupPath;
  onChange: (path: SetupPath) => void;
}

export interface ModelRecommendationProps {
  recommended?: ModelSummary;
  alternatives: ModelSummary[];
  onSelect: (modelId?: string) => void;
}
```

## 15.3 Route table

| Path | Route component | Loader commands | Mobile nav |
|---|---|---|---|
| `/setup` | `SetupRoute` | `capability_probe`, `model_list` | launch-only |
| `/` | `HomeView` | `home_snapshot` | Home |
| `/library` | `LibraryView` | `pack_list`, `job_list`, `storage_roots_list` | Library |
| `/reader/:packId/:articleKey` | `ReaderView` | `reader_article`, `sidecar_layers` | contextual |
| `/ask` | `AskView` | `conversation_list`, `backend_status` | Ask |
| `/ask/:conversationId` | `AskView` | `conversation_get`, `backend_status` | Ask |
| `/notes` | `NotesView` | `annotation_list`, `notebook_tree`, `tag_list`, `sidecar_layers` | Notes |
| `/memory` | `MemoryView` | `lore_inbox`, `lore_search` | Memory |
| `/settings` | `SettingsView` | `settings_get`, `backend_status`, `model_list` | Settings |

`DownloadPlanDialog` is route-preserving modal state.
`AnnotationWorkspace` is a reader substate.

## 15.4 Zustand store boundaries

Stores contain serializable UI/read-model state.
No store imports Tauri directly.
All IPC goes through `ipcClient`.

```ts
export interface AppStore {
  bootstrap?: AppBootstrap;
  bootState: "idle" | "loading" | "ready" | "failed";
  setBootstrap: (bootstrap: AppBootstrap) => void;
}

export interface HomeStore {
  snapshot?: HomeSnapshot;
  loading: boolean;
  load: () => Promise<void>;
}

export interface LibraryStore {
  packs: PackManifest[];
  catalogEntries: CatalogEntry[];
  downloads: DownloadJob[];
  storageRoots: StorageDestination[];
  installPlan?: InstallPlan;
  catalogQuery: CatalogSearchRequest;
  loadLibrary: () => Promise<void>;
  searchCatalog: (query: CatalogSearchRequest) => Promise<void>;
  planInstall: (entryId: Ulid, rootId: Ulid) => Promise<void>;
  startInstall: (installPlanId: Ulid) => Promise<void>;
}

export interface ReaderStore {
  currentArticle?: ArticleResponse;
  history: ReaderHistoryItem[];
  selection?: SelectorBundle;
  annotationWorkspaceOpen: boolean;
  activeTool: AnnotationTool;
  openArticle: (packId: Ulid, articleKey?: string) => Promise<void>;
  handleRendererMessage: (message: RendererMessageV1) => Promise<void>;
}

export interface AnnotationStore {
  annotations: AnnotationRecord[];
  notebooks: NotebookNode[];
  tags: TagSummary[];
  layers: SidecarLayer[];
  filter: AnnotationListRequest;
  selectedAnnotationId?: Ulid;
  load: () => Promise<void>;
  create: (draft: AnnotationDraft) => Promise<void>;
  update: (
    annotationId: Ulid,
    expectedRevision: number,
    draft: AnnotationDraft
  ) => Promise<void>;
}

export interface AskStore {
  conversations: ConversationSummary[];
  activeConversation?: ConversationDetail;
  scope: RetrievalScope;
  backendStatus: BackendStatus[];
  streamingRunId?: Ulid;
  streamingText: string;
  createConversation: (
    title: string,
    scope: RetrievalScope
  ) => Promise<void>;
  send: (text: string) => Promise<void>;
  cancel: () => Promise<void>;
}

export interface MemoryStore {
  candidates: LoreCandidate[];
  memories: LoreMemory[];
  selectedMemoryId?: Ulid;
  load: () => Promise<void>;
  accept: (candidateId: Ulid) => Promise<void>;
  reject: (candidateId: Ulid) => Promise<void>;
  correct: (request: LoreCorrectRequest) => Promise<void>;
}

export interface SettingsStore {
  settings?: SettingsSnapshot;
  backends: BackendStatus[];
  models: ModelSummary[];
  load: () => Promise<void>;
  update: (settings: SettingsSnapshot) => Promise<void>;
}

export interface TaskStore {
  tasks: UserTask[];
  resolve: (
    taskId: Ulid,
    resolution: "resolved" | "dismissed"
  ) => Promise<void>;
}
```

Store modules:

```text
stores/appStore.ts
stores/homeStore.ts
stores/libraryStore.ts
stores/readerStore.ts
stores/annotationStore.ts
stores/askStore.ts
stores/memoryStore.ts
stores/settingsStore.ts
stores/taskStore.ts
```

## 15.5 IPC client

```ts
export interface IpcClient {
  invoke<TRequest, TResponse>(
    command: TauriCommandName,
    request: TRequest
  ): Promise<TResponse>;
  listen<TPayload>(
    event: TauriEventName,
    handler: (payload: EventEnvelope<TPayload>) => void
  ): Promise<() => void>;
}
```

`TauriCommandName` is generated from the registry in §14.4.
`TauriEventName` is generated from §14.6.

## 15.6 Aurora Glass design system

```css
:root {
  --ez-color-bg-canvas: #090b13;
  --ez-color-bg-elevated: #111521;
  --ez-color-bg-glass: rgba(23, 28, 44, 0.72);
  --ez-color-bg-glass-strong: rgba(28, 34, 54, 0.9);
  --ez-color-border-subtle: rgba(189, 198, 255, 0.14);
  --ez-color-border-focus: #7c8cff;
  --ez-color-text-primary: #f4f6ff;
  --ez-color-text-secondary: #b7bdd4;
  --ez-color-text-muted: #7f879f;
  --ez-color-primary: #7c8cff;
  --ez-color-primary-hover: #91a0ff;
  --ez-color-accent: #34e0d4;
  --ez-color-success: #4ce3a4;
  --ez-color-warning: #ffca6a;
  --ez-color-danger: #ff6f91;
  --ez-color-highlight-yellow: #ffe27a;
  --ez-color-highlight-mint: #79f2c0;
  --ez-color-highlight-pink: #ff91c8;
  --ez-color-highlight-blue: #82c7ff;
  --ez-font-ui: "Inter", system-ui, sans-serif;
  --ez-font-mono: "JetBrains Mono", ui-monospace, monospace;
  --ez-font-reader: "Literata", Georgia, serif;
  --ez-radius-sm: 8px;
  --ez-radius-md: 14px;
  --ez-radius-lg: 22px;
  --ez-space-1: 4px;
  --ez-space-2: 8px;
  --ez-space-3: 12px;
  --ez-space-4: 16px;
  --ez-space-5: 20px;
  --ez-space-6: 24px;
  --ez-space-8: 32px;
  --ez-space-10: 40px;
  --ez-shadow-panel: 0 16px 48px rgba(0, 0, 0, 0.32);
  --ez-blur-glass: 18px;
  --ez-duration-fast: 120ms;
  --ez-duration-normal: 200ms;
  --ez-focus-ring: 0 0 0 3px rgba(124, 140, 255, 0.45);
}
```

Design rules:

- glass is reserved for chrome and overlays, not long-form article surfaces;
- article content uses a quiet opaque surface;
- verification and availability include icon plus text;
- periwinkle is primary action/navigation;
- teal is verified/live/connected state;
- annotation colours never encode trust state;
- minimum text contrast is WCAG 2.2 AA;
- motion is removed when `prefers-reduced-motion` or setting is active;
- `Inter` is UI text;
- `JetBrains Mono` is telemetry, IDs and sizes;
- reader typeface is user-selectable.

---

# 16. Entity Table

This section pre-commits public identifiers.
Case and spelling are normative.

## 16.1 Rust crates and modules

| Identifier | Kind | Owner |
|---|---|---|
| `enzime-domain` | crate | domain entities |
| `enzime-ports` | crate | replaceable engine ports |
| `enzime-application` | crate | use cases |
| `enzime-storage-sqlite` | crate | SQLite repositories |
| `enzime-object-store` | crate | immutable objects/staging |
| `enzime-zim-anzimmerman` | crate | ZIM adapter |
| `enzime-catalog-openzim` | crate | catalog adapter |
| `enzime-transfer-http` | crate | HTTP transfer |
| `enzime-transfer-file` | crate | file/SAF transfer |
| `enzime-annotations` | crate | annotation service |
| `enzime-sidecar` | crate | CBOR/signature |
| `enzime-inference-wllama` | crate | wllama3 inference (all platforms) |
| `enzime-retrieval` | crate | retrieval/citations |
| `enzime-lore` | crate | memory |
| `enzime-entitlement` | crate | offline policy |
| `enzime-revenuecat-adapter` | crate | purchase events |
| `enzime-platform-adapter` | crate | platform adapters (Android/Linux/Windows `#[cfg]` modules) |
| `enzime-backup` | crate | backup/restore |
| `enzime-diagnostics` | crate | redacted diagnostics |
| `domain::identity` | module | stable IDs |
| `domain::pack` | module | pack entities |
| `domain::reader` | module | article/read models |
| `domain::annotation` | module | selectors/records |
| `domain::conversation` | module | messages/citations |
| `domain::lore` | module | candidates/memory |
| `domain::entitlement` | module | capability decisions |
| `domain::jobs` | module | durable state |
| `ports::zim` | module | ZIM traits |
| `ports::catalog` | module | catalog trait |
| `ports::transfer` | module | transfer trait |
| `ports::inference` | module | inference traits |
| `ports::embedding` | module | embedding trait |
| `ports::signature` | module | signing |
| `ports::entitlement` | module | provider boundary |
| `ports::platform` | module | native access |
| `application::home` | module | dashboard snapshot |
| `application::library` | module | packs/catalog/jobs |
| `application::reader` | module | reader |
| `application::annotations` | module | notes |
| `application::ask` | module | conversations |
| `application::memory` | module | lore |
| `application::settings` | module | settings/readiness |
| `application::backup` | module | backup/restore |

## 16.2 Rust public structs and enums

| Identifier | Kind | Defined in |
|---|---|---|
| `PackId` | struct | `domain::identity` |
| `PackVersionId` | struct | `domain::identity` |
| `ArticleKey` | struct | `domain::identity` |
| `EntryKey` | struct | `domain::identity` |
| `ContentDigest` | struct | `domain::identity` |
| `DigestAlgorithm` | enum | `domain::identity` |
| `ZimOpenRequest` | struct | `ports::zim` |
| `ZimMetadata` | struct | `ports::zim` |
| `ZimEntryDescriptor` | struct | `ports::zim` |
| `ZimEntryKind` | enum | `ports::zim` |
| `ArticleDocument` | struct | `ports::zim` |
| `InternalLink` | struct | `ports::zim` |
| `InternalLinkTarget` | enum | `ports::zim` |
| `ResourceDocument` | struct | `ports::zim` |
| `TitleSearchQuery` | struct | `ports::zim` |
| `TitleSearchHit` | struct | `ports::zim` |
| `ZimVerificationReport` | struct | `ports::zim` |
| `ZimVerificationFinding` | struct | `ports::zim` |
| `ZimError` | enum | `ports::zim` |
| `PackManifest` | struct | `domain::pack` |
| `PackKind` | enum | `domain::pack` |
| `PackVersionSummary` | struct | `domain::pack` |
| `VerificationState` | enum | `domain::pack` |
| `CatalogQuery` | struct | `ports::catalog` |
| `CatalogPage` | struct | `ports::catalog` |
| `CatalogEntry` | struct | `ports::catalog` |
| `TrustPresentation` | struct | `domain::trust` |
| `CatalogRefreshReport` | struct | `ports::catalog` |
| `CatalogError` | enum | `ports::catalog` |
| `TransferProbeRequest` | struct | `ports::transfer` |
| `TransferProbe` | struct | `ports::transfer` |
| `TransferRange` | struct | `ports::transfer` |
| `TransferError` | enum | `ports::transfer` |
| `InstallPlan` | struct | `application::library` |
| `StorageDestination` | struct | `domain::storage` |
| `TransferMode` | enum | `domain::jobs` |
| `VerificationPlan` | struct | `domain::jobs` |
| `SelectorBundle` | struct | `domain::annotation` |
| `ArticleSelector` | struct | `domain::annotation` |
| `TextQuoteSelector` | struct | `domain::annotation` |
| `TextPositionSelector` | struct | `domain::annotation` |
| `DomPathSelector` | struct | `domain::annotation` |
| `ImageRegionSelector` | struct | `domain::annotation` |
| `AnnotationType` | enum | `domain::annotation` |
| `AnnotationDraft` | struct | `domain::annotation` |
| `AnnotationStyle` | struct | `domain::annotation` |
| `AnnotationRecord` | struct | `domain::annotation` |
| `TagSummary` | struct | `domain::annotation` |
| `AnnotationResolution` | struct | `domain::annotation` |
| `AnnotationResolutionState` | enum | `domain::annotation` |
| `AnnotationAssetInput` | struct | `application::annotations` |
| `AnnotationAssetRecord` | struct | `domain::annotation` |
| `AnnotationError` | enum | `application::annotations` |
| `SignatureEnvelope` | struct | `domain::trust` |
| `SignatureVerification` | struct | `domain::trust` |
| `SignatureVerificationState` | enum | `domain::trust` |
| `SignatureError` | enum | `ports::signature` |
| `BackendDescriptor` | struct | `ports::inference` |
| `BackendKind` | enum | `ports::inference` |
| `ModelFormat` | enum | `ports::inference` |
| `AcceleratorKind` | enum | `ports::inference` |
| `BackendHealth` | struct | `ports::inference` |
| `ModelManifest` | struct | `ports::inference` |
| `ModelKind` | enum | `ports::inference` |
| `SessionOptions` | struct | `ports::inference` |
| `InferenceRequest` | struct | `ports::inference` |
| `InferenceMessage` | struct | `ports::inference` |
| `InferenceEvidence` | struct | `ports::inference` |
| `InferenceChunk` | enum | `ports::inference` |
| `InferenceError` | enum | `ports::inference` |
| `EmbeddingDescriptor` | struct | `ports::embedding` |
| `EmbeddingInput` | struct | `ports::embedding` |
| `EmbeddingOutput` | struct | `ports::embedding` |
| `BackendSelectionInput` | struct | `application::ask` |
| `ModelInstallationSummary` | struct | `domain::model` |
| `BackendSelection` | struct | `application::ask` |
| `RetrievalQuery` | struct | `domain::retrieval` |
| `RetrievalScope` | enum | `domain::retrieval` |
| `RetrievalHit` | struct | `domain::retrieval` |
| `CitationLocator` | struct | `domain::retrieval` |
| `ContextEvidence` | struct | `domain::retrieval` |
| `ContextSet` | struct | `domain::retrieval` |
| `RetrievalError` | enum | `application::ask` |
| `LoreCandidate` | struct | `domain::lore` |
| `LoreEvidence` | struct | `domain::lore` |
| `LoreMemory` | struct | `domain::lore` |
| `LoreMemoryStatus` | enum | `domain::lore` |
| `LoreCorrection` | struct | `domain::lore` |
| `LoreQuery` | struct | `domain::lore` |
| `LoreError` | enum | `application::memory` |
| `ProviderEntitlementEvent` | struct | `ports::entitlement` |
| `EntitlementRefreshRequest` | struct | `ports::entitlement` |
| `EntitlementRestoreRequest` | struct | `ports::entitlement` |
| `PlatformError` | enum | `ports::platform` |
| `PlatformCapabilityReport` | struct | `ports::platform` |
| `PlatformStorageRoot` | struct | `ports::platform` |
| `StorageRootKind` | enum | `ports::platform` |
| `PowerState` | struct | `ports::platform` |
| `ThermalLoad` | enum | `ports::platform` |
| `NetworkState` | enum | `ports::platform` |
| `CapabilityDecision` | struct | `domain::entitlement` |
| `CapabilityDecisionSource` | enum | `domain::entitlement` |
| `EntitlementError` | enum | `application::entitlement` |

## 16.3 Rust traits and release implementations

| Trait | Release implementation |
|---|---|
| `ZimEngine` | `AnZimmermanZimEngine` |
| `ZimArchive` | `AnZimmermanZimArchive` |
| `CatalogSource` | `OpenZimXmlCatalogSource` |
| `CatalogSource` | `EnzimeJsonCatalogSource` |
| `CatalogSource` | `LocalManifestCatalogSource` |
| `TransferSource` | `HttpTransferSource` |
| `TransferSource` | `LocalFileTransferSource` |
| `AnnotationResolver` | `MultiSelectorAnnotationResolver` |
| `AnnotationService` | `SqliteAnnotationService` |
| `SignatureProvider` | `OsKeystoreEd25519SignatureProvider` |
| `InferenceBackend` | `WllamaInferenceBackend` |
| `InferenceSession` | `WllamaInferenceSession` |
| `EmbeddingBackend` | `WllamaEmbeddingBackend` |
| `Retriever` | `HybridLocalRetriever` |
| `ContextBuilder` | `BudgetedContextBuilder` |
| `LoreService` | `SqliteLoreService` |
| `EntitlementSource` | `RevenueCatEntitlementSource` |
| `CapabilityPolicy` | `SignedArtifactCapabilityPolicy` |

## 16.4 SQL entities

| Identifier | Kind |
|---|---|
| `schema_migrations` | table |
| `app_meta` | table |
| `devices` | table |
| `settings` | table |
| `storage_roots` | table |
| `capability_reports` | table |
| `catalog_sources` | table |
| `catalog_entries` | table |
| `packs` | table |
| `pack_versions` | table |
| `pack_locations` | table |
| `selection_profiles` | table |
| `selection_profile_revisions` | table |
| `download_jobs` | table |
| `download_chunks` | table |
| `install_transactions` | table |
| `reader_positions` | table |
| `reader_history` | table |
| `bookmarks` | table |
| `notebooks` | table |
| `tags` | table |
| `sidecar_layers` | table |
| `annotations` | table |
| `annotation_revisions` | table |
| `annotation_assets` | table |
| `annotation_tags` | table |
| `sidecar_imports` | table |
| `conversations` | table |
| `messages` | table |
| `retrieval_documents` | table |
| `retrieval_chunks` | table |
| `retrieval_chunks_fts` | virtual table |
| `embedding_spaces` | table |
| `chunk_embeddings` | table |
| `inference_backends` | table |
| `models` | table |
| `model_installations` | table |
| `inference_runs` | table |
| `message_citations` | table |
| `lore_evidence` | table |
| `lore_candidates` | table |
| `lore_candidate_evidence` | table |
| `lore_memories` | table |
| `lore_memory_revisions` | table |
| `lore_memory_evidence` | table |
| `trust_keys` | table |
| `entitlement_artifacts` | table |
| `entitlement_grants` | table |
| `coupon_ledger` | table |
| `backups` | table |
| `user_tasks` | table |
| `audit_events` | table |
| `trg_pack_active_version_same_pack` | trigger |
| `trg_pack_active_version_insert` | trigger |
| `trg_annotation_revision_insert` | trigger |
| `trg_lore_revision_insert` | trigger |
| `trg_retrieval_chunk_fts_insert` | trigger |
| `trg_retrieval_chunk_fts_delete` | trigger |
| `trg_download_state_guard` | trigger |
| `trg_install_state_guard` | trigger |

All index identifiers are exactly those in §6.2.

## 16.5 TypeScript public entities

The public TypeScript entities are exactly the exported declarations in
§§7.3, 14.1–14.6 and 15.2–15.5.
They are generated into:

```text
packages/contracts/src/generated/entities.ts
packages/contracts/src/generated/commands.ts
packages/contracts/src/generated/events.ts
packages/contracts/src/generated/routes.ts
```

Handwritten aliases may re-export but may not rename them.

## 16.6 React components

| Identifier | File |
|---|---|
| `App` | `src/App.tsx` |
| `AppShell` | `src/components/shell/AppShell.tsx` |
| `NavigationRail` | `src/components/shell/NavigationRail.tsx` |
| `JobStrip` | `src/components/shell/JobStrip.tsx` |
| `ReadinessIndicator` | `src/components/shell/ReadinessIndicator.tsx` |
| `HomeView` | `src/routes/home/HomeView.tsx` |
| `ContinueReadingCard` | `src/routes/home/ContinueReadingCard.tsx` |
| `RecentPacksList` | `src/routes/home/RecentPacksList.tsx` |
| `TaskInbox` | `src/routes/home/TaskInbox.tsx` |
| `MemoryInboxCard` | `src/routes/home/MemoryInboxCard.tsx` |
| `ReadinessPanel` | `src/routes/home/ReadinessPanel.tsx` |
| `LibraryView` | `src/routes/library/LibraryView.tsx` |
| `LibraryToolbar` | `src/routes/library/LibraryToolbar.tsx` |
| `PackGrid` | `src/routes/library/PackGrid.tsx` |
| `PackCard` | `src/routes/library/PackCard.tsx` |
| `CatalogBrowser` | `src/routes/library/CatalogBrowser.tsx` |
| `CatalogEntryCard` | `src/routes/library/CatalogEntryCard.tsx` |
| `DownloadQueue` | `src/routes/library/DownloadQueue.tsx` |
| `DownloadJobRow` | `src/routes/library/DownloadJobRow.tsx` |
| `StoragePlanner` | `src/routes/library/StoragePlanner.tsx` |
| `DownloadPlanDialog` | `src/routes/library/DownloadPlanDialog.tsx` |
| `InstallPlanSummary` | `src/routes/library/InstallPlanSummary.tsx` |
| `StorageImpactBar` | `src/routes/library/StorageImpactBar.tsx` |
| `ReaderView` | `src/routes/reader/ReaderView.tsx` |
| `BreadcrumbNav` | `src/routes/reader/BreadcrumbNav.tsx` |
| `ArticleToolbar` | `src/routes/reader/ArticleToolbar.tsx` |
| `ReaderPane` | `src/routes/reader/ReaderPane.tsx` |
| `ArticleContent` | `src/routes/reader/ArticleContent.tsx` |
| `AnnotationWorkspace` | `src/routes/reader/AnnotationWorkspace.tsx` |
| `AnnotationPanel` | `src/routes/reader/AnnotationPanel.tsx` |
| `TextHighlighter` | `src/routes/reader/TextHighlighter.tsx` |
| `InkCanvas` | `src/routes/reader/InkCanvas.tsx` |
| `VoiceRecorder` | `src/routes/reader/VoiceRecorder.tsx` |
| `SidecarImportDialog` | `src/routes/reader/SidecarImportDialog.tsx` |
| `ReaderCompanionDrawer` | `src/routes/reader/ReaderCompanionDrawer.tsx` |
| `AskView` | `src/routes/ask/AskView.tsx` |
| `ConversationSidebar` | `src/routes/ask/ConversationSidebar.tsx` |
| `ScopeSelector` | `src/routes/ask/ScopeSelector.tsx` |
| `BackendStatusBadge` | `src/routes/ask/BackendStatusBadge.tsx` |
| `ConversationThread` | `src/routes/ask/ConversationThread.tsx` |
| `MessageBubble` | `src/routes/ask/MessageBubble.tsx` |
| `CitationChip` | `src/routes/ask/CitationChip.tsx` |
| `Composer` | `src/routes/ask/Composer.tsx` |
| `NotesView` | `src/routes/notes/NotesView.tsx` |
| `NotebookTree` | `src/routes/notes/NotebookTree.tsx` |
| `NotesToolbar` | `src/routes/notes/NotesToolbar.tsx` |
| `TagFilter` | `src/routes/notes/TagFilter.tsx` |
| `SidecarLayerToggle` | `src/routes/notes/SidecarLayerToggle.tsx` |
| `AnnotationList` | `src/routes/notes/AnnotationList.tsx` |
| `AnnotationListItem` | `src/routes/notes/AnnotationListItem.tsx` |
| `AnnotationDetailPanel` | `src/routes/notes/AnnotationDetailPanel.tsx` |
| `MemoryView` | `src/routes/memory/MemoryView.tsx` |
| `MemoryInbox` | `src/routes/memory/MemoryInbox.tsx` |
| `MemoryWorkspace` | `src/routes/memory/MemoryWorkspace.tsx` |
| `EntityGraph` | `src/routes/memory/EntityGraph.tsx` |
| `EpisodeTimeline` | `src/routes/memory/EpisodeTimeline.tsx` |
| `MemoryDetailPanel` | `src/routes/memory/MemoryDetailPanel.tsx` |
| `SettingsView` | `src/routes/settings/SettingsView.tsx` |
| `NetworkPolicyPanel` | `src/routes/settings/NetworkPolicyPanel.tsx` |
| `ComputePanel` | `src/routes/settings/ComputePanel.tsx` |
| `AccessibilityPanel` | `src/routes/settings/AccessibilityPanel.tsx` |
| `EntitlementPanel` | `src/routes/settings/EntitlementPanel.tsx` |
| `BackupRestorePanel` | `src/routes/settings/BackupRestorePanel.tsx` |
| `SetupRoute` | `src/routes/setup/SetupRoute.tsx` |
| `SetupWizard` | `src/routes/setup/SetupWizard.tsx` |
| `CapabilityReport` | `src/routes/setup/CapabilityReport.tsx` |
| `SetupPathChooser` | `src/routes/setup/SetupPathChooser.tsx` |
| `ModelRecommendation` | `src/routes/setup/ModelRecommendation.tsx` |

## 16.7 Routes, storage keys and settings

| Identifier | Kind |
|---|---|
| `/` | route |
| `/library` | route |
| `/reader/:packId/:articleKey` | route |
| `/ask` | route |
| `/ask/:conversationId` | route |
| `/notes` | route |
| `/memory` | route |
| `/settings` | route |
| `/setup` | route |
| `enzime.ui.theme` | local-storage key |
| `enzime.ui.library.view_mode` | local-storage key |
| `enzime.ui.reader.panel_width` | local-storage key |
| `enzime.ui.notes.panel_width` | local-storage key |
| `enzime.ui.last_route` | local-storage key |
| `network.allow_internet_catalog_refresh` | setting key |
| `network.allow_internet_downloads` | setting key |
| `network.allow_metered_downloads` | setting key |
| `network.wifi_only_large_transfers` | setting key |
| `compute.preferred_backend_id` | setting key |
| `compute.preferred_chat_model_id` | setting key |
| `compute.preferred_embedding_model_id` | setting key |
| `compute.battery_saver_behavior` | setting key |
| `compute.thermal_behavior` | setting key |
| `accessibility.font_scale` | setting key |
| `accessibility.reduce_motion` | setting key |
| `accessibility.high_contrast` | setting key |
| `accessibility.reader_typeface` | setting key |
| `accessibility.line_height` | setting key |
| `backup.reminder_interval_days` | setting key |
| `backup.include_conversations` | setting key |
| `backup.include_lore` | setting key |
| `backup.encrypt_by_default` | setting key |
| `setup.completed` | app-meta key |
| `schema.current_version` | app-meta key |

## 16.8 CSS variables

The CSS identifiers are exactly those in §15.6.
Components may define private variables prefixed with their component name.
No new global variable may omit the `--ez-` prefix.

---

# 17. Platform Adapters

All platform adapters live in one crate, `crates/platform-adapter`, which
exposes the §17.2 `PlatformAdapter` port and selects per-target behaviour with
`#[cfg(target_os = "...")]` modules `android.rs`, `linux.rs` and `windows.rs`.
There are no separate per-platform adapter crates; the Android, Linux and
Windows adapter types below are the `#[cfg]` implementations inside that one
crate.

## 17.1 Concern matrix

| Concern | Android adapter | Linux adapter | Windows adapter |
|---|---|---|---|
| app-private root | `AndroidAppStorage` | `LinuxAppStorage` | `WindowsAppStorage` |
| user file import | `AndroidSafFileAccess` | `LinuxNativeFileAccess` | `WindowsNativeFileAccess` |
| export destination | `AndroidSafExportAccess` | `LinuxNativeExportAccess` | `WindowsNativeExportAccess` |
| keystore | `AndroidKeystore` | `LinuxSecretServiceKeystore` | `WindowsCredentialKeystore` |
| audio capture | `AndroidAudioCapture` | `CpalAudioCapture` | `CpalAudioCapture` |
| background transfer | `AndroidForegroundTransfer` | `DesktopTransferTask` | `DesktopTransferTask` |
| power status | `AndroidPowerStatus` | `LinuxPowerStatus` | `WindowsPowerStatus` |
| thermal status | `AndroidThermalStatus` | `LinuxThermalStatus` | `WindowsThermalStatus` |
| network policy | `AndroidNetworkPolicy` | `DesktopNetworkPolicy` | `DesktopNetworkPolicy` |
| share receive | `AndroidShareReceiver` | `DesktopOpenWithReceiver` | `DesktopOpenWithReceiver` |

## 17.2 Platform port

`PlatformAdapter` is the single port through which the application observes the
device and obtains validated file grants. Its return types are **port-level
observations**, not command DTOs: `PlatformCapabilityReport` and
`PlatformStorageRoot` carry the raw facts the adapter measures, and the
application layer projects them down to the persisted, UI-facing DTOs
(`CapabilityReport`, `StorageDestination`) defined in §14. `PowerState` and
`NetworkState` carry exactly the signals the §8 transfer policy and the
§10.4 / §21 compute and thermal behaviour reason about — no more.

`open_read` and `open_write` return `tokio::io::AsyncRead` / `AsyncWrite`
handles. This is deliberate: the release async runtime is Tokio (Tauri 2), and
the platform file implementations (`tokio::fs::File`, the Android SAF bridge,
and the Linux / Windows native handles) produce Tokio async I/O directly. The
`futures_core::Stream` abstraction used by `ZimEntryStream` (§7.2) and
`TransferByteStream` (§8.2) is a different shape — a pull-based *item* stream —
and does not govern byte-handle I/O. The two therefore coexist by design:
`futures_core::Stream` for engine and transfer chunk streams,
`tokio::io::AsyncRead` / `AsyncWrite` for platform file handles. Consequently
`crates/ports` depends on `tokio` (with the `io-util` feature); this is
expected, not a surprise.

```rust
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::inference::AcceleratorKind;

#[derive(Clone, Debug, thiserror::Error)]
pub enum PlatformError {
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    #[error("capability unavailable: {0}")]
    CapabilityUnavailable(String),
    #[error("storage root unavailable: {0}")]
    StorageRootUnavailable(String),
    #[error("invalid or expired file grant: {0}")]
    InvalidGrant(String),
    #[error("cancelled by user")]
    Cancelled,
    #[error("unsupported on this platform: {0}")]
    UnsupportedOnPlatform(String),
    #[error("I/O error: {0}")]
    Io(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlatformCapabilityReport {
    pub cpu: String,
    pub ram_bytes: u64,
    pub free_storage_bytes: u64,
    pub accelerators: Vec<AcceleratorKind>,
    pub background_transfer_granted: bool,
    pub notifications_granted: bool,
    pub microphone_granted: bool,
    pub inference_viable: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageRootKind {
    AppPrivate,
    UserSelected,
    Removable,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlatformStorageRoot {
    pub storage_root_id: String,
    pub kind: StorageRootKind,
    pub platform_locator: String,
    pub display_name: String,
    pub writable: bool,
    pub capacity_bytes: Option<u64>,
    pub free_bytes: Option<u64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThermalLoad {
    Nominal,
    Moderate,
    Severe,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PowerState {
    pub on_charger: bool,
    pub battery_percent: Option<u8>,
    pub battery_saver: bool,
    pub thermal_load: ThermalLoad,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkState {
    Offline,
    Metered,
    Unmetered,
}

#[async_trait]
pub trait PlatformAdapter: Send + Sync {
    fn platform(&self) -> PlatformKind;
    async fn capability_report(&self) -> Result<PlatformCapabilityReport, PlatformError>;
    async fn storage_roots(&self) -> Result<Vec<PlatformStorageRoot>, PlatformError>;
    async fn choose_import_file(
        &self,
        accepted_media_types: &[String],
    ) -> Result<PlatformFileToken, PlatformError>;
    async fn choose_export_file(
        &self,
        suggested_name: &str,
        media_type: &str,
    ) -> Result<PlatformFileToken, PlatformError>;
    async fn open_read(
        &self,
        token: &PlatformFileToken,
    ) -> Result<Box<dyn tokio::io::AsyncRead + Unpin + Send>, PlatformError>;
    async fn open_write(
        &self,
        token: &PlatformFileToken,
        truncate: bool,
    ) -> Result<Box<dyn tokio::io::AsyncWrite + Unpin + Send>, PlatformError>;
    async fn power_state(&self) -> Result<PowerState, PlatformError>;
    async fn network_state(&self) -> Result<NetworkState, PlatformError>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlatformKind {
    Android,
    Linux,
    Windows,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlatformFileToken {
    pub opaque_token: String,
    pub display_name: String,
    pub persistable: bool,
}
```

Opaque file tokens:

- are process-local handles to validated platform grants;
- are not persisted unless the platform grant is persistable;
- never permit path traversal;
- expire after 15 minutes if non-persistable;
- are scoped to read or write.

Storage roots and capability reports:

- `PlatformStorageRoot` maps one-for-one to the `storage_roots` columns (§6.2).
  Removability is `kind == StorageRootKind::Removable`, and the default
  destination is `kind == StorageRootKind::AppPrivate`; neither is a separate
  field, matching the table's `kind` CHECK constraint exactly.
  `last_probed_at_ms`, `created_at_ms` and `updated_at_ms` are persistence
  bookkeeping set by the storage layer on insert or probe, so the adapter does
  not report them.
- `PlatformCapabilityReport` is the raw device probe. The application combines
  it with installed model and engine state to assemble the persisted
  `CapabilityReport` DTO (§14) and its `warnings: ReadinessFinding[]`. The
  three permission flags gate live subsystems directly:
  `microphone_granted` → voice annotations (`AnnotationType::Voice`),
  `background_transfer_granted` → background downloads (§8, §17.3), and
  `notifications_granted` → user-task delivery. `inference_viable` is the
  hardware assessment (RAM and an accelerator sufficient to run the release
  model); the DTO's `inferenceReady` additionally requires a compatible model
  to be installed.

## 17.3 Android

Android minimum:

```text
API 28
ABI arm64-v8a
```

Android release requirements:

- process death resumes durable jobs;
- foreground service is shown for user-initiated long transfer;
- SAF permissions are persisted when granted;
- model and pack storage estimates include filesystem allocation overhead;
- audio capture requests permission at the voice action;
- network and battery policy are enforced before resuming background work;
- the single `platform-adapter` crate compiles for Android, Linux and Windows;
  platform-specific code is selected by `#[cfg(target_os = "...")]`, never by a
  Cargo feature.

## 17.4 Linux

Linux release target:

```text
Ubuntu 24.04 LTS and newer
```

Packages:

- AppImage;
- `.deb`.

Secret Service is required for signing keys.
If unavailable, signed export setup explains the limitation and offers an
encrypted key-file workflow selected explicitly by the user.

## 17.5 Windows

Windows release target:

```text
Windows 10 22H2 and Windows 11
x86_64
```

Packages:

- MSIX;
- signed standalone installer.

Windows long paths are enabled in the application manifest.
All user-selected paths pass canonicalization and reserved-name checks.

---

# 18. Security & Trust Boundaries

## 18.1 Threat actors

- malicious ZIM publisher;
- compromised catalog or mirror;
- malicious sidecar sender;
- local unprivileged process;
- hostile web content inside an article;
- compromised model artifact;
- billing/provider outage or false event;
- accidental user path selection;
- corrupted removable storage.

## 18.2 Path canonicalization

```rust
pub fn resolve_beneath(
    root: &Path,
    relative: &Path,
) -> Result<PathBuf, PathSecurityError>;
```

Rules:

- `relative` must be relative;
- `..` is rejected before resolution;
- NUL is rejected;
- Windows device names are rejected;
- symlink/reparse-point traversal is rejected for app-managed roots;
- resolved parent must remain beneath canonical root;
- archive entry paths are never written directly;
- caller supplies a generated safe filename for exported resources.

## 18.3 Article sandbox

Required CSP:

```text
default-src 'none';
img-src enzime-resource: data:;
media-src enzime-resource:;
style-src 'unsafe-inline' enzime-resource:;
font-src enzime-resource:;
connect-src 'none';
frame-src 'none';
object-src 'none';
form-action 'none';
base-uri 'none';
```

Forbidden article capabilities:

- Tauri invoke;
- arbitrary navigation;
- network fetch;
- cookies;
- local storage;
- service workers;
- popups;
- downloads;
- clipboard write without user mediation.

## 18.4 Signature domains

| Artifact | Domain |
|---|---|
| sidecar | `enzime.sidecar.v1\0` |
| entitlement | `enzime.entitlement.v1\0` |
| catalog | `enzime.catalog.v1\0` |
| model manifest | `enzime.model.v1\0` |
| release manifest | `enzime.release.v1\0` |
| backup manifest | `enzime.backup.v1\0` |
| coupon receipt | `enzime.coupon.receipt.v1\0` |

Signatures are not interchangeable across domains.

## 18.5 Secrets

Secrets:

- device Ed25519 private key;
- RevenueCat restore/session token;
- encrypted backup key material;
- optional coupon secret before redemption.

Secrets are never:

- logged;
- included in diagnostics;
- stored in `settings`;
- sent to the renderer;
- embedded in the reboot pak;
- committed to Git;
- placed in a URL.

## 18.6 Logging

Default logs may include:

- opaque entity IDs;
- error codes;
- byte counts;
- durations;
- state transitions;
- app/platform versions.

Default logs exclude:

- article text;
- annotation text;
- message text;
- lore text;
- filesystem paths;
- URLs with query strings;
- tokens;
- keys;
- coupon envelopes.

## 18.7 Quarantine

Quarantined content:

- cannot be opened by a production engine;
- is stored outside active roots;
- has an `audit_events` record;
- records source, expected/actual digest and verifier error;
- can be deleted;
- cannot be “use anyway” activated.

---

# 19. Test Architecture

## 19.1 Test pyramid

| Level | Scope | Per merge |
|---|---|---|
| unit | policies, parsers, reducers | Yes |
| contract | traits, DTO/schema parity | Yes |
| repository | SQLite migrations/transactions | Yes |
| conformance | ZIM, sidecar, entitlement | Yes |
| integration | vertical use cases | Yes |
| component | React states/accessibility | Yes |
| end-to-end | Tauri flows | Yes |
| physical device | Android lifecycle/performance | Release |
| fuzz | untrusted formats | Nightly/release |
| performance | budgets | Release |

## 19.2 Required contract tests

| Test ID | Assertion |
|---|---|
| `CT-IPC-001` | Rust command names equal TS union |
| `CT-IPC-002` | request/response serialization round-trips |
| `CT-IPC-003` | unknown fields rejected where schema is closed |
| `CT-SQL-001` | migration applies to empty database |
| `CT-SQL-002` | foreign keys pass |
| `CT-SQL-003` | invalid job transition aborts |
| `CT-SQL-004` | active pack must be verified |
| `CT-ZIM-001` | all §7.4 fixtures pass |
| `CT-SIDE-001` | deterministic CBOR golden bytes |
| `CT-SIDE-002` | signature domain separation |
| `CT-ENT-001` | perpetual artifact verifies offline |
| `CT-ENT-002` | expired term grant denies only term capability |
| `CT-RET-001` | citation maps to exact locator |
| `CT-RET-002` | unknown model citation is rejected |
| `CT-LORE-001` | candidate never auto-promotes |
| `CT-LORE-002` | deletion removes retrieval projection |
| `CT-PLAT-001` | Android dependency graph contains the full crate set |
| `CT-PLAT-002` | Linux and Windows dependency graphs each contain the full crate set |

## 19.3 UI state matrix

Every primary view tests:

- loading;
- complete empty state;
- populated state;
- recoverable error;
- non-recoverable error;
- offline;
- capability denied;
- reduced motion;
- high contrast;
- 200% font scale;
- narrow mobile width;
- keyboard-only desktop.

“Complete empty state” means a user can take the next valid action.
It is not placeholder copy.

## 19.4 End-to-end release stories

### `E2E-01-airplane-reader`

1. import verified ZIM;
2. search title;
3. open article;
4. enter airplane mode;
5. navigate internal link;
6. create highlight;
7. restart app;
8. highlight remains placed.

### `E2E-02-interrupted-download`

1. plan download;
2. start transfer;
3. terminate process;
4. reopen app;
5. resume verified chunks;
6. corrupt one staged chunk;
7. verifier rejects chunk;
8. last active pack remains readable;
9. retry completes and activates candidate.

### `E2E-03-sidecar-portability`

1. create highlight, comment, ink and voice note;
2. export signed sidecar;
3. import on clean installation;
4. verify signer;
5. resolve annotations against same source;
6. disable imported layer;
7. local annotations remain unaffected.

### `E2E-04-local-ask`

1. install verified local model;
2. index selected pack;
3. ask article-scoped question offline;
4. stream response;
5. open every citation;
6. each opens exact source passage;
7. remove model;
8. conversation and citations remain readable.

### `E2E-05-lore-correction`

1. conversation produces candidate;
2. candidate appears in inbox;
3. user accepts;
4. memory appears in retrieval;
5. user corrects it;
6. previous revision remains auditable;
7. current revision is used;
8. user deletes it;
9. deleted text is absent from retrieval and diagnostics.

### `E2E-06-offline-entitlement`

1. import valid signed perpetual artifact;
2. enable airplane mode;
3. restart device;
4. paid capability remains allowed;
5. expire a term grant;
6. only the term capability becomes denied;
7. existing user data remains readable/exportable.

### `E2E-07-backup-restore`

1. create encrypted backup;
2. verify artifact;
3. install clean app;
4. restore offline;
5. annotations, notebooks, conversations, lore, settings and bindings return;
6. absent pack bytes are represented as source unavailable;
7. no record pretends the source is locally installed.

## 19.5 Capability evidence registry

Machine-readable file:

```text
tests/evidence/capability-evidence.json
```

```ts
export interface CapabilityEvidenceRecord {
  capabilityId: string;
  requirementIds: string[];
  testIds: string[];
  fixtureDigests: Sha256Hex[];
  platforms: Array<"android" | "linux" | "windows">;
  status: "unproven" | "passed" | "failed";
  observedAtMs?: UnixTimeMs;
  ciRunId?: string;
  artifactSha256?: Sha256Hex;
}
```

No capability is marketed as shipped while status is `unproven`.

---

# 20. Build & Release Pipeline

## 20.1 Toolchain locks

Lock files:

- `rust-toolchain.toml`;
- `Cargo.lock`;
- `pnpm-lock.yaml`;
- `VENDOR/LOCKFILE.md`;
- Android Gradle wrapper;
- JDK version file;
- Node version file.

Release build rejects dirty vendored trees.

## 20.2 Release scripts

No `.forgejo/` or `.github/` workflow is authored. Each release check is a
local script under `tools/release/`, invoked manually on the release commit.
The script names are normative; their targets are:

```text
tools/release/lint-rust.sh
tools/release/lint-typescript.sh
tools/release/test-domain.sh
tools/release/test-storage.sh
tools/release/test-contracts.sh
tools/release/test-zim-conformance.sh
tools/release/test-sidecar-conformance.sh
tools/release/test-entitlement-conformance.sh
tools/release/test-react.sh
tools/release/test-e2e-linux.sh
tools/release/check-android-graph.sh
tools/release/check-linux-graph.sh
tools/release/check-windows-graph.sh
tools/release/build-android-debug.sh
tools/release/build-linux-debug.sh
tools/release/build-windows-debug.sh
tools/release/audit-dependencies.sh
tools/release/generate-sbom.sh
tools/release/verify-vendor-lock.sh
tools/release/verify-entity-table.sh
tools/release/verify-no-production-placeholders.sh
```

The `check-*-graph` scripts assert that the Android, Linux and Windows
dependency graphs each contain the full crate set (§5.2, `CT-PLAT-001`,
`CT-PLAT-002`). The `verify-entity-table` script is the local
`tools/contract-gen` mode for the SC1 SP-4 entity-table orphan/stranded check.

## 20.3 Placeholder gate

The release gate scans production source and user-visible strings for:

```text
TODO
FIXME
stub
mock
placeholder
coming soon
not implemented
lorem ipsum
test only
bypass
skip entitlement
```

Allow-list entries require:

- exact file/line pattern;
- reason;
- test-only or documentation-only classification;
- expiry release;
- reviewer.

Production behavior cannot be allow-listed as a placeholder.

## 20.4 Artifact matrix

| Platform | Artifact | Signing |
|---|---|---|
| Android | `.aab` | Play upload/app signing |
| Android | universal `.apk` | release key |
| Linux | `.AppImage` | release manifest Ed25519 |
| Linux | `.deb` | package signature + release manifest |
| Windows | `.msix` | Authenticode |
| Windows | `.exe` installer | Authenticode |
| Shared | SBOM CycloneDX | release manifest |
| Shared | source archive | release manifest |

## 20.5 Release manifest

The signed manifest contains:

- version;
- Git commit;
- clean-tree assertion;
- toolchain versions;
- vendored source digests;
- artifact names, sizes and SHA-256;
- SBOM digest;
- migration set digest;
- schema digest;
- capability evidence digest;
- signer key ID;
- build timestamp.

## 20.6 Release gates

Release requires:

- all local release scripts pass on the release commit;
- physical Android `E2E-01` through `E2E-07`;
- Linux and Windows desktop smoke;
- database backup/restore drill;
- accessibility audit;
- privacy/diagnostic review;
- threat-model review;
- third-party notice completeness;
- no unproven advertised capability;
- no unresolved severity-high audit event;
- no production placeholder;
- all ten UI contracts implemented, whether or not Stitch was used to draft
  their visual layout.

---

# 21. Performance Targets

Targets are release pass/fail values on declared reference hardware.

## 21.1 Reference hardware

### `REF-ANDROID-MID`

```text
Android 14
8 GiB RAM
UFS 3.1
arm64
midrange 2024 SoC
```

### `REF-DESKTOP-LOW`

```text
Windows 10 22H2 or Ubuntu 24.04
4-core x86_64 CPU
8 GiB RAM
SATA SSD
no discrete GPU
```

### `REF-DESKTOP-GPU`

```text
Windows 11 or Ubuntu 24.04
8-core x86_64 CPU
32 GiB RAM
NVMe SSD
24 GiB NVIDIA GPU
```

## 21.2 Budgets

| Metric | Target |
|---|---:|
| cold app shell to interactive desktop | ≤ 2.5 s |
| cold app shell to interactive Android | ≤ 3.5 s |
| warm cached article render Android | ≤ 500 ms |
| warm cached article render desktop | ≤ 250 ms |
| title-search first result | ≤ 150 ms |
| annotation local acknowledgement | ≤ 100 ms |
| open Notes with 10,000 annotations | ≤ 800 ms |
| Home snapshot | ≤ 300 ms |
| SQLite migration from previous release | ≤ 30 s or progress UI |
| transfer sustained overhead vs direct download | ≤ 12% |
| resume after crash | ≤ 2 s plus source handshake |
| sidecar export 10,000 text annotations | ≤ 5 s |
| sidecar import plan 10,000 annotations | ≤ 8 s |
| FTS indexing | ≥ 5 MiB normalized text/s |
| embedding UI main-thread blocking | 0 frames > 50 ms |
| ask cancel acknowledgement | ≤ 250 ms |
| first token GPU reference | measured profile ≤ 2.5 s |
| first token CPU low reference | measured profile ≤ 12 s |
| reader RSS without inference Android | ≤ 450 MiB |
| reader RSS without inference desktop | ≤ 600 MiB |

## 21.3 Measurement

Every performance result records:

- reference hardware ID;
- OS;
- app version/commit;
- pack/model digest;
- cold/warm state;
- sample count;
- p50;
- p95;
- maximum;
- profiler artifact.

Marketing uses measured named configurations.

---

# 22. Migration Strategy

## 22.1 Donor rule

Donor repositories are not merged wholesale.
The only source admitted initially is the locked vendor tree.

Any later donor requires a `DonorReceipt`:

```ts
export interface DonorReceipt {
  donorId: string;
  sourceUrl: string;
  sourceCommit: string;
  licenseSpdx: string;
  importedPaths: string[];
  destinationPaths: string[];
  publicEntitiesUsed: string[];
  testsAdded: string[];
  rejectedDependencies: string[];
  reviewer: string;
}
```

## 22.2 Initial vendor inputs

Every vendored component is compiled from source held under `VENDOR/`
(AMD-3). Expected locked inputs:

| Vendor ID | Role |
|---|---|
| `vendor.tauri.tauri-v2.11.1` | shell/build reference |
| `vendor.wllama3.3.5.1` | inference runtime (sole local engine, all platforms) |
| `vendor.anzimmerman-src.c73245e` | multilingual ZIM library |

Exact commits/digests are authority of `VENDOR-MANIFEST.md`.
If that manifest cannot prove an input, implementation does not begin.

## 22.3 Migration stages

### Stage M0 — cold-start audit

- verify pak manifest;
- verify vendor manifest;
- confirm no absolute-path dependency;
- confirm every entity resolves;
- confirm all checklist markers are `[ ]`.

### Stage M1 — contract skeleton

- create repository tree;
- implement domain types;
- apply SQLite migration;
- generate TypeScript contracts;
- create the single release composition;
- pass contract tests.

### Stage M2 — first complete vertical slice

- import one real ZIM;
- verify;
- activate;
- search;
- read;
- annotate;
- restart;
- re-open annotation.

### Stage M3 — complete library and transfer

- catalog;
- plan dialog;
- resumable transfer;
- last-known-good update;
- storage planner;
- queue UI.

### Stage M4 — complete sidecars and notes

- notebooks/tags/layers;
- rich annotations;
- signed export/import;
- cross-pack Notes.

### Stage M5 — complete inference and retrieval

- model install;
- backend selection;
- indexing;
- bounded cited Ask;
- citation open.

### Stage M6 — complete lore

- candidates;
- inbox;
- accept/reject;
- correction;
- pin/delete;
- retrieval projection.

### Stage M7 — complete commerce and recovery

- signed artifacts;
- RevenueCat normalization;
- coupon ledger;
- backup/restore;
- diagnostics;
- privacy controls.

### Stage M8 — hardening and release

- all platform builds;
- accessibility;
- performance;
- security;
- signing;
- store artifacts.

## 22.4 Prohibited migration shortcuts

- copying historical `lib.rs` module declarations;
- copying a checklist with completion markers;
- preserving historical package versions without lock evidence;
- importing a UI whose nouns contradict §16;
- adding compatibility aliases for unapproved old entity names;
- retaining bypass gates;
- leaving historical “temporary” captions;
- treating a successful compile as product acceptance;
- claiming a screen exists because a prompt was submitted;
- claiming Stitch output exists without `get_screen` evidence.

---

# 23. Diagram Index

| Diagram ID | Section | Format | Purpose |
|---|---:|---|---|
| `D-01-system-context` | 2.1 | Mermaid | actors/external systems |
| `D-02-trust-boundaries` | 2.4 | Mermaid | privileged/untrusted boundaries |
| `D-03-layer-dependencies` | 3.4 | Mermaid | dependency direction |
| `D-04-download-state` | 8.4 | Mermaid | durable transfer state |
| `D-05-model-lifecycle` | 10.5 | Mermaid | model installation |
| `D-06-retrieval-pipeline` | 11.1 | Mermaid | staged retrieval |
| `D-07-lore-lifecycle` | 12.1 | Mermaid | memory governance |
| `D-08-component-tree` | 15.1 | tree | React composition |

The repository also stores rendered/exportable diagram sources:

```text
DOCS/diagrams/D-01-system-context.mmd
DOCS/diagrams/D-02-trust-boundaries.mmd
DOCS/diagrams/D-03-layer-dependencies.mmd
DOCS/diagrams/D-04-download-state.mmd
DOCS/diagrams/D-05-model-lifecycle.mmd
DOCS/diagrams/D-06-retrieval-pipeline.mmd
DOCS/diagrams/D-07-lore-lifecycle.mmd
DOCS/diagrams/D-08-component-tree.puml
```

---

# 24. Entity-to-UI Binding Contract

The following table is normative.
“None” means the entity is derived from a ZIM/runtime response and has no
canonical SQLite row.

| UI component | TypeScript entity | Tauri command(s) | SQLite table(s) | Zustand slice |
|---|---|---|---|---|
| `HomeView` | `HomeSnapshot` | `home_snapshot` | projection across tables | `homeStore.snapshot` |
| `ContinueReadingCard` | `ReaderHistoryItem` | `reader_history` | `reader_history` | `homeStore.snapshot.continueReading` |
| `RecentPacksList` | `PackManifest[]` | `pack_list` | `packs`, `pack_versions` | `homeStore.snapshot.recentPacks` |
| `TaskInbox` | `UserTask[]` | `task_list`, `task_resolve` | `user_tasks` | `taskStore.tasks` |
| `MemoryInboxCard` | `number` | `lore_inbox` | `lore_candidates` | `homeStore.snapshot.memoryInboxCount` |
| `ReadinessPanel` | `SystemReadiness` | `readiness_get` | `capability_reports`, `settings` | `homeStore.snapshot.readiness` |
| `ReadinessIndicator` | `SystemReadiness` | `readiness_get` | `capability_reports`, `settings` | `appStore.bootstrap.readiness` |
| `JobStrip` | `DownloadJob[]` | `job_list` | `download_jobs` | `libraryStore.downloads` |
| `LibraryView` | `PackManifest[]` | `pack_list` | `packs`, `pack_versions`, `pack_locations` | `libraryStore.packs` |
| `LibraryToolbar` | `CatalogSearchRequest` | `catalog_search`, `pack_import_choose` | `catalog_entries` | `libraryStore.catalogQuery` |
| `PackGrid` | `PackManifest[]` | `pack_list` | `packs`, `pack_versions` | `libraryStore.packs` |
| `PackCard` | `PackManifest` | `pack_list`, `pack_metadata` | `packs`, `pack_versions`, `pack_locations` | `libraryStore.packs` |
| `CatalogBrowser` | `CatalogEntry[]` | `catalog_search`, `catalog_refresh` | `catalog_sources`, `catalog_entries` | `libraryStore.catalogEntries` |
| `CatalogEntryCard` | `CatalogEntry` | `install_plan` | `catalog_entries` | `libraryStore.catalogEntries` |
| `DownloadQueue` | `DownloadJob[]` | `job_list`, `job_pause`, `job_resume`, `job_cancel` | `download_jobs`, `download_chunks` | `libraryStore.downloads` |
| `DownloadJobRow` | `DownloadJob` | `job_get`, `job_pause`, `job_resume`, `job_cancel` | `download_jobs`, `download_chunks` | `libraryStore.downloads` |
| `StoragePlanner` | `StorageDestination[]` | `storage_roots_list` | `storage_roots` | `libraryStore.storageRoots` |
| `DownloadPlanDialog` | `InstallPlan` | `install_plan`, `install_start` | `install_transactions`, `download_jobs` | `libraryStore.installPlan` |
| `InstallPlanSummary` | `InstallPlan` | `install_plan` | `catalog_entries`, `storage_roots` | `libraryStore.installPlan` |
| `StorageImpactBar` | `InstallPlan` | `install_plan` | `storage_roots` | `libraryStore.installPlan` |
| `ReaderView` | `ArticleResponse` | `reader_open`, `reader_article` | `reader_history`, `reader_positions` | `readerStore.currentArticle` |
| `BreadcrumbNav` | `BreadcrumbItem[]` | `reader_article` | None | `readerStore.currentArticle.breadcrumb` |
| `ArticleToolbar` | `ArticleResponse` | `bookmark_set`, `reader_article` | `bookmarks`, `annotations` | `readerStore.currentArticle` |
| `ReaderPane` | `ArticleResponse` | `reader_article` | None | `readerStore.currentArticle` |
| `ArticleContent` | `ArticleResponse` | `reader_article`, `reader_position_save` | `reader_positions` | `readerStore.currentArticle` |
| `AnnotationWorkspace` | `AnnotationRecord[]` | `annotation_list`, `annotation_create`, `annotation_update` | `annotations`, `annotation_revisions` | `annotationStore.annotations` |
| `AnnotationPanel` | `AnnotationRecord[]` | `annotation_list`, `annotation_delete`, `annotation_restore` | `annotations`, `annotation_revisions` | `annotationStore.annotations` |
| `TextHighlighter` | `SelectorBundle` | `annotation_create` | `annotations`, `annotation_revisions` | `readerStore.selection` |
| `InkCanvas` | `AnnotationAssetBeginResponse` | `annotation_asset_begin`, `annotation_asset_chunk`, `annotation_asset_commit` | `annotation_assets` | `annotationStore.annotations` |
| `VoiceRecorder` | `AnnotationAssetBeginResponse` | `annotation_asset_begin`, `annotation_asset_chunk`, `annotation_asset_commit` | `annotation_assets` | `annotationStore.annotations` |
| `SidecarImportDialog` | `SidecarImportPlan` | `sidecar_import_choose`, `sidecar_import_plan`, `sidecar_import_commit` | `sidecar_imports`, `sidecar_layers` | `annotationStore.layers` |
| `ReaderCompanionDrawer` | `ConversationDetail` | `conversation_create`, `ask_start` | `conversations`, `messages` | `askStore.activeConversation` |
| `AskView` | `ConversationDetail` | `conversation_list`, `conversation_get`, `ask_start`, `ask_cancel` | `conversations`, `messages`, `inference_runs` | `askStore.activeConversation` |
| `ConversationSidebar` | `ConversationSummary[]` | `conversation_list`, `conversation_create` | `conversations` | `askStore.conversations` |
| `ScopeSelector` | `RetrievalScope` | `pack_list`, `notebook_tree` | `packs`, `notebooks` | `askStore.scope` |
| `BackendStatusBadge` | `BackendStatus` | `backend_status`, `backend_select` | `inference_backends`, `model_installations` | `askStore.backendStatus` |
| `ConversationThread` | `ConversationMessage[]` | `conversation_get` | `messages`, `message_citations` | `askStore.activeConversation.messages` |
| `MessageBubble` | `ConversationMessage` | `conversation_get` | `messages`, `message_citations` | `askStore.activeConversation.messages` |
| `CitationChip` | `CitationRecord` | `citation_open` | `message_citations`, `retrieval_chunks` | `askStore.activeConversation.messages` |
| `Composer` | `AskStartRequest` | `ask_start`, `ask_cancel` | `messages`, `inference_runs` | `askStore.streamingRunId` |
| `NotesView` | `AnnotationRecord[]` | `annotation_list` | `annotations`, `annotation_revisions` | `annotationStore.annotations` |
| `NotebookTree` | `NotebookNode[]` | `notebook_tree`, `notebook_create`, `notebook_update` | `notebooks` | `annotationStore.notebooks` |
| `NotesToolbar` | `AnnotationListRequest` | `annotation_list` | `annotations`, `annotation_tags` | `annotationStore.filter` |
| `TagFilter` | `TagSummary[]` | `tag_list`, `tag_create`, `annotation_list` | `tags`, `annotation_tags` | `annotationStore.tags` |
| `SidecarLayerToggle` | `SidecarLayer[]` | `sidecar_layers`, `annotation_list` | `sidecar_layers` | `annotationStore.layers` |
| `AnnotationList` | `AnnotationRecord[]` | `annotation_list` | `annotations`, `annotation_revisions` | `annotationStore.annotations` |
| `AnnotationListItem` | `AnnotationRecord` | `annotation_list` | `annotations`, `annotation_revisions` | `annotationStore.annotations` |
| `AnnotationDetailPanel` | `AnnotationRecord` | `annotation_update`, `annotation_delete` | `annotations`, `annotation_revisions` | `annotationStore.selectedAnnotationId` |
| `MemoryView` | `LoreMemory[]` | `lore_search`, `lore_inbox` | `lore_memories`, `lore_candidates` | `memoryStore.memories` |
| `MemoryInbox` | `LoreCandidate[]` | `lore_inbox`, `lore_accept`, `lore_reject` | `lore_candidates`, `lore_candidate_evidence` | `memoryStore.candidates` |
| `MemoryWorkspace` | `LoreMemory[]` | `lore_search` | `lore_memories`, `lore_memory_revisions` | `memoryStore.memories` |
| `EntityGraph` | `EntityGraphNode[]` | `lore_search` | `lore_memories`, `lore_memory_evidence` | `memoryStore.memories` |
| `EpisodeTimeline` | `LoreMemory[]` | `lore_search` | `lore_memories`, `lore_memory_revisions` | `memoryStore.memories` |
| `MemoryDetailPanel` | `LoreMemory` | `lore_correct`, `lore_pin`, `lore_delete` | `lore_memories`, `lore_memory_revisions` | `memoryStore.selectedMemoryId` |
| `SettingsView` | `SettingsSnapshot` | `settings_get`, `settings_update` | `settings` | `settingsStore.settings` |
| `NetworkPolicyPanel` | `NetworkPolicy` | `settings_get`, `settings_update` | `settings` | `settingsStore.settings.networkPolicy` |
| `ComputePanel` | `ComputePolicy` | `settings_update`, `backend_status`, `model_list`, `model_install_plan`, `model_remove` | `settings`, `models`, `model_installations` | `settingsStore.settings.computePolicy` |
| `AccessibilityPanel` | `AccessibilitySettings` | `settings_get`, `settings_update` | `settings` | `settingsStore.settings.accessibility` |
| `EntitlementPanel` | `CapabilityDecision[]` | `entitlement_decisions`, `entitlement_refresh`, `entitlement_restore`, `coupon_redeem` | `entitlement_artifacts`, `entitlement_grants`, `coupon_ledger` | `settingsStore.settings.capabilities` |
| `BackupRestorePanel` | `BackupSettings` | `backup_create`, `backup_restore_plan`, `backup_restore_commit` | `settings`, `backups` | `settingsStore.settings.backup` |
| `SetupRoute` | `CapabilityReport` | `capability_probe`, `model_list` | `capability_reports`, `models` | `appStore.bootstrap` |
| `SetupWizard` | `CapabilityReport` | `capability_probe`, `settings_update` | `capability_reports`, `app_meta` | `appStore.bootstrap` |
| `CapabilityReport` | `CapabilityReport` | `capability_probe` | `capability_reports` | `appStore.bootstrap` |
| `SetupPathChooser` | `SetupPath` | `pack_import_choose`, `catalog_search` | `app_meta` | `appStore.bootstrap` |
| `ModelRecommendation` | `ModelSummary[]` | `model_list`, `model_install_plan` | `models`, `model_installations` | `settingsStore.models` |

## 24.1 Binding validation

The build tool `tools/contract-gen` reads:

- the command registry;
- generated TypeScript entities;
- component binding manifest;
- SQLite schema;
- store interfaces.

It fails when:

- a component references an undeclared entity;
- a command is absent from Rust or TypeScript;
- a table name is absent from the migration;
- a store slice is absent;
- a component public name differs from §16.6;
- a screen prompt uses a different entity name.

---

# 25. Architecture Completeness Attestation

The architecture is complete only when the following statements remain true:

- every product surface has a route or declared modal/substate;
- every stateful component has a binding row;
- every binding row has an entity, command, source and store;
- every command has request and return DTOs;
- every durable entity has an exact SQL representation;
- every replaceable engine has an exact Rust trait;
- every release platform has complete adapter bindings;
- every long-running mutation has persisted state;
- every activation retains last-known-good until verification;
- every user-created entity has export/backup behavior;
- every untrusted format has validation and quarantine behavior;
- every entitlement decision works offline from signed evidence;
- every inference answer in grounded mode has citation validation;
- every memory change preserves evidence and correction semantics;
- every release claim has capability evidence;
- no implementation step requires prior EnZIME context.

Any failed statement reopens architecture work.
It is not delegated to a coder as “figure it out.”

## 25.1 Re-attestation (2026-07-27)

On 2026-07-27 the §1–§25 baseline was reconciled in place to the §26
amendments AMD-1 through AMD-4, and the §7.1 `DigestAlgorithm` compile defect
was corrected. After that reconciliation:

- this document is **complete, stub-free and end-state** — it contains no
  `TBD`, no “future work”, no “open issue”, no placeholder and no stub;
- the baseline (§1–§25) and the §16 entity table are internally consistent
  with the effective architecture (baseline overlaid with §26);
- every public entity named here resolves to exactly one definition; no entity
  has a second name;
- the contract chain (Rust identities → SQL schema → TypeScript contracts →
  §24 entity-to-UI binding) is serializable end-to-end.

**Attestation timestamp (ISO-8601, UTC):** `2026-07-27T14:31:47Z`
**Authoring agent:** model family `GLM`, version `5.2` (`glm-5.2`), harness
`opencode`.

### Amendment — §17.2 platform types defined (2026-07-27)

On 2026-07-27 the five types referenced only at their use sites inside the
`PlatformAdapter` trait body were given their definitions in §17.2, and the
§16.2 entity table was extended with their rows. The five types are:
`PlatformError`, `PlatformCapabilityReport`, `PlatformStorageRoot`,
`PowerState`, `NetworkState` (plus the two supporting enums their shapes
require, `StorageRootKind` and `ThermalLoad`). With these definitions in place,
the §25.1 entity-resolution claim — "every public entity named here resolves to
exactly one definition" — now holds for the §17.2 platform port: a coder
transcribing §17.2 produces code that compiles, with no undefined type
references. `open_read` / `open_write` were confirmed to intend Tokio async I/O
(`tokio::io::AsyncRead` / `AsyncWrite`), distinct from the `futures_core::Stream`
used for §7.2 / §8.2 item streams; §17.2 prose now states this and the resulting
`crates/ports` → `tokio` dependency. This note is appended; the attestation
above is not rewritten.

---

# 26. Amendments (append-only; SC1 SP-6)

The §1–§25 baseline and the §16 entity table are frozen. Every subsequent
change lands here as a diff-shaped overlay. Effective architecture =
baseline overlaid with this section, in order.

## AMD-1 (2026-07-27) — One codebase, one composition, all platforms, all functionality

Operator-directed. Android, Linux and Windows ship the **same Tauri 2
codebase with the complete feature set**. No Cargo feature removes a
user-visible capability from any platform.

```text
MODIFY §5.2: replace the android/linux/windows feature compositions with:

[features]
default = [
  "dep:platform-adapter",
  "dep:inference-wllama",
  "dep:revenuecat-adapter"
]

Platform difference is expressed ONLY by #[cfg(target_os = "...")] inside
`crates/platform-adapter`, never by a feature that omits a crate.
```

- `crates/platform-android` and `crates/platform-desktop` are superseded by a
  single `crates/platform-adapter` exposing the §17.2 platform port, with
  per-target `#[cfg]` modules `android.rs`, `linux.rs`, `windows.rs`.
- The §5.2 "exactly one of android/linux/windows is enabled" assertion is
  retired. The replacement assertion: **every platform's dependency graph is
  identical**; a graph difference is a build failure.
- `CT-PLAT-001`/`CT-PLAT-002` (§19.2) are restated as: the Android, Linux and
  Windows dependency graphs each contain the full crate set.

## AMD-2 (2026-07-27) — wllama3 is the sole local inference engine, all platforms

Operator-directed, absolute requirement.

```text
MODIFY §10.3 Release backends:
  RETIRE  inference-litert (LiteRT-LM)   — removed from the release set
  RETIRE  inference-gguf                 — removed from the release set
  ADD     crates/inference-wllama        — wllama3, ALL platforms

MODIFY §1.1 platform posture, row "Local inference":
  Android | Linux | Windows  →  wllama3 (all three)

MODIFY §16.1 crate rows: replace inference-litert / inference-gguf rows
  with a single inference-wllama row.
```

The §10.1/§10.2 `InferenceBackend`, `InferenceSession` and `EmbeddingModel`
traits are **unchanged** — they are provider-agnostic ports, and wllama3 is a
provider behind them. §10.4's selection policy resolves to one backend.

## AMD-3 (2026-07-27) — Build from our own vendored source

Operator-directed. Every vendored component is compiled from source we hold,
so an upstream change can neither impair us nor lock us out of servicing our
own code.

- Vendored trees live at `VENDOR/<component>/` (§5), cloned once at that
  permanent path, each with a sibling `<component>.UPSTREAM-VENDOR.lock`
  pinning remote, tag, commit, commit date, vendor date and tree digest
  (`VENDOR-MANIFEST.md` § "Tree digest algorithm").
- Vendor set: `wllama3`, `tauri` (v2.11.1), `AnZIMmermanLib`.
- Upstream prebuilt binaries may be used **only** as development-time
  scaffolding to keep work moving; they are never linked into a release
  artifact, and each such use carries a from-source successor task.

## AMD-4 (2026-07-27) — No CI until public-release operator signoff

Operator-directed global rule.

- No `.forgejo/` or `.github/` workflow is authored.
- §20.2's job names become local scripts under `tools/release/`, invoked
  manually; §20.6's "all CI jobs green" is restated as "all local release
  scripts pass on the release commit".
- SC1 SP-4's entity-table orphan/stranded check becomes a local
  `tools/contract-gen` mode, not a CI gate.
- Version stamping uses the canonical script (`scripts/update-version.sh`,
  copied from `~/Admin-Manual/versioning/update-version.sh`), never a manual
  edit. MAJOR=4. Artifacts are staged in the canonical checkout's tracked
  `dist/`, slug-first: `enzime-reader-v<MAJOR.MINOR.BUILD>-<qualifier>`.
