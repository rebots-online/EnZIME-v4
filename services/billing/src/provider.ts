// Commerce-provider boundary. Chargebee implements it today
// (`chargebee.ts`); the WooCommerce ecosystem plugin will implement the same
// interface later. Nothing outside this file and the adapters knows which
// provider is live, and the desktop app never does.

export type PurchaseKind =
  | { type: "subscription"; paidThroughMs: number }
  | { type: "one_time" };

export interface Purchase {
  providerProductId: string;
  kind: PurchaseKind;
  /** Provider's own subscription/invoice id; digested, never shipped raw. */
  reference: string;
}

export type BillingErrorKind = "unavailable" | "rejected" | "not_found";

export class BillingError extends Error {
  readonly kind: BillingErrorKind;
  constructor(kind: BillingErrorKind, message: string) {
    super(message);
    this.kind = kind;
  }
}

export interface BillingProvider {
  readonly providerId: string;
  /** Create the customer (id = our license id) if it does not exist yet. */
  ensureCustomer(licenseId: string): Promise<void>;
  checkoutUrl(licenseId: string, providerProductId: string, oneTime: boolean): Promise<string>;
  portalUrl(licenseId: string): Promise<string>;
  /** Every purchase that currently entitles the customer. */
  purchases(licenseId: string): Promise<Purchase[]>;
}

const LICENSE_RE = /^enz_[0-9a-z]{26}$/;
const CROCKFORD = "0123456789abcdefghjkmnpqrstvwxyz";

/**
 * New license id: `enz_` + 26 lowercase Crockford base32 chars (130 random
 * bits). It is the bearer restore key users paste on another device, so it
 * must never be derived from email or a counter.
 */
export function newLicenseId(): string {
  const bytes = crypto.getRandomValues(new Uint8Array(26));
  let s = "enz_";
  for (const b of bytes) s += CROCKFORD[b & 31];
  return s;
}

export const validLicenseId = (s: unknown): s is string =>
  typeof s === "string" && LICENSE_RE.test(s);

export const validDeviceId = (s: unknown): s is string =>
  typeof s === "string" && /^[0-9A-Za-z_-]{8,64}$/.test(s);
