// CatalogBrowser — the DynDon catalog search panel on the Library route.
//
// Drives `libraryStore.searchCatalog()` from a debounced query input plus
// optional language / category filters, then renders the resulting
// `CatalogEntry` rows. Selecting a row opens the install-plan flow by setting
// `activeCatalogEntryId` on the store (LibraryView mounts DownloadPlanDialog
// in response). The artifact size is shown via `formatBytes` so the user can
// eyeball fit before the plan is even computed.

import { useEffect, useRef, useState, type ReactNode } from "react";
import { useLibraryStore } from "../../stores/libraryStore";
import type { CatalogEntry } from "../../services/libraryService";
import {
  RetryButton,
  SkeletonBlock,
  StatusBanner,
  formatBytes,
} from "../../components/home/shared";

const DEBOUNCE_MS = 280;
const INITIAL_QUERY = "";
export function catalogLanguage(value: string): string | undefined {
  const language = value.trim().toLowerCase();
  const common: Record<string, string> = { en: "eng", fr: "fra", de: "deu", es: "spa", pt: "por", zh: "zho", ja: "jpn", ar: "ara", ru: "rus" };
  return common[language] ?? (/^[a-z]{3}$/.test(language) ? language : undefined);
}

export function CatalogBrowser(): ReactNode {
  const source = useLibraryStore(s => s.catalogSource);
  const notice = useLibraryStore(s => s.catalogNotice);
  const total = useLibraryStore(s => s.catalogTotal);
  const nextOffset = useLibraryStore(s => s.catalogNextOffset);
  const loadMore = useLibraryStore(s => s.loadMoreCatalog);
  const slice = useLibraryStore((s) => s.catalogEntries);
  const searchCatalog = useLibraryStore((s) => s.searchCatalog);
  const setActiveCatalogEntry = useLibraryStore(
    (s) => s.setActiveCatalogEntry,
  );

  const [query, setQuery] = useState(INITIAL_QUERY);
  const [language, setLanguage] = useState<string>("");
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Debounced fan-out: every change to the query / language re-runs the
  // search after a short quiet period. An empty query is sent as-is — the
  // Rust side is expected to return a default ranking when no term is given.
  useEffect(() => {
    if (debounceRef.current) clearTimeout(debounceRef.current);
    if (language.trim() && !catalogLanguage(language)) return;
    debounceRef.current = setTimeout(() => {
      void searchCatalog({
        query: query.trim(),
        languageBcp47: catalogLanguage(language),
      });
    }, DEBOUNCE_MS);
    return () => {
      if (debounceRef.current) clearTimeout(debounceRef.current);
    };
  }, [query, language, searchCatalog]);

  const state = slice.status === "error" ? (slice.error?.kind ?? "recoverable") : (slice.data?.length ?? 0) > 0 ? "populated" : slice.status === "loading" || slice.status === "idle" ? "loading" : "empty";
  const entries = slice.data ?? [];

  let body: ReactNode;
  if (state === "loading") {
    body = <SkeletonBlock lines={4} />;
  } else if (state === "populated") {
    body = (
      <ul className="lib-list" aria-label="Catalog results">
        {entries.map((entry) => (
          <CatalogRow
            key={entry.catalogEntryId}
            entry={entry}
            onPlan={() => setActiveCatalogEntry(entry.catalogEntryId)}
          />
        ))}
      </ul>
    );
  } else if (state === "empty") {
    body = (
      <StatusBanner
        tone="info"
        icon="i"
        title="No matches"
        detail={
          query.trim()
            ? `Nothing in the catalog matches "${query.trim()}".`
            : "No entries are available for these filters."
        }
      />
    );
  } else if (state === "recoverable") {
    body = (
      <StatusBanner
        tone="warning"
        icon="!"
        title="Search failed"
        detail={slice.error?.message ?? "Unexpected error."}
        actions={
          <RetryButton
            onClick={() =>
              searchCatalog({
                query: query.trim(),
                languageBcp47: catalogLanguage(language),
              })
            }
            label="Retry search"
          />
        }
      />
    );
  } else if (state === "non-recoverable") {
    body = (
      <StatusBanner
        tone="error"
        icon="×"
        title="Catalog unavailable"
        detail={slice.error?.message ?? "The catalog endpoint could not be reached."}
      />
    );
  } else {
    body = (
      <StatusBanner
        tone="locked"
        icon="∅"
        title="Catalog access denied"
        detail={slice.error?.message ?? "This profile cannot search the catalog."}
      />
    );
  }

  return (
    <section className="lib-card" aria-labelledby="lib-catalog-heading">
      <header className="lib-card__header">
        <h2 className="lib-card__title" id="lib-catalog-heading">
          Catalog
        </h2>
        <span className="lib-card__hint">Kiwix · {source === "cache" ? "cached results" : source === "live" ? "live catalog" : "catalog"}</span>
      </header>

      <div className="lib-search">
        <input
          className="lib-input"
          type="search"
          inputMode="search"
          placeholder="Search packs — e.g. Wikipedia, TED, Project Gutenberg…"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          aria-label="Search catalog"
        />
        <input
          className="lib-input"
          type="text"
          placeholder="eng, fra, deu…"
          value={language}
          onChange={(e) => setLanguage(e.target.value)}
          aria-label="Filter by language (ISO 639-3)"
          style={{ flex: "0 0 8rem" }}
        />
      </div>

      <p className="lib-section-intro">Language uses ISO 639-3 (eng, fra). Common two-letter codes such as en and fr are also accepted.</p>
      {language.trim() && !catalogLanguage(language) ? <p role="alert">Enter a three-letter language code, or a supported common two-letter code.</p> : null}
      {notice ? <p role="status">{notice}</p> : null}
      {source === "cache" ? <p role="status">Cached catalog results. Availability may have changed; installation requires a reachable source.</p> : null}
      {body}
      {total !== null ? <p>{entries.length} of {total} catalog entries</p> : entries.length > 0 ? <p>{entries.length} entries loaded</p> : null}
      {nextOffset !== null ? <button className="lib-button lib-button--ghost" type="button" disabled={slice.status === "loading"} onClick={() => void loadMore()}>{slice.status === "loading" ? "Loading more…" : "Load more"}</button> : null}
    </section>
  );
}

function CatalogRow({
  entry,
  onPlan,
}: {
  entry: CatalogEntry;
  onPlan: () => void;
}): ReactNode {
  const meta = [
    entry.publisher,
    entry.category,
    entry.artifactSizeBytes > 0 ? `${formatBytes(entry.artifactSizeBytes)} full archive (advertised)` : "Full size not advertised",
    entry.languageBcp47,
  ]
    .filter(Boolean)
    .join(" · ");

  return (
    <li>
      <button
        type="button"
        className="lib-row"
        onClick={onPlan}
        aria-label={`Plan download for ${entry.title}`}
      >
        <span className="lib-row__main">
          <span className="lib-row__label">{entry.title}</span>
          <span className="lib-row__meta">{meta}</span>
          {entry.description ? (
            <span className="lib-section-intro">{entry.description}</span>
          ) : null}
        </span>
        <span className="lib-pill lib-pill--accent">Plan ↓</span>
      </button>
    </li>
  );
}
