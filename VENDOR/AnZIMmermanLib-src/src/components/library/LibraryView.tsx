// Copyright (C) 2025–2026 Robin L. M. Cheung, MBA
// All rights reserved.
// Unauthorized use without prior written consent is strictly prohibited.

// EnZIM - Offline ZIM Reader & Knowledge Explorer
// Copyright (C) 2025 Robin L. M. Cheung, MBA. All rights reserved.

import { useState, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import { Grid, List, Plus, Download, Trash2, X } from 'lucide-react';
import { useArchiveStore } from '../../stores/archiveStore';
import { zimService } from '../../services/zimService';
import { ArchiveCard } from './ArchiveCard';
import { DropZone } from './DropZone';
import { CatalogBrowser } from './CatalogBrowser';

type ViewMode = 'grid' | 'list';

export function LibraryView() {
  const navigate = useNavigate();
  const [viewMode, setViewMode] = useState<ViewMode>('grid');
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [catalogOpen, setCatalogOpen] = useState(false);
  const { archives, addArchive, removeArchive, setCurrentArchive } = useArchiveStore();

  const toggleSelect = useCallback((id: string) => {
    setSelectedIds(prev => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }, []);

  const clearSelection = useCallback(() => setSelectedIds(new Set()), []);

  const handleRemoveSelected = useCallback(() => {
    if (selectedIds.size === 0) return;
    for (const id of selectedIds) {
      removeArchive(id);
    }
    setSelectedIds(new Set());
  }, [selectedIds, removeArchive]);

  const handleOpenArchive = (archive: typeof archives[0]) => {
    setCurrentArchive(archive);
    navigate(`/reader/${archive.id}`);
  };

  const handleAddArchive = useCallback(async () => {
    try {
      const source = await zimService.openFileDialog();
      if (source) {
        const archiveInfo = await zimService.loadArchive(source);
        if (archiveInfo) {
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
      }
    } catch (error) {
      console.error('Failed to add archive:', error);
    }
  }, [addArchive]);

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-[var(--border)]">
        <div>
          <h1 className="text-xl font-semibold">Library</h1>
          <p className="text-sm text-secondary">
            {archives.length} {archives.length === 1 ? 'archive' : 'archives'}
          </p>
        </div>
        
        <div className="flex items-center gap-2">
          {/* View Toggle */}
          <div className="flex surface rounded-lg p-1">
            <button
              onClick={() => setViewMode('grid')}
              className={`p-2 rounded transition-colors ${
                viewMode === 'grid' 
                  ? 'bg-[var(--accent)] text-white' 
                  : 'hover:bg-black/5 dark:hover:bg-white/5'
              }`}
              aria-label="Grid view"
            >
              <Grid className="w-4 h-4" />
            </button>
            <button
              onClick={() => setViewMode('list')}
              className={`p-2 rounded transition-colors ${
                viewMode === 'list' 
                  ? 'bg-[var(--accent)] text-white' 
                  : 'hover:bg-black/5 dark:hover:bg-white/5'
              }`}
              aria-label="List view"
            >
              <List className="w-4 h-4" />
            </button>
          </div>

          {/* Download Archives Button */}
          <button
            onClick={() => setCatalogOpen(true)}
            className="surface flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium hover:border-[var(--accent)] transition-colors"
          >
            <Download className="w-4 h-4" />
            Download Archives
          </button>

          {/* Add Archive Button */}
          <button onClick={handleAddArchive} className="btn-primary flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium">
            <Plus className="w-4 h-4" />
            Add Archive
          </button>
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-auto p-4">
        {archives.length === 0 ? (
          <DropZone onDownload={() => setCatalogOpen(true)} />
        ) : (
          <>
            {/* Archive Grid/List */}
            <div className={
              viewMode === 'grid'
                ? 'grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4'
                : 'flex flex-col gap-2'
            }>
              {archives.map((archive) => (
                <ArchiveCard
                  key={archive.id}
                  archive={archive}
                  viewMode={viewMode}
                  onOpen={() => handleOpenArchive(archive)}
                  selected={selectedIds.has(archive.id)}
                  onToggleSelect={() => toggleSelect(archive.id)}
                />
              ))}
            </div>

            {/* Drop zone at bottom */}
            <div className="mt-6">
              <DropZone compact onDownload={() => setCatalogOpen(true)} />
            </div>
          </>
        )}
      </div>

      {/* Selection action bar — bottom-right, only when items selected */}
      {selectedIds.size > 0 && (
        <div className="border-t border-[var(--border)] px-4 py-2 flex items-center justify-between bg-[var(--bg-surface)]">
          <span className="text-sm text-secondary">
            {selectedIds.size} {selectedIds.size === 1 ? 'archive' : 'archives'} selected
          </span>
          <div className="flex items-center gap-2">
            <button
              onClick={clearSelection}
              className="flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-lg hover:bg-black/5 dark:hover:bg-white/5 transition-colors"
            >
              <X className="w-4 h-4" />
              Clear
            </button>
            <button
              onClick={handleRemoveSelected}
              className="flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-lg text-red-500 hover:bg-red-500/10 transition-colors"
            >
              <Trash2 className="w-4 h-4" />
              Remove Selected
            </button>
          </div>
        </div>
      )}

      {/* Catalog Browser Dialog */}
      <CatalogBrowser open={catalogOpen} onClose={() => setCatalogOpen(false)} />
    </div>
  );
}
