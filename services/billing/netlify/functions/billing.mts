// Netlify Functions (v2) entry point. Free tier: 125k invocations / month,
// far above what entitlement refreshes need. All secrets come from Netlify
// environment variables — see services/billing/README.md.

import catalogJson from "../../catalog.json" with { type: "json" };
import { handle, type ApiDeps } from "../../src/api.ts";
import { parseCatalog } from "../../src/catalog.ts";
import { ChargebeeProvider } from "../../src/chargebee.ts";
import { loadSigningKey } from "../../src/issue.ts";

let deps: ApiDeps | undefined;

function env(name: string): string {
  const v = process.env[name];
  if (!v) throw new Error(`missing environment variable ${name}`);
  return v;
}

function init(): ApiDeps {
  if (deps) return deps;
  deps = {
    catalog: parseCatalog(JSON.stringify(catalogJson)),
    provider: new ChargebeeProvider(
      env("CHARGEBEE_SITE"),
      env("CHARGEBEE_API_KEY"),
      env("BILLING_RETURN_URL"),
    ),
    signingKey: loadSigningKey(env("ENTITLEMENT_KEY_ID"), env("ENTITLEMENT_SIGNING_KEY")),
  };
  return deps;
}

export default async (req: Request): Promise<Response> => {
  let d: ApiDeps;
  try {
    d = init();
  } catch (e) {
    console.error("billing function misconfigured:", (e as Error).message);
    return new Response(
      JSON.stringify({ error: { code: "backend_unavailable", message: "Billing not configured" } }),
      { status: 503, headers: { "Content-Type": "application/json" } },
    );
  }
  return handle(req, d);
};

export const config = { path: ["/v1/*"] };
