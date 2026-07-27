// Copyright (C) 2025–2026 Robin L. M. Cheung, MBA
// All rights reserved.
// Unauthorized use without prior written consent is strictly prohibited.

// EnZIM - Offline ZIM Reader & Knowledge Explorer
// Download state management for in-app ZIM downloads

import { create } from 'zustand';
import type { CatalogEntry } from '../services/catalogService';

export type DownloadStatus = 'queued' | 'downloading' | 'paused' | 'completed' | 'failed';

export interface DownloadItem {
  id: string;
  entry: CatalogEntry;
  status: DownloadStatus;
  progress: number;
  bytesDownloaded: number;
  totalBytes: number;
  speed: number;
  error?: string;
  filePath?: string;
  startedAt?: string;
  completedAt?: string;
}

interface DownloadState {
  downloads: DownloadItem[];
  activeDownloadId: string | null;

  addDownload: (entry: CatalogEntry) => void;
  updateProgress: (id: string, progress: number, bytesDownloaded: number, speed: number) => void;
  setStatus: (id: string, status: DownloadStatus, error?: string) => void;
  setFilePath: (id: string, filePath: string) => void;
  removeDownload: (id: string) => void;
  clearCompleted: () => void;
  setActiveDownload: (id: string | null) => void;
}

export const useDownloadStore = create<DownloadState>((set) => ({
  downloads: [],
  activeDownloadId: null,

  addDownload: (entry) => set((state) => {
    const existing = state.downloads.find(d => d.entry.id === entry.id);
    if (existing && existing.status !== 'failed') return state;

    const item: DownloadItem = {
      id: `dl-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
      entry,
      status: 'queued',
      progress: 0,
      bytesDownloaded: 0,
      totalBytes: entry.size,
      speed: 0,
      startedAt: new Date().toISOString(),
    };

    return {
      downloads: [...state.downloads.filter(d => d.entry.id !== entry.id), item],
    };
  }),

  updateProgress: (id, progress, bytesDownloaded, speed) => set((state) => ({
    downloads: state.downloads.map(d =>
      d.id === id ? { ...d, progress, bytesDownloaded, speed, status: 'downloading' as DownloadStatus } : d
    ),
  })),

  setStatus: (id, status, error) => set((state) => ({
    downloads: state.downloads.map(d =>
      d.id === id ? {
        ...d,
        status,
        error,
        completedAt: status === 'completed' ? new Date().toISOString() : d.completedAt,
      } : d
    ),
  })),

  setFilePath: (id, filePath) => set((state) => ({
    downloads: state.downloads.map(d =>
      d.id === id ? { ...d, filePath } : d
    ),
  })),

  removeDownload: (id) => set((state) => ({
    downloads: state.downloads.filter(d => d.id !== id),
    activeDownloadId: state.activeDownloadId === id ? null : state.activeDownloadId,
  })),

  clearCompleted: () => set((state) => ({
    downloads: state.downloads.filter(d => d.status !== 'completed'),
  })),

  setActiveDownload: (id) => set({ activeDownloadId: id }),
}));
