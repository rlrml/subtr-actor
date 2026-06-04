import type { PositioningEvent } from "./generated/PositioningEvent.ts";
import type { PositioningStats } from "./generated/PositioningStats.ts";
import type { StatsFrame, MaterializedStatsTimeline } from "./statsTimeline.ts";

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
  if (event.active) {
    stats.active_game_time = addF32(stats.active_game_time, event.duration);
  }
  if (event.tracked) {
    stats.tracked_time = addF32(stats.tracked_time, event.duration);
    if (event.distance_to_teammates != null) {
      stats.sum_distance_to_teammates = addF32(
        stats.sum_distance_to_teammates,
        event.distance_to_teammates * event.duration,
      );
    }
    if (event.distance_to_ball != null) {
      const distanceIntegral = event.distance_to_ball * event.duration;
      stats.sum_distance_to_ball = addF32(stats.sum_distance_to_ball, distanceIntegral);
      if (event.possession_state === "has_possession") {
        stats.sum_distance_to_ball_has_possession = addF32(
          stats.sum_distance_to_ball_has_possession,
          distanceIntegral,
        );
      } else if (event.possession_state === "no_possession") {
        stats.sum_distance_to_ball_no_possession = addF32(
          stats.sum_distance_to_ball_no_possession,
          distanceIntegral,
        );
      }
    }
    if (event.possession_state === "has_possession") {
      stats.time_has_possession = addF32(stats.time_has_possession, event.duration);
    } else if (event.possession_state === "no_possession") {
      stats.time_no_possession = addF32(stats.time_no_possession, event.duration);
    }
    switch (event.teammate_role) {
      case "no_teammates":
        stats.time_no_teammates = addF32(stats.time_no_teammates, event.duration);
        break;
      case "most_back":
        stats.time_most_back = addF32(stats.time_most_back, event.duration);
        break;
      case "most_forward":
        stats.time_most_forward = addF32(stats.time_most_forward, event.duration);
        break;
      case "mid":
        stats.time_mid_role = addF32(stats.time_mid_role, event.duration);
        break;
      case "other":
        stats.time_other_role = addF32(stats.time_other_role, event.duration);
        break;
    }
    stats.time_defensive_third = addF32(
      stats.time_defensive_third,
      event.duration * event.defensive_zone_fraction,
    );
    stats.time_neutral_third = addF32(
      stats.time_neutral_third,
      event.duration * event.neutral_zone_fraction,
    );
    stats.time_offensive_third = addF32(
      stats.time_offensive_third,
      event.duration * event.offensive_zone_fraction,
    );
    stats.time_defensive_half = addF32(
      stats.time_defensive_half,
      event.duration * event.defensive_half_fraction,
    );
    stats.time_offensive_half = addF32(
      stats.time_offensive_half,
      event.duration * event.offensive_half_fraction,
    );
    if (event.closest_to_ball) {
      stats.time_closest_to_ball = addF32(stats.time_closest_to_ball, event.duration);
    }
    if (event.farthest_from_ball) {
      stats.time_farthest_from_ball = addF32(stats.time_farthest_from_ball, event.duration);
    }
    stats.time_behind_ball = addF32(
      stats.time_behind_ball,
      event.duration * event.behind_ball_fraction,
    );
    stats.time_level_with_ball = addF32(
      stats.time_level_with_ball,
      event.duration * event.level_with_ball_fraction,
    );
    stats.time_in_front_of_ball = addF32(
      stats.time_in_front_of_ball,
      event.duration * event.in_front_of_ball_fraction,
    );
  }
  if (event.demolished) {
    stats.time_demolished = addF32(stats.time_demolished, event.duration);
  }
  if (event.caught_ahead_of_play_on_conceded_goal) {
    stats.times_caught_ahead_of_play_on_conceded_goals += 1;
  }
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
