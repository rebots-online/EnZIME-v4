import type { JSX } from "react";
import { useEffect, useId, useState, type CSSProperties } from "react";
import type {
  AnnotationDraft,
  AnnotationRecord,
  AnnotationResolution,
  AnnotationStyle,
  SelectorBundle,
  Ulid,
} from "../../services/annotationService";
import {
  EzButton,
  EzIconButton,
  InlineStatus,
  resolveHighlightColor,
  resolutionStateText,
} from "./sharedUi";

export interface AnnotationDetailPanelProps {
  annotation?: AnnotationRecord;
  availableTags: { tagId: Ulid; name: string; colorToken: string }[];
  onUpdate?: (
    annotationId: Ulid,
    expectedRevision: number,
    draft: AnnotationDraft,
  ) => void;
  onDelete?: (annotationId: Ulid, expectedRevision: number) => void;
  onOpenSource?: (annotationId: Ulid) => void;
}

export function AnnotationDetailPanel(
  props: AnnotationDetailPanelProps,
): JSX.Element {
  const headingId = useId();
  const annotation = props.annotation;
  const [bodyText, setBodyText] = useState<string>(annotation?.bodyText ?? "");
  const [colorToken, setColorToken] = useState<string>(
    annotation?.style.colorToken ?? "highlight-yellow",
  );
  const [tagIds, setTagIds] = useState<Ulid[]>(
    annotation?.tags.map((tag) => tag.tagId) ?? [],
  );
  const [editing, setEditing] = useState(false);

  useEffect(() => {
    setBodyText(annotation?.bodyText ?? "");
    setColorToken(annotation?.style.colorToken ?? "highlight-yellow");
    setTagIds(annotation?.tags.map((tag) => tag.tagId) ?? []);
    setEditing(false);
  }, [
    annotation?.annotationId,
    annotation?.bodyText,
    annotation?.style.colorToken,
    annotation?.tags,
  ]);

  if (!annotation) {
    return (
      <section aria-labelledby={headingId} style={panelStyle}>
        <h3 id={headingId} style={headingStyle}>
          Annotation detail
        </h3>
        <p style={{margin: 0, color: "var(--ez-color-text-muted)"}}>
          Select an annotation from the list to see its captured passage, edit
          its body, or open its source.
        </p>
      </section>
    );
  }

  const resolution = resolutionStateText(annotation.resolution.state);
  const resolutionIcon =
    resolution.tone === "success"
      ? "✓"
      : resolution.tone === "danger"
        ? "⊗"
        : "!";

  const commitEdit = () => {
    if (!props.onUpdate) {
      setEditing(false);
      return;
    }
    const style: AnnotationStyle = {
      colorToken,
      emphasis: annotation.style.emphasis,
      strokeWidth: annotation.style.strokeWidth,
    };
    const selector: SelectorBundle = annotation.selector;
    const draft: AnnotationDraft = {
      annotationType: annotation.annotationType,
      selector,
      bodyText: bodyText.trim().length > 0 ? bodyText.trim() : undefined,
      style,
      notebookId: annotation.notebookId,
      tagIds,
      sidecarLayerId: annotation.sidecarLayerId,
    };
    props.onUpdate(annotation.annotationId, annotation.revision, draft);
    setEditing(false);
  };

  const toggleTag = (tagId: Ulid) => {
    if (tagIds.includes(tagId)) {
      setTagIds(tagIds.filter((id) => id !== tagId));
    } else {
      setTagIds([...tagIds, tagId]);
    }
  };

  return (
    <section aria-labelledby={headingId} style={panelStyle}>
      <header style={headerStyle}>
        <h3 id={headingId} style={headingStyle}>
          Annotation detail
        </h3>
        <div style={{display: "inline-flex", gap: "var(--ez-space-2)"}}>
          {props.onOpenSource ? (
            <EzIconButton
              ariaLabel="Open source passage for this annotation"
              onClick={() => props.onOpenSource?.(annotation.annotationId)}
              title="Open source"
            >
              ↗
            </EzIconButton>
          ) : null}
          {props.onUpdate ? (
            <EzIconButton
              ariaLabel={editing ? "Save edits" : "Edit annotation"}
              onClick={() => (editing ? commitEdit() : setEditing(true))}
              title={editing ? "Save" : "Edit"}
            >
              {editing ? "⏎" : "✎"}
            </EzIconButton>
          ) : null}
          {props.onDelete ? (
            <EzIconButton
              ariaLabel="Delete annotation"
              onClick={() =>
                props.onDelete?.(annotation.annotationId, annotation.revision)
              }
              title="Delete"
            >
              🗑
            </EzIconButton>
          ) : null}
        </div>
      </header>
      <ResolutionState
        resolution={annotation.resolution}
        tone={resolution.tone}
        icon={resolutionIcon}
        label={resolution.label}
      />
      <PassageBlock annotation={annotation} />
      {editing ? (
        <EditorBody
          bodyText={bodyText}
          onBodyTextChange={setBodyText}
          colorToken={colorToken}
          onColorTokenChange={setColorToken}
          tagIds={tagIds}
          onToggleTag={toggleTag}
          availableTags={props.availableTags}
          onSave={commitEdit}
          onCancel={() => setEditing(false)}
          annotationType={annotation.annotationType}
        />
      ) : (
        <ReadonlyBody annotation={annotation} />
      )}
    </section>
  );
}

