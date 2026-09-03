import type { ReactNode } from "react";

/**
 * §16.6 JobStrip — compact background-transfer status under the top bar.
 * Reads `libraryStore.downloads` (§24 binding) once the Library milestone
 * lands. In M1 there are no transfers, so it renders the truthful idle state.
 */
export function JobStrip(): ReactNode {
  const items: readonly { readonly label: string; readonly detail: string }[] = [];
  return (
    <div className="ez-job-strip" role="status" aria-live="polite">
      {items.length === 0 ? (
        <span className="ez-job-strip__empty">Idle</span>
      ) : (
        items.map((it) => (
          <span className="ez-job-strip__chip" key={it.label}>
            <span className="ez-job-strip__pulse" aria-hidden="true" />
            {it.label} · {it.detail}
          </span>
        ))
      )}
    </div>
  );
}
