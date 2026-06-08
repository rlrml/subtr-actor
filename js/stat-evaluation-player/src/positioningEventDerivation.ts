import type { PositioningActivityEvent } from "./generated/PositioningActivityEvent.ts";
import type { PositioningBallDepthEvent } from "./generated/PositioningBallDepthEvent.ts";
import type { PositioningBallProximityEvent } from "./generated/PositioningBallProximityEvent.ts";
import type { PositioningFieldZoneEvent } from "./generated/PositioningFieldZoneEvent.ts";
import type { PositioningGoalContextEvent } from "./generated/PositioningGoalContextEvent.ts";
import type { PositioningPossessionEvent } from "./generated/PositioningPossessionEvent.ts";
import type { PositioningSignalSnapshot } from "./generated/PositioningSignalSnapshot.ts";
import type { PositioningStats } from "./generated/PositioningStats.ts";
import type { PositioningTeamStats } from "./generated/PositioningTeamStats.ts";
import type { PositioningTeammateRoleEvent } from "./generated/PositioningTeammateRoleEvent.ts";
import type { ReplayStatsPositioningSummary } from "./generated/ReplayStatsPositioningSummary.ts";
import type { StatsFrame, MaterializedStatsTimeline } from "./statsTimeline.ts";

type PositioningTimedEvent =
  | PositioningActivityEvent
  | PositioningPossessionEvent
  | PositioningFieldZoneEvent
  | PositioningBallDepthEvent
  | PositioningTeammateRoleEvent
  | PositioningBallProximityEvent
  | PositioningGoalContextEvent;

interface EventStreamCursor {
  applyThroughFrame(frameNumber: number, frameTime: number): void;
}

interface DurationalSpan {
  frame: number;
  end_frame: number;
  time: number;
  end_time: number;
  duration: number;
}

