// Chargebee (API v2, Product Catalog 2.0) implementation of BillingProvider.
// Holds the full-access API key; runs only inside the Netlify function.

import { BillingError, type BillingProvider, type Purchase } from "./provider.ts";

type FetchFn = typeof fetch;

const ENTITLING_SUB_STATUSES = ["active", "in_trial", "non_renewing"];

interface ListResponse<K extends string, T> {
  list: Array<Record<K, T>>;
  next_offset?: string;
}

interface CbSubscription {
  id: string;
  status: string;
  current_term_end?: number;
  trial_end?: number;
  subscription_items?: Array<{ item_price_id: string }>;
}

interface CbInvoice {
  id: string;
  status: string;
  line_items?: Array<{ entity_type: string; entity_id?: string }>;
}

export class ChargebeeProvider implements BillingProvider {
  readonly providerId = "chargebee";
  private readonly base: string;
  private readonly auth: string;
  private readonly returnUrl: string;
  private readonly fetchFn: FetchFn;

  constructor(site: string, apiKey: string, returnUrl: string, fetchFn: FetchFn = fetch) {
    this.returnUrl = returnUrl;
    this.fetchFn = fetchFn;
    if (!/^[a-z0-9-]+$/.test(site)) throw new Error("chargebee: invalid site name");
    if (!apiKey) throw new Error("chargebee: missing API key");
    this.base = `https://${site}.chargebee.com/api/v2`;
    this.auth = "Basic " + Buffer.from(`${apiKey}:`).toString("base64");
  }

  private async call<T>(
    method: "GET" | "POST",
    path: string,
    params: Record<string, string> = {},
  ): Promise<T> {
    const body = new URLSearchParams(params).toString();
    const url = method === "GET" && body ? `${this.base}${path}?${body}` : `${this.base}${path}`;
    let res: Response;
    try {
      res = await this.fetchFn(url, {
        method,
        headers: {
          Authorization: this.auth,
          Accept: "application/json",
          ...(method === "POST" ? { "Content-Type": "application/x-www-form-urlencoded" } : {}),
        },
        body: method === "POST" ? body : undefined,
      });
    } catch (e) {
      throw new BillingError("unavailable", `chargebee unreachable: ${String(e)}`);
    }
    const text = await res.text();
    if (res.status === 404) throw new BillingError("not_found", "chargebee: not found");
    if (res.status >= 500 || res.status === 429) {
      throw new BillingError("unavailable", `chargebee ${res.status}`);
    }
    if (!res.ok) {
      // Chargebee error bodies carry api_error_code + message; never the key.
      let code = String(res.status);
      try {
        code = (JSON.parse(text) as { api_error_code?: string }).api_error_code ?? code;
      } catch {
        /* non-JSON error body */
      }
      throw new BillingError("rejected", `chargebee rejected: ${code}`);
    }
    return JSON.parse(text) as T;
  }

  async ensureCustomer(licenseId: string): Promise<void> {
    try {
      await this.call("GET", `/customers/${encodeURIComponent(licenseId)}`);
      return;
    } catch (e) {
      if (!(e instanceof BillingError && e.kind === "not_found")) throw e;
    }
    // Checkout would otherwise auto-create a customer with a *generated* id,
    // breaking the license-id ↔ customer-id mapping. Create it explicitly.
    await this.call("POST", "/customers", {
      id: licenseId,
      "meta_data": JSON.stringify({ source: "enzime-reader" }),
    });
  }

  async checkoutUrl(licenseId: string, productId: string, oneTime: boolean): Promise<string> {
    const path = oneTime
      ? "/hosted_pages/checkout_one_time_for_items"
      : "/hosted_pages/checkout_new_for_items";
    const params: Record<string, string> = {
      "customer[id]": licenseId,
      redirect_url: this.returnUrl,
    };
    if (oneTime) {
      params["item_prices[item_price_id][0]"] = productId;
      params["item_prices[quantity][0]"] = "1";
    } else {
      params["subscription_items[item_price_id][0]"] = productId;
      params["subscription_items[quantity][0]"] = "1";
    }
    const r = await this.call<{ hosted_page: { url: string } }>("POST", path, params);
    return r.hosted_page.url;
  }

  async portalUrl(licenseId: string): Promise<string> {
    const r = await this.call<{ portal_session: { access_url: string } }>(
      "POST",
      "/portal_sessions",
      { "customer[id]": licenseId, redirect_url: this.returnUrl },
    );
    return r.portal_session.access_url;
  }

  async purchases(licenseId: string): Promise<Purchase[]> {
    const out: Purchase[] = [];

    const subs = await this.listAll<"subscription", CbSubscription>("/subscriptions", "subscription", {
      "customer_id[is]": licenseId,
      "status[in]": JSON.stringify(ENTITLING_SUB_STATUSES),
    });
    for (const s of subs) {
      if (!ENTITLING_SUB_STATUSES.includes(s.status)) continue;
      const end = s.status === "in_trial" ? (s.trial_end ?? s.current_term_end) : s.current_term_end;
      if (!end) continue;
      for (const item of s.subscription_items ?? []) {
        out.push({
          providerProductId: item.item_price_id,
          kind: { type: "subscription", paidThroughMs: end * 1000 },
          reference: `sub:${s.id}`,
        });
      }
    }

    const invoices = await this.listAll<"invoice", CbInvoice>("/invoices", "invoice", {
      "customer_id[is]": licenseId,
      "status[is]": "paid",
    });
    for (const inv of invoices) {
      for (const li of inv.line_items ?? []) {
        if (li.entity_type === "charge_item_price" && li.entity_id) {
          out.push({
            providerProductId: li.entity_id,
            kind: { type: "one_time" },
            reference: `inv:${inv.id}`,
          });
        }
      }
    }
    return out;
  }

  private async listAll<K extends string, T>(
    path: string,
    key: K,
    params: Record<string, string>,
  ): Promise<T[]> {
    const items: T[] = [];
    let offset: string | undefined;
    // Hard page cap: a single customer never legitimately has >1000 records.
    for (let page = 0; page < 10; page++) {
      const r = await this.call<ListResponse<K, T>>("GET", path, {
        ...params,
        limit: "100",
        ...(offset ? { offset } : {}),
      });
      for (const row of r.list) items.push(row[key]);
      if (!r.next_offset) break;
      offset = r.next_offset;
    }
    return items;
  }
}
