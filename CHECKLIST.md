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

- ✅ **AZR-01 Consolidate original-library Rust implementation and preserve EnZIME adapter**
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
| `archive::AnZimmermanZimArchive` | R/src/archive/mod.rs:1 | Public archive wrapper; `Inner` in archive/inner.rs, file helpers in archive/io.rs, layout types/snapshot in archive/layout.rs, library `ZimArchive` implementation in archive/traits.rs |
| `archive::open_archive` | R/src/archive/mod.rs:1 | `pub async fn open_archive(request: &model::ZimOpenRequest) -> Result<AnZimmermanZimArchive, error::ZimError>` |
| `archive::{ZimLayoutEntry,ZimLayoutSnapshot}` | R/src/archive/mod.rs:1 | Definitions in R/src/archive/layout.rs, reexported from archive/mod.rs; exact existing fields with library-owned types; preserve `layout_snapshot(&self) -> Result<ZimLayoutSnapshot,ZimError>` |
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
| canonical conformance | R/tests/conformance.rs:1, R/tests/conformance/{cases_1,cases_2,cases_3}.rs:1 and R/tests/support/mod.rs:1 | Copy existing generic conformance suite/builders, change imports to canonical types; retain adapter suite |
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


## RC — Reader canvas integration prerequisites (2026-10-12)

- ✅ **RC-01 Trusted renderer bridge and canvas highlights** — frontend TypeScript and42 deterministic tests verified; native protocol remains separate.
- ✅ **RC-02 Reliable annotation save, retry and voice MIME** — frontend TypeScript and42 deterministic tests verified; real device media remains separate.

The following embedded contract governs RC-01/02 only. The full goal additionally requires the later all-modality on-canvas work; these prerequisites do not satisfy that goal alone.

# Reader canvas annotation integration

Status: proposed implementation contract, 2026-10-12. No implementation or verification is claimed by this document. Authority: master architecture §§7.3, 9, 18.3; user requirement to download via DynDon and annotate in the reader. Preserve the current reader and annotation UI. This is an integration repair, not a new visual design.

Archive parsing belongs exclusively to the canonical AnZimmermanLib Rust library. `crates/zim-anzimmerman` remains its application adapter. This feature does not introduce a parser or download whole archives.

## Scope and boundaries

The privileged React shell owns annotation persistence and routing. The article remains in an opaque-origin iframe with no Tauri API, same-origin permission, top navigation, forms, popups or downloads. A first-party bridge implements the already specified `RendererMessageV1`. Original archive scripts never execute. Highlight marks appear on the article canvas; existing comments, ink and voice tools remain in the adjacent annotation workspace. This contract does not misrepresent the existing separate ink drawing panel as a freehand overlay on article pixels.

Remote downloads remain native/backend work; CORS is not a reason to weaken the article sandbox. Parent supplies a scoped resource URL builder. It must resolve only installed/registered packs and permitted archive entries. Backend resource delivery must work through the canonical library and bounded DynDon cluster store, with explicit missing-cluster errors. No filesystem paths appear in renderer messages.

## Exact entity table

Line numbers below are observed anchors for existing entities; new files start at line 1. Edits may move anchors, names remain binding.

| Entity | Target | Role and exact contract |
|---|---|---|
| `ReaderResourceContext` | `apps/reader/src/routes/reader/rendererBridge.ts:1` (new) | `{ urlForEntry: (packId: string, entryKey: string) => string; cspSources: readonly string[] }`; trusted parent injection, never populated from article content |
| `RendererMessageV1` | same new file | Discriminated union with `schemaVersion: 1`, `channel: string`; kinds `ready`, `open_article` (`articleKey`, optional `fragment`), `selection_changed` (`selector?: SelectorBundle`), `resource_failed` (`entryKey`, `reason`), `scroll_position` (`scrollAnchor`, `progressFraction`) |
| `RendererParentMessageV1` | same | Version/channel plus kinds `annotations_changed` (`annotations: readonly RendererHighlight[]`), `focus_annotation` (`annotationId: string`) |
| `RendererHighlight` | same | `{ annotationId: string; selector: SelectorBundle; colorToken: string }`; filtered to active article in parent |
| `resolveZimReference` | same | `(articleKey: string, reference: string) => { entryKey: string; fragment?: string } \| undefined`; reject external/active schemes, credentials, backslashes and root escape; support archive namespace, nested paths, Unicode and fragment-only links |
| `validateRendererMessage` | same | `(value: unknown, channel: string, article: { packId: string; articleKey: string }) => RendererMessageV1 \| undefined`; no coercion, bounded strings and safe integer offsets, active article binding |
| `buildRendererBridgeScript` | same | `(channel: string, article: { packId: string; articleKey: string }) => string`; static audited first-party script with JSON-serialized configuration escaped for script terminators |
| `buildSandboxedDoc` | `apps/reader/src/routes/reader/ArticleContent.tsx:151` | Export for tests; `(rawHtml: string, packId: string, articleKey: string, channel: string, nonce: string, resources: ReaderResourceContext) => string` |
| `ArticleContentProps` | same:190 | Existing `article`, `packId`; add required `articleKey`, `resources: ReaderResourceContext`, `annotations: readonly AnnotationRecord[]`, `onSelectionChange: (selection?: SelectorBundle) => void`, `onOpenArticle: (articleKey: string, fragment?: string) => void`; optional `selectedAnnotationId: string` |
| `ReaderView` | `apps/reader/src/routes/reader/ReaderView.tsx:256` | Own selection per pack/article, pass renderer callbacks and actual parent backend resource context; pass selected annotation ID and article-filtered records |
| `ReaderStyles` | same:245 | Pure stylesheet component; remove module-level mounted flag so remount never loses CSS |
| `AnnotationDrawerProps` | `apps/reader/src/routes/reader/AnnotationDrawer.tsx:9` | Add `selection?: SelectorBundle`; forward through `AnnotationWorkspaceForArticle` to existing workspace selection prop |
| `handleCreateComment`, `handleCreateHighlight`, `handleCommitInk`, `handleVoiceCapture`, `handleFilmCapture` | `apps/reader/src/components/annotation/AnnotationWorkspace.tsx:163` | Await real persistence; no success callbacks or clearing before committed success; disable duplicate submits; retain failed media bytes/strokes and committed annotation ID for retry |
| `InkCanvasProps.onCommitSvg` | `apps/reader/src/components/annotation/InkCanvas.tsx:19` | Change to `(svg: string) => Promise<void>`; local busy/error state; clear strokes only on resolution, retain on rejection |
| `VoiceCapture` | `apps/reader/src/components/annotation/VoiceRecorder.tsx:30` | Export `{ bytes: Uint8Array; durationMs: number; mimeType: string }`; used by captureRef and `VoiceRecorderSelfManagedProps.onCapture: (capture: VoiceCapture) => Promise<void>` |
| `useAnnotationStore.create` | `apps/reader/src/stores/annotationStore.ts:250` | Preserve existing return type; committed create is applied and returned before optional notebook refresh; refresh error cannot convert committed create to failure |
| `useAnnotationStore.uploadVoice` | same:371 and its interface declaration | `(annotationId: Ulid, bytes: Uint8Array, durationMs: number, mimeType: string) => Promise<AnnotationRecord \| undefined>`; transmit actual supported MIME through existing `uploadAnnotationAsset` |
| `AnnotationPanel` | `apps/reader/src/components/annotation/AnnotationPanel.tsx:30` | Call `useReducedMotion` once at component top, before empty return; do not invoke hooks inside changing annotation list |
| `reader-canvas.test.tsx` | `apps/reader/src/routes/reader/reader-canvas.test.tsx:1` (new) | DOM integration/sanitizer/message/path/selection fixtures |
| `annotation-save.test.tsx` | `apps/reader/src/components/annotation/annotation-save.test.tsx:1` (new) | Save failures, retry, hooks, MIME and media retained-state integration tests |
| `vitest.config.ts` | `apps/reader/vitest.config.ts:1` (new) | Explicit jsdom test environment and local test includes; new infrastructure, not an existing project runner |

Imports reuse `SelectorBundle`, `AnnotationRecord`, `AnnotationDraft`, and `Ulid` from `services/annotationService.ts`. Do not create conflicting annotation wire types.

## Isolation and resource rules

1. Generate independent cryptographically random nonce and channel per article document. React rerenders or annotation updates do not recreate the document; a new article does. Ignore old-document messages after navigation.
2. DOMParser sanitation removes scripts, frames, objects, embeds, forms, base, refresh/CSP meta, all event attributes, original nonce attributes, active URLs, arbitrary targets and non-stylesheet links. Do not enable scripting on the old regex fallback: parser failure produces a safe plain-text error document.
3. Insert CSP before article content, preserving §18.3 restrictions and adding only `script-src 'nonce-<generated>'` for the injected first-party bridge. `sandbox="allow-scripts"` only. Never add `allow-same-origin` or `allow-top-navigation-by-user-activation`.
4. Resource URL/CSP sources come from trusted `ReaderResourceContext`. A protocol mapping to HTTP on a platform must use an exact scoped source, never `http:`/`https:` wildcards. Strip remote images/media/CSS URLs. Permit bounded passive image data URLs only; reject SVG data URLs unless separately sanitized. Rewrite src, srcset, poster and stylesheet href plus CSS imports/url references; backend stylesheet rewriting may own CSS parsing, but must be specified and verified before claiming resource support. Inline CSS cannot weaken CSP.
5. Internal links retain data entry keys and optional fragments; prevent default navigation in bridge, send validated request to parent. Fragment-only links scroll locally. External links cannot navigate iframe or shell; adding an external-link UX is outside this repair.
6. Parent accepts only messages whose source is the current iframe window, exact channel and schema match. Origin is opaque `null`, which is insufficient authentication alone. Bridge accepts parent messages only from `window.parent` with matching channel/schema. Do not send arbitrary invoke names, paths, or HTML in the protocol.
7. Bound article selection to 16,384 UTF-16 code units, context to 128 each side, entry keys to 4,096 characters, error reasons to 512, and parent highlight updates to 2,000 records. Reject nonfinite/fractional/negative offsets, inverted ranges, wrong article identity, unknown fields capable of altering authority and oversized payloads.

## Selection and highlights

Build a normalized text stream from article text nodes excluding script/style/hidden bridge nodes. Collapse whitespace consistently and retain a mapping from normalized UTF-16 offsets back to DOM node offsets. Capture a selection spanning nodes against that mapping; store exact quote, prefix/suffix, normalized offsets and active article identity. Both capture and replay use the same normalization. Emoji/surrogate pairs must not be split.

Replay first checks offsets against exact quote; otherwise resolve quote plus prefix/suffix. A unique match may render; zero/multiple matches produce unresolved/ambiguous state, never silently select the first occurrence. Use controlled DOM Range marks with fixed allowlisted colors, not raw HTML. Reapplying marks is idempotent: unwrap previous marks, normalize text nodes and rebuild mapping. An annotation update must not reset scroll or clear the user's pending selection. Deletion removes its marks. Clicking a list entry focuses the corresponding mark. Durable records are loaded from the existing store after restart, not reconstructed from DOM marks.

## Save and retry semantics

Creation and asset attachment are distinct committed stages. On successful create, remember its annotation ID for the current draft immediately. Subsequent upload retry reuses that ID; it must not create another empty annotation. Lock edits while upload is in flight. A user modifying a failed draft starts a new operation only after explicit discard/reset of the pending attachment. Failures retain the comment/drawing/recording until successful persistence or explicit discard. This is in-session failure retention; unsaved capture recovery across process termination is not claimed.

Store helpers currently return undefined on failure. Workspace callers must turn that result into a rejected capture promise, because the recorder clears bytes on promise resolution. Existing mutation error UI remains visible. Do not conflate optional notebook-count refresh failure with unsuccessful annotation creation. Apply the same success boundary to ink/voice/video upload and preserve committed record results if a later list refresh fails.

Voice uses an explicitly supported Opus recording MIME selected with `MediaRecorder.isTypeSupported`: prefer `audio/ogg;codecs=opus`, then `audio/webm;codecs=opus`. Pass the recorder's actual MIME with captured bytes. If neither is available, show an unsupported-recording state; do not silently label another codec Opus. Keep existing `voice_opus` asset kind because Opus in Ogg/WebM is its content, with the true container MIME recorded. Native asset handling must accept these MIME forms or return a clear unsupported error. Recording must stop before exceeding the backend 32 MiB asset limit; chunk accounting bounds retained capture bytes. Saved playback requires durable asset resource delivery and is a native integration dependency; transient blob playback alone is insufficient evidence.

## Self-contained checklist insertion text

### RC-01 Trusted renderer bridge and canvas highlights

**Depends:** parent-approved native `ReaderResourceContext` provider backed by AnZimmermanLib/DynDon; this document's entities. No implementation may guess a backend URL or create a second archive parser.

**Files:** `apps/reader/src/routes/reader/rendererBridge.ts`, `ArticleContent.tsx`, `ReaderView.tsx`, `AnnotationDrawer.tsx`, `reader-canvas.test.tsx`; `apps/reader/vitest.config.ts`, `apps/reader/package.json`, root `pnpm-lock.yaml`.

**Do:** Implement the exact entities, isolation/resource rules and normalized capture/replay above. Reuse current reader/annotation layout and existing store/service types. Remove ReaderStyles mount flag. Add `vitest`, `jsdom`, `@testing-library/react` as development test dependencies using the existing pnpm workspace; add `test:reader` script `vitest run --config vitest.config.ts`. Tests exercise exported functions and mounted components with mocked native services; no network, live archive, generated binary or device required. DOM tests must not claim browser sandbox enforcement.

**Verify:** `pnpm --dir apps/reader exec tsc --noEmit`; `pnpm --dir apps/reader run test:reader`.

**Accept:** TypeScript command exits 0; deterministic tests pass for nested/Unicode/fragment paths, hostile URLs and parser failure; source/channel/article mismatch rejection; cross-node and emoji selection; repeated quote ambiguity; highlight save/reload/delete and article switch without stale marks; repeated renderer mounts retain style. Security fixtures assert one authorized bridge, no original scripts/handlers/nonces/targets, exact CSP and sandbox token set. Browser sandbox enforcement remains separate operator protocol below.

### RC-02 Reliable annotation save, retry and voice MIME

**Depends:** RC-01 test runner; existing annotation service and native persistence.

**Files:** `apps/reader/src/components/annotation/AnnotationWorkspace.tsx`, `InkCanvas.tsx`, `VoiceRecorder.tsx`, `VideoRecorder.tsx`, `AnnotationPanel.tsx`, `annotation-save.test.tsx`; `apps/reader/src/stores/annotationStore.ts`, `apps/reader/src/services/annotationService.ts` (actual MIME and the approved independently encoded raw-byte chunk repair).

**Do:** Implement save/retry semantics and exact callback/signature changes above. Propagate async InkCanvas callback through ActiveToolSurface. Keep one pending created annotation ID per active media draft; retry only its failed upload. Preserve committed store records despite auxiliary refresh errors. Call useReducedMotion once at top of AnnotationPanel. Carry MIME in VoiceCapture and uploadVoice; select supported Opus container and enforce capture byte budget. Preserve existing video behavior except correcting premature successful save/clear and retry duplication.

**Verify:** `pnpm --dir apps/reader exec tsc --noEmit`; `pnpm --dir apps/reader run test:reader`.

**Accept:** Both commands exit 0. Tests inject create failure, successful-create/refresh-failure, upload failure and later retry. Comment/strokes/capture survive failure; success clears exactly once; retry uses same annotation ID; second clicks during save do not duplicate; unsupported MIME rejected honestly; actual MIME/bytes reach assetBegin; capture limit prevents oversized retained recording; zero-to-one annotation list transition does not change hook order.

## Operator native verification protocol (not CHECKLIST Accept)

Use a built native artifact and a genuinely downloaded partial ZIM registered by DynDon. Record build/commit, source archive identity and selected disk/RAM budgets; use an isolated app data profile so evidence is reproducible without deleting user data.

1. Open downloaded article with retained image and stylesheet; follow nested internal link then fragment. Verify the app does not reload or lose archive registry. Disconnect network and repeat against retained clusters; missing content must explicitly report unavailable.
2. Select text across formatting nodes; highlight and add comment. Capture canvas marks and notes. Navigate away/back and restart application; verify persisted records and marks retain article identity. Repeat with repeated quote and Unicode.
3. Save ink and Opus voice, restart, reopen and play/view saved assets. Inject a controlled local persistence/upload failure; verify draft retention, visible error and successful retry without duplicate annotation.
4. Load hostile archive fixture containing scripts/events/navigation/forms and forged bridge messages. Verify no original script executes, shell navigation or Tauri calls occur, and trusted selection still works. Prove this on native WebKit/WebView, not solely jsdom.
5. Exercise reader→library→reader and narrow desktop width. Capture build-specific screenshots/video and process memory during reading. Do not claim whole-product release from this bounded protocol.

## Review gates

This document is ready for parent review and checklist insertion; implementation remains unstarted. Resource delivery and persisted media playback must be connected to the actual backend before the goal is complete. Parent must reconcile the nonce bridge extension with master §18.3 explicitly: it authorizes only application-owned script in an opaque origin, never archive script capability.


### RC native resource integration contract clarification

Parent owns native handler implementation in `apps/reader/src-tauri/src/reader_resources.rs` and its registration; frontend work proceeds against this exact contract while native work proceeds independently. Protocol name is `enzime-resource`. Use `convertFileSrc("", "enzime-resource")` as the base (remove trailing slash), then append `/pack/` plus encoded pack ID and separately percent-encoded entry-key segments joined by `/`. Preserve hierarchical slashes so stylesheet-relative URLs resolve inside the pack. Only registered pack identifiers and archive entry keys resolve. Windows/Android generated HTTP(S) custom-protocol origin is allowed explicitly, never a general HTTP source. `ReaderView` constructs `ReaderResourceContext` using imported `convertFileSrc` from `@tauri-apps/api/core`; `cspSources` is the scheme or exact generated origin (for a custom scheme use `enzime-resource:`, for HTTP(S) use URL.origin). This does not require changing readerService. No filesystem paths or whole-file resource bridge. Resource response and stylesheet URL rewriting are native ownership and not verified by frontend unit tests.

The authorized nonce bridge realizes master architecture §7.3 RendererMessageV1; no archive-authored scripts receive permission. RC-01/02 source ownership excludes native files and excludes RC-03 until separately contracted.


## RC-NATIVE — Scoped reader and annotation asset delivery

