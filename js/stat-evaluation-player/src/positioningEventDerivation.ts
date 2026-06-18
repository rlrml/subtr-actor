import type { PositioningStats } from "./generated/PositioningStats.ts";
import type { PositioningTeamStats } from "./generated/PositioningTeamStats.ts";
import type {
  BallDepthEvent,
  BallProximityEvent,
  DepthRoleEvent,
  FieldHalfEvent,
  FieldThirdEvent,
  PlayerActivityEvent,
  ShadowDefenseEvent,
  StatsFrame,
  MaterializedStatsTimeline,
} from "./statsTimeline.ts";
import { statsEventPayloads } from "./statsTimeline.ts";

type PositioningSpanEvent =
  | PlayerActivityEvent
  | FieldThirdEvent
  | FieldHalfEvent
  | BallDepthEvent
  | DepthRoleEvent
  | BallProximityEvent
  | ShadowDefenseEvent;

interface EventStreamCursor {
  applyThroughFrame(frame: StatsFrame): void;
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
    time_closest_to_ball_team: 0,
    time_closest_to_ball_absolute: 0,
    time_farthest_from_ball: 0,
    time_shadow_defense: 0,
    time_behind_ball: 0,
    time_level_with_ball: 0,
    time_in_front_of_ball: 0,
  };
}

function defaultPositioningTeamStats(): PositioningTeamStats {
  return {
    tracked_time: 0,
    time_closest_to_ball_team: 0,
    time_closest_to_ball_absolute: 0,
  };
}

function sortPositioningEvents<TEvent extends PositioningSpanEvent>(
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
  event: PositioningSpanEvent,
): PositioningStats {
  const playerKey = remoteIdKey(event.player);
  const stats = players.get(playerKey) ?? defaultPositioningStats();
  players.set(playerKey, stats);
  return stats;
}

function applyActivityEvent(stats: PositioningStats, event: PlayerActivityEvent): void {
  stats.active_game_time = addF32(stats.active_game_time, event.duration);
  switch (event.state) {
    case "tracked":
      stats.tracked_time = addF32(stats.tracked_time, event.duration);
      break;
    case "demolished":
      stats.time_demolished = addF32(stats.time_demolished, event.duration);
      break;
  }
}

function applyFieldThirdEvent(stats: PositioningStats, event: FieldThirdEvent): void {
  switch (event.state) {
    case "defensive":
      stats.time_defensive_third = addF32(stats.time_defensive_third, event.duration);
      break;
    case "neutral":
      stats.time_neutral_third = addF32(stats.time_neutral_third, event.duration);
      break;
    case "offensive":
      stats.time_offensive_third = addF32(stats.time_offensive_third, event.duration);
      break;
  }
}

function applyFieldHalfEvent(stats: PositioningStats, event: FieldHalfEvent): void {
  switch (event.state) {
    case "defensive":
      stats.time_defensive_half = addF32(stats.time_defensive_half, event.duration);
      break;
    case "offensive":
      stats.time_offensive_half = addF32(stats.time_offensive_half, event.duration);
      break;
  }
}

function applyBallDepthEvent(stats: PositioningStats, event: BallDepthEvent): void {
  switch (event.state) {
    case "behind_ball":
      stats.time_behind_ball = addF32(stats.time_behind_ball, event.duration);
      break;
    case "level_with_ball":
      stats.time_level_with_ball = addF32(stats.time_level_with_ball, event.duration);
      break;
    case "ahead_of_ball":
      stats.time_in_front_of_ball = addF32(stats.time_in_front_of_ball, event.duration);
      break;
  }
}

