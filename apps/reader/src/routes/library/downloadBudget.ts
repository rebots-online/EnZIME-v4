export const MIN_BUDGET_BYTES = 16 * 1024 ** 2;

export function payloadCapacity(budget: number): number {
  if (!Number.isSafeInteger(budget) || budget < 0) return 0;
  return budget - Math.min(64 * 1024 ** 2, Math.floor(budget / 8))
    - Math.min(512 * 1024 ** 2, Math.floor(budget / 4));
}

export function downloadBudgetBounds(available: number, sourceSize: number): { min: number; max: number } {
  if (!Number.isSafeInteger(available) || available < MIN_BUDGET_BYTES) return { min: MIN_BUDGET_BYTES, max: 0 };
  if (!Number.isSafeInteger(sourceSize) || sourceSize <= 0) {
    return { min: MIN_BUDGET_BYTES, max: available };
  }
  let low = MIN_BUDGET_BYTES, high = available;
  // Integer reservations can drop capacity by one byte at an eight-byte boundary.
  while (high >= Math.max(MIN_BUDGET_BYTES, available - 7) && payloadCapacity(high) < sourceSize) high--;
  if (high < Math.max(MIN_BUDGET_BYTES, available - 7)) return { min: MIN_BUDGET_BYTES, max: available };
  while (low < high) {
    const middle = low + Math.floor((high - low) / 2);
    if (payloadCapacity(middle) >= sourceSize) high = middle;
    else low = middle + 1;
  }
  for (let candidate = Math.max(MIN_BUDGET_BYTES, low - 7); candidate < low; candidate++) {
    if (payloadCapacity(candidate) >= sourceSize) return { min: MIN_BUDGET_BYTES, max: candidate };
  }
  return { min: MIN_BUDGET_BYTES, max: low };
}
