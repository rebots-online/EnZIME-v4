// InstallPlanSummary — compact detail table for a computed InstallPlan.
//
// Pairs with StorageImpactBar inside DownloadPlanDialog: the impact bar is
// the visual headline (does it fit?), this is the numeric detail (what
// exactly will land, how is it verified). Mirrors the home/reader pattern of
// small presentational components composed by their parent.

import { type ReactNode } from "react";
import type { InstallPlan } from "../../services/libraryService";
import { formatBytes } from "../../components/home/shared";

export function InstallPlanSummary({
  plan,
}: {
  plan: InstallPlan;
}): ReactNode {
  const verification = plan.verification;
  const catalog = plan.offlineCatalog;
  return (
    <div className="lib-summary" aria-label="Install plan summary">
      <Row k="Destination" v={plan.destination.displayName} />
      <Row
        k="Transfer mode"
        v={transferLabel(plan.transferMode)}
        accent
      />
      <Row k={plan.transferMode === "ranged" ? "Planned partial download" : "Download"} v={formatBytes(plan.downloadBytes)} />
      {catalog ? <>
        <Row k="Planned offline articles" v={catalog.plannedArticleCount.toLocaleString()} />
        <Row k="Planned offline resources" v={catalog.plannedResourceCount.toLocaleString()} />
        <Row k="Planned retained redirects" v={catalog.plannedAliasCount.toLocaleString()} />
        <Row k="Catalog entries inspected" v={`${catalog.scannedEntries.toLocaleString()} / ${catalog.totalEntries.toLocaleString()}`} />
        <Row k="Catalog transfer allowance used" v={`${formatBytes(catalog.networkReservedBytes)} / ${formatBytes(catalog.networkAllowanceBytes)}`} />
        <Row k="Catalog metadata received" v={formatBytes(catalog.networkReceivedBytes)} />
        <p>Planned entries become available offline after their content finishes downloading. Metadata traffic is separate from the planned content download; retained metadata uses your storage budget.</p>
        {!catalog.complete ? <p role="status">Partial catalog discovery: {catalog.stopReason === "metadata_budget" ? "the metadata storage budget was reached" : "the metadata transfer allowance was reached"}. This plan covers the indexed subset shown above, not every article in the source archive.</p> : null}
      </> : null}
      <Row k="Temporary scratch" v={formatBytes(plan.temporaryBytes)} />
      <Row k="Final footprint" v={formatBytes(plan.finalBytes)} />
      <Row k="Free before" v={formatBytes(plan.freeBytesBefore)} />
      <Row
        k="Free after"
        v={formatBytes(Math.max(0, plan.freeBytesAfter))}
        accent={plan.freeBytesAfter >= 0}
      />
      <Row
        k="Full archive size"
        v={formatBytes(verification.expectedSizeBytes)}
      />
      <Row k="Upstream full archive SHA-256" v={shortHash(verification.expectedSha256)} />
      {plan.transferMode === "ranged" ? <p>Partial download: source consistency is pinned and retained data uses local integrity checks. The full archive SHA-256 is not verified by this download.</p> : null}
      <Row
        k="Signature"
        v={verification.signatureRequired ? "required" : "not required"}
      />
      <Row
        k="ZIM structure check"
        v={verification.zimStructureCheck ? "enabled" : "disabled"}
      />
    </div>
  );
}

function Row({
  k,
  v,
  accent,
}: {
  k: string;
  v: string;
  accent?: boolean;
}): ReactNode {
  return (
    <div className={`lib-summary__row${accent ? " lib-summary__row--accent" : ""}`}>
      <span className="lib-summary__k">{k}</span>
      <span className="lib-summary__v">{v}</span>
    </div>
  );
}

function shortHash(sha: string | null): string {
  if (!sha) return "Not provided by source";
  if (sha.length <= 14) return sha;
  return `${sha.slice(0, 8)}…${sha.slice(-4)}`;
}

function transferLabel(mode: InstallPlan["transferMode"]): string {
  switch (mode) {
    case "ranged":
      return "ranged (DynDon subset)";
    case "full":
      return "full download";
    case "local_copy":
      return "local copy";
    default:
      return mode;
  }
}
