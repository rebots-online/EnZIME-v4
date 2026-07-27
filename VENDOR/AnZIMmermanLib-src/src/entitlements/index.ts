// Copyright (C) 2025–2026 Robin L. M. Cheung, MBA
// All rights reserved.
// Unauthorized use without prior written consent is strictly prohibited.

import { createDefaultEntitlements } from './defaults';
import { EntitlementsGatekeeper } from './gatekeeper';
import type { EntitlementsProvider } from './types';
import { MockLocalEntitlementsProvider } from './providers/mockLocalProvider';
import { RemoteEntitlementsProvider } from './providers/remoteProvider';
import { RevenueCatProvider } from './providers/revenueCatProvider';
import { EntitlementsTokenCache } from './tokenCache';
import { useEntitlementsStore } from './store';

const fallbackEntitlements = createDefaultEntitlements();
const defaultProvider = new MockLocalEntitlementsProvider(fallbackEntitlements);

export const getEntitlementsGatekeeper = (): EntitlementsGatekeeper =>
  useEntitlementsStore.getState().gatekeeper;

export const initializeEntitlements = async (provider?: EntitlementsProvider): Promise<EntitlementsGatekeeper> => {
  const store = useEntitlementsStore.getState();
  let gatekeeper = store.gatekeeper;

  if (provider) {
    gatekeeper = new EntitlementsGatekeeper(provider, fallbackEntitlements);
  } else if (import.meta.env.VITE_REVENUECAT_API_KEY) {
    const rcProvider = new RevenueCatProvider({
      apiKey: import.meta.env.VITE_REVENUECAT_API_KEY,
      appUserId: import.meta.env.VITE_REVENUECAT_APP_USER_ID,
      fallbackProvider: defaultProvider,
    });
    gatekeeper = new EntitlementsGatekeeper(rcProvider, fallbackEntitlements);
  } else if (import.meta.env.VITE_BILLEDR_URL) {
    const remoteProvider = new RemoteEntitlementsProvider({
      endpoint: import.meta.env.VITE_BILLEDR_URL,
      fallbackProvider: defaultProvider,
      tokenCache: new EntitlementsTokenCache(),
      publicKey: import.meta.env.VITE_BILLEDR_PUBLIC_KEY,
    });
    gatekeeper = new EntitlementsGatekeeper(remoteProvider, fallbackEntitlements);
  }

  await gatekeeper.refresh();
  useEntitlementsStore.setState({ gatekeeper, ready: true, lastUpdated: new Date().toISOString() });
  return gatekeeper;
};

export type { EntitlementKey, Tier } from './types';
