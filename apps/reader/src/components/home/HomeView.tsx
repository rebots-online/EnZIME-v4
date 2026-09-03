import { useEffect, type ReactNode } from "react";
import { useHomeStore } from "../../stores/homeStore";
import { ContinueReadingCard } from "./ContinueReadingCard";
import { OpenPackCard } from "./OpenPackCard";
import { HomeStyles, useOnlineSubscription } from "./shared";
import { MemoryInboxCard } from "./MemoryInboxCard";
import { ReadinessPanel } from "./ReadinessPanel";
import { RecentPacksList } from "./RecentPacksList";
import { TaskInbox } from "./TaskInbox";

export function HomeView(): ReactNode {
  const loadAll = useHomeStore((s) => s.loadAll);
  const setOnline = useHomeStore((s) => s.setOnline);
  const online = useHomeStore((s) => s.online);
  const anyLoading = useHomeStore(
    (s) =>
      s.continueReading.status === "loading" ||
      s.recentPacks.status === "loading" ||
      s.taskInbox.status === "loading" ||
      s.memoryInbox.status === "loading" ||
      s.readiness.status === "loading",
  );

  useOnlineSubscription(setOnline);

  useEffect(() => {
    void loadAll();
  }, [loadAll]);

  return (
    <main className="home-surface" aria-busy={anyLoading}>
      <HomeStyles />
      <OpenPackCard />
      {!online ? (
        <p
          className="home-pill home-pill--warning"
          role="status"
          aria-live="polite"
          style={{ alignSelf: "flex-start" }}
        >
          ! Offline — showing cached content
        </p>
      ) : null}
      <div className="home-grid">
        <ContinueReadingCard />
        <ReadinessPanel />
        <RecentPacksList />
        <TaskInbox />
        <MemoryInboxCard />
      </div>
    </main>
  );
}
