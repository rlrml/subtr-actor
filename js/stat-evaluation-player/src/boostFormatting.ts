const BOOST_RAW_MAX = 255;

export function toBoostDisplayUnits(amount: number): number {
  return (amount * 100) / BOOST_RAW_MAX;
}

export function formatBoostDisplayAmount(amount: number | null | undefined): string {
  if (amount == null) return "?";
  return toBoostDisplayUnits(amount).toFixed(0);
}

export function formatCollectedWithRespawnBound(
  amountCollected: number | null | undefined,
  amountRespawned: number | null | undefined,
): string {
  const collectedDisplay = formatBoostDisplayAmount(amountCollected);
  if (amountCollected == null || amountRespawned == null) {
    return collectedDisplay;
  }

  const respawnInclusiveDisplay = formatBoostDisplayAmount(
    amountCollected + amountRespawned,
  );
  return `${collectedDisplay} (${respawnInclusiveDisplay})`;
}
