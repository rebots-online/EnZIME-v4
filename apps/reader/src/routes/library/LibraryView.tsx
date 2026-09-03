// LibraryView — §15.1 / §15.3 `/library` route.
//
// The user-facing entry to EnZIME's DynDon download — the USP: "download only
// what fits your storage". Composition: LibraryView mounts the scoped Aurora
// Glass stylesheet once, then composes PackGrid (installed), CatalogBrowser
// (search), StoragePlanner + InstallPlanSummary + StorageImpactBar (the fit
// moment, hosted inside DownloadPlanDialog), and DownloadQueue (active jobs).
//
// Styling follows the home/reader convention: a private `--lib-*` token set
// scoped to `.lib-surface`, mapped onto the global `--ez-*` Aurora Glass
// tokens (§15.6). Animation is the default experience (the fit bar grows in,
// the FITS badge breathes); prefers-reduced-motion is the a11y override only.
// Reading-adjacent tiles stay glass; long-form reading surfaces elsewhere
// remain opaque (§15.6 chrome-vs-content split).

import { useEffect, type ReactNode } from "react";
import { useLibraryStore } from "../../stores/libraryStore";
import { useOnlineSubscription } from "../../components/home/shared";
import { PackGrid } from "./PackGrid";
import { CatalogBrowser } from "./CatalogBrowser";
import { DownloadQueue } from "./DownloadQueue";
import { DownloadPlanDialog } from "./DownloadPlanDialog";

