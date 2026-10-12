import type { ReactNode } from "react";

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

export function ReaderStyles(): ReactNode {
  return <style aria-hidden="true">{READER_STYLE_TEXT}</style>;
}

