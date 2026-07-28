import { useEffect, useState, type CSSProperties, type ReactNode } from "react";
import type { Slice } from "../../stores/homeStore";
import type { ServiceError } from "../../services/homeService";

export type ResolvedState =
  | "offline-empty"
  | "loading"
  | "populated"
  | "empty"
  | "recoverable"
  | "non-recoverable"
  | "capability-denied";

export function resolveState<T>(
  online: boolean,
  slice: Slice<T>,
  hasData: (data: T) => boolean,
): ResolvedState {
  if (!online && slice.data === null) return "offline-empty";
  if (slice.status === "loading" || slice.status === "idle") {
    if (slice.data !== null) return "populated";
    return "loading";
  }
  if (slice.status === "error" && slice.error) {
    if (slice.error.kind === "capability-denied") return "capability-denied";
    if (slice.error.kind === "non-recoverable") return "non-recoverable";
    return "recoverable";
  }
  if (slice.status === "ready" && slice.data !== null) {
    return hasData(slice.data) ? "populated" : "empty";
  }
  return "loading";
}

const STYLE_TEXT = `
.home-surface {
  --home-fg: #F4F6FF;
  --home-fg-muted: rgba(244, 246, 255, 0.66);
  --home-fg-faint: rgba(244, 246, 255, 0.42);
  --home-glass: rgba(255, 255, 255, 0.04);
  --home-glass-strong: rgba(255, 255, 255, 0.07);
  --home-border: rgba(255, 255, 255, 0.10);
  --home-border-strong: rgba(255, 255, 255, 0.20);
  --home-accent: #7CC4FF;
  --home-accent-fg: #07111E;
  --home-ready: #6FE3A8;
  --home-warning: #F4C36B;
  --home-degraded: #F49B6B;
  --home-unavailable: #F47B7B;
  --home-focus: #A8CFFF;
  box-sizing: border-box;
  width: 100%;
  max-width: 72rem;
  margin: 0 auto;
  padding: clamp(1rem, 2.5vw, 2.5rem);
  display: grid;
  gap: clamp(0.75rem, 1.6vw, 1.25rem);
  font-size: clamp(0.9rem, 1vw, 1rem);
}
.home-surface *,
.home-surface *::before,
.home-surface *::after { box-sizing: border-box; }

.home-grid {
  display: grid;
  gap: clamp(0.75rem, 1.6vw, 1.25rem);
  grid-template-columns: minmax(0, 2fr) minmax(0, 1fr);
  grid-template-areas:
    "continue readiness"
    "recent   readiness"
    "tasks    memory"
    "tasks    memory";
}
@media (max-width: 56rem) {
  .home-grid {
    grid-template-columns: minmax(0, 1fr);
    grid-template-areas:
      "continue"
      "readiness"
      "recent"
      "tasks"
      "memory";
  }
}

.home-card {
  position: relative;
  background: var(--home-glass);
  border: 1px solid var(--home-border);
  border-radius: 0.85rem;
  padding: clamp(0.85rem, 1.4vw, 1.25rem);
  backdrop-filter: blur(8px);
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  min-width: 0;
}
.home-card--continue { grid-area: continue; }
.home-card--recent { grid-area: recent; }
.home-card--tasks { grid-area: tasks; }
.home-card--memory { grid-area: memory; }
.home-card--readiness { grid-area: readiness; }

.home-card__header {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 0.75rem;
  flex-wrap: wrap;
}
.home-card__title {
  margin: 0;
  font-size: 1.05em;
  font-weight: 600;
  letter-spacing: 0.01em;
}
.home-card__hint {
  color: var(--home-fg-muted);
  font-size: 0.85em;
  font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, Menlo, monospace;
}

.home-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  min-width: 0;
}
.home-list__item { margin: 0; }

.home-row {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  width: 100%;
  text-align: left;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 0.55rem;
  padding: 0.6rem 0.7rem;
  color: inherit;
  font: inherit;
  cursor: pointer;
  transition: background-color 160ms ease, border-color 160ms ease,
    transform 160ms ease;
  text-decoration: none;
  min-width: 0;
}
.home-row:hover { background: var(--home-glass-strong); }
.home-row:focus-visible {
  outline: 2px solid var(--home-focus);
  outline-offset: 2px;
  border-color: var(--home-border-strong);
}
.home-row:active { transform: translateY(1px); }

.home-row__main {
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
  flex: 1 1 auto;
  min-width: 0;
}
.home-row__label {
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.home-row__meta {
  color: var(--home-fg-muted);
  font-size: 0.85em;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.home-row__aside {
  color: var(--home-fg-muted);
  font-size: 0.8em;
  font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, Menlo, monospace;
  white-space: nowrap;
}

.home-button {
  display: inline-flex;
  align-items: center;
  gap: 0.4em;
  background: var(--home-accent);
  color: var(--home-accent-fg);
  border: 1px solid transparent;
  border-radius: 999px;
  padding: 0.5em 1em;
  font: inherit;
  font-weight: 600;
  text-decoration: none;
  cursor: pointer;
  transition: background-color 160ms ease, transform 160ms ease,
    border-color 160ms ease;
}
.home-button:hover { background: #9BD3FF; }
.home-button:focus-visible {
  outline: 2px solid var(--home-focus);
  outline-offset: 2px;
}
.home-button:active { transform: translateY(1px); }
.home-button--ghost {
  background: transparent;
  color: var(--home-fg);
  border-color: var(--home-border-strong);
}
.home-button--ghost:hover { background: var(--home-glass-strong); }
.home-button--danger { background: var(--home-unavailable); color: #1A0606; }
.home-button--success { background: var(--home-ready); color: #052014; }

.home-status {
  display: flex;
  align-items: flex-start;
  gap: 0.6rem;
  padding: 0.75rem 0.85rem;
  border-radius: 0.55rem;
  border: 1px solid var(--home-border);
  background: var(--home-glass);
}
.home-status__icon {
  flex: 0 0 auto;
  width: 1.25rem;
  height: 1.25rem;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-weight: 700;
  border-radius: 50%;
  font-size: 0.85em;
}
.home-status--info .home-status__icon { background: var(--home-accent); color: var(--home-accent-fg); }
.home-status--warning .home-status__icon { background: var(--home-warning); color: #2A1A03; }
.home-status--error .home-status__icon { background: var(--home-unavailable); color: #1A0606; }
.home-status--locked .home-status__icon { background: var(--home-fg-faint); color: #090B13; }
.home-status--offline .home-status__icon { background: var(--home-degraded); color: #2A1306; }
.home-status__body { display: flex; flex-direction: column; gap: 0.35rem; flex: 1 1 auto; min-width: 0; }
.home-status__title { font-weight: 600; }
.home-status__detail { color: var(--home-fg-muted); font-size: 0.9em; }
.home-status__actions { display: flex; gap: 0.5rem; flex-wrap: wrap; margin-top: 0.1rem; }

.home-skeleton {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}
.home-skeleton__line {
  height: 0.85rem;
  border-radius: 0.3rem;
  background: linear-gradient(
    90deg,
    rgba(255, 255, 255, 0.04) 0%,
    rgba(255, 255, 255, 0.12) 50%,
    rgba(255, 255, 255, 0.04) 100%
  );
  background-size: 200% 100%;
  animation: home-shimmer 1400ms ease-in-out infinite;
}
.home-skeleton__line--w-70 { width: 70%; }
.home-skeleton__line--w-40 { width: 40%; }
.home-skeleton__line--w-90 { width: 90%; }
.home-skeleton__line--w-50 { width: 50%; }

.home-progress {
  position: relative;
  width: 100%;
  height: 0.45rem;
  border-radius: 999px;
  background: var(--home-glass-strong);
  overflow: hidden;
}
.home-progress__bar {
  position: absolute;
  inset: 0 auto 0 0;
  background: var(--home-accent);
  border-radius: inherit;
  transition: width 220ms ease;
}
.home-progress--indeterminate .home-progress__bar {
  inset: 0;
  width: 35%;
  animation: home-progress-slide 1400ms ease-in-out infinite;
}
.home-progress__stripe {
  position: absolute;
  inset: 0;
  width: 100%;
  background-image: repeating-linear-gradient(
    45deg,
    rgba(255,255,255,0.18) 0 0.5rem,
    transparent 0.5rem 1rem
  );
  animation: home-progress-stripe 900ms linear infinite;
}

.home-pill {
  display: inline-flex;
  align-items: center;
  gap: 0.3em;
  padding: 0.15em 0.55em;
  font-size: 0.78em;
  font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, Menlo, monospace;
  border-radius: 999px;
  border: 1px solid var(--home-border);
  color: var(--home-fg-muted);
  background: var(--home-glass);
  white-space: nowrap;
}
.home-pill--ready { color: var(--home-ready); border-color: rgba(111, 227, 168, 0.35); }
.home-pill--warning { color: var(--home-warning); border-color: rgba(244, 195, 107, 0.35); }
.home-pill--degraded { color: var(--home-degraded); border-color: rgba(244, 155, 107, 0.35); }
.home-pill--unavailable { color: var(--home-unavailable); border-color: rgba(244, 123, 123, 0.35); }

.home-continue {
  display: grid;
  gap: 0.4rem;
}
.home-continue__article {
  font-size: 1.15em;
  font-weight: 600;
}
.home-continue__pack { color: var(--home-fg-muted); font-size: 0.9em; }
.home-continue__actions {
  display: flex;
  gap: 0.5rem;
  flex-wrap: wrap;
  margin-top: 0.4rem;
}

.home-flag {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  padding: 0.55rem 0.7rem;
  border-radius: 0.55rem;
  border: 1px solid var(--home-border);
  background: var(--home-glass);
}
.home-flag__head {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}
.home-flag__title { font-weight: 600; }
.home-flag__detail { color: var(--home-fg-muted); font-size: 0.88em; }

.home-readiness__overall {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  font-size: 1.05em;
  font-weight: 600;
}
.home-readiness__summary { color: var(--home-fg-muted); font-size: 0.92em; }
.home-readiness__flags {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}

.home-section-intro {
  color: var(--home-fg-muted);
  font-size: 0.92em;
  line-height: 1.4;
}

.home-memory__row {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
  padding: 0.55rem 0.7rem;
  border-radius: 0.55rem;
  border: 1px solid var(--home-border);
  background: var(--home-glass);
}
.home-memory__summary { font-size: 0.95em; }
.home-memory__meta {
  display: flex;
  gap: 0.5rem;
  flex-wrap: wrap;
  color: var(--home-fg-muted);
  font-size: 0.8em;
}
.home-memory__actions { display: flex; gap: 0.5rem; flex-wrap: wrap; }

.home-tagstrip {
  display: flex;
  flex-wrap: wrap;
  gap: 0.35rem;
}

@keyframes home-shimmer {
  0% { background-position: 200% 0; }
  100% { background-position: -200% 0; }
}
@keyframes home-progress-slide {
  0% { transform: translateX(-100%); }
  100% { transform: translateX(285%); }
}
@keyframes home-progress-stripe {
  0% { background-position: 0 0; }
  100% { background-position: 1rem 0; }
}

@media (prefers-reduced-motion: reduce) {
  .home-row,
  .home-button,
  .home-progress__bar,
  .home-skeleton__line,
  .home-progress--indeterminate .home-progress__bar,
  .home-progress__stripe {
    transition: none !important;
    animation: none !important;
  }
}
`;

