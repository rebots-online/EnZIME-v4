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

import { useEffect, useRef, useState, type ReactNode } from "react";
import { useLibraryStore } from "../../stores/libraryStore";
import { libraryService, type CatalogEntry } from "../../services/libraryService";
import {
  Progress,
  RetryButton,
  SkeletonBlock,
  StatusBanner,
  formatBytes,
} from "../../components/home/shared";
import { StoragePlanner } from "./StoragePlanner";
import { StorageImpactBar } from "./StorageImpactBar";
import { InstallPlanSummary } from "./InstallPlanSummary";

export function budgetBytesFor(value: string, unit: "MiB" | "GiB"): number | undefined {
  const amount = Number(value), bytes = amount * (unit === "MiB" ? 1024 ** 2 : 1024 ** 3);
  return value.trim() && Number.isFinite(amount) && amount > 0 && Number.isSafeInteger(bytes) ? bytes : undefined;
}

export function DownloadPlanDialog(): ReactNode {
  const [budget, setBudget] = useState("1");
  const [unit, setUnit] = useState<"MiB" | "GiB">("GiB");
  const [cancelRequested, setCancelRequested] = useState(false);
  const [cancelError, setCancelError] = useState<string>();
  const [computedBudget, setComputedBudget] = useState<number>();
  const budgetBytes = budgetBytesFor(budget, unit);
  const starting = useLibraryStore(s => s.installStarting);
  const downloads = useLibraryStore(s => s.downloads.data);
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
  }, [starting]);

  const close = () => {
    if (starting) return;
    clearInstallPlan();
    setActiveCatalogEntry(null);
  };

  const entry: CatalogEntry | undefined = (catalogEntries ?? []).find(
    (e) => e.catalogEntryId === activeEntryId,
  );

  const plan = planSlice.data;
  const jobPrefix = plan ? `job-${plan.installPlanId}` : "";
  const currentJob = plan ? downloads?.find(job => job.downloadJobId === jobPrefix || (job.downloadJobId.startsWith(`${jobPrefix}-retry-`) && /^[0-9A-HJKMNP-TV-Z]{26}$/.test(job.downloadJobId.slice(jobPrefix.length + 7)))) : undefined;
  const retainedBytes = currentJob?.bytesRetained ?? 0;
  const progress = plan && plan.downloadBytes > 0 ? Math.min(1, Math.max(0, retainedBytes / plan.downloadBytes)) : 0;
  const planning = planSlice.status === "loading";
  const planResolved = plan !== null && plan !== undefined && plan.destination.storageRootId === selectedRootId && computedBudget === budgetBytes;
  const planError = planSlice.status === "error" ? planSlice.error : null;

  const canCompute = !!entry && !!selectedRootId && !!budgetBytes && !planning && !starting && !planResolved;
  const canStart = planResolved && !planning && !starting;
  const compute = () => { if (entry && selectedRootId && budgetBytes) { setComputedBudget(budgetBytes); return computePlan(entry.catalogEntryId, selectedRootId, budgetBytes); } return Promise.resolve(); };

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
        <fieldset disabled={planning || starting} style={{ border: 0, padding: 0 }}>
          <legend>Maximum storage budget</legend>
          <div className="lib-search"><input className="lib-input" type="number" min="0.001" step="any" value={budget} aria-label="Storage budget" onChange={event => setBudget(event.target.value)} />
          <select className="lib-select" aria-label="Storage budget unit" value={unit} onChange={event => setUnit(event.target.value as "MiB" | "GiB")}><option value="MiB">MiB</option><option value="GiB">GiB</option></select></div>
          <p>DynDon retains selected clusters within this budget, including its index and working space.</p>
          {!budgetBytes ? <p role="alert">Enter a positive storage budget that resolves to a safe whole byte count.</p> : null}
        </fieldset>

        {!selectedRootId ? (
          <p className="lib-section-intro">
            Pick a destination above to compute the storage fit.
          </p>
        ) : null}

        {planError ? (
          <StatusBanner
            tone="warning"
            icon="!"
            title={plan ? "Download did not finish" : "Could not compute plan"}
            detail={planError.message}
            actions={
              entry && selectedRootId ? (
                <RetryButton
                  onClick={compute}
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

        {starting && plan ? <section aria-label="Current download progress" aria-live="polite">
          <p>{currentJob ? currentJob.state === "completed" ? "Finalizing reader availability…" : `Download state: ${currentJob.state}` : "Preparing transfer…"}</p>
          <p>{formatBytes(retainedBytes)} retained / {formatBytes(plan.downloadBytes)} planned partial payload</p>
          <Progress value={progress} indeterminate={!currentJob || plan.downloadBytes <= 0} />
          {currentJob?.errorDetail ? <p role="alert">{currentJob.errorDetail}</p> : null}
        </section> : null}
        {cancelError ? <p role="alert">{cancelError}</p> : null}
        {starting && cancelRequested ? <p role="status">Cancellation requested. Waiting for the current transfer to stop…</p> : null}
        <div className="lib-dialog__actions">
          {starting && plan ? <button type="button" className="lib-button lib-button--danger" disabled={cancelRequested} onClick={() => {
            setCancelRequested(true); setCancelError(undefined);
            void libraryService.installCancel({ installPlanId: plan.installPlanId }).catch(error => { setCancelRequested(false); setCancelError(error.message ?? String(error)); });
          }}>Cancel download</button> : null}
          <button
            type="button"
            className="lib-button lib-button--ghost"
            onClick={close}
            disabled={starting}
          >
            Cancel
          </button>
          {entry && selectedRootId && !planResolved ? (
            <button
              type="button"
              className="lib-button"
              disabled={!canCompute}
              onClick={compute}
            >
              {planning ? "Computing…" : "Compute plan"}
            </button>
          ) : null}
          {planResolved && plan ? (
            <button
              type="button"
              className="lib-button lib-button--success"
              disabled={!canStart}
              onClick={() => { setCancelRequested(false); setCancelError(undefined); void startInstall(plan.installPlanId).then(success => { if (success) { clearInstallPlan(); setActiveCatalogEntry(null); } }); }}
            >
              {starting ? "Downloading…" : planError ? "Retry download" : "Start download"}
            </button>
          ) : null}
        </div>
      </div>
    </div>
  );
}
