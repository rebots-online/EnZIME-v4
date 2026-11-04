// readerService — Tauri bridge for the Reader vertical (§24 binding).
//
// Frozen M1 contract (DOCS/ARCHITECTURE.md §24 + task spec):
//   readerOpen(packId)      → invoke("reader_open",    { packId })            :: OpenPackResponse
//   readerArticle(packId, articleKey)
//                         → invoke("reader_article", { packId, articleKey }) :: ArticleResponse
//
// The Rust `reader_open` / `reader_article` handlers are implemented in
// parallel; `tsc`/`vite` do not resolve Tauri command names at build time, so
// this file builds green regardless of handler availability. Errors thrown by
// `invoke` are classified into the same `ServiceError` shape the Home vertical
// uses, so the shared `resolveState`/`StatusBanner` primitives compose.

import { invoke } from "@tauri-apps/api/core";
import { track } from "./analytics";

export type OpenPackResponse = {
  readonly packId: string;
  readonly title: string;
  readonly mainPageKey: string | null;
};

export type ArticleResponse = {
  readonly title: string;
  readonly rawHtml: string;
  readonly mimeType: string;
  readonly internalLinks: readonly string[];
};

export type ReaderTitleSearchItem = {
  readonly articleKey: string;
  readonly title: string;
  readonly score: number;
};

export type ReaderTitleSearchPage = {
  readonly packId: string;
  readonly query: string;
  readonly offset: number;
  readonly limit: number;
  readonly hasMore: boolean;
  readonly items: readonly ReaderTitleSearchItem[];
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

export const readerService = {
  readerSearchTitles(packId: string, query: string, offset?: number, limit?: number): Promise<ReaderTitleSearchPage> {
    // Event only — the query text is never sent.
    if (!offset) track({ name: "title_search" });
    return call<ReaderTitleSearchPage>("reader_search_titles", { packId, query, offset, limit });
  },
  /** Opens (or attaches to) a ZIM pack by its installed `packId`; returns its id/title/main page. */
  readerOpen(packId: string): Promise<OpenPackResponse> {
    return call<OpenPackResponse>("reader_open", { packId });
  },
  /** Fetches a rendered article payload for a given pack + article key. */
  readerArticle(packId: string, articleKey: string): Promise<ArticleResponse> {
    return call<ArticleResponse>("reader_article", { packId, articleKey });
  },
};

export type ReaderService = typeof readerService;