let styleMounted = false;

export function HomeStyles(): ReactNode {
  const [mounted, setMounted] = useState(styleMounted);
  useEffect(() => {
    if (styleMounted) return;
    styleMounted = true;
    setMounted(true);
  }, []);
  if (!mounted) return null;
  return <style aria-hidden="true">{STYLE_TEXT}</style>;
}

export function SkeletonLine({
  width,
}: {
  width: "w-40" | "w-50" | "w-70" | "w-90";
}): ReactNode {
  return <div className={`home-skeleton__line home-skeleton__line--${width}`} aria-hidden="true" />;
}

export function SkeletonBlock({ lines }: { lines: number }): ReactNode {
  const widths: Array<"w-40" | "w-50" | "w-70" | "w-90"> = [
    "w-90",
    "w-70",
    "w-50",
    "w-40",
  ];
  const items = Array.from({ length: lines }, (_, i) => (
    <SkeletonLine key={i} width={widths[i % widths.length]} />
  ));
  return <div className="home-skeleton" aria-hidden="true">{items}</div>;
}

export type StatusTone =
  | "info"
  | "warning"
  | "error"
  | "locked"
  | "offline";

export function StatusBanner({
  tone,
  icon,
  title,
  detail,
  actions,
}: {
  tone: StatusTone;
  icon: string;
  title: string;
  detail?: string;
  actions?: ReactNode;
}): ReactNode {
  return (
    <div
      className={`home-status home-status--${tone}`}
      role="status"
      aria-live="polite"
    >
      <span className="home-status__icon" aria-hidden="true">{icon}</span>
      <div className="home-status__body">
        <span className="home-status__title">{title}</span>
        {detail ? <span className="home-status__detail">{detail}</span> : null}
        {actions ? <div className="home-status__actions">{actions}</div> : null}
      </div>
    </div>
  );
}

