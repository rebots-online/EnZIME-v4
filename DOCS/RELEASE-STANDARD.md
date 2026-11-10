# EnZIME Reader — Release Standard

**Adopted:** 2026-11-10 (operator-approved plan, same date).
**Scope:** the bar a build must meet before public commercial release.
**Relation to other documents:** `CHECKLIST.md`, `DOCS/TEST_RUBRIC.md` and
`DOCS/ARCHITECTURE.md` remain inputs and evidence. Where one of them is stricter
than a line here, the stricter one applies. Build mechanics are governed by
`~/.claude/BUILD_CONVENTIONS.md` and Admin-Manual CC11/TC14; this document does
not restate them, it depends on them.

A build is releasable only when every line below is true and has recorded evidence
from the exact artifacts being published. Code inspection and "it compiled" do not
satisfy any line.

## Operator directives that shape this standard (2026-11-10)

1. Tauri 2 multiplatform. Matrix per CC11: Windows `.exe`, NSIS installer, `.msi`,
   `.msix`; Linux AppImage, `.deb`, `.rpm`; web; Android release `.apk`, `.aab`,
   native-symbols ZIP.
2. Every platform is built together by `build_all.sh`. No manual, isolated
   or per-platform build. Windows `.exe` and NSIS cross-compile from Linux; MSI and
   MSIX are built together by `build_all_windows.ps1` when booted into Windows, for
   the same version.
3. Version bumps automatically on every build invocation. No two builds share a
   version. No build artifact exists without its version in the filename, and the
   running app shows version/build in the bottom-right corner of the window.
4. Production builds only, signed with the organization-wide production keystore
   resolved from Admin-Manual. No debug builds.
5. No unit-test stage and no smoke-test stage in the build.
6. Security hardening is not a release gate. Checks that detect corrupted or
   truncated data remain, as data correctness (R2).
7. No CI until public release (TC14). A workflow may be checked in dormant.

## R1 — Truth

Every capability named in the store listing, the plan catalogue or the UI exists,
works offline, and is enforced where the work happens (Rust command), not only
hidden in the UI. No placeholder value ships. No control exists that cannot
succeed. No frontend command name is unregistered. The displayed price and the
charged price come from one source.

## R2 — User data is never lost

- Every write path is crash-safe: terminating the process at any point leaves the
  last good state readable.
- Upgrading from every previously distributed version (1.0, 1.6, 1.14) preserves
  all notes and installed packs. Note: Android builds from this standard onward
  are signed with the organization keystore, while 1.0 and 1.6 were signed with
  the project keystore in `signing/`; Android treats that as a different app
  signature, so an in-place update over 1.0/1.6 is not possible and the upgrade
  path there is export → install → import.
- Export → wipe → import, and encrypted backup → restore, are lossless for every
  annotation type on every platform.
- A downloaded pack whose bytes do not match its recorded digest is never
  activated; a stored cluster that no longer matches is refetched or reported.
- Failures keep the original data and offer retry.

## R3 — Performance on a Wikipedia-class pack (≥ 50 GB source, ≥ 20 GB retained)

| Metric | Desktop (4-core, 8 GB, SATA SSD) | Android (mid-range 2024, 8 GB) |
|---|---|---|
| Cold start to usable Home, independent of installed pack size | ≤ 2.0 s | ≤ 3.0 s |
| Article open, p95 (offline, 40 images) | ≤ 400 ms | ≤ 1.0 s |
| Title search, p95 per keystroke | ≤ 100 ms | ≤ 300 ms |
| Install throughput | ≥ 80 % of the slower of network and disk | same |
| Install plan shown, or progress visible | ≤ 3 s | ≤ 5 s |
| Idle memory with 5 packs installed | ≤ 350 MB | ≤ 250 MB |
| UI thread blocked > 100 ms / Android ANR | none | none |

Where `DOCS/ARCHITECTURE.md` §21 is stricter, §21 wins. Measured on the built
artifacts on reference hardware, by the operator's recorded gauntlet.

## R4 — Evidence

Acceptance is observed behaviour of the built, versioned artifacts on each
platform, recorded against `DOCS/TEST_RUBRIC.md` (the operator gauntlet and the
seven §19.4 end-to-end stories). Static checks that already exist in the build
(`tsc --noEmit`, version/identity/ABI/signature validation of each artifact, the
build-manifest verifier) stay. Build preparation must also stop when a frontend
command name has no registered handler. `tools/contract-gen validate` cannot be
that gate yet: it checks the whole §24 binding table against generated TypeScript
entities that are not in the repository, so it would fail every build today. Until
those inputs exist, the gate is `scripts/check-ipc-commands.py`, run in build
preparation: it compares the command names the frontend uses with the
`generate_handler!` list and stops the build on any unregistered name.

## R5 — Accessibility

WCAG 2.2 AA on every shipped surface: full keyboard operation, visible focus,
screen-reader pass (NVDA, Orca, TalkBack), contrast, 200 % text scaling,
reduced motion respected.

## R6 — Platform quality

- `build_all.sh` on a clean clone produces the whole host matrix or exits
  nonzero. `scripts/verify-build-set.py --require-full-matrix` must pass for a
  release, which requires the Windows-boot MSI/MSIX as well.
- The build manifest records `monetization_env`; a release candidate has all three
  billing values `set`.
- Android: current Play target API, 16 KB page-size compatible, all four ABIs.
- SBOM, third-party notices and licence inventory generated from the release commit.
- Desktop auto-update with rollback; store updates on Android.
- Opt-in crash reporting; crash-free sessions ≥ 99.5 % through public beta.

## R7 — Commerce and legal

Live-mode proof of purchase, restore on a second device, 30-day offline use,
cancellation, refund and revocation, with user data intact in every case. Taxes
handled by the provider. Published Terms, Privacy Policy and refund policy that
match actual data flows. Content and model redistribution rights inventoried.

## R8 — Operations

Support channel, in-app diagnostics export with consent, staged rollout with a
tested rollback, release notes, and an incident procedure for the billing service.

## R9 — Product finish

First-run setup takes a new user to a readable offline article without
documentation. User-facing strings are externalised; right-to-left and CJK article
content renders correctly. A feature is complete or it is absent.

## Scope defaults (operator may override)

| # | Decision | Default |
|---|---|---|
| D1 | Platforms | Linux, Windows, Android. macOS needs a lane, hardware and an Apple account; iOS needs App Store purchase, which conflicts with the Chargebee flow. |
| D2 | Ask and Memory in first release | Not included; they ship in the following version to this same standard. The catalogue sells only what the build delivers. |
| D3 | Prices | Operator sets. Current catalogue values are placeholders. |
| D4 | Coupons | Not in first release. |
| D5 | Windows channel | NSIS installer and standalone `.exe` from Linux; MSI/MSIX from Windows boot. |

## Milestones

| | Outcome | Standard |
|---|---|---|
| M0 | Clean clone builds the whole host matrix with one command; this document; contract gate | R4, R6 |
| M1 | Reader core holds up on real packs: boot, article read, search, install, integrity | R2, R3 |
| M2 | User work survives: sidecar, voice, backup/restore, migration | R2 |
| M3 | Commerce and finish: capability enforcement, catalogue truth, prices, setup, dead surfaces | R1, R7, R9 |
| M4 | Hardening: accessibility, auto-update, crash reporting, device matrix, legal, SBOM | R5, R6, R8 |
| M5 | Closed beta → public beta → staged release from verified hashes | R6, R8 |
| M6 | Ask and Memory | all |
