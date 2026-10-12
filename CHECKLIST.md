# EnZIME Reader Reboot IV — Fresh Execution Checklist

**Initial state (2026-07-26 historical):** every executable task was unstarted.
**Rule:** each dispatched task includes the full row below.
**Scope:** complete Android, Linux and Windows public release.

Marker workflow is governed by `DOCS/REBOOT-PAK/KICKOFF-PROMPT.md`.

## Release reassessment — 2026-09-13

Current source baseline: `a727a75`, version `1.6.55832`. See
[release readiness and commercial path](DOCS/RELEASE-READINESS-2026-09-13.md).
All 11 phase gates remain unchecked. Historical marks are retained; this static
assessment grants no new runtime acceptance. P5-03's mark records supersession,
not verification of a currently supported second inference engine.

Next phase is ARCHITECT reconciliation, including completion of the commercial
rubric before CODE, followed by bounded checklist refinement:
remote DynDon planning, correct-volume verified installation, crash/restart recovery,
resource rendering and portable user work. Existing task priority is P2/P3, then
P4/P7 recovery, with P5/P6/P8 integration in parallel after shared contracts freeze;
P9 and P10 complete the commercial gate. The linked report maps F01–F10 to exact
current source evidence and these task groups. It is not a second task checklist.

Runtime/device procedures belong in the evidence rubric. Refined task Accept clauses
must be idempotent end-state assertions; the July live/manual gates below require
that reconciliation before redispatch. Existing binaries are provisional under
`dist/NOT-A-RELEASE.md`.

The operator then tried v1.6.55832 and rejected its unstyled Library. Execute the
bounded P8-03-STYLE and P8-03-NARROW repairs added two tasks to the 75-row historical
snapshot in the report. Both passed their bounded native appearance protocol in
v1.8.21560; see the indexed evidence under dist/rubric-runs. This does not grant
M1 acceptance or change the remote catalog/download/restart frontier.

## Phase 0 — Closed-world launch gate

- [ ] **P0-01 Verify pak integrity**
  - Allowed files: `DOCS/REBOOT-PAK/PAK-MANIFEST.sha256`, audit evidence.
  - Do: verify every manifest entry and archive extraction containment.
  - Verify: manifest command exits zero; no unlisted pak file exists.
  - Accept: exact pak bytes are known before implementation.

- [ ] **P0-02 Attest vendor inputs**
  - Allowed files: `DOCS/REBOOT-PAK/VENDOR-MANIFEST.md`, vendor audit evidence.
  - Do: verify Tauri `v2.11.1`, LiteRT-LM `1ff29cc`, and the supplied
    AnZIMmerman source revision/tree digest/licence.
  - Verify: all three vendor records are `VERIFIED`; trees are clean.
  - Accept: no floating or unknown vendor input remains.

- [ ] **P0-03 Pass cold-start audit**
  - Allowed files: `DOCS/REBOOT-PAK/audits/cold-start-audit.md`.
  - Do: execute every row from a clean extraction containing only pak/vendor.
  - Verify: result block says PASS with pak/vendor digests.
  - Accept: no previous repository, path, transcript or secret is required.

- [ ] **P0-04 Validate architecture registries**
  - Allowed files: `DOCS/REBOOT-PAK/ARCHITECTURE.md`, audit evidence.
  - Do: parse SQL, command, event, component, route and binding identifiers.
  - Verify: no duplicate/undefined identifier; SQLite SQL parses.
  - Accept: architecture is internally executable.

### Phase 0 gate

- [ ] **P0-GATE Authorize implementation**
  - Allowed files: audit evidence only.
  - Do: review P0 evidence.
  - Verify: P0-01 through P0-04 independently pass.
  - Accept: application source creation is authorized.

## Phase 1 — Repository and contracts

- ✅ **P1-01 Create locked monorepo skeleton** — glm-4.7/crush 2026-07-27; orchestrator-attested (read Cargo.toml, apps/reader/src-tauri/Cargo.toml, rust-toolchain.toml against §5 + §26 AMD-1; `cargo metadata` loads 20 packages)
  - Allowed files: root manifests/toolchain files and empty directories in
    `ARCHITECTURE.md` §5.
  - Do: create workspace with no default platform feature.
  - Verify: package/Cargo metadata loads; tree matches §5.
  - Accept: no production feature module or placeholder implementation exists.

- ✅ **P1-02 Implement domain identities** — glm-4.7/crush 2026-07-27; orchestrator-attested (read crates/domain/src/identity.rs against §7.1; 12 tests pass)
  - Allowed files: `crates/domain/src/identity.rs`, module export, tests.
  - Do: implement `PackId`, `PackVersionId`, `ArticleKey`, `EntryKey`,
    `ContentDigest`, `DigestAlgorithm` with validation.
  - Verify: canonical serialization/invalid digest tests pass.
  - Accept: IDs contain no path or URL.

