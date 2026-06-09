import type { AccumulationQuantity } from "./generated/AccumulationQuantity.ts";
import type { AccumulationTrack } from "./generated/AccumulationTrack.ts";
import type { BoostPickupEvent } from "./generated/BoostPickupEvent.ts";
import type { BoostStats } from "./generated/BoostStats.ts";
import type { RespawnEvent } from "./generated/RespawnEvent.ts";
import type {
  PlayerStatsSnapshot,
  StatsFrame,
  MaterializedStatsTimeline,
} from "./statsTimeline.ts";
import { statsEventPayloads } from "./statsTimeline.ts";

// Boost per-frame stat derivation for the event-only scaffold.
//
// The Rust core no longer ships ledger/state events; it ships discrete pickup/respawn events plus
// compressed per-frame accumulation tracks (boost amount + cumulative used / vertical-used /
// collected / stolen / overfill). This module rebuilds the per-frame `BoostStats` the scaffold
// frames omit:
//   - discrete fields (collected/stolen/overfill/respawned + pad counts) accumulate from the
//     pickup/respawn events;
//   - the continuous integral + time-in-band fields are reconstructed from the boost-amount track
//     using the same sample math as the Rust accumulator;
//   - cumulative used (+ grounded/air/supersonic) is read straight off the cumulative tracks.
// Track-derived fields stay zero until the accumulation tracks are wired through the replay worker
// (see replayLoader.ts), so the derivation degrades gracefully.

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

const TRACK_BOOST_FIELDS = [
  "amount_used",
  "amount_used_while_grounded",
  "amount_used_while_airborne",
  "amount_used_while_supersonic",
] as const;

const EVENT_BOOST_FIELDS = [
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
] as const;

const DERIVED_BOOST_FIELDS = [
  ...CONTINUOUS_BOOST_FIELDS,
  ...TRACK_BOOST_FIELDS,
  ...EVENT_BOOST_FIELDS,
] as const;

type DerivedBoostField = (typeof DERIVED_BOOST_FIELDS)[number];
type DerivedBoostStats = Pick<BoostStats, DerivedBoostField>;
type BoostMismatchScope = "team_zero" | "team_one" | "player";

export interface BoostTrackDerivationMismatch {
  frame: number;
  time: number;
  scope: BoostMismatchScope;
  playerId?: string;
  field: DerivedBoostField;
  expected: number;
  actual: number;
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

function boostPercentToAmount(boostPercent: number): number {
  return f32(mulF32(divF32(boostPercent, 100), BOOST_MAX_AMOUNT));
}

function createDerivedBoostStats(): DerivedBoostStats {
  const stats = {} as DerivedBoostStats;
  for (const field of DERIVED_BOOST_FIELDS) {
    stats[field] = 0;
  }
  return stats;
}

function remoteIdKey(playerId: Record<string, unknown>): string {
  const [kind, value] = Object.entries(playerId)[0] ?? ["Unknown", "unknown"];
  return `${kind}:${typeof value === "string" ? value : JSON.stringify(value)}`;
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

function addContinuousBoostSample(
  stats: DerivedBoostStats,
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
      intervalFractionInBoostRange(previous, current, BOOST_FULL_BAND_MIN_RAW, BOOST_MAX_AMOUNT + 1),
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
      intervalFractionInBoostRange(previous, current, boostPercentToAmount(25), boostPercentToAmount(50)),
    ),
  );
  stats.time_boost_50_75 = addF32(
    stats.time_boost_50_75,
    mulF32(
      sampleDt,
      intervalFractionInBoostRange(previous, current, boostPercentToAmount(50), boostPercentToAmount(75)),
    ),
  );
  stats.time_boost_75_100 = addF32(
    stats.time_boost_75_100,
    mulF32(
      sampleDt,
      intervalFractionInBoostRange(previous, current, boostPercentToAmount(75), BOOST_MAX_AMOUNT + 1),
    ),
  );
}

interface BoostAccumulator {
  stats: DerivedBoostStats;
  previousBoostAmount: number | null;
  isTeamZero: boolean;
}

function createBoostAccumulator(): BoostAccumulator {
  return { stats: createDerivedBoostStats(), previousBoostAmount: null, isTeamZero: true };
}

