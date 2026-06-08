import type { PositioningActivityEvent } from "./generated/PositioningActivityEvent.ts";
import type { PositioningBallDepthEvent } from "./generated/PositioningBallDepthEvent.ts";
import type { PositioningBallProximityEvent } from "./generated/PositioningBallProximityEvent.ts";
import type { PositioningDistanceEvent } from "./generated/PositioningDistanceEvent.ts";
import type { PositioningFieldZoneEvent } from "./generated/PositioningFieldZoneEvent.ts";
import type { PositioningGoalContextEvent } from "./generated/PositioningGoalContextEvent.ts";
import type { PositioningStats } from "./generated/PositioningStats.ts";
import type { PositioningTeamStats } from "./generated/PositioningTeamStats.ts";
import type { PositioningTeammateRoleEvent } from "./generated/PositioningTeammateRoleEvent.ts";
import type { StatsFrame, MaterializedStatsTimeline } from "./statsTimeline.ts";

type PositioningTimedEvent =
  | PositioningActivityEvent
  | PositioningDistanceEvent
  | PositioningFieldZoneEvent
  | PositioningBallDepthEvent
  | PositioningTeammateRoleEvent
  | PositioningBallProximityEvent
  | PositioningGoalContextEvent;

interface EventStreamCursor {
  applyThroughFrame(frameNumber: number): void;
}

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
    time_closest_to_ball_team: 0,
    time_closest_to_ball_absolute: 0,
    time_farthest_from_ball: 0,
    time_behind_ball: 0,
    time_level_with_ball: 0,
    time_in_front_of_ball: 0,
    times_caught_ahead_of_play_on_conceded_goals: 0,
  };
}

function defaultPositioningTeamStats(): PositioningTeamStats {
  return {
    tracked_time: 0,
    time_closest_to_ball: 0,
    time_closest_to_ball_team: 0,
    time_closest_to_ball_absolute: 0,
  };
}

function sortPositioningEvents<TEvent extends PositioningTimedEvent>(
  events: readonly TEvent[],
): TEvent[] {
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

function playerStatsFor(
  players: Map<string, PositioningStats>,
  event: PositioningTimedEvent,
): PositioningStats {
  const playerKey = remoteIdKey(event.player);
  const stats = players.get(playerKey) ?? defaultPositioningStats();
  players.set(playerKey, stats);
  return stats;
}

function applyActivityEvent(stats: PositioningStats, event: PositioningActivityEvent): void {
  if (event.active) {
    stats.active_game_time = addF32(stats.active_game_time, event.duration);
  }
  if (event.tracked) {
    stats.tracked_time = addF32(stats.tracked_time, event.duration);
  }
  if (event.demolished) {
    stats.time_demolished = addF32(stats.time_demolished, event.duration);
  }
}

function applyDistanceEvent(stats: PositioningStats, event: PositioningDistanceEvent): void {
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
}

function applyFieldZoneEvent(stats: PositioningStats, event: PositioningFieldZoneEvent): void {
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
}

function applyBallDepthEvent(stats: PositioningStats, event: PositioningBallDepthEvent): void {
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

function applyTeammateRoleEvent(
  stats: PositioningStats,
  event: PositioningTeammateRoleEvent,
): void {
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
}

function applyBallProximityEvent(
  stats: PositioningStats,
  teamStats: PositioningTeamStats,
  event: PositioningBallProximityEvent,
): void {
  if (event.closest_to_ball_team) {
    teamStats.tracked_time = addF32(teamStats.tracked_time, event.duration);
    teamStats.time_closest_to_ball = addF32(teamStats.time_closest_to_ball, event.duration);
    teamStats.time_closest_to_ball_team = addF32(
      teamStats.time_closest_to_ball_team,
      event.duration,
    );
    stats.time_closest_to_ball = addF32(stats.time_closest_to_ball, event.duration);
    stats.time_closest_to_ball_team = addF32(stats.time_closest_to_ball_team, event.duration);
  }
  if (event.closest_to_ball_absolute) {
    teamStats.time_closest_to_ball_absolute = addF32(
      teamStats.time_closest_to_ball_absolute,
      event.duration,
    );
    stats.time_closest_to_ball_absolute = addF32(
      stats.time_closest_to_ball_absolute,
      event.duration,
    );
  }
  if (event.farthest_from_ball) {
    stats.time_farthest_from_ball = addF32(stats.time_farthest_from_ball, event.duration);
  }
}

function applyGoalContextEvent(stats: PositioningStats, event: PositioningGoalContextEvent): void {
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

function assignPositioningTeamStats(
  target: PositioningTeamStats,
  source: PositioningTeamStats | undefined,
): void {
  Object.assign(target, source ?? defaultPositioningTeamStats());
}

function createEventStreamCursor<TEvent extends PositioningTimedEvent>(
  events: readonly TEvent[],
  apply: (event: TEvent) => void,
): EventStreamCursor {
  const sortedEvents = sortPositioningEvents(events);
  let index = 0;
  return {
    applyThroughFrame(frameNumber: number): void {
      while (index < sortedEvents.length && sortedEvents[index]!.frame <= frameNumber) {
        apply(sortedEvents[index]!);
        index += 1;
      }
    },
  };
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
  const players = new Map<string, PositioningStats>();
  const teamZero = defaultPositioningTeamStats();
  const teamOne = defaultPositioningTeamStats();

  const streams: EventStreamCursor[] = [
    createEventStreamCursor(timeline.events.positioning_activity ?? [], (event) =>
      applyActivityEvent(playerStatsFor(players, event), event),
    ),
    createEventStreamCursor(timeline.events.positioning_distance ?? [], (event) =>
      applyDistanceEvent(playerStatsFor(players, event), event),
    ),
    createEventStreamCursor(timeline.events.positioning_field_zone ?? [], (event) =>
      applyFieldZoneEvent(playerStatsFor(players, event), event),
    ),
    createEventStreamCursor(timeline.events.positioning_ball_depth ?? [], (event) =>
      applyBallDepthEvent(playerStatsFor(players, event), event),
    ),
    createEventStreamCursor(timeline.events.positioning_teammate_role ?? [], (event) =>
      applyTeammateRoleEvent(playerStatsFor(players, event), event),
    ),
    createEventStreamCursor(timeline.events.positioning_ball_proximity ?? [], (event) =>
      applyBallProximityEvent(
        playerStatsFor(players, event),
        event.is_team_0 ? teamZero : teamOne,
        event,
      ),
    ),
    createEventStreamCursor(timeline.events.positioning_goal_context ?? [], (event) =>
      applyGoalContextEvent(playerStatsFor(players, event), event),
    ),
  ];

  return {
    applyFrame(frame: StatsFrame): void {
      for (const stream of streams) {
        stream.applyThroughFrame(frame.frame_number);
      }

      assignPositioningTeamStats(frame.team_zero.positioning, teamZero);
      assignPositioningTeamStats(frame.team_one.positioning, teamOne);
      for (const player of frame.players) {
        assignPositioningStats(player.positioning, players.get(remoteIdKey(player.player_id)));
      }
    },
  };
}
