// Copyright (C) 2025–2026 Robin L. M. Cheung, MBA
// All rights reserved.
// Unauthorized use without prior written consent is strictly prohibited.

// EnZIM - Offline ZIM Reader & Knowledge Explorer
// Copyright (C) 2025 Robin L. M. Cheung, MBA. All rights reserved.

import { useState, useEffect, useCallback, useRef } from 'react';
import { 
  ChevronLeft, 
  ChevronRight, 
  Bookmark, 
  BookmarkCheck,
  Share2, 
  ZoomIn, 
  ZoomOut,
  Maximize,
  PanelRightOpen,
  Home,
  Loader2,
  MapPin,
  Highlighter,
  MessageSquare
} from 'lucide-react';
import { useArchiveStore, Article } from '../../stores/archiveStore';
import { useAnnotationStore } from '../../stores/annotationStore';
import { zimService } from '../../services/zimService';
import { AnnotationPanel } from '../annotations/AnnotationPanel';
import { ArticleBrowser } from './ArticleBrowser';
import { TextHighlighter } from '../annotations/TextHighlighter';
import { CanvasAnnotationLayer } from '../annotations/CanvasAnnotationLayer';
import type { AnnotationType, AnnotationColor, AnnotationPosition } from '../../types/annotation';

interface ReaderViewProps {
  onToggleMesh?: () => void;
  meshOpen?: boolean;
}

