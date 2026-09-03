// BreadcrumbNav — §15.1 chrome. Renders the Pack › Article location trail.
//
// The pack title may be unknown when the user arrives at the reader route by
// id (e.g. deep link / "Continue reading") without an in-session `reader_open`;
// in that case we show a neutral "Pack" label rather than a raw id.

import type { ReactNode } from "react";

export interface BreadcrumbNavProps {
  readonly packId: string;
  readonly packTitle: string | null;
  readonly articleTitle: string;
  readonly onHome: () => void;
}

export function BreadcrumbNav({
  packId,
  packTitle,
  articleTitle,
  onHome,
}: BreadcrumbNavProps): ReactNode {
  const crumbs: ReactNode[] = [
    <button
      key="home"
      type="button"
      className="reader-crumb"
      onClick={onHome}
    >
      Home
    </button>,
    <span key="pack" className="reader-crumb reader-crumb--muted" title={packId}>
      {packTitle ?? "Pack"}
    </span>,
  ];

  return (
    <nav className="reader-breadcrumb" aria-label="Article location">
      {crumbs.map((node, i) => (
        <span key={i} className="reader-breadcrumb__slot">
          {i > 0 ? (
            <span className="reader-breadcrumb__sep" aria-hidden="true">
              ›
            </span>
          ) : null}
          {node}
        </span>
      ))}
      <span className="reader-breadcrumb__slot">
        <span className="reader-breadcrumb__sep" aria-hidden="true">
          ›
        </span>
        <span className="reader-crumb reader-crumb--current" title={articleTitle}>
          {articleTitle}
        </span>
      </span>
    </nav>
  );
}
