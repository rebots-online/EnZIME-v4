import type { JSX } from "react";
import { type CSSProperties, useId, useState } from "react";
import type { TagSummary, Ulid } from "../../services/annotationService";
import { EzButton, resolveHighlightColor } from "./sharedUi";

export interface TagFilterProps {
  tags: TagSummary[];
  selectedTagIds: Ulid[];
  onChange: (tagIds: Ulid[]) => void;
  onCreateTag?: (name: string, colorToken: string) => void;
}

const COLOR_PALETTE: readonly string[] = [
  "highlight-yellow",
  "highlight-mint",
  "highlight-pink",
  "highlight-blue",
];

export function TagFilter({
  tags,
  selectedTagIds,
  onChange,
  onCreateTag,
}: TagFilterProps): JSX.Element {
  const headingId = useId();
  const searchId = useId();
  const newTagInputId = useId();
  const [query, setQuery] = useState("");
  const [newTagName, setNewTagName] = useState("");
  const [newTagColor, setNewTagColor] = useState<string>(COLOR_PALETTE[0] ?? "highlight-yellow");

  const filteredTags = filterTags(tags, query);
  const selected = new Set(selectedTagIds);
  const trimmedQuery = query.trim().toLowerCase();

  const toggle = (tagId: Ulid) => {
    if (selected.has(tagId)) {
      onChange(selectedTagIds.filter((id) => id !== tagId));
    } else {
      onChange([...selectedTagIds, tagId]);
    }
  };

  const handleCreate = () => {
    const trimmed = newTagName.trim();
    if (!trimmed || !onCreateTag) {
      return;
    }
    onCreateTag(trimmed, newTagColor);
    setNewTagName("");
  };

  return (
    <section
      aria-labelledby={headingId}
      style={sectionStyle}
    >
      <h3 id={headingId} style={headingStyle}>
        Tags
      </h3>
      <input
        id={searchId}
        type="search"
        value={query}
        onChange={(event) => setQuery(event.target.value)}
        placeholder="Filter tags by name"
        aria-label="Filter tags by name"
        style={inputStyle}
      />
      {filteredTags.length === 0 ? (
        <p style={{ margin: 0, color: "var(--ez-color-text-muted)" }}>
          {tags.length === 0
            ? "No tags yet. Create one to start grouping annotations."
            : `No tags match “${trimmedQuery}”.`}
        </p>
      ) : (
        <ul role="list" style={listStyle}>
          {filteredTags.map((tag) => {
            const active = selected.has(tag.tagId);
            return (
              <li key={tag.tagId} style={listItemStyle}>
                <button
                  type="button"
                  onClick={() => toggle(tag.tagId)}
                  aria-pressed={active}
                  style={tagButtonStyle(active)}
                  aria-label={`${active ? "Remove" : "Add"} tag ${tag.name}`}
                >
                  <span
                    aria-hidden="true"
                    style={{
                      ...swatchStyle,
                      background: resolveHighlightColor(tag.colorToken),
                    }}
                  />
                  <span>{tag.name}</span>
                </button>
              </li>
            );
          })}
        </ul>
      )}
      {onCreateTag ? (
        <form
          style={createFormStyle}
          onSubmit={(event) => {
            event.preventDefault();
            handleCreate();
          }}
        >
          <label htmlFor={newTagInputId} style={srLabelStyle}>
            New tag name
          </label>
          <input
            id={newTagInputId}
            type="text"
            value={newTagName}
            onChange={(event) => setNewTagName(event.target.value)}
            placeholder="New tag"
            style={inputStyle}
            autoComplete="off"
            spellCheck={false}
          />
          <div role="radiogroup" aria-label="New tag colour" style={colorRowStyle}>
            {COLOR_PALETTE.map((token) => (
              <label key={token} style={{ display: "inline-flex", alignItems: "center", gap: "var(--ez-space-1)" }}>
                <input
                  type="radio"
                  name="tag-color"
                  value={token}
                  checked={token === newTagColor}
                  onChange={() => setNewTagColor(token)}
                  aria-label={`Use colour ${token}`}
                  style={{
                    appearance: "none",
                    width: "1rem",
                    height: "1rem",
                    borderRadius: "50%",
                    border:
                      token === newTagColor
                        ? "2px solid var(--ez-color-border-focus)"
                        : "2px solid var(--ez-color-border-subtle)",
                    background: resolveHighlightColor(token),
                    margin: 0,
                    cursor: "pointer",
                  }}
                />
                <span style={srLabelStyle}>{token.replace(/^highlight-/, "")}</span>
              </label>
            ))}
          </div>
          <div style={{ display: "inline-flex", justifyContent: "flex-end" }}>
            <EzButton
              type="submit"
              disabled={newTagName.trim().length === 0}
              ariaLabel="Create tag"
            >
              Create tag
            </EzButton>
          </div>
        </form>
      ) : null}
    </section>
  );
}

function filterTags(tags: TagSummary[], query: string): TagSummary[] {
  const trimmed = query.trim().toLowerCase();
  if (!trimmed) {
    return [...tags].sort((a, b) => a.name.localeCompare(b.name));
  }
  return tags
    .filter((tag) => tag.name.toLowerCase().includes(trimmed))
    .sort((a, b) => a.name.localeCompare(b.name));
}

function tagButtonStyle(active: boolean): CSSProperties {
  return {
    display: "inline-flex",
    alignItems: "center",
    gap: "var(--ez-space-2)",
    padding: "var(--ez-space-1) var(--ez-space-3)",
    borderRadius: "var(--ez-radius-sm)",
    border: active
      ? "1px solid var(--ez-color-border-focus)"
      : "1px solid var(--ez-color-border-subtle)",
    background: active ? "rgba(124, 140, 255, 0.12)" : "transparent",
    color: "var(--ez-color-text-primary)",
    fontFamily: "inherit",
    fontSize: "0.9rem",
    cursor: "pointer",
    width: "100%",
  };
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

const inputStyle: CSSProperties = {
  background: "var(--ez-color-bg-elevated)",
  border: "1px solid var(--ez-color-border-subtle)",
  borderRadius: "var(--ez-radius-sm)",
  padding: "var(--ez-space-2) var(--ez-space-3)",
  color: "var(--ez-color-text-primary)",
  fontFamily: "inherit",
  fontSize: "0.95rem",
  width: "100%",
};

const listStyle: CSSProperties = {
  listStyle: "none",
  margin: 0,
  padding: 0,
  display: "flex",
  flexDirection: "column",
  gap: "var(--ez-space-1)",
};

const listItemStyle: CSSProperties = { margin: 0, padding: 0 };

const swatchStyle: CSSProperties = {
  width: "0.6rem",
  height: "0.6rem",
  borderRadius: "50%",
  display: "inline-block",
};

const createFormStyle: CSSProperties = {
  display: "flex",
  flexDirection: "column",
  gap: "var(--ez-space-2)",
  paddingTop: "var(--ez-space-3)",
  borderTop: "1px solid var(--ez-color-border-subtle)",
};

const srLabelStyle: CSSProperties = {
  fontSize: "0.8rem",
  color: "var(--ez-color-text-secondary)",
};

const colorRowStyle: CSSProperties = {
  display: "inline-flex",
  gap: "var(--ez-space-2)",
  flexWrap: "wrap",
};
