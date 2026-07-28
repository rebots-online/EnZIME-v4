import { invoke } from "@tauri-apps/api/core";

const COMMANDS = {
  loadContinueReading: "home_load_continue_reading",
  loadRecentPacks: "home_load_recent_packs",
  loadTaskInbox: "home_load_task_inbox",
  loadMemoryInbox: "home_load_memory_inbox",
  loadReadiness: "home_load_readiness",
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
  loadContinueReading(): Promise<ContinueReadingEntry | null> {
    return call<ContinueReadingEntry | null>(COMMANDS.loadContinueReading);
  },
  loadRecentPacks(): Promise<readonly PackSummary[]> {
    return call<readonly PackSummary[]>(COMMANDS.loadRecentPacks);
  },
  loadTaskInbox(): Promise<readonly TaskSummary[]> {
    return call<readonly TaskSummary[]>(COMMANDS.loadTaskInbox);
  },
  loadMemoryInbox(): Promise<readonly MemoryInboxEntry[]> {
    return call<readonly MemoryInboxEntry[]>(COMMANDS.loadMemoryInbox);
  },
  loadReadiness(): Promise<ReadinessReport> {
    return call<ReadinessReport>(COMMANDS.loadReadiness);
  },
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
