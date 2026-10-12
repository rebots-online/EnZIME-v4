# EnZIME-v4 — release reassessment and commercial path

**Assessment date:** 2026-09-13. **Source assessed:** `a727a75d99e816c1ae6ee1e1d5788699eeff5b63`.
**Version recorded by the project:** `1.6.55832`.
**Disposition:** integrated development prototype; not a commercial release candidate.
**Method:** `source-command-sc-analyze`, static source and artifact evidence review;
subsequent user-requested native artifact trial and bounded stylesheet repair.

## 1. Where the project stands

The repository contains substantial reusable implementation: domain/port boundaries,
SQLite migrations, a ZIM reader, bounded transfer adapters, a deterministic DynDon
cluster-fit planner, a cluster-cache archive, annotation persistence, sidecar codecs,
an inference adapter, citation orchestration, entitlement policy, and backup code.
The current shell connects Home, Library, Reader and Notes, including annotation
commands. September commits preserve more working source than the July checklist
and M1 dispatch narrative describe.

The critical unfinished work is the connection between these pieces and durable
customer journeys. The current DynDon path requires a complete local source ZIM;
installed archives are not reopened after restart; portable note import can skip
data; and production Ask, paid entitlement and backup/restore are not connected.
Historical packaging success does not establish that the current product works.

Retain the intended product: storage-constrained offline knowledge, durable portable
annotations, cited local Ask, governed lore, and offline ownership. Retain Android,
Linux and Windows under one composition (architecture AMD-1); reconcile the required
web/PWA complement in the release matrix. A reader-only public release would change
the agreed product, not simply finish this checklist sooner.

### Observed metrics and their limits

| Measure | Observation | Interpretation |
|---|---|---|
| CodeGraph | 214 indexed files; 4,324 nodes; 12,863 edges; SQLite quick_check passes | Structural coverage, including generated Android files; not test coverage or a dependency audit |
| Checklist rows at assessment | 75: 24 `[ ]`, 32 `[/]`, 9 `[X]`, 10 `✅` | Historical bookkeeping; one `✅` is the administrative supersession of P5-03 |
| Phase gates | All 11, P0-GATE through P10-GATE, unchecked | No documented progression to commercial acceptance |
| Tauri command registration | 27 registered versus 80 names in architecture §14.4 | Exposed integration surface, not a feature-completion percentage |
| Frontend routes | Home, Library, Notes, Reader | Ask, Memory, Settings and Setup are not registered |
| Historical artifact checks | All 10 v1.0 manifest entries match actual size and SHA-256 | Byte integrity only; signatures and working behavior were not reverified |
| Newest AppImage | v1.6.55832; 82,315,768 bytes | Entered Git in `b08e417`; unchanged by current annotation commit `a727a75` |
| Source publication at start | `master` is three commits ahead of freshly fetched `origin/master` (`44f6b40`) | Preserved implementation was not yet on the fetched primary branch |

The initial static assessment did not run builds, tests or the app. The user then
requested an artifact trial: the existing v1.6.55832 AppImage was launched on
`msi4090` with isolated data/cache directories and rejected for its visibly unstyled
Library. F10 records that observation and the subsequent bounded repair. Neither
this launch nor the stylesheet protocol establishes catalog acquisition or offline
reading. Model runs, fuzzing, benchmarks and the full commercial gauntlet remain
unexecuted here. No coverage percentage, performance pass, exploit demonstration,
signature certification or SHIP-READY verdict is asserted.

## 2. Prioritized findings

P1 means the finding prevents delivery of the specified commercial experience or
risks customer data. P2 means release preparation/contract debt that must also close.
These findings are bounded to the inspected composition; they are not an exhaustive
security certification. Paths and line numbers refer to the assessed source commit.

### F01 — P1: DynDon does not yet acquire an oversized remote pack

