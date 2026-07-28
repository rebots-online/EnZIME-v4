import { useCallback, useId, useState, type CSSProperties, type ChangeEvent } from "react";
import type {
  SidecarConflict,
  SidecarImportPlan,
} from "../../services/annotationService";
import {
  EzButton,
  InlineStatus,
  signatureStateText,
  useReducedMotion,
} from "./sharedUi";

export interface SidecarImportDialogProps {
  open: boolean;
  plan?: SidecarImportPlan;
  onChooseFile: () => void;
  onImport: (acceptUnsigned: boolean, layerName: string) => void;
  onCancel: () => void;
}

export function SidecarImportDialog({
  open,
  plan,
  onChooseFile,
  onImport,
  onCancel,
}: SidecarImportDialogProps): JSX.Element {
  const reduced = useReducedMotion();
  const [layerName, setLayerName] = useState("");
  const [acceptUnsigned, setAcceptUnsigned] = useState(false);
  const [exportPanelStyle] = useState<CSSProperties>(() => ({ ...dialogStyle }));
  const headingId = useId();
  const fileButtonId = useId();
  const layerInputId = useId();
  const acceptInputId = useId();

  const handleLayerChange = useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      setLayerName(event.target.value);
    },
    [],
  );

  const handleAcceptChange = useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      setAcceptUnsigned(event.target.checked);
    },
    [],
  );

  if (!open) {
    return <span aria-hidden="true" style={{ display: "none" }} />;
  }

  const sig = plan ? signatureStateText(plan.signatureState) : null;
  const requiresExplicitAccept = plan
    ? plan.signatureState !== "verified"
    : false;
  const layerNameTrimmed = layerName.trim();
  const canCommit =
    !!plan &&
    layerNameTrimmed.length > 0 &&
    (!requiresExplicitAccept || acceptUnsigned);

  return (
    <aside
      role="dialog"
      aria-modal="true"
      aria-labelledby={headingId}
      style={exportPanelStyle}
    >
      <div style={backdropStyle(reduced)} aria-hidden="true" />
      <section style={surfaceStyle}>
        <header style={headerStyle}>
          <h3 id={headingId} style={headingStyle}>
            Import annotation sidecar
          </h3>
        </header>
        <p style={{ margin: 0, color: "var(--ez-color-text-secondary)" }}>
          A sidecar layer carries annotations from another device or shared
          export. EnZIME never overwrites existing layers; each import lands in
          its own named layer you can review and toggle.
        </p>
        {!plan ? (
          <div style={chooseRowStyle}>
            <EzButton
              onClick={onChooseFile}
              ariaLabel="Choose sidecar file to import"
              aria-labelledby={fileButtonId}
            >
              <span id={fileButtonId}>Choose sidecar file…</span>
            </EzButton>
          </div>
        ) : (
          <PlanSummary plan={plan} />
        )}
        {sig ? (
          <InlineStatus
            tone={sig.tone}
            icon={sig.tone === "success" ? "✓" : "!"}
          >
            <span>{sig.label}</span>
          </InlineStatus>
        ) : null}
        {plan ? (
          <form
            style={formStyle}
            onSubmit={(event) => {
              event.preventDefault();
              if (canCommit) {
                onImport(acceptUnsigned, layerNameTrimmed);
              }
            }}
          >
            <label htmlFor={layerInputId} style={labelStyle}>
              Layer name
            </label>
            <input
              id={layerInputId}
              type="text"
              value={layerName}
              onChange={handleLayerChange}
              placeholder="Imported from collaborator"
              required
              style={inputStyle}
              autoComplete="off"
              spellCheck={false}
            />
            {requiresExplicitAccept ? (
              <label
                htmlFor={acceptInputId}
                style={{
                  ...labelStyle,
                  display: "inline-flex",
                  alignItems: "flex-start",
                  gap: "var(--ez-space-2)",
                }}
              >
                <input
                  id={acceptInputId}
                  type="checkbox"
                  checked={acceptUnsigned}
                  onChange={handleAcceptChange}
                  required
                  style={{ marginTop: "0.25rem" }}
                />
                <span>
                  I understand this sidecar is unsigned or has an untrusted
                  signature, and I want to import it into a sandboxed layer
                  anyway. Imported annotations stay quarantined in the layer
                  until I promote them.
                </span>
              </label>
            ) : null}
            <footer style={actionsStyle}>
              <EzButton variant="ghost" onClick={onCancel} ariaLabel="Cancel sidecar import">
                Cancel
              </EzButton>
              <EzButton
                type="submit"
                disabled={!canCommit}
                ariaLabel="Commit sidecar import"
              >
                Import sidecar
              </EzButton>
            </footer>
          </form>
        ) : (
          <footer style={actionsStyle}>
            <EzButton variant="ghost" onClick={onCancel} ariaLabel="Cancel sidecar import">
              Cancel
            </EzButton>
          </footer>
        )}
      </section>
    </aside>
  );
}