- [X] **P1-03 Implement ZIM ports** — glm-4.7/crush 2026-07-27 — ports/zim.rs written; awaiting semantic attestation
  - Allowed files: `crates/ports/src/zim.rs`, module export, tests.
  - Do: implement exact §7 public types and `ZimEngine`/`ZimArchive`.
  - Verify: public signature snapshot equals architecture.
  - Accept: port imports no AnZIMmerman/provider crate.

- [X] **P1-04 Implement catalog and transfer ports** — glm-4.7/crush 2026-07-27 — ports/catalog.rs, ports/transfer.rs, domain/pack.rs written; awaiting semantic attestation
  - Allowed files: `crates/ports/src/catalog.rs`,
    `crates/ports/src/transfer.rs`, exports, tests.
  - Do: implement exact §8 traits/types/errors.
  - Verify: signature/serialization snapshots pass.
  - Accept: application can plan without HTTP/file concrete imports.

- [X] **P1-05 Implement annotation and signature ports** — glm-4.7/crush 2026-07-27 — domain/annotation.rs, domain/trust.rs, ports/signature.rs written; traits relocated to ports by FIX-01; awaiting semantic attestation
  - Allowed files: domain annotation/trust modules, ports signature module,
    tests.
  - Do: implement §9 types/traits and validation.
  - Verify: selector bounds and signature-domain tests pass.
  - Accept: multi-selector and trust states are complete.

- [X] **P1-06 Implement inference, embedding and retrieval ports** — glm-4.7/crush 2026-07-27 — ports/inference.rs, ports/embedding.rs, ports/retrieval.rs, domain/model.rs written; BackendKind/ModelFormat repaired by FIX-02; awaiting semantic attestation
  - Allowed files: inference/embedding/retrieval port/domain modules, tests.
  - Do: implement exact §§10–11 types and traits.
  - Verify: signature/serde snapshot tests pass.
  - Accept: no runtime provider imported.

- [X] **P1-07 Implement lore and entitlement ports** — glm-4.7/crush 2026-07-27 — domain/lore.rs, domain/entitlement.rs, ports/lore.rs, ports/entitlement.rs written; awaiting semantic attestation
  - Allowed files: lore/entitlement domain/port modules, tests.
  - Do: implement exact §§12–13 entities, traits and errors.
  - Verify: state/policy unit tests pass.
  - Accept: candidate cannot become accepted memory implicitly.

- [X] **P1-08 Apply SQLite initial migration** — glm-4.7/crush 2026-07-27 — schemas/sqlite/0001_initial.sql + storage-sqlite/migration.rs written; awaiting semantic attestation
  - Allowed files: `schemas/sqlite/0001_initial.sql`,
    `crates/storage-sqlite/`, tests.
  - Do: transcribe §6 executable SQL exactly and implement migration runner.
  - Verify: empty migration, FK check, index/trigger inventory pass.
  - Accept: schema identifiers equal §16.4.

- [/] **P1-09 Generate TypeScript contracts**
  - Allowed files: `packages/contracts/`, `tools/contract-gen/`, tests.
  - Do: generate exact §14 entities/DTOs/command/event/route unions.
  - Verify: Rust↔TS round-trip and registry equality pass.
  - Accept: generated files are checked in and reproducible.

- [X] **P1-10 Create complete platform shells** — assigned to glm-4.7/crush 2026-07-27 as APP-01 (Tauri 2 shell + React frontend, single composition per AMD-1)
  - Allowed files: `apps/reader/`, platform composition manifests, smoke tests.
  - Do: create React/Tauri shell for Android/Linux/Windows with real bootstrap
    error handling.
  - Verify: exact target `cargo check` and UI build pass separately.
  - Accept: no desktop default; forbidden dependency graph tests pass.

### Phase 1 gate

- [ ] **P1-GATE Contract foundation**
  - Allowed files: evidence only.
  - Do: independently rerun P1 contract/schema/platform tests.
  - Verify: all pass on clean checkout.
  - Accept: no feature implementation begins on drifting contracts.

## Phase 2 — Real ZIM vertical slice

