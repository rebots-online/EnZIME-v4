// Copyright (C) 2025–2026 Robin L. M. Cheung, MBA
// All rights reserved.
// Unauthorized use without prior written consent is strictly prohibited.

// EnZIM - Offline ZIM Reader & Knowledge Explorer
// In-app Kiwix catalog browser — browse, search, and download ZIMs

import { useState, useEffect, useCallback } from 'react';
import { Search, Download, X, Loader2, ChevronLeft, ChevronRight, Globe, Tag } from 'lucide-react';
import { catalogService, type CatalogEntry } from '../../services/catalogService';
import { downloadService } from '../../services/downloadService';
import { useDownloadStore } from '../../stores/downloadStore';

interface CatalogBrowserProps {
  open: boolean;
  onClose: () => void;
}

function formatSize(bytes: number): string {
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let size = bytes;
  let unitIndex = 0;
  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex++;
  }
  return `${size.toFixed(1)} ${units[unitIndex]}`;
}

function formatNumber(num: number): string {
  if (num >= 1000000) return `${(num / 1000000).toFixed(1)}M`;
  if (num >= 1000) return `${(num / 1000).toFixed(1)}K`;
  return num.toString();
}

export function CatalogBrowser({ open, onClose }: CatalogBrowserProps) {
  const [entries, setEntries] = useState<CatalogEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [query, setQuery] = useState('');
  const [language, setLanguage] = useState('');
  const [category, setCategory] = useState('');
  const [page, setPage] = useState(0);
  const [totalResults, setTotalResults] = useState(0);
  const downloads = useDownloadStore((s) => s.downloads);
  const PAGE_SIZE = 24;

  const doSearch = useCallback(async (pageNum: number = 0) => {
    setLoading(true);
    setError(null);
    try {
      const result = await catalogService.search(query, {
        language: language || undefined,
        category: category || undefined,
        count: PAGE_SIZE,
        start: pageNum * PAGE_SIZE,
      });
      setEntries(result.entries);
      setTotalResults(result.totalResults);
      setPage(pageNum);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load catalog');
    } finally {
      setLoading(false);
    }
  }, [query, language, category]);

  useEffect(() => {
    if (open) {
      doSearch(0);
    }
  }, [open]);

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault();
    doSearch(0);
  };

  const handleDownload = (entry: CatalogEntry) => {
    downloadService.startDownload(entry);
  };

  const isDownloading = (entryId: string) => {
    return downloads.some(d => d.entry.id === entryId && (d.status === 'downloading' || d.status === 'queued'));
  };

  const isCompleted = (entryId: string) => {
    return downloads.some(d => d.entry.id === entryId && d.status === 'completed');
  };

  const getProgress = (entryId: string) => {
    const dl = downloads.find(d => d.entry.id === entryId);
    return dl?.progress ?? 0;
  };

  if (!open) return null;

  const totalPages = Math.ceil(totalResults / PAGE_SIZE);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="bg-[var(--bg-primary)] rounded-2xl shadow-2xl border border-[var(--border)] w-[90vw] max-w-6xl h-[85vh] flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-[var(--border)]">
          <div>
            <h2 className="text-xl font-semibold">Download Archives</h2>
            <p className="text-sm text-secondary">Browse the Kiwix library catalog</p>
          </div>
          <button onClick={onClose} className="p-2 rounded-lg hover:bg-black/5 dark:hover:bg-white/5">
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Search Bar */}
        <form onSubmit={handleSearch} className="px-6 py-3 border-b border-[var(--border)] flex gap-3">
          <div className="flex-1 relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-secondary" />
            <input
              type="text"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Search archives..."
              className="w-full pl-10 pr-4 py-2 rounded-lg bg-[var(--bg-surface)] border border-[var(--border)] text-sm focus:outline-none focus:border-[var(--accent)]"
            />
          </div>

          <div className="flex items-center gap-1.5">
            <Globe className="w-4 h-4 text-secondary" />
            <select
              value={language}
              onChange={(e) => setLanguage(e.target.value)}
              className="py-2 px-2 rounded-lg bg-[var(--bg-surface)] border border-[var(--border)] text-sm"
            >
              <option value="">All languages</option>
              <option value="eng">English</option>
              <option value="fra">French</option>
              <option value="deu">German</option>
              <option value="spa">Spanish</option>
              <option value="por">Portuguese</option>
              <option value="ita">Italian</option>
              <option value="rus">Russian</option>
              <option value="zho">Chinese</option>
              <option value="jpn">Japanese</option>
              <option value="ara">Arabic</option>
              <option value="mul">Multilingual</option>
            </select>
          </div>

          <div className="flex items-center gap-1.5">
            <Tag className="w-4 h-4 text-secondary" />
            <select
              value={category}
              onChange={(e) => setCategory(e.target.value)}
              className="py-2 px-2 rounded-lg bg-[var(--bg-surface)] border border-[var(--border)] text-sm"
            >
              <option value="">All categories</option>
              <option value="wikipedia">Wikipedia</option>
              <option value="wiktionary">Wiktionary</option>
              <option value="wikibooks">Wikibooks</option>
              <option value="wikiversity">Wikiversity</option>
              <option value="ted">TED</option>
              <option value="gutenberg">Project Gutenberg</option>
              <option value="stackexchange">Stack Exchange</option>
              <option value="freecodecamp">freeCodeCamp</option>
            </select>
          </div>

          <button type="submit" className="btn-primary px-4 py-2 rounded-lg text-sm font-medium">
            Search
          </button>
        </form>

        {/* Results */}
        <div className="flex-1 overflow-auto p-6">
          {loading ? (
            <div className="flex items-center justify-center h-full">
              <Loader2 className="w-8 h-8 animate-spin text-[var(--accent)]" />
            </div>
          ) : error ? (
            <div className="flex items-center justify-center h-full text-red-500">
              <p>{error}</p>
            </div>
          ) : entries.length === 0 ? (
            <div className="flex items-center justify-center h-full text-secondary">
              <p>No archives found. Try a different search.</p>
            </div>
          ) : (
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
              {entries.map((entry) => (
                <div
                  key={entry.id}
                  className="surface rounded-xl overflow-hidden border border-[var(--border)] flex flex-col"
                >
                  {/* Card Header */}
                  <div className="h-20 bg-gradient-to-br from-cyan-400/20 to-blue-600/20 flex items-center justify-center">
                    {entry.favicon ? (
                      <img src={entry.favicon} alt="" className="w-10 h-10 rounded" />
                    ) : (
                      <Globe className="w-8 h-8 text-[var(--accent)]" />
                    )}
                  </div>

                  {/* Card Body */}
                  <div className="p-3 flex-1 flex flex-col">
                    <h3 className="font-medium text-sm truncate mb-1">{entry.title}</h3>
                    <p className="text-xs text-secondary line-clamp-2 mb-2 flex-1">{entry.description}</p>

                    <div className="flex items-center justify-between text-xs text-secondary mb-3">
                      <span>{formatNumber(entry.articleCount)} articles</span>
                      <span>{formatSize(entry.size)}</span>
                      <span className="uppercase font-medium">{entry.language}</span>
                    </div>

                    {/* Download Button */}
                    {isCompleted(entry.id) ? (
                      <div className="text-center text-xs text-green-500 font-medium py-1.5">
                        ✓ Downloaded
                      </div>
                    ) : isDownloading(entry.id) ? (
                      <div className="space-y-1">
                        <div className="w-full bg-[var(--border)] rounded-full h-1.5">
                          <div
                            className="bg-[var(--accent)] h-1.5 rounded-full transition-all"
                            style={{ width: `${getProgress(entry.id)}%` }}
                          />
                        </div>
                        <p className="text-xs text-center text-secondary">
                          {getProgress(entry.id).toFixed(0)}%
                        </p>
                      </div>
                    ) : (
                      <button
                        onClick={() => handleDownload(entry)}
                        className="w-full flex items-center justify-center gap-1.5 py-1.5 rounded-lg bg-[var(--accent)] text-white text-xs font-medium hover:opacity-90 transition-opacity"
                      >
                        <Download className="w-3.5 h-3.5" />
                        Download · {formatSize(entry.size)}
                      </button>
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Pagination */}
        {totalPages > 1 && (
          <div className="flex items-center justify-center gap-4 px-6 py-3 border-t border-[var(--border)]">
            <button
              onClick={() => doSearch(page - 1)}
              disabled={page === 0}
              className="p-2 rounded-lg hover:bg-black/5 dark:hover:bg-white/5 disabled:opacity-30"
            >
              <ChevronLeft className="w-4 h-4" />
            </button>
            <span className="text-sm text-secondary">
              Page {page + 1} of {totalPages} · {totalResults} archives
            </span>
            <button
              onClick={() => doSearch(page + 1)}
              disabled={page >= totalPages - 1}
              className="p-2 rounded-lg hover:bg-black/5 dark:hover:bg-white/5 disabled:opacity-30"
            >
              <ChevronRight className="w-4 h-4" />
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
