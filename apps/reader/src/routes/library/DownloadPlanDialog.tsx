// DownloadPlanDialog — modal that walks the user through the DynDon plan.
//
// Opened by CatalogBrowser selecting a row (which sets `activeCatalogEntryId`
// on the store; LibraryView mounts this dialog in response). The flow:
//
//   1. StoragePlanner picks a destination (`selectedStorageRootId`).
//   2. "Compute plan" → store.computePlan() → install_plan command.
//   3. InstallPlanSummary + StorageImpactBar render the fit (the USP moment).
//   4. "Start install" → store.startInstall() → install_start, then the
//      download queue refreshes and the dialog closes.
//
// The dialog is a focus-managed modal: the panel is `role="dialog"`,
// `aria-modal="true"`, auto-focused on open, and Escape dismisses without
// committing. Reading-adjacent panels stay glass; the impact bar is the
// animated centrepiece.

import { useEffect, useRef, type ReactNode } from "react";
import { useLibraryStore } from "../../stores/libraryStore";
import type { CatalogEntry } from "../../services/libraryService";
import {
  RetryButton,
  SkeletonBlock,
  StatusBanner,
  formatBytes,
} from "../../components/home/shared";
import { StoragePlanner } from "./StoragePlanner";
import { StorageImpactBar } from "./StorageImpactBar";
import { InstallPlanSummary } from "./InstallPlanSummary";

export function DownloadPlanDialog(): ReactNode {
  const activeEntryId = useLibraryStore((s) => s.activeCatalogEntryId);
  const catalogEntries = useLibraryStore((s) => s.catalogEntries.data);
  const planSlice = useLibraryStore((s) => s.installPlan);
  const selectedRootId = useLibraryStore((s) => s.selectedStorageRootId);
  const computePlan = useLibraryStore((s) => s.computePlan);
  const startInstall = useLibraryStore((s) => s.startInstall);
  const clearInstallPlan = useLibraryStore((s) => s.clearInstallPlan);
  const setActiveCatalogEntry = useLibraryStore(
    (s) => s.setActiveCatalogEntry,
  );

  const panelRef = useRef<HTMLDivElement>(null);

  // Auto-focus the dialog panel on open.
  useEffect(() => {
    panelRef.current?.focus();
  }, []);

  // Escape dismisses (without committing). The store's `activeCatalogEntryId`
  // is the single source of truth for dialog visibility.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.stopPropagation();
        close();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const close = () => {
    clearInstallPlan();
    setActiveCatalogEntry(null);
  };

  const entry: CatalogEntry | undefined = (catalogEntries ?? []).find(
    (e) => e.catalogEntryId === activeEntryId,
  );

  const plan = planSlice.data;
  const planning = planSlice.status === "loading";
  const planResolved = planSlice.status === "ready" && plan !== null;
  const planError = planSlice.status === "error" ? planSlice.error : null;

  const canCompute = !!entry && !!selectedRootId && !planning && !planResolved;
  const canStart = planResolved && !planning;

  return (
    <div
      className="lib-dialog__backdrop"
      role="presentation"
      onClick={(e) => {
        if (e.target === e.currentTarget) close();
      }}
    >
      <div
        ref={panelRef}
        className="lib-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="lib-dialog-title"
        tabIndex={-1}
      >
        <header className="lib-dialog__head">
          <h2 className="lib-dialog__title" id="lib-dialog-title">
            {entry ? entry.title : "Plan download"}
          </h2>
          {entry ? (
            <p className="lib-dialog__sub">
              {entry.publisher} · {entry.category} ·{" "}
              {formatBytes(entry.artifactSizeBytes)}
            </p>
          ) : null}
        </header>

        <StoragePlanner />

        {!selectedRootId ? (
          <p className="lib-section-intro">
            Pick a destination above to compute the storage fit.
          </p>
        ) : null}

        {planError ? (
          <StatusBanner
            tone="warning"
            icon="!"
            title="Could not compute plan"
            detail={planError.message}
            actions={
              entry && selectedRootId ? (
                <RetryButton
                  onClick={() =>
                    computePlan(entry.catalogEntryId, selectedRootId)
                  }
                  label="Recompute"
                />
              ) : undefined
            }
          />
        ) : null}

        {planning && !planResolved ? <SkeletonBlock lines={4} /> : null}

        {planResolved && plan ? (
          <>
            <StorageImpactBar plan={plan} />
            <InstallPlanSummary plan={plan} />
          </>
        ) : null}

        <div className="lib-dialog__actions">
          <button
            type="button"
            className="lib-button lib-button--ghost"
            onClick={close}
          >
            Cancel
          </button>
          {entry && selectedRootId && !planResolved ? (
            <button
              type="button"
              className="lib-button"
              disabled={!canCompute}
              onClick={() => computePlan(entry.catalogEntryId, selectedRootId)}
            >
              {planning ? "Computing…" : "Compute plan"}
            </button>
          ) : null}
          {planResolved && plan ? (
            <button
              type="button"
              className="lib-button lib-button--success"
              disabled={!canStart}
              onClick={() => startInstall(plan.installPlanId).then(close)}
            >
              {planning ? "Starting…" : "Start install"}
            </button>
          ) : null}
        </div>
      </div>
    </div>
  );
}
