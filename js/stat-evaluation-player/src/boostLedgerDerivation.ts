import type { BoostLedgerEvent } from "./generated/BoostLedgerEvent.ts";
import type { BoostStateEvent } from "./generated/BoostStateEvent.ts";
import type { BoostStats } from "./generated/BoostStats.ts";
import {
  addContinuousBoostSample,
  applyBoostStateEvent,
  applyContinuousBoostSample,
} from "./boostLedgerContinuous.ts";
import {
  EVENT_DERIVED_BOOST_FIELDS,
  createLedgerBoostStats,
  type EventDerivedBoostStats,
  type EventDerivedBoostField,
} from "./boostLedgerStats.ts";
import {
  applyLedgerEvent,
  copyLedgerDerivedBoostStats,
  createLedgerAccumulator,
  remoteIdKey,
  type LedgerAccumulator,
} from "./boostLedgerAccumulator.ts";
import type {
  PlayerStatsSnapshot,
  StatsFrame,
  MaterializedStatsTimeline,
} from "./statsTimeline.ts";

const FLOAT_TOLERANCE = 0.001;

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
