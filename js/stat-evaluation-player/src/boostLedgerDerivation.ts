import type { BoostLedgerEvent } from "./generated/BoostLedgerEvent.ts";
import type { BoostStateEvent } from "./generated/BoostStateEvent.ts";
import type { BoostStats } from "./generated/BoostStats.ts";
import type {
  PlayerStatsSnapshot,
  StatsFrame,
  MaterializedStatsTimeline,
} from "./statsTimeline.ts";

const FLOAT_TOLERANCE = 0.001;
const BOOST_MAX_AMOUNT = 255;
const BOOST_ZERO_BAND_RAW = 1;
const BOOST_FULL_BAND_MIN_RAW = BOOST_MAX_AMOUNT - 1;
const F32_EPSILON = 1.1920928955078125e-7;

const CONTINUOUS_BOOST_FIELDS = [
  "tracked_time",
  "boost_integral",
  "time_zero_boost",
  "time_hundred_boost",
  "time_boost_0_25",
  "time_boost_25_50",
  "time_boost_50_75",
  "time_boost_75_100",
] as const;

const LEDGER_BOOST_FIELDS = [
  "amount_collected",
  "amount_collected_inactive",
  "big_pads_collected_inactive",
  "small_pads_collected_inactive",
  "amount_stolen",
  "big_pads_collected",
  "small_pads_collected",
  "big_pads_stolen",
  "small_pads_stolen",
  "amount_collected_big",
  "amount_stolen_big",
  "amount_collected_small",
  "amount_stolen_small",
  "amount_respawned",
  "overfill_total",
  "overfill_from_stolen",
  "amount_used",
  "amount_used_while_grounded",
  "amount_used_while_airborne",
  "amount_used_while_supersonic",
] as const;

const EVENT_DERIVED_BOOST_FIELDS = [...CONTINUOUS_BOOST_FIELDS, ...LEDGER_BOOST_FIELDS] as const;

type EventDerivedBoostField = (typeof EVENT_DERIVED_BOOST_FIELDS)[number];
type EventDerivedBoostStats = Pick<BoostStats, EventDerivedBoostField> &
  Pick<BoostStats, "labeled_amounts" | "labeled_counts">;
type BoostLedgerMismatchScope = "team_zero" | "team_one" | "player";

export interface BoostLedgerDerivationMismatch {
  frame: number;
  time: number;
  scope: BoostLedgerMismatchScope;
  playerId?: string;
  field: EventDerivedBoostField;
  expected: number;
  actual: number;
}