function PlanSummary({ plan }: { plan: SidecarImportPlan }): JSX.Element {
  return (
    <dl style={summaryStyle}>
      <SummaryRow term="Annotations" value={`${plan.annotationCount}`} />
      <SummaryRow term="Notebooks" value={`${plan.notebookCount}`} />
      <SummaryRow term="Tags" value={`${plan.tagCount}`} />
      <SummaryRow
        term="Object payload"
        value={`${(plan.objectBytes / 1024).toFixed(1)} KiB`}
      />
      {plan.signer ? <SummaryRow term="Signer" value={plan.signer} /> : null}
      <SummaryRow term="Artifact digest" value={shortDigest(plan.artifactSha256)} />
      {plan.conflicts.length > 0 ? (
        <ConflictList conflicts={plan.conflicts} />
      ) : null}
    </dl>
  );
}

function SummaryRow({
  term,
  value,
}: {
  term: string;
  value: string;
}): JSX.Element {
  return (
    <div
      style={{
        display: "grid",
        gridTemplateColumns: "minmax(8rem, 30%) 1fr",
        gap: "var(--ez-space-2)",
        fontFamily: "var(--ez-font-mono)",
        fontSize: "0.85rem",
        color: "var(--ez-color-text-secondary)",
      }}
    >
      <dt>{term}</dt>
      <dd style={{ margin: 0, color: "var(--ez-color-text-primary)" }}>{value}</dd>
    </div>
  );
}

function ConflictList({
  conflicts,
}: {
  conflicts: SidecarConflict[];
}): JSX.Element {
  return (
    <div style={{ marginTop: "var(--ez-space-2)" }}>
      <p
        style={{
          margin: 0,
          marginBottom: "var(--ez-space-1)",
          color: "var(--ez-color-warning)",
        }}
      >
        Conflicts detected ({conflicts.length}):
      </p>
      <ul style={{ margin: 0, paddingLeft: "var(--ez-space-4)" }}>
        {conflicts.map((conflict, index) => (
          <li key={`${conflict.entityKind}-${conflict.entityId}-${index}`}>
            <span style={{ color: "var(--ez-color-text-secondary)" }}>
              {conflict.entityKind} {conflict.entityId}:
            </span>{" "}
            <span>{conflict.explanation}</span>
          </li>
        ))}
      </ul>
    </div>
  );
}

function shortDigest(digest: string): string {
  return digest.length > 12 ? `${digest.slice(0, 10)}…${digest.slice(-4)}` : digest;
}

const dialogStyle: CSSProperties = {
  position: "fixed",
  inset: 0,
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  padding: "var(--ez-space-4)",
  zIndex: 60,
};

function backdropStyle(reduced: boolean): CSSProperties {
  return {
    position: "absolute",
    inset: 0,
    background: "rgba(9, 11, 19, 0.72)",
    backdropFilter: "blur(var(--ez-blur-glass))",
    transition: reduced ? "none" : "opacity var(--ez-duration-normal)",
  };
}

const surfaceStyle: CSSProperties = {
  position: "relative",
  background: "var(--ez-color-bg-glass-strong)",
  borderRadius: "var(--ez-radius-lg)",
  boxShadow: "var(--ez-shadow-panel)",
  padding: "var(--ez-space-5)",
  width: "min(40rem, 100%)",
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
  fontSize: "1.05rem",
};

const chooseRowStyle: CSSProperties = {
  display: "flex",
  gap: "var(--ez-space-2)",
};

const formStyle: CSSProperties = {
  display: "flex",
  flexDirection: "column",
  gap: "var(--ez-space-3)",
};

const labelStyle: CSSProperties = {
  fontSize: "0.85rem",
  color: "var(--ez-color-text-secondary)",
};

const inputStyle: CSSProperties = {
  background: "var(--ez-color-bg-elevated)",
  border: "1px solid var(--ez-color-border-subtle)",
  borderRadius: "var(--ez-radius-sm)",
  padding: "var(--ez-space-2) var(--ez-space-3)",
  color: "var(--ez-color-text-primary)",
  fontFamily: "inherit",
  fontSize: "1rem",
};

const actionsStyle: CSSProperties = {
  display: "flex",
  justifyContent: "flex-end",
  gap: "var(--ez-space-2)",
};

const summaryStyle: CSSProperties = {
  margin: 0,
  display: "flex",
  flexDirection: "column",
  gap: "var(--ez-space-1)",
};