function applyPickupEvent(accumulator: BoostAccumulator, event: BoostPickupEvent): void {
  const collected = f32(event.collected_amount);
  const overfill = f32(event.overfill_amount);
  const stats = accumulator.stats;
  const big = event.pad_type === "big";
  const small = event.pad_type === "small";

  if (event.activity === "inactive") {
    stats.amount_collected_inactive = addF32(stats.amount_collected_inactive, collected);
    if (big) {
      stats.big_pads_collected_inactive += 1;
    } else if (small) {
      stats.small_pads_collected_inactive += 1;
    }
  } else {
    stats.amount_collected = addF32(stats.amount_collected, collected);
    if (big) {
      stats.amount_collected_big = addF32(stats.amount_collected_big, collected);
      stats.big_pads_collected += 1;
    } else if (small) {
      stats.amount_collected_small = addF32(stats.amount_collected_small, collected);
      stats.small_pads_collected += 1;
    }
  }

  if (event.is_steal) {
    stats.amount_stolen = addF32(stats.amount_stolen, collected);
    if (big) {
      stats.big_pads_stolen += 1;
      stats.amount_stolen_big = addF32(stats.amount_stolen_big, collected);
    } else if (small) {
      stats.small_pads_stolen += 1;
      stats.amount_stolen_small = addF32(stats.amount_stolen_small, collected);
    }
  }

  if (overfill > 0) {
    stats.overfill_total = addF32(stats.overfill_total, overfill);
    if (event.field_half === "opponent") {
      stats.overfill_from_stolen = addF32(stats.overfill_from_stolen, overfill);
    }
  }
}

function applyRespawnEvent(accumulator: BoostAccumulator, event: RespawnEvent): void {
  if (event.boost_granted == null) {
    return;
  }
  accumulator.stats.amount_respawned = addF32(
    accumulator.stats.amount_respawned,
    f32(event.boost_granted),
  );
}

/// A per-player cursor over one accumulation track's change-points; samples the held value at a frame.
class TrackCursor {
  private index = 0;

  constructor(private readonly points: AccumulationTrack["points"]) {}

  sample(frameNumber: number): number {
    while (
      this.index + 1 < this.points.length &&
      this.points[this.index + 1]!.frame <= frameNumber
    ) {
      this.index += 1;
    }
    const point = this.points[this.index];
    if (!point || point.frame > frameNumber) {
      return 0;
    }
    return f32(point.value);
  }
}

type TrackKey = `${string}:${AccumulationQuantity}`;

function trackKey(playerKey: string, quantity: AccumulationQuantity): TrackKey {
  return `${playerKey}:${quantity}`;
}

function buildTrackCursors(timeline: MaterializedStatsTimeline): Map<TrackKey, TrackCursor> {
  const cursors = new Map<TrackKey, TrackCursor>();
  const tracks = (timeline as { accumulation_tracks?: AccumulationTrack[] }).accumulation_tracks;
  for (const track of tracks ?? []) {
    const playerKey = remoteIdKey(track.player_id as unknown as Record<string, unknown>);
    cursors.set(trackKey(playerKey, track.quantity), new TrackCursor(track.points));
  }
  return cursors;
}

function sortedPickupEvents(timeline: MaterializedStatsTimeline): BoostPickupEvent[] {
  return [...statsEventPayloads(timeline, "boost_pickup")].sort(
    (left, right) => left.frame - right.frame || left.time - right.time,
  );
}

function sortedRespawnEvents(timeline: MaterializedStatsTimeline): RespawnEvent[] {
  return [...statsEventPayloads(timeline, "respawn")].sort(
    (left, right) => left.frame - right.frame || left.time - right.time,
  );
}

function copyDerivedBoostStats(target: BoostStats, source: DerivedBoostStats): void {
  for (const field of DERIVED_BOOST_FIELDS) {
    target[field] = source[field];
  }
}

interface BoostFrameAccumulator {
  applyFrame(frame: StatsFrame): void;
}

