import type { PositioningEvent } from "./generated/PositioningEvent.ts";
import type { PositioningStats } from "./generated/PositioningStats.ts";
import type { StatsFrame, MaterializedStatsTimeline } from "./statsTimeline.ts";

const FLOAT_POSITIONING_FIELDS = [
  "active_game_time",
  "tracked_time",
  "sum_distance_to_teammates",
  "sum_distance_to_ball",
  "sum_distance_to_ball_has_possession",
  "time_has_possession",
  "sum_distance_to_ball_no_possession",
  "time_no_possession",
  "time_demolished",
  "time_no_teammates",
  "time_most_back",
  "time_most_forward",
  "time_mid_role",
  "time_other_role",
  "time_defensive_third",
  "time_neutral_third",
  "time_offensive_third",
  "time_defensive_half",
  "time_offensive_half",
  "time_closest_to_ball",
  "time_farthest_from_ball",
  "time_behind_ball",
  "time_level_with_ball",
  "time_in_front_of_ball",
] as const;

function addF32(left: number, right: number): number {
  return Math.fround(Math.fround(left) + Math.fround(right));
}

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

function defaultPositioningStats(): PositioningStats {
  return {
    active_game_time: 0,
    tracked_time: 0,
    sum_distance_to_teammates: 0,
    sum_distance_to_ball: 0,
    sum_distance_to_ball_has_possession: 0,
    time_has_possession: 0,
    sum_distance_to_ball_no_possession: 0,
    time_no_possession: 0,
    time_demolished: 0,
    time_no_teammates: 0,
    time_most_back: 0,
    time_most_forward: 0,
    time_mid_role: 0,
    time_other_role: 0,
    time_defensive_third: 0,
    time_neutral_third: 0,
    time_offensive_third: 0,
    time_defensive_half: 0,
    time_offensive_half: 0,
    time_closest_to_ball: 0,
    time_farthest_from_ball: 0,
    time_behind_ball: 0,
    time_level_with_ball: 0,
    time_in_front_of_ball: 0,
    times_caught_ahead_of_play_on_conceded_goals: 0,
  };
}

function sortPositioningEvents(events: readonly PositioningEvent[]): PositioningEvent[] {
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

function applyPositioningEvent(stats: PositioningStats, event: PositioningEvent): void {
  for (const field of FLOAT_POSITIONING_FIELDS) {
    stats[field] = addF32(stats[field], event[field]);
  }
  stats.times_caught_ahead_of_play_on_conceded_goals +=
    event.times_caught_ahead_of_play_on_conceded_goals;
}

function assignPositioningStats(
  target: PositioningStats,
  source: PositioningStats | undefined,
): void {
  Object.assign(target, source ?? defaultPositioningStats());
}

export function applyPositioningEventDerivedStats(
  timeline: MaterializedStatsTimeline,
): MaterializedStatsTimeline {
  const accumulator = createPositioningEventDerivedStatsAccumulator(timeline);

  for (const frame of timeline.frames) {
    accumulator.applyFrame(frame);
  }

  return timeline;
}

export function createPositioningEventDerivedStatsAccumulator(
  timeline: MaterializedStatsTimeline,
): {
  applyFrame(frame: StatsFrame): void;
} {
  const events = sortPositioningEvents(timeline.events.positioning ?? []);

  let eventIndex = 0;
  const players = new Map<string, PositioningStats>();

  return {
    applyFrame(frame: StatsFrame): void {
      while (eventIndex < events.length && events[eventIndex]!.frame <= frame.frame_number) {
        const event = events[eventIndex] as PositioningEvent;
        const playerKey = remoteIdKey(event.player);
        const playerStats = players.get(playerKey) ?? defaultPositioningStats();
        players.set(playerKey, playerStats);
        applyPositioningEvent(playerStats, event);
        eventIndex += 1;
      }

      for (const player of frame.players) {
        assignPositioningStats(player.positioning, players.get(remoteIdKey(player.player_id)));
      }
    },
  };
}
