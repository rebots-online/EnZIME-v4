import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

/**
 * §15.4 AppStore — bootstrap slice.
 *
 * `bootstrap()` calls the §14.4 `app_bootstrap` command and caches its
 * `AppBootstrap` result (routes, capabilities, readiness, firstLaunch). The
 * shell's ReadinessIndicator and NavigationRail read from here.
 *
 * Until the Rust handler is wired (Layer B of M1), `invoke` rejects and the
 * slice stays in the `loading`/`error` state — a truthful dev state, never a
 * silent placeholder.
 */
export type ReadinessLevel = "ready" | "warning" | "degraded" | "unavailable";

export type ReadinessFlag = {
  readonly id: string;
  readonly level: "warning" | "degraded" | "unavailable";
  readonly label: string;
  readonly detail: string;
  readonly settingPath: string;
};

export type SystemReadiness = {
  readonly overall: ReadinessLevel;
  readonly summary: string;
  readonly flags: readonly ReadinessFlag[];
  readonly checkedAt: string;
};

export type RouteDescriptor = {
  readonly path: string;
  readonly enabled: boolean;
};

export type CapabilityDecision = {
  readonly capabilityKey: string;
  readonly state: "granted" | "denied" | "unknown";
  readonly detail: string;
};

export type AppBootstrap = {
  readonly schemaVersion: 1;
  readonly appVersion: string;
  readonly platform: "android" | "linux" | "windows";
  readonly firstLaunch: boolean;
  readonly routes: readonly RouteDescriptor[];
  readonly capabilities: readonly CapabilityDecision[];
  readonly readiness: SystemReadiness;
};

export type BootstrapStatus = "idle" | "loading" | "ready" | "error";

export interface AppState {
  bootstrapStatus: BootstrapStatus;
  readiness: SystemReadiness | null;
  firstLaunch: boolean;
  routes: readonly RouteDescriptor[];
  capabilities: readonly CapabilityDecision[];
  error: string | null;

  bootstrap: () => Promise<void>;
}

const LOADING_READINESS: SystemReadiness = {
  overall: "ready",
  summary: "Starting EnZIME Reader…",
  flags: [],
  checkedAt: "",
};

function classifyMessage(raw: unknown): string {
  if (typeof raw === "string") return raw;
  if (raw instanceof Error) return raw.message;
  if (raw && typeof raw === "object" && "message" in raw) {
    const m = (raw as { message?: unknown }).message;
    if (typeof m === "string") return m;
  }
  return "Bootstrap is unavailable in this environment.";
}

export const useAppStore = create<AppState>((set) => ({
  bootstrapStatus: "idle",
  readiness: LOADING_READINESS,
  firstLaunch: true,
  routes: [],
  capabilities: [],
  error: null,

  bootstrap() {
    set({ bootstrapStatus: "loading", error: null });
    return invoke<AppBootstrap>("app_bootstrap")
      .then((b) => {
        set({
          bootstrapStatus: "ready",
          readiness: b.readiness,
          firstLaunch: b.firstLaunch,
          routes: b.routes,
          capabilities: b.capabilities,
        });
      })
      .catch((raw: unknown) => {
        set({
          bootstrapStatus: "error",
          error: classifyMessage(raw),
        });
      });
  },
}));