export function RetryButton({
  onClick,
  label,
}: {
  onClick: () => void;
  label: string;
}): ReactNode {
  return (
    <button
      type="button"
      className="home-button home-button--ghost"
      onClick={onClick}
    >
      {label}
    </button>
  );
}

export function LinkButton({
  to,
  navigate,
  children,
  variant = "primary",
  ariaLabel,
}: {
  to: string;
  navigate: (path: string) => void;
  children: ReactNode;
  variant?: "primary" | "ghost" | "danger" | "success";
  ariaLabel?: string;
}): ReactNode {
  return (
    <button
      type="button"
      className={`home-button home-button--${variant}`}
      onClick={() => navigate(to)}
      aria-label={ariaLabel}
    >
      {children}
    </button>
  );
}

export function Progress({
  value,
  indeterminate,
}: {
  value: number;
  indeterminate?: boolean;
}): ReactNode {
  const clamped = Math.max(0, Math.min(1, value));
  const pct = `${Math.round(clamped * 100)}%`;
  const cls = indeterminate
    ? "home-progress home-progress--indeterminate"
    : "home-progress";
  return (
    <div
      className={cls}
      role="progressbar"
      aria-valuemin={0}
      aria-valuemax={100}
      aria-valuenow={indeterminate ? undefined : Math.round(clamped * 100)}
      aria-label="task progress"
    >
      {indeterminate ? (
        <span className="home-progress__stripe" />
      ) : (
        <span className="home-progress__bar" style={{ width: pct }} />
      )}
    </div>
  );
}

