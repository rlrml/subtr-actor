import type { BumpEvent } from "./generated/BumpEvent.ts";
import type { BumpPlayerStats } from "./generated/BumpPlayerStats.ts";
import type { BumpTeamStats } from "./generated/BumpTeamStats.ts";
import type { StatsFrame, MaterializedStatsTimeline } from "./statsTimeline.ts";

function remoteIdKey(playerId: unknown): string {
  if (!playerId || typeof playerId !== "object") {
    return String(playerId);
  }
  const [kind, value] = Object.entries(playerId as Record<string, unknown>)[0] ?? [
    "Unknown",
    "unknown",
  ];
  return `${kind}:${typeof value === "string" ? value : JSON.stringify(value)}`;
}

function defaultBumpPlayerStats(): BumpPlayerStats {
  return {
    bumps_inflicted: 0,
    bumps_taken: 0,
    team_bumps_inflicted: 0,
    team_bumps_taken: 0,
    last_bump_time: null,
    last_bump_frame: null,
    last_bump_strength: null,
    max_bump_strength: 0,
    cumulative_bump_strength: 0,
  };
}

function defaultBumpTeamStats(): BumpTeamStats {
  return {
    bumps_inflicted: 0,
    team_bumps_inflicted: 0,
  };
}

function sortBumpEvents(events: readonly BumpEvent[]): BumpEvent[] {
  return events
    .map((event, index) => ({ event, index }))
    .sort((left, right) => {
      if (left.event.frame !== right.event.frame) {
        return left.event.frame - right.event.frame;
      }
      if (left.event.time !== right.event.time) {
        return left.event.time - right.event.time;
      }
      return left.index - right.index;
    })
    .map(({ event }) => event);
}

function recordBumpInflicted(stats: BumpPlayerStats, event: BumpEvent): void {
  stats.bumps_inflicted += 1;
  if (event.is_team_bump) {
    stats.team_bumps_inflicted += 1;
  }
  stats.last_bump_time = event.time;
  stats.last_bump_frame = event.frame;
  stats.last_bump_strength = event.strength;
  stats.max_bump_strength = Math.max(stats.max_bump_strength, event.strength);
  stats.cumulative_bump_strength += event.strength;
}

function recordBumpTaken(stats: BumpPlayerStats, event: BumpEvent): void {
  stats.bumps_taken += 1;
  if (event.is_team_bump) {
    stats.team_bumps_taken += 1;
  }
}

function recordBumpTeamStats(stats: BumpTeamStats, event: BumpEvent): void {
  stats.bumps_inflicted += 1;
  if (event.is_team_bump) {
    stats.team_bumps_inflicted += 1;
  }
}

function assignBumpPlayerStats(
  target: BumpPlayerStats,
  source: BumpPlayerStats | undefined,
): void {
  Object.assign(target, source ?? defaultBumpPlayerStats());
}

function assignBumpTeamStats(target: BumpTeamStats, source: BumpTeamStats): void {
  Object.assign(target, source);
}

export function applyBumpEventDerivedStats(timeline: MaterializedStatsTimeline): MaterializedStatsTimeline {
  const accumulator = createBumpEventDerivedStatsAccumulator(timeline);

  for (const frame of timeline.frames) {
    accumulator.applyFrame(frame);
  }

  return timeline;
}

export function createBumpEventDerivedStatsAccumulator(timeline: MaterializedStatsTimeline): {
  applyFrame(frame: StatsFrame): void;
} {
  const events = sortBumpEvents(timeline.events.bump ?? []);

  let eventIndex = 0;
  const players = new Map<string, BumpPlayerStats>();
  const teamZero = defaultBumpTeamStats();
  const teamOne = defaultBumpTeamStats();

  return {
    applyFrame(frame: StatsFrame): void {
      while (eventIndex < events.length && events[eventIndex]!.frame <= frame.frame_number) {
        const event = events[eventIndex] as BumpEvent;
        const initiatorKey = remoteIdKey(event.initiator);
        const initiatorStats = players.get(initiatorKey) ?? defaultBumpPlayerStats();
        players.set(initiatorKey, initiatorStats);
        recordBumpInflicted(initiatorStats, event);

        const victimKey = remoteIdKey(event.victim);
        const victimStats = players.get(victimKey) ?? defaultBumpPlayerStats();
        players.set(victimKey, victimStats);
        recordBumpTaken(victimStats, event);

        recordBumpTeamStats(event.initiator_is_team_0 ? teamZero : teamOne, event);
        eventIndex += 1;
      }

      assignBumpTeamStats(frame.team_zero.bump, teamZero);
      assignBumpTeamStats(frame.team_one.bump, teamOne);
      for (const player of frame.players) {
        assignBumpPlayerStats(player.bump, players.get(remoteIdKey(player.player_id)));
      }
    },
  };
}