`apps/reader/src-tauri/src/ipc.rs:1681–1705` rejects remote artifact URLs and opens
and hashes a complete local ZIM before planning. `catalog_search` at line 1224 reads
SQLite; startup exposes no `catalog_refresh` command. The storage-fit planner exists,
but requiring the entire source locally defeats the advertised storage-constrained
acquisition journey. The supplied selection profile is ignored at line 1624; domain,
topic and priority are synthesized at lines 1740–1742, and intent uses no selection
limits at lines 1811–1815.

**Close through:** P2-03, P3-01, P3-03, P3-06. Architect the remote directory/cluster
manifest and authenticated source-version contract, catalog population, selection
intent and quota accounting before dispatch. Preserve the cluster-cache/no-ZIM-writer
decision recorded in M1-STATE. Demonstrate a subset acquisition when the full source
cannot fit locally, without relying on an already-downloaded full archive.

### F02 — P1: an installed pack cannot be reopened after an application restart

`apps/reader/src-tauri/src/state.rs:80` initializes an empty archive map. Startup
(`lib.rs:29–56`) does not hydrate it from durable installation data. `reader_open`
(`ipc.rs:1001–1011`) only looks in that map; `install_start` registers an archive at
lines 1503–1511. Selection metadata and cluster files being persisted is insufficient
without the reopen path. This is a confirmed control-flow gap, not a measured device
failure. It contradicts P2-GATE's restart/offline requirement.

**Close through:** P2-03, P2-05, P3-04, P3-05. Specify startup recovery and lazy archive
reopening from the selected directory and verified on-disk clusters; retain last-known-
good state on interruption. Acceptance must include install → exit → relaunch offline
→ read, with the original source removed/unavailable.

### F03 — P1: the active installer bypasses durable verification and storage guarantees

`install_start` reads cluster ranges with no source-version token (`ipc.rs:1383–1385`),
computes a digest of the bytes it just read (line 1398), and labels each chunk verified
(line 1417) without comparing it to an expected trusted cluster digest. Planning
does verify the local full-file digest, but the later read can observe a changed
source. Self-hashing later bytes does not establish that they match the planned source.

Activation runs individual SQL writes under a mutex at lines 1446–1496, not a single
SQL transaction, and moves the active pointer before archive construction. Failed
follow-up writes or archive creation can leave partially activated state. The path
also writes to `app_data_dir/packs` (line 1358) while recording the chosen destination
root, so the storage plan can describe a different volume from the actual writes.

Every downloaded cluster remains in `local_clusters_raw` (lines 1363–1401), then in
the offline archive. Memory therefore grows with the selected compressed payload,
even though the planner's budget comes from free disk. A large selection can exhaust
memory; no OOM was reproduced in this static pass.

**Close through:** P2-02, P3-02 through P3-05, P9-02/P9-04. Wire the existing verified
object-store/activation abstractions into the real shell path; specify authenticated
cluster/version checks, bounded disk-backed cache, destination resolution, atomic
activation and durable job recovery. Measure architecture §21's 450 MiB Android /
600 MiB desktop reader RSS budgets with a pack larger than available RAM.

### F04 — P1: sidecar import can lose notes or persist unreadable records

`crates/annotations/src/import_flow.rs:280` assigns the first pack binding to every
annotation. Lines 294–297 skip notes if no source pack is present, yet lines 369–384
record the artifact as imported; retry is rejected as duplicate at lines 157–161.
The importer also does not replay the artifact's attachment objects/references.
Portable notes can therefore disappear on a clean installation or bind incorrectly
for multi-pack artifacts.

Lines 299–308 convert arbitrary CBOR selector/style to JSON without domain-shape
validation. SQL requires valid JSON, not a valid SelectorBundle/AnnotationStyle
(`schemas/sqlite/0001_initial.sql:266–267`). Later assembly rejects the shape
(`crates/annotations/src/lib.rs:323–326`) and listing propagates that row's error
(lines 283–287), making the affected Notes page fail repeatedly. An unsigned artifact
requires explicit user confirmation; confirmation does not validate its contents.

Signed envelopes are currently rejected as unknown-key (`import_flow.rs:149–155`),
and no sidecar export command is registered in the shell.