- ✅ **P2-01 Implement `AnZimmermanZimEngine`** — glm-4.7/crush; orchestrator-attested 2026-07-28 by reading crates/zim-anzimmerman/src/cluster.rs `extract_blob`: blob_count is derived from `first_offset / 4 - 1` per the openZIM cluster format — NOT the upstream break-on-zero-offset bug (recorded in DOCS/ANZIMMERMANLIB-UPSTREAM-FIXES.md finding 1). §7 hard requirements hold: alignment check, index bounds, inverted-offset detection, MAX_BLOB_BYTES allocation cap and end-past-buffer check, each returning a declared ZimError variant rather than panicking.
  - Allowed files: `crates/zim-anzimmerman/`, ZIM fixtures/tests.
  - Do: bind exact `ZimEngine`/`ZimArchive` methods to vendored source.
  - Verify: all §7.4 legal and malicious fixtures pass.
  - Accept: no panic, unbounded allocation or unsupported silent result.

- ✅ **P2-02 Implement object store** — glm-4.7/crush; orchestrator-attested 2026-07-28 by reading crates/object-store/src/store.rs `commit()`: the `fs::rename` at step 8 is the ONLY path bytes enter the store, reachable solely after length match, SHA-256 recomputed FROM DISK matching, and format-verifier pass; every failure branch routes to `quarantine_and_fail`. A-07 enforced by control flow, not by comment. Honest scope note in the source: §6.3 steps 9–10 (metadata txn, active pointer) are declared application-layer rather than faked.
  - Allowed files: `crates/object-store/`, tests.
  - Do: staging/hash/fsync/atomic-rename/quarantine algorithm from §6.3.
  - Verify: forced interruption/corruption/duplicate object tests pass.
  - Accept: unverified bytes cannot enter active objects.

- [/] **P2-03 Implement pack import/activation**
  - Allowed files: application library service, SQLite repositories, tests.
  - Do: local import plan, verify, `install_transactions`, active pointer.
  - Verify: valid import activates; corrupt import quarantines; previous stays.
  - Accept: A-04 and A-07 hold.

- [/] **P2-04 Implement safe article renderer**
  - Allowed files: reader service, sanitizer, scoped protocol, renderer tests.
  - Do: §7.3 pipeline/CSP/message validation.
  - Verify: scripts/network/native bridge blocked; resources/links work.
  - Accept: untrusted article cannot invoke privileged code.

- [/] **P2-05 Implement reader search/navigation**
  - Allowed files: application reader, reader repositories, tests.
  - Do: title search, open/main/internal navigation, history/position/bookmark.
  - Verify: real fixture search/navigation/restart tests pass.
  - Accept: reader works offline with no model.

- [/] **P2-06 Implement text annotation vertical slice**
  - Allowed files: annotation service/resolver/repositories, tests.
  - Do: create/update/delete/restore/list highlight/comment with selector bundle.
  - Verify: restart/re-anchor/ambiguity/source-unavailable tests pass.
  - Accept: annotation outlives current article bytes.

- [/] **P2-07 Implement minimal complete Reader UI**
  - Allowed files: shell/home/library/reader components and stores used by slice.
  - Do: real import, pack card, search, reader, bookmark, highlight/comment UI.
  - Verify: component and Tauri E2E flow passes desktop/Android.
  - Accept: every visible control works; no placeholder captions.

### Phase 2 gate

- [ ] **P2-GATE Airplane reader**
  - Allowed files: evidence only.
  - Do: execute `E2E-01-airplane-reader` on physical Android and desktop.
  - Verify: all steps pass after restart.
  - Accept: first complete daily-use loop is defensible.

## Phase 3 — Catalog, storage and durable transfer

- [X] **P3-01 Implement catalog adapters**
  - Allowed files: catalog adapter crates, catalog repositories, fixtures/tests.
  - Do: OpenZIM XML, EnZIME JSON, local manifest normalization/trust.
  - Verify: signed/unsigned/malformed/offline fixture matrix passes.
  - Accept: UI sees only canonical `CatalogEntry`.

- ✅ **P3-02 Implement HTTP and file transfer adapters** — glm-4.7/crush; orchestrator-attested 2026-07-28 by reading crates/transfer-http/src/lib.rs: §8.5 honoured — an HTTP 200 answering a Range request is REJECTED, never buffered as a chunk; it is reported as SourceChanged when an ETag was supplied and InvalidRangeResponse otherwise. 206 is required, Content-Range is required, and the returned range is verified to match the requested one.
  - Allowed files: transfer adapter crates, tests.
  - Do: bounded probe, valid ranges, full/local streams, retry/source-change.
  - Verify: ignored range, oversized response, ETag change tests pass.
  - Accept: no `200` range response is buffered as a chunk.

- [/] **P3-03 Implement install planner**
  - Allowed files: application library planner, storage repository, tests.
  - Do: calculate final/temp/retained/free impact and verification plan.
  - Verify: fit/temporary-shortfall/untrusted-source matrices pass.
  - Accept: `InstallPlan` contains every Download Plan field.

