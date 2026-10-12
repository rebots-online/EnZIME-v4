import { invoke } from "@tauri-apps/api/core";

export type Ulid = string;
export type Sha256Hex = string;
export type UnixTimeMs = number;

export interface EmptyRequest {}

export interface CommandResult<T> {
  ok: boolean;
  value?: T;
  error?: AppErrorDto;
}

export type AppErrorCode =
  | "invalid_request"
  | "not_found"
  | "permission_denied"
  | "offline"
  | "storage_full"
  | "source_changed"
  | "verification_failed"
  | "signature_invalid"
  | "format_invalid"
  | "unsupported"
  | "revision_conflict"
  | "capability_denied"
  | "backend_unavailable"
  | "model_unavailable"
  | "cancelled"
  | "busy"
  | "internal";

export interface AppErrorDto {
  code: AppErrorCode;
  message: string;
  action?: ErrorActionDto;
  correlationId: string;
  detail?: Record<string, unknown>;
}

export interface ErrorActionDto {
  label: string;
  action:
    | "retry"
    | "open_settings"
    | "choose_storage"
    | "install_model"
    | "restore_purchase"
    | "view_quarantine"
    | "report_problem";
}

export interface PageResult<T> {
  items: T[];
  total?: number;
  nextOffset?: number;
}

export interface ArticleResponse {
  packId: Ulid;
  articleKey: string;
  title: string;
  sanitizedHtml: string;
  normalizedTextDigest: Sha256Hex;
  breadcrumb: BreadcrumbItem[];
  previousArticleKey?: string;
  nextArticleKey?: string;
  annotations: AnnotationRecord[];
}

export interface BreadcrumbItem {
  label: string;
  articleKey?: string;
}

export interface SelectorBundle {
  article: {
    packId: Ulid;
    articleKey: string;
    sourceRevisionDigest?: Sha256Hex;
  };
  textQuote?: {
    exact: string;
    prefix: string;
    suffix: string;
  };
  textPosition?: {
    normalizedStart: number;
    normalizedEnd: number;
  };
  domPath?: {
    path: string;
    nodeTextDigest: Sha256Hex;
  };
  imageRegion?: {
    resourceEntryKey: string;
    xFraction: number;
    yFraction: number;
    widthFraction: number;
    heightFraction: number;
  };
  capturedContext: string;
}

export interface AnnotationStyle {
  colorToken: string;
  emphasis: string;
  strokeWidth?: number;
}

export interface AnnotationDraft {
  annotationType:
    | "highlight"
    | "comment"
    | "ink"
    | "voice"
    | "bookmark_note";
  selector: SelectorBundle;
  bodyText?: string;
  style: AnnotationStyle;
  notebookId?: Ulid;
  tagIds: Ulid[];
  sidecarLayerId: Ulid;
}

export interface AnnotationResolution {
  state: "resolved" | "ambiguous" | "source_unavailable" | "orphaned";
  confidence: number;
  matchedStart?: number;
  matchedEnd?: number;
  explanation: string;
}

export interface AnnotationRecord {
  annotationId: Ulid;
  annotationType:
    | "highlight"
    | "comment"
    | "ink"
    | "voice"
    | "bookmark_note";
  selector: SelectorBundle;
  bodyText?: string;
  style: AnnotationStyle;
  notebookId?: Ulid;
  tags: TagSummary[];
  sidecarLayerId: Ulid;
  resolution: AnnotationResolution;
  pinned: boolean;
  revision: number;
  createdAtMs: UnixTimeMs;
  updatedAtMs: UnixTimeMs;
}

export interface TagSummary {
  tagId: Ulid;
  name: string;
  colorToken: string;
}

export interface NotebookNode {
  notebookId: Ulid;
  parentNotebookId?: Ulid;
  name: string;
  description: string;
  annotationCount: number;
  children: NotebookNode[];
}

export interface SidecarLayer {
  sidecarLayerId: Ulid;
  name: string;
  visibility: "private" | "shared" | "imported";
  trustState: "local" | "verified" | "unverified" | "rejected";
  enabled: boolean;
  annotationCount: number;
}

