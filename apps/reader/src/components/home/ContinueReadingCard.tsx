import { type ReactNode } from "react";
import { useNavigate } from "react-router-dom";
import { useHomeStore } from "../../stores/homeStore";
import {
  LinkButton,
  RetryButton,
  SkeletonBlock,
  StatusBanner,
  formatRelative,
  resolveState,
} from "./shared";

export function ContinueReadingCard(): ReactNode {
  const navigate = useNavigate();
  const online = useHomeStore((s) => s.online);
  const slice = useHomeStore((s) => s.continueReading);
  const reload = useHomeStore((s) => s.loadContinueReading);

  const state = resolveState(online, slice, (data) => data !== null);
  const entry = slice.data;

  const openReader = () => {
    if (!entry) return;
    navigate(`/reader/${entry.packId}/${entry.articleKey}`);
  };

  let body: ReactNode;
  if (state === "offline-empty") {
    body = (
      <StatusBanner
        tone="offline"
        icon="!"
        title="Offline"
        detail="Reconnect to load your last reading position."
        actions={
          <LinkButton to="/library" navigate={navigate} variant="ghost">
            Browse library
          </LinkButton>
        }
      />
    );
  } else if (state === "loading") {
    body = <SkeletonBlock lines={3} />;
  } else if (state === "populated" && entry) {
    body = (
      <div className="home-continue">
        <span className="home-continue__article">{entry.articleTitle}</span>
        <span className="home-continue__pack">
          {entry.packTitle} · opened {formatRelative(entry.lastOpenedAt)}
        </span>
        <div className="home-continue__actions">
          <button
            type="button"
            className="home-button"
            onClick={openReader}
            aria-label={`Resume reading ${entry.articleTitle} from ${entry.packTitle}`}
          >
            Resume reading
          </button>
          <LinkButton to="/library" navigate={navigate} variant="ghost">
            Open library
          </LinkButton>
        </div>
      </div>
    );
  } else if (state === "empty") {
    body = (
      <StatusBanner
        tone="info"
        icon="i"
        title="No reading in progress"
        detail="Open a pack from the library to start reading."
        actions={
          <LinkButton to="/library" navigate={navigate}>
            Browse the library
          </LinkButton>
        }
      />
    );
  } else if (state === "recoverable") {
    body = (
      <StatusBanner
        tone="warning"
        icon="!"
        title="Could not load reading position"
        detail={slice.error?.message ?? "Unexpected error."}
        actions={
          <>
            <RetryButton onClick={reload} label="Try again" />
            <LinkButton to="/library" navigate={navigate} variant="ghost">
              Open library
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
        title="Reading history unavailable"
        detail={
          slice.error?.message ??
          "The reading history store could not be read."
        }
        actions={
          <LinkButton to="/library" navigate={navigate}>
            Open library
          </LinkButton>
        }
      />
    );
  } else {
    body = (
      <StatusBanner
        tone="locked"
        icon="∅"
        title="Reading history locked"
        detail={
          slice.error?.message ??
          "This profile cannot access reading history."
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
      className="home-card home-card--continue"
      aria-labelledby="home-continue-heading"
    >
      <header className="home-card__header">
        <h2 className="home-card__title" id="home-continue-heading">
          Continue reading
        </h2>
        {state === "populated" && entry ? (
          <span className="home-card__hint">
            {formatRelative(entry.lastOpenedAt)}
          </span>
        ) : null}
      </header>
      {body}
    </section>
  );
}
