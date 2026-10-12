import type { CSSProperties, ReactNode } from "react";
import { useEffect, useState, useId } from "react";
import { AnnotationPanel } from "../../components/annotation/AnnotationPanel";
import { NotebookTree } from "../../components/annotation/NotebookTree";
import { TagFilter } from "../../components/annotation/TagFilter";
import { SidecarLayerToggle } from "../../components/annotation/SidecarLayerToggle";
import { SidecarImportDialog } from "../../components/annotation/SidecarImportDialog";
import { EzButton } from "../../components/annotation/sharedUi";
import { useAnnotationStore } from "../../stores/annotationStore";

/// `/notes` (§15.3): the notes browser. Every note the device holds —
/// typed, drawn, spoken, or filmed onto its ZIM packs — is listed here with
/// notebook/tag/layer filters and the sidecar import flow.
export function NotesView(): ReactNode {
  const headingId = useId();
  const status = useAnnotationStore((s) => s.status);
  const notebooks = useAnnotationStore((s) => s.notebooks);
  const tags = useAnnotationStore((s) => s.tags);
  const layers = useAnnotationStore((s) => s.layers);
  const sidecarPlan = useAnnotationStore((s) => s.sidecarPlan);

  const load = useAnnotationStore((s) => s.load);
  const reload = useAnnotationStore((s) => s.reload);
  const selectAnnotation = useAnnotationStore((s) => s.selectAnnotation);
  const remove = useAnnotationStore((s) => s.remove);
  const restore = useAnnotationStore((s) => s.restore);
  const createNotebook = useAnnotationStore((s) => s.createNotebook);
  const createTag = useAnnotationStore((s) => s.createTag);
  const chooseSidecarImportSource = useAnnotationStore(
    (s) => s.chooseSidecarImportSource,
  );
  const commitSidecarImport = useAnnotationStore(
    (s) => s.commitSidecarImport,
  );

  const [selectedNotebookIds, setSelectedNotebookIds] = useState<string[]>([]);
  const [selectedTagIds, setSelectedTagIds] = useState<string[]>([]);
  const [selectedLayerIds, setSelectedLayerIds] = useState<string[]>([]);
  const [sidecarOpen, setSidecarOpen] = useState(false);

  useEffect(() => {
    if (status.kind === "idle") {
      void load();
    }
  }, [status.kind, load]);

  const annotations = useAnnotationStore((s) => s.annotations);
  const visible = annotations.filter((annotation) => {
    if (
      selectedNotebookIds.length > 0 &&
      (!annotation.notebookId || !selectedNotebookIds.includes(annotation.notebookId))
    ) {
      return false;
    }
    if (
      selectedTagIds.length > 0 &&
      !annotation.tags.some((tag) => selectedTagIds.includes(tag.tagId))
    ) {
      return false;
    }
    if (
      selectedLayerIds.length > 0 &&
      !selectedLayerIds.includes(annotation.sidecarLayerId)
    ) {
      return false;
    }
    return true;
  });

  return (
    <div className="notes-view" style={viewStyle}>
      <header style={headerStyle}>
        <h2 id={headingId} style={headingStyle}>
          Notes
        </h2>
        <div style={{ display: "inline-flex", gap: "var(--ez-space-2)" }}>
          <EzButton
            variant="ghost"
            onClick={() => void reload()}
            ariaLabel="Reload notes"
          >
            ↻ Refresh
          </EzButton>
          <EzButton
            onClick={() => setSidecarOpen(true)}
            ariaLabel="Import a sidecar layer"
          >
            ⇪ Import sidecar
          </EzButton>
        </div>
      </header>
      <p style={introStyle}>
        Everything you have typed, drawn, spoken, or filmed onto your packs —
        plus any imported note layers — lives here.
      </p>

      <div style={columnsStyle}>
        <section aria-label="Filters" style={columnStyle}>
          <NotebookTree
            nodes={notebooks}
            selectedNotebookIds={selectedNotebookIds}
            onSelectionChange={setSelectedNotebookIds}
            onCreate={(parentNotebookId) => {
              const name = window.prompt("Notebook name");
              if (name && name.trim().length > 0) {
                void createNotebook({
                  parentNotebookId,
                  name: name.trim(),
                  description: "",
                });
              }
            }}
          />
          <TagFilter
            tags={tags}
            selectedTagIds={selectedTagIds}
            onChange={setSelectedTagIds}
            onCreateTag={(name, colorToken) => {
              void createTag({ name, colorToken });
            }}
          />
          <SidecarLayerToggle
            layers={layers}
            selectedLayerIds={selectedLayerIds}
            onChange={setSelectedLayerIds}
          />
        </section>
        <section aria-label="Notes list" style={listColumnStyle}>
          <AnnotationPanel
            annotations={visible}
            selectedAnnotationId={useAnnotationStore.getState().selectedAnnotationId}
            onSelect={(annotationId) => selectAnnotation(annotationId)}
            onDelete={(annotationId, revision) => {
              void remove(annotationId, revision);
            }}
            onRestore={(annotationId) => {
              void restore(annotationId);
            }}
          />
        </section>
      </div>

      <SidecarImportDialog
        open={sidecarOpen}
        plan={sidecarPlan}
        onChooseFile={() => {
          void chooseSidecarImportSource();
        }}
        onImport={(acceptUnsigned, layerName) => {
          void commitSidecarImport({ acceptUnsigned, layerName }).then(
            (layer) => {
              if (layer) {
                setSidecarOpen(false);
              }
            },
          );
        }}
        onCancel={() => setSidecarOpen(false)}
      />
    </div>
  );
}

const viewStyle: CSSProperties = {
  display: "flex",
  flexDirection: "column",
  gap: "var(--ez-space-4)",
  padding: "var(--ez-space-4)",
};

const headerStyle: CSSProperties = {
  display: "flex",
  alignItems: "center",
  justifyContent: "space-between",
};

const headingStyle: CSSProperties = {
  margin: 0,
};

const introStyle: CSSProperties = {
  margin: 0,
  color: "var(--ez-color-text-secondary)",
};

const columnsStyle: CSSProperties = {
  display: "flex",
  gap: "var(--ez-space-4)",
  alignItems: "flex-start",
  flexWrap: "wrap",
};

const columnStyle: CSSProperties = {
  flex: "0 1 18rem",
  display: "flex",
  flexDirection: "column",
  gap: "var(--ez-space-3)",
};

const listColumnStyle: CSSProperties = {
  flex: "1 1 24rem",
  minWidth: "min(24rem, 100%)",
};