export function ReaderView({ onToggleMesh, meshOpen }: ReaderViewProps) {
  const { currentArticle, currentArchive, bookmarks, addBookmark, removeBookmark, setCurrentArticle } = useArchiveStore();
  const { addAnnotation } = useAnnotationStore();
  const [fontSize, setFontSize] = useState(16);
  const historyRef = useRef<Article[]>([]);
  const historyIndexRef = useRef(-1);
  const navigatingRef = useRef(false);
  const [isLoading, setIsLoading] = useState(false);
  const [articleContent, setArticleContent] = useState<string | null>(null);
  const [, forceUpdate] = useState(0);
  
  // Annotation state
  const [showAnnotationPanel, setShowAnnotationPanel] = useState(false);
  const [panelPosition, setPanelPosition] = useState<{ x: number; y: number } | null>(null);
  const [placementMode, setPlacementMode] = useState(false);
  const [selectedPlacementType, setSelectedPlacementType] = useState<AnnotationType>('text');
  const [highlightMode, setHighlightMode] = useState(false);
  const articleContentRef = useRef<HTMLDivElement>(null);

  // Track article navigation in history
  useEffect(() => {
    if (currentArticle && !navigatingRef.current) {
      const hist = historyRef.current;
      const idx = historyIndexRef.current;
      // Trim forward history when navigating to a new article
      historyRef.current = [...hist.slice(0, idx + 1), currentArticle];
      historyIndexRef.current = historyRef.current.length - 1;
      forceUpdate(n => n + 1);
    }
    navigatingRef.current = false;
  }, [currentArticle?.url, currentArticle?.archiveId]);

  useEffect(() => {
    async function loadContent() {
      if (!currentArticle || !currentArchive) {
        setArticleContent(null);
        return;
      }

      if (currentArticle.content) {
        setArticleContent(currentArticle.content);
        return;
      }

      setIsLoading(true);
      try {
        const entry = zimService.getEntryByUrl(currentArchive.id, currentArticle.url);
        if (entry && !entry.isRedirect) {
          const content = await zimService.getArticleContent(currentArchive.id, entry);
          if (content) {
            setArticleContent(content);
            setCurrentArticle({ ...currentArticle, content });
          } else {
            setArticleContent('<p>Failed to load article content.</p>');
          }
        } else {
          setArticleContent('<p>Article not found.</p>');
        }
      } catch (error) {
        console.error('Failed to load article:', error);
        setArticleContent('<p>Error loading article content.</p>');
      } finally {
        setIsLoading(false);
      }
    }

    loadContent();
  }, [currentArticle?.url, currentArchive?.id, setCurrentArticle]);

  const isBookmarked = currentArticle && bookmarks.some(
    b => b.url === currentArticle.url && b.archiveId === currentArticle.archiveId
  );

  const canGoBack = historyIndexRef.current > 0;
  const canGoForward = historyIndexRef.current < historyRef.current.length - 1;

  const handleGoBack = useCallback(() => {
    if (historyIndexRef.current > 0) {
      navigatingRef.current = true;
      historyIndexRef.current -= 1;
      setCurrentArticle(historyRef.current[historyIndexRef.current]);
      forceUpdate(n => n + 1);
    }
  }, [setCurrentArticle]);

  const handleGoForward = useCallback(() => {
    if (historyIndexRef.current < historyRef.current.length - 1) {
      navigatingRef.current = true;
      historyIndexRef.current += 1;
      setCurrentArticle(historyRef.current[historyIndexRef.current]);
      forceUpdate(n => n + 1);
    }
  }, [setCurrentArticle]);

  const handleShare = useCallback(() => {
    const text = `${currentArticle?.title} — ${currentArchive?.title}`;
    if (navigator.share) {
      navigator.share({ title: currentArticle?.title, text }).catch(() => {});
    } else {
      navigator.clipboard.writeText(text).catch(() => {});
    }
  }, [currentArticle, currentArchive]);

  const handleFullscreen = useCallback(() => {
    if (document.fullscreenElement) {
      document.exitFullscreen();
    } else {
      document.documentElement.requestFullscreen();
    }
  }, []);

  const handleBookmarkToggle = () => {
    if (!currentArticle) return;
    if (isBookmarked) {
      removeBookmark(currentArticle.url);
    } else {
      addBookmark(currentArticle);
    }
  };

  const handleZoomIn = () => setFontSize(prev => Math.min(prev + 2, 32));
  const handleZoomOut = () => setFontSize(prev => Math.max(prev - 2, 10));

  // Annotation handlers
  const handleTogglePlacementMode = () => {
    setPlacementMode(prev => !prev);
    setHighlightMode(false);
    if (!placementMode) {
      setShowAnnotationPanel(false);
    }
  };

  const handleToggleHighlightMode = () => {
    setHighlightMode(prev => !prev);
    setPlacementMode(false);
    if (!highlightMode) {
      setShowAnnotationPanel(false);
    }
  };

  const handleCanvasPlacement = (position: { x: number; y: number }) => {
    setPanelPosition(position);
    setShowAnnotationPanel(true);
    setPlacementMode(false);
  };

  const handleHighlight = useCallback((text: string, position: AnnotationPosition, color: AnnotationColor) => {
    if (!currentArticle || !currentArchive) return;
    
    addAnnotation({
      type: 'highlight',
      articleUrl: currentArticle.url,
      archiveId: currentArchive.id,
      content: text,
      position,
      color,
    });
  }, [currentArticle, currentArchive, addAnnotation]);

  const handleCloseAnnotationPanel = () => {
    setShowAnnotationPanel(false);
    setPanelPosition(null);
  };

  if (!currentArchive) {
    return (
      <div className="h-full flex items-center justify-center">
        <div className="text-center">
          <Home className="w-12 h-12 mx-auto mb-4 text-secondary" />
          <h2 className="text-xl font-semibold mb-2">No archive selected</h2>
          <p className="text-secondary">Select an archive from the library to start reading</p>
        </div>
      </div>
    );
  }

  if (!currentArticle) {
    return (
      <ArticleBrowser
        archiveId={currentArchive.id}
        archiveTitle={currentArchive.title || currentArchive.name}
        archiveDescription={currentArchive.description}
        onSelectArticle={(entry) => {
          setCurrentArticle({
            url: entry.url,
            title: entry.title,
            archiveId: currentArchive.id,
          });
        }}
      />
    );
  }

  return (
    <div className="h-full flex flex-col">
      {/* Reader Toolbar */}
      <div className="flex items-center justify-between px-4 py-2 border-b border-[var(--border)] bg-[var(--bg-surface)]">
        <div className="flex items-center gap-2">
          {/* Navigation */}
          <button 
            onClick={handleGoBack}
            disabled={!canGoBack}
            className="p-2 rounded-lg hover:bg-black/5 dark:hover:bg-white/5 disabled:opacity-30 transition-colors"
            aria-label="Go back"
          >
            <ChevronLeft className="w-5 h-5" />
          </button>
          <button 
            onClick={handleGoForward}
            disabled={!canGoForward}
            className="p-2 rounded-lg hover:bg-black/5 dark:hover:bg-white/5 disabled:opacity-30 transition-colors"
            aria-label="Go forward"
          >
            <ChevronRight className="w-5 h-5" />
          </button>

          {/* Breadcrumb */}
          <div className="ml-2 text-sm">
            <span className="text-secondary">{currentArchive.title} / </span>
            <span className="font-medium">{currentArticle.title}</span>
          </div>
        </div>

        <div className="flex items-center gap-1">
          {/* Zoom Controls */}
          <button 
            onClick={handleZoomOut}
            className="p-2 rounded-lg hover:bg-black/5 dark:hover:bg-white/5 transition-colors"
            aria-label="Zoom out"
          >
            <ZoomOut className="w-4 h-4" />
          </button>
          <span className="text-xs text-secondary w-10 text-center">{fontSize}px</span>
          <button 
            onClick={handleZoomIn}
            className="p-2 rounded-lg hover:bg-black/5 dark:hover:bg-white/5 transition-colors"
            aria-label="Zoom in"
          >
            <ZoomIn className="w-4 h-4" />
          </button>

          <div className="w-px h-6 bg-[var(--border)] mx-2" />

          {/* Actions */}
          <button 
            onClick={handleBookmarkToggle}
            className={`p-2 rounded-lg transition-colors ${
              isBookmarked 
                ? 'text-[var(--accent)] bg-[var(--accent)]/10' 
                : 'hover:bg-black/5 dark:hover:bg-white/5'
            }`}
            aria-label={isBookmarked ? 'Remove bookmark' : 'Add bookmark'}
          >
            {isBookmarked ? <BookmarkCheck className="w-4 h-4" /> : <Bookmark className="w-4 h-4" />}
          </button>
          <button 
            onClick={handleShare}
            className="p-2 rounded-lg hover:bg-black/5 dark:hover:bg-white/5 transition-colors"
            aria-label="Share"
          >
            <Share2 className="w-4 h-4" />
          </button>
          <button 
            onClick={handleFullscreen}
            className="p-2 rounded-lg hover:bg-black/5 dark:hover:bg-white/5 transition-colors"
            aria-label="Fullscreen"
          >
            <Maximize className="w-4 h-4" />
          </button>
          
          <div className="w-px h-6 bg-[var(--border)] mx-2" />

          {/* Annotation Actions */}
          <button 
            onClick={handleTogglePlacementMode}
            className={`p-2 rounded-lg transition-colors ${
              placementMode 
                ? 'text-[var(--accent)] bg-[var(--accent)]/10' 
                : 'hover:bg-black/5 dark:hover:bg-white/5'
            }`}
            aria-label={placementMode ? 'Exit placement mode' : 'Enter placement mode'}
            title="Place annotation pin"
          >
            <MapPin className="w-4 h-4" />
          </button>
          
          {/* Placement Type Selector - shown when in placement mode */}
          {placementMode && (
            <select
              value={selectedPlacementType}
              onChange={(e) => setSelectedPlacementType(e.target.value as AnnotationType)}
              className="text-xs bg-[var(--bg-primary)] border border-[var(--border)] rounded px-2 py-1"
            >
              <option value="text">Text</option>
              <option value="voice">Voice</option>
              <option value="ink">Ink</option>
              <option value="photo">Photo</option>
              <option value="video">Video</option>
            </select>
          )}

          <button 
            onClick={handleToggleHighlightMode}
            className={`p-2 rounded-lg transition-colors ${
              highlightMode 
                ? 'text-[var(--accent)] bg-[var(--accent)]/10' 
                : 'hover:bg-black/5 dark:hover:bg-white/5'
            }`}
            aria-label={highlightMode ? 'Exit highlight mode' : 'Enter highlight mode'}
            title="Highlight text"
          >
            <Highlighter className="w-4 h-4" />
          </button>

          <button 
            onClick={() => setShowAnnotationPanel(prev => !prev)}
            className={`p-2 rounded-lg transition-colors ${
              showAnnotationPanel 
                ? 'text-[var(--accent)] bg-[var(--accent)]/10' 
                : 'hover:bg-black/5 dark:hover:bg-white/5'
            }`}
            aria-label="Toggle annotation panel"
            title="Annotation panel"
          >
            <MessageSquare className="w-4 h-4" />
          </button>

          <div className="w-px h-6 bg-[var(--border)] mx-2" />

          {/* Mesh Toggle */}
          <button 
            onClick={onToggleMesh}
            className={`p-2 rounded-lg transition-colors ${
              meshOpen 
                ? 'text-[var(--accent)] bg-[var(--accent)]/10' 
                : 'hover:bg-black/5 dark:hover:bg-white/5'
            }`}
            aria-label="Toggle semantic mesh"
          >
            <PanelRightOpen className="w-4 h-4" />
          </button>
        </div>
      </div>

      {/* Article Content Area with Annotations */}
      <div className="flex-1 overflow-auto relative">
        {/* Text Highlighter - positioned absolutely */}
        {currentArticle && (
          <TextHighlighter
            containerRef={articleContentRef}
            onHighlight={handleHighlight}
          />
        )}

        {/* Canvas Annotation Layer for click-to-stick */}
        {currentArticle && currentArchive && (
          <CanvasAnnotationLayer
            articleUrl={currentArticle.url}
            archiveId={currentArchive.id}
            contentRef={articleContentRef}
            isPlacingMode={placementMode}
            placementType={selectedPlacementType}
            onPlacement={handleCanvasPlacement}
            onPinClick={(annotation) => {
              // Optionally open panel for editing
              console.log('Pin clicked:', annotation);
            }}
          />
        )}

        <article 
          ref={articleContentRef}
          className="max-w-3xl mx-auto px-8 py-6 prose prose-neutral dark:prose-invert relative"
          style={{ fontSize: `${fontSize}px` }}
        >
          <h1>{currentArticle.title}</h1>
          
          {isLoading ? (
            <div className="text-center py-12">
              <Loader2 className="w-8 h-8 mx-auto mb-4 animate-spin text-[var(--accent)]" />
              <p className="text-secondary">Loading article content...</p>
            </div>
          ) : articleContent ? (
            <div dangerouslySetInnerHTML={{ __html: articleContent }} />
          ) : (
            <div className="text-center py-12">
              <p className="text-secondary">No content available</p>
            </div>
          )}
        </article>
      </div>

      {/* Annotation Panel - Slide in from right */}
      {showAnnotationPanel && currentArticle && currentArchive && (
        <div className="absolute right-0 top-0 h-full z-40">
          <AnnotationPanel
            articleUrl={currentArticle.url}
            archiveId={currentArchive.id}
            onClose={handleCloseAnnotationPanel}
            initialPosition={panelPosition || undefined}
            initialType={selectedPlacementType}
          />
        </div>
      )}
    </div>
  );
}
