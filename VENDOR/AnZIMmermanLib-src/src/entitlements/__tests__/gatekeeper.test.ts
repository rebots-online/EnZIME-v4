// Copyright (C) 2025–2026 Robin L. M. Cheung, MBA
// All rights reserved.
// Unauthorized use without prior written consent is strictly prohibited.

import { describe, expect, it } from 'vitest';
import { createDefaultEntitlements } from '../defaults';
import { EntitlementsGatekeeper } from '../gatekeeper';
import type { Entitlements, EntitlementsProvider } from '../types';

const buildEntitlements = (overrides: Omit<Partial<Entitlements>, 'capabilities'> & { capabilities?: Partial<Entitlements['capabilities']> }): Entitlements => ({
  ...createDefaultEntitlements(),
  ...overrides,
  capabilities: {
    ...createDefaultEntitlements().capabilities,
    ...(overrides.capabilities ?? {}),
  } as Entitlements['capabilities'],
});

describe('EntitlementsGatekeeper', () => {
  it('free tier: reader unrestricted, annotations trial 25', () => {
    const fallback = createDefaultEntitlements();
    const provider: EntitlementsProvider = {
      getEntitlements: async () => fallback,
    };
    const gatekeeper = new EntitlementsGatekeeper(provider, fallback);

    expect(gatekeeper.can('reader.view')).toBe(true);
    expect(gatekeeper.can('archives.add')).toBe(true);
    expect(gatekeeper.can('search.view')).toBe(true);
    expect(gatekeeper.can('bookmarks.view')).toBe(true);
    expect(gatekeeper.can('history.view')).toBe(true);
    expect(gatekeeper.can('annotations.create')).toBe(true);
    expect(gatekeeper.limit('annotations.create')).toBe(25);
    expect(gatekeeper.can('mesh.view')).toBe(false);
    expect(gatekeeper.can('assistant.ai')).toBe(false);
    expect(gatekeeper.can('zimmer.extension')).toBe(false);
  });

  it('annotator tier: unlimited annotations, mesh, custom themes', async () => {
    const fallback = createDefaultEntitlements();
    const providerEntitlements = buildEntitlements({
      capabilities: {
        'annotations.create': { allowed: true, tier: 'annotator' },
        'themes.custom': { allowed: true, tier: 'annotator' },
        'mesh.view': { allowed: true, tier: 'annotator' },
      },
    });
    const provider: EntitlementsProvider = {
      getEntitlements: async () => providerEntitlements,
    };
    const gatekeeper = new EntitlementsGatekeeper(provider, fallback);

    await gatekeeper.refresh();

    expect(gatekeeper.can('annotations.create')).toBe(true);
    expect(gatekeeper.limit('annotations.create')).toBe(Infinity);
    expect(gatekeeper.can('mesh.view')).toBe(true);
    expect(gatekeeper.can('themes.custom')).toBe(true);
    expect(gatekeeper.tier('mesh.view')).toBe('annotator');
    expect(gatekeeper.can('assistant.ai')).toBe(false);
  });

  it('creator tier: everything unlocked', async () => {
    const fallback = createDefaultEntitlements();
    const providerEntitlements = buildEntitlements({
      capabilities: {
        'annotations.create': { allowed: true, tier: 'creator' },
        'themes.custom': { allowed: true, tier: 'creator' },
        'mesh.view': { allowed: true, tier: 'creator' },
        'assistant.ai': { allowed: true, tier: 'creator' },
        'zimmer.extension': { allowed: true, tier: 'creator' },
        'zim.editor': { allowed: true, tier: 'creator' },
        'annotations.export': { allowed: true, tier: 'creator' },
      },
    });
    const provider: EntitlementsProvider = {
      getEntitlements: async () => providerEntitlements,
    };
    const gatekeeper = new EntitlementsGatekeeper(provider, fallback);

    await gatekeeper.refresh();

    expect(gatekeeper.can('assistant.ai')).toBe(true);
    expect(gatekeeper.can('zimmer.extension')).toBe(true);
    expect(gatekeeper.can('zim.editor')).toBe(true);
    expect(gatekeeper.can('annotations.export')).toBe(true);
    expect(gatekeeper.tier('assistant.ai')).toBe('creator');
  });
});