**Close through:** P2-06, P4-03 through P4-05, P4-GATE, P9-01. Preserve source-unavailable
notes, validate every domain payload before committing, bind each note correctly,
restore attachment objects, and connect the trust registry and export. Verify a
signed, multi-pack, all-media round-trip onto a clean installation without source packs.

### F05 — P1: failed voice persistence is reported as success to the recorder

`apps/reader/src/stores/annotationStore.ts:396–399` catches upload failure and resolves
undefined. `AnnotationWorkspace.tsx:248` does not inspect that result. Consequently
`VoiceRecorder.tsx:268–277` clears its captured bytes and playback URL after a failed
upload. The video store has the same catch-and-resolve pattern at lines 488–490.
Voice capture also uses MediaRecorder's default output (`VoiceRecorder.tsx:195`) but
labels every upload `voice_opus`/`audio/ogg` (`annotationStore.ts:377–378`) without
container validation/conversion.

**Close through:** P4-02/P4-05, P9-04. Retain recordings and review state until durable
success; expose explicit upload success/failure and support retry. Bind actual media
format to the asset contract and verify disk-full, IPC failure and permission changes.

### F06 — P1: article resources, local Ask and several required surfaces are unconnected

ArticleContent permits `enzime-resource://` resources, but the Tauri builder registers
no custom resource protocol (`apps/reader/src-tauri/src/lib.rs:28–89`), and the UI
records the missing rewriting at `ArticleContent.tsx:21`. Complete offline rendering
is unproven. The iframe does omit scripts/same-origin and has restrictive CSP; no
native-bridge exploit was confirmed by this review.

`WllamaInferenceBackend` requires an injected bridge (`crates/inference-wllama/src/backend.rs:51`), but no production
transport is composed. A test ScriptedBridge **does** exist
(`crates/inference-wllama/tests/stream_translation.rs:53`); the old claim of no
implementation anywhere was inaccurate. `crates/retrieval/src/lib.rs` is empty.
AskService contains orchestration and citation checks, but its production retriever,
context builder, inference host and UI are missing. Startup has no Ask/inference
commands; the router has no Ask/Memory/Settings/Setup routes. Bootstrap nevertheless
advertises Ask enabled (`crates/application/src/bootstrap.rs:43`).

**Close through:** P2-04, P5-02/P5-04 through P5-07, P6-02 through P6-04, P8-01/P8-02/P8-05.
Connect the resource boundary and production providers, then prove model loading,
offline cited completion, cancellation, exact citation navigation, and deletion from
retrieval. Do not reopen LiteRT or a second GGUF engine: AMD-2 specifies wllama3.

### F07 — P1: customer ownership and recovery are not operational yet

`crates/revenuecat-adapter/src/lib.rs` is empty. No entitlement refresh/restore,
backup/restore or sidecar export commands are registered. BackupRestorer delegates
actual snapshot import to a port (`crates/backup/src/restore.rs:168`) whose identified
implementation is test-only; Android document restore returns an error at lines 216–219.
Standalone entitlement policy and backup libraries are not yet connected to customer
ownership and recovery flows.

**Close through:** P7-02 through P7-07 and P7-GATE. Connect the production database/
object snapshot adapter, platform destinations, controller and customer restore flow.
Reconcile global RevenueCat authority with architecture A-09's signed offline ownership
contract explicitly: define event-to-grant provenance, verification, refresh, revocation
and outage behavior. A processor callback must not directly grant a local capability.

### F08 — P1: artifact provenance and release evidence do not identify a tested candidate

The historical manifest's `gitCommit` is `660286c5…`
(`dist/release-manifest-v1.0.52684.json:9`), while its source archive's Git PAX comment
is `7afc5dfa…`. Both commits exist and their trees differ. Hash agreement proves the
listed bytes are intact, not that they were built from the asserted source.

