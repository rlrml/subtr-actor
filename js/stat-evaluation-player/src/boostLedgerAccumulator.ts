import type { BoostLedgerEvent } from "./generated/BoostLedgerEvent.ts";
import type { BoostStats } from "./generated/BoostStats.ts";
import { addF32, f32 } from "./boostLedgerFloat.ts";
import {
  EMPTY_LEDGER_BOOST_STATS,
  EVENT_DERIVED_BOOST_FIELDS,
  createLedgerBoostStats,
  type EventDerivedBoostStats,
} from "./boostLedgerStats.ts";

export interface LedgerAccumulator {
  stats: EventDerivedBoostStats;
  countedPickupKeys: Set<string>;
  currentBoostAmount: number | null;
  currentBoostBefore: number | null;
  currentBoostFrame: number | null;
  previousBoostAmount: number | null;
  labeledAmountsVersion: number;
  labeledAmountsSnapshot: EventDerivedBoostStats["labeled_amounts"];
  labeledAmountsSnapshotVersion: number;
  labeledCountsVersion: number;
  labeledCountsSnapshot: EventDerivedBoostStats["labeled_counts"];
  labeledCountsSnapshotVersion: number;
}

export function createLedgerAccumulator(): LedgerAccumulator {
  return {
    stats: createLedgerBoostStats(),
    countedPickupKeys: new Set(),
    currentBoostAmount: null,
    currentBoostBefore: null,
    currentBoostFrame: null,
    previousBoostAmount: null,
    labeledAmountsVersion: 0,
    labeledAmountsSnapshot: undefined,
    labeledAmountsSnapshotVersion: -1,
    labeledCountsVersion: 0,
    labeledCountsSnapshot: undefined,
    labeledCountsSnapshotVersion: -1,
  };
}

export function remoteIdKey(playerId: Record<string, unknown>): string {
  const [kind, value] = Object.entries(playerId)[0] ?? ["Unknown", "unknown"];
  return `${kind}:${typeof value === "string" ? value : JSON.stringify(value)}`;
}

function labelValue(event: BoostLedgerEvent, key: string): string | null {
  return event.labels?.find((label) => label.key === key)?.value ?? null;
}

function sortedLabels(labels: BoostLedgerEvent["labels"]): NonNullable<BoostLedgerEvent["labels"]> {
  return [...(labels ?? [])].sort((left, right) =>
    left.key === right.key
      ? left.value.localeCompare(right.value)
      : left.key.localeCompare(right.key),
  );
}

function labelSetKey(labels: BoostLedgerEvent["labels"]): string {
  return JSON.stringify(sortedLabels(labels));
}

function cloneLabels(labels: BoostLedgerEvent["labels"]): NonNullable<BoostLedgerEvent["labels"]> {
  return sortedLabels(labels).map((label) => ({ ...label }));
}

function addLabeledAmount(stats: EventDerivedBoostStats, event: BoostLedgerEvent): boolean {
  const amount = f32(event.amount);
  if (amount <= 0) {
    return false;
  }
  const entries = (stats.labeled_amounts ??= { entries: [] }).entries;
  const key = labelSetKey(event.labels);
  const existing = entries.find((entry) => labelSetKey(entry.labels) === key);
  if (existing) {
    existing.value = addF32(existing.value, amount);
    return true;
  }
  entries.push({ labels: cloneLabels(event.labels), value: amount });
  entries.sort((left, right) =>
    JSON.stringify(left.labels).localeCompare(JSON.stringify(right.labels)),
  );
  return true;
}

function addLabeledCount(
  stats: EventDerivedBoostStats,
  event: BoostLedgerEvent,
  count: number,
): boolean {
  if (count <= 0) {
    return false;
  }
  const entries = (stats.labeled_counts ??= { entries: [] }).entries;
  const key = labelSetKey(event.labels);
  const existing = entries.find((entry) => labelSetKey(entry.labels) === key);
  if (existing) {
    existing.count += count;
    return true;
  }
  entries.push({ labels: cloneLabels(event.labels), count });
  entries.sort((left, right) =>
    JSON.stringify(left.labels).localeCompare(JSON.stringify(right.labels)),
  );
  return true;
}

function countPickupOnce(accumulator: LedgerAccumulator, event: BoostLedgerEvent): void {
  if (event.count <= 0) {
    return;
  }

  const padSize = labelValue(event, "pad_size");
  if (padSize !== "big" && padSize !== "small") {
    return;
  }

  const activity = labelValue(event, "activity") ?? "unknown";
  const fieldHalf = labelValue(event, "field_half") ?? "unknown";
  const pickupKey = `${event.frame}:${remoteIdKey(event.player_id as Record<string, unknown>)}:${padSize}:${activity}:${fieldHalf}`;
  if (accumulator.countedPickupKeys.has(pickupKey)) {
    return;
  }
  accumulator.countedPickupKeys.add(pickupKey);

  if (activity === "inactive") {
    if (padSize === "big") {
      accumulator.stats.big_pads_collected_inactive += 1;
    } else {
      accumulator.stats.small_pads_collected_inactive += 1;
    }
    return;
  }

  if (padSize === "big") {
    accumulator.stats.big_pads_collected += 1;
  } else {
    accumulator.stats.small_pads_collected += 1;
  }
}