interface LedgerAccumulator {
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

function f32(value: number): number {
  return Math.fround(value);
}

function addF32(left: number, right: number): number {
  return f32(f32(left) + f32(right));
}

function subF32(left: number, right: number): number {
  return f32(f32(left) - f32(right));
}

function mulF32(left: number, right: number): number {
  return f32(f32(left) * f32(right));
}

function divF32(left: number, right: number): number {
  return f32(f32(left) / f32(right));
}

function createLedgerBoostStats(): EventDerivedBoostStats {
  return {
    tracked_time: 0,
    boost_integral: 0,
    time_zero_boost: 0,
    time_hundred_boost: 0,
    time_boost_0_25: 0,
    time_boost_25_50: 0,
    time_boost_50_75: 0,
    time_boost_75_100: 0,
    amount_collected: 0,
    amount_collected_inactive: 0,
    big_pads_collected_inactive: 0,
    small_pads_collected_inactive: 0,
    amount_stolen: 0,
    big_pads_collected: 0,
    small_pads_collected: 0,
    big_pads_stolen: 0,
    small_pads_stolen: 0,
    amount_collected_big: 0,
    amount_stolen_big: 0,
    amount_collected_small: 0,
    amount_stolen_small: 0,
    amount_respawned: 0,
    overfill_total: 0,
    overfill_from_stolen: 0,
    amount_used: 0,
    amount_used_while_grounded: 0,
    amount_used_while_airborne: 0,
    amount_used_while_supersonic: 0,
  };
}

const EMPTY_LEDGER_BOOST_STATS = createLedgerBoostStats();

function createLedgerAccumulator(): LedgerAccumulator {
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

function remoteIdKey(playerId: Record<string, unknown>): string {
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

function boostPercentToAmount(boostPercent: number): number {
  return divF32(mulF32(boostPercent, BOOST_MAX_AMOUNT), 100);
}

function intervalFractionInBoostRange(
  startBoost: number,
  endBoost: number,
  minBoost: number,
  maxBoost: number,
): number {
  const boostDelta = subF32(endBoost, startBoost);
  if (Math.abs(boostDelta) <= F32_EPSILON) {
    return startBoost >= minBoost && startBoost < maxBoost ? 1 : 0;
  }

  const tAtMin = divF32(subF32(minBoost, startBoost), boostDelta);
  const tAtMax = divF32(subF32(maxBoost, startBoost), boostDelta);
  const intervalStart = Math.max(Math.min(tAtMin, tAtMax), 0);
  const intervalEnd = Math.min(Math.max(tAtMin, tAtMax), 1);
  return Math.max(subF32(intervalEnd, intervalStart), 0);
}

function applyBoostStateEvent(accumulator: LedgerAccumulator, event: BoostStateEvent): void {
  accumulator.currentBoostAmount = f32(event.boost_amount);
  accumulator.currentBoostBefore = event.boost_before == null ? null : f32(event.boost_before);
  accumulator.currentBoostFrame = event.frame;
}

function addContinuousBoostSample(
  stats: EventDerivedBoostStats,
  previousBoostAmount: number,
  boostAmount: number,
  dt: number,
): void {
  const previous = f32(previousBoostAmount);
  const current = f32(boostAmount);
  const sampleDt = f32(dt);
  const averageBoostAmount = mulF32(addF32(previous, current), 0.5);

  stats.tracked_time = addF32(stats.tracked_time, sampleDt);
  stats.boost_integral = addF32(stats.boost_integral, mulF32(averageBoostAmount, sampleDt));
  stats.time_zero_boost = addF32(
    stats.time_zero_boost,
    mulF32(sampleDt, intervalFractionInBoostRange(previous, current, 0, BOOST_ZERO_BAND_RAW)),
  );
  stats.time_hundred_boost = addF32(
    stats.time_hundred_boost,
    mulF32(
      sampleDt,
      intervalFractionInBoostRange(
        previous,
        current,
        BOOST_FULL_BAND_MIN_RAW,
        BOOST_MAX_AMOUNT + 1,
      ),
    ),
  );
  stats.time_boost_0_25 = addF32(
    stats.time_boost_0_25,
    mulF32(sampleDt, intervalFractionInBoostRange(previous, current, 0, boostPercentToAmount(25))),
  );
  stats.time_boost_25_50 = addF32(
    stats.time_boost_25_50,
    mulF32(
      sampleDt,
      intervalFractionInBoostRange(
        previous,
        current,
        boostPercentToAmount(25),
        boostPercentToAmount(50),
      ),
    ),
  );
  stats.time_boost_50_75 = addF32(
    stats.time_boost_50_75,
    mulF32(
      sampleDt,
      intervalFractionInBoostRange(
        previous,
        current,
        boostPercentToAmount(50),
        boostPercentToAmount(75),
      ),
    ),
  );
  stats.time_boost_75_100 = addF32(
    stats.time_boost_75_100,
    mulF32(
      sampleDt,
      intervalFractionInBoostRange(
        previous,
        current,
        boostPercentToAmount(75),
        BOOST_MAX_AMOUNT + 1,
      ),
    ),
  );
}

function applyContinuousBoostSample(
  accumulator: LedgerAccumulator,
  dt: number,
  frameNumber: number,
): [number, number] | null {
  if (accumulator.currentBoostFrame !== frameNumber) {
    return null;
  }
  const boostAmount = accumulator.currentBoostAmount;
  if (boostAmount == null) {
    return null;
  }
  const previousBoostAmount = accumulator.currentBoostBefore ?? boostAmount;
  addContinuousBoostSample(accumulator.stats, previousBoostAmount, boostAmount, dt);
  accumulator.previousBoostAmount = boostAmount;
  return [previousBoostAmount, boostAmount];
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

function applyLedgerEvent(accumulator: LedgerAccumulator, event: BoostLedgerEvent): void {
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

function copyLedgerDerivedBoostStats(
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

function compareLedgerDerivedBoostStats(
  mismatches: BoostLedgerDerivationMismatch[],
  frame: MaterializedStatsTimeline["frames"][number],
  scope: BoostLedgerMismatchScope,
  actual: BoostStats,
  expected: EventDerivedBoostStats,
  player?: PlayerStatsSnapshot,
): void {
  for (const field of EVENT_DERIVED_BOOST_FIELDS) {
    const actualValue = actual[field];
    const expectedValue = expected[field];
    if (Math.abs(actualValue - expectedValue) <= FLOAT_TOLERANCE) {
      continue;
    }

    mismatches.push({
      frame: frame.frame_number,
      time: frame.time,
      scope,
      playerId: player ? remoteIdKey(player.player_id as Record<string, unknown>) : undefined,
      field,
      expected: expectedValue,
      actual: actualValue,
    });
  }
}

function sortedBoostLedgerEvents(timeline: MaterializedStatsTimeline): BoostLedgerEvent[] {
  return [...(timeline.events.boost_ledger ?? [])].sort((left, right) => {
    if (left.frame !== right.frame) {
      return left.frame - right.frame;
    }
    if (left.time !== right.time) {
      return left.time - right.time;
    }
    return remoteIdKey(left.player_id as Record<string, unknown>).localeCompare(
      remoteIdKey(right.player_id as Record<string, unknown>),
    );
  });
}

function sortedBoostStateEvents(timeline: MaterializedStatsTimeline): BoostStateEvent[] {
  return [...(timeline.events.boost_state ?? [])].sort((left, right) => {
    if (left.frame !== right.frame) {
      return left.frame - right.frame;
    }
    if (left.time !== right.time) {
      return left.time - right.time;
    }
    return remoteIdKey(left.player_id as Record<string, unknown>).localeCompare(
      remoteIdKey(right.player_id as Record<string, unknown>),
    );
  });
}

export function applyBoostLedgerDerivedStats(
  timeline: MaterializedStatsTimeline,
): MaterializedStatsTimeline {
  const accumulator = createBoostLedgerDerivedStatsAccumulator(timeline);

  for (const frame of timeline.frames) {
    accumulator.applyFrame(frame);
  }

  return timeline;
}

export function createBoostLedgerDerivedStatsAccumulator(timeline: MaterializedStatsTimeline): {
  applyFrame(frame: StatsFrame): void;
} {
  const ledgerEvents = sortedBoostLedgerEvents(timeline);
  const stateEvents = sortedBoostStateEvents(timeline);
  let ledgerEventIndex = 0;
  let stateEventIndex = 0;
  const players = new Map<string, LedgerAccumulator>();
  const teamZero = createLedgerAccumulator();
  const teamOne = createLedgerAccumulator();

  return {
    applyFrame(frame: StatsFrame): void {
      const stateEventPlayersThisFrame: Array<{ key: string; isTeamZero: boolean }> = [];
      while (
        stateEventIndex < stateEvents.length &&
        stateEvents[stateEventIndex]!.frame <= frame.frame_number
      ) {
        const event = stateEvents[stateEventIndex]!;
        const playerKey = remoteIdKey(event.player_id as Record<string, unknown>);
        let player = players.get(playerKey);
        if (!player) {
          player = createLedgerAccumulator();
          players.set(playerKey, player);
        }
        applyBoostStateEvent(player, event);
        if (event.frame === frame.frame_number) {
          stateEventPlayersThisFrame.push({ key: playerKey, isTeamZero: event.is_team_0 });
        }
        stateEventIndex += 1;
      }

      while (
        ledgerEventIndex < ledgerEvents.length &&
        ledgerEvents[ledgerEventIndex]!.frame <= frame.frame_number
      ) {
        const event = ledgerEvents[ledgerEventIndex]!;
        const playerKey = remoteIdKey(event.player_id as Record<string, unknown>);
        let player = players.get(playerKey);
        if (!player) {
          player = createLedgerAccumulator();
          players.set(playerKey, player);
        }
        applyLedgerEvent(player, event);
        applyLedgerEvent(event.is_team_0 ? teamZero : teamOne, event);
        ledgerEventIndex += 1;
      }

      for (const player of stateEventPlayersThisFrame) {
        const playerStats = players.get(player.key);
        if (!playerStats) {
          continue;
        }
        const continuousSample = applyContinuousBoostSample(
          playerStats,
          frame.dt,
          frame.frame_number,
        );
        if (continuousSample) {
          addContinuousBoostSample(
            player.isTeamZero ? teamZero.stats : teamOne.stats,
            continuousSample[0],
            continuousSample[1],
            frame.dt,
          );
        }
      }

      copyLedgerDerivedBoostStats(frame.team_zero.boost, teamZero);
      copyLedgerDerivedBoostStats(frame.team_one.boost, teamOne);
      for (const player of frame.players) {
        const playerStats = players.get(remoteIdKey(player.player_id as Record<string, unknown>));
        copyLedgerDerivedBoostStats(player.boost, playerStats);
      }
    },
  };
}

export function findBoostLedgerDerivationMismatches(
  timeline: MaterializedStatsTimeline,
): BoostLedgerDerivationMismatch[] {
  const ledgerEvents = sortedBoostLedgerEvents(timeline);
  const stateEvents = sortedBoostStateEvents(timeline);
  let ledgerEventIndex = 0;
  let stateEventIndex = 0;
  const players = new Map<string, LedgerAccumulator>();
  const teamZero = createLedgerAccumulator();
  const teamOne = createLedgerAccumulator();
  const mismatches: BoostLedgerDerivationMismatch[] = [];

  for (const frame of timeline.frames) {
    const stateEventPlayersThisFrame: Array<{ key: string; isTeamZero: boolean }> = [];
    while (
      stateEventIndex < stateEvents.length &&
      stateEvents[stateEventIndex]!.frame <= frame.frame_number
    ) {
      const event = stateEvents[stateEventIndex]!;
      const playerKey = remoteIdKey(event.player_id as Record<string, unknown>);
      let player = players.get(playerKey);
      if (!player) {
        player = createLedgerAccumulator();
        players.set(playerKey, player);
      }
      applyBoostStateEvent(player, event);
      if (event.frame === frame.frame_number) {
        stateEventPlayersThisFrame.push({ key: playerKey, isTeamZero: event.is_team_0 });
      }
      stateEventIndex += 1;
    }

    while (
      ledgerEventIndex < ledgerEvents.length &&
      ledgerEvents[ledgerEventIndex]!.frame <= frame.frame_number
    ) {
      const event = ledgerEvents[ledgerEventIndex]!;
      const playerKey = remoteIdKey(event.player_id as Record<string, unknown>);
      let player = players.get(playerKey);
      if (!player) {
        player = createLedgerAccumulator();
        players.set(playerKey, player);
      }
      applyLedgerEvent(player, event);
      applyLedgerEvent(event.is_team_0 ? teamZero : teamOne, event);
      ledgerEventIndex += 1;
    }

    for (const player of stateEventPlayersThisFrame) {
      const playerStats = players.get(player.key);
      if (!playerStats) {
        continue;
      }
      const continuousSample = applyContinuousBoostSample(
        playerStats,
        frame.dt,
        frame.frame_number,
      );
      if (continuousSample) {
        addContinuousBoostSample(
          player.isTeamZero ? teamZero.stats : teamOne.stats,
          continuousSample[0],
          continuousSample[1],
          frame.dt,
        );
      }
    }

    compareLedgerDerivedBoostStats(
      mismatches,
      frame,
      "team_zero",
      frame.team_zero.boost,
      teamZero.stats,
    );
    compareLedgerDerivedBoostStats(
      mismatches,
      frame,
      "team_one",
      frame.team_one.boost,
      teamOne.stats,
    );
    for (const player of frame.players) {
      const expected =
        players.get(remoteIdKey(player.player_id as Record<string, unknown>))?.stats ??
        createLedgerBoostStats();
      compareLedgerDerivedBoostStats(mismatches, frame, "player", player.boost, expected, player);
    }
  }

  return mismatches;
}