The v1.6.55832 AppImage was introduced in `b08e417`; `a727a75` adds annotations without
updating the artifact or version. No v1.6 manifest/SBOM binds it to the current source.
Its SHA-256 is `402cd1dbee127f3942d456cc0d0b7441d6f5271b2f25181d1f898f6c8cd94a4d`.
Other platform artifacts remain v1.0. Historical notes call them an unwired shell,
and record unsigned Linux/Windows bundles. The required MSIX and signed manifest
are absent. This review did not retest signing or infer infrastructure availability.

No `dist/rubric-runs/` evidence package exists. The designated local
`~/Admin-Manual/LOGS/EnZIME-v4/` contains logs/stills but the filename inventory found
no MP4/WebM/evidence ZIP. This is absence in the inspected locations, not a claim
that no recording exists anywhere. M1's built-AppImage and operator evidence remains
unverified. Its rubric is not the full commercial seven-story rubric.

**Close through:** P9, P10-02 through P10-GATE. Recover any genuine prior evidence
from its recorded host, otherwise run the complete candidate gauntlet when ready.
Produce one frozen source/version/artifact/evidence set. `dist/NOT-A-RELEASE.md` was
missing at assessment start and is added by this documentation pass; existing binaries
and historical manifests are preserved.

### F09 — P2: the planning and commercial delivery contracts need reconciliation

P5-02 still named the retired LiteRT path; P10-05 pointed to a nonexistent hyphenated
rubric. Those references are corrected by this pass. M1-STATE names a Stitch project
and ten captured screens, while tracked `STITCH/generation-evidence.json` still has
null project/screen IDs and all ten screens unverified. Recover actual tool/capture
evidence before treating either narrative as current; this is not an assumed service
outage or a requirement for the operator to repair the tooling.

`tools/release/` is empty; `tests/evidence/capability-evidence.json`, `DOCS/PRIVACY.md`
and `DOCS/THIRD-PARTY-NOTICES.md` are absent. The historical SBOM has 421 Cargo
components and does not demonstrate coverage of the shipped JavaScript dependencies.
Support, updates, pricing, content/model distribution terms and store acceptance
have not been established by this repository's release evidence.

**Close through:** P0/P1 gates, P8-04/P8-05, P10-01/P10-03/P10-05. Complete the architecture
amendment and exact entity/checklist contracts before coder dispatch. Put measured
behavior and immutable evidence in the capability registry; preserve historical
markers without converting them into fresh acceptance claims.

### F10 — P1 at assessment: native Library styling and minimum-width layout fail

**Bounded repair verified in v1.8.21560**, source `b574ed8`: pure route styles,
shell-owned shared tokens/rules and a stable navigation grid. Native evidence
confirms loading/search re-renders, Home→Library return, minimum-width layout
and scrolling. [View the indexed native evidence](../dist/rubric-runs/v1.8.21560-linux-styles-20260913T100046Z/RESULTS.md).
P8-03-STYLE/P8-03-NARROW pass; broader P8/M1/commercial gates remain open. The
original observation below is retained as the explanation for the repair.

The user-requested trial of the v1.6.55832 AppImage (SHA-256
`402cd1dbee127f3942d456cc0d0b7441d6f5271b2f25181d1f898f6c8cd94a4d`)
opened a real native window but displayed default-looking inputs, run-together
empty-state title/detail text and no Library card layout. The operator explicitly
rejected the appearance. Presenting a bootable artifact as a meaningful Kiwix trial
was an overstatement: the fresh catalog is empty and F01's local-only composition
remains. The shell's Ready indicator is not evidence that those journeys work.

CodeGraph identifies the matching defects in the assessed source:

- `LibraryStyles` (`LibraryView.tsx:528–531`) mutates `libStyleMounted` during
  render and returns null on subsequent renders, removing its own stylesheet.
  Normal store-driven re-renders trigger this independently of development
  StrictMode. The obsolete reset helper does not repair production behavior.
- `HomeStyles` is mounted only by `HomeView:32`. Library and Reader import
  shared `StatusBanner`, button and loading markup, but leaving Home unmounts
  the stylesheet. Visiting Home first therefore cannot repair sibling routes.
