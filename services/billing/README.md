# EnZIME billing service

Zero-dependency TypeScript **Netlify Function** that sells EnZIME plans through
**Chargebee** and issues **signed offline entitlements** the desktop app
verifies with an embedded public key. Free-tier hosting; no server to rent.

```text
EnZIME Reader ──HTTPS──▶ /v1/* (Netlify Function) ──▶ BillingProvider ──▶ Chargebee (today)
      ▲                         │                                        WooCommerce (later)
      └── enzime.entitlement.cbor.v1, Ed25519-signed ◀──┘
```

| Route | Body | Returns |
|---|---|---|
| `GET /v1/plans` | — | public plan list (no provider ids) |
| `POST /v1/checkout` | `{deviceId, planId, licenseId?}` | `{licenseId, url}` hosted checkout |
| `POST /v1/portal` | `{licenseId}` | `{url}` self-service portal |
| `POST /v1/entitlement` | `{deviceId, licenseId}` | `{artifact (base64 CBOR), grants, expiresAtMs}` or `404 no_purchases` |
| `GET /v1/health` | — | `{ok, provider}` |

The **license id** (`enz_` + 26 random chars) is the Chargebee customer id and
the user's restore key. It is minted on first checkout and shown in
*Settings → Plan*.

## One-time setup (≈20 minutes)

### 1. Chargebee (start with your **test** site)

Product Catalog 2.0 → create items whose **item price ids exactly match**
`catalog.json` (or edit `catalog.json` to match yours):

| catalog plan | Chargebee item type | item price id |
|---|---|---|
| `pro_monthly` | Plan, monthly | `enzime-pro-USD-Monthly` |
| `pro_yearly` | Plan, yearly | `enzime-pro-USD-Yearly` |
| `lifetime` | **Charge** (one-time) | `enzime-lifetime-USD` |

Then *Settings → Configure Chargebee → API keys*: create a **full-access**
key. It lives only in Netlify, never in the app.

### 2. Signing key

```bash
cd services/billing
node scripts/keygen.ts enzime-entitlement-2026
```

Keep an offline backup of `ENTITLEMENT_SIGNING_KEY`. The `ENZIME_*` lines are
public and go into the app build.

### 3. Netlify

New site from this repo → **Base directory** `services/billing` (the
`netlify.toml` there does the rest; the build step runs the test suite).
Environment variables:

| Variable | Value |
|---|---|
| `CHARGEBEE_SITE` | e.g. `enzime-test` (the `<site>.chargebee.com` subdomain) |
| `CHARGEBEE_API_KEY` | full-access API key |
| `BILLING_RETURN_URL` | `https://<your-site>.netlify.app/` (thank-you page in `public/`) |
| `ENTITLEMENT_KEY_ID` | from keygen |
| `ENTITLEMENT_SIGNING_KEY` | from keygen (secret) |

Verify: `curl https://<your-site>.netlify.app/v1/health`.

### 4. App build

Set at build time (compiled in via `option_env!`; the same names at runtime
override them, which is handy for testing):

```bash
ENZIME_BILLING_URL=https://<your-site>.netlify.app
ENZIME_ENTITLEMENT_KEY_ID=enzime-entitlement-2026
ENZIME_ENTITLEMENT_PUBKEY=<64 hex chars>
ENZIME_GA_MEASUREMENT_ID=G-XXXXXXXXXX     # optional, GA4
ENZIME_GA_API_SECRET=<measurement protocol secret>  # optional
```

A build without these simply hides purchasing and analytics; features the
user already owns stay unlocked offline.

## Local development

```bash
node scripts/dev-server.ts          # mock provider; prints throwaway ENZIME_ENTITLEMENT_* values
# in another shell, with those values exported:
ENZIME_BILLING_URL=http://127.0.0.1:8787 pnpm --dir apps/reader tauri dev
```

The mock "checkout" page marks the purchase paid when opened. Set
`CHARGEBEE_SITE`/`CHARGEBEE_API_KEY` to point the dev server at your Chargebee
test site instead.

## Tests

```bash
node --test test/*.test.ts            # unit + API + Chargebee adapter + golden
node scripts/golden.ts                # refresh the Rust cross-language fixture
cargo test -p entitlement --test ts_golden
# full purchase → offline-restart E2E, Rust app ↔ this server:
node scripts/dev-server.ts &   # note the printed key id / pubkey
ENZIME_E2E_BILLING_URL=http://127.0.0.1:8787 ENZIME_ENTITLEMENT_KEY_ID=dev-ephemeral \
  ENZIME_ENTITLEMENT_PUBKEY=<printed> cargo test -p reader --lib billing::tests::e2e
```

## Moving to WooCommerce later

Implement `BillingProvider` (`src/provider.ts`) in `src/woocommerce.ts`
against the ecosystem plugin, switch the provider in
`netlify/functions/billing.mts`, and point `provider_product_id`s in
`catalog.json` at WooCommerce product ids. The app, the artifact format and
already-issued entitlements are unaffected. Keep the same signing key. To
migrate existing customers, have the WooCommerce adapter also look up
Chargebee by license id until those subscriptions end.

The function can also run on shared hosting with Node 22+
(`scripts/dev-server.ts` shows the plain `node:http` wrapper).
