import type { JSX } from "react";
import {
  useCallback,
  useEffect,
  useMemo,
  useId,
  useState,
  type CSSProperties,
  type ReactNode,
} from "react";
import {
  type AnnotationDraft,
  type AnnotationRecord,
  type ArticleResponse,
  type SelectorBundle,
  type Ulid,
} from "../../services/annotationService";
import {
  CapabilityDeniedState,
  ErrorBoundaryState,
  EzButton,
  EzIconButton,
  InlineStatus,
  OfflineState,
  Spinner,
  useReducedMotion,
} from "./sharedUi";
import { AnnotationPanel } from "./AnnotationPanel";
import { TextHighlighter } from "./TextHighlighter";
import { InkCanvas } from "./InkCanvas";
import {
  VoiceRecorder,
  VoiceRecorderSelfManaged,
  type VoiceRecorderSelfManagedProps,
} from "./VoiceRecorder";
import { SidecarImportDialog } from "./SidecarImportDialog";
import {
  HIGHLIGHT_COLORS,
  useAnnotationStore,
  type LoadStatus,
} from "../../stores/annotationStore";

export type AnnotationTool =
  | "select"
  | "highlight"
  | "comment"
  | "ink"
  | "voice";

export interface AnnotationWorkspaceProps {
  article: ArticleResponse;
  selection?: SelectorBundle;
  annotations: AnnotationRecord[];
  activeTool: AnnotationTool;
  onToolChange: (tool: AnnotationTool) => void;
  onCreate: (draft: AnnotationDraft) => void;
  onUpdate: (
    annotationId: Ulid,
    expectedRevision: number,
    draft: AnnotationDraft,
  ) => void;
  onClose: () => void;
}

const TOOL_ORDER: readonly AnnotationTool[] = [
  "select",
  "highlight",
  "comment",
  "ink",
  "voice",
];

const TOOL_LABELS: Record<AnnotationTool, string> = {
  select: "Select",
  highlight: "Highlight",
  comment: "Comment",
  ink: "Ink",
  voice: "Voice",
};