- [/] **P3-04 Implement resumable download worker**
  - Allowed files: application job worker, download repositories, tests.
  - Do: persisted chunks, pause/resume/cancel/retry/verify.
  - Verify: process-kill resume repeats no verified chunk.
  - Accept: state transitions obey SQL guard.

- [/] **P3-05 Implement atomic candidate activation**
  - Allowed files: install service/repositories/object store, tests.
  - Do: staged verification then active pointer and retained prior.
  - Verify: failures at each transaction phase preserve last active.
  - Accept: `E2E-02-interrupted-download` passes.

- [/] **P3-06 Implement Library UI**
  - Allowed files: Library components/store/services/styles/tests from §16.6.
  - Do: installed/catalog/download/storage views and plan dialog.
  - Verify: §24 bindings and state/accessibility matrix pass.
  - Accept: install cannot start without valid explicit plan.

### Phase 3 gate

- [ ] **P3-GATE Durable acquisition**
  - Allowed files: evidence only.
  - Do: run catalog→plan→interrupt→resume→verify→activate on all platforms.
  - Verify: evidence names source/pack digest and state.
  - Accept: no last-known-good loss.

## Phase 4 — Rich annotations, Notes and sidecars

- [/] **P4-01 Implement notebook/tag/layer repositories**
  - Allowed files: storage repositories/application annotation service/tests.
  - Do: CRUD/query hierarchy/tags/layer visibility.
  - Verify: FK/delete/idempotence/filter tests pass.
  - Accept: cross-pack Note filters use canonical tables.

- [/] **P4-02 Implement ink and voice assets**
  - Allowed files: annotation service/object store/platform audio/tests.
  - Do: chunked asset upload, digest commit, Opus voice and SVG ink.
  - Verify: size/hash/cancel/permission/process-death tests pass.
  - Accept: no partial asset is referenced.

- ✅ **P4-03 Implement canonical sidecar codec/signing** — glm-4.7/crush; orchestrator-attested 2026-07-28 by reading crates/sidecar/src/codec.rs and import.rs: signing and verification both prefix the payload with SIGNATURE_DOMAIN (`domain_prefix`), and verification rejects an envelope whose `signed_domain` differs — so a signature valid under one domain string does not verify under another, which is the §9.5 domain-separation requirement.
  - Allowed files: sidecar/signature crates, CDDL, golden fixtures/tests.
  - Do: deterministic CBOR, artifact ID, Ed25519 domain sign/verify.
  - Verify: golden bytes and domain-separation tests pass.
  - Accept: repeated export with same canonical input is byte-identical except
    declared creation-time input.

- [/] **P4-04 Implement sidecar plan/import/export**
  - Allowed files: application sidecar service, repositories, tests.
  - Do: choose/plan/conflict/idempotent import and explicit export.
  - Verify: valid/unsigned/invalid/duplicate/divergent matrix passes.
  - Accept: invalid signatures never activate.

- [/] **P4-05 Implement Annotation Workspace and Notes UI**
  - Allowed files: named annotation/notes components/stores/services/tests.
  - Do: every §15 component/action/state.
  - Verify: §24 binding, accessibility and source-unavailable tests pass.
  - Accept: `E2E-03-sidecar-portability` passes.

### Phase 4 gate

- [ ] **P4-GATE Portable annotations**
  - Allowed files: evidence only.
  - Do: signed cross-install round-trip with all annotation types.
  - Verify: source placement and layer isolation pass.
  - Accept: no app-private-only user work remains.

## Phase 5 — Local inference and cited Ask

- [/] **P5-01 Implement model catalog/install lifecycle**
  - Allowed files: model domain/repository/application/object-store tests.
  - Do: model manifests, plan, verify, install/select/remove.
  - Verify: corrupt/incompatible/insufficient-space/active-session tests pass.
  - Accept: no automatic model download.

- [/] **P5-02 Implement wllama3 backend and production bridge** — RETURNED INCOMPLETE; reassessed 2026-09-13. The adapter exists under `crates/inference-wllama/`, but no production WebView transport is composed in the app. `ScriptedBridge` exists in `tests/stream_translation.rs` as a test implementation, so the former claim of no implementation anywhere was too broad. The shell registers no inference/Ask command. Architecture AMD-2 retains wllama3 as the sole engine. Refine exact bridge/host/frontend entities and paths before redispatch; a historical compile claim is not evidence of real model execution.
  - Allowed files: `crates/inference-wllama/`, vendored adapter boundary, tests; production host/frontend paths must be specified by the architecture reconciliation before dispatch.
  - Do: exact backend/session trait with streaming/cancel/health/model validation.
  - Verify: certified model fixture runs on Android and desktop.
  - Accept: runtime failure maps to `InferenceError`.

