// In-memory BillingProvider for local development and end-to-end tests.
// "Checkout" is a local URL that marks the purchase as paid when opened.

import type { BillingProvider, Purchase } from "./provider.ts";

export class MockProvider implements BillingProvider {
  readonly providerId = "mock";
  private readonly customers = new Set<string>();
  private readonly owned = new Map<string, Purchase[]>();
  private readonly baseUrl: string;

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl;
  }

  async ensureCustomer(licenseId: string): Promise<void> {
    this.customers.add(licenseId);
  }

  async checkoutUrl(licenseId: string, productId: string, oneTime: boolean): Promise<string> {
    const q = new URLSearchParams({ license: licenseId, product: productId, oneTime: String(oneTime) });
    return `${this.baseUrl}/mock-pay?${q}`;
  }

  async portalUrl(licenseId: string): Promise<string> {
    return `${this.baseUrl}/mock-portal?license=${licenseId}`;
  }

  async purchases(licenseId: string): Promise<Purchase[]> {
    return this.owned.get(licenseId) ?? [];
  }

  /** Simulate a completed payment. */
  pay(licenseId: string, productId: string, oneTime: boolean, nowMs = Date.now()): void {
    const list = this.owned.get(licenseId) ?? [];
    list.push({
      providerProductId: productId,
      kind: oneTime ? { type: "one_time" } : { type: "subscription", paidThroughMs: nowMs + 30 * 86_400_000 },
      reference: `mock:${list.length + 1}`,
    });
    this.owned.set(licenseId, list);
  }
}
