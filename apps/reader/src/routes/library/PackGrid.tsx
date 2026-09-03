// PackGrid — the "installed packs" tile grid on the Library route.
//
// Reads the `packs` slice from libraryStore and renders one tile per
// `PackManifest`. Selecting a tile opens the pack in the Reader via
// `/reader/:packId/:mainPageKey`. NOTE: `PackManifest` (§14.4) does not carry
// `mainPageKey` — the Reader's `reader_open` returns it, but that call belongs
// to the Reader vertical. Until the manifest DTO is extended, we route to the
// ZIM main-namespace landing key `"A"` as the conventional default; the
// Reader's own open flow resolves the true main page. (DTO gap flagged in the
// build report.)

import { type ReactNode } from "react";
import { useNavigate } from "react-router-dom";
import { useLibraryStore } from "../../stores/libraryStore";
import {
  LinkButton,
  RetryButton,
  SkeletonBlock,
  StatusBanner,
  formatBytes,
  formatRelative,
  resolveState,
} from "../../components/home/shared";

export function PackGrid(): ReactNode {
  const navigate = useNavigate();
  const online = useLibraryStore((s) => s.online);
  const slice = useLibraryStore((s) => s.packs);
  const reload = useLibraryStore((s) => s.loadPacks);

  const state = resolveState(online, slice, (data) => data.length > 0);
  const packs = slice.data ?? [];

  let body: ReactNode;
  if (state === "offline-empty") {
    body = (
      <StatusBanner
        tone="offline"
        icon="!"
        title="Offline"
        detail="Reconnect to load your installed packs."
      />
    );
  } else if (state === "loading") {
    body = (
      <div className="lib-grid" aria-hidden="true">
        {Array.from({ length: 4 }, (_, i) => (
          <div className="lib-tile" key={i}>
            <SkeletonBlock lines={3} />
          </div>
        ))}
      </div>
    );
  } else if (state === "populated") {
    body = (
      <div className="lib-grid">
        {packs.map((pack) => {
          const meta = [
            pack.articleCount === 1
              ? "1 article"
              : `${pack.articleCount} articles`,
            formatBytes(pack.sizeBytes),
            pack.lastOpenedAt
              ? `opened ${formatRelative(pack.lastOpenedAt)}`
              : "not opened",
          ].join(" · ");
          return (
            <button
              key={pack.packId}
              type="button"
              className="lib-tile"
              onClick={() =>
                navigate(`/reader/${encodeURIComponent(pack.packId)}/A`)
              }
              aria-label={`Open pack ${pack.displayName}`}
            >
              <span className="lib-tile__title">{pack.displayName}</span>
              <span className="lib-tile__meta">{meta}</span>
              <span className="lib-pill lib-pill--accent lib-tile__lang">
                {pack.languageBcp47}
              </span>
            </button>
          );
        })}
      </div>
    );
  } else if (state === "empty") {
    body = (
      <StatusBanner
        tone="info"
        icon="i"
        title="No packs installed"
        detail="Search the catalog below to download your first pack."
      />
    );
  } else if (state === "recoverable") {
    body = (
      <StatusBanner
        tone="warning"
        icon="!"
        title="Could not load installed packs"
        detail={slice.error?.message ?? "Unexpected error."}
        actions={<RetryButton onClick={reload} label="Try again" />}
      />
    );
  } else if (state === "non-recoverable") {
    body = (
      <StatusBanner
        tone="error"
        icon="×"
        title="Pack manifest unavailable"
        detail={slice.error?.message ?? "The pack store could not be read."}
      />
    );
  } else {
    body = (
      <StatusBanner
        tone="locked"
        icon="∅"
        title="Installed-pack access denied"
        detail={
          slice.error?.message ??
          "This profile cannot list installed packs."
        }
      />
    );
  }

  return (
    <section
      className="lib-card"
      aria-labelledby="lib-packs-heading"
    >
      <header className="lib-card__header">
        <h2 className="lib-card__title" id="lib-packs-heading">
          Installed packs
        </h2>
        {state === "populated" && packs.length > 0 ? (
          <span className="lib-card__hint">
            {packs.length} pack{packs.length === 1 ? "" : "s"}
          </span>
        ) : null}
      </header>
      {body}
    </section>
  );
}

/** Re-exported for the dialog's "browse catalog" affordance. */
export function BrowseCatalogLink({
  navigate,
}: {
  navigate: (path: string) => void;
}): ReactNode {
  return (
    <LinkButton to="/library" navigate={navigate} variant="ghost">
      Browse catalog
    </LinkButton>
  );
}
