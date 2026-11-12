import { describe, expect, it } from "vitest";
import { downloadBudgetBounds, MIN_BUDGET_BYTES, payloadCapacity } from "./downloadBudget";
import { budgetBytesFor } from "./DownloadPlanDialog";

describe("real storage budget bounds", () => {
  it("reserves metadata and workspace before calculating payload", () => {
    expect(payloadCapacity(1024 ** 3)).toBe(704 * 1024 ** 2);
    expect(payloadCapacity(16 * 1024 ** 2)).toBe(10 * 1024 ** 2);
    expect(payloadCapacity(NaN)).toBe(0);
    expect(payloadCapacity(-1)).toBe(0);
    expect(payloadCapacity(1.5)).toBe(0);
  });

  it("caps small archives instead of reserving a default GiB", () => {
    expect(downloadBudgetBounds(10 * 1024 ** 3, 1_800_000)).toEqual({ min: MIN_BUDGET_BYTES, max: MIN_BUDGET_BYTES });
    const size = 100 * 1024 ** 2;
    const bounds = downloadBudgetBounds(10 * 1024 ** 3, size);
    expect(payloadCapacity(bounds.max)).toBeGreaterThanOrEqual(size);
    for (let n = bounds.max - 8; n < bounds.max; n++) expect(payloadCapacity(n)).toBeLessThan(size);
  });

  it("uses actual available space for large or unknown sources", () => {
    const available = 512 * 1024 ** 2;
    expect(downloadBudgetBounds(available, 100 * 1024 ** 3).max).toBe(available);
    expect(downloadBudgetBounds(available, 0).max).toBe(available);
    for (const free of [0, 1, MIN_BUDGET_BYTES - 1, NaN, Infinity, -1]) expect(downloadBudgetBounds(free, 123).max).toBe(0);
  });

  it("keeps fractional MiB/GiB choices in whole bytes", () => {
    expect(budgetBytesFor("0.1", "GiB")).toBe(Math.floor(0.1 * 1024 ** 3));
    expect(budgetBytesFor("16.5", "MiB")).toBe(17_301_504);
    expect(budgetBytesFor("0.0000001", "MiB")).toBeUndefined();
  });

  it("finds minimal source caps across reservation boundaries", () => {
    for (const source of [10_485_761, 50_000_001, 335_544_319, 704 * 1024 ** 2 + 1, 3 * 1024 ** 3, 1_000_000_003]) {
      const max = downloadBudgetBounds(20 * 1024 ** 3, source).max;
      expect(payloadCapacity(max)).toBeGreaterThanOrEqual(source);
      for (let n = Math.max(MIN_BUDGET_BYTES, max - 8); n < max; n++) expect(payloadCapacity(n)).toBeLessThan(source);
    }
  });
});