- Shared `--home-*` variables live only on `.home-surface` (`shared.tsx:36–50`),
  so persistent stylesheet ownership also needs shared token inheritance.

**Close through:** P8-03-STYLE, frozen in architecture §15.6 and the bounded native
protocol in TEST_RUBRIC. Restore the existing design with pure stylesheet renderers
and shell-owned shared rules/tokens. Verify loading, typing, route round-trips and
resizing in a newly stamped native artifact. Record the outcome in BUILD-RECIPES
and indexed evidence. This repair does not close the catalog, download or restart
findings, and does not substitute for P8-04's broader design reconciliation.

Both the original app and the v1.8.21560 runtime logged
`GStreamer element appsink not found`. Media playback
is unverified by the appearance trial; the warning needs diagnosis in the media
journey before its acceptance. The isolated original window capture is retained at
`~/Admin-Manual/.staging/core-sesh-recovery-20260913/artifact-trial/enzime-reader-v1.6.55832-window.png`.

The first repair build v1.7.21541 passed desktop styling but exposed an existing
48rem shell-media conflict at the configured 720 CSS pixel minimum: a visible rail
lost its named grid area and displaced content. v1.8.21560 retains the existing rail,
allows the content track to shrink, and contains the search label. Both the failure
and successful replacement have recorded native evidence. The current app still
has empty catalog data, disabled/unconnected destinations, an inert global search
control and Ready/M1 copy that must be reconciled with actual product capability.
Restoring styles is not the commercial design/interaction acceptance pass.

## 3. Commercial release path

This is an ordered plan, not a new parallel checklist. CHECKLIST.md remains the task
contract. Architecture and checklist refinement precede code. No calendar promise is
defensible before the integrated loop and actual target-device performance are known.

| Gate | Work and existing task groups | Evidence needed to advance |
|---|---|---|
| 1. Reconciled execution contract | Resolve F01–F10 into architecture/entity table and exact CHECKLIST rows; reconcile Stitch, M1 and full commercial rubric; P0/P1/P8-04/P8-05 | Every implementation task has exact paths/entities/signatures and repeatable acceptance; all contradictions have recorded dispositions |
| 2. Defensible DynDon reader | Remote catalog/manifest acquisition → chosen storage-fit subset → verified durable job → correct destination → restart hydration → offline resource rendering; P2/P3 | Built AppImage shows oversized-source subset acquisition, interruption/resume, source-unavailable restart and readable offline resources; then equivalent Android/Windows behavior |
| 3. Durable user work | Fix sidecar/recording loss, all-media export/import, source-unavailable notes and production backup/restore; P4/P7 | Clean-install signed round-trip and encrypted restore preserve every note/asset; failures preserve original data and allow retry |
| 4. Complete knowledge and ownership product | Production wllama bridge, retrieval, cited Ask, lore lifecycle, RevenueCat normalization, capability controller, Setup/Settings; P5/P6/P7/P8 | Seven architecture E2E stories run on the target artifacts; no paid claim depends on unregistered commands or test providers |
| 5. Release-quality and commercial readiness | P9 security/performance/a11y/lifecycle; P10-01 disclosures/SBOM; customer offer, support/updates and channel requirements | No unresolved P1 data-loss/security issue; measured §21 budgets, accessibility evidence, verified purchase/restore/outage behavior and complete disclosures |
| 6. One candidate, then commercial publication | P10 manual build/sign/manifest/capability evidence; complete agreed platform matrix; full gauntlet and operator review | Exact source/version matches every binary, archive, SBOM and indexed recording; final approval refers to those immutable hashes; publish only that set |

After gate 1, independent work can run in parallel: acquisition/storage recovery;
annotation portability/backup; and inference/retrieval/UI. One integrator owns shared
IPC registration, SQLite migrations, final version freeze and candidate assembly.
Security/resource-boundary review and signing/toolchain preparation can progress
alongside those streams. CI follows the proven manual recipe and the existing
AMD-4/public-release authorization; it is not the present critical path.

