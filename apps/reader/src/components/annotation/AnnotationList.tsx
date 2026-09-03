import type { JSX } from "react";
import { type CSSProperties, useId } from "react";
import type { AnnotationRecord, Ulid } from "../../services/annotationService";
import { AnnotationListItem } from "./AnnotationListItem";

export interface AnnotationListProps {
  annotations: AnnotationRecord[];
  selectedAnnotationId?: Ulid;
  onSelect: (annotationId: Ulid) => void;
}

export function AnnotationList({
  annotations,
  selectedAnnotationId,
  onSelect,
}: AnnotationListProps): JSX.Element {
  const headingId = useId();
  if (annotations.length === 0) {
    return (
      <section
        aria-labelledby={headingId}
        style={sectionStyle}
      >
        <h3 id={headingId} style={headingStyle}>
          Annotations
        </h3>
        <p style={{ margin: 0, color: "var(--ez-color-text-secondary)" }}>
          No annotations match the current filter. Adjust the search, tags, or
          layer toggles, or open an article and create one.
        </p>
      </section>
    );
  }
  return (
    <section aria-labelledby={headingId} style={sectionStyle}>
      <h3 id={headingId} style={headingStyle}>
        Annotations ({annotations.length})
      </h3>
      <ul role="list" style={listStyle}>
        {annotations.map((annotation) => (
          <AnnotationListItem
            key={annotation.annotationId}
            annotation={annotation}
            selected={annotation.annotationId === selectedAnnotationId}
            onSelect={onSelect}
          />
        ))}
      </ul>
    </section>
  );
}

const sectionStyle: CSSProperties = {
  background: "var(--ez-color-bg-glass)",
  borderRadius: "var(--ez-radius-md)",
  padding: "var(--ez-space-4)",
  display: "flex",
  flexDirection: "column",
  gap: "var(--ez-space-3)",
};

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