/// Fraction of a span's duration that has elapsed by the playhead at `(currentFrame,
/// currentTime)`. A span is fully credited once the playhead reaches its `end_frame`
/// (guaranteeing the final snapshot matches the exported totals regardless of dt
/// rounding); while in progress it accrues linearly in time across `[time, end_time]`.
function spanCreditFraction(
  span: DurationalSpan,
  currentFrame: number,
  currentTime: number,
): number {
  if (currentFrame >= span.end_frame) {
    return 1;
  }
  if (currentFrame < span.frame) {
    return 0;
  }
  const denominator = span.end_time - span.time;
  if (denominator <= 0) {
    return 1;
  }
  const fraction = (currentTime - span.time) / denominator;
  if (fraction <= 0) {
    return 0;
  }
  return fraction >= 1 ? 1 : fraction;
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

function applyActivityEvent(
  stats: PositioningStats,
  event: PositioningActivityEvent,
  creditedDuration: number,
): void {
  if (event.active) {
    stats.active_game_time = addF32(stats.active_game_time, creditedDuration);
  }
  if (event.tracked) {
    stats.tracked_time = addF32(stats.tracked_time, creditedDuration);
  }
  if (event.demolished) {
    stats.time_demolished = addF32(stats.time_demolished, creditedDuration);
  }
}

/// Seed the distance/possession fields from the cumulative signal shipped on the frame.
/// Distance to ball/teammates is a continuous quantity, so it travels as a per-frame
/// sampled snapshot rather than as events; the client reads it directly here.
function applyPositioningSignal(stats: PositioningStats, signal: PositioningSignalSnapshot): void {
  stats.sum_distance_to_teammates = signal.sum_distance_to_teammates;
  stats.sum_distance_to_ball = signal.sum_distance_to_ball;
  stats.sum_distance_to_ball_has_possession = signal.sum_distance_to_ball_has_possession;
  stats.sum_distance_to_ball_no_possession = signal.sum_distance_to_ball_no_possession;
}

function applyPossessionEvent(
  stats: PositioningStats,
  event: PositioningPossessionEvent,
  creditedDuration: number,
): void {
  if (event.possession_state === "has_possession") {
    stats.time_has_possession = addF32(stats.time_has_possession, creditedDuration);
  } else if (event.possession_state === "no_possession") {
    stats.time_no_possession = addF32(stats.time_no_possession, creditedDuration);
  }
}

function applyFieldZoneEvent(
  stats: PositioningStats,
  event: PositioningFieldZoneEvent,
  creditedDuration: number,
): void {
  stats.time_defensive_third = addF32(
    stats.time_defensive_third,
    creditedDuration * event.defensive_zone_fraction,
  );
  stats.time_neutral_third = addF32(
    stats.time_neutral_third,
    creditedDuration * event.neutral_zone_fraction,
  );
  stats.time_offensive_third = addF32(
    stats.time_offensive_third,
    creditedDuration * event.offensive_zone_fraction,
  );
  stats.time_defensive_half = addF32(
    stats.time_defensive_half,
    creditedDuration * event.defensive_half_fraction,
  );
  stats.time_offensive_half = addF32(
    stats.time_offensive_half,
    creditedDuration * event.offensive_half_fraction,
  );
}

function applyBallDepthEvent(
  stats: PositioningStats,
  event: PositioningBallDepthEvent,
  creditedDuration: number,
): void {
  stats.time_behind_ball = addF32(
    stats.time_behind_ball,
    creditedDuration * event.behind_ball_fraction,
  );
  stats.time_level_with_ball = addF32(
    stats.time_level_with_ball,
    creditedDuration * event.level_with_ball_fraction,
  );
  stats.time_in_front_of_ball = addF32(
    stats.time_in_front_of_ball,
    creditedDuration * event.in_front_of_ball_fraction,
  );
}

function applyTeammateRoleEvent(
  stats: PositioningStats,
  event: PositioningTeammateRoleEvent,
  creditedDuration: number,
): void {
  switch (event.teammate_role) {
    case "no_teammates":
      stats.time_no_teammates = addF32(stats.time_no_teammates, creditedDuration);
      break;
    case "most_back":
      stats.time_most_back = addF32(stats.time_most_back, creditedDuration);
      break;
    case "most_forward":
      stats.time_most_forward = addF32(stats.time_most_forward, creditedDuration);
      break;
    case "mid":
      stats.time_mid_role = addF32(stats.time_mid_role, creditedDuration);
      break;
    case "other":
      stats.time_other_role = addF32(stats.time_other_role, creditedDuration);
      break;
  }
}

function applyBallProximityEvent(
  stats: PositioningStats,
  teamStats: PositioningTeamStats,
  event: PositioningBallProximityEvent,
  creditedDuration: number,
): void {
  if (event.closest_to_ball_team) {
    teamStats.tracked_time = addF32(teamStats.tracked_time, creditedDuration);
    teamStats.time_closest_to_ball = addF32(teamStats.time_closest_to_ball, creditedDuration);
    teamStats.time_closest_to_ball_team = addF32(
      teamStats.time_closest_to_ball_team,
      creditedDuration,
    );
    stats.time_closest_to_ball = addF32(stats.time_closest_to_ball, creditedDuration);
    stats.time_closest_to_ball_team = addF32(stats.time_closest_to_ball_team, creditedDuration);
  }
  if (event.closest_to_ball_absolute) {
    teamStats.time_closest_to_ball_absolute = addF32(
      teamStats.time_closest_to_ball_absolute,
      creditedDuration,
    );
    stats.time_closest_to_ball_absolute = addF32(
      stats.time_closest_to_ball_absolute,
      creditedDuration,
    );
  }
  if (event.farthest_from_ball) {
    stats.time_farthest_from_ball = addF32(stats.time_farthest_from_ball, creditedDuration);
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

/// Cursor for durational span events. Each span accrues its `duration` to the stats
/// proportionally to how much of it the playhead has passed, so a coalesced multi-frame
/// span behaves like the per-frame stream it was compacted from rather than jumping its
/// whole duration at its start frame. Spans are fully credited once the playhead reaches
/// their `end_frame`, so the final snapshot matches the exported totals exactly.
function createProratingCursor<TEvent extends PositioningTimedEvent & DurationalSpan>(
  events: readonly TEvent[],
  apply: (event: TEvent, creditedDuration: number) => void,
): EventStreamCursor {
  const sortedEvents = sortPositioningEvents(events);
  let nextIndex = 0;
  const active: { event: TEvent; credited: number }[] = [];
  return {
    applyThroughFrame(frameNumber: number, frameTime: number): void {
      while (nextIndex < sortedEvents.length && sortedEvents[nextIndex]!.frame <= frameNumber) {
        active.push({ event: sortedEvents[nextIndex]!, credited: 0 });
        nextIndex += 1;
      }
      for (let index = active.length - 1; index >= 0; index -= 1) {
        const entry = active[index]!;
        const target =
          entry.event.duration * spanCreditFraction(entry.event, frameNumber, frameTime);
        const delta = target - entry.credited;
        if (delta !== 0) {
          apply(entry.event, delta);
          entry.credited = target;
        }
        if (frameNumber >= entry.event.end_frame) {
          active.splice(index, 1);
        }
      }
    },
  };
}

/// Cursor for point events (no duration) such as goal-context: each event is applied once
/// as soon as the playhead reaches its frame.
function createDiscreteCursor<TEvent extends PositioningTimedEvent>(
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

  // Distance is a continuous magnitude computed over the whole match and shipped once as a
  // per-player summary, not reconstructed from events. Index it by player so every frame's
  // snapshot reports the same match-level distance totals.
  const distanceSummary = new Map<string, PositioningSignalSnapshot>();
  const summaryEntries =
    (timeline as { positioning_summary?: ReplayStatsPositioningSummary[] }).positioning_summary ??
    [];
  for (const entry of summaryEntries) {
    distanceSummary.set(remoteIdKey(entry.player_id), entry.distance);
  }

  const streams: EventStreamCursor[] = [
    createProratingCursor(timeline.events.positioning_activity ?? [], (event, creditedDuration) =>
      applyActivityEvent(playerStatsFor(players, event), event, creditedDuration),
    ),
    createProratingCursor(timeline.events.positioning_possession ?? [], (event, creditedDuration) =>
      applyPossessionEvent(playerStatsFor(players, event), event, creditedDuration),
    ),
    createProratingCursor(timeline.events.positioning_field_zone ?? [], (event, creditedDuration) =>
      applyFieldZoneEvent(playerStatsFor(players, event), event, creditedDuration),
    ),
    createProratingCursor(timeline.events.positioning_ball_depth ?? [], (event, creditedDuration) =>
      applyBallDepthEvent(playerStatsFor(players, event), event, creditedDuration),
    ),
    createProratingCursor(
      timeline.events.positioning_teammate_role ?? [],
      (event, creditedDuration) =>
        applyTeammateRoleEvent(playerStatsFor(players, event), event, creditedDuration),
    ),
    createProratingCursor(
      timeline.events.positioning_ball_proximity ?? [],
      (event, creditedDuration) =>
        applyBallProximityEvent(
          playerStatsFor(players, event),
          event.is_team_0 ? teamZero : teamOne,
          event,
          creditedDuration,
        ),
    ),
    createDiscreteCursor(timeline.events.positioning_goal_context ?? [], (event) =>
      applyGoalContextEvent(playerStatsFor(players, event), event),
    ),
  ];

  return {
    applyFrame(frame: StatsFrame): void {
      for (const stream of streams) {
        stream.applyThroughFrame(frame.frame_number, frame.time);
      }

      assignPositioningTeamStats(frame.team_zero.positioning, teamZero);
      assignPositioningTeamStats(frame.team_one.positioning, teamOne);
      for (const player of frame.players) {
        const playerKey = remoteIdKey(player.player_id);
        assignPositioningStats(player.positioning, players.get(playerKey));
        const distance = distanceSummary.get(playerKey);
        if (distance) {
          applyPositioningSignal(player.positioning, distance);
        }
      }
    },
  };
}