function ResolutionState({
  resolution,
  tone,
  icon,
  label,
}: {
  resolution: AnnotationResolution;
  tone: "info" | "success" | "warning" | "danger" | "neutral";
  icon: string;
  label: string;
}): JSX.Element {
  return (
    <div
      style={{
        background: "var(--ez-color-bg-elevated)",
        borderRadius: "var(--ez-radius-sm)",
        padding: "var(--ez-space-3)",
        display: "flex",
        flexDirection: "column",
        gap: "var(--ez-space-1)",
      }}
    >
      <InlineStatus tone={tone} icon={icon}>
        <span>{label}</span>
      </InlineStatus>
      <span
        style={{
          color: "var(--ez-color-text-secondary)",
          fontFamily: "var(--ez-font-mono)",
          fontSize: "0.8rem",
        }}
      >
        Confidence {Math.round(resolution.confidence * 100)}%
      </span>
      <p
        style={{
          margin: 0,
          color: "var(--ez-color-text-secondary)",
          fontSize: "0.9rem",
        }}
      >
        {resolution.explanation}
      </p>
    </div>
  );
}

function PassageBlock({
  annotation,
}: {
  annotation: AnnotationRecord;
}): JSX.Element {
  const quote = annotation.selector.textQuote?.exact;
  const captured = annotation.selector.capturedContext.trim();
  const pack = annotation.selector.article.packId;
  const articleKey = annotation.selector.article.articleKey;
  return (
    <div
      style={{
        borderLeft: `3px solid ${resolveHighlightColor(annotation.style.colorToken)}`,
        paddingInlineStart: "var(--ez-space-3)",
        display: "flex",
        flexDirection: "column",
        gap: "var(--ez-space-1)",
      }}
    >
      {quote ? (
        <blockquote
          style={{
            margin: 0,
            fontFamily: "var(--ez-font-reader)",
            color: "var(--ez-color-text-primary)",
            whiteSpace: "pre-wrap",
          }}
        >
          “{quote}”
        </blockquote>
      ) : null}
      <p
        style={{
          margin: 0,
          color: "var(--ez-color-text-muted)",
          fontSize: "0.85rem",
        }}
      >
        Source pack {pack} · article {articleKey}
      </p>
      {annotation.selector.textPosition ? (
        <p
          style={{
            margin: 0,
            color: "var(--ez-color-text-muted)",
            fontFamily: "var(--ez-font-mono)",
            fontSize: "0.8rem",
          }}
        >
          Normalised span {annotation.selector.textPosition.normalizedStart}–
          {annotation.selector.textPosition.normalizedEnd}
        </p>
      ) : null}
      {captured ? (
        <p
          style={{
            margin: 0,
            color: "var(--ez-color-text-secondary)",
            fontStyle: "italic",
          }}
        >
          Captured context: {captured}
        </p>
      ) : null}
    </div>
  );
}

