# EnZIME Reader — Final Test Rubric / Gauntlet (SC4) — DynDon M1

- **Authority**: SC4 (`~/Admin-Manual/DOCS/SPEC_CONVENTIONS/final-test-rubric-gauntlet.md`); TC11 screencast archival; SC2 idempotent accept clauses (structural); SC3 semantic names.
- **Authored/re-formed**: ARCHITECT phase, 2026-07-29 — re-formed after the M1 reader-entry re-architecture (file-picker → **Library → DynDon storage-fit download → read**). Earlier file-picker rows are void.
- **Execution timing (GR-2)**: run only after the M1 DynDon wave (application unblock + planner + storage write-repos + cluster-cache wrapper + Library UI + integration coder E) is green end-to-end. Partial runs are debugging, not verdicts.
- **Verdict (GR-3/GR-3a)**: exactly two terminal outcomes — **SHIP-READY** or **DEFECTIVE**. Any ❌ ⇒ DEFECTIVE; no waiver. Agent rows all ✅ + R-OPERATOR un-run ⇒ **INCOMPLETE—pending-operator**.
- **Evidence (GR-4 / TC11)**: every PASS is granted by *looking at* a **screencast of the built Linux AppImage** (not the vite dev server). Logs/greps/exit codes are FAIL-triggers/prerequisites only. The web/vite app is source-level testing only.
- **Marker discipline (GR-4a)**: a `[X]`/✅ is a claim; only direct semantic observation justifies flipping it.

## Success gate (operator directive)

