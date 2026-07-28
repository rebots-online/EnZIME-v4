import { type ReactNode } from "react";
import { useNavigate } from "react-router-dom";
import { useHomeStore } from "../../stores/homeStore";
import type { MemoryInboxEntry } from "../../services/homeService";
import {
  LinkButton,
  RetryButton,
  SkeletonBlock,
  StatusBanner,
  formatRelative,
  resolveState,
} from "./shared";

function sourceLabel(source: MemoryInboxEntry["source"]): string {
  switch (source) {
    case "ai":
      return "from AI";
    case "user":
      return "your note";
    case "import":
      return "imported";
    default:
      return "";
  }
}

function MemoryRow({
  entry,
  navigate,
  accept,
  dismiss,
}: {
  entry: MemoryInboxEntry;
  navigate: (path: string) => void;
  accept: (id: string) => Promise<void>;
  dismiss: (id: string) => Promise<void>;
}): ReactNode {
  return (
    <li className="home-list__item">
      <article className="home-memory__row">
        <button
          type="button"
          className="home-row"
          onClick={() =>
            navigate(`/memory?inbox=${encodeURIComponent(entry.memoryId)}`)
          }
          aria-label={`Open memory: ${entry.summary}`}
          style={{ marginBottom: 0 }}
        >
          <span className="home-row__main">
            <span className="home-row__label">{entry.summary}</span>
            <span className="home-row__meta">
              {sourceLabel(entry.source)} · {formatRelative(entry.createdAt)}
            </span>
          </span>
          <span className="home-row__aside" aria-hidden="true">→</span>
        </button>
        <div className="home-memory__actions">
          <button
            type="button"
            className="home-button home-button--success"
            onClick={() => accept(entry.memoryId)}
            aria-label={`Accept memory: ${entry.summary}`}
          >
            Accept
          </button>
          <button
            type="button"
            className="home-button home-button--ghost"
            onClick={() => dismiss(entry.memoryId)}
            aria-label={`Dismiss memory: ${entry.summary}`}
          >
            Dismiss
          </button>
        </div>
      </article>
    </li>
  );
}

export function MemoryInboxCard(): ReactNode {
  const navigate = useNavigate();
  const online = useHomeStore((s) => s.online);
  const slice = useHomeStore((s) => s.memoryInbox);
  const reload = useHomeStore((s) => s.loadMemoryInbox);
  const accept = useHomeStore((s) => s.acceptMemory);
  const dismiss = useHomeStore((s) => s.dismissMemory);

  const state = resolveState(online, slice, (data) => data.length > 0);
  const entries = slice.data ?? [];

  let body: ReactNode;
  if (state === "offline-empty") {
    body = (
      <StatusBanner
        tone="offline"
        icon="!"
        title="Offline"
        detail="Reconnect to load memory inbox."
        actions={
          <LinkButton to="/memory" navigate={navigate} variant="ghost">
            Open memory
          </LinkButton>
        }
      />
    );
  } else if (state === "loading") {
    body = <SkeletonBlock lines={2} />;
  } else if (state === "populated") {
    body = (
      <ul className="home-list" aria-label="Memory inbox">
        {entries.map((entry) => (
          <MemoryRow
            key={entry.memoryId}
            entry={entry}
            navigate={navigate}
            accept={accept}
            dismiss={dismiss}
          />
        ))}
      </ul>
    );
  } else if (state === "empty") {
    body = (
      <StatusBanner
        tone="info"
        icon="✓"
        title="Memory inbox is clear"
        detail="New memories proposed by the AI or imported from a sidecar will appear here for review."
        actions={
          <LinkButton to="/memory" navigate={navigate} variant="ghost">
            Open memory workspace
          </LinkButton>
        }
      />
    );
  } else if (state === "recoverable") {
    body = (
      <StatusBanner
        tone="warning"
        icon="!"
        title="Could not load memory inbox"
        detail={slice.error?.message ?? "Unexpected error."}
        actions={
          <>
            <RetryButton onClick={reload} label="Try again" />
            <LinkButton to="/memory" navigate={navigate} variant="ghost">
              Open memory
            </LinkButton>
          </>
        }
      />
    );
  } else if (state === "non-recoverable") {
    body = (
      <StatusBanner
        tone="error"
        icon="×"
        title="Memory store unavailable"
        detail={slice.error?.message ?? "The memory store could not be read."}
        actions={
          <LinkButton to="/memory" navigate={navigate}>
            Open memory
          </LinkButton>
        }
      />
    );
  } else {
    body = (
      <StatusBanner
        tone="locked"
        icon="∅"
        title="Memory inbox locked"
        detail={
          slice.error?.message ?? "This profile cannot review memories."
        }
        actions={
          <LinkButton to="/settings" navigate={navigate} variant="ghost">
            Open settings
          </LinkButton>
        }
      />
    );
  }

  return (
    <section
      className="home-card home-card--memory"
      aria-labelledby="home-memory-heading"
    >
      <header className="home-card__header">
        <h2 className="home-card__title" id="home-memory-heading">
          Memory inbox
        </h2>
        {state === "populated" && entries.length > 0 ? (
          <span className="home-card__hint">
            {entries.length} pending
          </span>
        ) : null}
      </header>
      {body}
    </section>
  );
}
