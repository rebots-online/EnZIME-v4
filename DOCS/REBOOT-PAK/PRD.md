# EnZIME Reader — Canonical Product Requirements

**Version:** 4.0.0
**Date:** 2026-07-26
**Status:** Normative

## 1. Product

EnZIME Reader is an offline-first, multilingual ZIM knowledge reader,
annotator and cited local-conversation product for Android, Linux and Windows.

It lets a user:

- import or acquire verified knowledge packs;
- understand storage cost before transfer;
- read and search without a network;
- create durable text, ink and voice annotations;
- move annotations through signed sidecars;
- ask an installed local model questions over explicit scopes;
- open exact supporting citations;
- govern proposed long-term lore memory;
- retain purchased capability offline;
- back up and restore primary user data.

The product must remain a useful reader and annotator without an AI model.

## 2. Product promise

> The knowledge you acquire and the work you add to it remain usable when the
> network, model, vendor, source location or prior application state is absent.

## 3. Platforms

- Android API 28+, arm64-v8a.
- Ubuntu 24.04 LTS+.
- Windows 10 22H2 and Windows 11, x86_64.

All share one domain and IPC contract.
Platform adapters are compiled only into their target.

## 4. Personas

| Persona | Primary job |
|---|---|
| Offline reader | Find and read reference knowledge without connectivity |
| Knowledge worker | Annotate, connect, retrieve and cite across packs |
| Storage-constrained user | Fit a deliberate reserve into available storage |
| Preparedness user | Know that reading, notes and entitlements work offline |
| Accessibility user | Complete every core flow with assistive technology |

## 5. Binding product principles

1. Offline reading, annotation, local search, export and signed entitlement
   verification require no account or internet.
2. User work survives archive eviction and model replacement.
3. Location is not identity.
4. No unverified artifact becomes active.
5. The last verified artifact remains active until replacement verifies.
6. AI receives bounded explicit scope and produces resolvable citations.
7. Model output cannot silently become accepted memory.
8. Commerce cannot revoke perpetual local ownership.
9. Generated UI follows the architecture entity table.
10. The reboot pak plus vendored source is sufficient to start.

## 6. Required product surfaces

### 6.1 Home

Home shows:

- continue reading;
- recent packs;
- unresolved tasks;
- memory inbox count;
- system readiness.

### 6.2 Library

Library provides:

- installed pack grid/list;
- catalog browse/search/filter;
- local import;
- storage planner;
- download queue;
- pause/resume/cancel;
- verified update/removal;
- pre-download impact plan.

### 6.3 Reader

Reader provides:

- sanitized article rendering;
- breadcrumb/back/forward navigation;
- internal links/resources;
- title/full-text search entry;
- bookmarking;
- annotation tools;
- Ask scope entry;
- persisted reading position.

### 6.4 Annotation workspace

Annotation provides:

- highlight;
- comment;
- freehand ink;
- voice note;
- notebooks;
- tags;
- layer visibility;
- resolution status;
- signed sidecar import/export.

### 6.5 Ask

Ask provides:

- conversations;
- passage/article/pack/notebook/all-local scope;
- backend/model status;
- streaming local response;
- cancellation;
- citation chips;
- citation-to-source navigation;
- useful model-not-installed state.

### 6.6 Notes

Notes provides:

- cross-pack annotations;
- notebook tree;
- tags;
- sidecar layers;
- text/resolution filters;
- annotation detail/source navigation.

### 6.7 Memory

Memory provides:

- candidate inbox;
- evidence;
- accept/reject;
- correction;
- pin;
- delete;
- entity graph;
- revision/episode timeline.

### 6.8 Settings

Settings provides:

- network policy;
- model/backend policy;
- accessibility;
- entitlement status/restore/coupon;
- backup/restore;
- diagnostics;
- privacy/export/delete controls.

### 6.9 Setup

Setup provides:

- capability probe;
- warnings and actionable readiness;
- existing files / choose packs / explore without AI paths;
- compatible model recommendation;
- completion without model installation.

### 6.10 Download plan

Before transfer, show:

- item and version;
- download/final/temporary size;
- destination;
- free space before/after;
- source;
- range/full mode;
- checksum/signature/format verification;
- retained last-known-good version.

## 7. Functional requirements

