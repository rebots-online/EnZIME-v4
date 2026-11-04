import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createPublicKey, generateKeyPairSync, verify } from "node:crypto";
import { test } from "node:test";

import { encode, map } from "../src/cbor.ts";
import { parseCatalog } from "../src/catalog.ts";
import { ChargebeeProvider } from "../src/chargebee.ts";
import { handle, type ApiDeps } from "../src/api.ts";
import { grantsFor } from "../src/issue.ts";
import { newLicenseId, validLicenseId, type BillingProvider, type Purchase } from "../src/provider.ts";
import { goldenVector } from "./golden-vector.ts";
import catalogJson from "../catalog.json" with { type: "json" };

const catalog = parseCatalog(JSON.stringify(catalogJson));

test("cbor matches RFC 8949 appendix A vectors", () => {
  const hex = (v: Parameters<typeof encode>[0]) => Buffer.from(encode(v)).toString("hex");
  assert.equal(hex(0), "00");
  assert.equal(hex(23), "17");
  assert.equal(hex(24), "1818");
  assert.equal(hex(1000), "1903e8");
  assert.equal(hex(1000000), "1a000f4240");
  assert.equal(hex(1_800_000_000_000), "1b000001a3185c5000");
  assert.equal(hex(-1), "20");
  assert.equal(hex(-1000), "3903e7");
  assert.equal(hex(null), "f6");
  assert.equal(hex("IETF"), "6449455446");
  assert.equal(hex(new Uint8Array([1, 2, 3, 4])), "4401020304");
  assert.equal(hex([1, [2, 3]]), "8201820203");
  assert.equal(hex(map(["a", 1], ["b", [2, 3]])), "a26161016162820203");
});

test("shipped catalog is valid; core and unknown keys are rejected", () => {
  assert.equal(catalog.plans.length, 3);
  const bad = structuredClone(catalogJson) as { plans: Array<{ capabilities: string[] }> };
  bad.plans[0].capabilities = ["reader.core"];
  assert.throws(() => parseCatalog(JSON.stringify(bad)));
});

test("license ids are unguessable-format and validated", () => {
  const id = newLicenseId();
  assert.ok(validLicenseId(id), id);
  assert.notEqual(id, newLicenseId());
  assert.ok(!validLicenseId("enz_short"));
  assert.ok(!validLicenseId(id.slice(0, -1) + "&"));
});

test("grants: lifetime beats subscription, later term beats earlier, unknown products ignored", () => {
  const p = (id: string, kind: Purchase["kind"]): Purchase => ({ providerProductId: id, kind, reference: id });
  const g = grantsFor(
    [
      p("enzime-pro-USD-Monthly", { type: "subscription", paidThroughMs: 100 }),
      p("enzime-pro-USD-Yearly", { type: "subscription", paidThroughMs: 500 }),
      p("enzime-lifetime-USD", { type: "one_time" }),
      p("someone-elses-product", { type: "one_time" }),
    ],
    catalog,
  );
  const byKey = Object.fromEntries(g.map((x) => [x.capability_key, x]));
  assert.equal(byKey["annotations.rich_media"].grant_kind, "perpetual");
  assert.equal(byKey["models.premium_catalog"].grant_kind, "term");
  assert.equal(byKey["models.premium_catalog"].valid_until_ms, 500);
  assert.equal(g.length, 7);
});

test("golden artifact signature verifies and fixture is current", () => {
  const v = goldenVector();
  const bytes = Buffer.from(v.artifactHex, "hex");
  // Signature value is the last 64 bytes of the encoding (final bstr field).
  const sig = bytes.subarray(bytes.length - 64);
  const signatureMapStart = bytes.indexOf(Buffer.from("697369676e6174757265", "hex")); // "signature"
  // Unsigned fields = everything between the map(11) header and the "signature" key.
  const unsignedFields = bytes.subarray(1, signatureMapStart);
  const canonical = Buffer.concat([Buffer.from([0xaa]), unsignedFields]); // map(10)
  const message = Buffer.concat([Buffer.from("enzime.entitlement.v1\0\0"), canonical]);
  const pub = createPublicKey({
    key: Buffer.concat([Buffer.from("302a300506032b6570032100", "hex"), Buffer.from(v.publicKeyHex, "hex")]),
    format: "der",
    type: "spki",
  });
  assert.ok(verify(null, message, pub, sig));
  assert.equal(bytes[0], 0xab, "signed artifact is a map of 11 fields");

  const fixture = readFileSync(
    new URL("../../../crates/entitlement/tests/fixtures/ts_golden.txt", import.meta.url),
    "utf8",
  ).split("\n");
  assert.equal(fixture[3], v.artifactHex, "run `node scripts/golden.ts` to refresh the Rust fixture");
});

// ----- API router against a fake provider -----

