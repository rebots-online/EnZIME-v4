import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { SettingsView } from "./SettingsView";
import { ConsentBanner } from "../../components/shell/ConsentBanner";
import { useEntitlementStore } from "../../stores/entitlementStore";
import { routeTemplate } from "../../services/analytics";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

const LICENSE = "enz_0123456789abcdefghjkmnpqrs";
const decisions = (paid: boolean) => [
  { capabilityKey: "reader.core", allowed: true, source: "core", validUntilMs: null, explanation: "" },
  {
    capabilityKey: "annotations.rich_media",
    allowed: paid,
    source: paid ? "signed_artifact" : "denied",
    validUntilMs: null,
    explanation: "",
  },
];
const plans = [
  { id: "pro_yearly", name: "EnZIME Pro — Yearly", priceLabel: "$36 / year", kind: "subscription", capabilities: ["annotations.rich_media"], highlight: true },
  { id: "lifetime", name: "EnZIME Lifetime", priceLabel: "$89 once", kind: "lifetime", capabilities: ["annotations.rich_media"], highlight: false },
];

type Handler = (args?: Record<string, unknown>) => unknown;
function mockBackend(overrides: Record<string, Handler> = {}) {
  let paid = false;
  let license: string | null = null;
  const base: Record<string, Handler> = {
    billing_status: () => ({ ok: true, value: { purchasingEnabled: true, licenseId: license, decisions: decisions(paid) } }),
    billing_plans: () => ({ ok: true, value: plans }),
    billing_checkout: () => {
      license = LICENSE;
      return { ok: true, value: { url: "https://pay.example/x", licenseId: LICENSE } };
    },
    entitlement_restore: (a) => {
      if (a?.licenseId !== LICENSE) return { ok: false, error: { code: "not_found", message: "No active purchase was found for this license key" } };
      paid = true;
      license = LICENSE;
      return { ok: true, value: { purchasingEnabled: true, licenseId: license, decisions: decisions(paid) } };
    },
    analytics_state: () => ({ configured: true, consent: "unset" }),
    analytics_set_consent: (a) => ({ configured: true, consent: a?.granted ? "granted" : "denied" }),
    analytics_track: () => undefined,
    ...overrides,
  };
  vi.mocked(invoke).mockImplementation(async (cmd: string, args?: unknown) => {
    const h = base[cmd];
    if (!h) throw new Error(`unexpected command ${cmd}`);
    return h(args as Record<string, unknown> | undefined) as never;
  });
}

const initial = useEntitlementStore.getState();
beforeEach(() => {
  vi.clearAllMocks();
  useEntitlementStore.setState(initial, true);
});
afterEach(cleanup);

it("shows free edition, plans, and opens checkout for the chosen plan", async () => {
  mockBackend();
  render(<SettingsView />);
  expect(await screen.findByText("Free edition")).toBeTruthy();
  expect(screen.getByText("Not included")).toBeTruthy();
  fireEvent.click(await screen.findByRole("button", { name: "Buy lifetime" }));
  await waitFor(() => expect(invoke).toHaveBeenCalledWith("billing_checkout", { planId: "lifetime" }));
  expect(await screen.findByText(/Checkout opened in your browser/)).toBeTruthy();
  expect(await screen.findByText(LICENSE)).toBeTruthy();
});

it("restores with a license key and reports a wrong key accessibly", async () => {
  mockBackend();
  render(<SettingsView />);
  const input = await screen.findByLabelText("Restore a purchase");
  fireEvent.change(input, { target: { value: "enz_wrongwrongwrongwrongwrongwr" } });
  fireEvent.click(screen.getByRole("button", { name: "Restore" }));
  expect((await screen.findByRole("alert")).textContent).toMatch(/No active purchase/);

  fireEvent.change(input, { target: { value: LICENSE } });
  fireEvent.click(screen.getByRole("button", { name: "Restore" }));
  expect(await screen.findByText("EnZIME Pro is active")).toBeTruthy();
  expect(screen.getAllByText("✓ Included")).toHaveLength(2);
});

it("hides purchasing in builds without a billing service but keeps owned features", async () => {
  mockBackend({
    billing_status: () => ({ ok: true, value: { purchasingEnabled: false, licenseId: null, decisions: decisions(false) } }),
  });
  render(<SettingsView />);
  expect(await screen.findByText(/Purchasing isn't available in this build/)).toBeTruthy();
  expect(screen.queryByRole("button", { name: "Buy lifetime" })).toBeNull();
});

it("consent banner sends nothing until Allow, then records consent", async () => {
  mockBackend();
  render(<ConsentBanner />);
  fireEvent.click(await screen.findByRole("button", { name: "Allow" }));
  await waitFor(() => expect(invoke).toHaveBeenCalledWith("analytics_set_consent", { granted: true }));
  const calls = vi.mocked(invoke).mock.calls.map((c) => c[0]);
  expect(calls.indexOf("analytics_track")).toBeGreaterThan(calls.indexOf("analytics_set_consent"));
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
});

it("route templates never carry ids or article keys", () => {
  expect(routeTemplate("/reader/01HXYZ/A%2FSecret_Title")).toBe("/reader/:packid/:articlekey");
  expect(routeTemplate("/settings")).toBe("/settings");
  expect(routeTemplate("/some/unknown/thing")).toBe("/other");
});