export function progressStyle(): CSSProperties {
  return { position: "relative" };
}

export function formatRelative(iso: string): string {
  const then = Date.parse(iso);
  if (Number.isNaN(then)) return iso;
  const ms = Date.now() - then;
  const sec = Math.round(ms / 1000);
  if (sec < 45) return "just now";
  const min = Math.round(sec / 60);
  if (min < 60) return `${min} min ago`;
  const hr = Math.round(min / 60);
  if (hr < 24) return `${hr} hr ago`;
  const day = Math.round(hr / 24);
  if (day < 7) return `${day} day${day === 1 ? "" : "s"} ago`;
  const wk = Math.round(day / 7);
  if (wk < 5) return `${wk} wk ago`;
  return new Date(then).toLocaleDateString();
}

export function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes < 0) return "—";
  if (bytes < 1024) return `${bytes} B`;
  const units = ["KB", "MB", "GB", "TB"];
  let value = bytes / 1024;
  let i = 0;
  while (value >= 1024 && i < units.length - 1) {
    value /= 1024;
    i += 1;
  }
  return `${value.toFixed(value < 10 ? 1 : 0)} ${units[i]}`;
}

export function useOnlineSubscription(setOnline: (online: boolean) => void): void {
  useEffect(() => {
    const handleOnline = () => setOnline(true);
    const handleOffline = () => setOnline(false);
    window.addEventListener("online", handleOnline);
    window.addEventListener("offline", handleOffline);
    setOnline(navigator.onLine);
    return () => {
      window.removeEventListener("online", handleOnline);
      window.removeEventListener("offline", handleOffline);
    };
  }, [setOnline]);
}

export type { ServiceError };
