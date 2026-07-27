# EnZIME Reader Reboot IV — Fresh Execution Checklist

**Initial state:** every executable task is unstarted.
**Rule:** each dispatched task includes the full row below.
**Scope:** complete Android, Linux and Windows public release.

Marker workflow is governed by `KICKOFF-PROMPT.md`.

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

- [/] **P1-03 Implement ZIM ports**
  - Allowed files: `crates/ports/src/zim.rs`, module export, tests.
  - Do: implement exact §7 public types and `ZimEngine`/`ZimArchive`.
  - Verify: public signature snapshot equals architecture.
  - Accept: port imports no AnZIMmerman/provider crate.

- [/] **P1-04 Implement catalog and transfer ports**
  - Allowed files: `crates/ports/src/catalog.rs`,
    `crates/ports/src/transfer.rs`, exports, tests.
  - Do: implement exact §8 traits/types/errors.
  - Verify: signature/serialization snapshots pass.
  - Accept: application can plan without HTTP/file concrete imports.

- [/] **P1-05 Implement annotation and signature ports**
  - Allowed files: domain annotation/trust modules, ports signature module,
    tests.
  - Do: implement §9 types/traits and validation.
  - Verify: selector bounds and signature-domain tests pass.
  - Accept: multi-selector and trust states are complete.

- [/] **P1-06 Implement inference, embedding and retrieval ports**
  - Allowed files: inference/embedding/retrieval port/domain modules, tests.
  - Do: implement exact §§10–11 types and traits.
  - Verify: signature/serde snapshot tests pass.
  - Accept: no runtime provider imported.

- [/] **P1-07 Implement lore and entitlement ports**
  - Allowed files: lore/entitlement domain/port modules, tests.
  - Do: implement exact §§12–13 entities, traits and errors.
  - Verify: state/policy unit tests pass.
  - Accept: candidate cannot become accepted memory implicitly.

- [/] **P1-08 Apply SQLite initial migration**
  - Allowed files: `schemas/sqlite/0001_initial.sql`,
    `crates/storage-sqlite/`, tests.
  - Do: transcribe §6 executable SQL exactly and implement migration runner.
  - Verify: empty migration, FK check, index/trigger inventory pass.
  - Accept: schema identifiers equal §16.4.

- [ ] **P1-09 Generate TypeScript contracts**
  - Allowed files: `packages/contracts/`, `tools/contract-gen/`, tests.
  - Do: generate exact §14 entities/DTOs/command/event/route unions.
  - Verify: Rust↔TS round-trip and registry equality pass.
  - Accept: generated files are checked in and reproducible.

- [ ] **P1-10 Create complete platform shells**
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

- [ ] **P2-01 Implement `AnZimmermanZimEngine`**
  - Allowed files: `crates/zim-anzimmerman/`, ZIM fixtures/tests.
  - Do: bind exact `ZimEngine`/`ZimArchive` methods to vendored source.
  - Verify: all §7.4 legal and malicious fixtures pass.
  - Accept: no panic, unbounded allocation or unsupported silent result.

- [ ] **P2-02 Implement object store**
  - Allowed files: `crates/object-store/`, tests.
  - Do: staging/hash/fsync/atomic-rename/quarantine algorithm from §6.3.
  - Verify: forced interruption/corruption/duplicate object tests pass.
  - Accept: unverified bytes cannot enter active objects.

- [ ] **P2-03 Implement pack import/activation**
  - Allowed files: application library service, SQLite repositories, tests.
  - Do: local import plan, verify, `install_transactions`, active pointer.
  - Verify: valid import activates; corrupt import quarantines; previous stays.
  - Accept: A-04 and A-07 hold.

- [ ] **P2-04 Implement safe article renderer**
  - Allowed files: reader service, sanitizer, scoped protocol, renderer tests.
  - Do: §7.3 pipeline/CSP/message validation.
  - Verify: scripts/network/native bridge blocked; resources/links work.
  - Accept: untrusted article cannot invoke privileged code.

- [ ] **P2-05 Implement reader search/navigation**
  - Allowed files: application reader, reader repositories, tests.
  - Do: title search, open/main/internal navigation, history/position/bookmark.
  - Verify: real fixture search/navigation/restart tests pass.
  - Accept: reader works offline with no model.

- [ ] **P2-06 Implement text annotation vertical slice**
  - Allowed files: annotation service/resolver/repositories, tests.
  - Do: create/update/delete/restore/list highlight/comment with selector bundle.
  - Verify: restart/re-anchor/ambiguity/source-unavailable tests pass.
  - Accept: annotation outlives current article bytes.

- [ ] **P2-07 Implement minimal complete Reader UI**
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

- [ ] **P3-01 Implement catalog adapters**
  - Allowed files: catalog adapter crates, catalog repositories, fixtures/tests.
  - Do: OpenZIM XML, EnZIME JSON, local manifest normalization/trust.
  - Verify: signed/unsigned/malformed/offline fixture matrix passes.
  - Accept: UI sees only canonical `CatalogEntry`.

