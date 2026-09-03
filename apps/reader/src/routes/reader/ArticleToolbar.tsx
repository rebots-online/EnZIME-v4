// ArticleToolbar — §15.1 quiet glass chrome above the reader pane.
//
// M1 scope: shows the article's mime/key provenance and a lightweight action
// row (Home, and an explicit "reload" hook for the recoverable-error path).
// Bookmark / annotation toggles live on later milestones; the slots are
// reserved via `reader-toolbar__actions` so the chrome is stable.

import type { ReactNode } from "react";

export interface ArticleToolbarProps {
  readonly articleKey: string;
  readonly mimeType: string;
  readonly onHome: () => void;
  readonly onReload: () => void;
}

export function ArticleToolbar({
  articleKey,
  mimeType,
  onHome,
  onReload,
}: ArticleToolbarProps): ReactNode {
  return (
    <div className="reader-toolbar" role="toolbar" aria-label="Article actions">
      <div className="reader-toolbar__meta">
        <span className="reader-pill" title={articleKey}>
          {articleKey || "—"}
        </span>
        {mimeType ? (
          <span className="reader-pill reader-pill--muted">{mimeType}</span>
        ) : null}
      </div>
      <div className="reader-toolbar__actions">
        <button
          type="button"
          className="reader-chip"
          onClick={onReload}
          aria-label="Reload article"
        >
          ↻ Reload
        </button>
        <button
          type="button"
          className="reader-chip reader-chip--ghost"
          onClick={onHome}
          aria-label="Back to Home"
        >
          Home
        </button>
      </div>
    </div>
  );
}
