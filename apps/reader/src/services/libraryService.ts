// libraryService — Tauri bridge for the Library vertical (§14.4 / §24 binding).
//
// This is the user-facing entry to EnZIME's DynDon download — the USP:
// "download only what fits your storage". The service invokes the Rust-side
// `storage_roots_list` / `pack_list` / `catalog_search` / `install_plan` /
// `install_start` / `job_list` commands by their exact §14.4 names with
// camelCase args. Errors thrown by `invoke` are classified into the same
// `ServiceError` shape the Home and Reader verticals use, so the shared
// `resolveState` / `StatusBanner` primitives compose without adaptation.
//
// `tsc` / `vite` do not resolve Tauri command names at build time, so this
// file builds green regardless of handler availability on the Rust side.

import { invoke } from "@tauri-apps/api/core";

// ─── Paging types ───────────────────────────────────────────────────────────
// homeService does not export PageRequest / PageResult (it has no paged
// commands); annotationService does, with the shape `{ items, total?,
// nextOffset? }` / `{ offset, limit }`. We mirror that exact shape here so the
// contract is consistent across verticals.

export interface PageRequest {
  readonly offset?: number;
  readonly limit?: number;
}

export interface PageResult<T> {
  readonly items: readonly T[];
  readonly total?: number | null;
  readonly nextOffset?: number | null;
}

// ─── DTOs (§14.4 contract) ──────────────────────────────────────────────────

export type StorageDestination = {
  readonly storageRootId: string;
  readonly displayName: string;
  readonly writable: boolean;
};

export type PackManifest = {
  readonly packId: string;
  readonly displayName: string;
  readonly description: string;
  readonly languageBcp47: string;
  readonly articleCount: number;
  readonly sizeBytes: number;
  readonly lastOpenedAt?: string;
};

export type CatalogEntry = {
  readonly catalogEntryId: string;
  readonly title: string;
  readonly description: string;
  readonly languageBcp47: string;
  readonly category: string;
  readonly publisher: string;
  readonly artifactUrl: string;
  readonly artifactSizeBytes: number;
  readonly artifactSha256: string | null;
};

/** Argument shape for `catalog_search` (§14.4). Optional filters are omitted
 *  from the wire payload when absent so the Rust side can default them. */
export type CatalogSearchRequest = {
  readonly query: string;
  readonly languageBcp47?: string;
  readonly category?: string;
  readonly offset?: number;
  readonly limit?: number;
};

export interface CatalogPage extends PageResult<CatalogEntry> { readonly source: "live" | "cache"; readonly notice?: string | null }

export type TransferMode = "ranged" | "full" | "local_copy";

export type InstallPlanVerification = {
  readonly expectedSizeBytes: number;
  readonly expectedSha256: string | null;
  readonly signatureRequired: boolean;
  readonly zimStructureCheck: boolean;
};

export type InstallPlan = {
  readonly installPlanId: string;
  readonly catalogEntry: CatalogEntry;
  readonly destination: StorageDestination;
  readonly transferMode: TransferMode;
  readonly downloadBytes: number;
  readonly temporaryBytes: number;
  readonly finalBytes: number;
  readonly freeBytesBefore: number;
  readonly freeBytesAfter: number;
  readonly verification: InstallPlanVerification;
};

export type DownloadJobState =
  | "planned"
  | "fetching"
  | "paused"
  | "verifying"
  | "staged"
  | "activating"
  | "completed"
  | "failed"
  | "cancelled";

export type DownloadJob = {
  readonly downloadJobId: string;
  readonly state: DownloadJobState;
  readonly bytesVerified: number;
  readonly bytesRetained: number;
  readonly transferScope: "partial" | "full";
  readonly errorDetail?: string | null;
  readonly expectedSizeBytes: number;
};

// ─── Error classification (mirrors homeService / readerService verbatim) ────

export type ServiceErrorKind =
  | "recoverable"
  | "non-recoverable"
  | "capability-denied";

export type ServiceError = {
  readonly kind: ServiceErrorKind;
  readonly message: string;
  readonly cause: unknown;
};

function stringifyError(raw: unknown): string {
  if (typeof raw === "string") return raw;
  if (raw instanceof Error) return raw.message;
  if (raw !== null && typeof raw === "object") {
    const record = raw as { message?: unknown };
    if (typeof record.message === "string") return record.message;
  }
  return "Unknown service error";
}

function classifyError(raw: unknown): ServiceError {
  const message = stringifyError(raw);
  if (/capability|denied|permission|forbidden/i.test(message)) {
    return { kind: "capability-denied", message, cause: raw };
  }
  if (/non[- ]recoverable|fatal|corrupt/i.test(message)) {
    return { kind: "non-recoverable", message, cause: raw };
  }
  return { kind: "recoverable", message, cause: raw };
}

async function call<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (raw) {
    throw classifyError(raw);
  }
}

// ─── Service surface ────────────────────────────────────────────────────────

/** Strip `undefined` optionals so they are absent on the wire (not `null`),
 *  which keeps the Rust serde struct parsing strict. */
function compactArgs(
  record: Record<string, unknown>,
): Record<string, unknown> {
  const out: Record<string, unknown> = {};
  for (const [key, value] of Object.entries(record)) {
    if (value !== undefined) out[key] = value;
  }
  return out;
}

function pageArgs(page: PageRequest): Record<string, unknown> {
  return compactArgs({ offset: page.offset, limit: page.limit });
}

export const libraryService = {
  /** Lists writable storage destinations the user can install a pack into. */
  storageRootsList(): Promise<StorageDestination[]> {
    return call<StorageDestination[]>("storage_roots_list");
  },

  /** Pages the manifests of packs already installed in this profile. */
  packList(page: PageRequest): Promise<PageResult<PackManifest>> {
    return call<PageResult<PackManifest>>("pack_list", pageArgs(page));
  },

  /** Searches the public DynDon catalog for downloadable packs. */
  catalogSearch(
    request: CatalogSearchRequest,
  ): Promise<CatalogPage> {
    return call<CatalogPage>(
      "catalog_search",
      compactArgs({
        query: request.query,
        languageBcp47: request.languageBcp47,
        category: request.category,
        offset: request.offset,
        limit: request.limit,
      }),
    );
  },

  /** Computes an install plan: the storage fit for the chosen destination,
   *  including the DynDon subset decision (`downloadBytes` may be less than
   *  `artifactSizeBytes` when the full pack would not fit). */
  installPlan(args: {
    catalogEntryId: string;
    destinationStorageRootId: string;
    selectionProfileId?: string;
    budgetBytes?: number;
  }): Promise<InstallPlan> {
    return call<InstallPlan>(
      "install_plan",
      compactArgs({
        catalogEntryId: args.catalogEntryId,
        destinationStorageRootId: args.destinationStorageRootId,
        selectionProfileId: args.selectionProfileId,
        budgetBytes: args.budgetBytes,
      }),
    );
  },

  /** Starts a download job for a computed install plan; returns the initial
   *  `DownloadJob` row (state typically `planned` → `fetching`). */
  installStart(args: { installPlanId: string }): Promise<DownloadJob> {
    return call<DownloadJob>("install_start", {
      installPlanId: args.installPlanId,
    });
  },

  installCancel(args: { installPlanId: string }): Promise<void> {
    return call<void>("install_cancel", { installPlanId: args.installPlanId });
  },

  /** Pages active and recent download jobs for the queue view. */
  jobList(page: PageRequest): Promise<PageResult<DownloadJob>> {
    return call<PageResult<DownloadJob>>("job_list", pageArgs(page));
  },
};

export type LibraryService = typeof libraryService;
