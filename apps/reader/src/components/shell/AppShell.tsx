import { useEffect, useState } from "react";
import { Outlet, useLocation } from "react-router-dom";
import type { ReactNode } from "react";
import { useAppStore } from "../../stores/appStore";
import { HomeStyles } from "../home/shared";
import { NavigationRail } from "./NavigationRail";
import { JobStrip } from "./JobStrip";
import { ReadinessIndicator } from "./ReadinessIndicator";
import { AboutDialog } from "./AboutDialog";
import { ConsentBanner } from "./ConsentBanner";
import { routeTemplate, track } from "../../services/analytics";
import { useEntitlementStore } from "../../stores/entitlementStore";

const APP_VERSION: string =
  typeof __APP_VERSION__ !== "undefined" ? __APP_VERSION__ : "1.0.52684";

/**
 * §16.6 AppShell — the Aurora Glass chrome that hosts every route via its
 * <Outlet/>. Per §15.1 the shell owns NavigationRail, GlobalSearchButton,
 * JobStrip, ReadinessIndicator, RouteOutlet, ToastRegion and CommandPalette.
 */
export function AppShell(): ReactNode {
  const bootstrap = useAppStore((s) => s.bootstrap);
  const [aboutOpen, setAboutOpen] = useState(false);

  const location = useLocation();
  const loadEntitlements = useEntitlementStore((s) => s.load);

  useEffect(() => {
    void bootstrap();
    void loadEntitlements();
    track({ name: "app_open" });
  }, [bootstrap, loadEntitlements]);

  // Route templates only — ids and article keys never leave the device.
  useEffect(() => {
    track({ name: "page_view", params: { page_path: routeTemplate(location.pathname) } });
  }, [location.pathname]);

  return (
    <div className="ez-shell">
      <HomeStyles />
      <div className="ez-shell__aurora" aria-hidden="true" />

      <header className="ez-shell__topbar">
        {/* §Mandatory App Chrome: the brand area is the Help | About trigger.
            It's a real <button> so it lands in the tab order and exposes the
            "About EnZIME Reader" affordance to AT. Styled to look like plain
            brand chrome (unobtrusive). */}
        <button
          type="button"
          className="ez-shell__brand"
          aria-label="About EnZIME Reader"
          title="About EnZIME Reader"
          onClick={() => setAboutOpen(true)}
        >
          <span className="ez-shell__brand-mark" aria-hidden="true">Z</span>
          <span className="ez-shell__brand-text">EnZIME Reader</span>
        </button>
        <button type="button" className="ez-shell__search" aria-label="Search local library">
          <span aria-hidden="true">⌕</span>
          <span>Search packs, notes, memory…</span>
        </button>
        <span className="ez-shell__top-actions">
          <ReadinessIndicator />
        </span>
      </header>

      <NavigationRail />

      <main className="ez-shell__main">
        <JobStrip />
        <div className="ez-route-outlet">
          <Outlet />
        </div>
      </main>

      <div id="app-version" aria-label="application version">
        EnZIME Reader v{APP_VERSION}
      </div>
      {/* ToastRegion is mounted by the ToastProvider in the App tree. */}

      <ConsentBanner />

      <AboutDialog
        open={aboutOpen}
        onClose={() => setAboutOpen(false)}
      />
    </div>
  );
}