- ✅ **P5-03 Implement desktop GGUF backend** — SUPERSEDED by §26 AMD-2, closed 2026-07-28. The GGUF *engine* (`crates/inference-gguf`) is retired: wllama3 is the sole local inference engine on every platform, so a separate desktop backend would violate AMD-1's single-composition rule. GGUF survives only as the model FILE FORMAT wllama3 consumes (`ModelFormat::Gguf`). No code is owed; building this row would build a retired engine.
  - Allowed files: `crates/inference-gguf/`, desktop composition/tests.
  - Do: exact backend/session trait for Linux/Windows.
  - Verify: desktop model fixture streams/cancels; Android graph excludes crate.
  - Accept: contracts do not expose provider-specific fields.

- [/] **P5-04 Implement indexing and embeddings**
  - Allowed files: retrieval/storage/inference embedding modules/tests.
  - Do: document/chunk projection, FTS sync, embedding spaces/vectors/rebuild.
  - Verify: deterministic chunking, FTS and model-replacement tests pass.
  - Accept: user primary data is not stored only in an index.

- [/] **P5-05 Implement hybrid retrieval/context**
  - Allowed files: retrieval crate/application ask/tests.
  - Do: lexical/vector merge, dedupe, rerank, diversity, §11.3 budget.
  - Verify: golden retrieval/context fixtures pass.
  - Accept: evidence list is bounded and locators exact.

- [/] **P5-06 Implement cited conversation service**
  - Allowed files: application ask, repositories, tests.
  - Do: conversation/message/run, stream events, cancel, citation validation.
  - Verify: known/unknown/zero-citation/failure/restart tests pass.
  - Accept: assistant success cannot contain invalid citation records.

- [/] **P5-07 Implement Ask UI**
  - Allowed files: named Ask components/store/services/styles/tests.
  - Do: conversations, scope, backend status, stream/cancel/citation navigation,
    model-not-installed complete state.
  - Verify: §24 bindings and UI matrix pass.
  - Accept: `E2E-04-local-ask` passes offline.

### Phase 5 gate

- [ ] **P5-GATE Grounded local conversation**
  - Allowed files: evidence only.
  - Do: physical Android and desktop cited Ask.
  - Verify: all citations open exact passages; network denial records zero calls.
  - Accept: model removal preserves conversation/citations.

## Phase 6 — Lore memory

- ✅ **P6-01 Implement lore repositories/policy** — glm-4.7/crush; orchestrator-attested 2026-07-28 by reading crates/lore/src/service.rs `accept()` and `extract_candidates()`: promotion happens only via explicit `accept()` or a user-configured `PromotionPolicy` rule recorded with `source='policy'` and `rule.reason` stored as the revision's `correction_reason` for Memory Centre display — exactly what §12.3 authorises. Default constructor is `without_promotion`. A-08 (model output never silently becomes lore) holds.
  - Allowed files: lore/storage crates and tests.
  - Do: evidence/candidate/memory/revision/evidence links and forbidden classes.
  - Verify: FK, revision conflict, forbidden promotion tests pass.
  - Accept: accepted memory requires explicit/policy evidence.

- [/] **P6-02 Implement extraction/consolidation**
  - Allowed files: lore service/retrieval projection/tests.
  - Do: opt-in candidate extraction, duplicate merge and correction proposal.
  - Verify: source quotation and sensitive inference rejection fixtures pass.
  - Accept: consolidation cannot accept/delete/rewrite user corrections.

- [/] **P6-03 Implement correction/pin/delete/retrieval**
  - Allowed files: lore/retrieval services/repositories/tests.
  - Do: revisioned correction, pin, tombstone deletion and projection removal.
  - Verify: `CT-LORE-002` and revision/evidence tests pass.
  - Accept: deleted text is absent from Ask/diagnostics.

- [/] **P6-04 Implement Memory UI**
  - Allowed files: named Memory components/store/services/styles/tests.
  - Do: inbox, graph, timeline, detail, accept/reject/correct/pin/delete.
  - Verify: §24 bindings and accessibility state matrix pass.
  - Accept: `E2E-05-lore-correction` passes.

### Phase 6 gate

- [ ] **P6-GATE Governed memory**
  - Allowed files: evidence only.
  - Do: independently inspect candidate-to-delete lifecycle.
  - Verify: evidence and historical correction remain; deleted text unavailable.
  - Accept: model output is never memory by decree.

## Phase 7 — Entitlement, backup and diagnostics

- ✅ **P7-01 Implement signed entitlement codec/policy** — glm-4.7/crush; orchestrator-attested 2026-07-28 by reading crates/entitlement/src/policy.rs `decide_inner()`: `READER_CORE` returns allowed BEFORE the artifact, clock or revocation list is consulted; perpetual grants bypass expiry citing A-09 explicitly; denial paths preserve existing user data. A-09 structurally enforced.
  - Allowed files: entitlement/signature/storage crates, CDDL, tests.
  - Do: deterministic artifact verify and capability decisions.
  - Verify: valid/perpetual/term/expired/invalid/revoked matrix passes offline.
  - Accept: reader core and user export never denied.