export function AnnotationWorkspace(
  props: AnnotationWorkspaceProps,
): JSX.Element {
  const reduced = useReducedMotion();
  const headingId = useId();
  const toolBarId = useId();
  const [highlightColor, setHighlightColor] = useState<string>(
    HIGHLIGHT_COLORS[0] ?? "highlight-yellow",
  );
  const [commentText, setCommentText] = useState("");
  const [inkColor, setInkColor] = useState<string>(HIGHLIGHT_COLORS[3] ?? "highlight-blue");
  const [inkStrokeWidth, setInkStrokeWidth] = useState(3);
  const [sidecarOpen, setSidecarOpen] = useState(false);

  const store = useAnnotationStore();
  const status = store.status;
  const mutation = store.mutation;
  const articleAnnotations = useMemo(
    () =>
      props.annotations.filter(
        (annotation) =>
          annotation.selector.article.packId === props.article.packId &&
          annotation.selector.article.articleKey === props.article.articleKey,
      ),
    [props.annotations, props.article.packId, props.article.articleKey],
  );

  useEffect(() => {
    if (status.kind === "idle") {
      void store.load();
    }
  }, [status.kind, store]);

  const handleCreateHighlight = useCallback(() => {
    if (!props.selection) {
      return;
    }
    const draft: AnnotationDraft = {
      annotationType: "highlight",
      selector: props.selection,
      style: {
        colorToken: highlightColor,
        emphasis: "solid",
      },
      tagIds: [],
      sidecarLayerId: localLayerId(store.layers),
    };
    void store.create(draft);
    props.onCreate(draft);
  }, [highlightColor, props, store]);

  const handleCreateComment = useCallback(() => {
    if (!props.selection) {
      return;
    }
    const trimmed = commentText.trim();
    if (trimmed.length === 0) {
      return;
    }
    const draft: AnnotationDraft = {
      annotationType: "comment",
      selector: props.selection,
      bodyText: trimmed,
      style: {
        colorToken: highlightColor,
        emphasis: "underline",
      },
      tagIds: [],
      sidecarLayerId: localLayerId(store.layers),
    };
    void store.create(draft);
    setCommentText("");
    props.onCreate(draft);
  }, [commentText, highlightColor, props, store]);

  const handleCommitInk = useCallback(
    (svg: string) => {
      if (!props.selection) {
        return;
      }
      const draft: AnnotationDraft = {
        annotationType: "ink",
        selector: props.selection,
        bodyText: svg,
        style: {
          colorToken: inkColor,
          emphasis: "solid",
          strokeWidth: inkStrokeWidth,
        },
        tagIds: [],
        sidecarLayerId: localLayerId(store.layers),
      };
      void store.create(draft).then((created) => {
        if (created) {
          void store.uploadInk(
            created.annotationId,
            svg,
            inkColor,
            inkStrokeWidth,
          );
        }
      });
      props.onCreate(draft);
    },
    [inkColor, inkStrokeWidth, props, store],
  );

  const handleVoiceCapture = useCallback<VoiceRecorderSelfManagedProps["onCapture"]>(
    async (capture) => {
      if (!props.selection) {
        return;
      }
      const draft: AnnotationDraft = {
        annotationType: "voice",
        selector: props.selection,
        style: {
          colorToken: highlightColor,
          emphasis: "solid",
        },
        tagIds: [],
        sidecarLayerId: localLayerId(store.layers),
      };
      const created = await store.create(draft);
      if (!created) {
        throw new Error("Annotation was not created; voice upload aborted.");
      }
      props.onCreate(draft);
      await store.uploadVoice(created.annotationId, capture.bytes, capture.durationMs);
    },
    [highlightColor, props, store],
  );

  const handleDeleteFromPanel = useCallback(
    (annotationId: Ulid, revision: number) => {
      void store.remove(annotationId, revision);
    },
    [store],
  );

  const handleRestoreFromPanel = useCallback(
    (annotationId: Ulid) => {
      void store.restore(annotationId);
    },
    [store],
  );

  const handleSelectInPanel = useCallback(
    (annotationId: Ulid) => {
      store.selectAnnotation(annotationId);
    },
    [store],
  );

  const openSidecarImport = useCallback(() => {
    setSidecarOpen(true);
  }, []);

  const handleSidecarChooseFile = useCallback(() => {
    void store.chooseSidecarImportSource();
  }, [store]);

  const handleSidecarImport = useCallback(
    (acceptUnsigned: boolean, layerName: string) => {
      void store
        .commitSidecarImport({
          artifactSha256:
            store.layers.length > 0
              ? `pending-${Date.now().toString(36)}`
              : "",
          acceptUnsigned,
          layerName,
        })
        .then((layer) => {
          if (layer) {
            setSidecarOpen(false);
          }
        });
    },
    [store],
  );

  return (
    <section
      aria-labelledby={headingId}
      style={{ ...panelStyle(reduced), width: "min(28rem, 100%)" }}
    >
      <header style={headerStyle}>
        <h3 id={headingId} style={headingStyle}>
          Annotate · {props.article.title}
        </h3>
        <div style={{ display: "inline-flex", gap: "var(--ez-space-2)" }}>
          <EzIconButton
            ariaLabel="Open sidecar import"
            onClick={openSidecarImport}
            title="Import sidecar"
          >
            ⇪
          </EzIconButton>
          <EzIconButton
            ariaLabel="Close annotation workspace"
            onClick={props.onClose}
            title="Close"
          >
            ✕
          </EzIconButton>
        </div>
      </header>

      <StatusAdapter
        status={status}
        onRetry={() => void store.reload()}
        offlineMessage="Annotations cannot be loaded while offline."
        deniedMessage="Annotations.rich_media is not granted on this device."
      >
        <AnnotationPanel
          annotations={articleAnnotations}
          selectedAnnotationId={store.selectedAnnotationId}
          onSelect={handleSelectInPanel}
          onDelete={handleDeleteFromPanel}
          onRestore={handleRestoreFromPanel}
        />
      </StatusAdapter>

      <MutationStatus status={mutation} onClear={() => store.clearMutation()} />

      <div role="toolbar" aria-label="Annotation tools" id={toolBarId} style={toolbarStyle}>
        {TOOL_ORDER.map((tool) => (
          <EzButton
            key={tool}
            variant={tool === props.activeTool ? "primary" : "ghost"}
            ariaPressed={tool === props.activeTool}
            onClick={() => props.onToolChange(tool)}
            ariaLabel={`Use ${TOOL_LABELS[tool]} tool`}
          >
            {TOOL_LABELS[tool]}
          </EzButton>
        ))}
      </div>

      <ActiveToolSurface
        tool={props.activeTool}
        selection={props.selection}
        article={props.article}
        highlightColor={highlightColor}
        onHighlightColorChange={setHighlightColor}
        onCreateHighlight={handleCreateHighlight}
        commentText={commentText}
        onCommentTextChange={setCommentText}
        onCreateComment={handleCreateComment}
        inkColor={inkColor}
        onInkColorChange={setInkColor}
        inkStrokeWidth={inkStrokeWidth}
        onInkStrokeWidthChange={setInkStrokeWidth}
        onCommitInk={handleCommitInk}
        onCancelInk={() => props.onToolChange("select")}
        onVoiceCapture={handleVoiceCapture}
        localLayerAvailable={hasLocalLayer(store.layers)}
      />

      <SidecarImportDialog
        open={sidecarOpen}
        onChooseFile={handleSidecarChooseFile}
        onImport={handleSidecarImport}
        onCancel={() => setSidecarOpen(false)}
      />
    </section>
  );
}

