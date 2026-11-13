// StoragePlanner — shared-storage medium control for an install plan.
//
// Downloads land in one shared common location (`mba.robin`) so every
// ecosystem app reuses the same bytes; sharing is fixed in this build and is
// never a per-app option. The user selects only the physical medium:
// internal storage or an SD card. With one available medium this renders a
// plain summary; with multiple media it renders the existing selectable
// rows. Enumeration is a local operation and never depends on the network
// online flag, and missing SD cards are simply absent — no invented rows.
//
// On Android, shared-file access is a real system permission: until it is
// granted, enumeration returns the actionable `shared_storage_access_required`
// error, and this component offers `Allow shared storage access`, which
// opens the system settings screen through the actual command. After the
// user returns, the root list refreshes on window focus/visibility.

import { useEffect, useState, type ReactNode } from "react";
import { useLibraryStore } from "../../stores/libraryStore";
import {
  libraryService,
  type ServiceError,
  type StorageDestination,
} from "../../services/libraryService";
import {
  RetryButton,
  StatusBanner,
  formatBytes,
} from "../../components/home/shared";

const ACCESS_REQUIRED_CODE = "shared_storage_access_required";

const SD_CARD_NOTE =
  "SD cards can be slower and can be removed. Internal storage is recommended for model files because loading and memory-mapped access can be affected.";

function messageOf(error: unknown): string {
  if (typeof error === "string") return error;
  if (error !== null && typeof error === "object" && "message" in error) {
    const message = (error as { message?: unknown }).message;
    if (typeof message === "string") return message;
  }
  return "Unexpected error.";
}

function rootMeta(root: StorageDestination): string {
  return `Shared across your apps · ${formatBytes(root.freeBytes)} free · ${formatBytes(
    root.availableBytes,
  )} available after system reserve`;
}

export function StoragePlanner(): ReactNode {
  const slice = useLibraryStore((s) => s.storageRoots);
  const reload = useLibraryStore((s) => s.loadStorageRoots);
  const selectedId = useLibraryStore((s) => s.selectedStorageRootId);
  const setSelected = useLibraryStore((s) => s.setSelectedStorageRoot);
  const planning = useLibraryStore((s) => s.installPlan.status === "loading");
  const installing = useLibraryStore((s) => s.installStarting);
  const plan = useLibraryStore((s) => s.installPlan.data);
  const busy = planning || installing;

  const [accessBusy, setAccessBusy] = useState(false);
  const [accessError, setAccessError] = useState<string | null>(null);

  // The system permission screen has no completion callback; re-enumerate
  // the shared roots whenever the window regains focus or visibility so the
  // granted state is observed on return.
  useEffect(() => {
    const refresh = () => {
      if (document.visibilityState === "visible") void reload();
    };
    window.addEventListener("focus", refresh);
    document.addEventListener("visibilitychange", refresh);
    return () => {
      window.removeEventListener("focus", refresh);
      document.removeEventListener("visibilitychange", refresh);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const allowAccess = () => {
    setAccessBusy(true);
    setAccessError(null);
    libraryService
      .requestSharedStorageAccess()
      .then(() => reload())
      .catch((error: unknown) => setAccessError(messageOf(error)))
      .finally(() => setAccessBusy(false));
  };

  const roots = slice.data ?? [];
  const selectedRoot = roots.find((root) => root.storageRootId === selectedId);
  const error: ServiceError | null = slice.status === "error" ? slice.error : null;
  const accessRequired =
    slice.status === "error" && !!error?.message.includes(ACCESS_REQUIRED_CODE);

  let body: ReactNode;
  if (slice.status === "loading" || slice.status === "idle") {
    body = <p role="status">Reading shared storage…</p>;
  } else if (slice.status === "error" && accessRequired) {
    body = (
      <StatusBanner
        tone="locked"
        icon="∅"
        title="Shared storage access is required"
        detail="EnZIME keeps large downloads in a shared location every ecosystem app can reuse. Allow shared storage access, then return to this app to continue."
        actions={
          <RetryButton
            onClick={allowAccess}
            label={accessBusy ? "Opening settings…" : "Allow shared storage access"}
          />
        }
      />
    );
  } else if (slice.status === "error") {
    body = (
      <StatusBanner
        tone="warning"
        icon="!"
        title="Could not load shared storage"
        detail={error?.message ?? "Unexpected error."}
        actions={<RetryButton onClick={reload} label="Try again" />}
      />
    );
  } else if (roots.length === 1) {
    // One available medium: a summary, not a pointless choice.
    const only = roots[0] as StorageDestination;
    body = (
      <p className="lib-row__meta" aria-label="Storage medium">
        {only.displayName} · {rootMeta(only)}
      </p>
    );
  } else if (roots.length > 1) {
    body = (
      <ul className="lib-list" aria-label="Storage media">
        {roots.map((root) => {
          const isSelected = root.storageRootId === selectedId;
          return (
            <li key={root.storageRootId}>
              <button
                type="button"
                className={`lib-row${isSelected ? " lib-row--selected" : ""}`}
                onClick={() => setSelected(root.storageRootId)}
                disabled={busy || !root.writable}
                aria-pressed={isSelected}
                aria-label={`Select ${root.displayName} as the storage medium`}
              >
                <span className="lib-row__main">
                  <span className="lib-row__label">{root.displayName}</span>
                  <span className="lib-row__meta">{rootMeta(root)}</span>
                </span>
                {isSelected ? (
                  <span className="lib-pill lib-pill--ready">selected</span>
                ) : null}
              </button>
            </li>
          );
        })}
      </ul>
    );
  } else {
    body = (
      <StatusBanner
        tone="warning"
        icon="!"
        title="No shared storage available"
        detail="No shared storage medium is currently available. Insert a card or free up internal storage, then try again."
        actions={<RetryButton onClick={reload} label="Try again" />}
      />
    );
  }

  return (
    <section className="lib-card" aria-labelledby="lib-storage-heading">
      <header className="lib-card__header">
        <h2 className="lib-card__title" id="lib-storage-heading">
          Storage
        </h2>
        {plan ? (
          <span className="lib-card__hint">
            {formatBytes(plan.freeBytesBefore)} free
          </span>
        ) : null}
      </header>
      <p className="lib-section-intro">
        Downloads are shared across your EnZIME ecosystem apps, so an edition
        is acquired only once and every app reuses it. Choose the physical
        medium to store it on.
      </p>
      {body}
      {accessError ? <p role="alert">{accessError}</p> : null}
      {selectedRoot?.medium === "sdcard" ? (
        <p role="note">{SD_CARD_NOTE}</p>
      ) : null}
    </section>
  );
}