export interface AnnotationListRequest {
  packIds?: Ulid[];
  notebookIds?: Ulid[];
  tagIds?: Ulid[];
  sidecarLayerIds?: Ulid[];
  resolutionStates?: AnnotationResolution["state"][];
  text?: string;
  offset: number;
  limit: number;
}

export interface AnnotationCreateRequest {
  draft: AnnotationDraft;
}

export interface AnnotationUpdateRequest {
  annotationId: Ulid;
  expectedRevision: number;
  draft: AnnotationDraft;
}

export interface AnnotationDeleteRequest {
  annotationId: Ulid;
  expectedRevision: number;
}

export interface AnnotationRestoreRequest {
  annotationId: Ulid;
}

export interface AnnotationAssetBeginRequest {
  annotationId: Ulid;
  assetKind: "ink_svg" | "voice_opus" | "image" | "file";
  mimeType: string;
  expectedBytes: number;
  durationMs?: number;
}

export interface AnnotationAssetBeginResponse {
  uploadToken: string;
  maximumChunkBytes: number;
}

export interface AnnotationAssetChunkRequest {
  uploadToken: string;
  sequenceNo: number;
  bytesBase64: string;
}

export interface AnnotationAssetCommitRequest {
  uploadToken: string;
  expectedSha256: Sha256Hex;
}

export interface NotebookCreateRequest {
  parentNotebookId?: Ulid;
  name: string;
  description: string;
}

export interface NotebookUpdateRequest {
  notebookId: Ulid;
  name: string;
  description: string;
  parentNotebookId?: Ulid;
}

export interface TagCreateRequest {
  name: string;
  colorToken: string;
}

export interface SidecarConflict {
  entityKind: string;
  entityId: string;
  explanation: string;
}

export interface SidecarImportPlan {
  artifactSha256: Sha256Hex;
  signatureState: "verified" | "unsigned" | "invalid" | "unknown_key";
  signer?: string;
  annotationCount: number;
  notebookCount: number;
  tagCount: number;
  objectBytes: number;
  conflicts: SidecarConflict[];
}

export interface SidecarImportCommitRequest {
  bytesBase64: string;
  acceptUnsigned: boolean;
  layerName: string;
}

export interface ChosenSidecarFile {
  name: string;
  bytesBase64: string;
}

export class IpcError extends Error {
  readonly code: AppErrorCode;
  readonly action?: ErrorActionDto;
  readonly correlationId: string;
  readonly detail?: Record<string, unknown>;
  readonly raw: unknown;

  constructor(dto: AppErrorDto, raw: unknown) {
    super(dto.message);
    this.name = "IpcError";
    this.code = dto.code;
    this.action = dto.action;
    this.correlationId = dto.correlationId;
    this.detail = dto.detail;
    this.raw = raw;
  }
}

function unwrap<T>(payload: unknown): T {
  if (payload && typeof payload === "object" && "ok" in (payload as object)) {
    const result = payload as CommandResult<T>;
    if (result.ok) {
      return result.value as T;
    }
    if (result.error) {
      throw new IpcError(result.error, payload);
    }
    throw new IpcError(
      {
        code: "internal",
        message: "Command returned neither value nor error.",
        correlationId: "unknown",
      },
      payload,
    );
  }
  return payload as T;
}

async function call<TRequest, TResponse>(
  command: string,
  request: TRequest,
): Promise<TResponse> {
  let raw: unknown;
  try {
    raw = await invoke<unknown>(command, request as unknown as Record<string, unknown>);
  } catch (invokeError) {
    const normalised = normaliseInvokeError(invokeError);
    throw new IpcError(normalised.dto, invokeError);
  }
  return unwrap<TResponse>(raw);
}