### Customer offer and operating readiness

Use the product's demonstrated value as the offer: a deliberately sized offline
knowledge reserve with durable portable work and cited local assistance. Avoid the
literal claim that the complete Wikipedia fits when only a subset is retained; show
included/excluded content, storage savings and offline availability honestly.

The existing capability model (§13.1) already preserves free reader.core, text notes,
bookmarks, local import and human-readable export. Validate paid packaging around
the defined rich-media, local Ask, lore, encrypted backup and signed-export capabilities;
do not invent a price or subscription commitment in this assessment. Before collecting
payment, demonstrate purchase, restore, long-offline use, refund/revocation semantics,
and preservation of user data. Validate price and willingness to pay with intended
users after the core loop works; track time to first offline article, successful
restart reading, note portability success, repeat use and support incidents.

Commercial operations must identify the supported OS/device matrix, update and
rollback policy, support contact/diagnostic consent, content and model distribution
rights inventory, privacy/data flows, and a generated download/release manifest.
These are deliverables to verify, not claims that legal review or store access has
already occurred.

### Current external channel checks (2026-09-13)

- Google Play currently requires new app/update submissions to target Android 16 /
  API 36 from August 31, 2026. The checked-in Android Gradle file already sets
  `targetSdk = 36` at line 23; this is configuration evidence, not a Play validation.
  Source: [Google Play target API requirements](https://support.google.com/googleplay/android-developer/answer/11926878?hl=en).
- Native Android packages need 16 KB page-size compatibility checks across the
  shipped native libraries and an actual compatible device/emulator run. This audit
  did not certify the binaries. Source: [Android page-size guidance](https://developer.android.com/guide/practices/page-sizes).
- If the publishing account is a personal developer account created after November
  13, 2023, production access requires at least 12 continuously opted-in closed-test
  users for 14 days. Account type/eligibility is unverified here; do not impose this
  schedule on a different account type. Source: [Play testing requirements](https://support.google.com/googleplay/android-developer/answer/14151465?hl=en-GB).

## 4. Immediate next action and evidence discipline

The user's artifact trial added P8-03-STYLE/P8-03-NARROW; both are now verified
within their bounded appearance scope in v1.8.21560. The next product phase is
ARCHITECT: reconcile the DynDon remote-source, storage, activation,
restart and portability contracts against the current composition, then amend the
existing checklist with bounded tasks. Do not dispatch against July's stale in-flight
narrative. The first executable milestone is a durable DynDon reader, not another
packaging attempt or a new inference-engine rewrite.

Keep code presence, observed static defects, historical command results and runtime
acceptance separate. Document dynamic protocols in the rubric/evidence documents;
CHECKLIST Accept clauses remain idempotent end-state assertions. No existing source
file was changed by the initial static assessment. The subsequent stylesheet fix
has its own exact task, native protocol and evidence; it grants no M1/commercial gate.

## 5. Assessment provenance

Core continuity was repaired before dependent project assessment resumed. Canonical
runtime/recovery/deployment sources were recovered from Admin-Manual and verified
`asrock` (`192.168.0.57`); the no-bypass correction is published as Admin-Manual commit
`2d1e044`. Role-scoped sesh was re-executed using verified CodeGraph and local contract/
Git state. Context-MCP's unregistered-project response was optional enrichment, not
the continuity authority or a reason to bypass the restored skill.

CodeGraph supplied the source observations. Its demonstrated Markdown/JSON/config
coverage gaps permitted named contract/manifest reads; the sidecar SQL constraint
was read after its own recorded parser gap. Artifact hashes/archive metadata and Git
history were inspected read-only. Three bounded independent reviews covered security/
user-data integrity, inference/retrieval, and release delivery. External channel facts
above were checked against primary documentation; no third-party dependency CVE audit
or license adjudication was performed.


## 6. Active public-release progress — 2026-10-12

This update supersedes the September immediate-next-action paragraph. Public release is **not yet verified**; the active goal remains the real Kiwix → bounded DynDon subset → offline restart → on-canvas annotation flow.

Completed source tasks are committed and pushed to GitHub branch `codex/reader-dyndon` at https://github.com/rebots-online/EnZIME-v4. Forgejo remains unavailable by explicit operator instruction and was not contacted. Source-mirror commits preserve the existing GitHub dist tree; newly built local artifacts are not claimed as externally published downloads.

- Canonical AnZimmerman Rust now owns ZIM parsing/decoding and supports sparse offline entry providers. The application adapter stays thin. The Python edition has bounded I/O/decode work; the other language parity gates remain pending.
- OI-02 durable catalog: 16 cache tests passed (9 new catalog, 6 disk-range, 1 planner). OI-03 reader integration: final parent native run passed 3 tests, worker TypeScript check and 13 library tests passed. Coverage includes unvisited article, redirect, resource and title search after restart with zero further network calls; transfer allowance and physical storage pressure have regression coverage. These results do not substitute for packaged runtime acceptance.
- Initial annotation scope is ink, typed text, voice recording, photograph and video clip on the article canvas. Automatic transcription is deferred. Real native capture/playback and offline annotation restart remain unverified.
- Sole operator build command is `bash scripts/build-all.sh`: one frozen source/version, concurrent platform lanes, generated current-build manifest and download page with a permanent Windows NSIS row. No platform subset switch. Latest completed attempt is **1.11.63172**, source parent510e7302cee0d04f1b17a3b9bfcb7842cb862ddd; independent manifest verification reports **incomplete, 4/12 artifacts** (NSIS, web, prepared source, SBOM). NSIS is unsigned. Linux produced three packages but collection rejected the wrong expected internal package name; that validator is fixed in subsequent source. Android APK/AAB signatures verified, but collection correctly rejected Tauri's regenerated versionCode1074172 versus frozen100011. Android symbol production also needs DWARF inputs and exact output collection repair. No stale package substitutes were promoted.
- MSI/MSIX executor remains unconfigured. Admin-Manual inventory identifies no native Windows endpoint; read-only SSH attempts to the three documented Proxmox nodes returned no route to host. This does not prove no Windows host exists. Endpoint/access/signing information has been requested while independent work continues.
- Native v1.11.63172 Linux diagnostic did load the live Kiwix catalog and search Football Database. Its plan attempt failed honestly on the real v6.3 archive's absent title-index marker (u64 all-ones). KI-01 specifies a canonical Rust/Python fix, regression fixtures and follow-on language propagation. The diagnostic predates the new offline catalog and is not a full acceptance run. Evidence: `~/Admin-Manual/LOGS/EnZIME-v4/enzime-v1.11.63172-library-diagnostic.mp4` and stills/DOM under `~/Admin-Manual/.staging/enzime/native-trial-v1.11.63172/`.

Next execution sequence: fix KI-01 and Android packaging inputs; commit and push each completed task; run the entire unified build again; verify the actual fresh Linux artifact against the reader rubric, including bounded live source, never-visited retained entry after offline restart, all five annotation modes and saved media replay. Broader public/commercial gates remain platform/device runtime, update/rollback, notes portability/backup, entitlement/payment behavior for paid distribution, licenses/privacy/support and operator acceptance. No store readiness or commercial sign-off is inferred from build/test success.

### Interrupted unified attempt — 2026-10-12 16:20 EDT resume

Attempt1.12.63579 (source d12b258, workspace20261012T141831.680726Z-d12b2582a9c9) stopped during compilation across the chat interruption. Resume observed no build/packaging process, compilation-only log tails and no finalized1.12manifest. Only prepared source/web archives exist in rootdist. Preserve these as interrupted-attempt evidence, not validated platform releases. The canonical version stamps are committed before the next complete-script run; next execution uses a user systemd transient unit so chat lifetime does not own the build process. Forgejo remains untouched.