A recorded screencast of the **Linux AppImage** in which the app boots, opens the **Library**, selects a pack, **DynDon computes a storage-fit plan** (the USP: a subset that fits when the full pack wouldn't), **downloads** that subset, and **renders a real article offline** from the local clusters — i.e. the "Airplane reader" + the DynDon USP certified by observation.

## Rows

| ID | Requirement (what the screencast must show) | Evidence | Exec |
|---|---|---|---|
| **R-BOOT** | AppImage boots into Aurora Glass — no blank screen, no stub `<h1>`; version label = stamped `MAJOR.MINOR.BUILD` (frozen via `release.lock` for the whole cycle). | screenshot + `#app-version` text | agent |
| **R-LIBRARY** | Library route opens; catalog entries + storage roots render (live Kiwix library results, persisted cache clearly identified when offline — never placeholder text). | screencast segment | agent |
| **R-PLAN (USP)** | Selecting a pack **larger than the chosen storage budget**, DynDon's planner returns a **subset that fits**: `downloadBytes < artifactSizeBytes` AND `freeBytesAfter ≥ 0`. DownloadPlanDialog/StorageImpactBar visibly show the tailored fit. **This is the market-differentiating USP — it must be observable.** | screencast segment: plan numbers + "fits/truncated" UI | agent |
| **R-DOWNLOAD** | `install_start` downloads the selected clusters (resumable); the download job progresses to `completed` with retained-byte progress reaching the selected payload total. A partial archive is identified as source-pinned/partial; retained bytes are never presented as verification of the full upstream archive. | screencast segment: progress → completed | agent |
| **R-READ** | Reader opens an article **from the downloaded subset**; the ZIM's actual HTML renders (sandboxed §7.3 — no scripts/same-origin). Not placeholder, not blank. | screencast segment: article body readable | agent |
| **R-OFFLINE** | After quitting and relaunching the artifact with network unavailable, R-READ and its retained resources work — clusters and required metadata are local; reader is NOT gated on AI or network (P2-GATE "Airplane reader"). | screencast with network off | agent |
| **R-ANNOTATE** | On the real article canvas, draw ink, type a text annotation, record voice separately, capture a photo and record a short video clip. Save each at a visible article location. Voice recording is not labeled transcription. | native capture/save/replay screencast segments, actual artifact version | agent |
| **R-ANNOTATION-RESTART** | Quit and reopen with network unavailable. Reopen the same article; each saved annotation appears at its original anchor. Ink and typed text render, photo displays, and voice/video play from local annotation assets. | restart + offline on-canvas playback screencast | agent |
| **R-ANNOTATION-FAILURE** | A failed capture/save remains retryable without losing the captured data or duplicating the annotation. Navigation with an unsaved draft offers Stay/Discard; delayed article responses never replace a newer article or misbind its annotations. | deterministic component/store tests plus native draft-navigation/capture-error observation | agent |
| **R-BUILD** | `tauri build` (Linux) AppImage lives in tracked `dist/` (tracked artifact with SHA-256; Forgejo publication suspended by explicit operator outage notice; never `.tmp/`); filename CC12 slug-first `<slug>-v<version>-<qualifier>`; one frozen version for the whole cycle. | the AppImage + `git lfs ls-files` + it runs | agent |
| **R-SCREENCAST** | The AppImage screencast is archived per TC11 under `~/Admin-Manual/LOGS/EnZIME-v4/` with a semantic filename. | the archived screencast file | agent |
| **R-OPERATOR** | Operator views the screencast and attests the reading + the storage-fit USP are real and complete. | operator sign-off note | **operator** |

## Two-outcome verdict

- **SHIP-READY**: R-BOOT … R-SCREENCAST all ✅ by semantic observation AND R-OPERATOR ✅.
- **DEFECTIVE**: any of R-BOOT…R-SCREENCAST ❌.
- **INCOMPLETE—pending-operator**: R-BOOT…R-SCREENCAST all ✅, R-OPERATOR un-run.

## Bounded stylesheet repair protocol — 2026-09-13

This is a debugging protocol for CHECKLIST P8-03-STYLE, not an M1 verdict or
replacement for any R-* row. The user rejected the visibly unstyled v1.6.55832
AppImage. On the newly stamped native AppImage, record its version and SHA-256,
then capture the actual window through these steps:

1. Launch with an isolated application-data profile. Open Library and let its
   data loading settle. Cards, two-column layout at desktop width, styled
   inputs and separate empty-state title/detail lines remain visible.
2. Type a catalog query and language, causing route re-renders. Styling persists
   after search completion; an empty result is reported as empty, not invented.
3. Navigate Library → Home → Library and repeat. Shared status icons, text and
   buttons retain their styles without depending on a prior Home visit.
4. Resize the native window to its 720 CSS pixel minimum, narrower than Library's 60rem breakpoint; the
   single-column flow is readable and inputs remain contained. Restore the
   desktop size and leave Library open for the operator.

Archive the window recording/stills and results in the project evidence home.
Record catalog/download/offline-reading as unverified by this bounded run;
restoring styles does not connect the remote catalog or close F01/F02.

The v1.7.21541 run passed desktop loading/search/navigation but failed step 4:
the shell's conflicting 48rem rules displaced the route. P8-03-NARROW closes
that observed CSS defect before repeating this same protocol on the next artifact.

## What this rubric is NOT

- A green `cargo check`/`tsc`/`vite build` is **not** a PASS (GR-4) — prerequisite only.
- A full-pack download that happens to fit is **not** R-PLAN — the USP requires demonstrating a **subset fit when the full pack exceeds the budget**.
- A file-picker "open local .zim" path is **void** for M1 (superseded by Library→DynDon).

## Initial-release annotation scope — 2026-10-12

Voice recording and typing are separate tools. Automatic transcription is deferred by explicit user decision and is not claimed or required for this release. A later transcription feature must retain the original audio asset, annotation ID and article anchor. The native run must verify all five annotation modes, not substitute a sidebar list or mocked asset URLs for on-canvas playback. Camera/microphone permission denial must show an actionable error and leave the reader usable.

Record the selected Kiwix entry and exact pinned source size/identity, user storage budget, actual retained payload and metadata size, and offline restart result. Keep whole-archive digest verification distinct from partial source identity and local asset integrity. Tests alone do not establish real Kiwix acquisition or native media-device behavior.