export function applyLedgerEvent(accumulator: LedgerAccumulator, event: BoostLedgerEvent): void {
  const amount = f32(Number.isFinite(event.amount) ? event.amount : 0);
  if (event.transaction !== "used") {
    if (addLabeledAmount(accumulator.stats, event)) {
      accumulator.labeledAmountsVersion += 1;
    }
  }
  if (event.transaction === "collected") {
    if (addLabeledCount(accumulator.stats, event, Math.max(event.count, 1))) {
      accumulator.labeledCountsVersion += 1;
    }
  }
  const padSize = labelValue(event, "pad_size");
  const activity = labelValue(event, "activity") ?? "active";
  const fieldHalf = labelValue(event, "field_half");

  switch (event.transaction) {
    case "collected":
      countPickupOnce(accumulator, event);
      if (activity === "inactive") {
        accumulator.stats.amount_collected_inactive = addF32(
          accumulator.stats.amount_collected_inactive,
          amount,
        );
        break;
      }
      accumulator.stats.amount_collected = addF32(accumulator.stats.amount_collected, amount);
      if (padSize === "big") {
        accumulator.stats.amount_collected_big = addF32(
          accumulator.stats.amount_collected_big,
          amount,
        );
      } else if (padSize === "small") {
        accumulator.stats.amount_collected_small = addF32(
          accumulator.stats.amount_collected_small,
          amount,
        );
      }
      break;

    case "stolen":
      accumulator.stats.amount_stolen = addF32(accumulator.stats.amount_stolen, amount);
      if (padSize === "big") {
        accumulator.stats.big_pads_stolen += 1;
        accumulator.stats.amount_stolen_big = addF32(accumulator.stats.amount_stolen_big, amount);
      } else if (padSize === "small") {
        accumulator.stats.small_pads_stolen += 1;
        accumulator.stats.amount_stolen_small = addF32(
          accumulator.stats.amount_stolen_small,
          amount,
        );
      }
      break;

    case "overfill":
      accumulator.stats.overfill_total = addF32(accumulator.stats.overfill_total, amount);
      if (fieldHalf === "opponent") {
        accumulator.stats.overfill_from_stolen = addF32(
          accumulator.stats.overfill_from_stolen,
          amount,
        );
      }
      countPickupOnce(accumulator, event);
      break;

    case "respawn":
      accumulator.stats.amount_respawned = addF32(accumulator.stats.amount_respawned, amount);
      break;

    case "used":
      accumulator.stats.amount_used = addF32(accumulator.stats.amount_used, amount);
      break;

    case "used_allocation":
      if (labelValue(event, "vertical_state") === "grounded") {
        accumulator.stats.amount_used_while_grounded = addF32(
          accumulator.stats.amount_used_while_grounded,
          amount,
        );
      } else if (labelValue(event, "vertical_state") === "aerial") {
        accumulator.stats.amount_used_while_airborne = addF32(
          accumulator.stats.amount_used_while_airborne,
          amount,
        );
      }
      if (labelValue(event, "supersonic") === "true") {
        accumulator.stats.amount_used_while_supersonic = addF32(
          accumulator.stats.amount_used_while_supersonic,
          amount,
        );
      }
      break;
  }
}

function getLabeledAmountsSnapshot(
  accumulator: LedgerAccumulator,
): EventDerivedBoostStats["labeled_amounts"] {
  if (accumulator.labeledAmountsSnapshotVersion !== accumulator.labeledAmountsVersion) {
    accumulator.labeledAmountsSnapshot =
      accumulator.stats.labeled_amounts && accumulator.stats.labeled_amounts.entries.length > 0
        ? {
            entries: accumulator.stats.labeled_amounts.entries.map((entry) => ({
              labels: entry.labels.map((label) => ({ ...label })),
              value: entry.value,
            })),
          }
        : undefined;
    accumulator.labeledAmountsSnapshotVersion = accumulator.labeledAmountsVersion;
  }
  return accumulator.labeledAmountsSnapshot;
}

function getLabeledCountsSnapshot(
  accumulator: LedgerAccumulator,
): EventDerivedBoostStats["labeled_counts"] {
  if (accumulator.labeledCountsSnapshotVersion !== accumulator.labeledCountsVersion) {
    accumulator.labeledCountsSnapshot =
      accumulator.stats.labeled_counts && accumulator.stats.labeled_counts.entries.length > 0
        ? {
            entries: accumulator.stats.labeled_counts.entries.map((entry) => ({
              labels: entry.labels.map((label) => ({ ...label })),
              count: entry.count,
            })),
          }
        : undefined;
    accumulator.labeledCountsSnapshotVersion = accumulator.labeledCountsVersion;
  }
  return accumulator.labeledCountsSnapshot;
}

export function copyLedgerDerivedBoostStats(
  target: BoostStats,
  accumulator: LedgerAccumulator | undefined,
): void {
  const source = accumulator?.stats ?? EMPTY_LEDGER_BOOST_STATS;
  for (const field of EVENT_DERIVED_BOOST_FIELDS) {
    target[field] = source[field];
  }
  const labeledAmounts = accumulator ? getLabeledAmountsSnapshot(accumulator) : undefined;
  if (labeledAmounts) {
    target.labeled_amounts = labeledAmounts;
  } else {
    delete target.labeled_amounts;
  }
  const labeledCounts = accumulator ? getLabeledCountsSnapshot(accumulator) : undefined;
  if (labeledCounts) {
    target.labeled_counts = labeledCounts;
  } else {
    delete target.labeled_counts;
  }
}
