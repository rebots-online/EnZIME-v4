// Turn normalized purchases into a signed `enzime.entitlement.cbor.v1`
// artifact (ARCHITECTURE.md §13.3). Byte layout mirrors the Rust structs in
// crates/entitlement/src/artifact.rs field-for-field, in declaration order.

import { createHash, createPrivateKey, randomBytes, sign, type KeyObject } from "node:crypto";
import { encode, map, type CborMap, type CborValue } from "./cbor.ts";
import { planByProduct, type Catalog } from "./catalog.ts";
import type { Purchase } from "./provider.ts";

export const ARTIFACT_SCHEMA = "enzime.entitlement.cbor.v1";
/** Rust constant includes the trailing NUL ... */
export const ENTITLEMENT_SIGNING_DOMAIN = "enzime.entitlement.v1\0";
/** ... and the Ed25519 provider appends one more NUL as the domain separator. */
const SIGNED_PREFIX = new TextEncoder().encode(ENTITLEMENT_SIGNING_DOMAIN + "\0");

/** Clock-skew allowance so a freshly issued artifact is valid on a slow device clock. */
const NOT_BEFORE_SKEW_MS = 5 * 60 * 1000;

export interface Grant {
  capability_key: string;
  grant_kind: "perpetual" | "term";
  quantity: null;
  valid_until_ms: number | null;
}

export interface IssueInput {
  deviceId: string;
  providerId: string;
  purchases: Purchase[];
  catalog: Catalog;
  nowMs: number;
  /** For deterministic tests only. */
  nonce?: Uint8Array;
  artifactId?: string;
}

export interface SigningKey {
  keyId: string;
  key: KeyObject;
}

export function loadSigningKey(keyId: string, pkcs8Base64: string): SigningKey {
  const key = createPrivateKey({
    key: Buffer.from(pkcs8Base64, "base64"),
    format: "der",
    type: "pkcs8",
  });
  if (key.asymmetricKeyType !== "ed25519") throw new Error("signing key is not Ed25519");
  return { keyId, key };
}

/** Merge purchases into one grant per capability; perpetual beats term, later term beats earlier. */
export function grantsFor(purchases: Purchase[], catalog: Catalog): Grant[] {
  const byCap = new Map<string, Grant>();
  for (const p of purchases) {
    const plan = planByProduct(catalog, p.providerProductId);
    if (!plan) continue; // product not sold through EnZIME's catalog
    for (const cap of plan.capabilities) {
      const next: Grant =
        p.kind.type === "one_time"
          ? { capability_key: cap, grant_kind: "perpetual", quantity: null, valid_until_ms: null }
          : {
              capability_key: cap,
              grant_kind: "term",
              quantity: null,
              valid_until_ms: p.kind.paidThroughMs,
            };
      const prev = byCap.get(cap);
      const better =
        !prev ||
        (prev.grant_kind === "term" &&
          (next.grant_kind === "perpetual" ||
            (next.valid_until_ms ?? 0) > (prev.valid_until_ms ?? 0)));
      if (better) byCap.set(cap, next);
    }
  }
  return [...byCap.values()].sort((a, b) => a.capability_key.localeCompare(b.capability_key));
}

const grantCbor = (g: Grant): CborMap =>
  map(
    ["capability_key", g.capability_key],
    ["grant_kind", g.grant_kind],
    ["quantity", g.quantity],
    ["valid_until_ms", g.valid_until_ms],
  );

export interface IssuedArtifact {
  artifactCbor: Uint8Array;
  artifactId: string;
  grants: Grant[];
  expiresAtMs: number | null;
}

/** Returns null when nothing entitles the customer. */
export function issueArtifact(input: IssueInput, key: SigningKey): IssuedArtifact | null {
  const grants = grantsFor(input.purchases, input.catalog);
  if (grants.length === 0) return null;

  const expiresAtMs = grants.some((g) => g.grant_kind === "perpetual")
    ? null
    : Math.max(...grants.map((g) => g.valid_until_ms ?? 0));

  const seen = new Set<string>();
  const receipts: CborValue[] = [];
  for (const p of input.purchases) {
    const digest = createHash("sha256")
      .update(`${input.providerId}\0${p.reference}\0${p.providerProductId}`)
      .digest("hex");
    if (seen.has(digest)) continue;
    seen.add(digest);
    receipts.push(map(["provider", input.providerId], ["event_digest", digest]));
  }

  const artifactId = input.artifactId ?? `ent_${randomBytes(16).toString("hex")}`;
  const nonce = input.nonce ?? new Uint8Array(randomBytes(32));
  if (nonce.length !== 32) throw new Error("nonce must be 32 bytes");

  const unsignedFields: Array<readonly [string, CborValue]> = [
    ["schema", ARTIFACT_SCHEMA],
    ["artifact_id", artifactId],
    ["issuer_key_id", key.keyId],
    ["subject", map(["kind", "device"], ["subject_id", input.deviceId])],
    ["issued_at_ms", input.nowMs],
    ["not_before_ms", input.nowMs - NOT_BEFORE_SKEW_MS],
    ["expires_at_ms", expiresAtMs],
    ["grants", grants.map(grantCbor)],
    ["provider_receipts", receipts],
    ["nonce", nonce],
  ];

  const canonical = encode(map(...unsignedFields));
  const message = new Uint8Array(SIGNED_PREFIX.length + canonical.length);
  message.set(SIGNED_PREFIX, 0);
  message.set(canonical, SIGNED_PREFIX.length);
  const signature = new Uint8Array(sign(null, message, key.key));

  const artifactCbor = encode(
    map(
      ...unsignedFields,
      [
        "signature",
        map(
          ["algorithm", "ed25519"],
          ["key_id", key.keyId],
          ["signed_domain", ENTITLEMENT_SIGNING_DOMAIN],
          ["value", signature],
        ),
      ],
    ),
  );
  return { artifactCbor, artifactId, grants, expiresAtMs };
}
