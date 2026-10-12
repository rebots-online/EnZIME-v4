import { create } from "zustand";
import {
  annotationService,
  IpcError,
  sha256Hex,
  uploadAnnotationAsset,
  type AnnotationDraft,
  type AnnotationListRequest,
  type AnnotationRecord,
  type AppErrorDto,
  type NotebookCreateRequest,
  type NotebookNode,
  type NotebookUpdateRequest,
  type SidecarImportPlan,
  type SidecarLayer,
  type TagCreateRequest,
  type TagSummary,
  type Ulid,
  chooseSidecarFile,
} from "../services/annotationService";

export type LoadStatus =
  | { kind: "idle" }
  | { kind: "loading" }
  | { kind: "loaded" }
  | { kind: "empty" }
  | { kind: "recoverable_error"; error: AppErrorDto }
  | { kind: "non_recoverable_error"; error: AppErrorDto }
  | { kind: "offline"; message: string }
  | { kind: "capability_denied"; error: AppErrorDto };

export type MutationStatus =
  | { kind: "idle" }
  | { kind: "busy"; operation: string }
  | { kind: "error"; error: AppErrorDto; operation: string };

function classifyError(error: unknown): LoadStatus {
  if (error instanceof IpcError) {
    const dto: AppErrorDto = {
      code: error.code,
      message: error.message,
      action: error.action,
      correlationId: error.correlationId,
      detail: error.detail,
    };
    if (dto.code === "offline") {
      return { kind: "offline", message: dto.message };
    }
    if (dto.code === "capability_denied") {
      return { kind: "capability_denied", error: dto };
    }
    if (dto.code === "internal" || dto.code === "unsupported") {
      return { kind: "non_recoverable_error", error: dto };
    }
    return { kind: "recoverable_error", error: dto };
  }
  return {
    kind: "non_recoverable_error",
    error: {
      code: "internal",
      message:
        error instanceof Error ? error.message : "Unexpected failure.",
      correlationId: "unknown",
    },
  };
}

function classifyMutation(error: unknown, operation: string): MutationStatus {
  if (error instanceof IpcError) {
    return {
      kind: "error",
      operation,
      error: {
        code: error.code,
        message: error.message,
        action: error.action,
        correlationId: error.correlationId,
        detail: error.detail,
      },
    };
  }
  return {
    kind: "error",
    operation,
    error: {
      code: "internal",
      message:
        error instanceof Error ? error.message : "Unexpected failure.",
      correlationId: "unknown",
    },
  };
}

export const DEFAULT_FILTER: AnnotationListRequest = {
  offset: 0,
  limit: 100,
};

const HIGHLIGHT_COLOR_TOKENS = [
  "highlight-yellow",
  "highlight-mint",
  "highlight-pink",
  "highlight-blue",
] as const;

export const HIGHLIGHT_COLORS: readonly string[] = HIGHLIGHT_COLOR_TOKENS;

export interface AnnotationStoreState {
  annotations: AnnotationRecord[];
  notebooks: NotebookNode[];
  tags: TagSummary[];
  layers: SidecarLayer[];
  sidecarFile?: { name: string; bytesBase64: string };
  sidecarPlan?: SidecarImportPlan;
  filter: AnnotationListRequest;
  selectedAnnotationId?: Ulid;
  status: LoadStatus;
  mutation: MutationStatus;
  load: () => Promise<void>;
  reload: () => Promise<void>;
  create: (draft: AnnotationDraft) => Promise<AnnotationRecord | undefined>;
  update: (
    annotationId: Ulid,
    expectedRevision: number,
    draft: AnnotationDraft,
  ) => Promise<AnnotationRecord | undefined>;
  remove: (
    annotationId: Ulid,
    expectedRevision: number,
  ) => Promise<boolean>;
  restore: (annotationId: Ulid) => Promise<AnnotationRecord | undefined>;
  uploadInk: (
    annotationId: Ulid,
    svg: string,
    colorToken: string,
    strokeWidth: number,
  ) => Promise<AnnotationRecord | undefined>;
  uploadVoice: (
    annotationId: Ulid,
    bytes: Uint8Array,
    durationMs: number,
    mimeType: string,
  ) => Promise<AnnotationRecord | undefined>;
  uploadVideo: (
    annotationId: Ulid,
    bytes: Uint8Array,
    mimeType: string,
    durationMs: number,
  ) => Promise<AnnotationRecord | undefined>;
  createNotebook: (
    request: NotebookCreateRequest,
  ) => Promise<NotebookNode | undefined>;
  updateNotebook: (
    request: NotebookUpdateRequest,
  ) => Promise<NotebookNode | undefined>;
  createTag: (request: TagCreateRequest) => Promise<TagSummary | undefined>;
  setFilter: (filter: AnnotationListRequest) => void;
  selectAnnotation: (annotationId?: Ulid) => void;
  clearMutation: () => void;
  chooseSidecarImportSource: () => Promise<SidecarImportPlan | undefined>;
  commitSidecarImport: (request: {
    acceptUnsigned: boolean;
    layerName: string;
  }) => Promise<SidecarLayer | undefined>;
}

