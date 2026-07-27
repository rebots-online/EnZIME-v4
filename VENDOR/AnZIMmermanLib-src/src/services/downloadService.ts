// Copyright (C) 2025–2026 Robin L. M. Cheung, MBA
// All rights reserved.
// Unauthorized use without prior written consent is strictly prohibited.

// EnZIM - Offline ZIM Reader & Knowledge Explorer
// Download service — downloads ZIM files to app storage with progress tracking

import type { CatalogEntry } from './catalogService';
import { useDownloadStore } from '../stores/downloadStore';
import { zimService } from './zimService';
import { useArchiveStore } from '../stores/archiveStore';

async function getDownloadDir(): Promise<string> {
  try {
    const { appDataDir } = await import('@tauri-apps/api/path');
    const { mkdir, exists } = await import('@tauri-apps/plugin-fs');
    const base = await appDataDir();
    const dlDir = `${base}/downloads`;
    if (!(await exists(dlDir))) {
      await mkdir(dlDir, { recursive: true });
    }
    return dlDir;
  } catch {
    // Browser fallback — can't save to filesystem
    throw new Error('File downloads require the desktop app');
  }
}

function sanitizeFilename(name: string): string {
  return name.replace(/[^a-zA-Z0-9._-]/g, '_').replace(/_+/g, '_');
}

export class DownloadService {
  private abortControllers = new Map<string, AbortController>();

  async startDownload(entry: CatalogEntry): Promise<void> {
    const store = useDownloadStore.getState();

    // Add to queue
    store.addDownload(entry);
    const download = useDownloadStore.getState().downloads.find(d => d.entry.id === entry.id);
    if (!download) return;

    const dlId = download.id;
    store.setActiveDownload(dlId);

    try {
      const downloadDir = await getDownloadDir();
      const filename = sanitizeFilename(`${entry.name}.zim`);
      const filePath = `${downloadDir}/${filename}`;

      store.setFilePath(dlId, filePath);
      store.setStatus(dlId, 'downloading');

      // Use Tauri HTTP plugin for download with progress
      await this.downloadWithTauri(dlId, entry.url, filePath);

      store.setStatus(dlId, 'completed');
      store.setActiveDownload(null);

      // Auto-import the downloaded ZIM into library
      await this.importToLibrary(filePath);

      // Start next queued download
      this.processQueue();
    } catch (error) {
      const msg = error instanceof Error ? error.message : 'Download failed';
      if (msg !== 'Download cancelled') {
        store.setStatus(dlId, 'failed', msg);
      }
      store.setActiveDownload(null);
      this.processQueue();
    }
  }

  cancelDownload(dlId: string): void {
    const controller = this.abortControllers.get(dlId);
    if (controller) {
      controller.abort();
      this.abortControllers.delete(dlId);
    }
    const store = useDownloadStore.getState();
    store.removeDownload(dlId);
    store.setActiveDownload(null);
    this.processQueue();
  }

  private async downloadWithTauri(dlId: string, url: string, filePath: string): Promise<void> {
    const store = useDownloadStore.getState();
    const controller = new AbortController();
    this.abortControllers.set(dlId, controller);

    try {
      const { fetch: tauriFetch } = await import('@tauri-apps/plugin-http');
      const { writeFile } = await import('@tauri-apps/plugin-fs');

      const response = await tauriFetch(url, {
        method: 'GET',
        signal: controller.signal as AbortSignal,
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const contentLength = parseInt(response.headers.get('content-length') ?? '0');
      const reader = response.body?.getReader();
      if (!reader) throw new Error('No response body');

      const chunks: Uint8Array[] = [];
      let received = 0;
      let lastTime = Date.now();
      let lastBytes = 0;

      while (true) {
        const { done, value } = await reader.read();
        if (done) break;

        chunks.push(value);
        received += value.length;

        // Calculate speed and progress
        const now = Date.now();
        const elapsed = (now - lastTime) / 1000;
        if (elapsed >= 0.5) {
          const speed = (received - lastBytes) / elapsed;
          const progress = contentLength > 0 ? (received / contentLength) * 100 : 0;
          store.updateProgress(dlId, progress, received, speed);
          lastTime = now;
          lastBytes = received;
        }
      }

      // Concatenate chunks and write to file
      const totalLength = chunks.reduce((sum, c) => sum + c.length, 0);
      const data = new Uint8Array(totalLength);
      let offset = 0;
      for (const chunk of chunks) {
        data.set(chunk, offset);
        offset += chunk.length;
      }

      await writeFile(filePath, data);
      store.updateProgress(dlId, 100, totalLength, 0);
    } finally {
      this.abortControllers.delete(dlId);
    }
  }

  private async importToLibrary(filePath: string): Promise<void> {
    try {
      const archiveInfo = await zimService.loadArchive(filePath);
      if (archiveInfo) {
        const { addArchive } = useArchiveStore.getState();
        addArchive({
          id: archiveInfo.id,
          name: archiveInfo.name,
          title: archiveInfo.title,
          description: archiveInfo.description,
          language: archiveInfo.language,
          articleCount: archiveInfo.articleCount,
          mediaCount: 0,
          size: archiveInfo.size,
          path: archiveInfo.path,
          favicon: archiveInfo.favicon,
          addedAt: new Date(),
        });
      }
    } catch (error) {
      console.error('Failed to import downloaded ZIM:', error);
    }
  }

  private processQueue(): void {
    const { downloads, activeDownloadId } = useDownloadStore.getState();
    if (activeDownloadId) return;

    const next = downloads.find(d => d.status === 'queued');
    if (next) {
      this.startDownload(next.entry);
    }
  }
}

export const downloadService = new DownloadService();
