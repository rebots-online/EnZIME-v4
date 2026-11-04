// Local billing server.
//   node scripts/dev-server.ts            → mock provider (no Chargebee needed)
//   CHARGEBEE_SITE=… CHARGEBEE_API_KEY=… node scripts/dev-server.ts  → real Chargebee (test site!)
// Env: PORT (default 8787), ENTITLEMENT_KEY_ID, ENTITLEMENT_SIGNING_KEY
// (if absent a throwaway key is generated and its public half printed).
// Run the app against it:
//   ENZIME_BILLING_URL=http://127.0.0.1:8787 ENZIME_ENTITLEMENT_KEY_ID=… ENZIME_ENTITLEMENT_PUBKEY=… pnpm tauri dev

import { generateKeyPairSync } from "node:crypto";
import { readFileSync } from "node:fs";
import { createServer } from "node:http";
import { handle, type ApiDeps } from "../src/api.ts";
import { parseCatalog } from "../src/catalog.ts";
import { ChargebeeProvider } from "../src/chargebee.ts";
import { loadSigningKey, type SigningKey } from "../src/issue.ts";
import { MockProvider } from "../src/mock.ts";

const port = Number(process.env.PORT ?? 8787);
const base = `http://127.0.0.1:${port}`;
const catalog = parseCatalog(readFileSync(new URL("../catalog.json", import.meta.url), "utf8"));

let signingKey: SigningKey;
if (process.env.ENTITLEMENT_SIGNING_KEY && process.env.ENTITLEMENT_KEY_ID) {
  signingKey = loadSigningKey(process.env.ENTITLEMENT_KEY_ID, process.env.ENTITLEMENT_SIGNING_KEY);
} else {
  const { privateKey, publicKey } = generateKeyPairSync("ed25519");
  signingKey = { keyId: "dev-ephemeral", key: privateKey };
  const raw = publicKey.export({ format: "der", type: "spki" }).subarray(12).toString("hex");
  console.log(`ENZIME_ENTITLEMENT_KEY_ID=dev-ephemeral`);
  console.log(`ENZIME_ENTITLEMENT_PUBKEY=${raw}`);
}

const mock = new MockProvider(base);
const provider =
  process.env.CHARGEBEE_SITE && process.env.CHARGEBEE_API_KEY
    ? new ChargebeeProvider(process.env.CHARGEBEE_SITE, process.env.CHARGEBEE_API_KEY, `${base}/`)
    : mock;
const deps: ApiDeps = { provider, catalog, signingKey };

createServer(async (req, res) => {
  const url = new URL(req.url ?? "/", base);
  if (provider === mock && url.pathname === "/mock-pay") {
    mock.pay(url.searchParams.get("license") ?? "", url.searchParams.get("product") ?? "", url.searchParams.get("oneTime") === "true");
    res.writeHead(200, { "Content-Type": "text/html" }).end("<h1>Mock payment complete</h1><p>Return to EnZIME.</p>");
    return;
  }
  const chunks: Buffer[] = [];
  for await (const c of req) chunks.push(c as Buffer);
  const request = new Request(url, {
    method: req.method,
    headers: req.headers as Record<string, string>,
    body: req.method === "GET" || req.method === "HEAD" ? undefined : Buffer.concat(chunks),
  });
  const response = await handle(request, deps);
  res.writeHead(response.status, Object.fromEntries(response.headers));
  res.end(Buffer.from(await response.arrayBuffer()));
}).listen(port, "127.0.0.1", () => {
  console.log(`billing dev server (${provider.providerId}) on ${base}`);
});