function refreshLists(): Promise<{
  annotations: AnnotationRecord[];
  notebooks: NotebookNode[];
  tags: TagSummary[];
  layers: SidecarLayer[];
}> {
  return Promise.all([
    annotationService.list({ offset: 0, limit: 200 }),
    annotationService.notebookTree(),
    annotationService.tagList(),
    annotationService.sidecarLayers(),
  ]).then(([page, notebooks, tags, layers]) => ({
    annotations: page.items,
    notebooks,
    tags,
    layers,
  }));
}

function upsertAnnotation(
  annotations: AnnotationRecord[],
  record: AnnotationRecord,
): AnnotationRecord[] {
  const index = annotations.findIndex(
    (annotation) => annotation.annotationId === record.annotationId,
  );
  if (index === -1) {
    return [record, ...annotations];
  }
  const next = annotations.slice();
  next[index] = record;
  return next;
}

export const useAnnotationStore = create<AnnotationStoreState>((set, get) => ({
  annotations: [],
  notebooks: [],
  tags: [],
  layers: [],
  sidecarFile: undefined,
  sidecarPlan: undefined,
  filter: { ...DEFAULT_FILTER },
  selectedAnnotationId: undefined,
  status: { kind: "idle" },
  mutation: { kind: "idle" },

  async load() {
    const current = get().status;
    if (current.kind === "loading") {
      return;
    }
    set({ status: { kind: "loading" } });
    try {
      const result = await refreshLists();
      const annotations = applyFilter(result.annotations, get().filter);
      set({
        annotations: result.annotations,
        notebooks: result.notebooks,
        tags: result.tags,
        layers: result.layers,
        status: annotations.length === 0 ? { kind: "empty" } : { kind: "loaded" },
      });
    } catch (error) {
      set({ status: classifyError(error) });
    }
  },

  async reload() {
    set({ status: { kind: "loading" }, mutation: { kind: "idle" } });
    try {
      const result = await refreshLists();
      const annotations = applyFilter(result.annotations, get().filter);
      set({
        annotations: result.annotations,
        notebooks: result.notebooks,
        tags: result.tags,
        layers: result.layers,
        status: annotations.length === 0 ? { kind: "empty" } : { kind: "loaded" },
      });
    } catch (error) {
      set({ status: classifyError(error) });
    }
  },

  async create(draft) {
    set({ mutation: { kind: "busy", operation: "create" } });
    try {
      const created = await annotationService.create(draft);
      // Local apply + one cheap tree refresh (notebook counts changed);
      // the previous 4-command refetch per mutation is gone.
      set((current) => ({
        annotations: upsertAnnotation(current.annotations, created),
        status: { kind: "loaded" },
        mutation: { kind: "idle" },
      }));
      // A committed annotation remains successful even if count refresh fails.
      void annotationService.notebookTree().then(notebooks => set({ notebooks })).catch(() => undefined);
      return created;
    } catch (error) {
      set({ mutation: classifyMutation(error, "create") });
      return undefined;
    }
  },

  async update(annotationId, expectedRevision, draft) {
    set({ mutation: { kind: "busy", operation: "update" } });
    try {
      const updated = await annotationService.update(
        annotationId,
        expectedRevision,
        draft,
      );
      // Pure local apply — nothing else in the corpus changed.
      set((current) => ({
        annotations: upsertAnnotation(current.annotations, updated),
        status: { kind: "loaded" },
        mutation: { kind: "idle" },
      }));
      return updated;
    } catch (error) {
      set({ mutation: classifyMutation(error, "update") });
      return undefined;
    }
  },

  async remove(annotationId, expectedRevision) {
    set({ mutation: { kind: "busy", operation: "delete" } });
    try {
      await annotationService.delete(annotationId, expectedRevision);
      set((current) => ({
        annotations: current.annotations.filter(
          (annotation) => annotation.annotationId !== annotationId,
        ),
        mutation: { kind: "idle" },
      }));
      void annotationService.notebookTree().then(notebooks => set({ notebooks })).catch(() => undefined);
      return true;
    } catch (error) {
      set({ mutation: classifyMutation(error, "delete") });
      return false;
    }
  },

  async restore(annotationId) {
    set({ mutation: { kind: "busy", operation: "restore" } });
    try {
      const restored = await annotationService.restore(annotationId);
      set((current) => ({
        annotations: upsertAnnotation(current.annotations, restored),
        mutation: { kind: "idle" },
      }));
      void annotationService.notebookTree().then(notebooks => set({ notebooks })).catch(() => undefined);
      return restored;
    } catch (error) {
      set({ mutation: classifyMutation(error, "restore") });
      return undefined;
    }
  },

  async uploadInk(annotationId, svg, _colorToken, _strokeWidth) {
    set({ mutation: { kind: "busy", operation: "asset_upload" } });
    try {
      const bytes = new TextEncoder().encode(svg);
      const digest = await sha256Hex(bytes);
      const record = await uploadAnnotationAsset({
        annotationId,
        assetKind: "ink_svg",
        mimeType: "image/svg+xml; charset=utf-8",
        bytes,
        sha256Hex: digest,
      });
      // Style is already part of the created draft. The committed asset is
      // authoritative; an auxiliary refresh cannot turn it into upload failure.
      set(current => ({ annotations: upsertAnnotation(current.annotations, record), status: { kind: "loaded" }, mutation: { kind: "idle" } }));
      return record;
    } catch (error) {
      set({ mutation: classifyMutation(error, "asset_upload") });
      return undefined;
    }
  },

  async uploadVoice(annotationId, bytes, durationMs, mimeType) {
    if (!/^audio\/(ogg|webm);\s*codecs=opus$/i.test(mimeType) || bytes.byteLength > 32 * 1024 * 1024) {
      set({ mutation: classifyMutation(new Error("Unsupported Opus MIME or voice capture exceeds 32 MiB."), "asset_upload") });
      return undefined;
    }
    set({ mutation: { kind: "busy", operation: "asset_upload" } });
    try {
      const digest = await sha256Hex(bytes);
      const record = await uploadAnnotationAsset({
        annotationId,
        assetKind: "voice_opus",
        mimeType,
        bytes,
        sha256Hex: digest,
        durationMs,
      });
      set(current => ({ annotations: upsertAnnotation(current.annotations, record), status: { kind: "loaded" }, mutation: { kind: "idle" } }));
      return record;
    } catch (error) {
      set({ mutation: classifyMutation(error, "asset_upload") });
      return undefined;
    }
  },

  async createNotebook(request) {
    set({ mutation: { kind: "busy", operation: "notebook_create" } });
    try {
      const node = await annotationService.notebookCreate(request);
      const notebooks = await annotationService.notebookTree();
      set({
        notebooks,
        mutation: { kind: "idle" },
      });
      return node;
    } catch (error) {
      set({ mutation: classifyMutation(error, "notebook_create") });
      return undefined;
    }
  },

  async updateNotebook(request) {
    set({ mutation: { kind: "busy", operation: "notebook_update" } });
    try {
      const node = await annotationService.notebookUpdate(request);
      const notebooks = await annotationService.notebookTree();
      set({
        notebooks,
        mutation: { kind: "idle" },
      });
      return node;
    } catch (error) {
      set({ mutation: classifyMutation(error, "notebook_update") });
      return undefined;
    }
  },

  async createTag(request) {
    set({ mutation: { kind: "busy", operation: "tag_create" } });
    try {
      const tag = await annotationService.tagCreate(request);
      const tags = await annotationService.tagList();
      set({
        tags,
        mutation: { kind: "idle" },
      });
      return tag;
    } catch (error) {
      set({ mutation: classifyMutation(error, "tag_create") });
      return undefined;
    }
  },

  setFilter(filter) {
    const annotations = applyFilter(get().annotations, filter);
    set({
      filter,
      status:
        get().status.kind === "loaded" || get().status.kind === "empty"
          ? annotations.length === 0
            ? { kind: "empty" }
            : { kind: "loaded" }
          : get().status,
    });
  },

  selectAnnotation(annotationId) {
    set({ selectedAnnotationId: annotationId });
  },

  clearMutation() {
    set({ mutation: { kind: "idle" } });
  },

  async uploadVideo(annotationId, bytes, mimeType, durationMs) {
    set({ mutation: { kind: "busy", operation: "asset_upload" } });
    try {
      const digest = await sha256Hex(bytes);
      const record = await uploadAnnotationAsset({
        annotationId,
        assetKind: "file",
        mimeType,
        durationMs,
        bytes,
        sha256Hex: digest,
      });
      set((current) => ({
        annotations: upsertAnnotation(current.annotations, record),
        mutation: { kind: "idle" },
      }));
      return record;
    } catch (error) {
      set({ mutation: classifyMutation(error, "asset_upload") });
      return undefined;
    }
  },

  async chooseSidecarImportSource() {
    set({ mutation: { kind: "busy", operation: "sidecar_choose" } });
    try {
      const file = await chooseSidecarFile();
      if (!file) {
        set({ mutation: { kind: "idle" } });
        return undefined;
      }
      const plan = await annotationService.sidecarImportPlan(file.bytesBase64);
      set({
        sidecarFile: file,
        sidecarPlan: plan,
        mutation: { kind: "idle" },
      });
      return plan;
    } catch (error) {
      set({ mutation: classifyMutation(error, "sidecar_choose") });
      return undefined;
    }
  },

  async commitSidecarImport(request) {
    set({ mutation: { kind: "busy", operation: "sidecar_commit" } });
    const file = get().sidecarFile;
    if (!file) {
      set({
        mutation: classifyMutation(
          new Error("no sidecar file chosen"),
          "sidecar_commit",
        ),
      });
      return undefined;
    }
    try {
      const layer = await annotationService.sidecarImportCommit({
        bytesBase64: file.bytesBase64,
        acceptUnsigned: request.acceptUnsigned,
        layerName: request.layerName,
      });
      const layers = await annotationService.sidecarLayers();
      set({
        layers,
        sidecarFile: undefined,
        sidecarPlan: undefined,
        mutation: { kind: "idle" },
      });
      return layer;
    } catch (error) {
      set({ mutation: classifyMutation(error, "sidecar_commit") });
      return undefined;
    }
  },
}));

