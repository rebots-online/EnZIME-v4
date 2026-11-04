// Plan catalog: stable EnZIME plan id → provider product id + §13.1
// capability keys. Provider ids stay server-side; the app sees PublicPlan.

export const CAPABILITY_KEYS = [
  "reader.unlimited_packs",
  "annotations.rich_media",
  "ask.local_inference",
  "memory.lore",
  "backup.encrypted",
  "sidecar.signed_export",
  "models.premium_catalog",
] as const;
export type CapabilityKey = (typeof CAPABILITY_KEYS)[number];

export type PlanKind = "subscription" | "lifetime";

export interface Plan {
  id: string;
  name: string;
  price_label: string;
  kind: PlanKind;
  /** Chargebee item_price_id today; WooCommerce product/variation id later. */
  provider_product_id: string;
  capabilities: CapabilityKey[];
  highlight?: boolean;
}

export interface PublicPlan {
  id: string;
  name: string;
  priceLabel: string;
  kind: PlanKind;
  capabilities: CapabilityKey[];
  highlight: boolean;
}

export interface Catalog {
  plans: Plan[];
}

export function parseCatalog(json: string): Catalog {
  const raw = JSON.parse(json) as Catalog;
  if (!raw || !Array.isArray(raw.plans) || raw.plans.length === 0) {
    throw new Error("catalog: no plans");
  }
  const ids = new Set<string>();
  const products = new Set<string>();
  for (const p of raw.plans) {
    if (!p.id || !p.provider_product_id || !p.name || !p.price_label) {
      throw new Error("catalog: plan missing id/name/price_label/provider_product_id");
    }
    if (p.kind !== "subscription" && p.kind !== "lifetime") {
      throw new Error(`catalog: plan ${p.id} has invalid kind`);
    }
    if (ids.has(p.id)) throw new Error(`catalog: duplicate plan ${p.id}`);
    if (products.has(p.provider_product_id)) {
      throw new Error(`catalog: duplicate product ${p.provider_product_id}`);
    }
    ids.add(p.id);
    products.add(p.provider_product_id);
    if (!Array.isArray(p.capabilities) || p.capabilities.length === 0) {
      throw new Error(`catalog: plan ${p.id} grants nothing`);
    }
    for (const c of p.capabilities) {
      // reader.core is always free (§13.1) and therefore absent from the list.
      if (!(CAPABILITY_KEYS as readonly string[]).includes(c)) {
        throw new Error(`catalog: plan ${p.id} has unknown capability ${c}`);
      }
    }
  }
  return raw;
}

export const publicPlans = (c: Catalog): PublicPlan[] =>
  c.plans.map((p) => ({
    id: p.id,
    name: p.name,
    priceLabel: p.price_label,
    kind: p.kind,
    capabilities: [...p.capabilities],
    highlight: Boolean(p.highlight),
  }));

export const planById = (c: Catalog, id: string): Plan | undefined =>
  c.plans.find((p) => p.id === id);

export const planByProduct = (c: Catalog, productId: string): Plan | undefined =>
  c.plans.find((p) => p.provider_product_id === productId);
