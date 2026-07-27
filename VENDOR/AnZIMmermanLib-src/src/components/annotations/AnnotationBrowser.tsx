// Copyright (C) 2025–2026 Robin L. M. Cheung, MBA
// All rights reserved.
// Unauthorized use without prior written consent is strictly prohibited.

// EnZIM - Annotation Browser: per-archive annotation navigator (USP surface)

import { useState, useMemo, useCallback } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import {
  StickyNote, Mic, Pencil, Camera, Video, Highlighter, Monitor,
  ChevronDown, ChevronRight, Search, X, Trash2, Download, Filter,
  ArrowUpDown, BookOpen, Clock,
} from 'lucide-react';
import { useAnnotationStore } from '../../stores/annotationStore';
import { useArchiveStore } from '../../stores/archiveStore';
import type { Annotation, AnnotationType, AnnotatedArticleSummary } from '../../types/annotation';

const TYPE_ICONS: Record<AnnotationType, typeof StickyNote> = {
  text: StickyNote,
  voice: Mic,
  ink: Pencil,
  photo: Camera,
  video: Video,
  highlight: Highlighter,
  'screen-clip': Monitor,
};

const TYPE_LABELS: Record<AnnotationType, string> = {
  text: 'Text',
  voice: 'Voice',
  ink: 'Ink',
  photo: 'Photo',
  video: 'Video',
  highlight: 'Highlight',
  'screen-clip': 'Screen Clip',
};

type SortMode = 'date' | 'article' | 'count';

interface AnnotationBrowserProps {
  archiveId?: string;
}

function formatRelativeTime(iso: string): string {
  const diff = Date.now() - new Date(iso).getTime();
  const mins = Math.floor(diff / 60000);
  if (mins < 1) return 'just now';
  if (mins < 60) return `${mins}m ago`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  if (days < 30) return `${days}d ago`;
  return new Date(iso).toLocaleDateString();
}

function AnnotationCard({ annotation, onNavigate, onDelete }: {
  annotation: Annotation;
  onNavigate: () => void;
  onDelete: () => void;
}) {
  const Icon = TYPE_ICONS[annotation.type];

  return (
    <div
      className="flex items-start gap-3 p-3 rounded-lg hover:bg-black/5 dark:hover:bg-white/5 transition-colors cursor-pointer group"
      onClick={onNavigate}
    >
      <div className={`w-8 h-8 rounded-lg flex items-center justify-center flex-shrink-0 ${
        annotation.color
          ? `bg-${annotation.color}-400/20`
          : 'bg-[var(--accent)]/10'
      }`}>
        <Icon className="w-4 h-4 text-[var(--accent)]" />
      </div>

      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2 mb-1">
          <span className="text-xs font-medium text-secondary uppercase">
            {TYPE_LABELS[annotation.type]}
          </span>
          <span className="text-xs text-secondary">
            {formatRelativeTime(annotation.modifiedAt)}
          </span>
        </div>

        {/* Content preview */}
        {(annotation.type === 'text' || annotation.type === 'highlight') && (
          <p className="text-sm line-clamp-2">{annotation.content}</p>
        )}
        {annotation.type === 'voice' && (
          <p className="text-sm text-secondary italic">
            {annotation.transcription
              ? <span className="not-italic text-[var(--text-primary)]">{annotation.transcription.slice(0, 100)}...</span>
              : `Voice note${annotation.duration ? ` (${Math.round(annotation.duration)}s)` : ''}`}
          </p>
        )}
        {(annotation.type === 'ink' || annotation.type === 'photo' || annotation.type === 'screen-clip') && annotation.thumbnail && (
          <img src={annotation.thumbnail || annotation.content} alt="" className="w-20 h-14 object-cover rounded mt-1" />
        )}
        {annotation.type === 'video' && (
          <p className="text-sm text-secondary italic">
            Video{annotation.duration ? ` (${Math.round(annotation.duration)}s)` : ''}
          </p>
        )}

        {/* Tags */}
        {annotation.tags && annotation.tags.length > 0 && (
          <div className="flex flex-wrap gap-1 mt-1.5">
            {annotation.tags.slice(0, 5).map((tag) => (
              <span key={tag} className="text-xs px-1.5 py-0.5 bg-[var(--accent)]/10 text-[var(--accent)] rounded">
                {tag}
              </span>
            ))}
          </div>
        )}
      </div>

      {/* Delete button */}
      <button
        onClick={(e) => { e.stopPropagation(); onDelete(); }}
        className="p-1 rounded hover:bg-red-500/10 opacity-0 group-hover:opacity-100 transition-opacity flex-shrink-0"
        title="Delete annotation"
      >
        <Trash2 className="w-3.5 h-3.5 text-red-500" />
      </button>
    </div>
  );
}

