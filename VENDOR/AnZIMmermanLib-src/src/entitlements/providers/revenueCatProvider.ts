// Copyright (C) 2025–2026 Robin L. M. Cheung, MBA
// All rights reserved.
// Unauthorized use without prior written consent is strictly prohibited.

import type { Entitlements, EntitlementsProvider, Tier } from '../types';
import { createDefaultEntitlements, createAnnotatorEntitlements, createCreatorEntitlements } from '../defaults';

// RevenueCat entitlement identifiers (configured in RC dashboard)
const RC_ENTITLEMENT_ANNOTATOR = 'annotator';
const RC_ENTITLEMENT_CREATOR = 'creator';

interface RevenueCatCustomerInfo {
  request_date: string;
  subscriber: {
    entitlements: Record<string, {
      expires_date: string | null;
      product_identifier: string;
      purchase_date: string;
    }>;
    subscriptions: Record<string, unknown>;
    non_subscriptions: Record<string, unknown[]>;
  };
}

interface RevenueCatProviderOptions {
  apiKey: string;
  appUserId?: string;
  fallbackProvider?: EntitlementsProvider;
}

export class RevenueCatProvider implements EntitlementsProvider {
  private apiKey: string;
  private appUserId: string;
  private fallbackProvider: EntitlementsProvider;

  constructor(options: RevenueCatProviderOptions) {
    this.apiKey = options.apiKey;
    this.appUserId = options.appUserId ?? '$RCAnonymousID';
    this.fallbackProvider = options.fallbackProvider ?? {
      getEntitlements: async () => createDefaultEntitlements(),
    };
  }

  async getEntitlements(): Promise<Entitlements> {
    try {
      const customerInfo = await this.fetchCustomerInfo();
      if (!customerInfo) {
        return this.fallbackProvider.getEntitlements();
      }

      const tier = this.resolveTier(customerInfo);
      return this.buildEntitlements(tier);
    } catch {
      return this.fallbackProvider.getEntitlements();
    }
  }

  private resolveTier(info: RevenueCatCustomerInfo): Tier {
    const entitlements = info.subscriber.entitlements;

    // Creator includes everything — check first
    if (entitlements[RC_ENTITLEMENT_CREATOR]) {
      const exp = entitlements[RC_ENTITLEMENT_CREATOR].expires_date;
      if (!exp || new Date(exp) > new Date()) {
        return 'creator';
      }
    }

    // Annotator
    if (entitlements[RC_ENTITLEMENT_ANNOTATOR]) {
      const exp = entitlements[RC_ENTITLEMENT_ANNOTATOR].expires_date;
      if (!exp || new Date(exp) > new Date()) {
        return 'annotator';
      }
    }

    return 'free';
  }

  private buildEntitlements(tier: Tier): Entitlements {
    switch (tier) {
      case 'creator':
        return createCreatorEntitlements();
      case 'annotator':
        return createAnnotatorEntitlements();
      default:
        return createDefaultEntitlements();
    }
  }

  private async fetchCustomerInfo(): Promise<RevenueCatCustomerInfo | null> {
    try {
      const response = await fetch(
        `https://api.revenuecat.com/v1/subscribers/${encodeURIComponent(this.appUserId)}`,
        {
          headers: {
            'Authorization': `Bearer ${this.apiKey}`,
            'Content-Type': 'application/json',
          },
        }
      );
      if (!response.ok) {
        return null;
      }
      return (await response.json()) as RevenueCatCustomerInfo;
    } catch {
      return null;
    }
  }
}