interface ActiveToolSurfaceProps {
  tool: AnnotationTool;
  selection?: SelectorBundle;
  article: ArticleResponse;
  highlightColor: string;
  onHighlightColorChange: (color: string) => void;
  onCreateHighlight: () => void;
  commentText: string;
  onCommentTextChange: (text: string) => void;
  onCreateComment: () => void;
  inkColor: string;
  onInkColorChange: (color: string) => void;
  inkStrokeWidth: number;
  onInkStrokeWidthChange: (width: number) => void;
  onCommitInk: (svg: string) => void;
  onCancelInk: () => void;
  onVoiceCapture: VoiceRecorderSelfManagedProps["onCapture"];
  localLayerAvailable: boolean;
}

function ActiveToolSurface(props: ActiveToolSurfaceProps): JSX.Element | null {
  if (props.tool === "select") {
    return (
      <p style={hintStyle}>
        Select a tool above to begin annotating. Highlight, comment, ink, and
        voice tools attach to the current text selection in the article.
      </p>
    );
  }
  if (!props.localLayerAvailable) {
    return (
      <InlineStatus tone="warning" icon="!">
        <span>
          No local annotation layer is available. Create a local layer in
          Settings to attach new annotations.
        </span>
      </InlineStatus>
    );
  }
  switch (props.tool) {
    case "highlight":
      return (
        <TextHighlighter
          selection={props.selection}
          colors={[...HIGHLIGHT_COLORS]}
          selectedColor={props.highlightColor}
          onColorChange={props.onHighlightColorChange}
          onCreate={props.onCreateHighlight}
        />
      );
    case "comment":
      return (
        <CommentComposer
          text={props.commentText}
          onTextChange={props.onCommentTextChange}
          onSubmit={props.onCreateComment}
          selection={props.selection}
        />
      );
    case "ink":
      return (
        <InkCanvas
          enabled
          colorToken={props.inkColor}
          strokeWidth={props.inkStrokeWidth}
          onCommitSvg={props.onCommitInk}
          onCancel={props.onCancelInk}
        />
      );
    case "voice":
      return (
        <VoiceRecorderSelfManaged
          onCapture={props.onVoiceCapture}
          onStart={() => undefined}
          onStop={() => undefined}
          onCommit={() => undefined}
          onDiscard={() => undefined}
        />
      );
    default:
      return null;
  }
}

