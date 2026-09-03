import { create } from "zustand";
import {
  homeService,
  type ContinueReadingEntry,
  type MemoryInboxEntry,
  type PackSummary,
  type ReadinessReport,
  type ServiceError,
  type TaskSummary,
} from "../services/homeService";

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

export interface HomeState {
  online: boolean;
  continueReading: Slice<ContinueReadingEntry | null>;
  recentPacks: Slice<readonly PackSummary[]>;
  taskInbox: Slice<readonly TaskSummary[]>;
  memoryInbox: Slice<readonly MemoryInboxEntry[]>;
  readiness: Slice<ReadinessReport>;

  setOnline: (online: boolean) => void;
  loadContinueReading: () => Promise<void>;
  loadRecentPacks: () => Promise<void>;
  loadTaskInbox: () => Promise<void>;
  loadMemoryInbox: () => Promise<void>;
  loadReadiness: () => Promise<void>;
  runReadinessCheck: () => Promise<void>;
  acceptMemory: (memoryId: string) => Promise<void>;
  dismissMemory: (memoryId: string) => Promise<void>;
  loadAll: () => Promise<void>;
}

function initialOnline(): boolean {
  if (typeof navigator === "undefined") return true;
  return navigator.onLine;
}

export const useHomeStore = create<HomeState>((set, get) => ({
  online: initialOnline(),
  continueReading: emptySlice<ContinueReadingEntry | null>(),
  recentPacks: emptySlice<readonly PackSummary[]>(),
  taskInbox: emptySlice<readonly TaskSummary[]>(),
  memoryInbox: emptySlice<readonly MemoryInboxEntry[]>(),
  readiness: emptySlice<ReadinessReport>(),

  setOnline(online) {
    if (get().online === online) return;
    set({ online });
  },

  // The granular `home_load_*` commands are retired (not in §14.4). Home now
  // loads via a single `home_snapshot` call (see `loadAll`). The per-slice
  // loaders are kept because the components still bind to them as `reload`
  // handlers; each simply re-runs the full snapshot fetch.
  loadContinueReading() {
    return get().loadAll();
  },

  loadRecentPacks() {
    return get().loadAll();
  },

  loadTaskInbox() {
    return get().loadAll();
  },

  loadMemoryInbox() {
    return get().loadAll();
  },

  loadReadiness() {
    return get().loadAll();
  },

  runReadinessCheck() {
    set({ readiness: startLoading(get().readiness) });
    return homeService
      .runReadinessCheck()
      .then((data) => {
        set({ readiness: finishData(data) });
      })
      .catch((error: ServiceError) => {
        set({ readiness: finishError(get().readiness, error) });
      });
  },

  // M1 DEBT: `home_accept_memory` / `home_dismiss_memory` are not in §14.4
  // (memory accept/reject should route through `lore_accept` / `lore_reject`).
  // For M1 the buttons stay: optimistically remove the row from the slice, then
  // fire-and-forget the existing call with errors swallowed so the UI never
  // blocks on (or surfaces) the not-yet-contracted command.
  acceptMemory(memoryId) {
    const current = get().memoryInbox;
    if (current.data) {
      set({
        memoryInbox: {
          ...current,
          data: current.data.filter((m) => m.memoryId !== memoryId),
        },
      });
    }
    return homeService.acceptMemory(memoryId).catch(() => {
      /* M1 debt: swallowed intentionally — see comment above. */
    });
  },

  dismissMemory(memoryId) {
    const current = get().memoryInbox;
    if (current.data) {
      set({
        memoryInbox: {
          ...current,
          data: current.data.filter((m) => m.memoryId !== memoryId),
        },
      });
    }
    return homeService.dismissMemory(memoryId).catch(() => {
      /* M1 debt: swallowed intentionally — see comment above. */
    });
  },

  // Single source of truth for Home hydration: ONE `home_snapshot` call, fanned
  // into the five slices. All slices share one loading window and one error
  // path so a snapshot failure is reported consistently across cards.
  loadAll() {
    set((s) => ({
      continueReading: startLoading(s.continueReading),
      recentPacks: startLoading(s.recentPacks),
      taskInbox: startLoading(s.taskInbox),
      memoryInbox: startLoading(s.memoryInbox),
      readiness: startLoading(s.readiness),
    }));
    return homeService
      .homeSnapshot()
      .then((snap) => {
        set({
          continueReading: finishData(snap.continueReading),
          recentPacks: finishData(snap.recentPacks),
          taskInbox: finishData(snap.taskInbox),
          memoryInbox: finishData(snap.memoryInbox),
          readiness: finishData(snap.readiness),
        });
      })
      .catch((error: ServiceError) => {
        set((s) => ({
          continueReading: finishError(s.continueReading, error),
          recentPacks: finishError(s.recentPacks, error),
          taskInbox: finishError(s.taskInbox, error),
          memoryInbox: finishError(s.memoryInbox, error),
          readiness: finishError(s.readiness, error),
        }));
      });
  },
}));
