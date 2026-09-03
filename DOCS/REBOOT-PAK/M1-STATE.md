# EnZIME Reader — Milestone 1 State (compaction-safe recovery doc)

**Success gate (operator directive 2026-07-29):** a **recorded screencast of the
built Linux AppImage** actually reading a ZIM article offline. The web/vite app
is source-testing only. The gate is formalised in `DOCS/TEST_RUBRIC.md` (SC4).
Priority is **thoroughness/correctness, not speed** — but via a parallel coder
swarm (architected self-contained tasks), not solo hand-coding.

## What "broken" means
The app booted to a blank screen (`App.tsx` stub `<h1>` views shadowing real
components) and **produced no reading** — the ZIM engine exists and is attested
working (P2-01 ✅), but nothing wired it to the UI. Root cause = a 3-layer
integration seam gap (frontend stub / no IPC handlers / no read projections).

## Done this session (Layer A — frontend render + versioning)
- `apps/reader/src/styles/global.css` — §15.6 `--ez-*` tokens + animation primitives (aurora/glow/shimmer/pulse; reduced-motion = a11y fallback only).
- `apps/reader/src/stores/appStore.ts` — §15.4 bootstrap slice (calls `app_bootstrap`).
- `apps/reader/src/components/shell/{providers,AppShell,NavigationRail,JobStrip,ReadinessIndicator}.tsx`.
- `apps/reader/src/App.tsx` — stubs removed → provider tree. `router.tsx`/`main.tsx` — AppShell layout + real HomeView.
- Versioning fixed: `vite.config.ts` reads canonical `version.txt`; `apps/reader/package.json` runs `scripts/update-version.sh` on every `dev`/`build`. Stamped version now `1.1.55721` (was stale `1.0.52684` baseline).
- Pre-existing type debt cleared to get a green build: React-19 `JSX` import added to 14 annotation files; ambient `Buffer` shim (`src/types/node-buffer-fallback.d.ts`); one widened return type.
- Verified: `pnpm build` green; rendered at `localhost:4173` (version `v1.1.55721`, aurora bg, nav rail, real HomeView composed).
- Backups: every edited file has a `.bak.20260729_<HHMMSS>` beside it.
- `node_modules` (74 MB) restored from `pnpm-lock.yaml` — keep until M1 verified, then clear per operator.

## Version discipline (operator directive 2026-07-29, post-compaction)
The parallel coders each ran `update-version.sh` inside their own verify-builds,
free-bumping MINOR (`1.1`→`1.3`→`1.5`, versionCode `100001`→`100003`→`100005`).
That is the "overlapping versions make chaos" the operator flagged. **Protocol
for the integration→AppImage→screencast cycle (orchestrator-owned):**
1. Ensure no `release.lock` (resume bumping).
2. Run `update-version.sh` ONCE → captures a fresh version (bumps MINOR).
3. Immediately write `release.lock` (sourced: `MAJOR=… MINOR=… BUILD_NUM=…`)
   freezing that value — every later `update-version.sh` (vite, cargo, tauri)
   then stamps the identical string. Zero divergence across sub-builds.
4. AppImage label, `dist/` artifact name, commit `v{VERSION}: `, and screencast
   annotation ALL carry that one frozen version.
5. After the screencast is archived+annotated, delete `release.lock` to resume.
`cargo check` does NOT invoke the stamper (only the npm `build`/`dev` scripts
do), so Rust verify-builds do not disturb the version.

## Coder status (3 parallel, non-overlapping files)
1. **Rust IPC seam** — `apps/reader/src-tauri/{Cargo.toml,src/lib.rs,src/ipc.rs,src/state.rs}` + `crates/storage-sqlite/src/{lib.rs,query.rs}` + `crates/application/src/{lib.rs,home.rs,bootstrap.rs,readiness.rs}`. Implements `app_bootstrap`, `home_snapshot`, `readiness_get`, `reader_open`, `reader_article` (§14.4) wired to `AnZimmermanZimEngine` + storage. **Status: at `cargo check -p reader` verify gate** (lib.rs confirmed wired: SQLite open + migration + AppState + 5-command handler). ⏳
2. **Reader UI** — ✅ LANDED. `routes/reader/{ReaderView,ArticleContent,BreadcrumbNav,ArticleToolbar}.tsx` + `stores/readerStore.ts` + `services/readerService.ts` + `/reader/:packId/:articleKey` + `OpenPackCard` on Home. §7.3 sandboxed `<iframe>` (no `allow-scripts`, no `allow-same-origin`, CSP `default-src 'none'`). `pnpm build` green (64 modules).
3. **Home wiring** — ✅ LANDED. `homeService`/`homeStore` collapsed to one `home_snapshot`; slice/component contract unchanged. `pnpm build` green (57 modules).

### Seam to reconcile on Rust landing — `reader_open` arg shape
- **Frozen DTO** (`ARCHITECTURE.md` L3816): `ReaderOpenRequest{packId, articleKey?}` — presumes an **imported** pack (packId already known).
- **Reader UI** shipped: `reader_open({path})` → `{packId,title,mainPageKey}` — open-by-**path** (file picker; M1 has no import flow yet).
- **`reader_article`** matches `{packId, articleKey}` ✓ — no seam there.
- **Architect decision (apply when Rust reports):** M1's `reader_open` is open-by-path (the file-picker entry); the frozen `{packId}` DTO is the *post-import* shape for a later milestone. If the Rust coder shipped `{path}` → frontend/backend already agree, record a §26 amendment noting the open-by-path M1 entry. If it shipped `{packId}` → reconcile frontend `readerService.readerOpen` + `readerStore.open` to the path the Rust handler expects (or extend the handler). The architect amends; no coder spelunks.
- **Deferred (coder-flagged):** `@tauri-apps/plugin-dialog` not in deps — `OpenPackCard` dynamic-imports it (gated on `__TAURI_INTERNALS__`); installing+registering it is the post-M1 hardening (or pre-screencast if the picker is needed for R-OPEN). Article images need `enzime-resource://` rewriting (Rust side) — safe under CSP, just not pretty until then.

