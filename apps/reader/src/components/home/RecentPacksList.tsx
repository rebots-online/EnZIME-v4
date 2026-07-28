import { type ReactNode } from "react";
import { useNavigate } from "react-router-dom";
import { useHomeStore } from "../../stores/homeStore";
import { PackCard } from "./PackCard";
import {
  LinkButton,
  RetryButton,
  SkeletonBlock,
  StatusBanner,
  resolveState,
} from "./shared";

export function RecentPacksList(): ReactNode {
  const navigate = useNavigate();
  const online = useHomeStore((s) => s.online);
  const slice = useHomeStore((s) => s.recentPacks);
  const reload = useHomeStore((s) => s.loadRecentPacks);

  const state = resolveState(online, slice, (data) => data.length > 0);
  const packs = slice.data ?? [];

  let body: ReactNode;
  if (state === "offline-empty") {
    body = (
      <StatusBanner
        tone="offline"
        icon="!"
        title="Offline"
        detail="Reconnect to load your recent packs."
        actions={
          <LinkButton to="/library" navigate={navigate} variant="ghost">
            Open library
          </LinkButton>
        }
      />
    );
  } else if (state === "loading") {
    body = <SkeletonBlock lines={3} />;
  } else if (state === "populated") {
    body = (
      <ul className="home-list" aria-label="Recent packs">
        {packs.map((pack) => (
          <PackCard key={pack.packId} pack={pack} navigate={navigate} />
        ))}
      </ul>
    );
  } else if (state === "empty") {
    body = (
      <StatusBanner
        tone="info"
        icon="i"
        title="No packs yet"
        detail="Install a pack from the catalog to see it here."
        actions={
          <LinkButton to="/library" navigate={navigate}>
            Open the library
          </LinkButton>
        }
      />
    );
  } else if (state === "recoverable") {
    body = (
      <StatusBanner
        tone="warning"
        icon="!"
        title="Could not load recent packs"
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
        title="Pack list unavailable"
        detail={slice.error?.message ?? "The pack catalog could not be read."}
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
        title="Catalog access denied"
        detail={
          slice.error?.message ??
          "This profile cannot list installed packs."
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
      className="home-card home-card--recent"
      aria-labelledby="home-recent-heading"
    >
      <header className="home-card__header">
        <h2 className="home-card__title" id="home-recent-heading">
          Recent packs
        </h2>
        {state === "populated" && packs.length > 0 ? (
          <LinkButton to="/library" navigate={navigate} variant="ghost">
            See all
          </LinkButton>
        ) : null}
      </header>
      {body}
    </section>
  );
}
