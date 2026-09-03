import { useEffect, useState } from "react";
import type { ReactNode } from "react";
import { useAppStore } from "../../stores/appStore";

const APP_VERSION: string =
  typeof __APP_VERSION__ !== "undefined" ? __APP_VERSION__ : "1.0.52684";

/** Minimum brand-impression hold (ms) — §Mandatory App Chrome. */
const MIN_HOLD_MS = 1400;
/** Fade-out duration (ms) — opacity 1→0 plus a slight scale-up. */
const FADE_MS = 320;

/**
 * §Mandatory App Chrome — Splash overlay (BUILD_CONVENTIONS §"Mandatory App
 * Chrome" + R-BOOT rubric).
 *
 * Launch overlay (Aurora Glass) that doubles as the loading state. Shows
 * brand mark, product name, full version, and copyright while the bootstrap
 * slice settles. Fades out (opacity 1→0 over FADE_MS plus a slight scale-up)
 * once BOTH:
 *   (a) the bootstrap slice has reached a terminal status — `ready` OR
 *       `error`. `error` is treated as terminal so a web-only dev runtime
 *       whose `app_bootstrap` invoke rejects still releases the splash
 *       (otherwise it would hang forever outside the Tauri shell), AND
 *   (b) at least MIN_HOLD_MS have elapsed (whichever is later).
 *
 * Mounted in App.tsx as a `position: fixed` overlay ABOVE RouterProvider, so
 * the router + AppShell initialise underneath and call `bootstrap()` while
 * the splash is visible. The splash never blocks app init — it simply sits
 * on top until dismissed, then unmounts (returns null).
 *
 * Animation is the default (Aurora Glass: drifting aurora planes, pulsing
 * brand glow, shimmer loading bar); `prefers-reduced-motion` is honoured by
 * the global CSS override in global.css.
 */
export function Splash(): ReactNode {
  const bootstrapStatus = useAppStore((s) => s.bootstrapStatus);
  const [holdElapsed, setHoldElapsed] = useState(false);
  const [phase, setPhase] = useState<"hold" | "fade" | "done">("hold");

  // Terminal = the bootstrap slice has settled (reported its state).
  const terminal =
    bootstrapStatus === "ready" || bootstrapStatus === "error";

  // Minimum brand-impression hold — guarantees the mark + version land even
  // when bootstrap resolves in well under MIN_HOLD_MS.
  useEffect(() => {
    const t = setTimeout(() => setHoldElapsed(true), MIN_HOLD_MS);
    return () => clearTimeout(t);
  }, []);

  // Dismiss gate: terminal bootstrap AND minimum hold elapsed (whichever is
  // later). Fade, then unmount.
  useEffect(() => {
    if (!terminal || !holdElapsed) return;
    setPhase("fade");
    const t = setTimeout(() => setPhase("done"), FADE_MS);
    return () => clearTimeout(t);
  }, [terminal, holdElapsed]);

  if (phase === "done") return null;

  const fading = phase === "fade";

  return (
    <div
      className={`ez-splash${fading ? " ez-splash--fading" : ""}`}
      role="status"
      aria-live="polite"
      aria-label={`EnZIME Reader version ${APP_VERSION} starting`}
    >
      <div className="ez-splash__aurora" aria-hidden="true">
        <span className="ez-splash__aurora-a" />
        <span className="ez-splash__aurora-b" />
      </div>

      <div className="ez-splash__content">
        <div className="ez-splash__mark" aria-hidden="true">Z</div>
        <h1 className="ez-splash__title">EnZIME Reader</h1>
        <div className="ez-splash__version">v{APP_VERSION}</div>

        <div
          className="ez-splash__shimmer"
          role="progressbar"
          aria-label="Loading"
          aria-busy="true"
        >
          <span className="ez-splash__shimmer-bar" />
        </div>

        <div className="ez-splash__copyright">Robin L. M. Cheung, MBA</div>
      </div>
    </div>
  );
}