function CommentComposer({
  text,
  onTextChange,
  onSubmit,
  selection,
}: {
  text: string;
  onTextChange: (text: string) => void;
  onSubmit: () => void;
  selection?: SelectorBundle;
}): JSX.Element {
  const textareaId = useId();
  const quote = selection?.textQuote?.exact?.trim();
  const trimmed = text.trim();
  const canSubmit = trimmed.length > 0 && !!selection;
  return (
    <form
      style={formStyle}
      onSubmit={(event) => {
        event.preventDefault();
        if (canSubmit) {
          onSubmit();
        }
      }}
    >
      <label htmlFor={textareaId} style={labelStyle}>
        Comment on selection
      </label>
      {quote ? (
        <blockquote style={quoteStyle}>{quote}</blockquote>
      ) : (
        <p style={hintStyle}>
          Highlight a passage in the article first, then write the comment.
        </p>
      )}
      <textarea
        id={textareaId}
        value={text}
        onChange={(event) => onTextChange(event.target.value)}
        placeholder="Write a note tied to the selected passage."
        style={textareaStyle}
        rows={4}
      />
      <div style={{ display: "flex", justifyContent: "flex-end" }}>
        <EzButton type="submit" disabled={!canSubmit} ariaLabel="Attach comment">
          Attach comment
        </EzButton>
      </div>
    </form>
  );
}

function StatusAdapter(props: {
  status: LoadStatus;
  onRetry: () => void;
  offlineMessage: string;
  deniedMessage: string;
  children: ReactNode;
}): JSX.Element {
  const { status } = props;
  if (status.kind === "loading" || status.kind === "idle") {
    return (
      <div style={centeredStyle}>
        <Spinner label="Loading annotations…" />
      </div>
    );
  }
  if (status.kind === "offline") {
    return (
      <OfflineState message={props.offlineMessage} onRetry={props.onRetry} />
    );
  }
  if (status.kind === "capability_denied") {
    return <CapabilityDeniedState message={props.deniedMessage} />;
  }
  if (status.kind === "recoverable_error") {
    return (
      <ErrorBoundaryState error={status.error} onRetry={props.onRetry} />
    );
  }
  if (status.kind === "non_recoverable_error") {
    return (
      <ErrorBoundaryState
        error={status.error}
        correlationLabel="Diagnostics correlation"
      />
    );
  }
  if (status.kind === "empty") {
    return <>{props.children}</>;
  }
  return <>{props.children}</>;
}

function MutationStatus({
  status,
  onClear,
}: {
  status: ReturnType<typeof useAnnotationStore.getState>["mutation"];
  onClear: () => void;
}): JSX.Element | null {
  if (status.kind === "idle") {
    return null;
  }
  if (status.kind === "busy") {
    return (
      <InlineStatus tone="info" icon="•">
        <span>{busyLabel(status.operation)}…</span>
      </InlineStatus>
    );
  }
  return (
    <div
      role="alert"
      style={{
        background: "rgba(255, 111, 145, 0.12)",
        border: "1px solid var(--ez-color-danger)",
        borderRadius: "var(--ez-radius-sm)",
        padding: "var(--ez-space-3)",
        display: "flex",
        flexDirection: "column",
        gap: "var(--ez-space-2)",
      }}
    >
      <InlineStatus tone="danger" icon="!">
        <span>
          {operationLabel(status.operation)} failed: {status.error.message}
        </span>
      </InlineStatus>
      {status.error.action ? (
        <span style={{ color: "var(--ez-color-text-secondary)" }}>
          Suggested: {status.error.action.label}
        </span>
      ) : null}
      <div style={{ display: "inline-flex", justifyContent: "flex-end" }}>
        <EzButton variant="ghost" onClick={onClear} ariaLabel="Dismiss error">
          Dismiss
        </EzButton>
      </div>
    </div>
  );
}

