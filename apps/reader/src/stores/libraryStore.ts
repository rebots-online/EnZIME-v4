// libraryStore — zustand store for the Library vertical (§15.1 / §15.3
// `/library`).
//
// Mirrors the homeStore slice pattern verbatim: `Slice<T>` with
// `idle|loading|ready|error` status, `emptySlice` / `startLoading` /
// `finishData` / `finishError` helpers, and an `online` flag bound by the
// view layer via `setOnline` (driven by `useOnlineSubscription`).
//
// `loadAll()` fans out the three independent reads (storage roots, installed
// packs, active downloads) in parallel — each slice resolves independently so
// a failure in one (e.g. catalog offline) does not block the others. The
// catalog search slice is query-driven and loaded on demand by
// `searchCatalog()`.

import { create } from "zustand";
import {
  libraryService,
  type CatalogEntry,
  type CatalogSearchRequest,
  type DownloadJob,
  type InstallPlan,
  type PackManifest,
  type PageRequest,
  type ServiceError,
  type StorageDestination,
} from "../services/libraryService";

export type SliceStatus = "idle" | "loading" | "ready" | "error";

export type Slice<T> = {
  readonly status: SliceStatus;
  readonly data: T | null;
  readonly error: ServiceError | null;
  readonly lastUpdated: string | null;
};

function emptySlice<T>(): Slice<T> {
  return { status: "idle", data: null, error: null, lastUpdated: null };
}

function startLoading<T>(slice: Slice<T>): Slice<T> {
  return { ...slice, status: "loading", error: null };
}

function finishData<T>(data: T): Slice<T> {
  return {
    status: "ready",
    data,
    error: null,
    lastUpdated: new Date().toISOString(),
  };
}

function finishError<T>(slice: Slice<T>, error: ServiceError): Slice<T> {
  return { ...slice, status: "error", error };
}

const DEFAULT_PAGE_LIMIT = 24;

function pageOf(page: PageRequest): Required<Pick<PageRequest, "offset" | "limit">> {
  return {
    offset: page.offset ?? 0,
    limit: page.limit ?? DEFAULT_PAGE_LIMIT,
  };
}

export interface LibraryState {
  online: boolean;

  storageRoots: Slice<readonly StorageDestination[]>;
  packs: Slice<readonly PackManifest[]>;
  catalogEntries: Slice<readonly CatalogEntry[]>;
  catalogSource: "live" | "cache" | null;
  catalogNotice: string | null;
  catalogTotal: number | null;
  catalogNextOffset: number | null;
  catalogRequest: CatalogSearchRequest | null;
  loadMoreCatalog: () => Promise<void>;
  installStarting: boolean;
  installPlan: Slice<InstallPlan | null>;
  downloads: Slice<readonly DownloadJob[]>;

  /** The destination the user has chosen for the in-flight plan, if any. */
  selectedStorageRootId: string | null;
  /** The catalog entry currently being planned, if any. */
  activeCatalogEntryId: string | null;

  setOnline: (online: boolean) => void;
  setSelectedStorageRoot: (storageRootId: string | null) => void;
  setActiveCatalogEntry: (catalogEntryId: string | null) => void;

  loadStorageRoots: () => Promise<void>;
  loadPacks: () => Promise<void>;
  loadDownloads: () => Promise<void>;
  searchCatalog: (request: CatalogSearchRequest) => Promise<void>;

  /** Computes an install plan for `catalogEntryId` into `storageRootId`.
   *  Stores the result in `installPlan` (or an error on that slice). */
  computePlan: (
    catalogEntryId: string,
    storageRootId: string,
    budgetBytes?: number,
  ) => Promise<void>;

  /** Starts the install for a computed plan, then refreshes the download
   *  queue so the new job appears immediately. */
  startInstall: (installPlanId: string) => Promise<boolean>;

  clearInstallPlan: () => void;
  loadAll: () => Promise<void>;
}

function initialOnline(): boolean {
  if (typeof navigator === "undefined") return true;
  return navigator.onLine;
}

function unwrap<T>(page: { items?: readonly T[] } | null | undefined): T[] {
  if (!page) return [];
  // Tauri serialises Vec<T> as a JSON array; some handlers may return the
  // page wrapper, others a bare array. Accept both shapes defensively so a
  // backend evolution does not blank the list.
  if (Array.isArray(page)) return page as unknown as T[];
  return Array.isArray(page.items) ? [...page.items] : [];
}

let catalogGeneration = 0;
let planGeneration = 0;
let pageLoading = false;

