import { boostAmountToPercent } from "@rlrml/player";

// The raw `0..=255` -> `0..=100` conversion lives in `@rlrml/player`
// (`boostAmountToPercent`) so every subtr-actor JS consumer rescales boost the
// same way. This module keeps the stat-player display conventions (the `"?"`
// missing-value sentinel and the respawn-inclusive bound) on top of it.
export { boostAmountToPercent as toBoostDisplayUnits } from "@rlrml/player";

export function formatBoostDisplayAmount(amount: number | null | undefined): string {
  if (amount == null) return "?";
  return boostAmountToPercent(amount).toFixed(0);
}

export function formatCollectedWithRespawnBound(
  amountCollected: number | null | undefined,
  amountRespawned: number | null | undefined,
): string {
  const collectedDisplay = formatBoostDisplayAmount(amountCollected);
  if (amountCollected == null || amountRespawned == null) {
    return collectedDisplay;
  }

  const respawnInclusiveDisplay = formatBoostDisplayAmount(amountCollected + amountRespawned);
  return `${collectedDisplay} (${respawnInclusiveDisplay})`;
}
