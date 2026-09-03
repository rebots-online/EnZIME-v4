import { useLocation, useNavigate } from "react-router-dom";
import type { ReactNode } from "react";

/**
 * §16.6 NavigationRail. A destination is either `built` (navigates to a
 * registered route) or `scheduled` (rendered, not yet navigable). The set of
 * built destinations grows by milestone; scheduled entries are visually
 * deemphasized and announce their section name only — never a placeholder.
 */
type Destination = {
  readonly path: string;
  readonly label: string;
  readonly glyph: string;
  readonly built: boolean;
};

const DESTINATIONS: readonly Destination[] = [
  { path: "/", label: "Home", glyph: "⌂", built: true },
  { path: "/library", label: "Library", glyph: "▤", built: true },
  { path: "/ask", label: "Ask", glyph: "◆", built: false },
  { path: "/notes", label: "Notes", glyph: "✎", built: false },
  { path: "/memory", label: "Memory", glyph: "◈", built: false },
  { path: "/settings", label: "Settings", glyph: "⚙", built: false },
];

export function NavigationRail(): ReactNode {
  const navigate = useNavigate();
  const location = useLocation();

  return (
    <nav className="ez-shell__rail" aria-label="Primary">
      {DESTINATIONS.map((d) => {
        const active = d.built && location.pathname === d.path;
        const className = [
          "ez-nav-item",
          active ? "ez-nav-item--active" : "",
          d.built ? "" : "ez-nav-item--scheduled",
        ]
          .filter(Boolean)
          .join(" ");
        return (
          <a
            key={d.path}
            href={d.built ? `#${d.path}` : undefined}
            className={className}
            aria-current={active ? "page" : undefined}
            aria-disabled={d.built ? undefined : true}
            tabIndex={d.built ? 0 : -1}
            onClick={(e) => {
              if (!d.built) {
                e.preventDefault();
                return;
              }
              e.preventDefault();
              navigate(d.path);
            }}
          >
            <span className="ez-nav-item__glyph" aria-hidden="true">
              {d.glyph}
            </span>
            <span>{d.label}</span>
          </a>
        );
      })}
    </nav>
  );
}