export function AnnotationBrowser({ archiveId: propArchiveId }: AnnotationBrowserProps) {
  const { archiveId: routeArchiveId } = useParams<{ archiveId?: string }>();
  const archiveId = propArchiveId || routeArchiveId;
  const navigate = useNavigate();
  const { archives, setCurrentArchive, setCurrentArticle } = useArchiveStore();
  const {
    getAnnotationsForArchive,
    getAnnotatedArticlesForArchive,
    getAnnotationCountForArchive,
    removeAnnotation,
    searchAnnotations,
  } = useAnnotationStore();

  const [searchQuery, setSearchQuery] = useState('');
  const [typeFilter, setTypeFilter] = useState<AnnotationType | null>(null);
  const [sortMode, setSortMode] = useState<SortMode>('date');
  const [expandedArticles, setExpandedArticles] = useState<Set<string>>(new Set());

  // Determine which archive(s) to show
  const targetArchives = archiveId
    ? archives.filter((a) => a.id === archiveId)
    : archives;

  const selectedArchive = archiveId
    ? archives.find((a) => a.id === archiveId)
    : null;

  // Get annotation data
  const allAnnotations = useMemo(() => {
    if (searchQuery) {
      const results = searchAnnotations(searchQuery);
      return archiveId ? results.filter((a) => a.archiveId === archiveId) : results;
    }
    if (archiveId) {
      return getAnnotationsForArchive(archiveId);
    }
    return useAnnotationStore.getState().annotations;
  }, [archiveId, searchQuery, getAnnotationsForArchive, searchAnnotations]);

  const filteredAnnotations = useMemo(() => {
    if (!typeFilter) return allAnnotations;
    return allAnnotations.filter((a) => a.type === typeFilter);
  }, [allAnnotations, typeFilter]);

  const articleSummaries = useMemo(() => {
    // Group filtered annotations by article
    const grouped = new Map<string, Annotation[]>();
    for (const a of filteredAnnotations) {
      const key = `${a.archiveId}::${a.articleUrl}`;
      const existing = grouped.get(key) || [];
      existing.push(a);
      grouped.set(key, existing);
    }

    const summaries: (AnnotatedArticleSummary & { archiveId: string; annotations: Annotation[] })[] = [];
    for (const [key, annotations] of grouped) {
      const [aid] = key.split('::');
      const types = [...new Set(annotations.map((a) => a.type))];
      const latest = annotations.reduce(
        (max, a) => (a.modifiedAt > max ? a.modifiedAt : max),
        annotations[0].modifiedAt,
      );
      summaries.push({
        articleUrl: annotations[0].articleUrl,
        archiveId: aid,
        count: annotations.length,
        types,
        latestTimestamp: latest,
        annotations: annotations.sort((a, b) => b.modifiedAt.localeCompare(a.modifiedAt)),
      });
    }

    if (sortMode === 'date') {
      summaries.sort((a, b) => b.latestTimestamp.localeCompare(a.latestTimestamp));
    } else if (sortMode === 'count') {
      summaries.sort((a, b) => b.count - a.count);
    } else {
      summaries.sort((a, b) => a.articleUrl.localeCompare(b.articleUrl));
    }

    return summaries;
  }, [filteredAnnotations, sortMode]);

  const toggleArticle = useCallback((key: string) => {
    setExpandedArticles((prev) => {
      const next = new Set(prev);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return next;
    });
  }, []);

  const handleNavigateToAnnotation = useCallback(
    (annotation: Annotation) => {
      const archive = archives.find((a) => a.id === annotation.archiveId);
      if (archive) {
        setCurrentArchive(archive);
        setCurrentArticle({
          url: annotation.articleUrl,
          title: annotation.articleUrl,
          archiveId: annotation.archiveId,
        });
        navigate(`/reader/${annotation.archiveId}`);
      }
    },
    [archives, setCurrentArchive, setCurrentArticle, navigate],
  );

  const totalCount = archiveId
    ? getAnnotationCountForArchive(archiveId)
    : useAnnotationStore.getState().annotations.length;

  const allTypes: AnnotationType[] = ['text', 'voice', 'ink', 'photo', 'video', 'highlight', 'screen-clip'];

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="p-4 border-b border-[var(--border)]">
        <div className="flex items-center justify-between mb-3">
          <div>
            <h1 className="text-xl font-semibold flex items-center gap-2">
              <StickyNote className="w-5 h-5 text-[var(--accent)]" />
              Annotations
            </h1>
            <p className="text-sm text-secondary">
              {selectedArchive
                ? `${selectedArchive.title || selectedArchive.name} — ${totalCount} annotation${totalCount !== 1 ? 's' : ''}`
                : `${totalCount} annotation${totalCount !== 1 ? 's' : ''} across ${targetArchives.length} archive${targetArchives.length !== 1 ? 's' : ''}`}
            </p>
          </div>
        </div>

        {/* Search bar */}
        <div className="relative mb-3">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-secondary" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search annotations..."
            className="w-full pl-9 pr-8 py-2 rounded-lg border border-[var(--border)] bg-[var(--bg-surface)] focus:outline-none focus:border-[var(--accent)] transition-colors text-sm"
          />
          {searchQuery && (
            <button
              onClick={() => setSearchQuery('')}
              className="absolute right-2 top-1/2 -translate-y-1/2 p-1 hover:bg-black/5 dark:hover:bg-white/5 rounded"
            >
              <X className="w-3.5 h-3.5 text-secondary" />
            </button>
          )}
        </div>

        {/* Filters + Sort */}
        <div className="flex items-center gap-2 flex-wrap">
          {/* Type filter chips */}
          <button
            onClick={() => setTypeFilter(null)}
            className={`px-2.5 py-1 rounded-full text-xs font-medium transition-colors ${
              !typeFilter
                ? 'bg-[var(--accent)] text-white'
                : 'bg-[var(--bg-surface)] border border-[var(--border)] hover:border-[var(--accent)]'
            }`}
          >
            All
          </button>
          {allTypes.map((t) => {
            const Icon = TYPE_ICONS[t];
            return (
              <button
                key={t}
                onClick={() => setTypeFilter(typeFilter === t ? null : t)}
                className={`flex items-center gap-1 px-2.5 py-1 rounded-full text-xs font-medium transition-colors ${
                  typeFilter === t
                    ? 'bg-[var(--accent)] text-white'
                    : 'bg-[var(--bg-surface)] border border-[var(--border)] hover:border-[var(--accent)]'
                }`}
              >
                <Icon className="w-3 h-3" />
                {TYPE_LABELS[t]}
              </button>
            );
          })}

          {/* Sort */}
          <div className="ml-auto flex items-center gap-1">
            <ArrowUpDown className="w-3.5 h-3.5 text-secondary" />
            <select
              value={sortMode}
              onChange={(e) => setSortMode(e.target.value as SortMode)}
              className="text-xs bg-transparent border-none focus:outline-none text-secondary cursor-pointer"
            >
              <option value="date">Latest</option>
              <option value="count">Most annotated</option>
              <option value="article">Article name</option>
            </select>
          </div>
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-auto p-4">
        {articleSummaries.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-center">
            <StickyNote className="w-12 h-12 text-secondary mb-3" />
            <p className="text-secondary">
              {searchQuery
                ? `No annotations matching "${searchQuery}"`
                : 'No annotations yet'}
            </p>
            <p className="text-xs text-secondary mt-1">
              Open an article and start annotating to see them here.
            </p>
          </div>
        ) : (
          <div className="space-y-2">
            {articleSummaries.map((summary) => {
              const key = `${summary.archiveId}::${summary.articleUrl}`;
              const isExpanded = expandedArticles.has(key);
              const archive = archives.find((a) => a.id === summary.archiveId);

              return (
                <div key={key} className="surface rounded-lg overflow-hidden">
                  {/* Article header */}
                  <button
                    onClick={() => toggleArticle(key)}
                    className="w-full flex items-center gap-3 p-3 hover:bg-black/5 dark:hover:bg-white/5 transition-colors text-left"
                  >
                    {isExpanded ? (
                      <ChevronDown className="w-4 h-4 text-secondary flex-shrink-0" />
                    ) : (
                      <ChevronRight className="w-4 h-4 text-secondary flex-shrink-0" />
                    )}
                    <BookOpen className="w-4 h-4 text-[var(--accent)] flex-shrink-0" />
                    <div className="flex-1 min-w-0">
                      <p className="text-sm font-medium truncate">
                        {summary.articleUrl.split('/').pop() || summary.articleUrl}
                      </p>
                      {!archiveId && archive && (
                        <p className="text-xs text-secondary truncate">
                          {archive.title || archive.name}
                        </p>
                      )}
                    </div>

                    {/* Type icons */}
                    <div className="flex items-center gap-1 flex-shrink-0">
                      {summary.types.map((t) => {
                        const Icon = TYPE_ICONS[t];
                        return <Icon key={t} className="w-3.5 h-3.5 text-secondary" />;
                      })}
                    </div>

                    {/* Count badge */}
                    <span className="text-xs text-secondary bg-[var(--accent)]/10 px-2 py-0.5 rounded-full flex-shrink-0">
                      {summary.count}
                    </span>

                    {/* Time */}
                    <span className="text-xs text-secondary flex-shrink-0 hidden sm:block">
                      {formatRelativeTime(summary.latestTimestamp)}
                    </span>
                  </button>

                  {/* Expanded: annotation cards */}
                  {isExpanded && (
                    <div className="border-t border-[var(--border)] divide-y divide-[var(--border)]">
                      {summary.annotations.map((annotation) => (
                        <AnnotationCard
                          key={annotation.id}
                          annotation={annotation}
                          onNavigate={() => handleNavigateToAnnotation(annotation)}
                          onDelete={() => removeAnnotation(annotation.id)}
                        />
                      ))}
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}
