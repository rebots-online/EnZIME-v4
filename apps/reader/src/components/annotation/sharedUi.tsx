import { useEffect, useState, type CSSProperties, type ReactNode } from "react";
import type {
  AppErrorDto,
  ErrorActionDto,
} from "../../services/annotationService";

const SPIN_KEYFRAME_ID = "ez-annotation-motion-keyframes";

function ensureMotionKeyframes(): void {
  if (typeof document === "undefined") {
    return;
  }
  if (document.getElementById(SPIN_KEYFRAME_ID)) {
    return;
  }
  const style = document.createElement("style");
  style.id = SPIN_KEYFRAME_ID;
  style.textContent = `@keyframes ez-spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }`;
  document.head.appendChild(style);
}

declare global {
  interface Window {
    ezReduceMotion?: boolean;
  }
}

export interface ResolvedColorTokens {
  highlightYellow: string;
  highlightMint: string;
  highlightPink: string;
  highlightBlue: string;
}

export function resolveHighlightColor(token: string): string {
  switch (token) {
    case "highlight-mint":
      return "var(--ez-color-highlight-mint)";
    case "highlight-pink":
      return "var(--ez-color-highlight-pink)";
    case "highlight-blue":
      return "var(--ez-color-highlight-blue)";
    case "highlight-yellow":
    default:
      return "var(--ez-color-highlight-yellow)";
  }
}

export function trustStateText(
  state: "local" | "verified" | "unverified" | "rejected",
): { label: string; tone: Tone } {
  switch (state) {
    case "verified":
      return { label: "Verified layer", tone: "success" };
    case "local":
      return { label: "Local layer", tone: "info" };
    case "unverified":
      return { label: "Unverified layer", tone: "warning" };
    case "rejected":
      return { label: "Rejected signature", tone: "danger" };
  }
}

export function signatureStateText(
  state: "verified" | "unsigned" | "invalid" | "unknown_key",
): { label: string; tone: Tone } {
  switch (state) {
    case "verified":
      return { label: "Signature verified", tone: "success" };
    case "unsigned":
      return { label: "Unsigned artifact", tone: "warning" };
    case "invalid":
      return { label: "Signature invalid", tone: "danger" };
    case "unknown_key":
      return { label: "Unknown signer key", tone: "warning" };
  }
}

export function resolutionStateText(
  state: "resolved" | "ambiguous" | "source_unavailable" | "orphaned",
): { label: string; tone: Tone } {
  switch (state) {
    case "resolved":
      return { label: "Resolved to current source", tone: "success" };
    case "ambiguous":
      return { label: "Anchor is ambiguous", tone: "warning" };
    case "source_unavailable":
      return {
        label: "Source unavailable; showing captured text",
        tone: "warning",
      };
    case "orphaned":
      return { label: "Source removed; annotation retained", tone: "danger" };
  }
}

export type Tone =
  | "info"
  | "success"
  | "warning"
  | "danger"
  | "neutral";

function toneColor(tone: Tone): string {
  switch (tone) {
    case "success":
      return "var(--ez-color-success)";
    case "warning":
      return "var(--ez-color-warning)";
    case "danger":
      return "var(--ez-color-danger)";
    case "info":
      return "var(--ez-color-accent)";
    case "neutral":
      return "var(--ez-color-text-muted)";
  }
}

export interface SpinnerProps {
  readonly label: string;
  readonly className?: string;
}

export function Spinner({ label }: SpinnerProps): JSX.Element {
  const reduced = useReducedMotion();
  useEffect(() => {
    ensureMotionKeyframes();
  }, []);
  const style: CSSProperties = {
    display: "inline-block",
    width: "1rem",
    height: "1rem",
    borderRadius: "50%",
    borderWidth: "2px",
    borderStyle: "solid",
    borderColor: "var(--ez-color-border-subtle)",
    borderTopColor: "var(--ez-color-primary)",
    animation: reduced ? "none" : "ez-spin 0.9s linear infinite",
    marginInlineEnd: "var(--ez-space-2)",
  };
  return (
    <span
      role="status"
      aria-live="polite"
      style={{ display: "inline-flex", alignItems: "center" }}
    >
      <span style={style} aria-hidden="true" />
      <span>{label}</span>
    </span>
  );
}

export interface InlineStatusProps {
  readonly tone: Tone;
  readonly icon: ReactNode;
  readonly children: ReactNode;
}