- [/] **P7-02 Implement RevenueCat adapter**
  - Allowed files: RevenueCat adapter/platform compositions/tests.
  - Do: normalize refresh/restore events into provider events and signed grants.
  - Verify: provider outage/malformed event/replay tests pass.
  - Accept: domain/UI import no RevenueCat SDK type.

- [/] **P7-03 Implement coupon ledger**
  - Allowed files: entitlement service/storage/tests.
  - Do: signed coupon verify, receipt derivation, local single redemption and
    later reconciliation.
  - Verify: duplicate/cross-device/expired/invalid fixtures pass.
  - Accept: reconciliation cannot revoke unrelated ownership/data.

- [X] **P7-04 Implement backup creation**
  - Allowed files: backup/object-store/platform/signature/tests.
  - Do: versioned manifest, optional encryption, atomic export and verification.
  - Verify: wrong destination/disk full/wrong passphrase/interruption tests pass.
  - Accept: backup is marked verified only after read-back validation.

- [/] **P7-05 Implement restore plan/commit**
  - Allowed files: backup/application/storage/tests.
  - Do: validate/migrate/preview conflicts/stage/atomic restore.
  - Verify: all conflict policies and source-absent representation pass.
  - Accept: failed restore leaves current data unchanged.

- ✅ **P7-06 Implement diagnostics/privacy controls** — glm-4.7/crush; orchestrator-attested 2026-07-28 by reading crates/diagnostics/src/audit.rs and bundle.rs: redaction is enforced by TYPE, not by discipline — the detail_json builder accepts only redacted typed values (`redacted_text`, `redacted_path`, `redacted_url`), so a caller cannot pass a raw secret or absolute path by forgetting to redact. §18.6 satisfied: the bundle is inspectable by the user before it is shared.
  - Allowed files: diagnostics/application/settings UI/tests and privacy docs.
  - Do: redacted inspectable bundle, export/delete controls, explicit sample opt-in.
  - Verify: secret/path/content redaction fixture passes.
  - Accept: support evidence needs no broad private content.

- [/] **P7-07 Implement Settings UI**
  - Allowed files: named Settings components/store/services/styles/tests.
  - Do: network/compute/accessibility/entitlement/backup/diagnostics flows.
  - Verify: §24 bindings and UI matrix pass.
  - Accept: `E2E-06` and `E2E-07` pass.

### Phase 7 gate

- [ ] **P7-GATE Ownership and recovery**
  - Allowed files: evidence only.
  - Do: long-offline entitlement restart and clean-install encrypted restore.
  - Verify: capabilities/data behave exactly by grant and source availability.
  - Accept: vendor/network failure cannot revoke perpetual local use.

## Phase 8 — Complete UI and setup

- [/] **P8-01 Implement Home UI**
  - Allowed files: named Home components/store/services/styles/tests.
  - Do: populated/empty/loading/offline/error/readiness states.
  - Verify: §24 bindings, keyboard/screen-reader/mobile tests pass.
  - Accept: each task/readiness action routes to a real complete flow.

- [/] **P8-02 Implement Setup UI**
  - Allowed files: named Setup components/store/services/styles/tests.
  - Do: probe/path/model/complete flows including Skip model.
  - Verify: each path reaches usable Home/Library; process restart resumes.
  - Accept: AI is not required or deceptively implied.

- [/] **P8-03 Apply Aurora Glass system**
  - Allowed files: design tokens, UI primitives, component styles/tests.
  - Do: implement exact §15.6 tokens and usage rules on all screens.
  - Verify: token inventory and contrast/scale/reduced-motion tests pass.
  - Accept: long article text is never on low-contrast glass.