function normaliseInvokeError(error: unknown): { dto: AppErrorDto } {
  if (error && typeof error === "object" && "code" in (error as object)) {
    const maybe = error as Partial<AppErrorDto> & { code: string };
    if (maybe.code && typeof maybe.message === "string") {
      return {
        dto: {
          code: maybe.code as AppErrorCode,
          message: maybe.message,
          action: maybe.action,
          correlationId: maybe.correlationId ?? "unknown",
          detail: maybe.detail,
        },
      };
    }
  }
  const message =
    error instanceof Error ? error.message : "IPC layer rejected the call.";
  return {
    dto: {
      code: inferCodeFromMessage(message),
      message,
      correlationId: "unknown",
    },
  };
}

function inferCodeFromMessage(message: string): AppErrorCode {
  const lower = message.toLowerCase();
  if (lower.includes("command") && lower.includes("not found")) {
    return "unsupported";
  }
  if (lower.includes("offline") || lower.includes("network")) {
    return "offline";
  }
  if (lower.includes("permission") || lower.includes("denied")) {
    return "capability_denied";
  }
  return "internal";
}

export const annotationService = {
  async list(request: AnnotationListRequest): Promise<PageResult<AnnotationRecord>> {
    return call<AnnotationListRequest, PageResult<AnnotationRecord>>(
      "annotation_list",
      request,
    );
  },
  async create(draft: AnnotationDraft): Promise<AnnotationRecord> {
    return call<AnnotationCreateRequest, AnnotationRecord>("annotation_create", {
      draft,
    });
  },
  async update(
    annotationId: Ulid,
    expectedRevision: number,
    draft: AnnotationDraft,
  ): Promise<AnnotationRecord> {
    return call<AnnotationUpdateRequest, AnnotationRecord>("annotation_update", {
      annotationId,
      expectedRevision,
      draft,
    });
  },
  async delete(annotationId: Ulid, expectedRevision: number): Promise<void> {
    await call<AnnotationDeleteRequest, void>("annotation_delete", {
      annotationId,
      expectedRevision,
    });
  },
  async restore(annotationId: Ulid): Promise<AnnotationRecord> {
    return call<AnnotationRestoreRequest, AnnotationRecord>(
      "annotation_restore",
      { annotationId },
    );
  },
  async assetBegin(
    request: AnnotationAssetBeginRequest,
  ): Promise<AnnotationAssetBeginResponse> {
    return call<AnnotationAssetBeginRequest, AnnotationAssetBeginResponse>(
      "annotation_asset_begin",
      request,
    );
  },
  async assetChunk(request: AnnotationAssetChunkRequest): Promise<void> {
    await call<AnnotationAssetChunkRequest, void>(
      "annotation_asset_chunk",
      request,
    );
  },
  async assetCommit(
    request: AnnotationAssetCommitRequest,
  ): Promise<AnnotationRecord> {
    return call<AnnotationAssetCommitRequest, AnnotationRecord>(
      "annotation_asset_commit",
      request,
    );
  },
  async notebookTree(): Promise<NotebookNode[]> {
    return call<EmptyRequest, NotebookNode[]>("notebook_tree", {});
  },
  async notebookCreate(request: NotebookCreateRequest): Promise<NotebookNode> {
    return call<NotebookCreateRequest, NotebookNode>("notebook_create", request);
  },
  async notebookUpdate(request: NotebookUpdateRequest): Promise<NotebookNode> {
    return call<NotebookUpdateRequest, NotebookNode>("notebook_update", request);
  },
  async tagList(): Promise<TagSummary[]> {
    return call<EmptyRequest, TagSummary[]>("tag_list", {});
  },
  async tagCreate(request: TagCreateRequest): Promise<TagSummary> {
    return call<TagCreateRequest, TagSummary>("tag_create", request);
  },
  async sidecarLayers(): Promise<SidecarLayer[]> {
    return call<EmptyRequest, SidecarLayer[]>("sidecar_layers", {});
  },
  async sidecarImportPlan(bytesBase64: string): Promise<SidecarImportPlan> {
    return call<{ bytesBase64: string }, SidecarImportPlan>(
      "sidecar_import_plan",
      { bytesBase64 },
    );
  },
  async sidecarImportCommit(
    request: SidecarImportCommitRequest,
  ): Promise<SidecarLayer> {
    return call<SidecarImportCommitRequest, SidecarLayer>(
      "sidecar_import_commit",
      request,
    );
  },
};