- [ ] **P3-02 Implement HTTP and file transfer adapters**
  - Allowed files: transfer adapter crates, tests.
  - Do: bounded probe, valid ranges, full/local streams, retry/source-change.
  - Verify: ignored range, oversized response, ETag change tests pass.
  - Accept: no `200` range response is buffered as a chunk.

- [ ] **P3-03 Implement install planner**
  - Allowed files: application library planner, storage repository, tests.
  - Do: calculate final/temp/retained/free impact and verification plan.
  - Verify: fit/temporary-shortfall/untrusted-source matrices pass.
  - Accept: `InstallPlan` contains every Download Plan field.

- [ ] **P3-04 Implement resumable download worker**
  - Allowed files: application job worker, download repositories, tests.
  - Do: persisted chunks, pause/resume/cancel/retry/verify.
  - Verify: process-kill resume repeats no verified chunk.
  - Accept: state transitions obey SQL guard.

- [ ] **P3-05 Implement atomic candidate activation**
  - Allowed files: install service/repositories/object store, tests.
  - Do: staged verification then active pointer and retained prior.
  - Verify: failures at each transaction phase preserve last active.
  - Accept: `E2E-02-interrupted-download` passes.

- [ ] **P3-06 Implement Library UI**
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

- [ ] **P4-01 Implement notebook/tag/layer repositories**
  - Allowed files: storage repositories/application annotation service/tests.
  - Do: CRUD/query hierarchy/tags/layer visibility.
  - Verify: FK/delete/idempotence/filter tests pass.
  - Accept: cross-pack Note filters use canonical tables.

- [ ] **P4-02 Implement ink and voice assets**
  - Allowed files: annotation service/object store/platform audio/tests.
  - Do: chunked asset upload, digest commit, Opus voice and SVG ink.
  - Verify: size/hash/cancel/permission/process-death tests pass.
  - Accept: no partial asset is referenced.

- [ ] **P4-03 Implement canonical sidecar codec/signing**
  - Allowed files: sidecar/signature crates, CDDL, golden fixtures/tests.
  - Do: deterministic CBOR, artifact ID, Ed25519 domain sign/verify.
  - Verify: golden bytes and domain-separation tests pass.
  - Accept: repeated export with same canonical input is byte-identical except
    declared creation-time input.

- [ ] **P4-04 Implement sidecar plan/import/export**
  - Allowed files: application sidecar service, repositories, tests.
  - Do: choose/plan/conflict/idempotent import and explicit export.
  - Verify: valid/unsigned/invalid/duplicate/divergent matrix passes.
  - Accept: invalid signatures never activate.

- [ ] **P4-05 Implement Annotation Workspace and Notes UI**
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

- [ ] **P5-01 Implement model catalog/install lifecycle**
  - Allowed files: model domain/repository/application/object-store tests.
  - Do: model manifests, plan, verify, install/select/remove.
  - Verify: corrupt/incompatible/insufficient-space/active-session tests pass.
  - Accept: no automatic model download.

- [ ] **P5-02 Implement LiteRT-LM backend**
  - Allowed files: `crates/inference-litert/`, vendored adapter boundary, tests.
  - Do: exact backend/session trait with streaming/cancel/health/model validation.
  - Verify: certified model fixture runs on Android and desktop.
  - Accept: runtime failure maps to `InferenceError`.

- [ ] **P5-03 Implement desktop GGUF backend**
  - Allowed files: `crates/inference-gguf/`, desktop composition/tests.
  - Do: exact backend/session trait for Linux/Windows.
  - Verify: desktop model fixture streams/cancels; Android graph excludes crate.
  - Accept: contracts do not expose provider-specific fields.

- [ ] **P5-04 Implement indexing and embeddings**
  - Allowed files: retrieval/storage/inference embedding modules/tests.
  - Do: document/chunk projection, FTS sync, embedding spaces/vectors/rebuild.
  - Verify: deterministic chunking, FTS and model-replacement tests pass.
  - Accept: user primary data is not stored only in an index.

- [ ] **P5-05 Implement hybrid retrieval/context**
  - Allowed files: retrieval crate/application ask/tests.
  - Do: lexical/vector merge, dedupe, rerank, diversity, §11.3 budget.
  - Verify: golden retrieval/context fixtures pass.
  - Accept: evidence list is bounded and locators exact.

- [ ] **P5-06 Implement cited conversation service**
  - Allowed files: application ask, repositories, tests.
  - Do: conversation/message/run, stream events, cancel, citation validation.
  - Verify: known/unknown/zero-citation/failure/restart tests pass.
  - Accept: assistant success cannot contain invalid citation records.

- [ ] **P5-07 Implement Ask UI**
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

