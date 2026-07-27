// Copyright (C) 2025–2026 Robin L. M. Cheung, MBA
// All rights reserved.
// Unauthorized use without prior written consent is strictly prohibited.

// EnZIM - Offline ZIM Reader & Knowledge Explorer
// Copyright (C) 2025 Robin L. M. Cheung, MBA. All rights reserved.

import { useState, useCallback } from 'react';
import { NavLink } from 'react-router-dom';
import { Library, BookOpen, History, Bookmark, Settings, FolderOpen, Search, Upload, StickyNote } from 'lucide-react';
import { zimService } from '../../services/zimService';
import { useArchiveStore } from '../../stores/archiveStore';
import { useAnnotationStore } from '../../stores/annotationStore';

interface NavItemProps {
  to: string;
  icon: React.ReactNode;
  label: string;
}

function NavItem({ to, icon, label }: NavItemProps) {
  return (
    <NavLink
      to={to}
      className={({ isActive }) =>
        `w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm transition-colors ${
          isActive 
            ? 'bg-[var(--accent)] text-white' 
            : 'hover:bg-black/5 dark:hover:bg-white/5'
        }`
      }
    >
      {icon}
      <span>{label}</span>
    </NavLink>
  );
}

export function Sidebar() {
  const [isDragging, setIsDragging] = useState(false);
  const { addArchive, archives } = useArchiveStore();
  const { getAnnotationCountForArchive } = useAnnotationStore();

  const loadSource = useCallback(async (source: string | File) => {
    try {
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
    } catch (error) {
      console.error('Failed to load archive:', error);
    }
  }, [addArchive]);

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
    const files = Array.from(e.dataTransfer.files);
    const zimFiles = files.filter(f => f.name.endsWith('.zim'));
    for (const file of zimFiles) {
      loadSource(file);
    }
  }, [loadSource]);

  const handleBrowse = useCallback(async () => {
    try {
      const source = await zimService.openFileDialog();
      if (source) {
        await loadSource(source);
      }
    } catch (error) {
      console.error('Failed to browse for archive:', error);
    }
  }, [loadSource]);

  return (
    <aside className="surface w-64 border-r flex flex-col">
      {/* Navigation */}
      <nav className="p-3 space-y-1">
        <NavItem to="/library" icon={<Library className="w-5 h-5" />} label="Library" />
        <NavItem to="/reader" icon={<BookOpen className="w-5 h-5" />} label="Reader" />
        <NavItem to="/search" icon={<Search className="w-5 h-5" />} label="Search" />
        <NavItem to="/history" icon={<History className="w-5 h-5" />} label="History" />
        <NavItem to="/bookmarks" icon={<Bookmark className="w-5 h-5" />} label="Bookmarks" />
        <NavItem to="/annotations" icon={<StickyNote className="w-5 h-5" />} label="Annotations" />
      </nav>

      {/* Divider */}
      <div className="border-t border-[var(--border)] mx-3 my-2"></div>

      {/* Archives Section */}
      <div className="flex-1 overflow-auto p-3">
        <h3 className="text-xs font-semibold text-secondary uppercase tracking-wider mb-2">
          Archives ({archives.length})
        </h3>
        <div className="space-y-1">
          {archives.map(archive => {
            const annotationCount = getAnnotationCountForArchive(archive.id);
            return (
              <NavLink
                key={archive.id}
                to={`/reader/${archive.id}`}
                className="flex items-center gap-2 p-2 rounded-lg text-sm hover:bg-black/5 dark:hover:bg-white/5 transition-colors"
              >
                {archive.favicon ? (
                  <img src={archive.favicon} alt="" className="w-4 h-4 flex-shrink-0" />
                ) : (
                  <BookOpen className="w-4 h-4 flex-shrink-0 text-[var(--accent)]" />
                )}
                <span className="truncate flex-1">{archive.title || archive.name}</span>
                {annotationCount > 0 && (
                  <span className="flex items-center gap-0.5 text-xs text-secondary bg-[var(--accent)]/10 px-1.5 py-0.5 rounded-full flex-shrink-0">
                    <StickyNote className="w-3 h-3" />
                    {annotationCount}
                  </span>
                )}
              </NavLink>
            );
          })}
          <div
            onDragOver={(e) => { e.preventDefault(); setIsDragging(true); }}
            onDragLeave={(e) => { e.preventDefault(); setIsDragging(false); }}
            onDrop={handleDrop}
            onClick={handleBrowse}
            className={`p-3 surface rounded-lg border border-dashed cursor-pointer text-center transition-colors ${
              isDragging
                ? 'border-[var(--accent)] bg-[var(--accent)]/10'
                : 'border-[var(--border)] hover:border-[var(--accent)]/50'
            }`}
          >
            {isDragging ? (
              <Upload className="w-6 h-6 mx-auto mb-1 text-[var(--accent)]" />
            ) : (
              <FolderOpen className="w-6 h-6 mx-auto mb-1 text-secondary" />
            )}
            <p className="text-xs text-secondary">Drop ZIM files here</p>
            <p className="text-xs text-secondary mt-0.5">or click to browse</p>
          </div>
        </div>
      </div>

      {/* Settings */}
      <div className="p-3 border-t border-[var(--border)]">
        <NavItem to="/settings" icon={<Settings className="w-5 h-5" />} label="Settings" />
      </div>
    </aside>
  );
}