- ✅ **P8-03-STYLE Repair stylesheet lifetime in the native Library** — TypeScript exits 0; independent source review passed; v1.8.21560 native loading/query/Home→Library captures reviewed (2026-09-13).
  - Contract: `DOCS/ARCHITECTURE.md` §15.6 stylesheet lifetime correction,
    2026-09-13. Preserve the existing Aurora Glass design and data behavior.
  - Allowed files: `apps/reader/src/routes/library/LibraryView.tsx`,
    `apps/reader/src/components/home/shared.tsx`,
    `apps/reader/src/components/shell/AppShell.tsx`,
    `apps/reader/src/components/home/HomeView.tsx`; this task's marker only.
  - Do: make `LibraryStyles(): ReactNode` always return its existing style
    element; remove `libStyleMounted`, `__resetLibraryStylesForDev` and their
    obsolete comments, and correct once-per-process comments.
    Make `HomeStyles(): ReactNode` always return its existing style element;
    remove `styleMounted` and the state/effect gate, and remove `useState` if
    unused (keep `useEffect` for `useOnlineSubscription`). In `STYLE_TEXT`,
    put the unchanged `--home-*` variable block under `.ez-shell, .home-surface`
    and retain box/layout declarations under `.home-surface` alone. Import
    `HomeStyles` in `AppShell` from `../home/shared` and mount it once directly
    inside `.ez-shell`; remove its import/mount from `HomeView`. Do not change
    colors, layout geometry, copy, data, IPC, navigation or other styles.
  - Verify: `cd apps/reader && pnpm exec tsc --noEmit`; independent review of
    the four files against this exact contract (no version bump from this check).
  - Accept: TypeScript exits zero; both style renderers are pure and return
    styles on every render; shared style ownership is in `AppShell` and
    tokens inherit to Library/Reader without imposing Home layout on the shell.
  - Native observation: integrator follows the bounded stylesheet protocol in
    `DOCS/TEST_RUBRIC.md` on a new, frozen-version AppImage. This does not mark
    P8-03, M1 or any commercial gate complete.

- ✅ **P8-03-NARROW Preserve the Library at the native minimum width** — TypeScript exits 0; independent CSS review passed; v1.8.21560 native 720 CSS pixel layout/scrolling and desktop restoration observed (2026-09-13).
  - Contract: architecture §15.6 native follow-up, observed in v1.7.21541.
  - Allowed files: `apps/reader/src/styles/global.css`; this task's marker only.
  - Do: in `.ez-shell`, change the content column from `1fr` to
    `minmax(0, 1fr)`. Remove the entire existing `@media (max-width: 48rem)`
    block that changes shell grid areas to one column and hides the rail;
    preserve the default two-column `rail top` / `rail main` grid and the rail.
    Add `min-width: 0` to `.ez-shell__topbar`, `.ez-shell__search`, and
    `.ez-shell__main`. Add `.ez-shell__search > span:last-child` with exactly
    `min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap`.
    Preserve all other tokens, geometry, controls and responsive Library rules.
  - Verify: `cd apps/reader && pnpm exec tsc --noEmit`; independent review of
    the CSS against this exact contract. Do not build or stamp from the coder.
  - Accept: typecheck exits zero; rail and route retain named grid areas;
    content/search can shrink; no contradictory rail hiding rule remains.
  - Native observation: repeat TEST_RUBRIC's bounded stylesheet protocol with
    a new stamped AppImage, including the configured 720 CSS pixel minimum.
    Root integrator owns build/version/evidence and publication.

- [/] **P8-04 Reconcile verified Stitch screens** — EVIDENCE RECONCILIATION REQUIRED, reassessed 2026-09-13. Tracked `DOCS/REBOOT-PAK/STITCH/generation-evidence.json` still has null project/screen IDs and ten unverified screens, while M1-STATE names project `9898008898121608042` and captured screens. Recover and verify the actual tool/capture evidence using `DOCS/REBOOT-PAK/STITCH/STITCH-MCP-RUNBOOK.md`; do not infer a service outage or make tool recovery an operator-only prerequisite. Update evidence only from observed results and preserve §24 entity bindings. This static assessment does not certify screen generation.
  - Allowed files: component layout/styles and
    `STITCH/generation-evidence.json`.
  - Do: use only screens with successful `get_screen` evidence as layout input.
  - Verify: entity names and §24 bindings unchanged.
  - Accept: visual drafts improve layout without altering contracts.

- [/] **P8-05 Validate full component binding manifest**
  - Allowed files: contract-gen/binding manifest/tests.
  - Do: validate every §24 row.
  - Verify: tool exits zero; no undeclared component/entity/command/table/store.
  - Accept: UI/backend vocabulary is one contract.

### Phase 8 gate

- [ ] **P8-GATE Ten complete UI surfaces**
  - Allowed files: evidence only.
  - Do: test Home, Library, Reader, Annotation, Ask, Notes, Memory, Settings,
    Setup and Download Plan desktop/mobile.
  - Verify: every state/action is complete; no placeholder.
  - Accept: public UI is release-appropriate.

## Phase 9 — Security, performance and accessibility

- [ ] **P9-01 Complete threat model and parser hardening**
  - Allowed files: threat model, security tests/fuzz targets, fixes.
  - Do: test §18 actors/boundaries and sustained malicious corpora.
  - Verify: no crash/panic/OOM/path escape/native bridge.
  - Accept: all findings resolved or release blocked.

