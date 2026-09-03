// StorageImpactBar — the DynDon payoff moment.
//
// Renders the storage "fit" for a computed InstallPlan: a horizontal bar
// whose full width is the destination's free space before install, with
// segments for the durable footprint (`finalBytes`) and the transient
// scratch (`temporaryBytes`), plus a marker showing where the FULL pack
// (`artifactSizeBytes`) would end. When the plan downloads less than the
// full artifact (`downloadBytes < artifactSizeBytes`), the subset decision is
// the whole point of DynDon — the marker visually calls out the slice that
// was *not* downloaded because the full pack wouldn't fit.
//
// Fit classification:
//   freeBytesAfter  < 0                      → "won't fit" (defensive — Rust
//                                              should refuse to plan these)
//   freeBytesAfter  < temporaryBytes         → "too tight" (warning)
//   otherwise                               → "fits" (success, animated)

import { type ReactNode } from "react";
import type { InstallPlan, TransferMode } from "../../services/libraryService";
import { formatBytes } from "../../components/home/shared";

export function StorageImpactBar({
  plan,
}: {
  plan: InstallPlan;
}): ReactNode {
  const free = Math.max(0, plan.freeBytesBefore);
  const finalBytes = Math.max(0, plan.finalBytes);
  const tempBytes = Math.max(0, plan.temporaryBytes);
  const downloadBytes = Math.max(0, plan.downloadBytes);
  const artifactBytes = Math.max(0, plan.catalogEntry.artifactSizeBytes);
  const after = plan.freeBytesAfter;

  const isSubset = downloadBytes < artifactBytes;

  // Fractions of the pre-install free space (clamped to the bar canvas).
  const frac = (bytes: number): number =>
    free > 0 ? Math.max(0, Math.min(1, bytes / free)) : 0;

  const finalPct = frac(finalBytes) * 100;
  const tempPct = frac(tempBytes) * 100;
  const markerPct = frac(artifactBytes) * 100;

  const fit: "fits" | "tight" | "no" =
    after < 0 ? "no" : after < tempBytes ? "tight" : "fits";

  const fitLabel =
    fit === "fits" ? "Fits" : fit === "tight" ? "Too tight" : "Won't fit";
  const fitBadgeClass =
    fit === "fits"
      ? "lib-fit-badge lib-fit-badge--fits"
      : fit === "tight"
        ? "lib-fit-badge lib-fit-badge--tight"
        : "lib-fit-badge lib-fit-badge--no";

  return (
    <div className="lib-impact">
      <div className="lib-impact__headline">
        <span>
          Pack <strong>{formatBytes(artifactBytes)}</strong> → plan downloads{" "}
          <strong>{formatBytes(downloadBytes)}</strong>
          {isSubset ? (
            <>
              {" "}
              <span className="lib-pill lib-pill--accent">
                subset · {transferLabel(plan.transferMode)}
              </span>
            </>
          ) : (
            <>
              {" "}
              <span className="lib-pill">{transferLabel(plan.transferMode)}</span>
            </>
          )}
        </span>
        <span className={fitBadgeClass} aria-label={`Storage fit: ${fitLabel}`}>
          {fit === "fits" ? "✓ " : fit === "tight" ? "! " : "× "}
          {fitLabel}
        </span>
      </div>

      <div
        className="lib-bar"
        role="img"
        aria-label={`Storage fit bar: ${formatBytes(finalBytes)} durable plus ${formatBytes(tempBytes)} temporary of ${formatBytes(free)} free, leaving ${formatBytes(Math.max(0, after))}.`}
      >
        <span className="lib-bar__track-free" />
        <span
          className="lib-bar__seg lib-bar__seg--final"
          style={{ width: `${finalPct}%` }}
        />
        <span
          className="lib-bar__seg lib-bar__seg--temp"
          style={{ left: `${finalPct}%`, width: `${tempPct}%` }}
        />
        {isSubset && markerPct > 0 && markerPct <= 100 ? (
          <span
            className="lib-bar__marker"
            style={{ left: `${markerPct}%` }}
            data-label={`full ${formatBytes(artifactBytes)}`}
          />
        ) : null}
      </div>

      <div className="lib-impact__headline" style={{ fontSize: "0.9em" }}>
        <span>
          Leaving <strong>{formatBytes(Math.max(0, after))}</strong> free after
          install.
        </span>
      </div>

      <div className="lib-impact__legend">
        <span>
          <span
            className="lib-legend__swatch"
            style={{
              background: "linear-gradient(90deg, var(--lib-accent), var(--lib-teal))",
            }}
          />
          {formatBytes(finalBytes)} kept
        </span>
        <span>
          <span
            className="lib-legend__swatch"
            style={{ background: "rgba(255,202,106,0.55)" }}
          />
          {formatBytes(tempBytes)} temp
        </span>
        {isSubset ? (
          <span>
            <span
              className="lib-legend__swatch"
              style={{ background: "var(--lib-fg)" }}
            />
            full pack {formatBytes(artifactBytes)}
          </span>
        ) : null}
      </div>
    </div>
  );
}

function transferLabel(mode: TransferMode): string {
  switch (mode) {
    case "ranged":
      return "ranged";
    case "full":
      return "full";
    case "local_copy":
      return "local copy";
    default:
      return mode;
  }
}
