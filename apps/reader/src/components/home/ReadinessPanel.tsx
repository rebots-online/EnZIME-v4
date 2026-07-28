import { type ReactNode } from "react";
import { useNavigate } from "react-router-dom";
import { useHomeStore } from "../../stores/homeStore";
import type { ReadinessLevel } from "../../services/homeService";
import {
  LinkButton,
  RetryButton,
  SkeletonBlock,
  StatusBanner,
  resolveState,
} from "./shared";

function overallTone(level: ReadinessLevel): {
  pill: string;
  icon: string;
} {
  switch (level) {
    case "ready":
      return { pill: "home-pill home-pill--ready", icon: "✓" };
    case "warning":
      return { pill: "home-pill home-pill--warning", icon: "!" };
    case "degraded":
      return { pill: "home-pill home-pill--degraded", icon: "!" };
    case "unavailable":
      return { pill: "home-pill home-pill--unavailable", icon: "×" };
    default:
      return { pill: "home-pill", icon: "?" };
  }
}

function levelLabel(level: ReadinessLevel): string {
  switch (level) {
    case "ready":
      return "Ready";
    case "warning":
      return "Warning";
    case "degraded":
      return "Degraded";
    case "unavailable":
      return "Unavailable";
    default:
      return "Unknown";
  }
}

function ReadinessFlagRow({
  flag,
  navigate,
}: {
  flag: {
    readonly id: string;
    readonly level: "warning" | "degraded" | "unavailable";
    readonly label: string;
    readonly detail: string;
    readonly settingPath: string;
  };
  navigate: (path: string) => void;
}): ReactNode {
  const tone = overallTone(flag.level);
  return (
    <li className="home-list__item">
      <button
        type="button"
        className="home-row"
        onClick={() => navigate(flag.settingPath)}
        aria-label={`${flag.label}: ${flag.detail}. Open settings to review.`}
      >
        <span className="home-row__main">
          <span className="home-flag__head">
            <span
              className={tone.pill}
              aria-hidden="true"
            >
              {tone.icon} {levelLabel(flag.level)}
            </span>
            <span className="home-flag__title">{flag.label}</span>
          </span>
          <span className="home-row__meta">{flag.detail}</span>
        </span>
        <span className="home-row__aside">review →</span>
      </button>
    </li>
  );
}

export function ReadinessPanel(): ReactNode {
  const navigate = useNavigate();
  const online = useHomeStore((s) => s.online);
  const slice = useHomeStore((s) => s.readiness);
  const reload = useHomeStore((s) => s.loadReadiness);
  const recheck = useHomeStore((s) => s.runReadinessCheck);

  const state = resolveState(online, slice, (data) => data !== null);
  const report = slice.data;

  let body: ReactNode;
  if (state === "offline-empty") {
    body = (
      <StatusBanner
        tone="offline"
        icon="!"
        title="Offline"
        detail="Reconnect to assess system readiness."
      />
    );
  } else if (state === "loading") {
    body = <SkeletonBlock lines={2} />;
  } else if (state === "populated" && report) {
    const tone = overallTone(report.overall);
    body = (
      <div className="home-readiness">
        <div
          className="home-readiness__overall"
          aria-label={`System readiness: ${levelLabel(report.overall)}`}
        >
          <span className={tone.pill} aria-hidden="true">
            {tone.icon}
          </span>
          <span>{report.summary || levelLabel(report.overall)}</span>
        </div>
        {report.flags.length === 0 ? (
          <p className="home-readiness__summary">
            No outstanding configuration issues.
          </p>
        ) : (
          <ul className="home-list home-readiness__flags" aria-label="Readiness flags">
            {report.flags.map((flag) => (
              <ReadinessFlagRow
                key={flag.id}
                flag={flag}
                navigate={navigate}
              />
            ))}
          </ul>
        )}
        <div className="home-status__actions">
          <button
            type="button"
            className="home-button home-button--ghost"
            onClick={() => void recheck()}
            aria-label="Re-run readiness check"
          >
            Re-run check
          </button>
        </div>
      </div>
    );
  } else if (state === "empty") {
    body = (
      <StatusBanner
        tone="info"
        icon="i"
        title="No readiness report yet"
        detail="Run the readiness check to inspect capabilities."
        actions={
          <button
            type="button"
            className="home-button"
            onClick={() => void recheck()}
          >
            Run check
          </button>
        }
      />
    );
  } else if (state === "recoverable") {
    body = (
      <StatusBanner
        tone="warning"
        icon="!"
        title="Could not load readiness"
        detail={slice.error?.message ?? "Unexpected error."}
        actions={
          <>
            <RetryButton onClick={reload} label="Try again" />
            <RetryButton
              onClick={() => void recheck()}
              label="Re-run check"
            />
          </>
        }
      />
    );
  } else if (state === "non-recoverable") {
    body = (
      <StatusBanner
        tone="error"
        icon="×"
        title="Readiness probe failed"
        detail={
          slice.error?.message ??
          "The readiness probe could not complete."
        }
        actions={
          <LinkButton to="/settings" navigate={navigate}>
            Open settings
          </LinkButton>
        }
      />
    );
  } else {
    body = (
      <StatusBanner
        tone="locked"
        icon="∅"
        title="Readiness check unavailable"
        detail={
          slice.error?.message ??
          "This profile cannot run the readiness check."
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
      className="home-card home-card--readiness"
      aria-labelledby="home-readiness-heading"
    >
      <header className="home-card__header">
        <h2 className="home-card__title" id="home-readiness-heading">
          Readiness
        </h2>
        {state === "populated" && report ? (
          <span className={overallTone(report.overall).pill}>
            {levelLabel(report.overall)}
          </span>
        ) : null}
      </header>
      {body}
    </section>
  );
}
