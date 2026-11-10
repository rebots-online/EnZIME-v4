# Handoff — 2026-11-05 — Billing, analytics, v1.14 build set

Session: cloud container (claude.ai/code), **not** the operator workstation.
All work is on `master` at `bab73f5`. Working tree clean. No open branches
or PRs (PR #1 was fast-forwarded into master and closed).

## What landed (commits, oldest first)

| Commit | Content |
|---|---|
| `ea2056b` | Chargebee billing + offline signed entitlements + opt-in GA4 analytics |
| `f259d40` | gitignore `.anchor/` (per-session hook state) |
| `3162966` | Stale Forgejo workstation path removed from `DOCS/BUILD-RECIPES.md` |
| `7e6f0e1` | Pin `tauri` 2.11 / `tauri-plugin-opener ~2.5` (see Bug below) |
| `bab73f5` | v1.14.97545 build set — 7/12 artifacts in `dist/` |

### Billing (ARCHITECTURE §13, amended; design note `DOCS/ARCHITECTURE/billing-and-analytics.md`)

```text
Reader ──HTTPS──▶ services/billing (Netlify Function, free tier) ──▶ BillingProvider ──▶ Chargebee
  ▲  verifies offline with embedded Ed25519 pubkey   │                               (WooCommerce later)
  └──── enzime.entitlement.cbor.v1 (device-bound) ◀──┘
```

- `services/billing/` — zero-dependency TypeScript. Routes: `GET /v1/plans`,
  `POST /v1/checkout|portal|entitlement`, `GET /v1/health`. Catalog in
  `catalog.json` (**prices are placeholders**). README has the full runbook.
- `apps/reader/src-tauri/src/billing.rs` — verify → SQLite
  (`entitlement_artifacts/grants/trust_keys`) → `OfflineCapabilityPolicy`.
  Commands `billing_*`, `entitlement_*`. Checkout/portal open in system browser.
- `/settings` route: Plan (capabilities, plans, license key, restore, manage
  billing) + Privacy. `useCapability(key)` in `stores/entitlementStore.ts`
  for gating — **not yet applied to any feature screen** (free/paid split is an
  operator decision).
- RevenueCat placeholder crate removed. ZIM backend (AnZIMmermanLib) untouched.

### Analytics (PRD §8.3)
GA4 Measurement Protocol from Rust (`analytics.rs`), off until opt-in (first-run
banner + Settings toggle), allow-listed events/keys, token-only values, route
templates, random install id. Content cannot leave the device.

### Tests (all passing at `7e6f0e1`)
billing service 10/10 · `cargo test -p entitlement --test ts_golden` 2/2
(TS↔Rust byte-identical) · reader billing/analytics/settings 8/8 incl. E2E
purchase→offline-restart · workspace cargo test green · frontend 85/85.

## Bug introduced and fixed in-session
Adding `tauri-plugin-opener = "2"` let cargo resolve `tauri` 2.12.5, which the
Tauri CLI rejects against `@tauri-apps/api` 2.11 (linux + windows-nsis lanes
failed). Fixed in `7e6f0e1` by rebuilding `Cargo.lock` from the pre-billing base
and pinning the opener to `~2.5`. **Do not bump `tauri` and `@tauri-apps/api`
independently.**

## Build state — v1.14.97545 (`dist/`, sha256 in `*-sha256sums.txt`)

| Lane | Result |
|---|---|
| linux (deb, AppImage, rpm) | ✅ |
| windows-nsis (`windows-x64-setup.exe`) | ✅ |
| sbom, source, web | ✅ |
| android (apk, aab, symbols zip) | ❌ no Android SDK/NDK in this container |
| windows-native (msi, msix) | ❌ by design — needs native Windows executor |

Also: the current matrix has **no lane for a standalone `windows-x64.exe`**
(v1.6 shipped one). Adding it means a new lane in `scripts/build-lane.sh` +
`verify-build-set.py`.

## Next steps (operator workstation)

1. `bash build_all.sh` — fills android apk/aab/symbols. Keystore is the
   single org keystore, pointer: `~/Admin-Manual/CREDENTIALS/EnZIME.md` →
   `CREDENTIALS/PlayStore/production.keystore`, alias `upload` (verified the
   only key in the store). `build_set.py` reads `signing/build-credentials.json`
   or `ENZIME_SIGNING_CONFIG`.
2. Go-live for billing (≈20 min, `services/billing/README.md`): Chargebee items
   matching `catalog.json` ids (test site first) → `node scripts/keygen.ts` →
   Netlify site, base dir `services/billing`, 5 env vars → app build with
   `ENZIME_BILLING_URL`, `ENZIME_ENTITLEMENT_KEY_ID`, `ENZIME_ENTITLEMENT_PUBKEY`,
   optional `ENZIME_GA_MEASUREMENT_ID`/`ENZIME_GA_API_SECRET`.
3. Decide free vs paid features; apply `useCapability()` gates.
4. Optional: standalone exe lane.

## Process notes for the next agent
- `CLAUDE.md` points at `~/Admin-Manual`; in a cloud container it is absent —
  `git clone https://github.com/rebots-online/admin-manual ~/Admin-Manual` first.
  KStore is authoritative when reachable; Admin-Manual is the fallback.
- Commit directly to `master`. No branches, no PRs.
- Builds only via `build_all.sh`. Never isolated lane builds.
- Credentials are referenced by pointer, never asked for, never copied.
- Cloud containers are ephemeral: toolchain (SDK/NDK, cargo-xwin, nsis, rpm,
  cargo-cyclonedx) is not present. A claude.ai/code environment setup script
  running `~/Admin-Manual/scripts/kickstart-tauri-build.sh all` would fix this.
- Stale `forgejo.robin.mba` references remain in `~/Admin-Manual/MANUAL.md` §6/§10
  and the GitHub repo description; fix on encounter per operator.
