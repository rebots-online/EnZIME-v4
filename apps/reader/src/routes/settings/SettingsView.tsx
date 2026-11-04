// SettingsView — §15.3 `/settings`. Hosts the Plan (EntitlementPanel, §16)
// and Privacy sections. Purchase happens on a hosted checkout in the user's
// browser; this screen shows CapabilityDecision[] only (§13.5) and never
// hides or blocks free/core features.

import { useEffect, useState, type FormEvent, type ReactNode } from "react";
import { CAPABILITY_LABELS, type Plan } from "../../services/billingService";
import {
  getAnalyticsState,
  setAnalyticsConsent,
  track,
  type AnalyticsState,
} from "../../services/analytics";
import { useEntitlementStore } from "../../stores/entitlementStore";
import "./settings.css";

const POLL_MS = 5000;
const POLL_LIMIT_MS = 10 * 60 * 1000;

function formatDate(ms: number): string {
  return new Date(ms).toLocaleDateString(undefined, { year: "numeric", month: "short", day: "numeric" });
}

function PlanCard({ plan, onBuy, busy, disabled }: {
  plan: Plan;
  onBuy: (id: string) => void;
  busy: boolean;
  disabled: boolean;
}): ReactNode {
  return (
    <li className={`ez-plan${plan.highlight ? " ez-plan--highlight" : ""}`}>
      {plan.highlight && <span className="ez-plan__badge">Best value</span>}
      <h3 className="ez-plan__name">{plan.name}</h3>
      <p className="ez-plan__price">{plan.priceLabel}</p>
      <p className="ez-plan__kind">
        {plan.kind === "lifetime" ? "One payment. Yours to keep, offline, forever." : "Renews automatically. Cancel anytime."}
      </p>
      <ul className="ez-plan__caps">
        {plan.capabilities.map((c) => (
          <li key={c}>{CAPABILITY_LABELS[c]?.title ?? c}</li>
        ))}
      </ul>
      <button
        type="button"
        className="ez-btn ez-btn--primary"
        disabled={disabled}
        aria-busy={busy}
        onClick={() => onBuy(plan.id)}
      >
        {busy ? "Opening checkout…" : plan.kind === "lifetime" ? "Buy lifetime" : "Subscribe"}
      </button>
    </li>
  );
}

