// ReaderView — §15.1 / §15.3 `/reader/:packId/:articleKey` route.
//
// Composition: ReaderView → BreadcrumbNav, ArticleToolbar, ReaderPane
// (ArticleContent). It reads route params, drives readerStore.loadArticle, and
// renders the loading / error / populated / empty states using the shared
// Aurora Glass primitives (resolveState, StatusBanner, SkeletonBlock,
// RetryButton). It also registers a pluggable navigator so the store's
// `navigate(href)` can drive client-side route changes without importing the
// router (which would create a circular dependency).
//
// The article surface is OPAQUE and quiet (Literata via --ez-font-reader);
// Aurora Glass is reserved for the chrome (AppShell, toolbar, breadcrumb).

import { useEffect, useState, type ReactNode } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { useReaderStore, setRouteNavigator } from "../../stores/readerStore";
import {
  resolveState,
  RetryButton,
  SkeletonBlock,
  StatusBanner,
} from "../../components/home/shared";
import { BreadcrumbNav } from "./BreadcrumbNav";
import { ArticleToolbar } from "./ArticleToolbar";
import { ArticleContent } from "./ArticleContent";
import { AnnotationDrawer } from "./AnnotationDrawer";

const READER_STYLE_TEXT = `
.reader-surface {
  --reader-fg: var(--ez-color-text-primary);
  --reader-fg-muted: var(--ez-color-text-secondary);
  --reader-fg-faint: var(--ez-color-text-muted);
  --reader-glass: var(--ez-color-bg-glass);
  --reader-glass-strong: var(--ez-color-bg-glass-strong);
  --reader-border: var(--ez-color-border-subtle);
  --reader-border-strong: rgba(189, 198, 255, 0.24);
  --reader-accent: var(--ez-color-primary);
  --reader-accent-fg: var(--ez-color-bg-canvas);
  --reader-surface-bg: #f7f8fc;
  box-sizing: border-box;
  width: 100%;
  max-width: 64rem;
  margin: 0 auto;
  padding: clamp(0.75rem, 2vw, 1.5rem);
  display: grid;
  gap: var(--ez-space-3);
  font-size: clamp(0.92rem, 1vw, 1rem);
}
.reader-surface *,
.reader-surface *::before,
.reader-surface *::after { box-sizing: border-box; }

.reader-breadcrumb {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.35rem;
  padding: 0.4rem 0.6rem;
  border-radius: var(--ez-radius-md);
  background: var(--reader-glass);
  backdrop-filter: blur(var(--ez-blur-glass));
  -webkit-backdrop-filter: blur(var(--ez-blur-glass));
  border: 1px solid var(--reader-border);
  font-size: 0.85rem;
}
.reader-breadcrumb__slot {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  min-width: 0;
}
.reader-breadcrumb__sep {
  color: var(--reader-fg-faint);
  user-select: none;
}
.reader-crumb {
  display: inline-flex;
  align-items: center;
  max-width: 22rem;
  border: 1px solid transparent;
  border-radius: 999px;
  padding: 0.2rem 0.6rem;
  background: transparent;
  color: var(--reader-fg);
  font: inherit;
  font-size: 0.85rem;
  cursor: pointer;
  text-decoration: none;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  transition: background-color var(--ez-duration-fast) ease,
    border-color var(--ez-duration-fast) ease;
}
.reader-crumb:hover { background: rgba(124, 140, 255, 0.12); }
.reader-crumb:focus-visible {
  outline: none;
  box-shadow: var(--ez-focus-ring);
  border-color: var(--ez-color-border-focus);
}
.reader-crumb--muted { color: var(--reader-fg-muted); cursor: default; }
.reader-crumb--muted:hover { background: transparent; }
.reader-crumb--current {
  color: var(--reader-accent);
  background: rgba(124, 140, 255, 0.14);
  cursor: default;
  font-weight: 600;
}
.reader-crumb--current:hover { background: rgba(124, 140, 255, 0.14); }

.reader-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--ez-space-3);
  flex-wrap: wrap;
  padding: 0.45rem 0.6rem;
  border-radius: var(--ez-radius-md);
  background: var(--reader-glass);
  backdrop-filter: blur(var(--ez-blur-glass));
  -webkit-backdrop-filter: blur(var(--ez-blur-glass));
  border: 1px solid var(--reader-border);
}
.reader-toolbar__meta {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  min-width: 0;
  flex: 1 1 auto;
}
.reader-toolbar__actions {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
}
.reader-pill {
  display: inline-flex;
  align-items: center;
  padding: 0.12em 0.55em;
  font-size: 0.74rem;
  font-family: var(--ez-font-mono);
  border-radius: 999px;
  border: 1px solid var(--reader-border);
  color: var(--reader-fg-muted);
  background: rgba(17, 21, 33, 0.45);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 18rem;
}
.reader-pill--muted { color: var(--reader-fg-faint); }
.reader-chip {
  display: inline-flex;
  align-items: center;
  gap: 0.3em;
  padding: 0.32em 0.8em;
  font-size: 0.82rem;
  font-weight: 600;
  border-radius: 999px;
  border: 1px solid transparent;
  background: var(--reader-accent);
  color: var(--reader-accent-fg);
  cursor: pointer;
  transition: background-color var(--ez-duration-fast) ease,
    transform var(--ez-duration-fast) ease;
}
.reader-chip:hover { background: var(--ez-color-primary-hover); }
.reader-chip:focus-visible {
  outline: none;
  box-shadow: var(--ez-focus-ring);
}
.reader-chip:active { transform: translateY(1px); }
.reader-chip--ghost {
  background: transparent;
  color: var(--reader-fg);
  border-color: var(--reader-border-strong);
}
.reader-chip--ghost:hover { background: rgba(124, 140, 255, 0.12); }

.reader-pane {
  position: relative;
  border-radius: var(--ez-radius-lg);
  border: 1px solid var(--reader-border);
  background: var(--reader-surface-bg);
  box-shadow: var(--ez-shadow-panel);
  overflow: hidden;
  min-height: 50vh;
}
.reader-pane[aria-busy="true"]::after {
  content: "";
  position: absolute;
  inset: 0;
  background: linear-gradient(
    180deg,
    rgba(124, 140, 255, 0.04) 0%,
    transparent 40%
  );
  pointer-events: none;
}
.reader-pane__loading {
  padding: clamp(1.5rem, 4vw, 3rem);
  display: grid;
  gap: 0.8rem;
}
.reader-pane__error,
.reader-pane__empty {
  padding: clamp(1.5rem, 4vw, 3rem);
}

.reader-article {
  width: 100%;
}
.reader-article__frame {
  width: 100%;
  min-height: 70vh;
  border: 0;
  display: block;
  background: var(--reader-surface-bg);
}
.reader-article__fallback {
  margin: 0;
  padding: clamp(1rem, 4vw, 2rem);
  font-family: var(--ez-font-mono);
  font-size: 0.85rem;
  white-space: pre-wrap;
  word-break: break-word;
  color: var(--ez-color-text-primary);
  background: var(--reader-surface-bg);
}
.reader-sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border: 0;
}
`;

