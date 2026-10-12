// LibraryView — §15.1 / §15.3 `/library` route.
//
// The user-facing entry to EnZIME's DynDon download — the USP: "download only
// what fits your storage". Composition: LibraryView renders the scoped Aurora
// Glass stylesheet, then composes PackGrid (installed), CatalogBrowser
// (search), StoragePlanner + InstallPlanSummary + StorageImpactBar (the fit
// moment, hosted inside DownloadPlanDialog), and DownloadQueue (active jobs).
//
// Styling follows the home/reader convention: a private `--lib-*` token set
// scoped to `.lib-surface`, mapped onto the global `--ez-*` Aurora Glass
// tokens (§15.6). Animation is the default experience (the fit bar grows in,
// the FITS badge breathes); prefers-reduced-motion is the a11y override only.
// Reading-adjacent tiles stay glass; long-form reading surfaces elsewhere
// remain opaque (§15.6 chrome-vs-content split).

import { useEffect, type ReactNode } from "react";
import { useLibraryStore } from "../../stores/libraryStore";
import { useOnlineSubscription } from "../../components/home/shared";
import { PackGrid } from "./PackGrid";
import { CatalogBrowser } from "./CatalogBrowser";
import { DownloadQueue } from "./DownloadQueue";
import { DownloadPlanDialog } from "./DownloadPlanDialog";

import { LibraryStyles } from "./LibraryStyles";
export { LibraryStyles } from "./LibraryStyles";

export function LibraryView(): ReactNode {
  const loadAll = useLibraryStore((s) => s.loadAll);
  const setOnline = useLibraryStore((s) => s.setOnline);
  const online = useLibraryStore((s) => s.online);
  const activeCatalogEntryId = useLibraryStore(
    (s) => s.activeCatalogEntryId,
  );
  const anyLoading = useLibraryStore(
    (s) =>
      s.storageRoots.status === "loading" ||
      s.packs.status === "loading" ||
      s.downloads.status === "loading",
  );

  useOnlineSubscription(setOnline);

  useEffect(() => {
    void loadAll();
  }, [loadAll]);

  return (
    <main className="lib-surface" aria-busy={anyLoading}>
      <LibraryStyles />
      <header className="lib-header">
        <h1 className="lib-header__title">Library</h1>
        <p className="lib-tagline">
          Download only <strong>what fits your storage</strong> — DynDon streams
          the slice you can actually keep.
        </p>
      </header>

      {!online ? (
        <p
          className="lib-pill lib-pill--warning"
          role="status"
          aria-live="polite"
          style={{ alignSelf: "flex-start" }}
        >
          Offline — retained packs remain readable; catalog may use cached results
        </p>
      ) : null}

      <div className="lib-layout">
        <div className="lib-col lib-col--main">
          <PackGrid />
          <CatalogBrowser />
        </div>
        <aside className="lib-col lib-col--side" aria-label="Storage and downloads">
          <DownloadQueue />
        </aside>
      </div>

      {activeCatalogEntryId ? <DownloadPlanDialog /> : null}
    </main>
  );
}