- ✅ **RC-NATIVE-01 Implement registered-archive resources and durable annotation playback**
  - Allowed reads/edits: `apps/reader/src-tauri/src/reader_resources.rs` (new), its `reader_resources/tests.rs` (new), `apps/reader/src-tauri/src/lib.rs`, `apps/reader/src-tauri/Cargo.toml`; read-only `state.rs`, `ipc/annotations.rs`, `crates/ports/src/zim.rs`, `crates/domain/src/{annotation,identity}.rs`, `crates/storage-sqlite/src/annotations.rs`, `crates/annotations/src/lib.rs`, `crates/object-store/src/{lib,store}.rs`, `tauri.conf.json`. Cargo.lock changes coordinated with integrator.
  - Entities: `reader_resources::register<R: tauri::Runtime>(tauri::Builder<R>) -> tauri::Builder<R>` registers asynchronous `enzime-resource`; `ReaderAnnotationAsset` has camelCase fields annotationAssetId, annotationId, assetKind, mimeType, byteLength:u64, durationMs:Option<u64>, resourceUrl:String; command `annotation_assets_list(State<AppState>, annotation_id:String) -> Result<Vec<ReaderAnnotationAsset>,String>`; `ResourceRoute::{Pack{pack_id:String,entry_key:String}, Annotation{annotation_id:String,asset_id:String}}`; pure `parse_resource_path(&str)->Result<ResourceRoute,String>`; pure `byte_range(Option<&str>,u64)->Result<(u64,u64,bool),String>` returns start/end-exclusive/partial; `passive_mime(&str)->bool`.
  - Do: wrap existing Builder with register and register command in existing generate_handler. Protocol GET/HEAD only, other methods 405. Route `/pack/<packId>/<namespace>/<path...>` parses percent encoding once and validates UTF-8, no traversal/dot segments/backslashes/NUL/empty identifiers; whole encoded payload from older clients may be rejected, not interpreted as filesystem. Clone registered `Arc<dyn ZimArchive>` under archive-map lock, release lock before await, then call `resource(EntryKey(entry_key))`. Unknown pack/resource returns 404; missing cluster reports unavailable, never fetch entire archive.
  - Do: annotation route `/annotation/<annotationId>/<assetId>` checks live nondeleted head with storage `annotation_row`, resolves asset by annotation_assets membership, then opens immutable content-addressed object using `state.annotations.object_store().read(ContentDigest::sha256_from_hex(...))`. Never accept caller hash or filesystem path. Verify file length equals persisted byte_length. Metadata command performs the same live-head check and returns passive assets only with trusted platform-specific URLs (`enzime-resource://localhost/...` Linux/macOS/iOS, `http://enzime-resource.localhost/...` Windows/Android with current useHttpsScheme default false). Current data is single-user local; do not invent foreign ownership rules.
  - Do: support RFC single bytes ranges including suffix/open-ended, reject malformed/multiple/out-of-bounds/zero suffix with416 and Content-Range bytes */length; wholeGET200, validrange206 exact Content-Range and Content-Length, HEAD body empty retaining selected length. File reads seek and read only selected bytes; bound a response to128MiB (existing video maximum) and concurrent handlers to2, release permits on every path. Passive MIME allowlist covers raster/SVG images, audio/video containers, CSS/fonts; deny HTML/JS/XML active documents. Add nosniff, response CSP `default-src 'none'; sandbox`, Accept-Ranges and no credentials/CORS wildcard authority. Return errors without filesystem paths.
  - Do: hierarchical resource URLs preserve CSS-relative URLs/imports naturally. Keep same pack prefix for root-relative CSS URLs/imports by rewriting them to `/pack/<encodedPackId>/...` using a real CSS parser, not regex; reject active/external CSS imports/url references (CSP also denies remote origins). Add `cssparser` dependency at version already resolved in Cargo.lock if needed; keep rewriting module under reader_resources/ and name its public entry `rewrite_stylesheet(css:&str,pack_id:&str)->Result<String,String>`. Do not modify archive parsing. The frontend iframe receives these native resources using its exact-origin CSP.
  - Verify: `cargo test -p reader reader_resources`; `cargo check -p reader`.
  - Accept: meaningful tests pass for Unicode/encoded keys and traversal rejection; unknown/deleted annotation and wrong asset membership; MIME allowlist; exact full/ranged/suffix/HEAD resource bytes; invalid ranges; CSS relative/root-relative/internal/external URLs and imports. Use in-memory SQLite with committed test fixture/object store for access tests, deterministic fake ZimArchive for handler resource tests; no network. Tests cover actual handler routing, not just helpers. Native WebView media playback remains runtime verification, not inferred from compile.

RC-01 schema clarification: `RendererParentMessageV1` additionally includes `focus_fragment { fragment: string }`, bounded to4096 characters. Parent sends the safely decoded route fragment after bridge ready; trusted bridge resolves with getElementById or named-anchor comparison, never selector interpolation. This completes cross-article fragment navigation without additional authority.


## DD-RANGE — Remote DynDon acquisition (2026-10-12)

