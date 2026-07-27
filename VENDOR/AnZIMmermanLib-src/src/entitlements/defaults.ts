// Copyright (C) 2025–2026 Robin L. M. Cheung, MBA
// All rights reserved.
// Unauthorized use without prior written consent is strictly prohibited.

import type { Entitlements } from './types';

export const ANNOTATION_TRIAL_LIMIT = 25;

export const createDefaultEntitlements = (): Entitlements => ({
  issuedAt: new Date().toISOString(),
  capabilities: {
    // Free — always on, unrestricted reader
    'reader.view':    { allowed: true, tier: 'free' },
    'archives.add':   { allowed: true, tier: 'free' },
    'search.view':    { allowed: true, tier: 'free' },
    'bookmarks.view': { allowed: true, tier: 'free' },
    'history.view':   { allowed: true, tier: 'free' },
    'themes.basic':   { allowed: true, tier: 'free' },

    // Annotator — trial 25, then paid
    'annotations.create': {
      allowed: true,
      limit: ANNOTATION_TRIAL_LIMIT,
      tier: 'free',
      quota: { remaining: ANNOTATION_TRIAL_LIMIT },
    },
    'themes.custom': { allowed: false, tier: 'free' },
    'mesh.view':     { allowed: false, tier: 'free' },

    // Creator — paid only
    'assistant.ai':      { allowed: false, tier: 'free' },
    'zimmer.extension':  { allowed: false, tier: 'free' },
    'zim.editor':        { allowed: false, tier: 'free' },
    'annotations.export': { allowed: false, tier: 'free' },
  },
});

export const createAnnotatorEntitlements = (): Entitlements => ({
  issuedAt: new Date().toISOString(),
  capabilities: {
    'reader.view':    { allowed: true, tier: 'annotator' },
    'archives.add':   { allowed: true, tier: 'annotator' },
    'search.view':    { allowed: true, tier: 'annotator' },
    'bookmarks.view': { allowed: true, tier: 'annotator' },
    'history.view':   { allowed: true, tier: 'annotator' },
    'themes.basic':   { allowed: true, tier: 'annotator' },

    'annotations.create': { allowed: true, tier: 'annotator' },
    'themes.custom':      { allowed: true, tier: 'annotator' },
    'mesh.view':          { allowed: true, tier: 'annotator' },

    'assistant.ai':       { allowed: false, tier: 'annotator' },
    'zimmer.extension':   { allowed: false, tier: 'annotator' },
    'zim.editor':         { allowed: false, tier: 'annotator' },
    'annotations.export': { allowed: false, tier: 'annotator' },
  },
});

export const createCreatorEntitlements = (): Entitlements => ({
  issuedAt: new Date().toISOString(),
  capabilities: {
    'reader.view':    { allowed: true, tier: 'creator' },
    'archives.add':   { allowed: true, tier: 'creator' },
    'search.view':    { allowed: true, tier: 'creator' },
    'bookmarks.view': { allowed: true, tier: 'creator' },
    'history.view':   { allowed: true, tier: 'creator' },
    'themes.basic':   { allowed: true, tier: 'creator' },

    'annotations.create': { allowed: true, tier: 'creator' },
    'themes.custom':      { allowed: true, tier: 'creator' },
    'mesh.view':          { allowed: true, tier: 'creator' },

    'assistant.ai':       { allowed: true, tier: 'creator' },
    'zimmer.extension':   { allowed: true, tier: 'creator' },
    'zim.editor':         { allowed: true, tier: 'creator' },
    'annotations.export': { allowed: true, tier: 'creator' },
  },
});
