import { type CSSProperties } from "react";
import type { AnnotationRecord, Ulid } from "../../services/annotationService";
import {
  InlineStatus,
  resolveHighlightColor,
  resolutionStateText,
} from "./sharedUi";

export interface AnnotationListItemProps {
  annotation: AnnotationRecord;
  selected: boolean;
  onSelect: (annotationId: Ulid) => void;
}

export function AnnotationListItem({
  annotation,
  selected,
  onSelect,
}: AnnotationListItemProps): JSX.Element {
  const preview =
    annotation.bodyText?.trim() ||
    annotation.selector.textQuote?.exact?.trim() ||
    annotation.selector.capturedContext.trim();
  const state = resolutionStateText(annotation.resolution.state);
  const icon = state.tone === "success" ? "✓" : state.tone === "danger" ? "⊗" : "!";
  const color = resolveHighlightColor(annotation.style.colorToken);
  return (
    <li
      role="listitem"
      style={{
        ...listItemStyle,
        borderColor: selected
          ? "var(--ez-color-border-focus)"
          : "var(--ez-color-border-subtle)",
        background: selected
          ? "rgba(124, 140, 255, 0.08)"
          : "var(--ez-color-bg-elevated)",
      }}
    >
      <button
        type="button"
        onClick={() => onSelect(annotation.annotationId)}
        aria-pressed={selected}
        aria-label={`Open annotation ${preview ? `: ${preview.slice(0, 80)}` : annotation.annotationId}`}
        style={buttonStyle}
      >
        <span
          aria-hidden="true"
          style={{
            ...swatchStyle,
            background: color,
            opacity: annotation.resolution.state === "orphaned" ? 0.55 : 1,
          }}
        />
        <span style={contentStyle}>
          <span style={previewStyle}>
            {preview ? preview : annotationLabel(annotation.annotationType)}
          </span>
          <span style={metaStyle}>
            <InlineStatus tone={state.tone} icon={icon}>
              <span>{state.label}</span>
            </InlineStatus>
            {annotation.tags.length > 0 ? (
              <span aria-hidden="true" style={tagStyle}>
                {annotation.tags
                  .map((tag) => tag.name)
                  .slice(0, 3)
                  .join(" · ")}
                {annotation.tags.length > 3
                  ? ` +${annotation.tags.length - 3}`
                  : ""}
              </span>
            ) : null}
            <time
              dateTime={new Date(annotation.updatedAtMs).toISOString()}
              style={timeStyle}
            >
              {formatTimestamp(annotation.updatedAtMs)}
            </time>
          </span>
        </span>
      </button>
    </li>
  );
}

function annotationLabel(
  type: AnnotationRecord["annotationType"],
): string {
  switch (type) {
    case "highlight":
      return "Highlight";
    case "comment":
      return "Comment";
    case "ink":
      return "Ink annotation";
    case "voice":
      return "Voice annotation";
    case "bookmark_note":
      return "Bookmark note";
  }
}

function formatTimestamp(ms: number): string {
  if (!Number.isFinite(ms) || ms <= 0) {
    return "Unknown date";
  }
  try {
    const date = new Date(ms);
    return date.toLocaleString(undefined, {
      year: "numeric",
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  } catch {
    return "Unknown date";
  }
}

const listItemStyle: CSSProperties = {
  listStyle: "none",
  borderRadius: "var(--ez-radius-sm)",
  border: "1px solid var(--ez-color-border-subtle)",
  padding: 0,
  overflow: "hidden",
};

const buttonStyle: CSSProperties = {
  display: "flex",
  alignItems: "flex-start",
  gap: "var(--ez-space-3)",
  padding: "var(--ez-space-3)",
  background: "transparent",
  color: "inherit",
  border: "none",
  width: "100%",
  textAlign: "left",
  cursor: "pointer",
  fontFamily: "inherit",
  fontSize: "1rem",
};

const swatchStyle: CSSProperties = {
  width: "0.4rem",
  alignSelf: "stretch",
  minHeight: "2.5rem",
  borderRadius: "2px",
  flexShrink: 0,
};

const contentStyle: CSSProperties = {
  display: "flex",
  flexDirection: "column",
  gap: "var(--ez-space-1)",
  minWidth: 0,
  flex: 1,
};

const previewStyle: CSSProperties = {
  whiteSpace: "nowrap",
  overflow: "hidden",
  textOverflow: "ellipsis",
  color: "var(--ez-color-text-primary)",
  fontFamily: "var(--ez-font-reader)",
};

const metaStyle: CSSProperties = {
  display: "flex",
  alignItems: "center",
  gap: "var(--ez-space-3)",
  flexWrap: "wrap",
  color: "var(--ez-color-text-muted)",
  fontSize: "0.8rem",
};

const tagStyle: CSSProperties = {
  fontFamily: "var(--ez-font-mono)",
  color: "var(--ez-color-text-secondary)",
};

const timeStyle: CSSProperties = {
  fontFamily: "var(--ez-font-mono)",
};