function applyFilter(
  annotations: AnnotationRecord[],
  filter: AnnotationListRequest,
): AnnotationRecord[] {
  const packIds = filter.packIds ? new Set(filter.packIds) : undefined;
  const notebookIds = filter.notebookIds
    ? new Set(filter.notebookIds)
    : undefined;
  const tagIds = filter.tagIds ? new Set(filter.tagIds) : undefined;
  const sidecarLayerIds = filter.sidecarLayerIds
    ? new Set(filter.sidecarLayerIds)
    : undefined;
  const resolutionStates = filter.resolutionStates
    ? new Set(filter.resolutionStates)
    : undefined;
  const text = filter.text?.trim().toLowerCase();
  return annotations.filter((annotation) => {
    if (packIds && !packIds.has(annotation.selector.article.packId)) {
      return false;
    }
    if (notebookIds && (!annotation.notebookId || !notebookIds.has(annotation.notebookId))) {
      return false;
    }
    if (tagIds && !annotation.tags.some((tag) => tagIds.has(tag.tagId))) {
      return false;
    }
    if (sidecarLayerIds && !sidecarLayerIds.has(annotation.sidecarLayerId)) {
      return false;
    }
    if (
      resolutionStates &&
      !resolutionStates.has(annotation.resolution.state)
    ) {
      return false;
    }
    if (text) {
      const haystack = [
        annotation.bodyText ?? "",
        annotation.selector.capturedContext ?? "",
        annotation.selector.textQuote?.exact ?? "",
      ]
        .join(" ")
        .toLowerCase();
      if (!haystack.includes(text)) {
        return false;
      }
    }
    return true;
  });
}