export const useLibraryStore = create<LibraryState>((set, get) => ({
  online: initialOnline(),

  storageRoots: emptySlice<readonly StorageDestination[]>(),
  packs: emptySlice<readonly PackManifest[]>(),
  catalogEntries: emptySlice<readonly CatalogEntry[]>(),
  catalogSource: null, catalogNotice: null, catalogTotal: null, catalogNextOffset: null, catalogRequest: null, installStarting: false,
  installPlan: emptySlice<InstallPlan | null>(),
  downloads: emptySlice<readonly DownloadJob[]>(),

  selectedStorageRootId: null,
  activeCatalogEntryId: null,

  setOnline(online) {
    if (get().online === online) return;
    set({ online });
  },

  setSelectedStorageRoot(storageRootId) {
    if (get().installStarting || get().selectedStorageRootId === storageRootId) return;
    planGeneration++;
    set({ selectedStorageRootId: storageRootId, installPlan: emptySlice<InstallPlan | null>() });
  },

  setActiveCatalogEntry(catalogEntryId) {
    if (get().installStarting) return;
    planGeneration++;
    set({ activeCatalogEntryId: catalogEntryId, installPlan: emptySlice<InstallPlan | null>() });
  },

  loadStorageRoots() {
    set({ storageRoots: startLoading(get().storageRoots) });
    return libraryService
      .storageRootsList()
      .then((roots) => {
        set({ storageRoots: finishData<readonly StorageDestination[]>(roots) });
      })
      .catch((error: ServiceError) => {
        set({ storageRoots: finishError(get().storageRoots, error) });
      });
  },

  loadPacks() {
    set({ packs: startLoading(get().packs) });
    return libraryService
      .packList(pageOf({ limit: DEFAULT_PAGE_LIMIT }))
      .then((page) => {
        set({ packs: finishData<readonly PackManifest[]>(unwrap(page)) });
      })
      .catch((error: ServiceError) => {
        set({ packs: finishError(get().packs, error) });
      });
  },

  loadDownloads() {
    set({ downloads: startLoading(get().downloads) });
    return libraryService
      .jobList(pageOf({ limit: DEFAULT_PAGE_LIMIT }))
      .then((page) => {
        set({ downloads: finishData<readonly DownloadJob[]>(unwrap(page)) });
      })
      .catch((error: ServiceError) => {
        set({ downloads: finishError(get().downloads, error) });
      });
  },

  async searchCatalog(request) {
    const generation = ++catalogGeneration; pageLoading = false;
    const query = { ...request, offset: 0, limit: request.limit ?? DEFAULT_PAGE_LIMIT };
    set({ catalogRequest: query, catalogEntries: startLoading(emptySlice()), catalogNextOffset: null, catalogTotal: null, catalogSource: null, catalogNotice: null });
    try {
      const page = await libraryService.catalogSearch(query);
      if (generation !== catalogGeneration) return;
      set({ catalogEntries: finishData(unwrap(page)), catalogSource: page.source, catalogNotice: page.notice ?? null, catalogTotal: page.total ?? null, catalogNextOffset: page.nextOffset ?? null });
    } catch (error) {
      if (generation === catalogGeneration) set({ catalogEntries: finishError(get().catalogEntries, error as ServiceError) });
    }
  },

  async loadMoreCatalog() {
    const state = get(), request = state.catalogRequest, offset = state.catalogNextOffset;
    if (!request || offset === null || pageLoading) return;
    const generation = catalogGeneration; pageLoading = true;
    set({ catalogEntries: startLoading(state.catalogEntries) });
    try {
      const page = await libraryService.catalogSearch({ ...request, offset });
      if (generation !== catalogGeneration) return;
      const items = new Map((get().catalogEntries.data ?? []).map(entry => [entry.catalogEntryId, entry]));
      for (const entry of page.items) items.set(entry.catalogEntryId, entry);
      set({ catalogEntries: finishData([...items.values()]), catalogSource: page.source, catalogNotice: page.notice ?? null, catalogTotal: page.total ?? null, catalogNextOffset: page.nextOffset ?? null });
    } catch (error) {
      if (generation === catalogGeneration) set({ catalogEntries: finishError(get().catalogEntries, error as ServiceError) });
    } finally { if (generation === catalogGeneration) pageLoading = false; }
  },

  computePlan(catalogEntryId, storageRootId, budgetBytes) {
    if (get().installStarting) return Promise.resolve();
    const generation = ++planGeneration;
    set({
      installPlan: startLoading(emptySlice()),
      activeCatalogEntryId: catalogEntryId,
      selectedStorageRootId: storageRootId,
    });
    return libraryService
      .installPlan({
        catalogEntryId,
        destinationStorageRootId: storageRootId,
        budgetBytes,
      })
      .then((plan) => {
        if (generation !== planGeneration) return;
        set({ installPlan: finishData<InstallPlan | null>(plan) });
      })
      .catch((error: ServiceError) => {
        if (generation === planGeneration) set({ installPlan: finishError(get().installPlan, error) });
      });
  },

  async startInstall(installPlanId) {
    const state = get();
    if (state.installStarting || state.installPlan.data?.installPlanId !== installPlanId || state.installPlan.data.destination.storageRootId !== state.selectedStorageRootId) return false;
    set({ installStarting: true, installPlan: startLoading(state.installPlan) });
    try {
      await libraryService.installStart({ installPlanId });
      await Promise.all([get().loadDownloads(), get().loadPacks()]);
      set({ installPlan: emptySlice<InstallPlan | null>() });
      return true;
    } catch (error) {
      set({ installPlan: finishError(state.installPlan, error as ServiceError) });
      await get().loadDownloads();
      return false;
    } finally { set({ installStarting: false }); }
  },

  clearInstallPlan() {
    if (get().installStarting) return;
    planGeneration++;
    set({
      installPlan: emptySlice<InstallPlan | null>(),
      activeCatalogEntryId: null,
    });
  },

  // Fan out the three independent reads in parallel. Each slice resolves on
  // its own; a catalog-offline failure must not blank the installed packs.
  loadAll() {
    set({
      storageRoots: startLoading(get().storageRoots),
      packs: startLoading(get().packs),
      downloads: startLoading(get().downloads),
    });
    return Promise.all([
      get().loadStorageRoots(),
      get().loadPacks(),
      get().loadDownloads(),
    ]).then(() => undefined);
  },
}));