function busyLabel(operation: string): string {
  switch (operation) {
    case "create":
      return "Saving annotation";
    case "update":
      return "Updating annotation";
    case "delete":
      return "Deleting annotation";
    case "restore":
      return "Restoring annotation";
    case "asset_upload":
      return "Uploading annotation asset";
    case "notebook_create":
      return "Creating notebook";
    case "notebook_update":
      return "Renaming notebook";
    case "tag_create":
      return "Creating tag";
    case "sidecar_choose":
      return "Choosing sidecar file";
    case "sidecar_plan":
      return "Reading sidecar plan";
    case "sidecar_commit":
      return "Committing sidecar import";
    default:
      return "Working";
  }
}

function operationLabel(operation: string): string {
  return busyLabel(operation);
}

function hasLocalLayer(layers: { visibility: "private" | "shared" | "imported"; trustState: "local" | "verified" | "unverified" | "rejected" }[]): boolean {
  return layers.length > 0;
}

function localLayerId(
  layers: { sidecarLayerId: string; visibility: "private" | "shared" | "imported" }[],
): string {
  const local = layers.find((layer) => layer.visibility === "private");
  if (local) {
    return local.sidecarLayerId;
  }
  if (layers.length > 0 && layers[0]) {
    return layers[0].sidecarLayerId;
  }
  return "";
}

function panelStyle(reduced: boolean): CSSProperties {
  return {
    background: "var(--ez-color-bg-glass-strong)",
    borderRadius: "var(--ez-radius-lg)",
    boxShadow: "var(--ez-shadow-panel)",
    padding: "var(--ez-space-5)",
    display: "flex",
    flexDirection: "column",
    gap: "var(--ez-space-4)",
    maxHeight: "85vh",
    overflowY: "auto",
    transition: reduced ? "none" : "box-shadow var(--ez-duration-normal)",
  };
}

const headerStyle: CSSProperties = {
  display: "flex",
  alignItems: "flex-start",
  justifyContent: "space-between",
  gap: "var(--ez-space-3)",
};

const headingStyle: CSSProperties = {
  margin: 0,
  fontSize: "1.05rem",
  lineHeight: 1.2,
};

const toolbarStyle: CSSProperties = {
  display: "flex",
  flexWrap: "wrap",
  gap: "var(--ez-space-2)",
};

const centeredStyle: CSSProperties = {
  display: "flex",
  justifyContent: "center",
  alignItems: "center",
  padding: "var(--ez-space-5)",
};

const hintStyle: CSSProperties = {
  margin: 0,
  color: "var(--ez-color-text-muted)",
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

const quoteStyle: CSSProperties = {
  margin: 0,
  background: "var(--ez-color-bg-elevated)",
  border: "1px solid var(--ez-color-border-subtle)",
  borderRadius: "var(--ez-radius-sm)",
  padding: "var(--ez-space-3)",
  fontFamily: "var(--ez-font-reader)",
  whiteSpace: "pre-wrap",
};

const textareaStyle: CSSProperties = {
  background: "var(--ez-color-bg-elevated)",
  border: "1px solid var(--ez-color-border-subtle)",
  borderRadius: "var(--ez-radius-sm)",
  padding: "var(--ez-space-3)",
  color: "var(--ez-color-text-primary)",
  fontFamily: "inherit",
  fontSize: "1rem",
  resize: "vertical",
  width: "100%",
};

// Keep the presentational VoiceRecorder import referenced for downstream
// container authors that prefer the controlled variant.
export const ControlledVoiceRecorder: typeof VoiceRecorder = VoiceRecorder;