const LIBRARY_STYLE_TEXT = `
.lib-surface {
  --lib-fg: var(--ez-color-text-primary);
  --lib-fg-muted: var(--ez-color-text-secondary);
  --lib-fg-faint: var(--ez-color-text-muted);
  --lib-glass: var(--ez-color-bg-glass);
  --lib-glass-strong: var(--ez-color-bg-glass-strong);
  --lib-border: var(--ez-color-border-subtle);
  --lib-border-strong: rgba(189, 198, 255, 0.26);
  --lib-accent: var(--ez-color-primary);
  --lib-accent-hover: var(--ez-color-primary-hover);
  --lib-accent-fg: var(--ez-color-bg-canvas);
  --lib-teal: var(--ez-color-accent);
  --lib-ready: var(--ez-color-success);
  --lib-warning: var(--ez-color-warning);
  --lib-danger: var(--ez-color-danger);
  --lib-focus: var(--ez-color-border-focus);
  box-sizing: border-box;
  width: 100%;
  max-width: 76rem;
  margin: 0 auto;
  padding: clamp(1rem, 2.5vw, 2.25rem);
  display: grid;
  gap: clamp(0.85rem, 1.6vw, 1.35rem);
  font-size: clamp(0.92rem, 1vw, 1rem);
}
.lib-surface *,
.lib-surface *::before,
.lib-surface *::after { box-sizing: border-box; }

.lib-header {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}
.lib-header__title {
  margin: 0;
  font-size: 1.45em;
  font-weight: 700;
  letter-spacing: 0.01em;
  background: linear-gradient(120deg,
    var(--lib-accent) 0%,
    var(--lib-teal) 70%,
    var(--lib-ready) 100%);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}
.lib-tagline {
  margin: 0;
  color: var(--lib-fg-muted);
  font-size: 0.95em;
}
.lib-tagline strong { color: var(--lib-fg); font-weight: 600; }

.lib-layout {
  display: grid;
  gap: clamp(0.85rem, 1.6vw, 1.35rem);
  grid-template-columns: minmax(0, 2.2fr) minmax(0, 1fr);
  grid-template-areas:
    "main side"
    "main side";
  align-items: start;
}
@media (max-width: 60rem) {
  .lib-layout {
    grid-template-columns: minmax(0, 1fr);
    grid-template-areas: "main" "side";
  }
}
.lib-col--main { grid-area: main; display: grid; gap: clamp(0.85rem, 1.6vw, 1.35rem); min-width: 0; }
.lib-col--side { grid-area: side; display: grid; gap: clamp(0.85rem, 1.6vw, 1.35rem); min-width: 0; }

.lib-card {
  position: relative;
  background: var(--lib-glass);
  border: 1px solid var(--lib-border);
  border-radius: var(--ez-radius-lg);
  padding: clamp(0.9rem, 1.4vw, 1.25rem);
  backdrop-filter: blur(var(--ez-blur-glass));
  -webkit-backdrop-filter: blur(var(--ez-blur-glass));
  box-shadow: var(--ez-shadow-panel);
  display: flex;
  flex-direction: column;
  gap: 0.8rem;
  min-width: 0;
}
.lib-card__header {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 0.75rem;
  flex-wrap: wrap;
}
.lib-card__title {
  margin: 0;
  font-size: 1.05em;
  font-weight: 600;
  letter-spacing: 0.01em;
}
.lib-card__hint {
  color: var(--lib-fg-muted);
  font-size: 0.82em;
  font-family: var(--ez-font-mono);
}

.lib-search {
  display: flex;
  gap: 0.5rem;
  flex-wrap: wrap;
}
.lib-input,
.lib-select {
  flex: 1 1 auto;
  min-width: 0;
  padding: 0.55rem 0.85rem;
  border-radius: var(--ez-radius-md);
  border: 1px solid var(--lib-border);
  background: rgba(17, 21, 33, 0.6);
  color: var(--lib-fg);
  font: inherit;
  transition: border-color var(--ez-duration-normal) ease,
    box-shadow var(--ez-duration-normal) ease;
}
.lib-input::placeholder { color: var(--lib-fg-faint); }
.lib-input:focus-visible,
.lib-select:focus-visible {
  outline: none;
  border-color: var(--lib-focus);
  box-shadow: var(--ez-focus-ring);
}

.lib-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.4em;
  background: var(--lib-accent);
  color: var(--lib-accent-fg);
  border: 1px solid transparent;
  border-radius: 999px;
  padding: 0.5em 1.05em;
  font: inherit;
  font-weight: 600;
  cursor: pointer;
  text-decoration: none;
  transition: background-color var(--ez-duration-normal) ease,
    transform var(--ez-duration-fast) ease,
    box-shadow var(--ez-duration-normal) ease,
    border-color var(--ez-duration-normal) ease;
}
.lib-button:hover { background: var(--lib-accent-hover); }
.lib-button:focus-visible { outline: none; box-shadow: var(--ez-focus-ring); }
.lib-button:active { transform: translateY(1px); }
.lib-button:disabled { opacity: 0.45; cursor: not-allowed; transform: none; }
.lib-button--ghost {
  background: transparent;
  color: var(--lib-fg);
  border-color: var(--lib-border-strong);
}
.lib-button--ghost:hover { background: rgba(124, 140, 255, 0.10); }
.lib-button--success { background: var(--lib-ready); color: #052014; }
.lib-button--success:hover { filter: brightness(1.08); }
.lib-button--danger { background: var(--lib-danger); color: #1A0606; }

.lib-grid {
  display: grid;
  gap: 0.75rem;
  grid-template-columns: repeat(auto-fill, minmax(11rem, 1fr));
}
.lib-tile {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
  text-align: left;
  background: rgba(17, 21, 33, 0.55);
  border: 1px solid var(--lib-border);
  border-radius: var(--ez-radius-md);
  padding: 0.85rem 0.9rem;
  color: inherit;
  font: inherit;
  cursor: pointer;
  overflow: hidden;
  transition: border-color var(--ez-duration-normal) ease,
    transform var(--ez-duration-fast) ease,
    box-shadow var(--ez-duration-normal) ease;
}
.lib-tile::after {
  content: "";
  position: absolute;
  inset: -40% -40% auto auto;
  width: 60%;
  height: 60%;
  background: radial-gradient(circle, rgba(124,140,255,0.18), transparent 70%);
  opacity: 0;
  transition: opacity var(--ez-duration-normal) ease;
  pointer-events: none;
}
.lib-tile:hover {
  border-color: var(--lib-border-strong);
  transform: translateY(-2px);
  box-shadow: 0 10px 28px rgba(0,0,0,0.30);
}
.lib-tile:hover::after { opacity: 1; }
.lib-tile:focus-visible { outline: none; box-shadow: var(--ez-focus-ring); }
.lib-tile__title { font-weight: 600; line-height: 1.25; }
.lib-tile__meta {
  color: var(--lib-fg-muted);
  font-size: 0.85em;
  font-family: var(--ez-font-mono);
}
.lib-tile__lang {
  align-self: flex-start;
}

.lib-row {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  width: 100%;
  text-align: left;
  background: transparent;
  border: 1px solid transparent;
  border-radius: var(--ez-radius-sm);
  padding: 0.55rem 0.65rem;
  color: inherit;
  font: inherit;
  cursor: pointer;
  transition: background-color var(--ez-duration-normal) ease,
    border-color var(--ez-duration-normal) ease;
}
.lib-row:hover { background: rgba(124,140,255,0.08); }
.lib-row:focus-visible { outline: none; box-shadow: var(--ez-focus-ring); }
.lib-row--selected {
  border-color: var(--lib-accent);
  background: rgba(124,140,255,0.12);
}
.lib-row__main { display: flex; flex-direction: column; gap: 0.15rem; flex: 1 1 auto; min-width: 0; }
.lib-row__label { font-weight: 500; }
.lib-row__meta {
  color: var(--lib-fg-muted);
  font-size: 0.85em;
  font-family: var(--ez-font-mono);
}

.lib-pill {
  display: inline-flex;
  align-items: center;
  gap: 0.3em;
  padding: 0.12em 0.6em;
  font-size: 0.76em;
  font-family: var(--ez-font-mono);
  border-radius: 999px;
  border: 1px solid var(--lib-border);
  color: var(--lib-fg-muted);
  background: rgba(17, 21, 33, 0.6);
  white-space: nowrap;
}
.lib-pill--accent { color: var(--lib-accent); border-color: rgba(124,140,255,0.4); }
.lib-pill--ready { color: var(--lib-ready); border-color: rgba(76,227,164,0.4); }
.lib-pill--warning { color: var(--lib-warning); border-color: rgba(255,202,106,0.4); }
.lib-pill--danger { color: var(--lib-danger); border-color: rgba(255,111,145,0.4); }

.lib-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
  min-width: 0;
}

/* ─── Storage impact bar — the DynDon payoff ────────────────────────────── */
.lib-impact { display: flex; flex-direction: column; gap: 0.5rem; }
.lib-impact__headline {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-wrap: wrap;
  font-size: 0.98em;
}
.lib-impact__headline strong { font-weight: 600; }
.lib-fit-badge {
  display: inline-flex;
  align-items: center;
  gap: 0.3em;
  padding: 0.15em 0.7em;
  border-radius: 999px;
  font-family: var(--ez-font-mono);
  font-size: 0.78em;
  font-weight: 700;
  letter-spacing: 0.06em;
  text-transform: uppercase;
}
.lib-fit-badge--fits {
  color: #052014;
  background: linear-gradient(120deg, var(--lib-ready), var(--lib-teal));
  box-shadow: 0 0 18px rgba(76,227,164,0.45);
  animation: lib-breathe 2.6s ease-in-out infinite;
}
.lib-fit-badge--tight {
  color: #2A1A03;
  background: var(--lib-warning);
  box-shadow: 0 0 14px rgba(255,202,106,0.4);
}
.lib-fit-badge--no { color: #1A0606; background: var(--lib-danger); }

.lib-bar {
  position: relative;
  width: 100%;
  height: 1.1rem;
  border-radius: 999px;
  background: rgba(17, 21, 33, 0.7);
  border: 1px solid var(--lib-border);
  overflow: hidden;
}
.lib-bar__track-free {
  position: absolute;
  inset: 0;
  background: repeating-linear-gradient(
    135deg,
    rgba(189,198,255,0.05) 0 0.4rem,
    transparent 0.4rem 0.8rem
  );
}
.lib-bar__seg {
  position: absolute;
  top: 0;
  bottom: 0;
  transition: width var(--ez-duration-normal) ease;
}
.lib-bar__seg--final {
  left: 0;
  background: linear-gradient(90deg, var(--lib-accent), var(--lib-teal));
  box-shadow: 0 0 16px rgba(52,224,212,0.5);
}
.lib-bar__seg--temp {
  background: repeating-linear-gradient(
    45deg,
    rgba(255,202,106,0.55) 0 0.45rem,
    rgba(255,202,106,0.25) 0.45rem 0.9rem
  );
  animation: lib-stripe 900ms linear infinite;
}
.lib-bar__marker {
  position: absolute;
  top: -0.25rem;
  bottom: -0.25rem;
  width: 2px;
  background: var(--lib-fg);
  box-shadow: 0 0 8px rgba(244,246,255,0.6);
}
.lib-bar__marker::after {
  content: attr(data-label);
  position: absolute;
  top: -1.05rem;
  left: 50%;
  transform: translateX(-50%);
  font-size: 0.66rem;
  font-family: var(--ez-font-mono);
  color: var(--lib-fg-muted);
  white-space: nowrap;
}
.lib-impact__legend {
  display: flex;
  gap: 0.85rem;
  flex-wrap: wrap;
  font-size: 0.8em;
  color: var(--lib-fg-muted);
}
.lib-legend__swatch {
  display: inline-block;
  width: 0.7rem;
  height: 0.7rem;
  border-radius: 3px;
  margin-right: 0.3em;
  vertical-align: -1px;
}

/* ─── Plan summary ──────────────────────────────────────────────────────── */
.lib-summary {
  display: grid;
  gap: 0.4rem;
  padding: 0.7rem 0.85rem;
  border-radius: var(--ez-radius-md);
  border: 1px solid var(--lib-border);
  background: rgba(17, 21, 33, 0.55);
}
.lib-summary__row {
  display: flex;
  justify-content: space-between;
  gap: 0.75rem;
  font-size: 0.9em;
}
.lib-summary__row--accent { color: var(--lib-teal); font-weight: 600; }
.lib-summary__k { color: var(--lib-fg-muted); }
.lib-summary__v { font-family: var(--ez-font-mono); }

/* ─── Dialog ────────────────────────────────────────────────────────────── */
.lib-dialog__backdrop {
  position: fixed;
  inset: 0;
  z-index: 70;
  background: rgba(5, 7, 13, 0.6);
  backdrop-filter: blur(6px);
  -webkit-backdrop-filter: blur(6px);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: clamp(1rem, 4vw, 3rem);
  animation: lib-fade-in var(--ez-duration-normal) ease both;
}
.lib-dialog {
  width: min(40rem, 100%);
  max-height: min(40rem, 88vh);
  overflow: auto;
  background: var(--lib-glass-strong);
  border: 1px solid var(--lib-border-strong);
  border-radius: var(--ez-radius-lg);
  box-shadow: var(--ez-shadow-panel);
  padding: clamp(1rem, 2vw, 1.5rem);
  display: flex;
  flex-direction: column;
  gap: 0.9rem;
  animation: lib-dialog-in var(--ez-duration-normal) ease both;
}
.lib-dialog:focus { outline: none; }
.lib-dialog__head { display: flex; flex-direction: column; gap: 0.2rem; }
.lib-dialog__title { margin: 0; font-size: 1.1em; font-weight: 700; }
.lib-dialog__sub { margin: 0; color: var(--lib-fg-muted); font-size: 0.9em; }
.lib-dialog__actions {
  display: flex;
  gap: 0.5rem;
  justify-content: flex-end;
  flex-wrap: wrap;
  margin-top: 0.2rem;
}

/* ─── Queue ─────────────────────────────────────────────────────────────── */
.lib-queue__row {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
  padding: 0.6rem 0.7rem;
  border-radius: var(--ez-radius-sm);
  border: 1px solid var(--lib-border);
  background: rgba(17, 21, 33, 0.5);
}
.lib-queue__meta {
  display: flex;
  justify-content: space-between;
  gap: 0.5rem;
  font-size: 0.8em;
  color: var(--lib-fg-muted);
  font-family: var(--ez-font-mono);
}

.lib-section-intro {
  color: var(--lib-fg-muted);
  font-size: 0.9em;
  line-height: 1.45;
}

@keyframes lib-breathe {
  0%, 100% { transform: scale(1); box-shadow: 0 0 18px rgba(76,227,164,0.45); }
  50% { transform: scale(1.035); box-shadow: 0 0 26px rgba(76,227,164,0.7); }
}
@keyframes lib-stripe {
  0% { background-position: 0 0; }
  100% { background-position: 0.9rem 0; }
}
@keyframes lib-fade-in {
  from { opacity: 0; }
  to { opacity: 1; }
}
@keyframes lib-dialog-in {
  from { opacity: 0; transform: translateY(10px) scale(0.985); }
  to { opacity: 1; transform: translateY(0) scale(1); }
}

@media (prefers-reduced-motion: reduce) {
  .lib-button,
  .lib-tile,
  .lib-row,
  .lib-bar__seg,
  .lib-fit-badge--fits,
  .lib-bar__seg--temp,
  .lib-dialog,
  .lib-dialog__backdrop {
    animation: none !important;
    transition: none !important;
  }
}
`;