export function InlineStatus({ tone, icon, children }: InlineStatusProps): JSX.Element {
  const color = toneColor(tone);
  return (
    <span
      style={{
        display: "inline-flex",
        alignItems: "center",
        gap: "var(--ez-space-2)",
        color,
      }}
    >
      <span aria-hidden="true" style={{ fontSize: "1rem", lineHeight: 1 }}>
        {icon}
      </span>
      <span>{children}</span>
    </span>
  );
}

export interface ErrorBoundaryStateProps {
  readonly error: AppErrorDto;
  readonly onRetry?: () => void;
  readonly correlationLabel?: string;
}

export function ErrorBoundaryState({
  error,
  onRetry,
  correlationLabel,
}: ErrorBoundaryStateProps): JSX.Element {
  const tone: Tone = error.code === "offline" ? "info" : "danger";
  return (
    <section
      role="alert"
      style={panelStyle}
      aria-labelledby="ez-error-title"
    >
      <h3 id="ez-error-title" style={{ margin: 0, fontSize: "1.1rem" }}>
        {error.code === "offline"
          ? "Annotation store is offline"
          : "Annotations could not load"}
      </h3>
      <p style={{ margin: 0, color: "var(--ez-color-text-secondary)" }}>
        {error.message}
      </p>
      <p
        style={{
          margin: 0,
          fontFamily: "var(--ez-font-mono)",
          fontSize: "0.8rem",
          color: "var(--ez-color-text-muted)",
        }}
      >
        {correlationLabel ?? "Correlation"}: {error.correlationId} · code:{" "}
        {error.code}
      </p>
      <div
        style={{
          display: "flex",
          gap: "var(--ez-space-3)",
          flexWrap: "wrap",
          marginTop: "var(--ez-space-3)",
        }}
      >
        {onRetry ? (
          <EzButton onClick={onRetry}>Retry</EzButton>
        ) : null}
        {error.action ? (
          <ErrorActionLink action={error.action} />
        ) : null}
      </div>
      <InlineStatus tone={tone} icon="!">
        <span aria-hidden="false">{tone === "info" ? "Offline" : "Error"}</span>
      </InlineStatus>
    </section>
  );
}

function ErrorActionLink({ action }: { action: ErrorActionDto }): JSX.Element {
  return (
    <span
      style={{
        color: "var(--ez-color-primary)",
        fontStyle: "italic",
      }}
      aria-label={`Suggested action: ${action.label}`}
    >
      Suggested: {action.label}
    </span>
  );
}

export interface EmptyStateProps {
  readonly title: string;
  readonly body: string;
  readonly actionLabel: string;
  readonly onAction: () => void;
  readonly hint?: string;
}

export function EmptyState({
  title,
  body,
  actionLabel,
  onAction,
  hint,
}: EmptyStateProps): JSX.Element {
  return (
    <section
      style={panelStyle}
      aria-labelledby="ez-empty-title"
    >
      <h3 id="ez-empty-title" style={{ margin: 0, fontSize: "1.1rem" }}>
        {title}
      </h3>
      <p style={{ margin: 0, color: "var(--ez-color-text-secondary)" }}>
        {body}
      </p>
      {hint ? (
        <p
          style={{
            margin: 0,
            color: "var(--ez-color-text-muted)",
            fontSize: "0.875rem",
          }}
        >
          {hint}
        </p>
      ) : null}
      <div style={{ marginTop: "var(--ez-space-3)" }}>
        <EzButton onClick={onAction}>{actionLabel}</EzButton>
      </div>
    </section>
  );
}

export interface CapabilityDeniedStateProps {
  readonly message: string;
}

export function CapabilityDeniedState({
  message,
}: CapabilityDeniedStateProps): JSX.Element {
  return (
    <section
      role="status"
      aria-live="polite"
      style={panelStyle}
      aria-labelledby="ez-denied-title"
    >
      <h3 id="ez-denied-title" style={{ margin: 0, fontSize: "1.1rem" }}>
        Capability not granted
      </h3>
      <p style={{ margin: 0, color: "var(--ez-color-text-secondary)" }}>
        {message}
      </p>
      <InlineStatus tone="warning" icon="⊗">
        <span>Capability denied</span>
      </InlineStatus>
    </section>
  );
}

export interface OfflineStateProps {
  readonly message: string;
  readonly onRetry: () => void;
}

