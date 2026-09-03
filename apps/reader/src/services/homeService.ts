import { invoke } from "@tauri-apps/api/core";

// M1 contract: Home binds to a SINGLE `home_snapshot` command (§14.4 / §24).
// The granular `home_load_*` commands that previously lived here were NOT in the
// §14.4 registry and have been retired. `home_snapshot` returns the full payload
// in one round-trip; the store fans it into the existing slices.
//
// M1 DEBT: `home_accept_memory` / `home_dismiss_memory` are also NOT in §14.4
// (memory accept/reject is `lore_accept` / `lore_reject`). They are retained
// here only as fire-and-forget calls so the M1 UI buttons do not block; the
// store swallows their errors. Replace with the `lore_*` commands in a later mile.
const COMMANDS = {
  snapshot: "home_snapshot",
  acceptMemory: "home_accept_memory",
  dismissMemory: "home_dismiss_memory",
  runReadinessCheck: "home_run_readiness_check",
} as const;

export type ArticleLocator = {
  readonly packId: string;
  readonly articleKey: string;
};

export type ContinueReadingEntry = {
  readonly packId: string;
  readonly packTitle: string;
  readonly articleKey: string;
  readonly articleTitle: string;
  readonly scrollAnchor: string | null;
  readonly lastOpenedAt: string;
};

export type PackSummary = {
  readonly packId: string;
  readonly title: string;
  readonly subtitle: string | null;
  readonly sizeBytes: number;
  readonly articleCount: number;
  readonly lastOpenedAt: string | null;
};

export type TaskKind = "download" | "index" | "inference" | "lore" | "backup";
export type TaskPhase = "queued" | "running" | "paused" | "awaiting-input";

export type TaskSummary = {
  readonly taskId: string;
  readonly kind: TaskKind;
  readonly phase: TaskPhase;
  readonly label: string;
  readonly progress: number;
  readonly target: ArticleLocator | null;
};

export type MemoryInboxEntry = {
  readonly memoryId: string;
  readonly summary: string;
  readonly source: "ai" | "user" | "import";
  readonly createdAt: string;
};

export type ReadinessLevel = "ready" | "warning" | "degraded" | "unavailable";

export type ReadinessFlag = {
  readonly id: string;
  readonly level: "warning" | "degraded" | "unavailable";
  readonly label: string;
  readonly detail: string;
  readonly settingPath: string;
};

export type ReadinessReport = {
  readonly overall: ReadinessLevel;
  readonly summary: string;
  readonly flags: readonly ReadinessFlag[];
  readonly checkedAt: string;
};

/**
 * Single payload returned by the `home_snapshot` command. The shape mirrors the
 * store slices one-to-one so the store can fan this object straight into them
 * without any re-shaping.
 */
export type HomeSnapshot = {
  readonly continueReading: ContinueReadingEntry | null;
  readonly recentPacks: readonly PackSummary[];
  readonly taskInbox: readonly TaskSummary[];
  readonly memoryInbox: readonly MemoryInboxEntry[];
  readonly readiness: ReadinessReport;
};

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

export const homeService = {
  /**
   * The single Home payload call. Replaces the five granular `home_load_*`
   * commands that were not in the §14.4 registry. The Rust `home_snapshot`
   * handler is implemented in parallel; the frontend calls it by this exact
   * name and builds green against this type.
   */
  homeSnapshot(): Promise<HomeSnapshot> {
    return call<HomeSnapshot>(COMMANDS.snapshot);
  },
  // M1 DEBT: see file header — these two are not in §14.4 (lore_accept/lore_reject
  // is the eventual contract). Kept fire-and-forget by the store for M1.
  acceptMemory(memoryId: string): Promise<void> {
    return call<void>(COMMANDS.acceptMemory, { memoryId });
  },
  dismissMemory(memoryId: string): Promise<void> {
    return call<void>(COMMANDS.dismissMemory, { memoryId });
  },
  runReadinessCheck(): Promise<ReadinessReport> {
    return call<ReadinessReport>(COMMANDS.runReadinessCheck);
  },
};

export type HomeService = typeof homeService;
