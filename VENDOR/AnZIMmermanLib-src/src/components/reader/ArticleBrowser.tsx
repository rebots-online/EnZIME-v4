// Copyright (C) 2025–2026 Robin L. M. Cheung, MBA
// All rights reserved.
// Unauthorized use without prior written consent is strictly prohibited.

// EnZIM - Article Browser Component
// Data model designed for graph/hypergraph compatibility (getrecall.ai, etc.)

import { useState, useMemo, useCallback } from 'react';
import { Search, FileText, ChevronLeft, ChevronRight, Hash, ArrowUpDown } from 'lucide-react';
import { zimService, ZIMEntry } from '../../services/zimService';

/** Graph-friendly article node — stable ID, typed relationships, metadata */
export interface ArticleNode {
  /** Stable unique ID: archiveId + ':' + url */
  id: string;
  url: string;
  title: string;
  archiveId: string;
  /** First letter for alphabetical grouping */
  indexLetter: string;
  /** Entry metadata for graph edges (populated lazily) */
  isRedirect: boolean;
  redirectTarget?: string;
}

interface ArticleBrowserProps {
  archiveId: string;
  archiveTitle: string;
  archiveDescription?: string;
  onSelectArticle: (entry: ZIMEntry) => void;
}

const PAGE_SIZE = 100;

type SortMode = 'alpha' | 'url';

/** Convert a ZIMEntry to a graph-friendly ArticleNode */
export function toArticleNode(entry: ZIMEntry, archiveId: string): ArticleNode {
  return {
    id: `${archiveId}:${entry.url}`,
    url: entry.url,
    title: entry.title,
    archiveId,
    indexLetter: (entry.title[0] || '#').toUpperCase(),
    isRedirect: entry.isRedirect,
    redirectTarget: entry.isRedirect ? String(entry.redirectIndex) : undefined,
  };
}

