# EnZIME Reader — Comprehensive Codebase Analysis and Remediation Architecture

**Document date:** 31 October 2026, 18:05
**Analyzed version:** v1.6.55832 (working tree at commit `26db277`)
**Scope:** full-project, multi-domain static analysis (architecture, quality, security, performance) + dogma-compliant remediation solution
**Method:** three parallel domain analyses over the Tauri 2 + Rust workspace; static/read-only; no builds executed, no unit tests, no smoke tests
**Authority order:** `DOCS/ARCHITECTURE.md` (normative contract) → this document's solution section → `CHECKLIST.md` marker workflow (`KICKOFF-PROMPT.md`)

---

## 1. Project Profile

- **Product**: enzime-reader — offline-first knowledge reader for ZIM archives / EnZIME packs; Windows (NSIS installer + portable exe) and Android (universal APK) release targets.
- **Stack**: Tauri 2 shell (`apps/reader/src-tauri`), React 19 + Vite 6 + TypeScript 5 + zustand renderer, Rust workspace (21 members: 20 crates + shell), SQLite (`enzime.sqlite.v1`), vendored wllama3 wasm runtime.
- **Intended architecture** (ARCHITECTURE.md §3.4): `domain` ← `ports` ← adapters ← `application` ← shell.
- **Actual architecture**: renderer → `services/*.ts` (thin `invoke` wrappers) → 11 registered Tauri commands → single 1,987-line `ipc.rs` → `AppState` (one `Arc<Mutex<rusqlite::Connection>>`, open-archive map, volatile install-plan map). The `application` layer is bypassed entirely.
- **Milestone state** (CHECKLIST.md): Phase 0 gate never checked; Phase 1 mostly complete ("awaiting semantic attestation"); Phases 2–3 largely partial.

### 1.1 Raw metrics

| Metric | Value |
|---|---|
| Frontend TS/TSX source | 57 files / ~11.7 kLOC (no test runner exists) |
| Rust `crates/*/src` | 112 files / ~30.1 kLOC; 324 inline test fns in 40 files; integration tests only in `zim-anzimmerman` (27 fns) |
| Rust shell `src-tauri/src` | 4 files / ~2.1 kLOC (`ipc.rs` alone 1,987); zero tests |
| Zero-test crates | `domain`, `ports`, `storage-sqlite`, `zim-cluster-cache` (+ all of the above) |
| Supply chain | 567 Cargo.lock packages; 172 pnpm-lock entries |
| Version skew | root `1.6.55832` vs `apps/reader` package `1.0.52684` |
| Seam width | 11 registered commands vs ~40+ command names typed in frontend; 0 backend events |
| Working tree | uncommitted `pnpm-workspace.yaml`/`version.json`; ~14 untracked build artifacts/logs; `dist/` = 250 MB |

---

## 2. Architecture Findings

