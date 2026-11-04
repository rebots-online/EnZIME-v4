import { invoke } from "@tauri-apps/api/core";

// §13 / §14.4 entitlement + billing wire contract. The UI only ever sees
// CapabilityDecision[] and public plan info — never provider state.

export type CapabilityKey =
  | "reader.core"
  | "reader.unlimited_packs"
  | "annotations.rich_media"
  | "ask.local_inference"
  | "memory.lore"
  | "backup.encrypted"
  | "sidecar.signed_export"
  | "models.premium_catalog";

export type CapabilityDecision = {
  readonly capabilityKey: CapabilityKey | string;
  readonly allowed: boolean;
  readonly source: "core" | "signed_artifact" | "grace_policy" | "denied";
  readonly validUntilMs: number | null;
  readonly explanation: string;
};

export type BillingStatus = {
  readonly purchasingEnabled: boolean;
  readonly licenseId: string | null;
  readonly decisions: readonly CapabilityDecision[];
};

export type Plan = {
  readonly id: string;
  readonly name: string;
  readonly priceLabel: string;
  readonly kind: "subscription" | "lifetime";
  readonly capabilities: readonly string[];
  readonly highlight: boolean;
};

export type BillingErrorCode =
  | "unsupported"
  | "offline"
  | "backend_unavailable"
  | "not_found"
  | "invalid_request"
  | "signature_invalid"
  | "internal";

export class BillingError extends Error {
  readonly code: BillingErrorCode;
  constructor(code: BillingErrorCode, message: string) {
    super(message);
    this.code = code;
  }
}

type Envelope<T> = {
  ok: boolean;
  value?: T | null;
  error?: { code: BillingErrorCode; message: string } | null;
};

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const r = await invoke<Envelope<T>>(command, args);
  if (!r.ok || r.value === undefined || r.value === null) {
    throw new BillingError(r.error?.code ?? "internal", r.error?.message ?? "Unexpected error");
  }
  return r.value;
}

export const billingService = {
  status: () => call<BillingStatus>("billing_status"),
  plans: () => call<Plan[]>("billing_plans"),
  checkout: (planId: string) => call<{ url: string; licenseId: string }>("billing_checkout", { planId }),
  portal: () => call<string>("billing_portal"),
  refresh: () => call<BillingStatus>("entitlement_refresh"),
  restore: (licenseId: string) => call<BillingStatus>("entitlement_restore", { licenseId }),
};

/** Human copy for capability keys (§13.1). */
export const CAPABILITY_LABELS: Record<string, { title: string; detail: string }> = {
  "reader.core": { title: "Offline reading", detail: "Read, highlight, comment, bookmark and export — always free." },
  "reader.unlimited_packs": { title: "Unlimited packs", detail: "Install as many knowledge packs as your storage fits." },
  "annotations.rich_media": { title: "Rich-media notes", detail: "Ink, voice, photo and video annotations." },
  "ask.local_inference": { title: "Ask, on-device", detail: "Private local AI answers with citations." },
  "memory.lore": { title: "Memory", detail: "Reviewable long-term memory built from your reading." },
  "backup.encrypted": { title: "Encrypted backup", detail: "Verified, encrypted backups you control." },
  "sidecar.signed_export": { title: "Signed sharing", detail: "Export signed note layers others can verify." },
  "models.premium_catalog": { title: "Premium models", detail: "Access to the premium on-device model catalog." },
};
