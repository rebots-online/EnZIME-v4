import type { JSX } from "react";
import { type CSSProperties, useId } from "react";
import type {
  AnnotationListRequest,
  AnnotationResolution,
  SidecarLayer,
  TagSummary,
  Ulid,
} from "../../services/annotationService";
import { EzButton } from "./sharedUi";
import { TagFilter } from "./TagFilter";
import { SidecarLayerToggle } from "./SidecarLayerToggle";

export interface NotesToolbarProps {
  filter: AnnotationListRequest;
  tags: TagSummary[];
  layers: SidecarLayer[];
  onFilterChange: (filter: AnnotationListRequest) => void;
  onCreateTag?: (name: string, colorToken: string) => void;
  onCreateNotebook?: () => void;
  onReload: () => void;
}

const RESOLUTION_STATES: ReadonlyArray<AnnotationResolution["state"]> = [
  "resolved",
  "ambiguous",
  "source_unavailable",
  "orphaned",
];

const RESOLUTION_LABELS: Record<AnnotationResolution["state"], string> = {
  resolved: "Resolved",
  ambiguous: "Ambiguous",
  source_unavailable: "Source unavailable",
  orphaned: "Orphaned",
};

export function NotesToolbar(props: NotesToolbarProps): JSX.Element {
  const headingId = useId();
  const textInputId = useId();
  const {
    filter,
    onFilterChange,
  } = props;
  const setText = (text: string) => {
    onFilterChange({...filter, text, offset: 0});
  };
  const setTagIds = (tagIds: Ulid[]) => {
    onFilterChange({...filter, tagIds, offset: 0});
  };
  const setLayerIds = (sidecarLayerIds: Ulid[]) => {
    onFilterChange({...filter, sidecarLayerIds, offset: 0});
  };
  const toggleResolution = (state: AnnotationResolution["state"]) => {
    const current = new Set(filter.resolutionStates ?? []);
    if (current.has(state)) {
      current.delete(state);
    } else {
      current.add(state);
    }
    const resolutionStates = Array.from(current);
    onFilterChange({
      ...filter,
      resolutionStates,
      offset: 0,
    });
  };
  const clearAll = () => {
    onFilterChange({offset: 0, limit: filter.limit});
  };

  return (
    <section
      aria-labelledby={headingId}
      style={sectionStyle}
    >
      <h3 id={headingId} style={headingStyle}>
        Filter annotations
      </h3>
      <div style={searchRowStyle}>
        <label htmlFor={textInputId} style={srLabelStyle}>
          Search annotation text
        </label>
        <input
          id={textInputId}
          type="search"
          value={filter.text ?? ""}
          onChange={(event) => setText(event.target.value)}
          placeholder="Search annotation text, body, or quoted passage"
          style={inputStyle}
        />
        <EzButton
          variant="ghost"
          onClick={props.onReload}
          ariaLabel="Reload annotations from the store"
        >
          Reload
        </EzButton>
      </div>
      <fieldset style={fieldsetStyle}>
        <legend style={legendStyle}>Resolution state</legend>
        <div role="group" aria-label="Filter by resolution state" style={chipsStyle}>
          {RESOLUTION_STATES.map((state) => {
            const active = (filter.resolutionStates ?? []).includes(state);
            return (
              <label key={state} style={chipLabelStyle}>
                <input
                  type="checkbox"
                  checked={active}
                  onChange={() => toggleResolution(state)}
                  aria-label={`${active ? "Hide" : "Show"} ${RESOLUTION_LABELS[state]} annotations`}
                />
                <span>{RESOLUTION_LABELS[state]}</span>
              </label>
            );
          })}
        </div>
      </fieldset>
      <div style={columnsStyle}>
        <TagFilter
          tags={props.tags}
          selectedTagIds={filter.tagIds ?? []}
          onChange={setTagIds}
          onCreateTag={props.onCreateTag}
        />
        <SidecarLayerToggle
          layers={props.layers}
          selectedLayerIds={filter.sidecarLayerIds ?? []}
          onChange={setLayerIds}
        />
      </div>
      <div style={{display: "inline-flex", justifyContent: "flex-end", gap: "var(--ez-space-2)"}}>
        <EzButton variant="ghost" onClick={clearAll} ariaLabel="Clear all filters">
          Clear filters
        </EzButton>
        {props.onCreateNotebook ? (
          <EzButton
            onClick={props.onCreateNotebook}
            ariaLabel="Create notebook"
          >
            New notebook
          </EzButton>
        ) : null}
      </div>
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

const searchRowStyle: CSSProperties = {
  display: "flex",
  gap: "var(--ez-space-2)",
  alignItems: "center",
  flexWrap: "wrap",
};

const inputStyle: CSSProperties = {
  background: "var(--ez-color-bg-elevated)",
  border: "1px solid var(--ez-color-border-subtle)",
  borderRadius: "var(--ez-radius-sm)",
  padding: "var(--ez-space-2) var(--ez-space-3)",
  color: "var(--ez-color-text-primary)",
  fontFamily: "inherit",
  fontSize: "1rem",
  flex: 1,
  minWidth: "12rem",
};

const fieldsetStyle: CSSProperties = {
  border: "1px solid var(--ez-color-border-subtle)",
  borderRadius: "var(--ez-radius-sm)",
  padding: "var(--ez-space-3)",
  margin: 0,
};

const legendStyle: CSSProperties = {
  paddingInline: "var(--ez-space-2)",
  color: "var(--ez-color-text-secondary)",
  fontSize: "0.85rem",
};

const chipsStyle: CSSProperties = {
  display: "flex",
  gap: "var(--ez-space-3)",
  flexWrap: "wrap",
};

const chipLabelStyle: CSSProperties = {
  display: "inline-flex",
  alignItems: "center",
  gap: "var(--ez-space-2)",
  color: "var(--ez-color-text-primary)",
  fontSize: "0.9rem",
};

const columnsStyle: CSSProperties = {
  display: "grid",
  gridTemplateColumns: "repeat(auto-fit, minmax(14rem, 1fr))",
  gap: "var(--ez-space-3)",
};

const srLabelStyle: CSSProperties = {
  position: "absolute",
  width: "1px",
  height: "1px",
  padding: 0,
  margin: "-1px",
  overflow: "hidden",
  clip: "rect(0,0,0,0)",
  whiteSpace: "nowrap",
  border: 0,
};