function PlanSection(): ReactNode {
  const { billing, plans, busy, notice, awaitingPurchase } = useEntitlementStore();
  const { load, loadPlans, checkout, refresh, restore, portal, stopAwaiting, clearNotice } =
    useEntitlementStore.getState();
  const [restoreKey, setRestoreKey] = useState("");
  const [copied, setCopied] = useState(false);
  const [pendingPlan, setPendingPlan] = useState<string | null>(null);

  useEffect(() => {
    void load();
    void loadPlans();
    track({ name: "view_plans" });
  }, [load, loadPlans]);

  // After checkout opens, poll quietly until the plan lands (bounded).
  useEffect(() => {
    if (!awaitingPurchase) return;
    const started = Date.now();
    const id = window.setInterval(() => {
      if (Date.now() - started > POLL_LIMIT_MS) {
        stopAwaiting();
        return;
      }
      void refresh({ silent: true });
    }, POLL_MS);
    return () => window.clearInterval(id);
  }, [awaitingPurchase, refresh, stopAwaiting]);

  const status = billing.status === "ready" ? billing.data : null;
  const paid = status?.decisions.filter((d) => d.allowed && d.source !== "core") ?? [];
  const purchasing = status?.purchasingEnabled ?? false;

  const onBuy = (planId: string): void => {
    setPendingPlan(planId);
    track({ name: "select_plan", params: { plan_id: planId } });
    void checkout(planId).finally(() => setPendingPlan(null));
  };

  const onRestore = (e: FormEvent): void => {
    e.preventDefault();
    void restore(restoreKey).then((ok) => ok && setRestoreKey(""));
  };

  const copyLicense = async (): Promise<void> => {
    if (!status?.licenseId) return;
    try {
      await navigator.clipboard.writeText(status.licenseId);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 2000);
    } catch {
      /* clipboard unavailable: the key stays visible and selectable */
    }
  };

  return (
    <section className="ez-settings__section" aria-labelledby="plan-heading">
      <h2 id="plan-heading">Plan</h2>

      {notice && (
        <div className={`ez-notice ez-notice--${notice.kind}`} role={notice.kind === "error" ? "alert" : "status"}>
          <span>{notice.text}</span>
          <button type="button" className="ez-btn ez-btn--ghost" onClick={clearNotice} aria-label="Dismiss message">×</button>
        </div>
      )}

      <div className="ez-card">
        <h3 className="ez-card__title">{paid.length > 0 ? "EnZIME Pro is active" : "Free edition"}</h3>
        <p className="ez-card__lead">
          Offline reading, highlights, comments, bookmarks and export are free forever. Paid
          features stay unlocked offline once verified on this device.
        </p>
        {billing.status === "loading" && <p className="ez-muted">Checking your plan…</p>}
        {billing.status === "error" && <p className="ez-muted" role="alert">{billing.message}</p>}
        {status && (
          <ul className="ez-caps" aria-label="Features on this device">
            {status.decisions.map((d) => {
              const label = CAPABILITY_LABELS[d.capabilityKey];
              return (
                <li key={d.capabilityKey} className={d.allowed ? "is-on" : "is-off"}>
                  <span className="ez-caps__state">{d.allowed ? "✓ Included" : "Not included"}</span>
                  <span className="ez-caps__title">{label?.title ?? d.capabilityKey}</span>
                  <span className="ez-caps__detail">
                    {label?.detail}
                    {d.allowed && d.validUntilMs ? ` Renews or ends ${formatDate(d.validUntilMs)}.` : ""}
                    {d.source === "grace_policy" ? " Grace period: reconnect to renew." : ""}
                  </span>
                </li>
              );
            })}
          </ul>
        )}
        {purchasing && status?.licenseId && (
          <div className="ez-actions">
            <button type="button" className="ez-btn" onClick={() => void refresh()} disabled={busy !== null} aria-busy={busy === "refresh"}>
              {busy === "refresh" ? "Checking…" : "Refresh"}
            </button>
            <button type="button" className="ez-btn" onClick={() => void portal()} disabled={busy !== null} aria-busy={busy === "portal"}>
              Manage billing
            </button>
          </div>
        )}
        {awaitingPurchase && (
          <p className="ez-muted" role="status">Waiting for payment confirmation… you can keep using EnZIME meanwhile.</p>
        )}
      </div>

      {purchasing ? (
        <>
          {plans.status === "loading" && <p className="ez-muted">Loading plans…</p>}
          {plans.status === "error" && (
            <p className="ez-muted" role="alert">
              Plans are unavailable right now ({plans.message}).{" "}
              <button type="button" className="ez-link" onClick={() => void loadPlans()}>Try again</button>
            </p>
          )}
          {plans.status === "ready" && (
            <ul className="ez-plans" aria-label="Available plans">
              {plans.data.map((p) => (
                <PlanCard
                  key={p.id}
                  plan={p}
                  onBuy={onBuy}
                  busy={busy === "checkout" && pendingPlan === p.id}
                  disabled={busy !== null}
                />
              ))}
            </ul>
          )}

          <div className="ez-card">
            <h3 className="ez-card__title">License key</h3>
            {status?.licenseId ? (
              <>
                <p className="ez-card__lead">Keep this key to restore your purchase on another device.</p>
                <div className="ez-license">
                  <code className="ez-license__key">{status.licenseId}</code>
                  <button type="button" className="ez-btn" onClick={() => void copyLicense()}>
                    {copied ? "Copied" : "Copy"}
                  </button>
                </div>
              </>
            ) : (
              <p className="ez-card__lead">You'll get a license key with your first purchase.</p>
            )}
            <form className="ez-restore" onSubmit={onRestore}>
              <label htmlFor="restore-key">Restore a purchase</label>
              <div className="ez-restore__row">
                <input
                  id="restore-key"
                  className="ez-input"
                  placeholder="enz_…"
                  autoComplete="off"
                  spellCheck={false}
                  value={restoreKey}
                  onChange={(e) => setRestoreKey(e.target.value)}
                />
                <button type="submit" className="ez-btn" disabled={busy !== null || restoreKey.trim().length === 0} aria-busy={busy === "restore"}>
                  {busy === "restore" ? "Restoring…" : "Restore"}
                </button>
              </div>
            </form>
          </div>
        </>
      ) : (
        status && (
          <p className="ez-muted">
            Purchasing isn't available in this build. Features you already own stay unlocked offline.
          </p>
        )
      )}
    </section>
  );
}

function PrivacySection(): ReactNode {
  const [state, setState] = useState<AnalyticsState | null>(null);

  useEffect(() => {
    getAnalyticsState().then(setState).catch(() => setState(null));
  }, []);

  if (!state?.configured) return null;
  const on = state.consent === "granted";

  return (
    <section className="ez-settings__section" aria-labelledby="privacy-heading">
      <h2 id="privacy-heading">Privacy</h2>
      <div className="ez-card">
        <label className="ez-toggle">
          <input
            type="checkbox"
            checked={on}
            onChange={(e) => void setAnalyticsConsent(e.target.checked).then(setState)}
          />
          <span>
            <strong>Share anonymous usage statistics</strong>
            <span className="ez-caps__detail">
              Screen visits and feature use (for example “opened Library”) are sent to Google
              Analytics under a random install id. Never sent: article titles, searches, notes,
              recordings, file names or your license key. Off by default.
            </span>
          </span>
        </label>
      </div>
    </section>
  );
}

export function SettingsView(): ReactNode {
  return (
    <div className="ez-settings">
      <header className="ez-settings__header">
        <h1>Settings</h1>
      </header>
      <PlanSection />
      <PrivacySection />
    </div>
  );
}
