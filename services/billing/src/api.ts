// Request router for the billing API. Pure function of (Request, deps) so it
// runs under Netlify Functions, `node --test`, or any fetch-style host.
//
//   GET  /v1/plans                         → { plans: PublicPlan[] }
//   POST /v1/checkout     {deviceId, planId, licenseId?} → { licenseId, url }
//   POST /v1/portal       {licenseId}                    → { url }
//   POST /v1/entitlement  {deviceId, licenseId}          → { artifact, grants, expiresAtMs } | 404
//   GET  /v1/health                        → { ok, provider }

import { planById, publicPlans, type Catalog } from "./catalog.ts";
import { issueArtifact, type SigningKey } from "./issue.ts";
import {
  BillingError,
  newLicenseId,
  validDeviceId,
  validLicenseId,
  type BillingProvider,
} from "./provider.ts";

export interface ApiDeps {
  provider: BillingProvider;
  catalog: Catalog;
  signingKey: SigningKey;
  now?: () => number;
}

const MAX_BODY = 4096;

function json(status: number, body: unknown): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: {
      "Content-Type": "application/json",
      "Cache-Control": "no-store",
      "Access-Control-Allow-Origin": "*",
    },
  });
}

const fail = (status: number, code: string, message: string): Response =>
  json(status, { error: { code, message } });

async function readBody(req: Request): Promise<Record<string, unknown> | null> {
  const text = await req.text();
  if (text.length > MAX_BODY) return null;
  try {
    const v = JSON.parse(text) as unknown;
    return v && typeof v === "object" && !Array.isArray(v) ? (v as Record<string, unknown>) : null;
  } catch {
    return null;
  }
}

function mapError(e: unknown): Response {
  if (e instanceof BillingError) {
    if (e.kind === "unavailable") return fail(503, "backend_unavailable", "Billing provider unavailable");
    if (e.kind === "not_found") return fail(404, "not_found", "License not found");
    return fail(502, "provider_rejected", e.message);
  }
  console.error("billing api internal error", e);
  return fail(500, "internal", "Internal error");
}

export async function handle(req: Request, deps: ApiDeps): Promise<Response> {
  const path = new URL(req.url).pathname.replace(/^\/api/, "").replace(/\/+$/, "");
  const now = deps.now ?? Date.now;

  if (req.method === "OPTIONS") {
    return new Response(null, {
      status: 204,
      headers: {
        "Access-Control-Allow-Origin": "*",
        "Access-Control-Allow-Methods": "GET, POST, OPTIONS",
        "Access-Control-Allow-Headers": "Content-Type",
      },
    });
  }

  try {
    if (req.method === "GET" && path === "/v1/health") {
      return json(200, { ok: true, provider: deps.provider.providerId });
    }
    if (req.method === "GET" && path === "/v1/plans") {
      return json(200, { plans: publicPlans(deps.catalog) });
    }
    if (req.method !== "POST") return fail(404, "not_found", "No such route");

    const body = await readBody(req);
    if (!body) return fail(400, "invalid_request", "Body must be a JSON object under 4 KiB");

    if (path === "/v1/checkout") {
      const { deviceId, planId } = body;
      if (!validDeviceId(deviceId)) return fail(400, "invalid_request", "deviceId invalid");
      const plan = typeof planId === "string" ? planById(deps.catalog, planId) : undefined;
      if (!plan) return fail(400, "invalid_request", "Unknown planId");
      let licenseId = body.licenseId;
      if (licenseId === undefined || licenseId === null) licenseId = newLicenseId();
      if (!validLicenseId(licenseId)) return fail(400, "invalid_request", "licenseId invalid");
      await deps.provider.ensureCustomer(licenseId);
      const url = await deps.provider.checkoutUrl(
        licenseId,
        plan.provider_product_id,
        plan.kind === "lifetime",
      );
      return json(200, { licenseId, url });
    }

    if (path === "/v1/portal") {
      if (!validLicenseId(body.licenseId)) return fail(400, "invalid_request", "licenseId invalid");
      return json(200, { url: await deps.provider.portalUrl(body.licenseId) });
    }

    if (path === "/v1/entitlement") {
      const { deviceId, licenseId } = body;
      if (!validDeviceId(deviceId)) return fail(400, "invalid_request", "deviceId invalid");
      if (!validLicenseId(licenseId)) return fail(400, "invalid_request", "licenseId invalid");
      const purchases = await deps.provider.purchases(licenseId);
      const issued = issueArtifact(
        {
          deviceId,
          providerId: deps.provider.providerId,
          purchases,
          catalog: deps.catalog,
          nowMs: now(),
        },
        deps.signingKey,
      );
      if (!issued) return fail(404, "no_purchases", "No active purchases for this license");
      return json(200, {
        artifact: Buffer.from(issued.artifactCbor).toString("base64"),
        artifactId: issued.artifactId,
        grants: issued.grants.map((g) => ({
          capabilityKey: g.capability_key,
          grantKind: g.grant_kind,
          validUntilMs: g.valid_until_ms,
        })),
        expiresAtMs: issued.expiresAtMs,
      });
    }

    return fail(404, "not_found", "No such route");
  } catch (e) {
    return mapError(e);
  }
}
