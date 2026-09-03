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
  return (
    <div className="lib-summary" aria-label="Install plan summary">
      <Row k="Destination" v={plan.destination.displayName} />
      <Row
        k="Transfer mode"
        v={transferLabel(plan.transferMode)}
        accent
      />
      <Row k="Download" v={formatBytes(plan.downloadBytes)} />
      <Row k="Temporary scratch" v={formatBytes(plan.temporaryBytes)} />
      <Row k="Final footprint" v={formatBytes(plan.finalBytes)} />
      <Row k="Free before" v={formatBytes(plan.freeBytesBefore)} />
      <Row
        k="Free after"
        v={formatBytes(Math.max(0, plan.freeBytesAfter))}
        accent={plan.freeBytesAfter >= 0}
      />
      <Row
        k="Expected size"
        v={formatBytes(verification.expectedSizeBytes)}
      />
      <Row k="SHA-256" v={shortHash(verification.expectedSha256)} />
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

function shortHash(sha: string): string {
  if (!sha) return "—";
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