function createAccumulatorState(timeline: MaterializedStatsTimeline): {
  applyFrame(frame: StatsFrame, onPlayer?: (key: string, isTeamZero: boolean, stats: DerivedBoostStats) => void): void;
} {
  const pickupEvents = sortedPickupEvents(timeline);
  const respawnEvents = sortedRespawnEvents(timeline);
  const cursors = buildTrackCursors(timeline);
  let pickupIndex = 0;
  let respawnIndex = 0;
  const players = new Map<string, BoostAccumulator>();
  const teamZero = createBoostAccumulator();
  const teamOne = createBoostAccumulator();

  const playerFor = (playerId: Record<string, unknown>, isTeamZero: boolean): BoostAccumulator => {
    const key = remoteIdKey(playerId);
    let player = players.get(key);
    if (!player) {
      player = createBoostAccumulator();
      players.set(key, player);
    }
    player.isTeamZero = isTeamZero;
    return player;
  };

  const applyCumulativeTracks = (
    stats: DerivedBoostStats,
    playerKey: string,
    frameNumber: number,
  ): void => {
    const read = (quantity: AccumulationQuantity): number =>
      cursors.get(trackKey(playerKey, quantity))?.sample(frameNumber) ?? 0;
    stats.amount_used = read("boost_used");
    stats.amount_used_while_grounded = read("boost_used_grounded");
    stats.amount_used_while_airborne = read("boost_used_airborne");
    stats.amount_used_while_supersonic = read("boost_used_supersonic");
  };

  return {
    applyFrame(frame, onPlayer): void {
      while (pickupIndex < pickupEvents.length && pickupEvents[pickupIndex]!.frame <= frame.frame_number) {
        const event = pickupEvents[pickupIndex]!;
        applyPickupEvent(playerFor(event.player_id as Record<string, unknown>, event.is_team_0), event);
        applyPickupEvent(event.is_team_0 ? teamZero : teamOne, event);
        pickupIndex += 1;
      }
      while (respawnIndex < respawnEvents.length && respawnEvents[respawnIndex]!.frame <= frame.frame_number) {
        const event = respawnEvents[respawnIndex]!;
        applyRespawnEvent(playerFor(event.player_id as Record<string, unknown>, event.is_team_0), event);
        applyRespawnEvent(event.is_team_0 ? teamZero : teamOne, event);
        respawnIndex += 1;
      }

      // Continuous integral / time-in-band from the boost-amount track, sampled on live frames.
      if (frame.is_live_play) {
        for (const player of frame.players) {
          const key = remoteIdKey(player.player_id as Record<string, unknown>);
          const cursor = cursors.get(trackKey(key, "boost_amount"));
          if (!cursor) {
            continue;
          }
          const boostAmount = cursor.sample(frame.frame_number);
          const accumulator = players.get(key) ?? playerFor(player.player_id as Record<string, unknown>, player.is_team_0);
          const previous = accumulator.previousBoostAmount ?? boostAmount;
          addContinuousBoostSample(accumulator.stats, previous, boostAmount, frame.dt);
          addContinuousBoostSample(
            player.is_team_0 ? teamZero.stats : teamOne.stats,
            previous,
            boostAmount,
            frame.dt,
          );
          accumulator.previousBoostAmount = boostAmount;
        }
      }

      // Cumulative used (+ vertical breakdown) is read per player off the tracks; team totals are
      // the sum of their players (there are no team-level tracks).
      for (const field of TRACK_BOOST_FIELDS) {
        teamZero.stats[field] = 0;
        teamOne.stats[field] = 0;
      }
      for (const [key, accumulator] of players) {
        applyCumulativeTracks(accumulator.stats, key, frame.frame_number);
        const team = accumulator.isTeamZero ? teamZero : teamOne;
        for (const field of TRACK_BOOST_FIELDS) {
          team.stats[field] = addF32(team.stats[field], accumulator.stats[field]);
        }
      }
      for (const player of frame.players) {
        const key = remoteIdKey(player.player_id as Record<string, unknown>);
        onPlayer?.(key, player.is_team_0, players.get(key)?.stats ?? createDerivedBoostStats());
      }

      copyDerivedBoostStats(frame.team_zero.boost, teamZero.stats);
      copyDerivedBoostStats(frame.team_one.boost, teamOne.stats);
      for (const player of frame.players) {
        const key = remoteIdKey(player.player_id as Record<string, unknown>);
        copyDerivedBoostStats(player.boost, players.get(key)?.stats ?? createDerivedBoostStats());
      }
    },
  };
}

export function applyBoostTrackDerivedStats(
  timeline: MaterializedStatsTimeline,
): MaterializedStatsTimeline {
  const accumulator = createAccumulatorState(timeline);
  for (const frame of timeline.frames) {
    accumulator.applyFrame(frame);
  }
  return timeline;
}

export function createBoostTrackDerivedStatsAccumulator(
  timeline: MaterializedStatsTimeline,
): BoostFrameAccumulator {
  const accumulator = createAccumulatorState(timeline);
  return {
    applyFrame(frame: StatsFrame): void {
      accumulator.applyFrame(frame);
    },
  };
}

export function findBoostTrackDerivationMismatches(
  timeline: MaterializedStatsTimeline,
): BoostTrackDerivationMismatch[] {
  const accumulator = createAccumulatorState(timeline);
  const mismatches: BoostTrackDerivationMismatch[] = [];

  const check = (
    frame: StatsFrame,
    scope: BoostMismatchScope,
    actual: BoostStats,
    expected: DerivedBoostStats,
    player?: PlayerStatsSnapshot,
  ): void => {
    for (const field of DERIVED_BOOST_FIELDS) {
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
  };

  for (const frame of timeline.frames) {
    // Snapshot the serialized stats before applyFrame overwrites them with the derived values.
    const original = new Map<string, BoostStats>();
    for (const player of frame.players) {
      original.set(remoteIdKey(player.player_id as Record<string, unknown>), { ...player.boost });
    }
    const expectedByPlayer = new Map<string, DerivedBoostStats>();
    accumulator.applyFrame(frame, (key, _isTeamZero, stats) => {
      expectedByPlayer.set(key, { ...stats });
    });
    for (const player of frame.players) {
      const key = remoteIdKey(player.player_id as Record<string, unknown>);
      const expected = expectedByPlayer.get(key);
      const serialized = original.get(key);
      if (expected && serialized) {
        check(frame, "player", serialized, expected, player);
      }
    }
  }

  return mismatches;
}