export async function uploadAnnotationAsset(params: {
  annotationId: Ulid;
  assetKind: "ink_svg" | "voice_opus" | "image" | "file";
  mimeType: string;
  durationMs?: number;
  bytes: Uint8Array;
  sha256Hex: Sha256Hex;
}): Promise<AnnotationRecord> {
  const begin = await annotationService.assetBegin({
    annotationId: params.annotationId,
    assetKind: params.assetKind,
    mimeType: params.mimeType,
    expectedBytes: params.bytes.byteLength,
    durationMs: params.durationMs,
  });
  const chunkSize = begin.maximumChunkBytes;
  if (!Number.isSafeInteger(chunkSize) || chunkSize <= 0) {
    throw new Error("The asset service returned an invalid chunk size.");
  }
  let sequenceNo = 0;
  for (let offset = 0; offset < params.bytes.byteLength; offset += chunkSize) {
    sequenceNo += 1;
    await annotationService.assetChunk({
      uploadToken: begin.uploadToken,
      sequenceNo,
      bytesBase64: base64FromBytes(params.bytes.subarray(offset, Math.min(offset + chunkSize, params.bytes.byteLength))),
    });
  }
  return annotationService.assetCommit({
    uploadToken: begin.uploadToken,
    expectedSha256: params.sha256Hex,
  });
}

function base64FromBytes(bytes: Uint8Array): string {
  const chunk = 0x8000;
  let binary = "";
  for (let i = 0; i < bytes.byteLength; i += chunk) {
    const slice = bytes.subarray(i, Math.min(i + chunk, bytes.byteLength));
    binary += String.fromCharCode(...slice);
  }
  if (typeof btoa === "function") {
    return btoa(binary);
  }
  const buf = Buffer.from(binary, "binary");
  return buf.toString("base64");
}

const SHA256 = "SHA-256" as const;

export async function sha256Hex(bytes: Uint8Array): Promise<Sha256Hex> {
  if (typeof crypto !== "undefined" && typeof crypto.subtle !== "undefined") {
    const digest = await crypto.subtle.digest(SHA256, bytes as BufferSource);
    return hexFromBuffer(digest);
  }
  throw new IpcError(
    {
      code: "unsupported",
      message: "SHA-256 is not available in this environment.",
      correlationId: "unknown",
    },
    null,
  );
}

function hexFromBuffer(buffer: ArrayBuffer): Sha256Hex {
  const bytes = new Uint8Array(buffer);
  let hex = "";
  for (let i = 0; i < bytes.byteLength; i += 1) {
    const value = bytes[i] as number;
    hex += value.toString(16).padStart(2, "0");
  }
  return hex;
}

export function chooseSidecarFile(): Promise<ChosenSidecarFile | null> {
  return new Promise((resolve) => {
    if (typeof document === "undefined") {
      resolve(null);
      return;
    }
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".sidecar,.cbor,application/octet-stream";
    input.onchange = () => {
      const file = input.files?.[0];
      if (!file) {
        resolve(null);
        return;
      }
      const reader = new FileReader();
      reader.onload = () => {
        resolve({
          name: file.name,
          bytesBase64: arrayBufferToBase64(reader.result as ArrayBuffer),
        });
      };
      reader.onerror = () => resolve(null);
      reader.readAsArrayBuffer(file);
    };
    input.oncancel = () => resolve(null);
    input.click();
  });
}

function arrayBufferToBase64(buffer: ArrayBuffer): string {
  const bytes = new Uint8Array(buffer);
  let binary = "";
  const chunk = 0x8000;
  for (let offset = 0; offset < bytes.byteLength; offset += chunk) {
    binary += String.fromCharCode(
      ...bytes.subarray(offset, Math.min(offset + chunk, bytes.byteLength)),
    );
  }
  if (typeof btoa === "function") {
    return btoa(binary);
  }
  return Buffer.from(binary, "binary").toString("base64");
}
