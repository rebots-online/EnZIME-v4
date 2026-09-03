// readerStore — zustand store backing the Reader route (§15.1 / §24).
//
// Exposes:
//   currentArticle   — Slice<ArticleResponse> (status/data/error/lastUpdated)
//   open(path)       — opens a ZIM file via readerService.readerOpen, remembers
//                       the pack, and eagerly loads its main page article.
//   loadArticle(id)  — fetches a specific article by (packId, articleKey).
//   navigate(href)   — resolves an internal ZIM article link (relative path,
//                       possibly carrying an `A/` namespace prefix) to a
//                       `/reader/:packId/:articleKey` route and drives the
//                       router through a pluggable navigator registered by the
//                       view layer (keeps the store free of router imports and
//                       avoids a circular dependency with router.tsx).

import { create } from "zustand";
import {
  readerService,
  type ArticleResponse,
  type OpenPackResponse,
  type ServiceError,
} from "../services/readerService";

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

/**
 * Pluggable navigator registered by the view layer (ReaderView) via
 * `setRouteNavigator(useNavigate())`. Keeping this indirect lets the store
 * trigger client-side route changes without importing react-router (which
 * would cycle router.tsx → ReaderView → readerStore → router.tsx).
 */
let routeNavigator: ((path: string) => void) | null = null;

export function setRouteNavigator(fn: ((path: string) => void) | null): void {
  routeNavigator = fn;
}

/**
 * Normalise an internal ZIM article reference into a bare article key.
 *
 * ZIM article links are relative paths scoped to the same pack and commonly
 * carry a single-letter namespace prefix such as `A/` (the article namespace).
 * We strip the namespace and any `./` `../` segments to form the articleKey
 * passed to `reader_article`. External / fragment-only hrefs return `null`.
 */
export function normalizeArticleKey(href: string): string | null {
  if (!href) return null;
  let raw = href.trim();
  // Fragment-only / external / non-article schemes are not article keys.
  if (raw === "" || raw.startsWith("#")) return null;
  if (/^(https?:|mailto:|tel:|data:|javascript:|vbscript:|enzime-resource:)/i.test(raw)) {
    return null;
  }
  // Already a reader route? Hand back the articleKey segment.
  if (raw.startsWith("/reader/")) {
    const parts = raw.split("/").filter(Boolean);
    // ["reader", packId, ...articleKeyParts]
    const rest = parts.slice(2);
    return rest.length ? decodeURIComponent(rest.join("/")) : null;
  }
  // Drop query/fragment.
  raw = raw.split("#")[0]!.split("?")[0]!;
  if (!raw) return null;
  try {
    raw = decodeURIComponent(raw);
  } catch {
    // malformed URI — keep original.
  }
  // Collapse leading ./ and ../ segments.
  while (/^(\.\.?\/)+/.test(raw)) {
    raw = raw.replace(/^(\.\.?\/)+/, "");
  }
  // Strip a ZIM namespace prefix like "A/", "C/", "I/", "M/", "-/" (single char + slash).
  raw = raw.replace(/^[A-Za-z-]\//, "");
  return raw === "" ? null : raw;
}

export interface ReaderState {
  currentPackId: string | null;
  currentPackTitle: string | null;
  currentArticleKey: string | null;
  currentArticle: Slice<ArticleResponse>;

  open: (path: string) => Promise<OpenPackResponse | null>;
  loadArticle: (packId: string, articleKey: string) => Promise<void>;
  resolveArticleRoute: (href: string) => string | null;
  navigate: (href: string) => void;
  clearArticle: () => void;
}

export const useReaderStore = create<ReaderState>((set, get) => ({
  currentPackId: null,
  currentPackTitle: null,
  currentArticleKey: null,
  currentArticle: emptySlice<ArticleResponse>(),

  open(path) {
    return readerService
      .readerOpen(path)
      .then((res) => {
        set({
          currentPackId: res.packId,
          currentPackTitle: res.title,
        });
        void get().loadArticle(res.packId, res.mainPageKey);
        return res;
      })
      .catch((error: ServiceError) => {
        set({
          currentArticle: finishError(get().currentArticle, error),
        });
        return null;
      });
  },

  loadArticle(packId, articleKey) {
    set({
      currentArticle: startLoading(get().currentArticle),
      currentArticleKey: articleKey,
      currentPackId: packId,
    });
    return readerService
      .readerArticle(packId, articleKey)
      .then((data) => {
        set({
          currentArticle: finishData(data),
          currentArticleKey: articleKey,
        });
      })
      .catch((error: ServiceError) => {
        set({
          currentArticle: finishError(get().currentArticle, error),
          currentArticleKey: articleKey,
        });
      });
  },

  resolveArticleRoute(href) {
    const packId = get().currentPackId;
    if (!packId) return null;
    const key = normalizeArticleKey(href);
    if (!key) return null;
    return `/reader/${packId}/${key}`;
  },

  navigate(href) {
    const path = get().resolveArticleRoute(href);
    if (path && routeNavigator) {
      routeNavigator(path);
    }
  },

  clearArticle() {
    set({
      currentArticle: emptySlice<ArticleResponse>(),
      currentArticleKey: null,
    });
  },
}));