function ReadonlyBody({
  annotation,
}: {
  annotation: AnnotationRecord;
}): JSX.Element {
  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        gap: "var(--ez-space-2)",
      }}
    >
      {annotation.bodyText ? (
        <p
          style={{
            margin: 0,
            color: "var(--ez-color-text-primary)",
            whiteSpace: "pre-wrap",
          }}
        >
          {annotation.bodyText}
        </p>
      ) : (
        <p style={{margin: 0, color: "var(--ez-color-text-muted)"}}>
          No body text. Use Edit to add a note.
        </p>
      )}
      {annotation.tags.length > 0 ? (
        <ul role="list" style={tagListStyle}>
          {annotation.tags.map((tag) => (
            <li key={tag.tagId} style={tagChipStyle(tag.colorToken)}>
              {tag.name}
            </li>
          ))}
        </ul>
      ) : null}
      <p
        style={{
          margin: 0,
          color: "var(--ez-color-text-muted)",
          fontFamily: "var(--ez-font-mono)",
          fontSize: "0.8rem",
        }}
      >
        Layer {annotation.sidecarLayerId} · revision {annotation.revision} · updated{" "}
        {formatTimestamp(annotation.updatedAtMs)}
      </p>
    </div>
  );
}

function EditorBody(props: {
  bodyText: string;
  onBodyTextChange: (text: string) => void;
  colorToken: string;
  onColorTokenChange: (token: string) => void;
  tagIds: Ulid[];
  onToggleTag: (tagId: Ulid) => void;
  availableTags: { tagId: Ulid; name: string; colorToken: string }[];
  onSave: () => void;
  onCancel: () => void;
  annotationType: AnnotationRecord["annotationType"];
}): JSX.Element {
  const textareaId = useId();
  return (
    <form
      style={{display: "flex", flexDirection: "column", gap: "var(--ez-space-3)"}}
      onSubmit={(event) => {
        event.preventDefault();
        props.onSave();
      }}
    >
      <label htmlFor={textareaId} style={{fontSize: "0.85rem", color: "var(--ez-color-text-secondary)"}}>
        Body
      </label>
      <textarea
        id={textareaId}
        value={props.bodyText}
        onChange={(event) => props.onBodyTextChange(event.target.value)}
        rows={4}
        style={textareaStyle}
        placeholder="Add a note to this annotation…"
      />
      <div role="radiogroup" aria-label="Highlight colour" style={{display: "inline-flex", gap: "var(--ez-space-2)", flexWrap: "wrap"}}>
        {[
          "highlight-yellow",
          "highlight-mint",
          "highlight-pink",
          "highlight-blue",
        ].map((token) => (
          <label
            key={token}
            style={{display: "inline-flex", alignItems: "center", gap: "var(--ez-space-1)"}}
          >
            <input
              type="radio"
              name="annotation-color"
              value={token}
              checked={props.colorToken === token}
              onChange={() => props.onColorTokenChange(token)}
              aria-label={`Use colour ${token}`}
              style={{
                appearance: "none",
                width: "1rem",
                height: "1rem",
                borderRadius: "50%",
                border:
                  props.colorToken === token
                    ? "2px solid var(--ez-color-border-focus)"
                    : "2px solid var(--ez-color-border-subtle)",
                background: resolveHighlightColor(token),
                margin: 0,
              }}
            />
            <span style={{fontSize: "0.8rem", color: "var(--ez-color-text-secondary)"}}>
              {token.replace(/^highlight-/, "")}
            </span>
          </label>
        ))}
      </div>
      <fieldset style={{border: "1px solid var(--ez-color-border-subtle)", borderRadius: "var(--ez-radius-sm)", padding: "var(--ez-space-2)"}}>
        <legend style={{paddingInline: "var(--ez-space-2)", fontSize: "0.85rem"}}>
          Tags
        </legend>
        {props.availableTags.length === 0 ? (
          <p style={{margin: 0, color: "var(--ez-color-text-muted)"}}>
            No tags exist yet. Create one from the toolbar to start labelling
            annotations.
          </p>
        ) : (
          <ul role="list" style={{listStyle: "none", margin: 0, padding: 0, display: "flex", gap: "var(--ez-space-2)", flexWrap: "wrap"}}>
            {props.availableTags.map((tag) => {
              const active = props.tagIds.includes(tag.tagId);
              return (
                <li key={tag.tagId}>
                  <label
                    style={{
                      display: "inline-flex",
                      alignItems: "center",
                      gap: "var(--ez-space-1)",
                      padding: "var(--ez-space-1) var(--ez-space-2)",
                      borderRadius: "var(--ez-radius-sm)",
                      border: active
                        ? "1px solid var(--ez-color-border-focus)"
                        : "1px solid var(--ez-color-border-subtle)",
                      background: active ? "rgba(124, 140, 255, 0.12)" : "transparent",
                      cursor: "pointer",
                      fontSize: "0.85rem",
                    }}
                  >
                    <input
                      type="checkbox"
                      checked={active}
                      onChange={() => props.onToggleTag(tag.tagId)}
                      aria-label={`${active ? "Remove" : "Add"} tag ${tag.name}`}
                    />
                    <span
                      aria-hidden="true"
                      style={{
                        width: "0.55rem",
                        height: "0.55rem",
                        borderRadius: "50%",
                        background: resolveHighlightColor(tag.colorToken),
                      }}
                    />
                    {tag.name}
                  </label>
                </li>
              );
            })}
          </ul>
        )}
      </fieldset>
      <div style={{display: "inline-flex", gap: "var(--ez-space-2)", justifyContent: "flex-end"}}>
        <EzButton variant="ghost" onClick={props.onCancel} ariaLabel="Cancel edits">
          Cancel
        </EzButton>
        <EzButton type="submit" ariaLabel="Save annotation edits">
          Save
        </EzButton>
      </div>
    </form>
  );
}

