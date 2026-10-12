import type { JSX, ReactNode } from "react";
import { useEffect, useMemo, useState } from "react";
import {
  AnnotationWorkspace,
  type AnnotationTool,
} from "../../components/annotation/AnnotationWorkspace";
import { useAnnotationStore } from "../../stores/annotationStore";

export interface AnnotationDrawerProps {
  packId: string;
  articleKey: string;
  title: string;
}

const COLLAPSED_LABEL = "Notes & media";

/// The reader-page annotation surface: one workspace instance bound to the
/// currently-open article. All modalities (comment, ink, voice, film) anchor
/// to the article itself; text-selection anchoring arrives with the
/// highlighter bridge (ArticleContent iframe postMessage).
export function AnnotationDrawer(props: AnnotationDrawerProps): ReactNode {
  const [open, setOpen] = useState(true);
  const [activeTool, setActiveTool] = useState<AnnotationTool>("select");
  const status = useAnnotationStore((s) => s.status);
  const load = useAnnotationStore((s) => s.load);

  // Fetch the notes corpus once per mount; the workspace filters to this
  // article. A future iteration narrows this to list_for_article.
  useEffect(() => {
    if (status.kind === "idle") {
      void load();
    }
  }, [status.kind, load]);

  const articleContext = useMemo(
    () => ({
      packId: props.packId,
      articleKey: props.articleKey,
      title: props.title,
    }),
    [props.packId, props.articleKey, props.title],
  );

  if (!open) {
    return (
      <div style={railStyle}>
        <button
          type="button"
          className="reader-chip reader-chip--ghost"
          onClick={() => setOpen(true)}
          aria-expanded="false"
        >
          {COLLAPSED_LABEL}
        </button>
      </div>
    );
  }

  return (
    <aside aria-label="Article annotations" style={asideStyle}>
      <AnnotationWorkspaceForArticle
        article={articleContext}
        activeTool={activeTool}
        onToolChange={setActiveTool}
        onClose={() => setOpen(false)}
      />
    </aside>
  );
}

function AnnotationWorkspaceForArticle(props: {
  article: { packId: string; articleKey: string; title: string };
  activeTool: AnnotationTool;
  onToolChange: (tool: AnnotationTool) => void;
  onClose: () => void;
}): JSX.Element {
  // The workspace needs the full AnnotationRecord[] prop; select it
  // field-wise so unrelated store churn (e.g. notebooks) does not re-render
  // the article surface.
  const annotations = useAnnotationStore((s) => s.annotations);
  return (
    <AnnotationWorkspace
      article={props.article}
      annotations={annotations}
      activeTool={props.activeTool}
      onToolChange={props.onToolChange}
      onCreate={() => undefined}
      onUpdate={() => undefined}
      onClose={props.onClose}
    />
  );
}

const asideStyle = {
  marginTop: "var(--ez-space-4)",
  display: "flex",
  justifyContent: "center",
} as const;

const railStyle = {
  marginTop: "var(--ez-space-4)",
  display: "flex",
  justifyContent: "flex-start",
} as const;