function applyDepthRoleEvent(stats: PositioningStats, event: DepthRoleEvent): void {
  switch (event.state) {
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
  event: BallProximityEvent,
): void {
  if (event.state.closest_to_ball_team) {
    teamStats.tracked_time = addF32(teamStats.tracked_time, event.duration);
    teamStats.time_closest_to_ball_team = addF32(
      teamStats.time_closest_to_ball_team,
      event.duration,
    );
    stats.time_closest_to_ball_team = addF32(stats.time_closest_to_ball_team, event.duration);
  }
  if (event.state.closest_to_ball_absolute) {
    teamStats.time_closest_to_ball_absolute = addF32(
      teamStats.time_closest_to_ball_absolute,
      event.duration,
    );
    stats.time_closest_to_ball_absolute = addF32(
      stats.time_closest_to_ball_absolute,
      event.duration,
    );
  }
  if (event.state.farthest_from_ball) {
    stats.time_farthest_from_ball = addF32(stats.time_farthest_from_ball, event.duration);
  }
}

function applyShadowDefenseEvent(stats: PositioningStats, event: ShadowDefenseEvent): void {
  switch (event.state) {
    case "shadowing":
      stats.time_shadow_defense = addF32(stats.time_shadow_defense, event.duration);
      break;
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

function createEventStreamCursor<TEvent extends PositioningSpanEvent>(
  events: readonly TEvent[],
  apply: (event: TEvent) => void,
): EventStreamCursor {
  const sortedEvents = sortPositioningEvents(events);
  const creditedDurations = new Array(sortedEvents.length).fill(0) as number[];
  return {
    applyThroughFrame(frame: StatsFrame): void {
      for (let index = 0; index < sortedEvents.length; index += 1) {
        const event = sortedEvents[index]!;
        if (event.frame > frame.frame_number) {
          break;
        }
        const targetDuration = creditedDurationThroughFrame(event, frame);
        const delta = targetDuration - creditedDurations[index]!;
        if (delta > 0) {
          creditedDurations[index] = targetDuration;
          apply({ ...event, duration: delta });
        }
      }
    },
  };
}

function creditedDurationThroughFrame(event: PositioningSpanEvent, frame: StatsFrame): number {
  if (frame.frame_number >= event.end_frame) {
    return event.duration;
  }
  const totalTime = event.end_time - event.time;
  if (totalTime <= 0) {
    return 0;
  }
  const elapsedTime = Math.max(0, frame.time - event.time);
  return event.duration * Math.min(1, elapsedTime / totalTime);
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
    createEventStreamCursor(statsEventPayloads(timeline, "player_activity"), (event) =>
      applyActivityEvent(playerStatsFor(players, event), event),
    ),
    createEventStreamCursor(statsEventPayloads(timeline, "field_third"), (event) =>
      applyFieldThirdEvent(playerStatsFor(players, event), event),
    ),
    createEventStreamCursor(statsEventPayloads(timeline, "field_half"), (event) =>
      applyFieldHalfEvent(playerStatsFor(players, event), event),
    ),
    createEventStreamCursor(statsEventPayloads(timeline, "ball_depth"), (event) =>
      applyBallDepthEvent(playerStatsFor(players, event), event),
    ),
    createEventStreamCursor(statsEventPayloads(timeline, "depth_role"), (event) =>
      applyDepthRoleEvent(playerStatsFor(players, event), event),
    ),
    createEventStreamCursor(statsEventPayloads(timeline, "ball_proximity"), (event) =>
      applyBallProximityEvent(
        playerStatsFor(players, event),
        event.is_team_0 ? teamZero : teamOne,
        event,
      ),
    ),
    createEventStreamCursor(statsEventPayloads(timeline, "shadow_defense"), (event) =>
      applyShadowDefenseEvent(playerStatsFor(players, event), event),
    ),
  ];

  return {
    applyFrame(frame: StatsFrame): void {
      for (const stream of streams) {
        stream.applyThroughFrame(frame);
      }

      assignPositioningTeamStats(frame.team_zero.positioning, teamZero);
      assignPositioningTeamStats(frame.team_one.positioning, teamOne);
      for (const player of frame.players) {
        assignPositioningStats(player.positioning, players.get(remoteIdKey(player.player_id)));
        assignPositioningSummary(player.positioning, timeline, player.player_id);
      }
    },
  };
}

function assignPositioningSummary(
  target: PositioningStats,
  timeline: MaterializedStatsTimeline,
  playerId: unknown,
): void {
  const summaries = (timeline as unknown as { positioning_summary?: unknown }).positioning_summary;
  if (!Array.isArray(summaries)) {
    return;
  }
  const playerKey = remoteIdKey(playerId);
  const summary = summaries.find((candidate) => {
    if (!candidate || typeof candidate !== "object") {
      return false;
    }
    return remoteIdKey((candidate as { player_id?: unknown }).player_id) === playerKey;
  }) as { distance?: Partial<PositioningStats> } | undefined;
  if (!summary?.distance) {
    return;
  }
  Object.assign(target, summary.distance);
}