## After coders land (sequential — depends on code above)
4. Integrate-verify: `cargo check` (workspace) + `pnpm build` + run `tauri dev`; confirm Home loads real (empty) data and reader opens a ZIM.
5. **AppImage build** (Linux): ensure system deps (webkit2gtk-4.1, etc.); `tauri build`; AppImage must live in TRACKED `dist/` (un-gitignore root `dist/` with `!/dist/` exception; nested `apps/*/dist` still ignored), git-LFS on `forgejo.robin.mba` (CC13), CC12 slug-first filename, never in `.tmp/`. Commit with `v{VERSION}: ` prefix.
6. **Gauntlet** (`DOCS/TEST_RUBRIC.md`): after all ✅, run on the AppImage — record screencast (TC11) showing boot→open ZIM→render article→navigate→offline (P2-GATE "Airplane reader"). Two-outcome: SHIP-READY or DEFECTIVE.

## Key references
- Architecture (frozen, attested §25.1): `DOCS/ARCHITECTURE.md` — §7 ZIM, §14.4 command registry, §15 frontend tree, §16.6 file paths, §24 bindings.
- Checklist: `CHECKLIST.md` Phase 2 (reader vertical slice, P2-03…P2-07 `[/]`; P2-GATE airplane reader).
- Rubric: `DOCS/TEST_RUBRIC.md` (authored this session).
- Stitch (frozen, MCP autonomous, no operator gate): project `9898008898121608042`, all 10 screens; "EnZIME Classic Reader" = reader design. Captured Home HTML at `DOCS/REBOOT-PAK/STITCH/captured/`.
- Versioning: `scripts/update-version.sh` (canonical, runs every build).
- Operator worldview: `~/Admin-Manual/DOCS/IDEOLOGIES/equilibrium-not-flattened-fairness.md`.
- Force-directed 3D 6DoF graph ("ThinkSpace Navigator", `EntityGraph`) = a SCHEDULED future milestone, NOT M1 — preserve a clean mount, don't preclude it.

## DYNDON PIVOT (2026-07-29, post-compaction) — the USP is ON the M1 path
Operator directive: M1 is **not** a file-picker reader. The reader entry is
**Library → DynDon storage-fit download → read offline**, and the DynDon
algorithm (the product's sole market differentiator — "download all Wikipedia
even when you don't have enough disc space") is in-scope for M1. v4's frozen
architecture **already scaffolds** it (`selection_profiles`+`_revisions`,
`download_jobs`/`download_chunks`, `install_transactions`,
`pack_kind='tailored_zim'`, `storage_roots.capacity_bytes`,
`TransferSource::open_range`, `InstallPlan.free_bytes_before/after`, and every
Library component + §24 binding). What was missing = the **planner brain**, the
**partial-cluster read path**, the **IPC/library UI wiring**, and an unblocked
`application` crate. None of the sibling EnZIMEs contain a DynDon *implementation*
(only PRD §3 prose) — so this is implement-fresh, not restore.

**Non-corner architecture decision:** NO ZIM writer. DynDon plans a cluster
subset from a `ClusterManifest`; the selected clusters are downloaded; a
`ClusterCacheArchive` (implements `ports::zim::ZimArchive`) reads articles from
the local cluster cache (directory always present; payloads cached / on-demand
via byte-range in production). This avoids the ZIM-writer corner that sank
prior attempts.

### DynDon wave — 5 parallel GLM coders (background, non-overlapping)
- **A** (task #9) `crates/application/src/{lib,transfer}.rs` — unblock the 22 errors. Verify `cargo check -p application`.
- **B** (task #10) NEW `crates/dyndon` — pure planner `plan_fit(manifest, intent) → ClusterFitPlan`. Verify `cargo check -p dyndon` (+test).
- **C** (task #11) `crates/storage-sqlite/src/write.rs` — INSERT/UPDATE repos. Verify `cargo check -p storage-sqlite`.
- **D** (task #12) NEW `crates/zim-cluster-cache` — `ClusterCacheArchive: ZimArchive` + one `pub fn decompress_cluster_blobs` exposed in zim-anzimmerman. Verify `cargo check -p zim-cluster-cache`.
- **F** (task #14) `apps/reader/src/routes/library/*` + `libraryStore`/`libraryService` + `/library` route — the USP display (DownloadPlanDialog/StorageImpactBar). Verify `tsc --noEmit && vite build`.
### Sequenced (after B/C/D land)
- **E** (task #13) `apps/reader/src-tauri/src/ipc.rs`+`lib.rs`+`state.rs` — wire `catalog_search`/`storage_roots_list`/`install_plan`/`install_start`/`job_list` to dyndon+storage+cluster-cache; repoint `reader_open` to installed packs (retire the file-picker `OpenPackCard` path). Verify `cargo check -p reader` + `pnpm build`.
### Then
- Single-version freeze (`release.lock`) → `tauri build` AppImage (tracked `dist/`, CC12/CC13) → SC4 gauntlet (re-formed `DOCS/TEST_RUBRIC.md`) → screencast (TC11). Do not stop until the screencast is archived + annotated.