| # | Severity | Finding | Evidence |
|---|---|---|---|
| A1 | Critical | Documented `application` layer (13 modules, ~7.6 kLOC: `bootstrap`, `home`, `readiness`, `reader_nav`, `install`, `transfer`, `activation`…) has zero dependents; the shell re-implements `AppBootstrap` verbatim. Two parallel use-case implementations guarantee drift; ARCHITECTURE.md §3.4's graph is not what ships | `apps/reader/src-tauri/src/ipc.rs:22-27`, `:898-930` vs `crates/application/src/bootstrap.rs` |
| A2 | Critical | IPC wire contract unchecked and already drifting: frontend sends `reader_open { path }`; handler requires `pack_id` and ignores `path` — the "Open ZIM file" card cannot succeed. Two conflicting `ArticleResponse` TS shapes (readerService `rawHtml`/`mimeType` vs annotationService `sanitizedHtml`/`normalizedTextDigest`/…). No generated contract; `packages/contracts` absent though the workspace globs `packages/*` | `apps/reader/src/services/readerService.ts:75`, `:9-11`; `ipc.rs:990-993`; `annotationService.ts` |
| A3 | Critical | Windows — a declared release platform — cannot complete the DynDon install flow: `probe_free_bytes` returns `None` on non-unix (`// TODO for the platform-adapter crate`), and `crates/platform-adapter` is a 0-byte stub, so the free-space ladder always ESCALATEs and `install_plan` always fails on Windows. Violates A-03 (platform differences expressed only via cfg inside `platform-adapter`) | `ipc.rs:104-108`, `:1573-1597`; `crates/platform-adapter/src/lib.rs` (empty) |
| A4 | High | 10 of 21 crates are dead relative to the product binary: `application`, `object-store`, `catalog-openzim`, `sidecar`, `lore`, `entitlement`, `backup`, `diagnostics` + empty stubs `annotations`, `retrieval`. `platform-adapter` and `revenuecat-adapter` are empty yet **default features** of the reader (contradicting A-03's no-default-feature-omission rule). `inference-wllama` (~1.4 kLOC) is never invoked — shipped product has no inference/entitlement/backup/lore path despite the v1.6.55832 "windows-android release" label | `apps/reader/src-tauri/Cargo.toml:12-16`; grep: zero uses |
| A5 | High | Annotation vertical is a frontend wired to nothing: 16 components + stores + service (~4.9 of 11.7 kLOC) invoke `annotation_list` etc., which are registered nowhere; the service even anticipates Tauri's "command not found" — a dead-end caption (§0 violation) | `lib.rs:50-63` (11 commands, none annotation); `annotationService.ts:355-365` |
| A6 | High | `install_start` (~235-line IPC handler) re-implements the install pipeline, bypassing `object-store`'s stage/verify/atomic-rename/quarantine (A-07); buffers every selected cluster in RAM simultaneously; writes via plain `tokio::fs::write`; no pause/resume/cancel (P3-04 requires resumable) | `ipc.rs:1290-1526`, `:1361`, `:1392` |
| A7 | High | Bootstrap advertises a product that does not exist: routes `/ask /notes /memory /settings /setup` all `enabled:true` while the router registers only `/`, `/library`, `/reader/*`; `capabilities` always empty; `map_pack` hard-codes `verification_state:"verified"`; trust state "M1: no signature infrastructure wired" | `ipc.rs:222-231`, `:927`, `:492`, `:1669-1673`; `router.tsx:17-28` |
| A8 | Medium | Dependency-direction violations: `zim-cluster-cache → zim-anzimmerman` (adapter→adapter, reusing concrete `layout_snapshot()`); `lore → storage-sqlite` (+ its own `Mutex<Connection>`); shell imports `domain`+`storage-sqlite`+4 adapters+`ports` directly. Positive: `domain` is a clean dependency-free leaf; no cycles | `crates/zim-cluster-cache/Cargo.toml:11`; `crates/lore/Cargo.toml:11`; `apps/reader/src-tauri/Cargo.toml:30-37` |
| A9 | Medium | Frontend infrastructure duplicated: `stringifyError`/`classifyError`/`call` trio copy-pasted ×3 with a fourth divergent normalizer in annotationService; wire DTOs re-declared per service | `readerService.ts:40-70`, `homeService.ts:102-132`, `libraryService.ts` |
| A10 | Medium | Dead event machinery: zero `emit` in the backend; frontend `EventBridge` subscribes to events never published; job progress observable only by polling `job_list` | `components/shell/providers.tsx:116-131` |
| A11 | Medium | `"csp": null` in `tauri.conf.json` (§18.3 mandates strict CSP); bundle targets `deb/appimage/msi` diverge from the actually shipped NSIS exe + APK | `apps/reader/src-tauri/tauri.conf.json:27` |
| A12 | Medium | Documented layout vs reality: §5 tree missing `packages/`, `fixtures/`, top-level `tests/`, 3 of 4 `tools/` subdirs, `LICENSES/`, `schemas/ipc/`; `dyndon` and `zim-cluster-cache` — the actual M1 core — undocumented; the normative `enzime.ipc.v1` schema file does not exist | `DOCS/ARCHITECTURE.md` §5, line 14 |
| A13 | Low | Reader resource gap: article images reference relative paths that never resolve (no custom protocol registered); `ClusterCacheArchive` returns empty `internal_links`/`normalized_text` by design, silently degrading §7.3 for DynDon packs | `ArticleContent.tsx:21-25`; `crates/zim-cluster-cache/src/lib.rs` header |
| A14 | Low | Repository-boundary leak: ~350 lines of hand-written SQL inside `ipc.rs` ("tables not covered by storage-sqlite::query at M1") | `ipc.rs:576-892` |

**Largest files**: `ipc.rs` 1,987 · `lore/service.rs` 1,531 · `application/reader_nav.rs` 1,364 · `application/sidecar.rs` 1,340 · `zim-anzimmerman/archive.rs` 1,133 · `application/transfer.rs` 1,052 · `application/activation.rs` 965. Frontend: `AnnotationWorkspace.tsx` 708 · `home/shared.tsx` 615 · `LibraryView.tsx` 595.

---

## 3. Quality Findings

**Verdict**: an unusually disciplined core — zero TODO/FIXME debt in 30 kLOC of Rust, `#![forbid(unsafe_code)]` in every crate (one documented `unsafe` block total), strict TypeScript with zero `any`/`@ts-ignore`, ~30 typed `thiserror` enums, spec-referenced doc comments — wrapped in an integration layer that violates the no-placeholder rule in letter and spirit. The leverage is in finishing the M1 seams, not touching the core.

### 3.1 Placeholder-rule violations (first-class operator rule)

- **Q1 [Critical]** Four empty crates shipped as workspace members, two compiled into every default build: `crates/{annotations, platform-adapter, retrieval, revenuecat-adapter}/src/lib.rs` all 0 bytes; `apps/reader/src-tauri/Cargo.toml:13` enables `platform-adapter` + `revenuecat-adapter` in `default` features.
- **Q2 [High]** Android SAF backup byte pipe is an admitted stub: `crates/backup/src/lib.rs:40-42` ("stubbed to `Err(Destination)` … swap when ports::platform lands"), `destination.rs:27-28,59` — Android backups cannot work.
- **Q3 [High]** "Verified" labeling theater: `ipc.rs:1398` computes `sha256_hex(&buf)`, then upserts chunk state `"verified"` + `bytes_verified` (~`:1410-1425`) **without ever comparing to an expected digest** (per-cluster expected digests absent from the plan; whole-artifact `expected_sha` in the job row never checked). Content-integrity signaling that verifies nothing, in the core install path.
- **Q4 [Medium]** Signature infrastructure is built (`sidecar` Ed25519 verify + quarantine; `catalog-openzim` signature states) but unwired in the install path: `ipc.rs` ~`:1664-1670` ships `signature_url: None`, `trust.state: "unverified"` — packs flow with no signature check.
- **Q5 [Medium]** Windows free-space probe TODO in production (see A3).
- **Q6 [Low]** Stale comment `appStore.ts:13` claims the bootstrap handler is unwired (it is wired at `ipc.rs:899`, registered `lib.rs:51`); dead `_selection_profile_id` parameter threaded through `build_install_plan` (`ipc.rs:1615`).

### 3.2 Test coverage (observation only — see §7 prohibition)

- **Q7 [High]** The two newest layers have zero tests: the TypeScript app has no test runner at all; `ipc.rs` (1,987 LOC, 11 commands) zero; `storage-sqlite` (899 LOC persistence heart) zero; also `domain`, `ports`, `zim-cluster-cache`. (Per the operator's 2026-10-31 prohibition, remediation does **not** add unit tests — regression safety is supervising semantic review + screencast rubric evidence.)

### 3.3 Panic risk, error handling, duplication

- **Q8 [Medium]** Eight lock-poisoning `.expect("catalog snapshot lock poisoned")` in catalog hot paths (`catalog-openzim/src/openzim_xml.rs:154,171,188,207`; `enzime_json.rs:142,159,176,195`) — one poisoned lock panics every subsequent catalog read; the codebase norm elsewhere degrades gracefully.
- **Q9 [Medium]** The entire IPC layer flattens errors to `String` (`err_str`, `ipc.rs:78-81`; all 11 handlers `Result<_, String>`), discarding the typed-error discipline and forcing frontend string-matching.
- **Q10 [Medium]** 21+ wire DTOs duplicated between `ipc.rs` and `crates/application` — self-acknowledged at `ipc.rs:22-27` ("once `application` compiles … this duplication will collapse"); it compiles; it never collapsed. Confirmed pairs: `AppBootstrap`, `HomeSnapshot`, `SystemReadiness`, `PackManifest`, `InstallPlan`, …
- **Q11 [Medium]** Five hand-rolled hex/sha256-hex encoders across crates (`lore/service.rs:79` ≡ `storage-sqlite/migration.rs:70`; `zim-anzimmerman/archive.rs:322,396`; `zim-cluster-cache/cache_archive.rs:228`) — digest formatting is integrity-relevant; drift risk.
- **Q12 [Low]** `panic!`/`unreachable!` in the canonical CBOR encoder (`sidecar/cbor.rs:133,169,261,302`); 9× `serde_json::to_value().unwrap()` in `diagnostics/audit.rs:148-208`; `transfer-http/lib.rs:65` constructor `.expect`; `sidecar/provider.rs:103,131` keystore lock `.unwrap()`; `Result<(),String>` fn-pointers in `object-store/migration.rs:64-85`; signature error detail discarded (`catalog-openzim/openzim_xml.rs:338-341`); best-effort fail transitions emit no diagnostics (`application/transfer.rs:789-905`); two unchecked `as unknown as` double-casts (`libraryStore.ts:115`, `annotationService.ts:324`).

**Explicitly cleared**: all 7 TS empty `catch {}` blocks carry fallbacks and comments; zero `println!`/`dbg!`/`console.log` residue; all `panic!("expected…")` hits are `#[cfg(test)]`; only 11 non-test `unwrap()`s in 30 kLOC.

---

## 4. Security Findings

**Verdict**: strong core engineering; the main hardening gap is the missing app-shell CSP.

- **S1 [High]** `"csp": null` app-wide (`tauri.conf.json:26-28`, also baked into the Android assets copy) — §18.3 mandates a strict CSP; the article iframe's CSP is exemplary but the app shell has none.
- **S2 [Medium]** Entitlement unwired + trust anchor unpinned: zero capability checks among the 11 registered commands; `revenuecat-adapter`/`platform-adapter` are 0-byte stubs (nothing to leak today); the issuer verifying-key registry is public/unpinned (`sidecar/provider.rs:44-55`) — when wired, a user could self-register a key and self-sign grants.
- **S3 [Medium]** Entitlement artifacts are replayable across devices: `verify_artifact`/`decide_inner` never bind `subject` to device identity (`entitlement/artifact.rs:300-339`, `policy.rs:125-257`).
- **S4 [Medium]** `not_before_ms` never checked — future-dated grants honored immediately (`policy.rs:196-229`).
- **S5 [Low]** Cleartext `http://` accepted for pack downloads (`ipc.rs:114`; `catalog-openzim/validate.rs:80-85`) — integrity saved by SHA-256-on-open, but Android release blocks cleartext → inconsistent cross-platform behavior.
- **S6 [Low]** Android `allowBackup` not disabled, no data-extraction rules (`gen/android/.../AndroidManifest.xml:8-12`) — SQLite DB auto-backed-up to the Google account; interacts with S3.
- **S7 [Low]** Backup restore trusts manifest-supplied Argon2 parameters (KDF DoS on a hostile `.enzime` file, `backup/crypto.rs:55-85`); restore directory token used as a raw path (`backup/restore.rs:210-215`, unwired today).

**Verified strengths**: 11-command IPC surface minimal and input-validated (clamps, empty-string rejection); no shell/opener/fs plugins; no remote content in the webview; every SQL statement parameterized (zero injection); article rendering boundary exemplary (sandboxed iframe without `allow-scripts`/`allow-same-origin`, inert-DOMParser sanitizer, per-article `default-src 'none'` CSP, regex fallback, non-HTML forced to `<pre>`); zero `dangerouslySetInnerHTML`/`eval`/`innerHTML` in the renderer; rustls-only with no invalid-cert acceptance, strict range/ETag verification, bounded streams; path traversal defended (`object-store/layout.rs:163-199`); ZIM decompression bombs capped (2 GiB streaming cap, 256 MiB index cap); no archive extraction (zip-slip N/A); secrets sweep otherwise clean; Android release Gradle sane (R8 + minify, cleartext off, no exported components beyond the launcher).

---

## 5. Performance Findings

**Verdict**: boot path and renderer lean; the DynDon install/read pipeline has three structural problems that M1-sized data hides and Wikipedia-class packs will expose.

- **P1 [High]** Unbounded memory: the "LRU" is a plain growing `HashMap` — `DEFAULT_LRU_CAPACITY = 8` is dead code and the comment admits no eviction happens (`zim-cluster-cache/cache_archive.rs:42,52-71`); `new_offline` decompresses **every** cluster eagerly (`:99-116`); install buffers all raw clusters in RAM (`ipc.rs:1361-1399`); `AppState.archives` has no removal path (`state.rs:37`). Resident ≈ full decompressed pack size per pack for process lifetime; ~2 GB pack ⇒ ~4 GB transient — OOM territory on Android.
- **P2 [High]** `install_plan` blocks a tokio worker: inline `open_archive` runs the full-file SHA-256 synchronously (`ipc.rs:1700-1708`; `archive.rs:264-267`) and `build_path_index` does one seek+read per directory entry — 100k+ random I/Os for a Wikipedia-class ZIM (`archive.rs:1087-1110`). `layout_snapshot` correctly uses `spawn_blocking`; `open` does not.
- **P3 [High]** `pack_list` N+1 with broken paging: per pack, a selection query + `serde_json::from_str` of canonical JSON embedding the **full ClusterDirectory** + a `MAX(visited_at_ms)` query (`ipc.rs:1146-1205`); `recent_packs` ignores `offset`, so page 2+ re-returns page 1 while `next_offset` advertises progress; `total = items.len()`.
- **P4 [Medium]** All DB access funnels through one `Mutex<Connection>` with `synchronous=FULL` (`state.rs:33`; `migration.rs:87-96`); the renderer fires three parallel invokes that serialize (`libraryStore.ts:239-250`); install commits once **per cluster** plus five job transitions (`ipc.rs:1402-1430`) — N fsyncs dominate install time.
- **P5 [Medium]** Reader hot path: single-slot cluster cache in `zim-anzimmerman` (`archive.rs:238-242`) — HTML/image interleaving thrashes up to 2 GiB per swap; `zim-cluster-cache` lookups are O(N) linear scans and `verify_structure` is O(N²) (`cache_archive.rs:283-289,343-348,395-399,482-532`); `article()` adds a full HTML parse + SHA-256 per fetch (`archive.rs:664-676`).
- **P6 [Low]** Splash enforces a 1.4 s minimum hold + 320 ms fade (`Splash.tsx:9-11`) — deliberate, but the largest fixed startup cost. Catalog `LIKE '%…%'` defeats its index; missing `reader_history(pack_id)` index (`ipc.rs:703-736`, `:820-828`). `EventBridge` snapshots listeners at mount — later registrations are never subscribed (`providers.tsx:120-139`). Dead disk writes: install writes cluster files that are never read back (`ipc.rs:1391-1394` vs `:1503-1506`); archives are volatile ⇒ after restart `reader_open` errors and re-install re-downloads everything. Dead crates compiled into the release binary; no `[profile.release]` tuning.

---

## 6. Cross-Domain Synthesis (ranked)

| Rank | Finding | Domains | Ship impact |
|---|---|---|---|
| 1 | **A2** IPC contract drift — `reader_open {path}` vs `pack_id` handler | Arch+Qual | "Open ZIM file" UI flow **cannot succeed** |
| 2 | **A3/Q1/Q5** Windows DynDon install always fails (`probe_free_bytes → None`, empty `platform-adapter`) | Arch+Qual | Flagship flow dead on a declared release platform |
| 3 | **Q3** Install chunks marked `"verified"` with the digest never compared | Qual | Integrity theater in the core install path |
| 4 | **P1** Unbounded RAM: eager full-pack decompression, never-evicting cache, no archive removal | Perf | OOM-kill on Android with real packs |
| 5 | **A1/Q10** Orphaned `application` layer duplicated by the 1,987-line `ipc.rs` (21+ DTOs ×2) | Arch+Qual | Root cause of rank 1; every future feature drifts |
| 6 | **Q7** Zero tests on TS app / `ipc.rs` / `storage-sqlite` / `domain` / `ports` | Qual | No regression safety net (remediation per §7: semantic review + screencast, not tests) |
| 7 | **A7** Bootstrap advertises routes/capabilities/`verified` states that do not exist | Arch+Qual | Product misrepresents itself to its own UI |
| 8 | **P3** Library N+1 full-JSON deserialize + paging that re-returns page 1 | Perf | Library degrades with pack size; scroll broken |
| 9 | **A4/A5** 10/21 crates dead; annotation vertical (~42% of frontend) wired to nothing | Arch+Qual | Placeholder-rule violations; misleading codebase |
| 10 | **A11/S1** `"csp": null` app shell | Arch+Sec | §18.3 violation; no containment backstop |

---

## 7. Remediation Solution — dogma-compliant iteration

**Operator directives recorded 2026-10-31**: (1) architect the solution first as a snapshot of the completed codebase per KStore/Admin-Manual conventions; (2) the checklist is a **recipe of four-state steps that are fully parallelizable — no dependencies; every task codable in or out of any order** (states: `[ ]` → `[/]` → `[X]` → `✅`); (3) coder subagents dispatched **in parallel**; (4) returned work is evaluated **only semantically against the task — no grepping**; (5) the rubric is evidenced **by screencast**; (6) **unit-testing and smoke-testing are prohibited** (build/type-check gates are prerequisites only, never evidence). The organization-wide signing setup is intentional per operator and is out of scope for analysis output.

### 7.1 ARCHITECT boundary (documentation only; no production source changes)

1. **This document** (analysis + solution) is the audit input.
2. **Architecture snapshot amendment** → `DOCS/ARCHITECTURE.md` (timestamped `.bak` first, per repo convention). The amended contract describes the **completed codebase end state**:
   - **§14.4 command registry**: single `reader_open(pack_id)` contract; one `ArticleResponse` shape; per-domain command modules replace the monolithic `ipc.rs` (this decomposition is the *enabler of order-free parallelism*: no shared file is contended).
   - **A-03 conformance**: `platform-adapter` owns the Windows `GetDiskFreeSpaceExW` free-space probe behind cfg; `capability_report` is real.
   - **A-07 conformance**: the plan carries per-cluster expected digests; `"verified"` is granted **only** on digest match; object-store stage/verify/atomic-rename governs all pack writes; streaming install (no full-pack RAM retention); install writes are the object-store's staged objects (eliminating the dead cluster-file writes).
   - **Bounded residency**: true LRU with eviction, lazy decompression, archive removal from `AppState`.
   - **Library contract**: OFFSET-correct paging; no ClusterDirectory deserialization per render.
   - **Truthful composition**: bootstrap routes/capabilities/readiness reflect what ships.
   - **§18.3 conformance**: strict app-shell CSP in `tauri.conf.json` + the Android assets copy.
   - Registry discipline: amendments keep every registry internally executable (P0-04: no duplicate/undefined identifiers).
3. **Checklist recipe** → `CHECKLIST.md` new wave section; each dispatched task includes its full row (KICKOFF-PROMPT). **The checklist is four-state**: every step advances through exactly four marker states —
   - **`[ ]` unstarted** — recipe row locked (architecture clause + Allowed files / Do / Verify / Accept authored);
   - **`[/]` in execution** — coder subagent implements exactly the recipe against the locked clause;
   - **`[X]` implemented** — work returned; supervising semantic evaluation against the task passed (no grep);
   - **`✅` verified** — independent semantic re-read confirms confluence AND the step's screencast rubric evidence is observed.

   **Order-freedom is structural**: no task's Do/Verify/Accept references another task, its output, or its marker state; every task codes against the amended normative contract alone; registration lines are specified verbatim per domain (additive, identical for any coding order). Any step may occupy any state independently of every other step — the wave is fully parallelizable.
   - **T1 Frontend contract** — Allowed: `apps/reader/src/{services,stores,components}`. pack_id open flow; single `ArticleResponse` DTO per §14.4; shared invoke/error helpers (collapses A9).
   - **T2 Platform probe** — Allowed: `crates/platform-adapter/**`, `crates/ports/src/platform.rs`, `src-tauri/src/commands/platform.rs`. Windows free-space probe behind cfg; real `capability_report`.
   - **T3 Verified streaming install** — Allowed: `crates/application/src/install.rs`, object-store wiring, `src-tauri/src/commands/install.rs`. Expected per-cluster digests; verify-before-`"verified"`; stage/verify/atomic-rename; bounded buffering.
   - **T4 Bounded cluster cache** — Allowed: `crates/zim-cluster-cache/**`. True LRU with eviction; lazy decompression; bounded residency; archive removal API.
   - **T5 Truthful library/bootstrap** — Allowed: `crates/application/src/{library,home}.rs`, `src-tauri/src/commands/library.rs`. OFFSET-correct paging; truthful routes/capabilities/readiness.
   - **T6 Strict CSP** — Allowed: `apps/reader/src-tauri/tauri.conf.json`, `gen/android/.../assets/tauri.conf.json`. §18.3 policy for the app shell.
4. **Rubric re-formed** → `DOCS/TEST_RUBRIC.md` new dated section (SC4 gauntlet form): one screencast-observable row per task — Windows DynDon plan computed from real Windows free space; open-by-pack_id renders an article; digest-mismatch job visibly fails unverified; resident memory stays bounded across a multi-pack install; Library page 2 returns distinct packs; app-shell CSP observed in devtools; boot capabilities truthful. Rows are independently demonstrable in any order; two-outcome verdict SHIP-READY / DEFECTIVE; R-OPERATOR sign-off; screencasts archived per TC11 (Admin-Manual path if present, else `LOGS/SCREENCASTS/`).
5. **Commit + push** at the boundary (separate commits: this report; architecture amendment + checklist + rubric); verify remote SHAs.

### 7.2 Parallel CODER wave

Dispatch **T1…T6 as parallel coder subagents, each in its own worktree isolation**, each receiving its full recipe row verbatim. Because no task depends on another, dispatch is all-at-once and **merges accept in any order**. Each subagent ends with its own commit; its step advances `[ ]`→`[/]` on dispatch and `[/]`→`[X]` on returned, semantically-conformant work. **No unit tests, no smoke tests are written or run.**

### 7.3 Supervising semantic REVIEW

As each subagent returns, evaluate the work **only by reading it against its task clause** — semantic conformance to the amended architecture (no grep, no test execution). Corrections return to the same subagent with the specific semantic gap; on confluence the step holds `[X]` until independent verification.

### 7.4 SCREENCAST rubric evidence (`✅`)

Build the Windows target (prerequisite gate only), record each rubric row as a screencast, archive per TC11, render the two-outcome verdict; R-OPERATOR attests. Only then does the next change's architecture amendment begin (operator rule 4 cadence).

### 7.5 Deferred roadmap (later iterations, priority-ranked — executable order-free when architected)

- **P2 structural**: wire-or-remove decision per dead crate (backup→SAF via real `ports::platform`, inference-wllama, entitlement, sidecar; empty crates cannot remain default features); typed IPC error contract replacing `Result<_, String>`; shared digest-hex utility; one hex/`sha256` implementation.
- **Entitlement hardening when wired**: pin issuer keys at compile time; bind subject to device identity; check `not_before_ms`.
- **P3 polish**: catalog FTS + `reader_history(pack_id)` index; `synchronous=NORMAL` + batched install transactions; second cluster slot in `zim-anzimmerman`; `EventBridge` re-arm; ARCHITECTURE §5/doc-reality alignment; version skew cleanup (root vs `apps/reader`); repo hygiene (`.bak` files, swarm logs).

---

## 8. Method note

Static analysis only, as required: no compilation, no runtime execution, no unit tests, no smoke tests were performed for this analysis. Findings carry file:line evidence for supervising semantic review. Domain analyses were produced by three parallel read-only analysis agents (architecture; quality; security+performance) and synthesized by the supervising architect.