- ✅ **DD-RANGE-01 Canonical library lazy range reader and streaming cluster decoding**
  - Allowed files: canonical R/src/range/**, R/src/cluster_stream.rs, R/src/{lib,cluster,header}.rs, R/src/archive/** for shared helper factoring, R/tests/{range_access,cluster_stream}.rs and R/tests/range_access/**, R/Cargo.toml, Cargo.lock; existing canonical models/parsers/tests read-only except narrow helper visibility needed. R is defined in embedded contract below.
  - Do: implement the exact library contract below; preserve local APIs, use original canonical parsing, add no app-specific dependencies.
  - Verify: `cargo test -p anzimmerman --test range_access --test cluster_stream`; `cargo test -p anzimmerman`; `cargo test -p zim-anzimmerman`.
  - Accept: all pass; virtual1TiB sparse test proves bounded requests/cache, lazy lookup and blob read; streaming limits and codec/error cases tested.

### DD-RANGE-01 complete coder contract

# DD-RANGE — DynDon remote acquisition through canonical AnZimmermanLib

Date: 2026-10-12. Architecture contract; verification belongs to the checklist.
Canonical library root R = `VENDOR/AnZIMmermanLib-src/AnZimmermanLIB/AnZimmermanLib/rust`.
Native cache root C = `crates/zim-cluster-cache`. Adapter root A = `crates/zim-anzimmerman`.

## Binding design

Reuse canonical Rust AnZimmerman header/directory/cluster parsers. Extend storage access, never add a competing ZIM interpreter. Preserve local reader APIs. Kiwix discovery and HTTPS range transfer run natively; no WebView CORS dependency. A full archive is never acquired implicitly. Metadata and cluster payloads have separate persistent storage, both charged to the user budget. A small fixed page cache and one bounded blob result replace whole-archive pointer/directory maps. Cluster decompression streams to budgeted disk before slicing a blob, so a compressed cluster need not fit in RAM.

## DD-RANGE-01 — Canonical range index and streaming cluster services

### Exact entities (new targets :1 are insertion anchors)

| Entity | File | Contract |
|---|---|---|
| `RandomAccessSource` | R/src/range/source.rs:1 | async_trait Send+Sync: `fn len(&self)->u64`; `async fn read_range(&self,offset:u64,length:u64)->Result<bytes::Bytes,ZimError>`; exact length or error, checked arithmetic, no full-open method |
| `RangeReadLimits` | R/src/range/source.rs:1 | Clone+Debug: `page_bytes:usize`, `cached_pages:usize`, `max_entry_bytes:usize`, `max_mime_bytes:usize`, `max_blob_bytes:u64`, `max_redirect_hops:u8`; Default values 65536,8, existing MAX_ENTRY_READ_BYTES as usize,1048576,33554432,16 |
| `PagedRangeReader` | R/src/range/source.rs:1 | `new(source:Arc<dyn RandomAccessSource>,limits:RangeReadLimits)->Result<Self,ZimError>`; `async fn read_exact(&self,offset:u64,length:usize)->Result<Bytes,ZimError>`; length cap <= max(max_entry_bytes,max_mime_bytes,page_bytes), fixed byte-accounted LRU; `fn len(&self)->u64` |
| `ClusterByteRange` | R/src/range/index.rs:1 | Clone+Copy+Debug+Serialize+Deserialize+Eq+PartialEq `{ start:u64, length:u64 }` |
| `IndexedEntry` | R/src/range/index.rs:1 | Clone+Debug `{ index:u32, key:EntryKey, raw:directory::RawEntry, mime_type:Option<String> }` |
| `RemoteZimIndex` | R/src/range/index.rs:1 | `async fn open(source:Arc<dyn RandomAccessSource>,limits:RangeReadLimits)->Result<Self,ZimError>`; `fn header(&self)->&header::ZimHeader`; `async fn entry_at(&self,index:u32)->Result<IndexedEntry,ZimError>`; `async fn find_entry(&self,key:&EntryKey)->Result<Option<IndexedEntry>,ZimError>`; `async fn cluster_range(&self,index:u32)->Result<ClusterByteRange,ZimError>`; `async fn resolve_entry(&self,key:&EntryKey,max_hops:u8)->Result<IndexedEntry,ZimError>`; `async fn main_entry(&self)->Result<Option<IndexedEntry>,ZimError>` |
| `ClusterBlobSource` | R/src/range/source.rs:1 | async_trait Send+Sync: `async fn read_blob(&self,cluster_index:u32,range:ClusterByteRange,blob_index:u32,maximum_bytes:u64)->Result<Bytes,ZimError>`; store/acquisition injection, no format interpretation |
| `RangeZimArchive` | R/src/range/archive.rs:1 | `async fn open(index:Arc<RemoteZimIndex>,blobs:Arc<dyn ClusterBlobSource>,metadata:ZimMetadata,limits:RangeReadLimits)->Result<Self,ZimError>`; implements canonical model::ZimArchive; `fn index(&self)->&Arc<RemoteZimIndex>` |
| `ClusterDecodeLimits` | R/src/cluster_stream.rs:1 | Clone+Copy+Debug `{ maximum_output_bytes:u64, maximum_blob_bytes:u64, maximum_decoder_window_bytes:u64 }`; Default 2147483648,33554432,67108864 |
| `DecodedCluster` | R/src/cluster_stream.rs:1 | Clone+Copy+Debug `{ length:u64, offset_width:u8 }` |
| `decode_cluster_to` | R/src/cluster_stream.rs:1 | `pub fn decode_cluster_to(input:&mut dyn std::io::Read,output:&mut dyn std::io::Write,limits:ClusterDecodeLimits)->Result<DecodedCluster,ZimError>`; reads compression tag itself, streams using64KiB buffer, returns decoded length/offset width |
| `read_cluster_blob` | R/src/cluster_stream.rs:1 | `pub fn read_cluster_blob(input:&mut (impl std::io::Read+std::io::Seek),decoded:DecodedCluster,blob_index:u32,maximum_bytes:u64)->Result<Bytes,ZimError>`; reads only first/table-selected offsets and requested blob |
| range exports | R/src/range/mod.rs:1 and R/src/lib.rs | Export all above types; public `range` and `cluster_stream` modules |
| `header::cluster_end` | R/src/header.rs:1 | `pub fn cluster_end(header:&ZimHeader,start:u64,next_cluster:Option<u64>,source_len:u64)->Result<u64,ZimError>`; shared canonical boundary used by local and range readers |
| common cluster helpers | R/src/cluster.rs | Factor shared compression-tag/offset validation into `pub(crate) fn cluster_encoding(tag:u8)->Result<(u8,u8),ZimError>` and `pub(crate) fn validate_blob_offsets(start:u64,end:u64,table_bytes:u64,total_bytes:u64,maximum_bytes:u64)->Result<(),ZimError>`; reuse both existing in-memory and streaming paths |
| range tests | R/tests/range_access.rs:1 and R/tests/range_access/** | Synthetic sparse ranged source + deterministic library fixture; no HTTP dependency |
| streaming tests | R/tests/cluster_stream.rs:1 | Real codec and offset tests; synthetic bounded reader/writer; temp files |

### Required library behavior

`RemoteZimIndex::open` reads 80 bytes via canonical parse_header/validate_header_offsets, validates pointer-table ends with checked count*8+offset <= source length and checksum bounds, reads MIME list incrementally with1MiB ceiling. It does not read all pointer tables/entries or cluster payloads. MIME parsing stops at its actual terminator, not a region containing unrelated bytes. Pointer pages are lazy; metadata cache capacity at default is512KiB plus bounded MIME and entry records. Limit fields validated to prevent zero sizes and unreasonable custom allocations; page_bytes1..65536, cached_pages1..16, max_entry_bytes<=65536, max_mime_bytes<=1048576, max_blob_bytes<=33554432, max_redirect_hops<=32. Do not reserve based on archive entry/cluster count.

`entry_at` reads one u64 pointer then at most max_entry_bytes (clamped to valid archive extent) and invokes canonical directory::parse_entry. Validate returned article cluster index and MIME index. Retain numeric index even for special/deleted entries. Key is namespace byte plus slash plus decoded URL. `find_entry` uses binary search over URL/path pointer ordering and compares UTF-8 bytes, not locale/case-folded strings. `resolve_entry` tracks at most max_hops indices, fails cycles, retains targets by original numeric indices. `main_entry` follows the header main-page index and redirects, no whole directory lookup. `cluster_range` reads pointer[index] and pointer[index+1] when present, then calls shared canonical `header::cluster_end(header:&ZimHeader,start:u64,next_cluster:Option<u64>,source_len:u64)->Result<u64,ZimError>`. The end is the earliest valid boundary strictly greater than start among next_cluster, nonzero path_ptr_pos, title_ptr_pos, cluster_ptr_pos, mime_list_pos, checksum_pos and source_len. Reject next_cluster<=start and any nonzero metadata boundary beyond source_len; reject empty ranges. Existing local archive Inner::cluster_byte_range must delegate to this helper too. This prevents the last cluster swallowing following pointer/directory metadata regions. Header section starts are conservative known boundaries; decoded codecs must stop at frame end and not interpret trailing metadata as another frame. Do not assume logical directory entries are physically contiguous. Test fixtures put clusters both before and after pointer/directory regions, with/without checksum.

`RangeZimArchive::open` takes catalog metadata as seed, overwrites UUID from parsed header and main_page from resolved header entry. Unknown article/media counts remain0 rather than full enumeration. It reads no payload during open. Entry/article/resource calls use index + blob provider and existing canonical classification/projection/digest behavior. Reuse canonical archive classification helpers by factoring visibility within R; do not duplicate classification regex/rules. Resources in namespace C can be HTML or other MIME types: classification follows MIME, not A-only assumptions. `entries()` returns a lazy stream, one indexed record at a time. `search_titles` scans sequentially with bounded top(offset+limit) hits; clamp limit<=200 and offset<=10000; never collect all matching records. It may be slow for huge archives but memory is bounded and the operation is cancellable by dropping its future. `verify_structure` validates accessed metadata only and truthfully reports checked_entries; it must not claim whole-archive/payload verification. Offline misses propagate from source/blob provider, not synthetic empty content.

Streaming decoder factors canonical codec semantics, with no app decoder. Codes0/1 uncompressed,2 supports original TypeScript/Python zlib-wrapped DEFLATE plus existing Rust raw-DEFLATE compatibility,5 Zstandard; add XZ/LZMA codec4 using `xz2 = "0.1"` with bounded decoder memory, and reject unsupported code3 explicitly. For codec2, inspect the first two payload bytes: valid zlib CMF/FLG (method8, window<=7, header divisible by31) selects flate2 ZlibDecoder and verifies trailer; reject preset dictionary unless supplied by format (none here). Otherwise use existing DeflateDecoder compatibility. Do not silently fall back to raw decoding after a recognized zlib stream fails checksum. Both streaming and existing in-memory functions share this selection; tests include independently produced original Python zlib.compress bytes, checked-in Rust raw fixtures, bad wrapped checksum and truncated streams. Existing Rust fixture builder strips the zlib header/trailer, so its passing tests alone do not establish real upstream framing. Validate compression bits; bit0x10 selects64-bit offsets; unknown high bits error. Enforce decoder memory window using codec-specific configuration, not just output byte counting (Zstd window cap and liblzma memory limit). If xz2 interface cannot enforce memory limit use its stream API memory limit; no unbounded default decoder. Output limit checked BEFORE each write. `read_cluster_blob` validates first offset divisible by width and at least two offsets, offset table <= total output, blob index bounds and selected endpoints; then reads only selected blob capped by32MiB. A blob larger than supported renderer limit fails explicitly; never allocates it first. Existing in-memory APIs retain signatures and tests, sharing tag/offset validation. No calls to full-buffer helper from range read.

### Library test gate

`cargo test -p anzimmerman --test range_access --test cluster_stream` plus `cargo test -p anzimmerman` and `cargo test -p zim-anzimmerman`.
Tests must assert a virtual1TiB archive with valid tiny sparse metadata opens/looks up/reads chosen article while no request exceeds65536 bytes (larger entry helper request internally pages), total requests bytes bounded independently of virtual size, and no full source read exists. Use explicit source request log and peak retained page counter; inspect actual reader cache count, not guessed process RSS. Include Unicode byte order, redirect chain/cycle, bad index/count/overflow, malformed/truncated MIME and entry, exact/short source reads, concurrent reads <= configured cache capacity, last cluster boundaries, namespace C HTML/resource distinction, lazy stream and bounded search result handling. Streaming tests cover none/zlib/zstd/xz,32/64bit offsets, compressed bombs/output limit before write, decoder window cap, truncated/inverted offsets, oversized blobs, and comparison with existing uncompressed/codec conformance fixtures. No network or huge real allocation in tests.

Library task is implementable independently. Native source/provider and application installation are successor tasks and do not change these signatures without explicit contract update.


- ✅ **DD-RANGE-02 Native durable range source, disk clusters and bounded DynDon planning**
  - Do/Allowed files: exact DD-RANGE-02 contract embedded below. Depends on01 signatures.
  - Verify: `cargo test -p transfer-http`; `cargo test -p zim-cluster-cache --test disk_range`; `cargo test -p zim-cluster-cache`.
  - Accept: exact range/source identity errors, quota accounting, planner equivalence and offline restart tests pass.

- [X] **DD-RANGE-03 Integrate catalog planning, install, offline reopen and reader adapter** — implementation checkpoint; acceptance remains partial (offline entry-index gap below).
  - Do/Allowed files: exact DD-RANGE-03 contract embedded below. Depends on01/02.
  - Verify: `cargo test -p zim-anzimmerman --test range_adapter`; `cargo test -p reader dyndon`; `cargo check -p reader`; frontend TypeScript check.
  - Accept: actual native integration test proves bounded remote acquisition, destination placement, restart/offline article+annotation identity and failure reporting; compilation alone is insufficient.

### DD-RANGE-02 and03 complete coder contracts


## DD-RANGE-02 — Native disk source and durable cluster store

Depends on DD-RANGE-01 signatures; can implement HTTP validator tests concurrently. Own C/src/disk/{mod,source,store,manifest,plan}.rs and C/tests/disk_range.rs; C/Cargo.toml; crates/transfer-http/src/{lib,parse,limit}.rs and tests/range_identity.rs; crates/ports/src/transfer.rs only for pinned-size API compatibility if needed. Canonical parsing is read-only.

| Entity | Target | Exact contract |
|---|---|---|
| `SourceIdentity` | C/src/disk/manifest.rs:1 | Serialize+Deserialize+Clone+Debug `{ uri:String, size_bytes:u64, etag:Option<String>, last_modified:Option<String>, zim_uuid:String }` |
| `DiskPackPolicy` | C/src/disk/manifest.rs:1 | Serialize+Deserialize+Clone+Debug `{ disk_budget_bytes:u64, metadata_budget_bytes:u64, workspace_budget_bytes:u64, offline:bool }` |
| `DiskPackManifest` | C/src/disk/manifest.rs:1 | Serialize+Deserialize+Clone+Debug `{ format_version:u32, pack_id:String, pack_version_id:String, identity:SourceIdentity, policy:DiskPackPolicy, metadata:anzimmerman::model::ZimMetadata, main_page:Option<String>, state:String }`; format_version1 |
| `DiskPackStore` | C/src/disk/store.rs:1 | `async fn create(root:PathBuf,manifest:DiskPackManifest,source:Arc<dyn ports::transfer::TransferSource>)->Result<Arc<Self>,String>`; `async fn reopen(root:PathBuf,source:Option<Arc<dyn TransferSource>>)->Result<Arc<Self>,String>`; `async fn ensure_cluster(&self,index:u32,range:ClusterByteRange,pinned:bool)->Result<PathBuf,String>`; `async fn set_offline(&self,offline:bool)->Result<(),String>`; `async fn used_bytes(&self)->Result<u64,String>`; implements canonical ClusterBlobSource |
| `DiskRangeSource` | C/src/disk/source.rs:1 | `pub fn new(store:Arc<DiskPackStore>)->Self`; implements canonical RandomAccessSource; metadata page reads only |
| `open_disk_archive` | C/src/disk/mod.rs:1 | `pub async fn open_disk_archive(store:Arc<DiskPackStore>)->Result<Arc<dyn anzimmerman::model::ZimArchive>,String>`; construct lazy index + store blobprovider + manifest seed |
| `PlanSummary` | C/src/disk/plan.rs:1 | Serialize+Deserialize+Clone+Debug `{ selected_count:u64, selected_bytes:u64, skipped_count:u64, metadata_reserved_bytes:u64, workspace_reserved_bytes:u64, disk_budget_bytes:u64 }` |
| `plan_disk_fit` | C/src/disk/plan.rs:1 | `pub async fn plan_disk_fit(store:Arc<DiskPackStore>,index:Arc<RemoteZimIndex>,intent:dyndon::SelectionIntent,pinned_clusters:&[u32])->Result<PlanSummary,String>` |
| `install_selected` | C/src/disk/plan.rs:1 | `pub async fn install_selected(store:Arc<DiskPackStore>)->Result<PlanSummary,String>`; streams selected rows, never selected vector |
| native exports | C/src/lib.rs | `pub mod disk;` reexports public above types |

`DiskPackStore` owns root/manifest and a private SQLite database `pack-index.sqlite` with bounded cache_size -2048, temp_store FILE, journal_mode DELETE, journal_size_limit1048576. Database schema v1: `clusters(cluster_index INTEGER PRIMARY KEY,start_byte INTEGER NOT NULL,byte_length INTEGER NOT NULL,priority INTEGER NOT NULL DEFAULT1,article_count INTEGER NOT NULL DEFAULT0,domain TEXT,topic TEXT,selected INTEGER NOT NULL DEFAULT0,pinned INTEGER NOT NULL DEFAULT0,state TEXT NOT NULL DEFAULT 'missing',stored_sha256 TEXT,last_used INTEGER NOT NULL DEFAULT0)`; `pages(page_index INTEGER PRIMARY KEY,byte_length INTEGER NOT NULL,stored_sha256 TEXT NOT NULL)`; `jobs(job_id TEXT PRIMARY KEY,state TEXT NOT NULL,bytes_stored INTEGER NOT NULL DEFAULT0,error TEXT)`; `policy(key TEXT PRIMARY KEY,value TEXT NOT NULL)`. All arithmetic checked and u64->SQLite i64 conversions rejected on overflow. Selection/availability persists in database, not large manifest JSON. Manifest is small atomic JSON. Paths numeric local identifiers; no remote path components become filesystem paths. Existing app DB remains authoritative for library listing; this DB is pack-local acquisition/index state.

Metadata source reads aligned64KiB pages, final page clamped to source length, cached at `pages/<pageIndex>` atomically with local hash. One source read may touch several pages but returned length capped to1MiB. No `open_full`, no sparse file with apparent full archive size. Metadata quota includes page files, index database and journaling; enforce remaining allowance before growth, page cache evicts unpinned pages LRU if necessary. Pin pages needed to resolve installed entries/main-page redirects and discovered resources; do not evict offline metadata required for retained articles. Planning pages not referenced by installed entries are evictable. Bootstrap pin records in policy/db, not an unbounded in-memory set. Metadata over-budget fails with `budget_exceeded`, never silently expands to full source.

Cluster files are `clusters/<index>.bin`; only fully length-checked streamed `.part` files are renamed. Incremental SHA256 recorded as `stored_sha256`, meaning local stored-integrity only. Never label a partial pack as full-artifact digest verified. Source identity includes strong ETag plus length; no expected ETag silently dropped. For servers without strong ETag, support Last-Modified with If-Unmodified-Since and pinned length, explicitly weaker source consistency; revalidate header UUID and source length before new cluster acquisition. If neither validator is present, fail ranged installation with source_identity_unavailable; browser/manual complete-file import is explicit fallback. URI must remain HTTPS through redirects for remote Kiwix. Local synthetic HTTP tests may use127.0.0.1.

Transport must send Accept-Encoding identity, reject nonidentity encoded ranges, reject200 for ranged request, reject missing/mismatched pinned ETag, bad Content-Range start/end/total or short/excess response. Persist final resolved source URL. Extend HttpTransferSource with an inherent `open_pinned_range(&self,uri:&str,range:TransferRange,identity:&TransferProbe)->Result<TransferByteStream,TransferError>` for Last-Modified/total checks if trait compatibility requires keeping open_range unchanged; DiskPackStore still accepts trait source, so provide `PinnedTransferSource` wrapper in C/src/disk/source.rs implementing TransferSource and validating pinned probe before each range for generic providers. Strong ETag expected passed for every range. HTTP provider exact response validation owns total; generic provider reprobe rejects size changes. Never interpret an outage as empty content.

At most one cluster acquisition/decode per store is in flight, using tokio async Mutex/semaphore; join duplicate requests under this lock. Metadata cache remains independently bounded. `.part` interrupted state never becomes available; reopening deletes/quarantines only its own known .part records after checking path, preserves complete pinned clusters. In-flight job resets to paused with bytes from complete clusters; retry resumes complete-cluster boundary. Store writes progress after each completed cluster. Existing complete file hashes checked lazily on use; corruption marks unavailable and online repair is atomic.

`read_blob` calls ensure_cluster, then canonical decode_cluster_to into `workspace/<index>.decoded` on blocking pool. Bound decoded disk output by min(policy.workspace_budget_bytes,canonical maximum_output_bytes), read only requested blob using canonical read_cluster_blob. Keep at most one decoded file and remove it after blob extraction; buffers max64KiB plus32MiB result and codec window cap64MiB. Decode workspace is pre-reserved in total disk budget, not additive. On error clean only own staging file. Main-page cluster remains pinned permanently; explicit user reinstall/remove is outside this scope. New on-demand article/resource cluster may evict nonpinned least-recently-used payloads to fit payload partition; document that evicted content is unavailable offline. Never evict annotation objects (outside pack store), main-page payload or required main-page metadata.

`plan_disk_fit` stores candidate cluster ranges in SQLite in pages of<=4096 rows, using RemoteZimIndex::cluster_range; never reads all entries for article counts or allocates full manifest. Unknown domain/topic are None and article_count0; no fabricated semantic selection. Catalog-level subject does not become per-cluster subject. Native candidates currently priority1 unless persisted explicit metadata supplies ranking. Compute domain filter, breadth slot aggregate order and depth tiers globally on disk matching dyndon::plan_fit semantics; SQL NULL grouping and stable tie-break must match Rust None ordering. Read eligible candidates ORDER BY priority DESC,article_count DESC,cluster_index ASC in4096-row batches; call existing dyndon::plan_fit with empty domain filters/no breadth/depth (already globally applied), remaining byte budget. Record selected results before next batch; this preserves stable greedy fit across batch boundaries. Pinned main cluster charged once before other candidates. Never place all selected IDs in memory. Differential tests compare database planner and plan_fit for generated small manifests across domains/breadth/depth/ties/oversized skips; one large synthetic set verifies batch bound. The private SQLite stores all candidates on disk; database size charged to metadata budget.

Policy requires total budget >=16MiB and <= verified free bytes minus safety margin. Reserve metadata=min(64MiB,budget/8), workspace=min(512MiB,budget/4); payload is remainder. A main cluster or decoded required article exceeding partitions gives an actionable budget error, not full download. Defaults may be overridden only through a reviewed policy update; available total storage is not permission to consume it all.

Verification: `cargo test -p transfer-http`; `cargo test -p zim-cluster-cache --test disk_range`; existing `cargo test -p zim-cluster-cache`. Tests exercise actual local HTTP handler responses206/200/changed/missingETag/changedlength/redirect/short/excess; disk restart offline mainpage/article/resource; corrupt file; interrupted .part; quota and LRU; immutable pinned main page; planner equivalence; recorded downloaded bytes far below virtual archive size. No external Kiwix availability needed for acceptance.

## DD-RANGE-03 — Thin adapter and reader installation integration

Depends on01/02. Own A/src/{lib,range_adapter}.rs and A/tests/range_adapter.rs; apps/reader/src-tauri/src/{ipc,state,lib}.rs plus new ipc/dyndon.rs and tests/dyndon_install.rs; apps/reader/src-tauri/Cargo.toml; frontend existing IPC contract and LibraryView budget field only after coordinating with RC owner. RC-NATIVE owns resource protocol registration; do not edit its module.

| Entity | Target | Contract |
|---|---|---|
| `RangeArchiveAdapter` | A/src/range_adapter.rs:1 | Wrapper `{ inner:Arc<dyn anzimmerman::model::ZimArchive>, metadata:ports::zim::ZimMetadata }`; `pub fn new(inner:Arc<dyn anzimmerman::model::ZimArchive>)->Self`; implements ports::zim::ZimArchive with existing canonical conversions |
| `DynDonInstallRequest` | reader/src/ipc/dyndon.rs:1 | Deserialize `{ catalog_entry_id:String,destination_storage_root_id:String,budget_bytes:u64,selection_profile_id:Option<String> }` |
| `DynDonPlanRecord` | reader/src/ipc/dyndon.rs:1 | Serialize+Deserialize `{ install_plan_id:String,pack_id:String,pack_version_id:String,store_relative_path:String,destination_storage_root_id:String,summary:PlanSummary }`; no directory/selected vector |
| `prepare_dyndon_install` | reader/src/ipc/dyndon.rs:1 | `pub async fn prepare_dyndon_install(state:&AppState,request:DynDonInstallRequest)->Result<InstallPlanDto,String>` |
| `run_dyndon_install` | reader/src/ipc/dyndon.rs:1 | `pub async fn run_dyndon_install(state:&AppState,install_plan_id:&str)->Result<DownloadJobDto,String>` |
| `restore_dyndon_archives` | reader/src/ipc/dyndon.rs:1 | `pub async fn restore_dyndon_archives(state:&AppState)->Result<(),String>`; offline startup, per-pack diagnostics |
| existing `install_plan`, `install_start` | reader/src/ipc.rs | Delegate to above; add optional `budget_bytes:Option<u64>` arg to install_plan retaining current callers; default min(1GiB,free_after_margin), never all free storage |
| existing `reader_open`, `reader_article` | reader/src/ipc.rs | Preserve public output; Arc archive registration uses RangeArchiveAdapter |

`prepare` resolves actual selected storage_root.platform_locator (not app_data default), live free space, current catalog entry from Kiwix native client/cache. Native Kiwix source metadata may lack SHA; retain unknown honestly, do not hashURL into expected artifact digest. If existing SQL CHECK mandates a digest, use schema migration adding nullable upstream digest/separate source identity rather than bogus64hex; exact migration belongs integration owner and must update read/write types/tests consistently. Derive pack_id from stable catalog ID, pack_version_id from pinned source UUID+length+validator identity hash explicitly named source identity (not artifact content hash). Persist store under `packs/<packVersionId>/`; plan JSON stored inside pack root and app selection revision contains only bounded identity/store reference, not full directory. Reopen plan by ID from persisted reference after restart, not only memory HashMap.

Open remote index first, resolve header main-page redirect to actual article, pin its cluster. If no valid article main page return actionable unsupported_main_page unless explicit user-selected starting entry supplied in future. Plan remaining clusters via store, reserve quotas. Do not declare main-page resources installed until actually fetched. Install selected payloads streams sequentially with durable job progress; failures surface failed/paused and retain complete clusters. Open range archive and request main article before marking completed; discover its passive CSS/image/font resources using existing reader resource handling and persist requested resources on use. Mainpage/resource closure must be bounded by budget; unavailable resources explicitly report missing with readable article preserved. CSS recursive imports through RC-NATIVE use same archive adapter and store quotas.

App DB pack/version/location writes occur transactionally only after manifest/index and main article are readable. Verification state is unverified/source_pinned/local_integrity according to actual schema migration; never claim upstream full SHA. Register Arc adapter under pack_id after releasing DB locks. `restore_dyndon_archives` scans active app DB locations and opens store metadata offline; no network/no eager payload decode, registers every valid archive, reports damaged pack independently without preventing unrelated packs. Async startup invokes restore before library declares ready; no mutex guard held across await. Retry can continue stored job without duplicate pack_version/location rows.

Frontend library must expose budget in MiB/GiB and send bytes; show planned partial download versus full archive size, progress based on retained bytes, cancel/failure states and offline availability honestly. Existing browsing-to-selection navigation remains. No browser launch is necessary for native CORS; system browser fallback is optional explicit full-file path only after navigation reaches actual HTTPS download URL, not substitute for ranged DynDon. Do not auto-download full archive on range rejection.

Verification: `cargo test -p zim-anzimmerman --test range_adapter`; `cargo test -p reader dyndon`; `cargo check -p reader`; existing frontend TypeScript check. Tests use synthetic HTTP range server+temporary actual SQLite/storage root, exercise plan->install->read->newAppState offline reopen->read, assert target root used, budgets/progress/source trust, duplicate retry idempotence, persisted plans, article resource lookup and errors. Persist an annotation via existing annotations command path and verify same pack/article/content identity after restart in integration test; no fake annotation storage. Runtime packaged native reader/catalog/annotation interaction remains separate operator evidence and must be run by parent before goal completion.

### DD-RANGE-02 approved source pinning clarification (2026-10-12)

`DiskRangeSource::new(store:Arc<DiskPackStore>)->Self` pins every metadata page it reads, including upgrading an existing evictable page on a disk hit. Add `DiskRangeSource::new_for_planning(store:Arc<DiskPackStore>)->Self` for unpinned, evictable planning metadata. `plan_disk_fit` opens its own lazy RemoteZimIndex with the planning source; the supplied index provides bounded header identity only. DD03 uses the default pinned source for bootstrap/main-page and archive access; this preserves metadata needed for offline retained content without pinning a whole archive's cluster table during planning. Persistent page pins/LRU live in SQLite policy rows, never an unbounded in-memory set.

Add default async `TransferSource::open_pinned_range(&self,uri:&str,range:TransferRange,identity:&TransferProbe)->Result<TransferByteStream,TransferError>`: reprobe and reject changed size/validator, then forward expected strong ETag to open_range. HttpTransferSource overrides this method with request If-Range/If-Unmodified-Since plus exact Content-Range total/encoding/validator validation. This keeps generic injected sources usable without downcasting and preserves existing open_range callers. Disk source PinnedTransferSource delegates this pinned method and does not silently discard the validator.

Native cache src/lib.rs exports edits authorized. Read-only DynDon planner source and its exported models are allowed to reproduce existing semantics in on-disk batching; canonical format parser source remains untouched.


### RC-01/02 approved implementation clarifications and verification (2026-10-12)

`RendererMessageV1` additionally admits `annotation_resolution { annotationId: string; state: "resolved" | "ambiguous" | "orphaned" }`. The identifier is nonempty and at most128 characters. Parent validates source/channel/schema and accepts it only for the current article's supplied annotation IDs. `ArticleContent` presents this as ephemeral **Annotation placement** status and never overwrites the durable native resolution. Article-level annotations without a text quote require no text mark and are omitted from renderer highlight updates. Tests cover ambiguous placement, unknown IDs, article changes, and unchanged durable records.

RC-02 authorizes the existing `uploadAnnotationAsset` in `apps/reader/src/services/annotationService.ts` to split the raw Uint8Array into independently base64-encoded chunks at the positive safe integer `maximumChunkBytes` boundary. Preserve order, one-based sequence numbers and the full-payload SHA256. Reject zero, negative, fractional and nonfinite limits rather than looping or submitting invalid chunks. Regression tests reconstruct exact4099-byte payloads across1,2,4,5 and1024-byte boundaries and cover invalid limits. Optional notebook refresh cannot undo the local successful create/delete/restore result; media commit results are applied directly.

RC-01/02 frontend verification executed: `pnpm --dir apps/reader exec tsc --noEmit` exits0; `pnpm --dir apps/reader run test:reader` passes42 tests in2 files. The tests cover message/source isolation, sanitation, normalized Unicode selection/replay/delete, placement ambiguity, fragment navigation, style remount, create/upload failures and retained retry state, stable annotation IDs, MIME, chunking, recording budget and hook ordering. These are TypeScript/jsdom gates, not native sandbox, packaged artifact, offline resource delivery, real device capture or all-modality on-canvas acceptance. RC-03 and the native operator protocol remain independent work.

## RC-03 — All media on the article canvas

- ✅ **RC-03 Anchored drawing, typing, photography and video capture/playback**
  - Depends: RC-01/02 and RC-NATIVE-01. Initial release includes voice recording; automatic transcription is deferred under RC-04. Do not label recording as transcription.
  - Allowed read/edit: apps/reader/src/routes/reader/{rendererBridge,ArticleContent,ReaderView,ReaderStyles,ReaderCanvasTools,ReaderCanvasCapture,canvasPersistence,canvasBridge}.tsx or .ts as appropriate; existing ReaderCanvasTools.tsx/ReaderCanvasCapture.tsx are new; canvasPersistence.ts/canvasBridge.ts new; apps/reader/src/components/annotation/{PhotoCapture,VideoRecorder}.tsx; apps/reader/src/services/annotationService.ts; apps/reader/src/stores/annotationStore.ts; existing reader-canvas.test.tsx and annotation-save.test.tsx, new canvas-modalities.test.tsx. Also allowed read/edit apps/reader/src/stores/readerStore.ts solely for the article request-generation guard and its deterministic tests. Read-only existing AnnotationWorkspace.tsx, VoiceRecorder.tsx, services/readerService.ts, router.tsx and shared UI tokens. No native edits; existing protocol/asset command are implemented. Existing source files may be split into named canvasBridge.ts and canvasPersistence.ts to retain manageable modules; no duplicate parser/protocol.
  - File-size amendment: ReaderStyles.tsx owns unchanged READER_STYLE_TEXT and exported ReaderStyles():ReactNode; ReaderView.tsx re-exports ReaderStyles for existing consumers. Styling and runtime behavior remain unchanged.
  - Entities: ReaderCanvasTool = 'select'|'highlight'|'draw'|'type'|'voice'|'photo'|'video'. ReaderCanvasTools props {tool:ReaderCanvasTool,onToolChange(tool):void,busy:boolean}; enabled supported modes including voice recording; typing remains separate. ReaderAnnotationAsset {annotationAssetId:string,annotationId:string,assetKind:'ink_svg'|'voice_opus'|'image'|'file',mimeType:string,byteLength:number,durationMs?:number|null,resourceUrl:string}. annotationAssets(annotationId:Ulid):Promise<ReaderAnnotationAsset[]> invokes annotation_assets_list with annotationId and consumes bare array. RendererCanvasAnnotation {annotationId,annotationType,selector,bodyText?,colorToken,assets:readonly ReaderAnnotationAsset[]} uses existing types.
  - Entities: CanvasPoint {x:number,y:number}; CanvasStroke {points:CanvasPoint[],colorToken:string,width:number}; CanvasDraft messages schemaVersion1/channel and draftId bounded128: canvas_type_commit {selector,bodyText,draftId}, canvas_ink_commit {selector,strokes,width,height,draftId}, canvas_capture_request {selector,captureKind:'photo'|'video'|'voice',draftId}, canvas_draft_state {draftId,dirty:boolean}. Parent union: tool_changed {tool}, canvas_annotations_changed {annotations}, draft_result {draftId,saved:boolean,error?:string}, discard_draft {draftId}. Reuse message validator and active article checks, reject unknown modes/nonfinite points/out-of-bounds selectors, bound text16384,1000strokes/100000points/8MiBserialized, dimensions1..4096 and stroke width0.5..32. Never accept raw SVG/HTML from iframe.
  - Entities: ReaderCanvasCapture props {kind:'photo'|'video'|'voice',selector:SelectorBundle,draftId:string,onSaved(annotation:AnnotationRecord):void,onCancel():void}; owned privileged capture card overlays article. PhotoCapture export {bytes:Uint8Array,mimeType:'image/jpeg',width:number,height:number}; PhotoCaptureSelfManaged props {onCapture(capture):Promise<void>,onDiscard():void}; camera preview/shutter/review/retake/save; max4096dimension/16MiB; retain capture after failure. canvasPersistence.ts exports canvasInkSvg(strokes:readonly CanvasStroke[],width:number,height:number):string, validates numeric bounds and allowlisted colors and emits passive SVG; saveCanvasDraft uses existing store create/upload path, pending annotation ID retained per draft to avoid retry duplicate. No new enum or persistence schema.
  - Do: toolbar above article iframe. Trusted bridge creates overlay nodes marked data-ez-canvas so text normalization ignores all draft/saved overlay content; marks remain text-neutral. Click anchor resolves nearest article block using canonical text quote/context/positions; image click stores existing imageRegion with scoped entry key and fractions. No fabricated digest or unsafe selector string. Draft drawing canvas overlays selected paragraph/image bounds; pointer capture supports mouse/touch/pen, scaled numeric points, save/discard. Typing card is textarea at anchor, textContent only saved cards. Highlight mode uses selected text, not fabricated ranges. Reflow/resize recomputes placement from anchor using ResizeObserver, without accumulating document listeners. Saved ink image scales to anchored box; typed/media cards attach adjacent at same anchor. Unresolved/ambiguous cards visibly remain in an article-canvas unresolved section, not hidden in drawer. Existing focus_annotation scrolls both highlights and media cards.
  - Do: parent loads current article annotation assets in bounded batches (max4 concurrent; max2000 article records), ignores late old-article responses, passes only native-returned validated resource URLs matching exact trusted protocol origin and annotation/asset identifiers. No arbitrary renderer URL authority. Parent generates SVG and uploads ink_svg; image attaches image/jpeg image kind to comment annotation; video attaches file with actual supported MIME and duration. Render images as img and audio/video with controls via scoped native protocol, never inline SVG/object/iframe; restart refreshes metadata from native storage. Preserve body text/transcript and media simultaneously.
  - Do: capture uses parent camera/microphone only; no iframe permission. Camera PhotoCapture cleans tracks and Blob URLs on cancellation/unmount, handles denied permission with retry, JPEG via bounded canvas.toBlob. VideoRecorder adds live muted preview, maximum120seconds and128MiB including final data chunk; stop safely before unlimited memory, show actionable oversized state if encoder final chunk exceeds limit, never relabel container. Retain failed captured bytes, disable duplicate saves. Native asset upload handles chunking from RC02; SHA from actual bytes. Reuse current writable layer selection from annotation store/AnnotationWorkspace, never invented layer IDs.
  - Do: guard unsaved in-frame draft, active capture and failed pending upload via ReaderView useBlocker (router is createBrowserRouter). Provide Stay and Discard & leave; block navigation while save in-flight. Use beforeunload for external close with dirty state. Explicit discard resets matching draft and pending capture only. Existing drawer edits remain its existing semantics; RC03 guard covers new canvas flow. Do not lose capture on tool switch; require explicit discard or finish before switching. All draft_result responses bound to original document/channel/draft so late save cannot alter another article.
  - Entity amendment: readerStore.ts module-local articleLoadGeneration:number increments for each loadArticle and clearArticle. loadArticle captures generation plus requested packId/articleKey; accept success or failure only when generation and current requested position still match. clearArticle invalidates pending requests. ReaderView renders loading until route and store position agree. Deterministic reordered-promise tests cover late success, late failure and close/reset.
  - Verify: pnpm --dir apps/reader exec tsc --noEmit; pnpm --dir apps/reader run test:reader.
  - Accept: actual mounted bridge/parent tests cover all supported modes, drawing numeric validation and SVG upload, text anchoring and reopen from records, camera JPEG shutter bytes and denial cleanup, video actual MIME and capture limits, media metadata rendering and stale article rejection, failed upload retry same ID, dirty navigation stay/discard, ambiguous/reflow anchors and exclusion of annotation text from selection stream. Browser/native runtime acceptance is separate and required for user goal; jsdom does not prove sandbox/camera behavior.
  - Observed 2026-10-12: TypeScript noEmit exit0; test:reader 59 tests across3 files passed. Includes actual serialized bridge, voice retry/reopen, scoped metadata cancellation, camera/video bounds, navigation save-in-flight protection and readerStore reordered request/close guards. Native/browser operator acceptance remains unverified by these frontend gates.

## DD-RANGE-03 schema and catalog integration clarification (binding)

The original v1 SQL must remain byte-for-byte unchanged. It requires non-null full digests and permits active only fully verified packs; partial content must not counterfeit those claims. Extend allowed files with schemas/sqlite/0002_dyndon_sources.sql, crates/storage-sqlite/src/{dyndon_migration,lib}.rs, crates/storage-sqlite/tests/dyndon_migration.rs, apps/reader/src-tauri/src/ipc/kiwix.rs, apps/reader/src/services/{libraryService,readerService}.ts, stores/{libraryStore,readerStore}.ts, routes/library/{LibraryView,CatalogBrowser,DownloadPlanDialog,InstallPlanSummary,DownloadQueue}.tsx and library-dyndon.test.tsx. Read-only schemas/sqlite/0001_initial.sql, crates/storage-sqlite/src/{migration,query,write}.rs and tests, ports/catalog.rs, catalog-openzim/src/kiwix.rs, application/reader_nav.rs. Frontend RC files excluded.

Exact migration entity apply_dyndon_schema(conn:&rusqlite::Connection)->Result<AppliedMigration,MigrationError> in dyndon_migration.rs, exported lib.rs; constants DYNDON_VERSION=2, DYNDON_NAME="dyndon_sources", DYNDON_SQL include new SQL. Call after apply_initial at reader startup and all DD integration test bootstrap. Preserve apply_initial legacy API/tests. SQL file contains DDL only (no transaction controls); helper checks recorded fingerprint idempotently, temporarily disables foreign keys OUTSIDE transaction, opens IMMEDIATE transaction, applies DDL, runs foreign_key_check and aborts on any violation, records migration hash/time IN SAME transaction, then commits and restores previous foreign_keys on every success/error path. Reject invocation inside another transaction. No writable_schema, no replacing v1 fingerprint, no table loss on failure.

Rebuild catalog_sources/catalog_entries/pack_versions/download_jobs preserving every old row, existing columns/constraints, affected indexes/triggers/FKs. New schema differences only: catalog_sources.source_kind additionally kiwix_opds; catalog_entries.artifact_sha256 nullable with NULL-or64 length check, artifact_size_bytes>=0 for unknown advertised size. pack_versions.artifact_sha256 nullable, new source_identity_sha256 nullable64, verification_scope TEXT NOT NULL DEFAULT 'full' CHECK full/partial; verification_state additionally source_pinned. source_pinned requires scope partial and non-null source_identity_sha256; verified requires full scope and non-null artifact_sha256. download_jobs.expected_sha256 nullable64, new bytes_retained INTEGER NOT NULL DEFAULT0 CHECK>=0 and transfer_scope TEXT NOT NULL DEFAULT 'full' CHECK full/partial. Preserve bytes_verified for actual full verification. Active-version trigger permits verified/full or source_pinned/partial with non-null source identity and same pack; no unverified/quarantined activation. Preserve legal job transition guard. Parent observed parser coverage gap for SQL via CodeGraph; targeted reading of exact v1 SQL is authorized for reproducing affected table/index/trigger definitions. Tests populate v1 catalog, pack/version/location, download job/chunks and annotations, upgrade and compare rows/FKs/indexes/triggers, repeat upgrade, wrong fingerprint, transaction/error rollback, missing/full/partial digest truth and activation rules.

DD03 may use its own typed SQL inserts for partial rows while preserving strict full-archive storage write APIs unchanged. CatalogEntryRow, LibraryCatalogEntry, InstallPlanVerificationDto digest fields become Option<String>; frontend corresponding artifactSha256/expectedSha256 become string|null. Full local legacy conversion requires present digest or explicit error (never fallback hash/zeroes). Upstream SHA from Metalink remains expected full-artifact digest only; partial installation verification_scope='partial', verification_state='source_pinned', bytes_retained counts physically retained payload, bytes_verified=0. UI states source consistency pinned/local integrity, not full SHA verified. edition_manifest_sha256 may be SHA256 of actual serialized small manifest, explicitly manifest digest. source_identity_sha256 hashes canonical identity JSON (URL,size,validator,UUID) and never flows into artifact_sha256.

Kiwix catalog entity search_kiwix_catalog(state:&AppState,query:&str,language:Option<&str>,offset:u32,limit:u32)->Result<CatalogPage,String> in ipc/kiwix.rs, CatalogPage uses existing items/total/nextOffset fields with truthful optional totals. Native catalog_search delegates Kiwix client (empty search browses). Persist returned bounded KiwixEntry JSON in catalog_entries.metadata_json under stable ID (SHA256 of upstream entry ID may identify row but not content); source ID 'kiwix-opds-v2' kindkiwix_opds endpointhttps://library.kiwix.org/catalog/v2/entries. Upsert source/entries transaction after network complete; no DB mutex await. Retain unknown digest NULL until actual Metalink resolve; advertised rounded size displayed only until exact probe. Language UI explicitly ISO639-3 with common mapping en->eng,fr->fra,de->deu,es->spa,pt->por,zh->zho,ja->jpn,ar->ara,ru->rus; other3letter pass validated or no filter. Requested category must be supported explicitly via cached result filtering with honest no total, never silently ignored. Offline fallback queries cache and indicates cached data, never converts source failure to no matches. CatalogPage add source:'live'|'cache' and frontend labels cached when appropriate. Existing page totals must not equal current page size when more exist. Add pagination to existing catalog if nextOffset present; stale response cannot overwrite new query.

prepare_dyndon_install resolves stored KiwixEntry through client.resolve, tries HTTPS mirrors with bounded probe failures, pins exact probe length matching exact Metalink size when supplied (not rounded OPDSlength), preserves upstream digest optional. Probe method injectable for synthetic tests, no external Kiwix necessary. Existing reader_open should accept pack_id and optional legacy path parameter; frontend sends selected packId, result path compatibility retained. Startup restore offline without network; subsequent online reader resource use may instantiate pinned source after explicit online state but retained content must read even when online request fails. Persist plan reference in new app_data/dyndon-plans/<planId>.json with destination ID and relative root, no full directory or selected vector; validate references resolve only under registered destination root. Tests all native paths use actual schema2.

Additional Verify: cargo test -p storage-sqlite --test dyndon_migration; cargo test -p storage-sqlite; existing DD03 commands. UI tests cover budget bytes conversion, unknown digests, retained-progress truth, live catalog error/cache and pagination.

DD02 native decode orchestration may live in `C/src/disk/blob.rs` to keep modules below500lines. It only calls canonical streaming decoder APIs. Public `DiskPackStore::manifest(&self)->Result<DiskPackManifest,String>` returns a cloned bounded manifest for DD03 integration. No parser or library ownership changes.

## RC-04 — Deferred transcription; initial-release voice scope

- RC-04 speech recognition is deferred by explicit user decision on 2026-10-12. It is not an initial-release acceptance gate and is not marked implemented or verified.
- RC-03 includes a **Voice** tool that records, previews, durably saves and replays audio on its article anchor; **Type** remains separate. Replace unavailable dictate mode with enabled voice mode. Use existing VoiceRecorder and its real MIME/duration metadata; retain recordings on failed save and preserve offline playback after restart.
- Later transcription adds editable text to the existing voice annotation without replacing its audio asset, durable ID or selector. No speech model download or transcription claim in this release.
- Parent scope adjustment: preserve unverified RC04 experimental files outside app source; remove only parent-added speech dependencies/Vite alias; retain the observed esbuild lifecycle repair and RC03 test configuration. Verify dependency installation and TypeScript compilation; publish scope adjustment separately from in-progress worker source.

DD02 test-only reads may reuse canonical `AnZimmermanLib/tests/fixtures/rust-consolidation.zim` and `rust/tests/support/mod.rs` ZimBuilder. Add its existing flate2/tempfile dependencies in native cache dev-dependencies; production remains canonical parser-only.

DD-RANGE-02 SQLite spill clarification (authorized parent): `disk/plan.rs::stage_candidates(conn: &Connection, intent: &SelectionIntent) -> Result<(), String>` constructs persistent `plan_eligible`, `plan_groups`, `plan_domains`, and `plan_priorities` tables before bounded inserts. `plan_eligible_rank` orders priority DESC/article_count DESC/cluster_index ASC; `plan_groups_rank` orders score DESC/domain_present ASC/domain ASC/topic_present ASC/topic ASC. Nullable grouping uses explicit presence flags so None and empty string stay distinct. Per-row indexed upserts aggregate group scores; indexed scans choose breadth/depth without GROUP BY, DISTINCT, or temporary sort B-trees. `CANDIDATE_SCAN`, `GROUP_SCAN`, and `PRIORITY_SCAN` constants are the actual production scans, with EXPLAIN QUERY PLAN unit coverage rejecting TEMP B-TREE. Main-database indices/journals remain covered by metadata reservation/max_page_count. `tests/disk_range.rs` covers many distinct groups under the physical quota.

DD-RANGE-03 approved implementation helper paths: `apps/reader/src-tauri/src/ipc/dyndon/{mod,prepare,install,persistence}.rs` (`ProbedRangeSource`, `prepare_dyndon_install`, `run_dyndon_install`, `restore_dyndon_archives`); focused existing-command splits `ipc/{dto,rows,home,library,reader}.rs`; `AppState::new_with_transfer(Connection, PathBuf, Arc<dyn TransferSource>) -> Result<Self,String>` with shared `transfer_source`, `dyndon_stores`, `restore_errors`.

DD-RANGE-02 progress accessor clarification: `disk/plan.rs` adds `impl DiskPackStore { pub fn retained_bytes(&self) -> Result<u64, String> }`, querying `SELECT COALESCE(SUM(byte_length),0) FROM clusters WHERE state='complete'`. The value counts complete stored payload bytes, excludes metadata/workspace, and does not claim full archive verification. DD03 uses it for acquisition progress.

DD02 parent verification: `cargo test -p zim-cluster-cache -p transfer-http` passed26 tests (7 disk/planner,19 HTTP). Parent reviewed durable store, quota reservations, indexed planning, bounded decode workspace and exact HTTP identity/range validation. Component evidence does not establish live Kiwix/native artifact acceptance; DD03 and native rubric remain open.


### DD-RANGE-03 frontend execution contract (2026-10-12)

Allowed frontend files are services/{libraryService,readerService}.ts, stores/{libraryStore,readerStore}.ts, routes/library/{LibraryView,LibraryStyles,CatalogBrowser,DownloadPlanDialog,InstallPlanSummary,DownloadQueue}.tsx, routes/library/styles/{catalog,downloads}.css, routes/library/library-dyndon.test.tsx, ipc.contract.test.ts at its existing project path, and vitest.config.ts. Existing readerStore articleLoadGeneration guard must remain. Read-only StoragePlanner.tsx and StorageImpactBar.tsx are authorized for destination and budget composition.

Entities: CatalogPage extends PageResult<CatalogEntry> with source:'live'|'cache', notice?:string|null; totals/nextOffset accept null. LibraryState adds catalogSource:'live'|'cache'|null, catalogNotice:string|null, catalogTotal:number|null, catalogNextOffset:number|null, catalogRequest:CatalogSearchRequest|null, loadMoreCatalog():Promise<void>, installStarting:boolean. Monotonic catalog request generation rejects obsolete success/failure; loadMoreCatalog appends deduplicated IDs only for the same query/filter generation. computePlan(catalogEntryId,storageRootId,budgetBytes?:number) sends safe integer bytes; startInstall(id):Promise<boolean> returns true only on completed successful command and preserves plan/error for retry on failure. Root/entry changes invalidate obsolete plan requests. CatalogBrowser labels live/cached Kiwix, shows native notice and Load more with optional truthful total; maps common two-letter languages to ISO639-3, rejects unsupported language input visibly rather than silently ignoring it.

DownloadPlanDialog exports budgetBytesFor(value:string,unit:'MiB'|'GiB'):number|undefined, rejects nonpositive/nonfinite/unsafe byte counts, defaults1GiB, sends explicit bytes; the budget and destination must match the shown computed plan. Failed install retains plan and dialog with Retry download. InstallPlanSummary distinguishes planned ranged payload/full archive and source consistency/local integrity from full SHA verification; nullable upstream SHA is explicitly unavailable. DownloadQueue uses bytesRetained for partial jobs, bytesVerified for full jobs, and truthful partial/offline/failure labels.

readerService.readerOpen(packId:string):Promise<OpenPackResponse> sends {packId}; readerStore.open's existing string argument represents the selected pack ID. LibraryStyles.tsx imports ordered catalog.css?inline and downloads.css?inline and exports LibraryStyles():ReactNode; LibraryView reexports it. Extract existing CSS without style changes, keeping semantic blocks intact and files below500lines.

Verify TypeScript and reader tests with library-dyndon and existing IPC contract tests included. Cover unit-to-byte conversion/wire args, nullable digest, retained-byte UI, cache notices/errors/pagination/stale replies and failed install retry retaining the same plan ID. Native runtime acceptance remains separate.

DD03 approved native cancellation: `cancel_dyndon_install(&AppState,&str)->Result<(),String>` in `ipc/dyndon/install.rs`, Tauri `install_cancel(install_plan_id:String)` in `ipc/library.rs`. Cancel stops polling install future (including current network/decode future), preserves complete cluster files and durable plan, forbids completed state after cancellation; retry resumes retained data through a legal new job attempt. Read-only integration fixture/annotation dependencies: canonical `rust/tests/support/mod.rs`, `crates/annotations/src/lib.rs`, `crates/ports/src/annotation.rs`, `crates/domain/src/annotation.rs`; `AnnotationDraft`, `SelectorBundle`, `AnnotationService::{create,list}` and existing service persistence. Earlier annotations lib80-line targeted testdependency read preceded exact pathlisting; parent acknowledged near-miss, no further survey.

DD03 cancellation/progress refinement: libraryService.installCancel({installPlanId}):Promise<void> invokes install_cancel. DownloadPlanDialog offers Cancel download during active install, waits for native termination, retains the same plan for retry and reports cancellation errors. DownloadJob.expectedSizeBytes remains full upstream source size; partial jobs display bytesRetained without a percentage/full-size denominator unless a proven matching selected-payload target exists. Do not invent a selectedBytes field.

DD03 frontend verification observed 2026-10-12: pnpm --dir apps/reader exec tsc --noEmit exit0; test:reader70 tests in4files passed, including11 library contract/component/store tests. Library stylesheet extraction retains identical CSS tokens; touched frontend files below500lines and scoped diffcheck clean. Native install_start completion-only success/error rejection confirmed by integration owner; its separate native/runtime gates determine DD03 completion.

DD03 reviewed edition identity: `pack_version_id` hashes source_identity_sha256 + canonical serialized SelectionIntent + budget; source_identity_sha256 remains source identity only. Schema2 `UNIQUE(pack_id, upstream_version, artifact_sha256, edition_manifest_sha256)` permits distinct budget editions of the same published source. edition_manifest_sha256 hashes the exact bounded serialized `install-plan.json` edition definition, not mutable offline flags or payload. Registry `AppState::dyndon_stores` is keyed by canonical root PathBuf; single Arc per root; acquisition_gate serializes prepare/install. Restore attaches native source but forces offline=true before index open, so startup performs zero HTTP; an explicit prepare/install permits network again. Activation retains all prior active locations in the same transaction. Cancel drops pending cluster acquisition on <=100ms poll; a decoder already running may finish bounded work, but cancelled state prevents activation/completion.

DD03 modal progress amendment: DownloadPlanDialog reads downloads.data from the existing library store and displays the current install plan's newest native job state and bytesRetained against plan.downloadBytes inside the modal while installStarting. Match the native job-{installPlanId} identity and confirmed retry suffix boundary, never arbitrary prefix collisions. Reuse DownloadQueue's existing refresh loop; no second timer. Retained progress is informational and never closes the modal or claims completion before install_start resolves. Add mounted dialog test updating the downloads slice while install_start remains pending and assert partial bytes/target are visible in the dialog.

Native job matching confirmed: exact job-{installPlanId} or job-{installPlanId}-retry-{ULID}, ULID26 uppercase Crockford characters. Native job_list is updated_at descending; first matching job is current. DownloadQueue retains one polling interval, skips hidden ticks and resumes visible ticks even when mounted hidden.

DD03 integration test helper approved: `apps/reader/src-tauri/tests/dyndon_install/support.rs` owns native HTTP fixture transport/server and AppState fixture setup; existing integration target remains `dyndon_install`.

DD03 native checkpoint evidence (2026-10-12): `cargo test -p reader --test dyndon_install` passes 3 actual-HTTP/SQLite tests (partial acquisition+offline main/resource+typedannotation persistence; failed/cancelled retries+independent restore errors; SHA256 distinct budget editions+singleactive location+shared store Arc). `cargo check -p reader --lib` passes. **Unchecked acceptance gap:** metadata coverage currently pins main-page/header/MIME paths and planning cluster pointers only; a selected but unvisited article may require uncached directory binary-search pages after restart. Large-directory offline subset discovery is not verified and requires the separately assigned bounded persisted-entry index contract. This checkpoint is suitable for a debugging artifact, not complete offline acceptance.


## LP — AnZimmerman portable edition repairs (parent-reviewed contract)

# AnZimmerman language parity

Status: parent-reviewed architecture contract; implementation tasks below are **not implemented** by this document. Native Rust acquisition remains the EnZIME application path. This document propagates portable library correctness and bounded access into the existing language editions; it does not create another ZIM library or duplicate application storage policy.

## Authority and observed baseline

`L` means `VENDOR/AnZIMmermanLib-src/AnZimmermanLIB/AnZimmermanLib`, relative to the EnZIME repository. All paths below are relative to `L` unless explicitly prefixed `DOCS/`. Existing line numbers identify current anchors; `:1 (new)` identifies a new module whose exact exported names are frozen here.

CodeGraph was verified on 2026-10-12: 84 files, 1,745 nodes, 4,753 edges, SQLite `quick_check=ok`. Mapping used canonical CodeGraph source/symbol views. Its JSON/module-manifest coverage gap justified targeted reads of `package.json`, `go.mod`, `composer.json`, `requirements.txt`, `TOOLS/zimmer-extension/package.json`, and `tests/fixtures/rust-consolidation.expected.json`. Those five package manifests are absent. The fixture expectation exists and is authoritative for the baseline payloads. No format specification was fetched or substituted for the existing Rust implementation.

| Edition / anchor | Observed implementation | Direct observation on this host |
|---|---|---|
| Rust `rust/src/header.rs:41`, `directory.rs:96`, `cluster_stream.rs:9`, `range/source.rs:8` | Canonical 80-byte header, checked directory variants, bounded cluster decoding, paged lazy source/index. Application disk quotas stay in EnZIME's `crates/zim-cluster-cache`, not here. | DD01 tests are parent-owned evidence; this architecture does not relabel them as cross-language coverage. |
| Python `zimlib.py:46,166,344` | Incompatible header fields/magic, incorrect directory discriminator, guessed 1,000-byte entries, eager entry/pointer lists, offsets read before decompression; writer shares these defects. | Opening the shared fixture raises `struct.error: unpack requires a buffer of 76 bytes`. |
| Go `zimlib.go:52,228,577` | Incompatible header/magic and directory layout; `Open` reads the whole file; eager pointer/entry tables; cluster decoder reads offsets before decompression. | `GO111MODULE=off go test zimlib.go` fails: undefined `zlib`, unused `directoryPos`, unused `indexPtrPos`. |
| PHP `zimlib.php:47,112,380,578` | Incompatible header/magic, native-endian binary formats, incorrect directory width, guessed entry length, eager tables, compressed-blob extraction in the wrong order. | `php -l zimlib.php` fails at line 32: constant expression contains invalid operations (`ord()` in class constants). |
| TypeScript `zimlib.ts:240,497,723` | Newer header/directory layout already present. Node reader loads the whole file; both readers eagerly enumerate entries, parse offsets before decompression, and use unbounded buffers. Node reader skips entries and thereby changes redirect indices. Browser class depends on Node `Buffer`/zlib. Writer has wrong cluster pointer accounting, invalid offset tables, and a placeholder checksum. | Node 24 opens three shared-fixture entries, but `getArticleContent(A/Café)` returns four zero bytes instead of `<p>Library fixture</p>`. |
| Browser extension `TOOLS/zimmer-extension/src/lib/zimlib-browser.js:173,393` | Header field sequence and basic uncompressed offset-table algorithm are newer; magic is still `0x4D495A5A`. Reads a complete ArrayBuffer; redirects recurse without cycle bounds. Missing decompressor returns compressed bytes as if usable content. Writer sorts without remapping redirect/main-page indices, uses locale ordering and a placeholder checksum. | Importing the reader and opening the shared fixture in Node's browser-compatible runtime raises `Invalid ZIM file format`. This is not a real-browser acceptance claim. |

Observed runtimes: Node `v24.14.1`, Python `3.14.7`, Go `go1.26.1 linux/amd64`, PHP CLI `8.3.6`. `phpunit` and Python `pytest` are absent; `npx --no-install ts-node --version` reports the package absent. Node's `--experimental-transform-types` executes the existing TypeScript module. New parity tests use standard-library runners, avoiding an invented package identity or installation prerequisite. Existing legacy suites import names absent from current public APIs; their presence is not conformance evidence and their migration is a separate explicit task below.

## Portable behavior contract

### Format and error boundary

Copy the **behavior** of Rust `header.rs`, `directory.rs`, `cluster.rs`, `cluster_stream.rs`, and `range/{source,index,archive}.rs`. Use the shared fixture and malformed cases to keep editions aligned. Do not copy EnZIME's SQLite schema, quota calculations, Kiwix catalog, HTTP client, content sanitization, annotation state, or Tauri interfaces.

All editions accept major versions 5 and 6, reject unsupported versions, and parse the fixed header: little-endian magic `0x044D495A` at 0; major/minor u16 at 4/6; UUID bytes at 8..24; entry/cluster u32 counts at 24/28; path/title/cluster/MIME u64 offsets at 32/40/48/56; main/layout u32 indices at 64/68; checksum u64 offset at 72. `0xFFFFFFFF` denotes no page. The UUID representation exposed in parity reports is lowercase 32-character hexadecimal. Source-relative arithmetic is checked before converting to an index, allocating, seeking, or reading. JS uses bigint for source offsets; PHP rejects values above `PHP_INT_MAX` and runtimes with `PHP_INT_SIZE != 8`, rather than silently using floats.

Directory entries begin with u16 MIME discriminator, u8 parameter length, u8 namespace, u32 revision. Articles contain u32 cluster/blob numbers; redirects contain u32 target index; link-target/deleted variants retain their actual index positions. URL and title require valid terminated UTF-8 within limits; parameter bytes following the strings are consumed and checked. Preserve original indices for every entry. Never remove unsupported entries from the index sequence. Looking up an unsupported entry produces `UnsupportedFeature`; bad pointers/discriminators/terminators produce `InvalidFormat`.

Redirect resolution is iterative, tracks visited indices, and stops after 16 hops by default (configurable maximum 32). A missing/out-of-range target, cycle, and hop-limit error are explicit. `getMainPage`/equivalent retains the raw entry API; `resolveEntry`/equivalent returns its final article. Content access accepts article or redirect and resolves through the same method. Python path namespaces compare numeric bytes to `ord(path[0])`.

Error codes across editions are `InvalidFormat`, `UnsupportedFeature`, `UnsupportedCompression`, `AllocationLimit`, `DecompressionLimit`, `EntryNotFound`, `RedirectCycle`, `RedirectLimit`, and `Io`. Python `ZimError.code`, JS `ZimError.code`, PHP `ZIMException::errorCode`, and Go `*ZimError.Code` carry these strings. Existing callers still receive their language's exception/error interface. Errors never return undecoded bytes, empty payloads, or an apparently successful short read.

### Bounded cluster and source behavior

Match canonical `rust/src/header.rs::cluster_end`: require any next cluster pointer to be greater than start and within source, then choose the earliest boundary after start among next cluster, path/title/cluster-pointer/MIME sections, checksum position and source length. Reject metadata boundaries beyond source and empty ranges. Do not assume the last cluster extends to the checksum when metadata follows it. Interpret compression as the low nibble and extended-offset width from bit `0x10`; reject other flag bits. Decompress the complete cluster body **before** interpreting its offset table. The first decoded offset gives table byte length; it must be aligned to width 4 or 8 and include at least two offsets. Selected endpoints must be ordered, start at/after the table, and end within decoded length. Empty blobs are valid.

Streaming decode writes to a caller-owned seekable store in chunks no larger than 64 KiB, then reads only the requested blob. Defaults: 2 GiB decoded-output ceiling, 32 MiB returned-blob ceiling, 64 MiB decoder-window ceiling. Callers may lower each. Reject a decoder/provider that cannot enforce the configured ceilings. Stop after one valid frame, matching canonical Rust; reject truncated/invalid frames and preset-dictionary streams. Codec 2 detects the same wrapped-zlib versus raw-DEFLATE distinction as Rust. No whole-cluster memory allocation is implied by the decoded-output ceiling. Decoder truncates the caller-owned output before writing and on error; it does not close/delete caller resources. Existing convenience content APIs without an explicit store use at most32MiB in memory and reject larger decoded clusters. Python adds `get_article_content_to(entry,store:BinaryIO)->bytes`, Go adds `GetContentWithStore(entry interface{},store io.ReadWriteSeeker)([]byte,error)`, PHP adds optional `$store=null` to getArticleContent, and browser adds optional `store` to async getArticleContent. These explicit store paths enable large decoded clusters without imposing an application-specific storage policy.

| Edition | Required codec support in these tasks | Explicit unsupported behavior |
|---|---|---|
| Rust | Existing 0/1, 2, 4, 5 remains the reference | Existing 3/unknown error behavior unchanged |
| Python | 0/1, 2 via `zlib.decompressobj`, 4 via bounded `lzma.LZMADecompressor`; preserve existing LZMA capability | 3/5 are `UnsupportedCompression` in this task; no claim of full codec parity |
| Go | 0/1, 2 via `compress/zlib` or `compress/flate`, `io.LimitedReader` and bounded output loop | 3/4/5 explicit `UnsupportedCompression` errors; no silent passthrough |
| Node TypeScript | 0/1, 2 via Node streaming zlib transforms with bounded input/output | 3/4/5 explicit `UnsupportedCompression` errors |
| Browser JS/TS | 0/1 with bounded caller-owned storage | 2/3/4/5 give `UnsupportedCompression` until a verified bounded codec provider exists; availability of DecompressionStream alone is insufficient allocator-bound evidence |
| PHP | 0/1 with bounded caller-owned storage | 2/3/4/5 give `UnsupportedCompression`; inflate_add has no output-allocation limit, so post-output length checks cannot establish the required bound |

Additional XZ/ZSTD providers require a separately reviewed dependency/provider contract with enforced window/output bounds and the same fixture suite. This is a precise capability boundary, not a claim that those formats work in all editions. For a codec API that emits a potentially oversized chunk before a caller can cap it, its provider must enforce the allocation bound internally or reject the operation; a check after unbounded allocation does not satisfy this contract.

Source primitives are transport-neutral: immutable byte length plus exact bounded range reads. They do not implement HTTP, source validators, downloads, retention, or disk quotas. The caller owns source identity and availability. The paged reader caches eight 64-KiB pages by default, accepts at most sixteen, and reports actual current/peak cache counts. A read result is at most 1 MiB. Entry reads cap at 64 KiB, MIME data at 1 MiB. Opening a reader reads only header/MIME data and lazy pointers; path lookup binary-searches the path pointer index. Cluster-pointer lookup fetches only one/two u64 values. Explicit `listArticles` remains an opt-in full enumeration API; new iterators avoid materializing it. No opening or lookup path constructs a complete directory/pointer array or copies a complete source.

### Writer consistency

Writer fixes are part of each edition's repair, not an independent incompatible format. Retain existing public writer entry points and route them to corrected serialization. Default output is version 6, explicit NONE compression code 1, canonical magic/header, correct u16+parameter directory encoding, correct cluster offset tables, exact pointer-array reservations, and 16-byte MD5 over bytes preceding checksumPos. This MD5 is format integrity, not an authenticity claim. Generate an archive UUID unless the caller supplies a test UUID. Write title-pointer indices with u32 entries or use zero to declare an absent optional title index; never point it at the u64 path pointer table.

Sort directory paths by namespace byte then raw UTF-8 bytes, never locale comparison. Record the original insertion-index-to-sorted-index mapping and apply it to redirects, main page, and title pointer indices. Reject duplicate namespace/path keys and dangling redirect indices before publishing output. Serialize compression at the **cluster body** level (offset table plus blobs), never per blob. The writer's buffered API remains explicitly bounded to 128 MiB total output by default; enforce this before growth. No bounded-reader claim applies to a caller explicitly requesting a buffered writer output.

## Frozen entity table

Common numeric defaults and error-code strings above are part of every signature below. Existing public facade names remain; internal modules keep each changed implementation file below 500 lines. New modules are language-edition internals inside this same library.

### Shared JavaScript format and storage primitives

| Exact entity | Target | Signature / fields |
|---|---|---|
| `ZimError` | `portable/format.mjs:1` (new) | `class ZimError extends Error { constructor(code, message); code: string }` |
| `parseHeader` / `validateHeaderBounds` | `portable/format.mjs:1` | `parseHeader(bytes:Uint8Array):Header`; `validateHeaderBounds(header:Header, sourceLength:bigint):void`; Header uses TS's existing camelCase fields and bigint offsets |
| `parseEntry` | `portable/format.mjs:1` | `(bytes:Uint8Array):{entry:Entry,consumed:number}`; Entry discriminant `kind:'article'|'redirect'|'linktarget'|'deleted'`, numeric namespace, existing camelCase fields, `parameters:Uint8Array` |
| `clusterEncoding` / `readBlobBounds` | `portable/format.mjs:1` | `(tag:number):{codec:number,offsetWidth:4|8}`; `(firstOffset:bigint,startOffset:bigint,endOffset:bigint,offsetWidth:4|8,blobIndex:number,decodedLength:bigint,maximumBytes:number):{start:bigint,length:number}`; callers fetch only first and selected offsets, not whole table |
| `RangeReadLimits` / `ClusterDecodeLimits` | `portable/format.d.mts:1` (new) | CamelCase equivalents of canonical Rust limit fields; `.mjs` functions validate their values at runtime. Same declaration file declares all shared JS types/functions below |
| `RandomAccessSource` / `SyncRandomAccessSource` | `portable/format.d.mts:1` | `{length:bigint;readRange(offset:bigint,length:number):Promise<Uint8Array>}` / same with immediate Uint8Array |
| `PagedRangeReader` / `SyncPagedRangeReader` | `portable/range.mjs:1` (new) | `constructor(source,limits)`; `readExact(offset:bigint,length:number)` async/sync respectively; `cacheCounts():{current:number,peak:number}`; shared LRU bookkeeping, different I/O scheduling only |
| `ArrayBufferSource` / `BlobSource` | `portable/range.mjs:1` | Constructors accept caller-owned ArrayBuffer / Blob; `length:bigint`; exact `readRange`; Blob uses bounded `.slice().arrayBuffer()`, never full `.arrayBuffer()` |
| `BoundedMemoryStore` | `portable/range.mjs:1` | `constructor(maximumBytes:number=33554432)` implements SeekableByteStore using checked64KiB pages; rejects growth before allocation; owns no filesystem state |
| `SeekableByteStore` | `portable/format.d.mts:1` | `{write(chunk:Uint8Array):Promise<void>;readAt(offset:bigint,length:number):Promise<Uint8Array>;truncate():Promise<void>}`; store owner controls storage location/lifetime |
| `md5` | `portable/md5.mjs:1` (new) | `(bytes:Uint8Array):Uint8Array` returns 16-byte legacy-format digest; bounded writer input only; independent known-vector tests and Node crypto comparison required |

### TypeScript and browser editions

| Exact entity | Target | Signature / responsibility |
|---|---|---|
| `ZIMReader`, `ZIMWriter`, `ZIMReaderBrowser` exports | `zimlib.ts:240,497,723` | Existing facade exports re-export implementations below; preserve constructors and existing methods; remove duplicated parser bodies |
| `ZIMHeader`, `DirectoryEntry`, `RedirectEntry`, `Entry` | `typescript/types.ts:1` (new) | Re-export shared format types; existing numeric namespace and camelCase fields preserved; special variants and parameter bytes added |
| `NodeFileSource` | `typescript/node-source.ts:1` (new) | `constructor(path:string)`; `length:bigint`; sync `readRange(offset:bigint,length:number):Uint8Array`; `close():void`; `openSync/fstatSync/readSync` exact loop, no readFileSync |
| `ZIMReader` | `typescript/reader.ts:1` (new) | `constructor(path:string,limits?:RangeReadLimits)`; existing `open/close/getHeader/getMimeTypes/getEntryByPath/getMainPage/listArticles`; `entryAt(index:number):Entry`; `resolveEntry(entry:Entry):DirectoryEntry`; `entries():IterableIterator<Entry>`; `getArticleContent(entry:Entry):Buffer`; `getArticleContentAsync(entry:Entry,store:SeekableByteStore):Promise<Buffer>` |
| `decodeClusterToSync` / `readClusterBlobSync` | `typescript/node-codecs.ts:1` (new) | `(input:Iterable<Uint8Array>,output:{write(chunk:Uint8Array):void},limits:ClusterDecodeLimits):{length:bigint,offsetWidth:4|8}`; `(store:{readAt(offset:bigint,length:number):Uint8Array},decoded,blobIndex:number,maximumBytes:number):Uint8Array`; preserve synchronous API only with genuinely bounded synchronous codec support, never block on a Promise |
| `ZIMReaderBrowser` typed adapter | `typescript/browser.ts:1` (new) | `constructor(source:ArrayBuffer|Blob|RandomAccessSource,limits?:RangeReadLimits)`; delegates shared browser reader; `open():Promise<void>`, `getEntryByPath(path:string):Promise<Entry|undefined>`, `getMainPage():Promise<Entry|undefined>`, `getArticleContent(entry:Entry,store?:SeekableByteStore):Promise<Uint8Array>`, `listArticles():Promise<DirectoryEntry[]>`; converts numeric namespace only at facade boundary |
| `ZIMWriter` | `typescript/writer.ts:1` (new) | Existing writer methods, corrected format contract; `setMainPage(index:number):void` added; `constructor(path:string,maximumBytes?:number)` |
| `ZIMReaderBrowser` | `TOOLS/zimmer-extension/src/lib/zim-reader-browser.js:1` (new) | `constructor(source:ArrayBuffer|Blob|RandomAccessSource,limits?)`; async `open():Promise<void>`, `entryAt(index:number):Promise<Entry>`, `resolveEntry(entry:Entry):Promise<Entry>`; `entries():AsyncIterable<Entry>`; `getArticleContent(entry,store?:SeekableByteStore):Promise<Uint8Array>`; `getEntryByUrl(namespace:string,url:string):Promise<Entry|undefined>`, `getMainPage():Promise<Entry|null>`, `listArticles():Promise<Entry[]>` replace the old synchronous return values and update the two named consumers below; namespaces remain strings at this facade boundary |
| `decodeClusterTo` / `readClusterBlob` | `portable/cluster.mjs:1` (new) | `async(input:AsyncIterable<Uint8Array>,output:SeekableByteStore,limits:ClusterDecodeLimits):{length:bigint,offsetWidth:4|8}`; `async(store,decoded,blobIndex:number,maximumBytes:number):Uint8Array`; browser codec capability check explicit |
| `ZIMWriterBrowser` | `TOOLS/zimmer-extension/src/lib/zim-writer-browser.js:1` (new) | Existing `addArticle/addRedirect/setMainPage/finalize/getBlob`, constructor optional maximumBytes; shared format/MD5, correct sorting/remapping |
| Browser facade and consumers | `TOOLS/zimmer-extension/src/lib/zimlib-browser.js:173,393`; `src/background.js:1`, `src/viewer/viewer.js:1` within extension | Facade re-exports reader/writer, keeps archive-conversion helpers. Consumers await reader lookups/enumeration. TypeScript browser export delegates to this same browser reader with a typed numeric-namespace adapter; no second browser parser |

**Synchronous Node codec decision:** Node's synchronous one-shot inflate cannot establish the streaming sink contract above for arbitrarily large clusters. Therefore `ZIMReader.getArticleContent` is a bounded compatibility operation: cap compressed and decoded buffers to 32 MiB using supported `maxOutputLength`; it may reject a large cluster even when the requested blob is small. Add `ZIMReader.getArticleContentAsync(entry:Entry,store:SeekableByteStore):Promise<Buffer>` and `decodeClusterTo` in `typescript/node-codecs.ts` for large-cluster streaming with Node transforms. Its exact signature is `decodeClusterTo(input:AsyncIterable<Uint8Array>,output:SeekableByteStore,limits:ClusterDecodeLimits):Promise<{length:bigint,offsetWidth:4|8}>`. `decodeClusterToSync` accepts only bounded input (at most32MiB) and uses a 32MiB maximum output regardless of larger caller limits; it must document this. This explicit capability distinction preserves the existing synchronous API without inventing synchronous streaming support.

### Python edition

| Exact entity | Target | Signature / fields |
|---|---|---|
| Public facade | `zimlib.py:1` | Re-export existing names from modules below, including constants/enums; API filenames remain stable |
| `ZimError`, `ZIMHeader`, `DirectoryEntry`, `RedirectEntry`, `SpecialEntry` | `zimlib_format.py:1` (new) | `ZimError(code:str,message:str)`; dataclasses use existing snake_case fields plus `uuid:bytes`, `path_ptr_pos:int`; remove fabricated serialized article/redirect counts; existing descriptive aliases `mime_type_list_pos`, `title_index_pos`, `main_page_index`, `layout_page_index` remain computed properties for canonical `mime_list_pos`, `title_ptr_pos`, `main_page`, `layout_page`; fabricated counts are not serialized; entry `parameters:bytes=b''` |
| `ZIMHeader.from_bytes/to_bytes`, `parse_entry`, `cluster_encoding` | `zimlib_format.py:1` | `(data:bytes)->ZIMHeader`, `()->bytes`; `(data:bytes)->tuple[Entry,int]`; `(tag:int)->tuple[int,int]`; exact canonical encoding |
| `RandomAccessSource`, `FileRangeSource`, `PagedRangeReader` | `zimlib_io.py:1` (new) | Protocol `length:int;read_range(offset:int,length:int)->bytes`; FileRangeSource(path:str), `close()`; PagedRangeReader(source,limits), `read_exact(offset,length)->bytes`, `cache_counts()->tuple[int,int]`; exact seek/read loop protected against interleaved operations |
| `RangeReadLimits` / `ClusterDecodeLimits` | `zimlib_io.py:1` | Dataclasses with snake_case versions of shared limit fields |
| `decode_cluster_to` / `read_cluster_blob` | `zimlib_cluster.py:1` (new) | `(input:BinaryIO,output:BinaryIO,limits:ClusterDecodeLimits)->tuple[int,int]` (length,width); `(store:BinaryIO,decoded:tuple[int,int],blob_index:int,maximum_bytes:int)->bytes`; caller supplies seekable bounded store |
| `ZIMReader` | `zimlib_reader.py:1` (new) | Existing constructor gains optional `source:RandomAccessSource=None,limits:RangeReadLimits=None`; existing methods preserved; `entry_at(index:int)->Entry`, `resolve_entry(entry:Entry)->DirectoryEntry`, `iter_entries()->Iterator[Entry]`; `get_article_content_to(entry:Entry,store:BinaryIO)->bytes`; lazy source/pointers |
| `ZIMWriter` | `zimlib_writer.py:1` (new) | Existing signatures preserved; `set_main_page(index:int)->None`; optional maximum_bytes constructor argument; correct shared format and hashlib MD5 |

### Go edition

| Exact entity | Target | Signature / fields |
|---|---|---|
| `Header`, `DirectoryEntry`, `RedirectEntry`, `SpecialEntry`, `ZimError` | `zimlib.go:52,70,81` | Canonical fields `MagicNumber uint32; MajorVersion,MinorVersion uint16; UUID [16]byte; EntryCount,ClusterCount uint32; PathPtrPos,TitlePtrPos,ClusterPtrPos,MimeListPos uint64; MainPage,LayoutPage uint32; ChecksumPos uint64`; entry discriminator uint16, parameters []byte; ZimError{Code,Message string} implements error |
| `ParseHeader` / `ParseEntry` / `SerializeHeader` | `zimlib_format.go:1` (new) | `([]byte)(*Header,error)`; `([]byte)(interface{},int,error)`; `(*Header)([]byte,error)` |
| `BinaryReader` / `BinaryWriter` | `zimlib_binary.go:1` (new; moved from zimlib.go:91,157) | Preserve existing public methods for compatibility; parser uses checked new `ReadExact(length int)([]byte,error)` / `SeekChecked(position int)error`, no panic-prone unchecked slice accesses |
| `RandomAccessSource` / `FileRangeSource` | `zimlib_range.go:1` (new) | interface `{Len() uint64;ReadRange(offset uint64,length uint32)([]byte,error)}`; `NewFileRangeSource(path string)(*FileRangeSource,error)`; Close()error; os.File.ReadAt exact loop |
| `RangeReadLimits` / `PagedRangeReader` | `zimlib_range.go:1` | PascalCase equivalents of shared fields; `NewPagedRangeReader(source RandomAccessSource,limits RangeReadLimits)(*PagedRangeReader,error)`; `ReadExact(offset uint64,length uint32)([]byte,error)`; `CacheCounts()(current,peak int)` |
| `ClusterDecodeLimits`, `DecodedCluster`, `DecodeClusterTo`, `ReadClusterBlob` | `zimlib_cluster.go:1` (new) | limits uint64 fields; DecodedCluster{Length uint64;OffsetWidth uint8}; `(input io.Reader,output io.Writer,limits ClusterDecodeLimits)(DecodedCluster,error)`; `(store io.ReaderAt,decoded DecodedCluster,blobIndex uint32,maximumBytes uint64)([]byte,error)` |
| `Reader` | `zimlib_reader.go:1` (moved from zimlib.go:228) | Existing NewReader/Open/Close/accessors preserved; `NewReaderFromSource(source RandomAccessSource,limits RangeReadLimits)(*Reader,error)`; `EntryAt(uint32)(interface{},error)`; `ResolveEntry(interface{})(*DirectoryEntry,error)`; `ForEachEntry(func(uint32,interface{})error)error`; `GetContent(interface{})([]byte,error)` resolves redirects, existing GetArticleContent delegates; `GetContentWithStore(entry interface{},store io.ReadWriteSeeker)([]byte,error)` |
| `Writer` | `zimlib_writer.go:1` (moved from zimlib.go:577) | Existing NewWriter/Create/AddArticle/AddRedirect/Finalize/Close preserved; `SetMainPage(index uint32)error`; corrected layout/crypto/md5; bounded 128MiB default |

No Go module name is invented. Test the explicit file set below with the standard library; imports within this package remain package-local. Migrating older relative-import examples/tests requires a later approved package identity.

### PHP edition

| Exact entity | Target | Signature / fields |
|---|---|---|
| Public facade/constants | `zimlib.php:1,19,31` | require_once the modules below and preserve classes/functions; namespace constants use numeric byte values valid in PHP constant expressions |
| `ZIMException`, `ZIMHeader`, `ZIMDirectoryEntry`, `ZIMRedirectEntry`, `ZIMSpecialEntry` | `php/format.php:1` (new) | ZIMException extends RuntimeException with public string errorCode; header canonical camelCase names with raw16byte uuid; integers64bitchecked; parameter bytes string |
| `ZIMHeader::fromBinary/toBinary`, `ZIMFormat::parseEntry/clusterEncoding` | `php/format.php:1` | `fromBinary(string $data):void`, `toBinary():string`; `parseEntry(string $data):array` returns [entry,consumed]; `clusterEncoding(int $tag):array` returns [codec,width]; explicit LE `v/V` and checked low/high words |
| `ZIMRandomAccessSource`, `ZIMFileRangeSource`, `ZIMPagedRangeReader` | `php/range.php:1` (new) | interface `length():int;readRange(int $offset,int $length):string`; file constructor(string $path), close():void; paged constructor(source,array $limits), readExact(int,int):string, cacheCounts():array{current:int,peak:int} |
| `ZIMCluster::decodeTo/readBlob` | `php/cluster.php:1` (new) | `decodeTo($input,$output,array $limits):array{length:int,offsetWidth:int}` resource streams; `readBlob($store,array $decoded,int $blobIndex,int $maximumBytes):string`; strict bounds/no raw fallback |
| `ZIMReader` | `php/reader.php:1` (moved from zimlib.php:380) | Existing constructor/methods preserved with optional source/limits; `entryAt(int $index)`, `resolveEntry($entry):ZIMDirectoryEntry`, `entries():Generator`; `getArticleContent($entry,$store=null):string` accepts redirect or article |
| `ZIMWriter` | `php/writer.php:1` (moved from zimlib.php:578) | Existing writer signatures preserved; `setMainPage(int $index):void`; PHP hash('md5',...,true), corrected layout/sort/remap |

## Self-contained task contracts for parent integration

LP-FIXTURE-01 precedes all fixture-consuming acceptance runs; independent implementation can proceed against the existing baseline fixture. All commands execute with cwd `L`. All tasks below read this complete document plus their named canonical references and fixtures; parent copies the complete relevant contract into the root CHECKLIST before coding. Each worker owns only its listed edition files and tests. Shared JS modules belong to LP-JS-01; dependent tasks do not edit them concurrently. No task edits Rust production parsing, EnZIME adapters, CI, package identities, or unrelated examples.

| ID / order | Do / exact allowed files | Verify / idempotent acceptance |
|---|---|---|
| LP-FIXTURE-01 | Read shared `.zim/.expected.json`, `rust/tests/support/mod.rs`, `rust/tests/cluster_stream.rs`, `rust/tests/range_access.rs`, and `rust/tests/range_access/edge_cases.rs`. Add `rust/examples/generate_language_parity.rs` that emits deterministic valid and malformed cases into `tests/fixtures/language-parity/` and `manifest.json`; add `tests/parity/fixture_manifest.py`. Use `ZimBuilder::add_article/add_redirect/write`; add shared codec fixture helpers to `rust/tests/support/mod.rs` named `encode_cluster_payload(codec:u8,width:u8,payload:&[u8])->Vec<u8>` and `replace_cluster(bytes:&[u8],cluster_index:u32,encoded:&[u8])->Vec<u8>` using canonical parsed header/pointers. Move/reuse the existing cluster_stream test encode/decoded logic into these helpers, updating `rust/tests/cluster_stream.rs` imports without changing assertions. Malformed cases patch a valid generated archive at parsed offsets; do not invent another archive parser/writer. Cases listed below; fixture names exact. Existing baseline fixture stays unchanged. | `python3 tests/parity/fixture_manifest.py`: verifies committed names/hashes and expected metadata, no writes. Generator rerun is preparation, not acceptance. Rust reference reads valid generated cases and rejects their specified malformed cases in `rust/tests/language_parity.rs`, `cargo test --manifest-path rust/Cargo.toml --test language_parity` passes. |
| LP-JS-01 | Own `portable/{format.mjs,format.d.mts,range.mjs,cluster.mjs,md5.mjs}` and `tests/parity/portable.test.mjs`. Implement exact common parsing, bounds, source cache, browser raw-codec capability/error behavior, and MD5 helper. Source-only Rust files above are read-only references. | `node --test tests/parity/portable.test.mjs`: shared header/entry/cluster cases, bounded synthetic source, decompression ceilings, missing codec failure, MD5 vectors all pass. |
| LP-PY-01 | Own `zimlib.py`, new `zimlib_{format,io,cluster,reader,writer}.py`, and `tests/parity/python_parity.py`. Apply format, redirect, streaming, lazy I/O and writer contract together. Use Python standard library only. | `python3 -m unittest discover -s tests/parity -p python_parity.py -v`: common cases, Python XZ cases, instrumented virtual source, writer remapping/digest and round-trip pass. |
| LP-GO-01 | Own `zimlib.go`, `zimlib_{format,binary,reader,writer,range,cluster}.go`, and `parity_test.go`. Repair existing compiler errors as part of moving methods; remove whole-file `data` storage; apply corrected writer. No go.mod or dependencies. | `GO111MODULE=off go test zimlib.go zimlib_format.go zimlib_binary.go zimlib_reader.go zimlib_writer.go zimlib_range.go zimlib_cluster.go parity_test.go`: standard-library tests all pass, no recovered panics counted as expected format errors. |
| LP-PHP-01 | Own `zimlib.php`, `php/{format,range,cluster,reader,writer}.php`, `tests/parity/php_parity.php`. Apply valid constant declarations, explicit endianness, lazy reads, codec bounds, writer repairs. | `php -l zimlib.php` plus `php tests/parity/php_parity.php`: script uses explicit exceptions/check functions independent of PHP assert settings, covers fixture/malformed/quota and writer behavior, exits0 only if all pass. |
| LP-TS-01 (after JS01 and BROWSER01) | Own `zimlib.ts`, `typescript/{types,node-source,node-codecs,reader,writer,browser}.ts`, `tests/parity/typescript.test.mjs`. Import shared format/range; preserve sync public Node behavior with documented32MiB cap; add async large-cluster method using caller-owned store. Browser export integration waits for LP-BROWSER-01; no second parser. | `node --experimental-transform-types --test tests/parity/typescript.test.mjs`: shared fixture actual payload, sync bound rejection, async bounded extraction, writer digest/remapping and virtual source tests pass. |
| LP-BROWSER-01 (after JS01) | Own extension `src/lib/{zimlib-browser.js,zim-reader-browser.js,zim-writer-browser.js}`, `src/{background.js,viewer/viewer.js}`, and `tests/parity/browser.test.mjs`. Preserve archive helper exports; update reader consumers to awaited lazy methods; pure browser APIs, no fs/Buffer/zlib imports. Bounded seekable storage is injected; if absent use a bounded32MiB memory store and fail above it. | `node --test tests/parity/browser.test.mjs`: shared fixture, Blob slicing counters, missing DecompressionStream, bounded raw stream, redirects, writer remap/digest pass. Node execution establishes library behavior only. Real extension load/offline read is a separate operator protocol. |
| LP-CROSS-01 (after all editions) | Own `tests/parity/run_cross_language.py`, `tests/parity/{emit_python.py,emit_php.php,emit_typescript.mjs,emit_browser.mjs}`, Go `parity_test.go` emit subtest, Rust `rust/tests/language_parity.rs`, `DOCS/language-capabilities.md`. Each writer emits the same semantic archive into a caller-specified repo-owned test directory. Every reader reads every emitted archive; Python standard-library MD5 independently checks each trailer. Capability table names supported codecs and test results per edition; no blanket parity badge. | `python3 tests/parity/run_cross_language.py`: all cross-reader payload/main/redirect/resource comparisons and trailer digest checks pass. Output staging lives under `L/test-output/language-parity/`, is recreated idempotently, and is not a published artifact. |

Existing legacy tests are retained. LP-CROSS-01 additionally records their exact import/API mismatches in the capability document; it does not rewrite them to pretend their historical API existed. Their integration can be contracted separately after deciding whether those APIs are intended. New conformance tests are the acceptance surface for these bounded tasks.

### Exact fixture and test matrix

`tests/fixtures/rust-consolidation.zim` remains the baseline (SHA256 `4f58d62aa5986ffec605e075f30c17198e1256c956fb5989f968b77333ecb4f6`). Expected three entries: `A/Café` (`text/html`, `<p>Library fixture</p>`), redirect `A/Home`→`A/Café`, and `I/pixel.txt` (`text/plain`, `pixel`); raw main page is `A/Home`. Tests verify content, MIME, Unicode keys, UUID and raw redirect index, not just successful open.

Fixture generator names: `none32.zim`, `none64.zim`, `zlib.zim`, `deflate-raw.zim`, `xz.zim`, `zstd.zim`, `redirect-chain.zim`, `redirect-cycle.zim`, `redirect-out-of-range.zim`, `special-entry-indices.zim`, `truncated-header.zim`, `bad-magic.zim`, `unsupported-version.zim`, `pointer-out-of-range.zim`, `truncated-directory.zim`, `unterminated-url.zim`, `bad-utf8.zim`, `truncated-codec.zim`, `bad-offset-width.zim`, `reversed-offsets.zim`, `offset-after-end.zim`, `unknown-codec.zim`, `huge-declared-table.zim`. `manifest.json` records each case's SHA256, expected valid payloads or error code, codec, offset width, and applicable capability. Rust harness maps existing UnsupportedFeature codec errors to the portable UnsupportedCompression result for comparison; it does not change production Rust errors. Unsupported codecs are tested for explicit UnsupportedCompression in editions lacking providers; they are never skipped and counted as decode success.

Each edition also supplies an instrumented **virtual 1 TiB** RandomAccessSource serving small fixture regions: opening/path lookup/main resolution must not allocate/read by source length or all directory count; maximum request64KiB, current/peak pages<=8 at defaults. Read requests greater than1MiB, short reads, pointer arithmetic overflow, and >configured MIME/entry/blob/decode limits fail explicitly. Test both returned bytes and allocation/request counters. No giant real fixture or sparse file is required.

Writer interoperability tests insert paths out of order and set main/redirect targets using insertion indices, then confirm remapping after sorting; include Unicode byte ordering, empty blob, duplicate path rejection, two clusters, parameter bytes, and 128MiB growth guard. A same-language round-trip alone is insufficient because two matching bugs can pass it.

## Integration and release interpretation

The parent may schedule independent Python, Go, PHP work after this contract is reviewed; shared JS precedes its Node/browser consumers. Every task ends with its own commit and GitHub publication through the named parent integrator; Forgejo remains out of service. Only files named by the task are staged.

This architecture establishes a repair plan, not an application dependency on all language editions. The current native reader continues to use canonical Rust. Cross-language correctness, codec capability, bounded access, browser integration, and commercial packaging remain separate verification claims. A passed Rust or Node test never substitutes for PHP, Python, Go, or a real browser run.

## Architecture handback evidence

This document records one library with several language editions. The repair scope is portable structural correctness plus verified bounded primitives; codecs lacking a bounded backend fail explicitly. No new third-party codec dependency, registry identity, network transport, or app-specific retention policy is implied. Existing reader/writer APIs remain where feasible; the browser lookup/enumeration scheduling change is explicit and names its actual consumers. All LP tasks remain unchecked until their named commands run and semantic assertions pass.

LP-PY-01 approved constructor refinement: ZIMReader(file_path:str="", source:RandomAccessSource=None, limits:RangeReadLimits=None, decode_limits:ClusterDecodeLimits=None). self.decode_limits controls output/blob/window bounds, independently validated; convenience extraction clamps configured output to32MiB and explicit store extraction retains configured output at most2GiB. No new codec dependencies.

LP-PY-01 canonical bounds clarification: read-only `L/rust/src/limits.rs` is authorized; align MAX_URL_BYTES/MAX_TITLE_BYTES=2048 and MAX_MIME_TYPES=65530 with canonical Rust, retaining the64KiB total directory-entry bound.


## UB01 — Unified all-platform build contract

- [ ] Implement and verify the full build-set contract below.

# Unified EnZIME build set

Contract date: 2026-10-12. Owner: root integrator. Status: implementation contract ready for integrator review; execution remains unverified.

## Requirement and observed baseline

Every build invocation attempts the complete platform matrix under one version, source snapshot and command. Windows NSIS is a first-class required output. A successful Linux lane does not complete a build set. No caller selects a platform subset. Manual script execution is the release path; no CI activation.

Bootstrap used deployed sesh architect role, CodeGraph status (261 files, 4,828 nodes), project CLAUDE and Admin-Manual CC4/CC11/CC12/CC14. CodeGraph explicitly reports parser gaps for `scripts/update-version.sh` and `DOCS/BUILD-RECIPES.md`; targeted reads of those files and adjacent package/Tauri configuration establish this contract. Only the version stamper and mirror helper exist in scripts. Recipes record working Linux deb/AppImage, Linux-cross Windows NSIS, signed Android APK/AAB, CycloneDX and source archive. Current host has cargo-xwin, makensis and cargo-cyclonedx. No Windows executor or makeappx/WiX/PowerShell executable was discovered in the named inventory/current PATH. That observation is not a claim that a remote Windows machine is unavailable.

Existing `release.lock` for 1.9.63091 is a historical Linux-only trial. Preserve it under the build log before starting a new set; do not reuse its version for the unified attempt. Existing dist artifacts remain historical and are never input to collection.

## Entities

New-file line anchors are line 1 until implementation freezes exact positions.

| Entity | Target | Role and exact interface |
|---|---|---|
| build_all | `scripts/build-all.sh:1` | Sole public entrypoint: `bash scripts/build-all.sh [--jobs N] [--source-ref REF]`; default REF=HEAD, N=3; forwards validated args to build_set.py. No platform selector, implicit upload, commit or push. |
| BuildSet | `scripts/build_set.py:1` | Standard-library Python orchestrator; `main(argv:list[str])->int`, `prepare_snapshot(repo:Path,source_ref:str)->BuildSet`, `run_lanes(build:BuildSet,jobs:int)->list[LaneResult]`, `finalize(build:BuildSet,results:list[LaneResult])->int`. Fields version, version_code, source_commit, source_archive_sha256, root, snapshot, started_at, artifact_dir. |
| LaneResult | `scripts/build_set.py:1` | Fields id, status (`success`,`failed`,`unavailable`), exit_code, reason, log_path, started_at, finished_at, artifacts. Every required lane produces one record even on preflight failure. |
| BuildArtifact | `scripts/build_set.py:1` | Fields id, platform, kind, filename, size_bytes, sha256, signing (`verified`,`unsigned`,`not-applicable`), locations; locations hold role/transport/host/path/public=false. |
| build_lane | `scripts/build-lane.sh:1` | Internal interface `bash scripts/build-lane.sh LANE SNAPSHOT TARGET_DIR OUTPUT_DIR`; LANE=linux/windows-nsis/android/sbom/windows-native. Validates paths and version; executes documented recipes; never stamps. |
| verify_build_set | `scripts/verify-build-set.py:1` | Read-only `python3 scripts/verify-build-set.py MANIFEST [--require-complete]`; validates schema, all required target records, exact version filenames, size/hash, source digest and complete status. Default accepts an honestly incomplete attempt; require-complete exits nonzero unless every required artifact succeeded. |
| current_build_manifest | `dist/current-build-manifest.json:1` | Generated atomic snapshot of most recent attempt, never a directory scan; full statuses and relative artifact locations. |
| current_complete_build_manifest | `dist/current-complete-build-manifest.json:1` | Generated only after all required artifacts pass; prior complete set may remain, clearly labeled previous complete version on page. |
| build_downloads | `dist/index.html:1` | Generated self-contained download table with latest attempted version, each target status, exact versioned relative links and hashes. Windows NSIS row always present. Failed current rows never link an old artifact. No remote JS/CDN. |
| build_manifest | `dist/enzime-reader-v<VERSION>-build-manifest.json:1` | Schema `mba.robin.enzime.build-set.v1`; immutable completed-attempt record. Fields version, version_code, status (`complete`,`incomplete`), source, started_at, finished_at, lanes, artifacts, required_artifact_ids. |
| build_checksum_manifest | `dist/enzime-reader-v<VERSION>-sha256sums.txt:1` | Hashes only artifacts produced in this attempt, source archive and SBOM; excludes itself and mutable current pointers. |
| AndroidSymbols | `apps/reader/src-tauri/gen/android/app/build.gradle.kts:39` | Release `ndk.debugSymbolLevel = "FULL"`; collection requires actual native-debug-symbols.zip and mapping.txt when shrinking enabled. |
| WindowsNativeExecutor | environment `ENZIME_WINDOWS_BUILD_EXECUTOR` | Optional absolute executable invoked with arguments SOURCE_ARCHIVE OUTPUT_DIR VERSION; receives exact prepared source archive, builds MSI/MSIX with native tools, returns versioned outputs and machine-readable signing verification. Missing executor emits unavailable required targets, never success or silent skip. Executor config is local, not secrets in source. |

## Preparation and concurrency

1. Acquire an exclusive repository build-set lock under `target/build-set/build.lock` before version mutation. Refuse a second invocation cleanly. Resolve source-ref to a commit and archive only that commit; do not include concurrent uncommitted source edits. Print commit and dirty-path count without file contents. User work stays untouched. The integrator commits intended app changes before invocation.
2. Create semantic workspace `target/build-set/<UTC>-<commit12>/`. Preserve preexisting canonical release.lock to `~/Admin-Manual/LOGS/EnZIME-v4/build-sets/<attempt>/previous-release.lock`. Do not delete or overwrite unrelated locks without provenance: the build mutex owns only its lock and the freshly written version freeze.
3. Extract committed source to snapshot. Copy current canonical version.txt into snapshot as the version counter authority, remove only the snapshot release.lock, invoke its existing `scripts/update-version.sh` exactly once, then generate release.lock from resulting version.json using numeric fields (no octal interpretation). Freeze all version fields including buildDate by preventing subsequent stamper invocation. Synchronize the stamper-owned version files and release.lock back to canonical checkout only if their pre-run bytes still match; a concurrent change fails preparation without overwriting it. Paths: version.txt, version.json, package.json, Cargo.toml, apps/reader/src-tauri/Cargo.toml, apps/reader/src-tauri/tauri.conf.json, apps/reader/src-tauri/gen/android/tauri.properties. Never overwrite other source files.
4. Install frontend dependencies with `pnpm install --frozen-lockfile` in snapshot using shared pnpm store. Run `pnpm --dir apps/reader exec tsc --noEmit` and `pnpm --dir apps/reader exec vite build` directly inside snapshot (or equivalent cwd), avoiding package build hook's stamper. Preserve ordinary direct-development package scripts. Run Cargo lock update needed for stamped workspace version once, before lanes, record resulting Cargo.lock in snapshot and sync canonical Cargo.lock with the same compare-before-write rule. Dependency resolution failure is preparation failure, not permission to ignore lock mismatches.
5. Create source tar.gz from exact prepared snapshot excluding generated frontend, node_modules, target, dist, signing credentials and local config. Include release.lock, stamped manifests and resolved lockfiles. Its SHA256 is authoritative build-source identity in addition to parent commit. Never label this archive as plain committed HEAD: stamping changed it. Dist/source archive never contains credentials. Use tracked committed paths plus the explicit stamped/generated lockfile allowlist.
6. Make independent lane workspaces from prepared snapshot using reflink/copy and shared immutable frontend bytes. Each has its own generated Android tree, Cargo target directory and merged Tauri config. Merge `build.beforeBuildCommand` to harmless `echo Using prepared frontend`; set frontendDist to its copied prepared frontend. No lane reruns stamper or Vite. This prevents both version and Android/desktop generated-code races. Concurrency maximum --jobs; launch all lanes, collect all outcomes even if one fails. Limit Cargo build jobs within each lane to avoid multiplying memory pressure unconstrained.
7. Logs stream with lane prefixes and durable per-lane logs under `~/Admin-Manual/LOGS/EnZIME-v4/build-sets/<attempt>/`. Never log environment dumps or signing secrets. Toolchain caches remain shared; build targets/workspaces isolated. On signal terminate process groups, wait, finalize interrupted statuses; never promote partial files.

## Required matrix and concrete recipes

All names start `enzime-reader-v<VERSION>-`. Collection reads only that lane's empty-at-start output and verifies exactly one matching result. Unstamped raw tool output stays inside target workspaces.

| Artifact ID | Qualifier | Recipe / verification |
|---|---|---|
| linux-appimage | linux-amd64.AppImage | `cargo tauri build --bundles deb,appimage,rpm --config <prepared-config>` in Linux lane apps/reader; executable file nonempty; hash. |
| linux-deb | linux-amd64.deb | Same invocation; `dpkg-deb --field` product/version/architecture matches prepared version. |
| linux-rpm | linux-amd64.rpm | Same invocation; `rpm -qp` package version/architecture; missing tool installed through existing host provisioning, never drop row. |
| windows-nsis | windows-x64-setup.exe | `cargo tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc --bundles nsis --config <prepared-config>`; verify PE installer and nonempty. Record unsigned unless actual signature verification succeeds. |
| windows-msi | windows-x64.msi | WindowsNativeExecutor required; verify native package product/version and hash. No relabeling NSIS as MSI. |
| windows-msix | windows-x64.msix | WindowsNativeExecutor required; verify package manifest identity/version and actual signature. No fabricated manifest identity or certificate; configured publisher must match existing product signing authority. |
| android-apk | android-universal-release-signed.apk | `cargo tauri android build --apk --aab --config <prepared-config>` all four ABI targets; apksigner with existing production keystore, `--ks-pass env:ENZIME_KEYSTORE_PASSWORD --key-pass env:ENZIME_KEY_PASSWORD`; verify signature and manifest versionCode/versionName. |
| android-aab | android-universal-release-signed.aab | Same build, jarsigner uses `-storepass:env ENZIME_KEYSTORE_PASSWORD -keypass:env ENZIME_KEY_PASSWORD`; verify signature, bundle manifest version and all ABI contents. Do not create/rotate keystore. |
| android-symbols | android-native-debug-symbols.zip | Require actual AGP native-debug-symbols.zip containing all four ABIs; optionally include mapping.txt in named archive; inspect ZIP entries. Empty symbols cannot pass. |
| web | web.tar.gz | Archive single prepared frontend; inspect index.html and JS/CSS assets. This is packaged web output, not proof browser-native backend features work. |
| source | source.tar.gz | Prepared allowlisted source archive above; record archive digest and parent commit. |
| sbom | sbom.cdx.json | `cargo cyclonedx --all --format json` in isolated sbom lane; merge dedup by component identity preserving relationships and app version; include frontend production dependency inventory from lockfile, mark dependency scope. Valid CycloneDX document, not renamed npm JSON. |

Keystore location is `ENZIME_KEYSTORE_PATH`, fallback existing repository `signing/production.keystore`; alias `ENZIME_KEYSTORE_ALIAS` default upload. Read password environment or existing local credential source through integrator provisioning, never hardcode donor recipe literals. Do not silently sign with a debug key. Windows Authenticode remains explicit verified/unsigned state; no claim of signed commercial readiness from unsigned NSIS.

MSI/MSIX are required matrix members. Their missing executor does not prevent the current Linux/NSIS/Android/web build attempts, but final exit is nonzero and current-complete pointer cannot advance. An unconfigured native executor is an actionable target failure, not permission to call all platforms complete. The executor implementation/host must be verified before claiming full-complement completion; this interface intentionally exposes that remaining fact rather than inventing a reachable machine.

## Finalization and delivery

Each successful lane copies into canonical tracked dist only after hash/version validation, using temporary files inside the build workspace and atomic replacement on same filesystem. Filename collision with different bytes is an error; do not overwrite historical artifact. Manifest records actual current attempt only. Set current-build-manifest and dist/index atomically even on failure, with failed/unavailable rows clearly visible; do not mix earlier Windows installers into current results. Promote current-complete only if all required IDs succeeded. Process exit 0 means full matrix built; any failed/unavailable/interrupted required target yields nonzero.

Source pushes remain integrator-owned GitHub code publication under outage instruction. No script network publication, Forgejo access, GitHub LFS upload or automatic git commit. Locally staged artifact availability and externally published availability are separate manifest locations and claims.

## Self-contained checklist contract

### UB01 — Implement one build-set entrypoint

**Do:** Add build_all/BuildSet/LaneResult/BuildArtifact/build_lane/verify_build_set and generated manifest/page entities at exact paths above. Implement preparation, immutable snapshot, single stamping, bounded parallel lanes, all-target outcome aggregation, source provenance, signing environment handling and atomic no-stale promotion exactly as this document. Modify only AndroidSymbols release block plus DOCS/BUILD-RECIPES.md to make `bash scripts/build-all.sh` the sole operator build command and retain old recipes as internal provenance. Do not alter app feature source, user credentials, remotes or CI. Files under dist are generated by the script, not hand-authored release claims.

**Verify:** `bash -n scripts/build-all.sh scripts/build-lane.sh`; `python3 -m py_compile scripts/build_set.py scripts/verify-build-set.py`. Run actual `bash scripts/build-all.sh` once after intended source is committed; this is production execution, not an added smoke suite. Independently run `python3 scripts/verify-build-set.py dist/current-build-manifest.json` and inspect lane log outcomes. Then `python3 scripts/verify-build-set.py dist/current-build-manifest.json --require-complete`.

**Accept:** Entry scripts parse; read-only verifier proves each manifest artifact exists with recorded digest and version; required artifact IDs match matrix exactly; no failed current target resolves to an older artifact. Full-complement acceptance requires `--require-complete` exit0. A nonzero build is a documented partial implementation/execution outcome, never a checked complete gate. Native UI/device runtime verification remains the separate reader rubric and is not inferred from package construction.

**Operator verification protocol:** Open dist/index.html and choose current Windows NSIS without searching historical files; confirm visible version equals Linux/Android/web/source manifest. Install/run each built platform under DOCS/TEST_RUBRIC.md. Preserve screencasts and named runtime limitations separately from build hashes.


## OI — Offline entry catalog repair

- [ ] OI-01 Canonical pluggable entry index.
- [ ] OI-02 Durable bounded catalog.
- [ ] OI-03 Native plan/activation coverage.

# DynDon offline entry catalog

Status: exact repair contract for parent review/dispatch. No product source was edited during this assessment. This closes a reproduced offline correctness gap; the existing payload installation tests do not establish this gate.

## Observed failure and cause

CodeGraph verified EnZIME at 261 files/4,829 nodes and canonical AnZimmerman at 84 files/1,745 nodes. Relevant current paths are `R/src/range/{index,archive}.rs`, `C/src/disk/{source,store,plan,mod}.rs`, and `N/{prepare,install}.rs`, where:

- `R = VENDOR/AnZIMmermanLib-src/AnZimmermanLIB/AnZimmermanLib/rust`.
- `C = crates/zim-cluster-cache`.
- `N = apps/reader/src-tauri/src/ipc/dyndon`.

`prepare.rs:198` pins main-entry metadata. `plan.rs:63` enumerates cluster pointers with an evictable metadata source. `install.rs:64` downloads selected cluster payloads and subsequently reads the main article. `RemoteZimIndex::find_entry` (`index.rs:165`) binary-searches original directory pointers and entries. `RangeZimArchive::search_titles` (`archive.rs:164`) scans every source directory entry. Neither operation can work generally offline when only previously visited metadata pages were retained.

**Direct counterexample, observed 2026-10-12:** canonical `ZimBuilder` generated 100 articles `A/a00000` through `A/a00099`, each with a 2,000-byte valid title and a small `<p>payload N</p>` body; main entry index0. Source size206,697bytes. Current `plan_disk_fit` selected all100 clusters; `install_selected` retained2,590payload bytes. The online main-page validation ran. All handles were dropped, then `DiskPackStore::reopen(root,None)` and `open_disk_archive` ran.

```text
source_bytes=206697 selected_clusters=100 retained_bytes=2590 target_payload_present=true
offline_main=Ok("<p>payload 0</p>")
offline_unvisited_retained=Err(Io("offline_metadata_missing"))
requests=103 cached_pages=["0", "3", "1"]
```

`clusters/90.bin` existed, but binary lookup needed unfetched page2. The diagnostic reused the existing canonical builder and disk test helpers, compiled Rust supplied on stdin against Cargo's actual test dependency arguments, and wrote only under `target/`. Initial standalone compilation selected incompatible dependency-feature artifacts; obtaining the exact Cargo arguments corrected the diagnostic setup. No source modification or purported passing product test was used as evidence.

## Chosen behavior

Persist **canonical parsed entry records** for the admitted offline subset. Offline lookup must not need the original archive's binary-search pivot pages. The same canonical AnZimmerman archive continues to parse directory records, resolve redirects, decode clusters, sanitize articles, classify resources, and score title searches. Application storage supplies records/ranges and availability; it does not become a second ZIM parser.

A selected payload cluster alone is not an advertised offline article. Advertised entries require a durable canonical entry record, its complete redirect chain when applicable, a known cluster range, and a complete retained payload. The catalog can represent additional blobs present incidentally in a cluster, but undiscovered blobs are not counted as readable entries. Title search/enumeration operate only on catalog entries whose terminal cluster is currently complete. Resources in retained clusters receive the same metadata treatment as articles. A resource outside the admitted subset reports unavailable offline; the reader does not silently start a network download in offline mode. This contract does not promise to recursively download every linked article, image, stylesheet, or external resource.

### Discovery and budget decisions

There is no reverse cluster-to-directory index in the current source format/API. Finding every entry belonging to arbitrary retained clusters can require scanning the complete directory metadata. Metadata is smaller than a full archive in ordinary cases, but must not be described as free or always small. Adversarial/scattered pointers can make page fetches approach archive size.

Therefore discovery is explicitly bounded and resumable:

1. Before payload transfer, provision the pack's catalog schema and pin/persist the canonical main entry and its complete redirect chain. Keep the header/MIME bootstrap pages pinned for existing canonical index construction. The main cluster remains mandatory.
2. Run existing cluster-fit selection provisionally. Open a separate evictable metadata source with a durable **catalog transfer allowance**, `min(256MiB, disk_budget_bytes/4)`, for this edition. Every network metadata cache miss reserves its full requested page length before opening a transfer. Failures consume the reservation conservatively; cache hits consume none. Never exceed the allowance; this controls metadata traffic separately from retained disk space.
3. Scan source directory indices ascending with `RemoteZimIndex::entry_record`, at most128 records in memory. Persist only article/resource records whose cluster is provisionally selected. For a redirect, canonically resolve its at-most16-hop chain; retain the complete chain only when its terminal article/resource is selected. Each chain is admitted atomically. Stop cleanly at transfer allowance exhaustion; retain a resumable source index cursor and explicit partial coverage. Do not report a complete directory scan.
4. Charge catalog rows, indices, SQLite rollback journal and cached pages to the existing metadata partition. Before each bounded batch call the shared quota reservation. On metadata admission pressure, evict transient pages first; then remove the lowest-ranked nonpinned provisional cluster and its catalog rows, using inverse DynDon ordering (priority ASC, article_count ASC, cluster_index DESC). Retry admission only within the same fixed allowance. If mandatory main record/chain cannot fit, return an actionable `budget_exceeded` error.
5. Finalize selection to clusters with admitted terminal article/resource records plus mandatory main cluster. Clusters with no admitted records are deselected before payload download. Catalog discovery may reduce the provisional fit; the returned plan is the finalized fit and reports actual discoverable counts. Preserve enough SQLite free-page capacity for complete-state/progress updates before accepting the plan.
6. Install finalized clusters. A row becomes readable through a join with `clusters.state='complete'`; no separate optimistic availability flag is authoritative. Only after main record/chain and main payload are readable may activation complete. Publication records catalog coverage and counts, separately from payload bytes.

Discovery may finish early with an explicitly partial but usable offline subset. The native API reports this outcome; it does not label all original source entries as available. Re-running discovery resumes at its saved cursor without changing source identity or downloading a full file. Increasing the exhausted allowance requires a new explicit planning request; no background retry silently increases it. The allowance is an acquisition cap, not extra disk space. Requested payload selection and metadata admission are reconciled before install.

Online article/resource misses may use canonical remote lookup, but must atomically persist their entry record and redirect chain under the same catalog quota **before** fetching/advertising their payload. Online expansion cannot evict main metadata or annotation objects. Existing payload LRU eviction immediately removes entries from offline search through the availability join; records may remain as an online lookup cache until quota cleanup. Offline mode never uses remote fallback.

## Exact entities and signatures

All new files start at line1; existing line anchors refer to the map above. Keep source files below500lines by the named modules. New APIs are additive except the explicitly generalized `RangeZimArchive::index` accessor. Existing `RangeZimArchive::open(Arc<RemoteZimIndex>,...)` remains available and delegates to `open_index`.

| Entity | Target | Exact contract |
|---|---|---|
| `CanonicalEntryRecord` | `R/src/range/index.rs:15` | `Clone+Debug { entry:IndexedEntry, raw_bytes:bytes::Bytes }`. raw_bytes contains exactly the bytes consumed by canonical `parse_entry`; no guessed schema or reserialization. |
| `RemoteZimIndex::entry_record` | `R/src/range/index.rs:133` | `pub async fn entry_record(&self,index:u32)->Result<CanonicalEntryRecord,ZimError>`; existing entry_at delegates and returns `.entry`; bounded existing directory read, capture parser's consumed length. |
| `RemoteZimIndex::resolve_records` | `R/src/range/index.rs:197` | `pub async fn resolve_records(&self,index:u32,max_hops:u8)->Result<Vec<CanonicalEntryRecord>,ZimError>`; iterative canonical cycle/range checks, at most33 records with hard max32hops, default16. Includes original and terminal records. |
| `IndexedEntryStream` | `R/src/range/index_source.rs:1` (new) | `Pin<Box<dyn Stream<Item=Result<IndexedEntry,ZimError>>+Send>>` |
| `ZimEntryIndex` | `R/src/range/index_source.rs:1` | async_trait Send+Sync; `fn header(&self)->&ZimHeader`; async `entry_at(u32)->Result<IndexedEntry,ZimError>`, `find_entry(&EntryKey)->Result<Option<IndexedEntry>,ZimError>`, `cluster_range(u32)->Result<ClusterByteRange,ZimError>`, `entries(self:Arc<Self>)->Result<IndexedEntryStream,ZimError>`; default `resolve_entry(&EntryKey,u8)->Result<IndexedEntry,ZimError>` and `main_entry()->Result<Option<IndexedEntry>,ZimError>` reuse the canonical bounded redirect algorithm. |
| `impl ZimEntryIndex for RemoteZimIndex` | `R/src/range/index_source.rs:1` | Delegates existing lookups; entries yields source indices lazily. No behavioral change to complete remote archives. |
| `RangeZimArchive::open_index` | `R/src/range/archive.rs:22` | `pub async fn open_index(index:Arc<dyn ZimEntryIndex>,blobs:Arc<dyn ClusterBlobSource>,metadata:ZimMetadata,limits:RangeReadLimits)->Result<Self,ZimError>`; stored index becomes trait object; `index()->&Arc<dyn ZimEntryIndex>`; descriptor/content use it; entries/search use its stream instead of iterating header.entry_count. Existing canonical score/sanitize/decoder remain sole implementations. |
| `OfflineCatalogProgress` | `C/src/disk/catalog.rs:1` (new) | Serialize+Deserialize+Clone+Debug `{ scanned_entries:u32,total_entries:u32,readable_articles:u64,readable_resources:u64,retained_aliases:u64,network_allowance_bytes:u64,network_reserved_bytes:u64,network_received_bytes:u64,complete:bool,stop_reason:Option<String> }`; stop reasons exactly `transfer_allowance`, `metadata_budget`, or None. Before payload installation counts mean **admitted planned** entries; native DTO labels them accordingly, and installed counts are recomputed against complete clusters. |
| `DiskEntryCatalog` | `C/src/disk/catalog.rs:1` | Holds Arc<DiskPackStore> and cloned canonical ZimHeader; `pub fn new(store:Arc<DiskPackStore>,header:ZimHeader)->Result<Self,String>`; implements ZimEntryIndex. SQL key/index lookup -> canonical parse_entry(raw_bytes), MIME from persisted validated record. No original directory page read for a hit. Missing offline key produces EntryNotFound. |
| `DiskPackStore::catalog_progress` | `C/src/disk/catalog.rs:1` | `pub fn catalog_progress(&self)->Result<OfflineCatalogProgress,String>`; bounded aggregate queries and policy fields; count availability through cluster state. |
| `ensure_catalog_schema` | `C/src/disk/catalog_schema.rs:1` (new) | `pub(super) fn ensure_catalog_schema(conn:&rusqlite::Connection)->Result<(),String>`; schema below, empty-table index creation, no unchecked growth or temp sorting. |
| `discover_offline_entries` | `C/src/disk/catalog_discovery.rs:1` (new) | `pub async fn discover_offline_entries(store:Arc<DiskPackStore>,index:Arc<RemoteZimIndex>)->Result<OfflineCatalogProgress,String>`; main-chain seed, ascending bounded scan, atomic chain admission, quota-driven inverse selection pruning, durable cursor. Uses a catalog-budgeted index for acquisition, not caller's unbounded source. |
| `admit_entry_chain` | `C/src/disk/catalog_discovery.rs:1` | `pub(super) fn admit_entry_chain(store:&DiskPackStore,records:&[CanonicalEntryRecord],terminal_cluster:u32)->Result<(),String>`; one transaction, exact raw records/key/MIME, known cluster mapping, checked size; caller already holds quota/discovery coordination guard. |
| `DiskRangeSource::new_for_catalog` | `C/src/disk/source.rs:86` | `pub fn new_for_catalog(store:Arc<DiskPackStore>)->Self`; evictable pages, durable network allowance charge on miss. Existing new/new_for_planning semantics unchanged. |
| `DiskPackStore::reserve_catalog_transfer` / `record_catalog_received` | `C/src/disk/catalog_budget.rs:1` (new) | `pub(super) fn reserve_catalog_transfer(&self,length:u64)->Result<(),String>` atomic DB checked reservation before network, error `catalog_transfer_allowance`; `pub(super) fn record_catalog_received(&self,length:u64)->Result<(),String>` actual received accounting. Read-page/fetch path passes catalog purpose; charge includes failed attempts, exact chunks received checked against reservation. |
| Catalog coordination | `C/src/disk/store.rs:59` | Add `catalog_gate:Arc<tokio::sync::Mutex<()>>`; discovery/selection mutation/online chain admission serialize under it. Preserve cluster_gate and metadata_gate. Lock order: catalog -> cluster when needed -> metadata; never hold SQLite Mutex across await. Quota reservation plus catalog row insertion must share this gate so discovery and on-demand expansion cannot race the metadata ceiling. |
| `open_disk_archive` | `C/src/disk/mod.rs:12` | Existing signature preserved. Open pinned header/MIME bootstrap index, clone header, construct DiskEntryCatalog, then canonical RangeZimArchive::open_index. Catalog required for installed partial packs. Missing catalog in an older edition reports `offline_catalog_upgrade_required` instead of claiming offline completeness. |
| Online fallback `CatalogLookupSource` | `C/src/disk/catalog.rs:1` | `pub fn with_remote(self,index:Arc<RemoteZimIndex>)->Self` on DiskEntryCatalog; absent in offline construction. A miss resolves canonical records, admits chain under budget, and only then permits existing blob source acquisition. `entries` stays local/available; it never scans remote archive during title search. |
| Native plan integration | `N/prepare.rs:201`, `N/install.rs:64`, `N/persistence.rs:1`, `N/mod.rs:1` | Provisional plan -> discover_offline_entries -> recompute finalized PlanSummary -> persist plan. Install requires catalog main coverage and final selection; activation stores catalog progress/counts; reopening uses persisted catalog and offline source. Source identity mismatch rejects reuse. |
| Native DTO fields | `apps/reader/src-tauri/src/dto.rs:1`, `apps/reader/src/contracts.ts:1` | Add optional `offline_catalog:OfflineCatalogProgressDto` to InstallPlanDto, using camelCase in TS and existing serde naming conventions. DTO fields mirror progress, but name counters `planned_article_count`, `planned_resource_count`, `planned_alias_count` in plan response; installed statistics separately report actual availability. No new app DB column required: serialize progress inside existing saved plan JSON. |
| Plan presentation | `apps/reader/src/routes/library/DynDonRoute.tsx:1` | Show finalized admitted article/resource counts and metadata allowance consumed, plus plain-language partial discovery when complete=false. Keep selected payload bytes distinct from metadata traffic and disk reservation. Existing user budget governs all retained data. |

### Persistent schema and lookup semantics

`catalog_entries(entry_index INTEGER PRIMARY KEY,entry_key TEXT NOT NULL UNIQUE,raw_entry BLOB NOT NULL,mime_type TEXT,owner_cluster INTEGER NOT NULL)`; `owner_cluster` is the terminal payload cluster, including redirect-chain rows. Create `catalog_entries_owner` on `(owner_cluster,entry_index)` before inserting rows. Exact-key lookup uses the unique entry_key index. Source index IDs remain unchanged; header.entry_count stays the original source count, never a sparse catalog count. Reconstruct raw entry using canonical `parse_entry`; validate consumed length equals stored bytes and reconstructed key/MIME agrees with persisted data.

Policy keys: `catalog.version=1`, `catalog.source_identity` (existing identity digest), `catalog.cursor`, `catalog.complete`, `catalog.stop_reason`, `catalog.network_limit`, `catalog.network_reserved`, `catalog.network_received`. Update cursor only after its entry/chain admission decision commits. A crash retries at most the current batch and cannot double-advertise entries. Reservations already spent are not refunded across retries. Reuse existing manifest and cluster range tables; do not create a second archive header or cluster mapping.

Stream queries use primary-key keyset pagination, at most128 rows per batch. Join `catalog_entries.owner_cluster=clusters.cluster_index AND clusters.state='complete'`; available redirect-chain rows follow the same terminal availability. Canonical search retains its existing bounded hit buffer and scoring. It searches retained indexed titles, not all source titles. No FTS extension, all-source title vector, GROUP BY temp sorter, or full result materialization is required.

For a planned catalog before payload completion, use the same query with `clusters.selected=1` solely for plan counts and main-coverage checks. Do not return planned rows as installed search hits. Raw record integrity follows the SQLite durable storage boundary; source identity and canonical parse checks are required. It is not a claim of full-artifact digest verification.

## Task contracts

Parent copies complete relevant sections into CHECKLIST before CODE and owns publication. All tasks preserve concurrent DD03/reader work. No changes to unrelated language editions or second parser are authorized.

| ID | Do / exact owned files | Verify / Accept |
|---|---|---|
| OI-01 Canonical pluggable entry index | Own R/src/range/{index,index_source,archive,mod}.rs; add R/tests/offline_catalog.rs. Implement the exact records/index trait and archive delegation above; retain RemoteZimIndex API and existing open constructor. In-memory sparse provider uses original IDs and includes a redirect chain/resource. | `cargo test --manifest-path R/Cargo.toml --test offline_catalog` and existing `--test range_access`: unvisited sparse lookup, main redirect, resource, title search, unavailable target, cycles/hop bounds all pass without source-pivot reads; existing remote behavior passes. |
| OI-02 Durable bounded catalog | Own C/src/disk/{catalog,catalog_schema,catalog_discovery,catalog_budget,source,store,plan,mod}.rs and C/tests/offline_catalog.rs. Implement schema, purpose-budgeted metadata source, seed/scan/admit/prune, serialized quota lifecycle, offline provider and online persistence. Keep existing manifest policy partitions. Existing files >500lines split helpers naturally into the four named catalog modules. | `cargo test -p zim-cluster-cache --test offline_catalog` plus existing cache tests: reproduced100article case opens unvisited A/a00090 and searches it after restart; resource and multi-hop alias on uncached pages work; transfer cap bounds source calls and yields explicit partial usable subset; metadata pressure prunes selection before payload; mandatory main failure actionable; corruption/eviction cannot advertise missing payload; restart resumes cursor, source changed rejects; physical bytes within total and metadata partitions. |
| OI-03 Native plan/activation coverage | Own N/{prepare,install,persistence,mod}.rs, app native dto.rs, apps/reader/src/contracts.ts, apps/reader/src/routes/library/DynDonRoute.tsx, apps/reader/src-tauri/tests/dyndon_install.rs. Integrate finalized catalog plan and DTO; activate only available main and truthful counts, reopen catalog offline; preserve existing cancellation/job state flow. | Existing native `cargo test --manifest-path apps/reader/src-tauri/Cargo.toml --test dyndon_install` with expanded multi-page fixture: prepare/install/drop/reopen offline then open/search unvisited selected article/resource; no network requests after offline restart. App TypeScript build/test gate covers additional optional DTO and partial-plan copy. |

**Operator protocol after these gates:** use an actual Kiwix pack with metadata spanning several pages; install a bounded subset, quit/reopen offline, search and open an article never visited online, follow a retained redirect, and load a retained image/stylesheet. Observe separate planned/available counts and an honest unavailable result for unretained resources. This is a runtime verification protocol, not an idempotent checklist acceptance assertion.
