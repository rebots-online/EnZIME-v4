import { type CSSProperties, useId } from "react";
import type { SidecarLayer, Ulid } from "../../services/annotationService";
import { InlineStatus, trustStateText } from "./sharedUi";

export interface SidecarLayerToggleProps {
  layers: SidecarLayer[];
  selectedLayerIds: Ulid[];
  onChange: (layerIds: Ulid[]) => void;
}

export function SidecarLayerToggle({
  layers,
  selectedLayerIds,
  onChange,
}: SidecarLayerToggleProps): JSX.Element {
  const headingId = useId();
  if (layers.length === 0) {
    return (
      <section aria-labelledby={headingId} style={sectionStyle}>
        <h3 id={headingId} style={headingStyle}>
          Sidecar layers
        </h3>
        <p style={{ margin: 0, color: "var(--ez-color-text-muted)" }}>
          No sidecar layers yet. Import a sidecar from an article workspace to
          see imported annotation layers here.
        </p>
      </section>
    );
  }
  const selected = new Set(selectedLayerIds);
  return (
    <section aria-labelledby={headingId} style={sectionStyle}>
      <h3 id={headingId} style={headingStyle}>
        Sidecar layers
      </h3>
      <ul role="list" style={listStyle}>
        {layers.map((layer) => {
          const checked = selected.has(layer.sidecarLayerId);
          const trust = trustStateText(layer.trustState);
          const icon =
            trust.tone === "success"
              ? "✓"
              : trust.tone === "danger"
                ? "⊗"
                : "!";
          return (
            <li key={layer.sidecarLayerId} style={listItemStyle}>
              <label style={rowStyle}>
                <input
                  type="checkbox"
                  checked={checked}
                  onChange={() => {
                    if (checked) {
                      onChange(
                        selectedLayerIds.filter(
                          (id) => id !== layer.sidecarLayerId,
                        ),
                      );
                    } else {
                      onChange([...selectedLayerIds, layer.sidecarLayerId]);
                    }
                  }}
                  aria-label={`Toggle layer ${layer.name}`}
                  style={{ margin: 0 }}
                />
                <span style={layerInfoStyle}>
                  <span style={layerNameStyle}>{layer.name}</span>
                  <InlineStatus tone={trust.tone} icon={icon}>
                    <span>{trust.label}</span>
                  </InlineStatus>
                  <span style={layerMetaStyle}>
                    {layer.visibility} · {layer.annotationCount} annotation
                    {layer.annotationCount === 1 ? "" : "s"}
                  </span>
                </span>
              </label>
            </li>
          );
        })}
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

const listItemStyle: CSSProperties = { margin: 0, padding: 0 };

const rowStyle: CSSProperties = {
  display: "flex",
  alignItems: "flex-start",
  gap: "var(--ez-space-3)",
  padding: "var(--ez-space-2)",
  borderRadius: "var(--ez-radius-sm)",
  background: "var(--ez-color-bg-elevated)",
  border: "1px solid var(--ez-color-border-subtle)",
};

const layerInfoStyle: CSSProperties = {
  display: "flex",
  flexDirection: "column",
  gap: "var(--ez-space-1)",
};

const layerNameStyle: CSSProperties = {
  fontWeight: 600,
  color: "var(--ez-color-text-primary)",
};

const layerMetaStyle: CSSProperties = {
  color: "var(--ez-color-text-muted)",
  fontSize: "0.8rem",
  fontFamily: "var(--ez-font-mono)",
};
