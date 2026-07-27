// Copyright (C) 2025–2026 Robin L. M. Cheung, MBA
// All rights reserved.
// Unauthorized use without prior written consent is strictly prohibited.

// ── Capability keys ──────────────────────────────────────────────────
// Free tier (unrestricted reader)
// Annotator tier ($)  — annotations, custom themes, mesh
// Creator tier ($$)   — AI, Zimmer extension, ZIM editor, export

export type EntitlementKey =
  // Free — always on
  | 'reader.view'
  | 'archives.add'
  | 'search.view'
  | 'bookmarks.view'
  | 'history.view'
  | 'themes.basic'
  // Annotator ($) — trial 25, then paid
  | 'annotations.create'
  | 'themes.custom'
  | 'mesh.view'
  // Creator ($$)
  | 'assistant.ai'
  | 'zimmer.extension'
  | 'zim.editor'
  | 'annotations.export';

export type Tier = 'free' | 'annotator' | 'creator';

export interface EntitlementsQuota {
  remaining: number;
  resetAt?: string;
}

export interface EntitlementValue {
  allowed: boolean;
  limit?: number;
  tier?: Tier;
  quota?: EntitlementsQuota;
}

export interface Entitlements {
  issuedAt: string;
  expiresAt?: string;
  capabilities: Record<EntitlementKey, EntitlementValue>;
}

export interface EntitlementsProvider {
  getEntitlements(): Promise<Entitlements>;
}

// ── Virtual currency / micropayments (future: BTC Lightning) ────────

export interface VirtualCurrencyBalance {
  balance: number;
  currency: 'credits' | 'sats';
}

export interface MicropaymentItem {
  id: string;
  name: string;
  description: string;
  priceCredits: number;
  priceSats?: number;
  type: 'annotation_pack' | 'highlight_pack' | 'theme_pack' | 'export_pack';
  quantity: number;
}

export interface MicropaymentProvider {
  getBalance(): Promise<VirtualCurrencyBalance>;
  purchase(item: MicropaymentItem): Promise<boolean>;
  topUp(amount: number, method: 'revenuecat' | 'lightning'): Promise<boolean>;
}
