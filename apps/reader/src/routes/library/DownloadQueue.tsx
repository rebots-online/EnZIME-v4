// DownloadQueue — the live download queue panel on the Library route.
//
// Reads the `downloads` slice and renders one row per `DownloadJob` with its
// state badge, retained bytes for partial packs and verified progress for full packs.
// Re-runs `loadDownloads()` on a slow interval so the queue advances without
// the user nudging it; the interval pauses when the tab is hidden.

import { useEffect, type ReactNode } from "react";
import { useLibraryStore } from "../../stores/libraryStore";
import type { DownloadJob, DownloadJobState } from "../../services/libraryService";
import {
  Progress,
  RetryButton,
  SkeletonBlock,
  StatusBanner,
  formatBytes,
  resolveState,
} from "../../components/home/shared";

const REFRESH_MS = 4000;

export function DownloadQueue(): ReactNode {
  const slice = useLibraryStore((s) => s.downloads);
  const reload = useLibraryStore((s) => s.loadDownloads);

  // Gentle polling: the queue is a background concern, so refresh on a slow
  // cadence. Pause while the tab is hidden to avoid wasted Tauri round-trips.
  useEffect(() => {
    const handle = setInterval(() => {
      if (typeof document === "undefined" || !document.hidden) {
        void reload();
      }
    }, REFRESH_MS);
    return () => clearInterval(handle);
  }, [reload]);

  const state = resolveState(true, slice, (data) => data.length > 0);
  const jobs = slice.data ?? [];

  let body: ReactNode;
  if (state === "offline-empty") {
    body = (
      <StatusBanner
        tone="offline"
        icon="!"
        title="Offline"
        detail="Job status is unavailable while offline."
      />
    );
  } else if (state === "loading") {
    body = <SkeletonBlock lines={2} />;
  } else if (state === "populated") {
    body = (
      <ul className="lib-list" aria-label="Download queue">
        {jobs.map((job) => (
          <JobRow key={job.downloadJobId} job={job} />
        ))}
      </ul>
    );
  } else if (state === "empty") {
    body = (
      <StatusBanner
        tone="info"
        icon="i"
        title="No downloads"
        detail="Plan a download from the catalog to populate the queue."
      />
    );
  } else if (state === "recoverable") {
    body = (
      <StatusBanner
        tone="warning"
        icon="!"
        title="Could not refresh queue"
        detail={slice.error?.message ?? "Unexpected error."}
        actions={<RetryButton onClick={reload} label="Try again" />}
      />
    );
  } else if (state === "non-recoverable") {
    body = (
      <StatusBanner
        tone="error"
        icon="×"
        title="Queue unavailable"
        detail={slice.error?.message ?? "The job store could not be read."}
      />
    );
  } else {
    body = (
      <StatusBanner
        tone="locked"
        icon="∅"
        title="Queue access denied"
        detail={slice.error?.message ?? "Cannot list download jobs."}
      />
    );
  }

  return (
    <section className="lib-card" aria-labelledby="lib-queue-heading">
      <header className="lib-card__header">
        <h2 className="lib-card__title" id="lib-queue-heading">
          Downloads
        </h2>
        {state === "populated" && jobs.length > 0 ? (
          <span className="lib-card__hint">
            {jobs.length} job{jobs.length === 1 ? "" : "s"}
          </span>
        ) : null}
      </header>
      {body}
    </section>
  );
}

function JobRow({ job }: { job: DownloadJob }): ReactNode {
  const retained = job.transferScope === "partial";
  const progressBytes = retained ? job.bytesRetained : job.bytesVerified;
  const ratio =
    !retained && job.expectedSizeBytes > 0
      ? progressBytes / job.expectedSizeBytes
      : 0;
  const ratioClamped = Math.max(0, Math.min(1, ratio));
  const pct = Math.round(ratioClamped * 100);

  const isActive =
    job.state === "fetching" ||
    job.state === "verifying" ||
    job.state === "activating" ||
    job.state === "planned";

  return (
    <li className="lib-queue__row">
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          gap: "0.5rem",
        }}
      >
        <StateBadge state={job.state} />
        <span className="lib-queue__meta">
          {formatBytes(progressBytes)} {retained ? "retained locally" : `verified / ${formatBytes(job.expectedSizeBytes)}`}
        </span>
      </div>
      {retained ? <p className="lib-queue__meta">Partial pack · {job.state === "completed" ? "retained content available offline" : "completed clusters kept for retry"}</p> : null}
      {job.errorDetail ? <p role="alert">{job.errorDetail}</p> : null}
      <Progress
        value={ratioClamped}
        indeterminate={isActive && (retained || job.expectedSizeBytes === 0)}
      />
      <div className="lib-queue__meta">
        <span title={job.downloadJobId}>{shortId(job.downloadJobId)}</span>
        {pct > 0 && pct < 100 && isActive ? <span>{pct}%</span> : null}
      </div>
    </li>
  );
}

function StateBadge({ state }: { state: DownloadJobState }): ReactNode {
  const cls =
    state === "completed"
      ? "lib-pill lib-pill--ready"
      : state === "failed"
        ? "lib-pill lib-pill--danger"
        : state === "cancelled" || state === "paused"
          ? "lib-pill lib-pill--warning"
          : "lib-pill lib-pill--accent";
  return <span className={cls}>{state}</span>;
}

function shortId(id: string): string {
  if (!id) return "—";
  if (id.length <= 10) return id;
  return `${id.slice(0, 6)}…${id.slice(-3)}`;
}
