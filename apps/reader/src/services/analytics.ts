import { invoke } from "@tauri-apps/api/core";

// Opt-in GA4 analytics. The native side owns the allow-list and drops
// anything that is not a short machine token, so content (titles, search
// text, notes) cannot leave the device even if a caller passes it by mistake.
// This module never throws and never blocks the UI.

export type AnalyticsEvent =
  | { name: "page_view"; params: { page_path: string } }
  | { name: "app_open" }
  | { name: "view_plans" }
  | { name: "select_plan"; params: { plan_id: string } }
  | { name: "begin_checkout"; params: { plan_id: string } }
  | { name: "purchase_refresh"; params: { result: "granted" | "none" | "error" } }
  | { name: "restore_purchase"; params: { result: "granted" | "none" | "error" } }
  | { name: "open_billing_portal" }
  | { name: "capability_gate"; params: { capability: string } }
  | { name: "pack_install_start" }
  | { name: "annotation_create"; params: { kind: string } }
  | { name: "title_search" };

export type AnalyticsState = {
  readonly configured: boolean;
  readonly consent: "granted" | "denied" | "unset";
};

export function track(event: AnalyticsEvent): void {
  const params = "params" in event ? (event.params as Record<string, string>) : {};
  void invoke("analytics_track", { event: { name: event.name, params } }).catch(() => undefined);
}

export const getAnalyticsState = (): Promise<AnalyticsState> =>
  invoke<AnalyticsState>("analytics_state");

export const setAnalyticsConsent = (granted: boolean): Promise<AnalyticsState> =>
  invoke<AnalyticsState>("analytics_set_consent", { granted });

/**
 * Collapse a concrete location into its route template so ids never leave
 * the device: `/reader/01HX…/A%2FFoo` → `/reader/:packid/:articlekey`.
 */
export function routeTemplate(pathname: string): string {
  if (pathname.startsWith("/reader/")) return "/reader/:packid/:articlekey";
  const known = ["/", "/library", "/notes", "/settings"];
  return known.includes(pathname) ? pathname : "/other";
}
