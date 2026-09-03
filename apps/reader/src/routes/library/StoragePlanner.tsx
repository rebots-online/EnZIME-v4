// StoragePlanner — destination picker for an install plan.
//
// Reads the `storageRoots` slice and renders one row per writable
// `StorageDestination`. The selected root is mirrored into
// `selectedStorageRootId` on the store (the plan flow reads it back when
// computing the plan). Once an `InstallPlan` is computed, the destination's
// free space is surfaced from the plan (`freeBytesBefore`), since the
// destination DTO itself does not carry live free-space — only the plan knows
// the post-fit numbers.

import { type ReactNode } from "react";
import { useLibraryStore } from "../../stores/libraryStore";
import {
  RetryButton,
  SkeletonBlock,
  StatusBanner,
  formatBytes,
  resolveState,
} from "../../components/home/shared";

export function StoragePlanner(): ReactNode {
  const online = useLibraryStore((s) => s.online);
  const slice = useLibraryStore((s) => s.storageRoots);
  const reload = useLibraryStore((s) => s.loadStorageRoots);
  const selectedId = useLibraryStore((s) => s.selectedStorageRootId);
  const setSelected = useLibraryStore((s) => s.setSelectedStorageRoot);
  const plan = useLibraryStore((s) => s.installPlan.data);

  const state = resolveState(online, slice, (data) => data.length > 0);
  const roots = slice.data ?? [];

  let body: ReactNode;
  if (state === "offline-empty") {
    body = (
      <StatusBanner
        tone="offline"
        icon="!"
        title="Storage offline"
        detail="Reconnect to enumerate storage destinations."
      />
    );
  } else if (state === "loading") {
    body = <SkeletonBlock lines={2} />;
  } else if (state === "populated") {
    body = (
      <ul className="lib-list" aria-label="Storage destinations">
        {roots.map((root) => {
          const isSelected = root.storageRootId === selectedId;
          const showFree =
            isSelected &&
            plan &&
            plan.destination.storageRootId === root.storageRootId;
          return (
            <li key={root.storageRootId}>
              <button
                type="button"
                className={`lib-row${isSelected ? " lib-row--selected" : ""}`}
                onClick={() => setSelected(root.storageRootId)}
                disabled={!root.writable}
                aria-pressed={isSelected}
                aria-label={`Select destination ${root.displayName}`}
                style={!root.writable ? { opacity: 0.5, cursor: "not-allowed" } : undefined}
              >
                <span className="lib-row__main">
                  <span className="lib-row__label">{root.displayName}</span>
                  <span className="lib-row__meta">
                    {root.writable ? "writable" : "read-only"}
                    {showFree && plan ? (
                      <> · {formatBytes(plan.freeBytesBefore)} free</>
                    ) : null}
                  </span>
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
  } else if (state === "empty") {
    body = (
      <StatusBanner
        tone="warning"
        icon="!"
        title="No writable storage"
        detail="Add a storage root in Settings before installing a pack."
      />
    );
  } else if (state === "recoverable") {
    body = (
      <StatusBanner
        tone="warning"
        icon="!"
        title="Could not load storage roots"
        detail={slice.error?.message ?? "Unexpected error."}
        actions={<RetryButton onClick={reload} label="Try again" />}
      />
    );
  } else if (state === "non-recoverable") {
    body = (
      <StatusBanner
        tone="error"
        icon="×"
        title="Storage unavailable"
        detail={slice.error?.message ?? "The storage index could not be read."}
      />
    );
  } else {
    body = (
      <StatusBanner
        tone="locked"
        icon="∅"
        title="Storage access denied"
        detail={slice.error?.message ?? "Cannot enumerate storage roots."}
      />
    );
  }

  return (
    <section className="lib-card" aria-labelledby="lib-storage-heading">
      <header className="lib-card__header">
        <h2 className="lib-card__title" id="lib-storage-heading">
          Destination
        </h2>
        {plan ? (
          <span className="lib-card__hint">
            {formatBytes(plan.freeBytesBefore)} free
          </span>
        ) : null}
      </header>
      <p className="lib-section-intro">
        Choose where the pack lands. DynDon computes the fit per destination.
      </p>
      {body}
    </section>
  );
}