export function OfflineState({ message, onRetry }: OfflineStateProps): JSX.Element {
  return (
    <section
      role="status"
      aria-live="polite"
      style={panelStyle}
      aria-labelledby="ez-offline-title"
    >
      <h3 id="ez-offline-title" style={{ margin: 0, fontSize: "1.1rem" }}>
        Offline
      </h3>
      <p style={{ margin: 0, color: "var(--ez-color-text-secondary)" }}>
        {message}
      </p>
      <InlineStatus tone="info" icon="○">
        <span>Connection unavailable</span>
      </InlineStatus>
      <div style={{ marginTop: "var(--ez-space-3)" }}>
        <EzButton onClick={onRetry}>Retry now</EzButton>
      </div>
    </section>
  );
}

export interface EzButtonProps {
  readonly children: ReactNode;
  readonly onClick?: () => void;
  readonly variant?: "primary" | "ghost" | "danger";
  readonly type?: "button" | "submit";
  readonly disabled?: boolean;
  readonly ariaLabel?: string;
  readonly ariaPressed?: boolean;
  readonly title?: string;
  readonly style?: CSSProperties;
}

export function EzButton({
  children,
  onClick,
  variant = "primary",
  type = "button",
  disabled,
  ariaLabel,
  ariaPressed,
  title,
  style,
}: EzButtonProps): JSX.Element {
  const background = disabled
    ? "rgba(124, 140, 255, 0.18)"
    : variant === "primary"
      ? "var(--ez-color-primary)"
      : variant === "danger"
        ? "var(--ez-color-danger)"
        : "transparent";
  const color = disabled
    ? "var(--ez-color-text-muted)"
    : variant === "ghost"
      ? "var(--ez-color-text-primary)"
      : "#0a0c14";
  const border =
    variant === "ghost"
      ? "1px solid var(--ez-color-border-subtle)"
      : "1px solid transparent";
  return (
    <button
      type={type}
      onClick={onClick}
      disabled={disabled}
      aria-label={ariaLabel}
      aria-pressed={ariaPressed}
      title={title}
      style={{
        background,
        color,
        border,
        borderRadius: "var(--ez-radius-sm)",
        padding: "var(--ez-space-2) var(--ez-space-4)",
        fontFamily: "var(--ez-font-ui)",
        fontWeight: 600,
        cursor: disabled ? "not-allowed" : "pointer",
        transition: useReducedMotion() ? "none" : "background var(--ez-duration-fast)",
        ...style,
      }}
    >
      {children}
    </button>
  );
}

export function EzIconButton({
  ariaLabel,
  onClick,
  children,
  pressed,
  disabled,
  title,
  style,
}: {
  ariaLabel: string;
  onClick?: () => void;
  children: ReactNode;
  pressed?: boolean;
  disabled?: boolean;
  title?: string;
  style?: CSSProperties;
}): JSX.Element {
  return (
    <button
      type="button"
      aria-label={ariaLabel}
      title={title ?? ariaLabel}
      onClick={onClick}
      disabled={disabled}
      aria-pressed={pressed}
      style={{
        background: "transparent",
        color: pressed ? "var(--ez-color-primary)" : "var(--ez-color-text-secondary)",
        border: "1px solid var(--ez-color-border-subtle)",
        borderRadius: "var(--ez-radius-sm)",
        width: "2.25rem",
        height: "2.25rem",
        display: "inline-flex",
        alignItems: "center",
        justifyContent: "center",
        cursor: disabled ? "not-allowed" : "pointer",
        transition: useReducedMotion() ? "none" : "color var(--ez-duration-fast)",
        ...style,
      }}
    >
      {children}
    </button>
  );
}

export const panelStyle: CSSProperties = {
  background: "var(--ez-color-bg-glass)",
  borderRadius: "var(--ez-radius-md)",
  boxShadow: "var(--ez-shadow-panel)",
  padding: "var(--ez-space-5)",
  display: "flex",
  flexDirection: "column",
  gap: "var(--ez-space-2)",
};

export function useReducedMotion(): boolean {
  const media = "(prefers-reduced-motion: reduce)";
  const [reduced, setReduced] = useState(
    () =>
      (typeof window !== "undefined" && window.matchMedia
        ? window.matchMedia(media).matches
        : false) ||
      (typeof window !== "undefined" && window.ezReduceMotion === true),
  );
  useEffect(() => {
    if (typeof window === "undefined" || !window.matchMedia) {
      return;
    }
    const query = window.matchMedia(media);
    const handler = (event: MediaQueryListEvent) => {
      setReduced(event.matches || window.ezReduceMotion === true);
    };
    query.addEventListener("change", handler);
    return () => {
      query.removeEventListener("change", handler);
    };
  }, []);
  return reduced;
}