function fakeDeps(purchases: Purchase[] = []): ApiDeps & { calls: string[] } {
  const calls: string[] = [];
  const provider: BillingProvider = {
    providerId: "fake",
    async ensureCustomer(id) { calls.push(`ensure:${id}`); },
    async checkoutUrl(id, product, oneTime) { calls.push(`checkout:${product}:${oneTime}`); return `https://pay.example/${id}`; },
    async portalUrl(id) { return `https://portal.example/${id}`; },
    async purchases() { return purchases; },
  };
  const { privateKey } = generateKeyPairSync("ed25519");
  return { provider, catalog, signingKey: { keyId: "k1", key: privateKey }, now: () => 1_800_000_000_000, calls };
}

const post = (path: string, body: unknown) =>
  new Request(`https://billing.example${path}`, { method: "POST", body: JSON.stringify(body) });

test("api: plans hide provider ids", async () => {
  const res = await handle(new Request("https://billing.example/v1/plans"), fakeDeps());
  const body = (await res.json()) as { plans: Array<Record<string, unknown>> };
  assert.equal(res.status, 200);
  assert.equal(body.plans.length, 3);
  assert.ok(!JSON.stringify(body).includes("enzime-pro-USD"));
});

test("api: checkout mints a license, ensures customer, routes lifetime to one-time", async () => {
  const deps = fakeDeps();
  const res = await handle(post("/v1/checkout", { deviceId: "device-123456", planId: "lifetime" }), deps);
  const body = (await res.json()) as { licenseId: string; url: string };
  assert.equal(res.status, 200);
  assert.ok(validLicenseId(body.licenseId));
  assert.deepEqual(deps.calls, [`ensure:${body.licenseId}`, "checkout:enzime-lifetime-USD:true"]);
});

test("api: input validation", async () => {
  const deps = fakeDeps();
  assert.equal((await handle(post("/v1/checkout", { deviceId: "x", planId: "lifetime" }), deps)).status, 400);
  assert.equal((await handle(post("/v1/checkout", { deviceId: "device-123456", planId: "nope" }), deps)).status, 400);
  assert.equal((await handle(post("/v1/portal", { licenseId: "enz_bad" }), deps)).status, 400);
  assert.equal((await handle(new Request("https://b.example/v1/entitlement", { method: "POST", body: "[]" }), deps)).status, 400);
});

test("api: entitlement 404 without purchases, signed artifact with purchases", async () => {
  const lic = newLicenseId();
  const none = await handle(post("/v1/entitlement", { deviceId: "device-123456", licenseId: lic }), fakeDeps());
  assert.equal(none.status, 404);
  const some = await handle(
    post("/v1/entitlement", { deviceId: "device-123456", licenseId: lic }),
    fakeDeps([{ providerProductId: "enzime-lifetime-USD", kind: { type: "one_time" }, reference: "inv:1" }]),
  );
  const body = (await some.json()) as { artifact: string; expiresAtMs: number | null; grants: unknown[] };
  assert.equal(some.status, 200);
  assert.equal(body.expiresAtMs, null);
  assert.equal(body.grants.length, 6);
  assert.ok(Buffer.from(body.artifact, "base64").length > 200);
});

// ----- Chargebee adapter against a recorded fake -----

test("chargebee: creates missing customer with our id, then parses subs + paid charge invoices", async () => {
  const seen: Array<{ url: string; method: string; body?: string }> = [];
  const fakeFetch = (async (url: string | URL, init?: RequestInit) => {
    const u = String(url);
    seen.push({ url: u, method: init?.method ?? "GET", body: init?.body as string | undefined });
    const ok = (b: unknown) => new Response(JSON.stringify(b), { status: 200 });
    if (u.includes("/customers/") && (init?.method ?? "GET") === "GET") {
      return new Response(JSON.stringify({ api_error_code: "resource_not_found" }), { status: 404 });
    }
    if (u.endsWith("/customers")) return ok({ customer: { id: "x" } });
    if (u.includes("/subscriptions")) {
      return ok({ list: [{ subscription: { id: "s1", status: "active", current_term_end: 1900000000, subscription_items: [{ item_price_id: "enzime-pro-USD-Yearly" }] } }] });
    }
    if (u.includes("/invoices")) {
      return ok({ list: [{ invoice: { id: "i1", status: "paid", line_items: [
        { entity_type: "charge_item_price", entity_id: "enzime-lifetime-USD" },
        { entity_type: "plan_item_price", entity_id: "enzime-pro-USD-Yearly" },
      ] } }] });
    }
    throw new Error(`unexpected ${u}`);
  }) as typeof fetch;

  const cb = new ChargebeeProvider("enzime-test", "test_key", "https://billing.example/", fakeFetch);
  const lic = newLicenseId();
  await cb.ensureCustomer(lic);
  assert.equal(seen[1].method, "POST");
  assert.ok(seen[1].body?.includes(`id=${lic}`));

  const purchases = await cb.purchases(lic);
  assert.deepEqual(purchases, [
    { providerProductId: "enzime-pro-USD-Yearly", kind: { type: "subscription", paidThroughMs: 1900000000000 }, reference: "sub:s1" },
    { providerProductId: "enzime-lifetime-USD", kind: { type: "one_time" }, reference: "inv:i1" },
  ]);
  const subUrl = seen.find((s) => s.url.includes("/subscriptions"))!.url;
  assert.ok(decodeURIComponent(subUrl).includes(`customer_id[is]=${lic}`));
});
