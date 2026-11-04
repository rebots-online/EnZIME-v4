import { useEffect, useState, type ReactNode } from "react";
import {
  getAnalyticsState,
  setAnalyticsConsent,
  track,
  type AnalyticsState,
} from "../../services/analytics";
import "../../routes/settings/settings.css";

/**
 * Asks once, only when analytics is configured in this build and the user
 * has not decided. "Not now" records a refusal; Settings → Privacy can change
 * it at any time. Nothing is sent before "Allow".
 */
export function ConsentBanner(): ReactNode {
  const [state, setState] = useState<AnalyticsState | null>(null);

  useEffect(() => {
    getAnalyticsState().then(setState).catch(() => setState(null));
  }, []);

  if (!state?.configured || state.consent !== "unset") return null;

  const decide = (granted: boolean): void => {
    void setAnalyticsConsent(granted)
      .then((s) => {
        setState(s);
        if (granted) track({ name: "app_open" });
      })
      .catch(() => setState(null));
  };

  return (
    <div className="ez-consent" role="dialog" aria-labelledby="consent-title" aria-live="polite">
      <p id="consent-title">
        <strong>Help improve EnZIME?</strong> Share anonymous usage statistics (which screens
        and features get used). Your reading, searches and notes are never sent.
      </p>
      <div className="ez-actions" style={{ marginTop: 0 }}>
        <button type="button" className="ez-btn ez-btn--primary" onClick={() => decide(true)}>
          Allow
        </button>
        <button type="button" className="ez-btn" onClick={() => decide(false)}>
          Not now
        </button>
      </div>
    </div>
  );
}