- [ ] **P6-01 Implement lore repositories/policy**
  - Allowed files: lore/storage crates and tests.
  - Do: evidence/candidate/memory/revision/evidence links and forbidden classes.
  - Verify: FK, revision conflict, forbidden promotion tests pass.
  - Accept: accepted memory requires explicit/policy evidence.

- [ ] **P6-02 Implement extraction/consolidation**
  - Allowed files: lore service/retrieval projection/tests.
  - Do: opt-in candidate extraction, duplicate merge and correction proposal.
  - Verify: source quotation and sensitive inference rejection fixtures pass.
  - Accept: consolidation cannot accept/delete/rewrite user corrections.

- [ ] **P6-03 Implement correction/pin/delete/retrieval**
  - Allowed files: lore/retrieval services/repositories/tests.
  - Do: revisioned correction, pin, tombstone deletion and projection removal.
  - Verify: `CT-LORE-002` and revision/evidence tests pass.
  - Accept: deleted text is absent from Ask/diagnostics.

- [ ] **P6-04 Implement Memory UI**
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

- [ ] **P7-01 Implement signed entitlement codec/policy**
  - Allowed files: entitlement/signature/storage crates, CDDL, tests.
  - Do: deterministic artifact verify and capability decisions.
  - Verify: valid/perpetual/term/expired/invalid/revoked matrix passes offline.
  - Accept: reader core and user export never denied.

- [ ] **P7-02 Implement RevenueCat adapter**
  - Allowed files: RevenueCat adapter/platform compositions/tests.
  - Do: normalize refresh/restore events into provider events and signed grants.
  - Verify: provider outage/malformed event/replay tests pass.
  - Accept: domain/UI import no RevenueCat SDK type.

- [ ] **P7-03 Implement coupon ledger**
  - Allowed files: entitlement service/storage/tests.
  - Do: signed coupon verify, receipt derivation, local single redemption and
    later reconciliation.
  - Verify: duplicate/cross-device/expired/invalid fixtures pass.
  - Accept: reconciliation cannot revoke unrelated ownership/data.

- [ ] **P7-04 Implement backup creation**
  - Allowed files: backup/object-store/platform/signature/tests.
  - Do: versioned manifest, optional encryption, atomic export and verification.
  - Verify: wrong destination/disk full/wrong passphrase/interruption tests pass.
  - Accept: backup is marked verified only after read-back validation.

- [ ] **P7-05 Implement restore plan/commit**
  - Allowed files: backup/application/storage/tests.
  - Do: validate/migrate/preview conflicts/stage/atomic restore.
  - Verify: all conflict policies and source-absent representation pass.
  - Accept: failed restore leaves current data unchanged.

- [ ] **P7-06 Implement diagnostics/privacy controls**
  - Allowed files: diagnostics/application/settings UI/tests and privacy docs.
  - Do: redacted inspectable bundle, export/delete controls, explicit sample opt-in.
  - Verify: secret/path/content redaction fixture passes.
  - Accept: support evidence needs no broad private content.

- [ ] **P7-07 Implement Settings UI**
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

- [ ] **P8-01 Implement Home UI**
  - Allowed files: named Home components/store/services/styles/tests.
  - Do: populated/empty/loading/offline/error/readiness states.
  - Verify: §24 bindings, keyboard/screen-reader/mobile tests pass.
  - Accept: each task/readiness action routes to a real complete flow.

- [ ] **P8-02 Implement Setup UI**
  - Allowed files: named Setup components/store/services/styles/tests.
  - Do: probe/path/model/complete flows including Skip model.
  - Verify: each path reaches usable Home/Library; process restart resumes.
  - Accept: AI is not required or deceptively implied.

- [ ] **P8-03 Apply Aurora Glass system**
  - Allowed files: design tokens, UI primitives, component styles/tests.
  - Do: implement exact §15.6 tokens and usage rules on all screens.
  - Verify: token inventory and contrast/scale/reduced-motion tests pass.
  - Accept: long article text is never on low-contrast glass.

- [ ] **P8-04 Reconcile verified Stitch screens**
  - Allowed files: component layout/styles and
    `STITCH/generation-evidence.json`.
  - Do: use only screens with successful `get_screen` evidence as layout input.
  - Verify: entity names and §24 bindings unchanged.
  - Accept: visual drafts improve layout without altering contracts.

- [ ] **P8-05 Validate full component binding manifest**
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
  - Allowed files: `DOCS/TEST-RUBRIC.md`, evidence.
  - Do: rerun all seven E2E stories and release gates from clean installs.
  - Verify: all pass on signed candidate artifacts.
  - Accept: candidate is SHIP-READY.

### Final gate

- [ ] **P10-GATE Ship EnZIME Reader 4.0**
  - Allowed files: signed release manifest and immutable evidence.
  - Do: independent verifier reviews source commit, artifacts, SBOM, capability
    evidence, E2E, security, performance, accessibility and recovery.
  - Verify: every checklist task is independently verified; tree clean.
  - Accept: publish exact signed artifacts without post-verification mutation.

