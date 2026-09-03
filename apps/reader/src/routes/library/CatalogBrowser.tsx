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
  resolveState,
} from "../../components/home/shared";

const DEBOUNCE_MS = 280;
const INITIAL_QUERY = "";

export function CatalogBrowser(): ReactNode {
  const online = useLibraryStore((s) => s.online);
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
    debounceRef.current = setTimeout(() => {
      void searchCatalog({
        query: query.trim(),
        languageBcp47: language.trim() || undefined,
      });
    }, DEBOUNCE_MS);
    return () => {
      if (debounceRef.current) clearTimeout(debounceRef.current);
    };
  }, [query, language, searchCatalog]);

  const state = resolveState(online, slice, (data) => data.length > 0);
  const entries = slice.data ?? [];

  let body: ReactNode;
  if (state === "offline-empty") {
    body = (
      <StatusBanner
        tone="offline"
        icon="!"
        title="Catalog offline"
        detail="Reconnect to search the DynDon catalog."
      />
    );
  } else if (state === "loading") {
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
            : "Start typing to search the catalog."
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
                languageBcp47: language.trim() || undefined,
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
        <span className="lib-card__hint">DynDon</span>
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
          placeholder="lang (en, fr, …)"
          value={language}
          onChange={(e) => setLanguage(e.target.value)}
          aria-label="Filter by language"
          style={{ flex: "0 0 8rem" }}
        />
      </div>

      {body}
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
    formatBytes(entry.artifactSizeBytes),
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
