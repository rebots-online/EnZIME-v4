import { type CSSProperties, useId } from "react";
import type { SelectorBundle } from "../../services/annotationService";
import { EzButton, resolveHighlightColor, useReducedMotion } from "./sharedUi";

export interface TextHighlighterProps {
  selection?: SelectorBundle;
  colors: string[];
  selectedColor: string;
  onColorChange: (colorToken: string) => void;
  onCreate: () => void;
}

export function TextHighlighter({
  selection,
  colors,
  selectedColor,
  onColorChange,
  onCreate,
}: TextHighlighterProps): JSX.Element {
  const reduced = useReducedMotion();
  const radioGroupName = useId();
  const exactQuote = selection?.textQuote?.exact?.trim();
  const hasSelection = Boolean(selection && exactQuote);
  return (
    <section
      aria-labelledby="ez-text-highlighter-heading"
      style={panelStyle}
    >
      <h3 id="ez-text-highlighter-heading" style={headingStyle}>
        Highlight
      </h3>
      <div role="radiogroup" aria-label="Highlight colour" style={rowStyle}>
        {colors.map((token) => {
          const active = token === selectedColor;
          return (
            <label
              key={token}
              style={{
                display: "inline-flex",
                alignItems: "center",
                gap: "var(--ez-space-2)",
                cursor: "pointer",
              }}
            >
              <input
                type="radio"
                name={radioGroupName}
                value={token}
                checked={active}
                onChange={() => onColorChange(token)}
                aria-label={`Highlight colour ${token}`}
                style={{
                  appearance: "none",
                  width: "1.25rem",
                  height: "1.25rem",
                  borderRadius: "50%",
                  border: active
                    ? "2px solid var(--ez-color-border-focus)"
                    : "2px solid var(--ez-color-border-subtle)",
                  background: resolveHighlightColor(token),
                  margin: 0,
                  cursor: "pointer",
                  transition: reduced
                    ? "none"
                    : "border-color var(--ez-duration-fast)",
                }}
              />
              <span style={swatchLabelStyle}>{token.replace(/^highlight-/, "")}</span>
            </label>
          );
        })}
      </div>
      <div aria-live="polite" style={selectionPreviewStyle}>
        {hasSelection ? (
          <blockquote style={quoteStyle}>
            “{exactQuote}”
          </blockquote>
        ) : (
          <p style={{ margin: 0, color: "var(--ez-color-text-muted)" }}>
            Select text in the article to highlight. The selection will appear
            here before you create the highlight.
          </p>
        )}
      </div>
      <div style={{ ...rowStyle, justifyContent: "flex-end" }}>
        {hasSelection ? (
          <EzButton
            onClick={onCreate}
            ariaLabel="Create highlight from current selection"
          >
            Create highlight
          </EzButton>
        ) : null}
      </div>
    </section>
  );
}

const panelStyle: CSSProperties = {
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

const rowStyle: CSSProperties = {
  display: "flex",
  flexWrap: "wrap",
  gap: "var(--ez-space-3)",
};

const swatchLabelStyle: CSSProperties = {
  fontFamily: "var(--ez-font-mono)",
  fontSize: "0.85rem",
  color: "var(--ez-color-text-secondary)",
};

const selectionPreviewStyle: CSSProperties = {
  background: "var(--ez-color-bg-elevated)",
  border: "1px solid var(--ez-color-border-subtle)",
  borderRadius: "var(--ez-radius-sm)",
  padding: "var(--ez-space-3)",
  fontSize: "0.95rem",
  color: "var(--ez-color-text-primary)",
  minHeight: "3rem",
};

const quoteStyle: CSSProperties = {
  margin: 0,
  whiteSpace: "pre-wrap",
  fontFamily: "var(--ez-font-reader)",
};
