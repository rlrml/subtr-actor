import type { PositioningEvent } from "./generated/PositioningEvent.ts";
import type { PositioningStats } from "./generated/PositioningStats.ts";
import type { StatsTimeline } from "./statsTimeline.ts";

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
  stats.active_game_time += event.active_game_time;
  stats.tracked_time += event.tracked_time;
  stats.sum_distance_to_teammates += event.sum_distance_to_teammates;
  stats.sum_distance_to_ball += event.sum_distance_to_ball;
  stats.sum_distance_to_ball_has_possession += event.sum_distance_to_ball_has_possession;
  stats.time_has_possession += event.time_has_possession;
  stats.sum_distance_to_ball_no_possession += event.sum_distance_to_ball_no_possession;
  stats.time_no_possession += event.time_no_possession;
  stats.time_demolished += event.time_demolished;
  stats.time_no_teammates += event.time_no_teammates;
  stats.time_most_back += event.time_most_back;
  stats.time_most_forward += event.time_most_forward;
  stats.time_mid_role += event.time_mid_role;
  stats.time_other_role += event.time_other_role;
  stats.time_defensive_third += event.time_defensive_third;
  stats.time_neutral_third += event.time_neutral_third;
  stats.time_offensive_third += event.time_offensive_third;
  stats.time_defensive_half += event.time_defensive_half;
  stats.time_offensive_half += event.time_offensive_half;
  stats.time_closest_to_ball += event.time_closest_to_ball;
  stats.time_farthest_from_ball += event.time_farthest_from_ball;
  stats.time_behind_ball += event.time_behind_ball;
  stats.time_level_with_ball += event.time_level_with_ball;
  stats.time_in_front_of_ball += event.time_in_front_of_ball;
  stats.times_caught_ahead_of_play_on_conceded_goals +=
    event.times_caught_ahead_of_play_on_conceded_goals;
}

function assignPositioningStats(
  target: PositioningStats,
  source: PositioningStats | undefined,
): void {
  Object.assign(target, source ?? defaultPositioningStats());
}

export function applyPositioningEventDerivedStats(timeline: StatsTimeline): StatsTimeline {
  const events = sortPositioningEvents(timeline.events.positioning ?? []);

  let eventIndex = 0;
  const players = new Map<string, PositioningStats>();

  for (const frame of timeline.frames) {
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
  }

  return timeline;
}
