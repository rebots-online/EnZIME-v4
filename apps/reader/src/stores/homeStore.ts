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

  loadContinueReading() {
    set({ continueReading: startLoading(get().continueReading) });
    return homeService
      .loadContinueReading()
      .then((data) => {
        set({
          continueReading: finishData(data),
        });
      })
      .catch((error: ServiceError) => {
        set({ continueReading: finishError(get().continueReading, error) });
      });
  },

  loadRecentPacks() {
    set({ recentPacks: startLoading(get().recentPacks) });
    return homeService
      .loadRecentPacks()
      .then((data) => {
        set({
          recentPacks: finishData(data),
        });
      })
      .catch((error: ServiceError) => {
        set({ recentPacks: finishError(get().recentPacks, error) });
      });
  },

  loadTaskInbox() {
    set({ taskInbox: startLoading(get().taskInbox) });
    return homeService
      .loadTaskInbox()
      .then((data) => {
        set({ taskInbox: finishData(data) });
      })
      .catch((error: ServiceError) => {
        set({ taskInbox: finishError(get().taskInbox, error) });
      });
  },

  loadMemoryInbox() {
    set({ memoryInbox: startLoading(get().memoryInbox) });
    return homeService
      .loadMemoryInbox()
      .then((data) => {
        set({
          memoryInbox: finishData(data),
        });
      })
      .catch((error: ServiceError) => {
        set({ memoryInbox: finishError(get().memoryInbox, error) });
      });
  },

  loadReadiness() {
    set({ readiness: startLoading(get().readiness) });
    return homeService
      .loadReadiness()
      .then((data) => {
        set({ readiness: finishData(data) });
      })
      .catch((error: ServiceError) => {
        set({ readiness: finishError(get().readiness, error) });
      });
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

  acceptMemory(memoryId) {
    const inbox = get().memoryInbox;
    return homeService
      .acceptMemory(memoryId)
      .then(() => {
        const current = get().memoryInbox;
        if (current.data) {
          set({
            memoryInbox: {
              ...current,
              data: current.data.filter((m) => m.memoryId !== memoryId),
            },
          });
        }
      })
      .catch((error: ServiceError) => {
        set({ memoryInbox: finishError(inbox, error) });
      });
  },

  dismissMemory(memoryId) {
    const inbox = get().memoryInbox;
    return homeService
      .dismissMemory(memoryId)
      .then(() => {
        const current = get().memoryInbox;
        if (current.data) {
          set({
            memoryInbox: {
              ...current,
              data: current.data.filter((m) => m.memoryId !== memoryId),
            },
          });
        }
      })
      .catch((error: ServiceError) => {
        set({ memoryInbox: finishError(inbox, error) });
      });
  },

  loadAll() {
    return Promise.all([
      get().loadContinueReading(),
      get().loadRecentPacks(),
      get().loadTaskInbox(),
      get().loadMemoryInbox(),
      get().loadReadiness(),
    ]).then(() => undefined);
  },
}));