| ID | Requirement |
|---|---|
| `FR-ZIM-001` | Open and validate real ZIM through `ZimEngine` |
| `FR-ZIM-002` | Render safe HTML/resources and internal links |
| `FR-ZIM-003` | Search titles and available full-text index offline |
| `FR-PACK-001` | Normalize approved catalogs |
| `FR-PACK-002` | Plan storage impact before transfer |
| `FR-PACK-003` | Resume verified chunks after process death |
| `FR-PACK-004` | Activate only verified candidate |
| `FR-PACK-005` | Preserve current active pack on failure |
| `FR-ANN-001` | Store multi-selector text annotations |
| `FR-ANN-002` | Store ink and voice assets by digest |
| `FR-ANN-003` | Resolve, mark ambiguous, unavailable or orphaned |
| `FR-SIDE-001` | Export deterministic signed `.ezsc` |
| `FR-SIDE-002` | Plan and idempotently import sidecar |
| `FR-AI-001` | Select compatible installed backend/model |
| `FR-AI-002` | Retrieve lexical/vector candidates in explicit scope |
| `FR-AI-003` | Enforce context budget |
| `FR-AI-004` | Stream and cancel local generation |
| `FR-CITE-001` | Persist only citations mapping to evidence |
| `FR-LORE-001` | Extract reviewable candidates |
| `FR-LORE-002` | Accept/reject/correct/pin/delete with evidence |
| `FR-ENT-001` | Normalize RevenueCat events behind port |
| `FR-ENT-002` | Verify signed local grants offline |
| `FR-ENT-003` | Record/reconcile offline coupons |
| `FR-BACK-001` | Create verified optional encrypted backup |
| `FR-BACK-002` | Plan conflicts before restore |
| `FR-DIAG-001` | Export inspectable redacted diagnostics |
| `FR-A11Y-001` | Complete core flows by keyboard/screen reader |

## 8. Non-functional requirements

### 8.1 Reliability

- SQLite WAL and foreign keys.
- Durable long-job checkpoints.
- Atomic object commit and activation.
- Idempotent sidecar import.
- Backup before destructive migration.
- No silent ambiguous repair.

### 8.2 Security

- Isolated article origin without native bridge.
- Strict CSP.
- Bounded parsing/decompression/allocation.
- Canonical path containment.
- Domain-separated Ed25519 signatures.
- OS-keystore secrets.
- Quarantine without “use anyway.”

### 8.3 Privacy

- No content telemetry.
- No account for core use.
- Diagnostics exclude content and secrets by default.
- Network use follows visible policy.
- Lore extraction is user-controlled.
- User data export remains available in all editions.

### 8.4 Accessibility

- WCAG 2.2 AA.
- Keyboard-complete desktop.
- Platform touch targets.
- Screen-reader names and live state.
- 200% font scale.
- Reduced motion.
- Status never colour-only.

### 8.5 Performance

The pass/fail budgets in `ARCHITECTURE.md` §21 are product requirements.

## 9. Commercial boundary

Free core:

- import/read/search;
- bookmarks;
- text highlights/comments;
- human-readable export.

Paid capabilities may include:

- unlimited packs;
- rich media annotations;
- supported local inference;
- lore memory;
- encrypted backup;
- signed sidecar export;
- premium model catalog.

Payment status never blocks user-created data access/export.

## 10. Release acceptance

Release requires:

1. real ZIM import/read/search/annotation in airplane mode;
2. interrupted download recovery with last-known-good preserved;
3. signed sidecar round-trip;
4. cited local Ask with citation navigation;
5. lore candidate/correction/deletion lifecycle;
6. offline perpetual entitlement after restart;
7. clean-install backup restore;
8. Android/Linux/Windows build and smoke evidence;
9. accessibility and threat-model review;
10. zero production stubs, bypass gates or end-user-inappropriate captions.

## 11. Non-goals

- social network;
- cloud-hosted account requirement;
- autonomous medical/legal/survival decisions;
- arbitrary web browsing inside the article renderer;
- public face/person identification;
- invalid “partial ZIM” format claims;
- browser extension or sibling-app scope in this reboot;
- Creator Studio in the Reader release;
- hidden cloud inference fallback.

## 12. Specification link

`ARCHITECTURE.md` defines every implementation contract.
`CHECKLIST.md` operationalizes it.
When a checklist row lacks enough information, architecture is corrected before
coding resumes.

