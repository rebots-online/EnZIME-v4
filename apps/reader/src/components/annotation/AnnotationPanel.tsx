import { type CSSProperties } from "react";
import type { AnnotationRecord, Ulid } from "../../services/annotationService";
import {
  InlineStatus,
  EzIconButton,
  resolutionStateText,
  useReducedMotion,
} from "./sharedUi";

export interface AnnotationPanelProps {
  annotations: AnnotationRecord[];
  selectedAnnotationId?: Ulid;
  onSelect: (annotationId: Ulid) => void;
  onDelete: (annotationId: Ulid, revision: number) => void;
  onRestore?: (annotationId: Ulid) => void;
}

const PANEL_STYLE: CSSProperties = {
  background: "var(--ez-color-bg-glass)",
  borderRadius: "var(--ez-radius-md)",
  padding: "var(--ez-space-4)",
  display: "flex",
  flexDirection: "column",
  gap: "var(--ez-space-3)",
  maxHeight: "60vh",
  overflowY: "auto",
};

export function AnnotationPanel({
  annotations,
  selectedAnnotationId,
  onSelect,
  onDelete,
  onRestore,
}: AnnotationPanelProps): JSX.Element {
  if (annotations.length === 0) {
    return (
      <section
        aria-labelledby="ez-annotation-panel-heading"
        style={PANEL_STYLE}
      >
        <h3 id="ez-annotation-panel-heading" style={headingStyle}>
          Annotations on this article
        </h3>
        <p style={{ margin: 0, color: "var(--ez-color-text-secondary)" }}>
          No annotations yet. Select text, ink, or voice from the toolbar to
          create the first one.
        </p>
      </section>
    );
  }
  return (
    <section
      aria-labelledby="ez-annotation-panel-heading"
      style={PANEL_STYLE}
    >
      <h3 id="ez-annotation-panel-heading" style={headingStyle}>
        Annotations on this article ({annotations.length})
      </h3>
      <ul role="list" style={listStyle}>
        {annotations.map((annotation) => {
          const isSelected = annotation.annotationId === selectedAnnotationId;
          return (
            <li
              key={annotation.annotationId}
              style={{
                ...itemStyle,
                borderColor: isSelected
                  ? "var(--ez-color-border-focus)"
                  : "var(--ez-color-border-subtle)",
              }}
            >
              <button
                type="button"
                onClick={() => onSelect(annotation.annotationId)}
                aria-pressed={isSelected}
                aria-label={`Open annotation ${annotation.annotationId}: ${previewText(
                  annotation,
                )}`}
                style={rowButtonStyle(useReducedMotion())}
              >
                <span style={previewStyle}>
                  {previewText(annotation) || "Annotation"}
                </span>
                <ResolutionBadge state={annotation.resolution.state} />
              </button>
              <div style={actionsStyle}>
                {onRestore && annotation.resolution.state === "orphaned" ? (
                  <EzIconButton
                    ariaLabel="Restore annotation from history"
                    onClick={() => onRestore(annotation.annotationId)}
                    title="Restore"
                  >
                    ↺
                  </EzIconButton>
                ) : null}
                <EzIconButton
                  ariaLabel={`Delete annotation ${annotation.annotationId}`}
                  onClick={() =>
                    onDelete(annotation.annotationId, annotation.revision)
                  }
                  title="Delete"
                >
                  🗑
                </EzIconButton>
              </div>
            </li>
          );
        })}
      </ul>
    </section>
  );
}

function ResolutionBadge({
  state,
}: {
  state: AnnotationRecord["resolution"]["state"];
}): JSX.Element {
  const { label, tone } = resolutionStateText(state);
  const icon = tone === "success" ? "✓" : tone === "danger" ? "⊗" : "!";
  return (
    <InlineStatus tone={tone} icon={icon}>
      <span>{label}</span>
    </InlineStatus>
  );
}

const headingStyle: CSSProperties = {
  margin: 0,
  fontSize: "0.95rem",
  letterSpacing: "0.04em",
  textTransform: "uppercase",
  color: "var(--ez-color-text-secondary)",
};

const listStyle: CSSProperties = {
  listStyle: "none",
  margin: 0,
  padding: 0,
  display: "flex",
  flexDirection: "column",
  gap: "var(--ez-space-2)",
};

const itemStyle: CSSProperties = {
  background: "var(--ez-color-bg-elevated)",
  border: "1px solid var(--ez-color-border-subtle)",
  borderRadius: "var(--ez-radius-sm)",
  padding: "var(--ez-space-3)",
  display: "flex",
  alignItems: "flex-start",
  justifyContent: "space-between",
  gap: "var(--ez-space-2)",
};

function rowButtonStyle(reduced: boolean): CSSProperties {
  return {
    flex: 1,
    minWidth: 0,
    background: "transparent",
    border: "none",
    color: "inherit",
    textAlign: "left",
    cursor: "pointer",
    padding: 0,
    fontFamily: "inherit",
    font: "inherit",
    display: "flex",
    flexDirection: "column",
    gap: "var(--ez-space-2)",
    transition: reduced ? "none" : "color var(--ez-duration-fast)",
  };
}

const previewStyle: CSSProperties = {
  whiteSpace: "nowrap",
  overflow: "hidden",
  textOverflow: "ellipsis",
};

const actionsStyle: CSSProperties = {
  display: "inline-flex",
  gap: "var(--ez-space-1)",
};

function previewText(annotation: AnnotationRecord): string {
  return (
    annotation.bodyText?.trim() ||
    annotation.selector.textQuote?.exact?.trim() ||
    annotation.selector.capturedContext.trim() ||
    annotation.annotationType
  );
}
