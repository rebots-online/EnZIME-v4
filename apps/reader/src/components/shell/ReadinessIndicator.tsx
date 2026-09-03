import type { ReactNode } from "react";
import { useAppStore } from "../../stores/appStore";

/**
 * §16.6 ReadinessIndicator — a compact, non-colour-only readiness affordance
 * (icon + text) for the top bar. Reads the bootstrap readiness slice.
 * Colour is accompanied by a label so status never relies on colour alone
 * (§15.6 rule 4).
 */
const LABEL: Record<string, string> = {
  ready: "Ready",
  warning: "Warning",
  degraded: "Degraded",
  unavailable: "Unavailable",
};

export function ReadinessIndicator(): ReactNode {
  const status = useAppStore((s) => s.bootstrapStatus);
  const readiness = useAppStore((s) => s.readiness);

  const level =
    status === "loading" || status === "idle"
      ? "loading"
      : (readiness?.overall ?? "unavailable");
  const label =
    status === "loading" || status === "idle"
      ? "Starting"
      : (readiness?.summary || LABEL[level] || "Unknown");

  return (
    <button
      type="button"
      className={`ez-readiness ez-readiness--${level}`}
      aria-label={`System readiness: ${LABEL[level] ?? "unknown"}. ${label}`}
      title={label}
    >
      <span className="ez-readiness__dot" aria-hidden="true" />
      <span>{LABEL[level] ?? "Unknown"}</span>
    </button>
  );
}
