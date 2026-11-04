import { create } from "zustand";
import {
  BillingError,
  billingService,
  type BillingStatus,
  type CapabilityDecision,
  type Plan,
} from "../services/billingService";
import { track } from "../services/analytics";

type Load<T> =
  | { status: "idle" }
  | { status: "loading" }
  | { status: "ready"; data: T }
  | { status: "error"; message: string };

type Busy = "checkout" | "refresh" | "restore" | "portal" | null;

type EntitlementState = {
  readonly billing: Load<BillingStatus>;
  readonly plans: Load<Plan[]>;
  readonly busy: Busy;
  readonly notice: { kind: "success" | "error" | "info"; text: string } | null;
  /** Set after a checkout opens; drives a bounded refresh poll. */
  readonly awaitingPurchase: boolean;

  load: () => Promise<void>;
  loadPlans: () => Promise<void>;
  checkout: (planId: string) => Promise<void>;
  refresh: (opts?: { silent?: boolean }) => Promise<void>;
  restore: (licenseId: string) => Promise<boolean>;
  portal: () => Promise<void>;
  stopAwaiting: () => void;
  clearNotice: () => void;
};

const message = (e: unknown): string =>
  e instanceof BillingError || e instanceof Error ? e.message : "Something went wrong";

const paidCount = (s: BillingStatus): number =>
  s.decisions.filter((d) => d.allowed && d.source !== "core").length;

export const useEntitlementStore = create<EntitlementState>((set, get) => ({
  billing: { status: "idle" },
  plans: { status: "idle" },
  busy: null,
  notice: null,
  awaitingPurchase: false,

  load: async () => {
    if (get().billing.status !== "ready") set({ billing: { status: "loading" } });
    try {
      set({ billing: { status: "ready", data: await billingService.status() } });
    } catch (e) {
      set({ billing: { status: "error", message: message(e) } });
    }
  },

  loadPlans: async () => {
    set({ plans: { status: "loading" } });
    try {
      set({ plans: { status: "ready", data: await billingService.plans() } });
    } catch (e) {
      set({ plans: { status: "error", message: message(e) } });
    }
  },

  checkout: async (planId) => {
    set({ busy: "checkout", notice: null });
    track({ name: "begin_checkout", params: { plan_id: planId } });
    try {
      await billingService.checkout(planId);
      set({
        awaitingPurchase: true,
        notice: { kind: "info", text: "Checkout opened in your browser. Your plan activates here as soon as payment completes." },
      });
      await get().load();
    } catch (e) {
      set({ notice: { kind: "error", text: message(e) } });
    } finally {
      set({ busy: null });
    }
  },

  refresh: async (opts) => {
    const silent = opts?.silent ?? false;
    const before = get().billing.status === "ready" ? paidCount((get().billing as { data: BillingStatus }).data) : 0;
    if (!silent) set({ busy: "refresh", notice: null });
    try {
      const data = await billingService.refresh();
      set({ billing: { status: "ready", data } });
      const after = paidCount(data);
      if (after > before) {
        set({ awaitingPurchase: false, notice: { kind: "success", text: "Your plan is active. Thank you for supporting EnZIME!" } });
        track({ name: "purchase_refresh", params: { result: "granted" } });
      } else if (!silent) {
        set({ notice: { kind: "info", text: after > 0 ? "Your plan is up to date." : "No active purchase found yet." } });
        track({ name: "purchase_refresh", params: { result: after > 0 ? "granted" : "none" } });
      }
    } catch (e) {
      if (!silent) {
        set({ notice: { kind: "error", text: message(e) } });
        track({ name: "purchase_refresh", params: { result: "error" } });
      }
    } finally {
      if (!silent) set({ busy: null });
    }
  },

  restore: async (licenseId) => {
    set({ busy: "restore", notice: null });
    try {
      const data = await billingService.restore(licenseId);
      set({ billing: { status: "ready", data }, notice: { kind: "success", text: "Purchase restored on this device." } });
      track({ name: "restore_purchase", params: { result: "granted" } });
      return true;
    } catch (e) {
      const none = e instanceof BillingError && e.code === "not_found";
      set({ notice: { kind: "error", text: message(e) } });
      track({ name: "restore_purchase", params: { result: none ? "none" : "error" } });
      return false;
    } finally {
      set({ busy: null });
    }
  },

  portal: async () => {
    set({ busy: "portal", notice: null });
    track({ name: "open_billing_portal" });
    try {
      await billingService.portal();
    } catch (e) {
      set({ notice: { kind: "error", text: message(e) } });
    } finally {
      set({ busy: null });
    }
  },

  stopAwaiting: () => set({ awaitingPurchase: false }),
  clearNotice: () => set({ notice: null }),
}));

/** Capability gate for feature code: `useCapability("annotations.rich_media")`. */
export function useCapability(key: string): CapabilityDecision | undefined {
  return useEntitlementStore((s) =>
    s.billing.status === "ready" ? s.billing.data.decisions.find((d) => d.capabilityKey === key) : undefined,
  );
}