function formatTimestamp(ms: number): string {
  if (!Number.isFinite(ms) || ms <= 0) {
    return "unknown date";
  }
  try {
    return new Date(ms).toLocaleString();
  } catch {
    return "unknown date";
  }
}

const panelStyle: CSSProperties = {
  background: "var(--ez-color-bg-glass)",
  borderRadius: "var(--ez-radius-md)",
  padding: "var(--ez-space-4)",
  display: "flex",
  flexDirection: "column",
  gap: "var(--ez-space-3)",
};

const headerStyle: CSSProperties = {
  display: "flex",
  alignItems: "center",
  justifyContent: "space-between",
};

const headingStyle: CSSProperties = {
  margin: 0,
  fontSize: "0.95rem",
  letterSpacing: "0.04em",
  textTransform: "uppercase",
  color: "var(--ez-color-text-secondary)",
};

const textareaStyle: CSSProperties = {
  background: "var(--ez-color-bg-elevated)",
  border: "1px solid var(--ez-color-border-subtle)",
  borderRadius: "var(--ez-radius-sm)",
  padding: "var(--ez-space-2) var(--ez-space-3)",
  color: "var(--ez-color-text-primary)",
  fontFamily: "inherit",
  fontSize: "1rem",
  width: "100%",
  resize: "vertical",
};

const tagListStyle: CSSProperties = {
  listStyle: "none",
  margin: 0,
  padding: 0,
  display: "flex",
  gap: "var(--ez-space-2)",
  flexWrap: "wrap",
};

function tagChipStyle(colorToken: string): CSSProperties {
  return {
    display: "inline-flex",
    alignItems: "center",
    gap: "var(--ez-space-1)",
    padding: "var(--ez-space-1) var(--ez-space-2)",
    border: "1px solid var(--ez-color-border-subtle)",
    borderRadius: "var(--ez-radius-sm)",
    fontFamily: "var(--ez-font-mono)",
    fontSize: "0.8rem",
    color: "var(--ez-color-text-primary)",
    background: resolveHighlightColor(colorToken),
    backgroundBlendMode: "soft-light",
  };
}