- [ ] **P9-02 Meet performance budgets**
  - Allowed files: performance harness/results and bounded fixes.
  - Do: measure §21 on all reference hardware.
  - Verify: p50/p95/max artifacts meet budgets.
  - Accept: no aspirational/estimated pass.

- [ ] **P9-03 Complete accessibility audit**
  - Allowed files: accessibility tests/report and fixes.
  - Do: WCAG AA, keyboard, screen reader, 200%, reduced motion, touch targets.
  - Verify: automated plus manual report passes every primary flow.
  - Accept: status never colour-only.

- [ ] **P9-04 Complete lifecycle/device matrix**
  - Allowed files: device matrix/results and fixes.
  - Do: process death, reboot, update, low storage, permission revocation,
    offline/metered/thermal/battery cases.
  - Verify: observed states match architecture.
  - Accept: no primary-data loss or false readiness.

### Phase 9 gate

- [ ] **P9-GATE Quality floor**
  - Allowed files: evidence only.
  - Do: independent review of P9 evidence.
  - Verify: no unresolved severity-high issue or failed budget.
  - Accept: release packaging authorized.

## Phase 10 — Release

- [ ] **P10-01 Finalize privacy/notices/licences**
  - Allowed files: `DOCS/PRIVACY.md`, `DOCS/THIRD-PARTY-NOTICES.md`,
    licences/SBOM.
  - Do: complete end-user documents from actual data/network/dependencies.
  - Verify: dependency/licence inventory has no omission.
  - Accept: store-facing documents match implementation.

- [ ] **P10-02 Build signed artifact matrix**
  - Allowed files: release scripts/config/manifests/artifacts.
  - Do: Android AAB/APK, Linux AppImage/DEB, Windows MSIX/installer.
  - Verify: clean install/launch/smoke and signature verification pass.
  - Accept: all artifact hashes enter signed release manifest.

- [ ] **P10-03 Generate SBOM and capability evidence**
  - Allowed files: SBOM, evidence registry, release manifest.
  - Do: generate from clean release commit.
  - Verify: advertised capabilities all `passed`; SBOM matches binaries.
  - Accept: no unproven product claim.

- [ ] **P10-04 Run production placeholder gate**
  - Allowed files: release audit evidence and required fixes.
  - Do: scan production code/strings using §20.3 plus bypass/secret checks.
  - Verify: zero forbidden production result.
  - Accept: no stub, mock path, bypass or unsuitable caption ships.

- [ ] **P10-05 Run complete release rubric**
  - Allowed files: `DOCS/TEST_RUBRIC.md`, evidence.
  - Prerequisite: ARCHITECT/PLAN has completed the commercial rubric, covering all seven architecture §19.4 E2E stories and full release gates, before implementation begins.
  - Do: run that completed commercial rubric from clean installs on the signed candidate artifacts.
  - Verify: all pass on signed candidate artifacts.
  - Accept: candidate is SHIP-READY.

### Final gate

- [ ] **P10-GATE Ship EnZIME Reader 4.0**
  - Allowed files: signed release manifest and immutable evidence.
  - Do: independent verifier reviews source commit, artifacts, SBOM, capability
    evidence, E2E, security, performance, accessibility and recovery.
  - Verify: every checklist task is independently verified; tree clean.
  - Accept: publish exact signed artifacts without post-verification mutation.


## AZR — Canonical AnZimmermanLib Rust consolidation (2026-10-12)

- [ ] **AZR-01 Consolidate original-library Rust implementation and preserve EnZIME adapter**
  - Contract: `DOCS/ARCHITECTURE/anzimmerman-rust-consolidation.md`; the complete executable contract is embedded below so the coder needs no architectural discovery.
  - Allowed reads/edits: root `Cargo.toml`, `Cargo.lock`; `crates/zim-anzimmerman/**`; original-library `rust/**`, `tests/fixtures/rust-consolidation.zim`, `tests/fixtures/rust-consolidation.expected.json`, `README.md`, `ARCHITECTURE.md`, `CHECKLIST.md`; existing `crates/domain/src/identity.rs`, `crates/ports/src/zim.rs` read-only for conversion; this checklist for status. No other application source edits.
  - Do: execute the contract below; move implementation ownership while preserving observable behavior; do not implement remote acquisition or modify other language code in this task.
  - Verify: `cargo test -p anzimmerman && cargo test -p zim-anzimmerman && cargo test -p zim-cluster-cache && cargo check -p reader`.
  - Accept: all three suites pass and reader checks successfully; `library_boundary` validates canonical dependency independence and adapter canonical path; `library_adapter` validates field/error/Unicode/stream parity. No implementation is marked verified before observed success.

### AZR-01 complete coder contract

#### Canonical consolidation specification

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
