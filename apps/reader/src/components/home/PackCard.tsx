import { type ReactNode } from "react";
import type { PackSummary } from "../../services/homeService";
import { formatBytes, formatRelative } from "./shared";

export function PackCard({
  pack,
  navigate,
}: {
  pack: PackSummary;
  navigate: (path: string) => void;
}): ReactNode {
  const openPack = () => navigate(`/library?pack=${encodeURIComponent(pack.packId)}`);
  const meta = [
    pack.articleCount === 1 ? "1 article" : `${pack.articleCount} articles`,
    formatBytes(pack.sizeBytes),
    pack.lastOpenedAt ? `opened ${formatRelative(pack.lastOpenedAt)}` : "not opened",
  ].join(" · ");

  return (
    <li className="home-list__item">
      <button
        type="button"
        className="home-row"
        onClick={openPack}
        aria-label={`Open pack ${pack.title} in library`}
      >
        <span className="home-row__main">
          <span className="home-row__label">{pack.title}</span>
          <span className="home-row__meta">
            {pack.subtitle ? `${pack.subtitle} · ` : ""}
            {meta}
          </span>
        </span>
        <span className="home-row__aside" aria-hidden="true">→</span>
      </button>
    </li>
  );
}
