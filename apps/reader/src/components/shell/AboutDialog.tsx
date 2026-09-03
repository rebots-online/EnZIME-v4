import { useEffect, useRef } from "react";
import type { MouseEvent as ReactMouseEvent, ReactNode } from "react";

const APP_VERSION: string =
  typeof __APP_VERSION__ !== "undefined" ? __APP_VERSION__ : "1.0.52684";

/**
 * §Mandatory App Chrome — Help | About dialog (BUILD_CONVENTIONS §"Mandatory
 * App Chrome" + R-BOOT rubric).
 *
 * Focus-managed modal (`role="dialog" aria-modal="true"`) with Aurora Glass
 * styling. Opened from the AppShell brand affordance; closes on Escape,
 * backdrop click, or the explicit Close button. While open, focus is moved
 * into the dialog and restored to the previously-focused element (the
 * trigger) on close.
 *
 * Controlled: parent owns `open` + `onClose`. Renders null when closed.
 */
export function AboutDialog({
  open,
  onClose,
}: {
  open: boolean;
  onClose: () => void;
}): ReactNode {
  const closeBtnRef = useRef<HTMLButtonElement>(null);

  // Escape-to-close + focus management. Re-runs only when `open` flips, so
  // the previously-focused element is captured at the moment of opening.
  useEffect(() => {
    if (!open) return;

    const previouslyFocused =
      (document.activeElement as HTMLElement | null) ?? null;

    // Move focus into the dialog.
    closeBtnRef.current?.focus();

    function onKey(e: KeyboardEvent): void {
      if (e.key === "Escape") {
        e.stopPropagation();
        onClose();
      }
    }
    document.addEventListener("keydown", onKey, true);

    return () => {
      document.removeEventListener("keydown", onKey, true);
      // Return focus to the trigger.
      previouslyFocused?.focus?.();
    };
  }, [open, onClose]);

  if (!open) return null;

  function handleBackdrop(e: ReactMouseEvent<HTMLDivElement>): void {
    // Only close when the scrim itself (not an inner element) is clicked.
    if (e.target === e.currentTarget) onClose();
  }

  return (
    <div
      className="ez-about__scrim"
      onClick={handleBackdrop}
      role="presentation"
    >
      <div
        className="ez-about"
        role="dialog"
        aria-modal="true"
        aria-labelledby="ez-about-title"
      >
        <div className="ez-about__mark" aria-hidden="true">Z</div>
        <h2 id="ez-about-title" className="ez-about__title">EnZIME Reader</h2>

        <dl className="ez-about__meta">
          <dt className="ez-about__meta-key">Version</dt>
          <dd className="ez-about__meta-val">v{APP_VERSION}</dd>
        </dl>

        <p className="ez-about__line">
          Copyright © 2026 Robin L. M. Cheung, MBA. All rights reserved.
        </p>
        <p className="ez-about__line ez-about__line--muted">
          Licensed per the project LICENSE.
        </p>

        <div className="ez-about__actions">
          <button
            ref={closeBtnRef}
            type="button"
            className="ez-about__close"
            onClick={onClose}
          >
            Close
          </button>
        </div>
      </div>
    </div>
  );
}