let libStyleMounted = false;

/**
 * Mounts the scoped Library stylesheet exactly once per process (mirrors the
 * home/reader HomeStyles/ReaderStyles pattern). LibraryView renders this; the
 * children reference the class names directly.
 */
export function LibraryStyles(): ReactNode {
  if (libStyleMounted) return null;
  libStyleMounted = true;
  return <style aria-hidden="true">{LIBRARY_STYLE_TEXT}</style>;
}

/** Reset the mounted flag — used only by tests / hot-reload in dev. Intentionally
 *  not used in production paths. */
export function __resetLibraryStylesForDev(): void {
  libStyleMounted = false;
}

export function LibraryView(): ReactNode {
  const loadAll = useLibraryStore((s) => s.loadAll);
  const setOnline = useLibraryStore((s) => s.setOnline);
  const online = useLibraryStore((s) => s.online);
  const activeCatalogEntryId = useLibraryStore(
    (s) => s.activeCatalogEntryId,
  );
  const anyLoading = useLibraryStore(
    (s) =>
      s.storageRoots.status === "loading" ||
      s.packs.status === "loading" ||
      s.downloads.status === "loading",
  );

  useOnlineSubscription(setOnline);

  useEffect(() => {
    void loadAll();
  }, [loadAll]);

  return (
    <main className="lib-surface" aria-busy={anyLoading}>
      <LibraryStyles />
      <header className="lib-header">
        <h1 className="lib-header__title">Library</h1>
        <p className="lib-tagline">
          Download only <strong>what fits your storage</strong> — DynDon streams
          the slice you can actually keep.
        </p>
      </header>

      {!online ? (
        <p
          className="lib-pill lib-pill--warning"
          role="status"
          aria-live="polite"
          style={{ alignSelf: "flex-start" }}
        >
          ! Offline — catalog search is unavailable
        </p>
      ) : null}

      <div className="lib-layout">
        <div className="lib-col lib-col--main">
          <PackGrid />
          <CatalogBrowser />
        </div>
        <aside className="lib-col lib-col--side" aria-label="Storage and downloads">
          <DownloadQueue />
        </aside>
      </div>

      {activeCatalogEntryId ? <DownloadPlanDialog /> : null}
    </main>
  );
}