let readerStyleMounted = false;

function ReaderStyles(): ReactNode {
  const [mounted, setMounted] = useState(readerStyleMounted);
  useEffect(() => {
    if (readerStyleMounted) return;
    readerStyleMounted = true;
    setMounted(true);
  }, []);
  if (!mounted) return null;
  return <style aria-hidden="true">{READER_STYLE_TEXT}</style>;
}

export function ReaderView(): ReactNode {
  const params = useParams<{ packId: string; articleKey: string }>();
  const routePackId = params.packId ?? "";
  const routeArticleKey = params.articleKey ?? "";

  const navigate = useNavigate();

  const currentPackId = useReaderStore((s) => s.currentPackId);
  const currentPackTitle = useReaderStore((s) => s.currentPackTitle);
  const currentArticleKey = useReaderStore((s) => s.currentArticleKey);
  const currentArticle = useReaderStore((s) => s.currentArticle);
  const loadArticle = useReaderStore((s) => s.loadArticle);

  // Register the router navigator so the store can resolve+drive article links.
  useEffect(() => {
    setRouteNavigator((path: string) => navigate(path));
    return () => setRouteNavigator(null);
  }, [navigate]);

  // Load whenever the route's (packId, articleKey) differs from what is loaded.
  // A given position is fetched exactly once; explicit retry is driven by the
  // ArticleToolbar / error-state RetryButton (which calls loadArticle directly),
  // so this effect must not re-fire on the same errored position (that would
  // loop).
  useEffect(() => {
    if (!routePackId || !routeArticleKey) return;
    let key = routeArticleKey;
    try {
      key = decodeURIComponent(routeArticleKey);
    } catch {
      // keep raw if malformed
    }
    const samePosition =
      currentPackId === routePackId && currentArticleKey === key;
    if (samePosition) return;
    void loadArticle(routePackId, key);
  }, [
    routePackId,
    routeArticleKey,
    currentPackId,
    currentArticleKey,
    loadArticle,
  ]);

  const state = resolveState(
    true,
    currentArticle,
    (data) => data.rawHtml.trim().length > 0,
  );
  const article = currentArticle.data;

  const goHome = () => navigate("/");

  let pane: ReactNode;
  if (state === "loading") {
    pane = (
      <div className="reader-pane__loading" aria-live="polite">
        <SkeletonBlock lines={4} />
      </div>
    );
  } else if (state === "populated" && article) {
    pane = (
      <>
        <ArticleContent article={article} packId={routePackId} />
        {currentArticleKey ? (
          <AnnotationDrawer
            packId={routePackId}
            articleKey={currentArticleKey}
            title={article.title}
          />
        ) : null}
      </>
    );
  } else if (state === "empty") {
    pane = (
      <div className="reader-pane__empty">
        <StatusBanner
          tone="info"
          icon="i"
          title="This article is empty"
          detail="The ZIM entry resolved, but contains no renderable content."
          actions={
            <button
              type="button"
              className="reader-chip reader-chip--ghost"
              onClick={goHome}
            >
              Back to Home
            </button>
          }
        />
      </div>
    );
  } else if (state === "recoverable") {
    pane = (
      <div className="reader-pane__error">
        <StatusBanner
          tone="warning"
          icon="!"
          title="Could not load article"
          detail={currentArticle.error?.message ?? "Unexpected error."}
          actions={
            <RetryButton
              onClick={() =>
                void loadArticle(routePackId, routeArticleKey || "")
              }
              label="Try again"
            />
          }
        />
      </div>
    );
  } else if (state === "non-recoverable") {
    pane = (
      <div className="reader-pane__error">
        <StatusBanner
          tone="error"
          icon="×"
          title="Article unavailable"
          detail={
            currentArticle.error?.message ??
            "The article could not be read from this pack."
          }
          actions={
            <button
              type="button"
              className="reader-chip reader-chip--ghost"
              onClick={goHome}
            >
              Back to Home
            </button>
          }
        />
      </div>
    );
  } else {
    // capability-denied
    pane = (
      <div className="reader-pane__error">
        <StatusBanner
          tone="locked"
          icon="∅"
          title="Article locked"
          detail={
            currentArticle.error?.message ??
            "This profile cannot read articles from this pack."
          }
          actions={
            <button
              type="button"
              className="reader-chip reader-chip--ghost"
              onClick={goHome}
            >
              Back to Home
            </button>
          }
        />
      </div>
    );
  }

  const busy = state === "loading";

  return (
    <main className="reader-surface" aria-busy={busy}>
      <ReaderStyles />
      <BreadcrumbNav
        packId={routePackId}
        packTitle={currentPackTitle}
        articleTitle={article?.title ?? routeArticleKey ?? "Article"}
        onHome={goHome}
      />
      <ArticleToolbar
        articleKey={routeArticleKey}
        mimeType={article?.mimeType ?? ""}
        onHome={goHome}
        onReload={() =>
          void loadArticle(routePackId, routeArticleKey || "")
        }
      />
      <div className="reader-pane" aria-busy={busy}>
        {pane}
      </div>
    </main>
  );
}