export function ArticleBrowser({
  archiveId,
  archiveTitle,
  archiveDescription,
  onSelectArticle,
}: ArticleBrowserProps) {
  const [searchQuery, setSearchQuery] = useState('');
  const [page, setPage] = useState(0);
  const [sortMode, setSortMode] = useState<SortMode>('alpha');

  // Get all articles once, memoized
  const allArticles = useMemo(() => {
    return zimService.getArticles(archiveId);
  }, [archiveId]);

  // Filter + sort
  const filtered = useMemo(() => {
    let results = allArticles;
    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      results = results.filter(
        a => a.title.toLowerCase().includes(q) || a.url.toLowerCase().includes(q)
      );
    }
    if (sortMode === 'alpha') {
      results = [...results].sort((a, b) => a.title.localeCompare(b.title));
    } else {
      results = [...results].sort((a, b) => a.url.localeCompare(b.url));
    }
    return results;
  }, [allArticles, searchQuery, sortMode]);

  // Pagination
  const totalPages = Math.max(1, Math.ceil(filtered.length / PAGE_SIZE));
  const pageArticles = useMemo(() => {
    const start = page * PAGE_SIZE;
    return filtered.slice(start, start + PAGE_SIZE);
  }, [filtered, page]);

  // Index letters for quick jump
  const indexLetters = useMemo(() => {
    const letters = new Set<string>();
    for (const a of filtered) {
      letters.add((a.title[0] || '#').toUpperCase());
    }
    return Array.from(letters).sort();
  }, [filtered]);

  const handleSearch = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    setSearchQuery(e.target.value);
    setPage(0);
  }, []);

  const jumpToLetter = useCallback((letter: string) => {
    const idx = filtered.findIndex(
      a => (a.title[0] || '#').toUpperCase() === letter
    );
    if (idx >= 0) {
      setPage(Math.floor(idx / PAGE_SIZE));
    }
  }, [filtered]);

  const toggleSort = useCallback(() => {
    setSortMode(prev => prev === 'alpha' ? 'url' : 'alpha');
    setPage(0);
  }, []);

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <div className="text-center px-6 pt-6 pb-2">
        <h2 className="text-xl font-semibold">{archiveTitle}</h2>
        {archiveDescription && (
          <p className="text-sm text-secondary mt-1 max-w-lg mx-auto">{archiveDescription}</p>
        )}
        <p className="text-xs text-secondary mt-2">
          {allArticles.length.toLocaleString()} articles
        </p>
      </div>

      {/* Search + Sort */}
      <div className="px-4 py-3 flex items-center gap-2">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-secondary" />
          <input
            type="text"
            placeholder="Search articles..."
            value={searchQuery}
            onChange={handleSearch}
            className="w-full pl-10 pr-4 py-2 rounded-lg border border-[var(--border)] bg-[var(--bg-primary)] text-sm focus:outline-none focus:border-[var(--accent)]"
          />
        </div>
        <button
          onClick={toggleSort}
          className="p-2 rounded-lg hover:bg-black/5 dark:hover:bg-white/5 transition-colors"
          title={sortMode === 'alpha' ? 'Sort by title' : 'Sort by URL'}
        >
          <ArrowUpDown className="w-4 h-4" />
        </button>
      </div>

      {/* Alphabet index strip */}
      {!searchQuery && indexLetters.length > 1 && (
        <div className="px-4 pb-2 flex flex-wrap gap-1">
          {indexLetters.map(letter => (
            <button
              key={letter}
              onClick={() => jumpToLetter(letter)}
              className="w-7 h-7 text-xs rounded hover:bg-[var(--accent)]/10 hover:text-[var(--accent)] transition-colors"
            >
              {letter}
            </button>
          ))}
        </div>
      )}

      {/* Article list */}
      <div className="flex-1 overflow-auto px-4">
        {pageArticles.length === 0 ? (
          <div className="text-center py-12">
            <Hash className="w-8 h-8 mx-auto mb-3 text-secondary" />
            <p className="text-secondary">
              {searchQuery ? 'No articles match your search' : 'No articles found'}
            </p>
          </div>
        ) : (
          <ul className="divide-y divide-[var(--border)]">
            {pageArticles.map((entry) => (
              <li key={entry.url}>
                <button
                  onClick={() => onSelectArticle(entry)}
                  className="w-full flex items-center gap-3 px-3 py-2.5 text-left hover:bg-[var(--accent)]/5 rounded-lg transition-colors group"
                >
                  <FileText className="w-4 h-4 text-secondary group-hover:text-[var(--accent)] shrink-0" />
                  <div className="min-w-0 flex-1">
                    <p className="text-sm font-medium truncate group-hover:text-[var(--accent)]">
                      {entry.title}
                    </p>
                    <p className="text-xs text-secondary truncate">{entry.url}</p>
                  </div>
                </button>
              </li>
            ))}
          </ul>
        )}
      </div>

      {/* Pagination */}
      {totalPages > 1 && (
        <div className="px-4 py-3 border-t border-[var(--border)] flex items-center justify-between">
          <button
            onClick={() => setPage(p => Math.max(0, p - 1))}
            disabled={page === 0}
            className="flex items-center gap-1 text-sm px-3 py-1.5 rounded-lg hover:bg-black/5 dark:hover:bg-white/5 disabled:opacity-30 transition-colors"
          >
            <ChevronLeft className="w-4 h-4" />
            Prev
          </button>
          <span className="text-sm text-secondary">
            Page {page + 1} of {totalPages}
            {searchQuery && ` (${filtered.length.toLocaleString()} results)`}
          </span>
          <button
            onClick={() => setPage(p => Math.min(totalPages - 1, p + 1))}
            disabled={page >= totalPages - 1}
            className="flex items-center gap-1 text-sm px-3 py-1.5 rounded-lg hover:bg-black/5 dark:hover:bg-white/5 disabled:opacity-30 transition-colors"
          >
            Next
            <ChevronRight className="w-4 h-4" />
          </button>
        </div>
      )}
    </div>
  );
}
