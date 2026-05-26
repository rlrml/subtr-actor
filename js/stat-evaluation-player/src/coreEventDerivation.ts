import type { CorePlayerStats } from "./generated/CorePlayerStats.ts";
import type { CorePlayerStatsEvent } from "./generated/CorePlayerStatsEvent.ts";
import type { CoreTeamStats } from "./generated/CoreTeamStats.ts";
import type { CoreTeamStatsEvent } from "./generated/CoreTeamStatsEvent.ts";
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

function addF32(left: number, right: number): number {
  return Math.fround(Math.fround(left) + Math.fround(right));
}

function defaultCoreTeamStats(): CoreTeamStats {
  return {
    score: 0,
    goals: 0,
    assists: 0,
    saves: 0,
    shots: 0,
    kickoff_goal_count: 0,
    short_goal_count: 0,
    medium_goal_count: 0,
    long_goal_count: 0,
    counter_attack_goal_count: 0,
    sustained_pressure_goal_count: 0,
    other_buildup_goal_count: 0,
    goal_ball_air_time_sample_count: 0,
    cumulative_goal_ball_air_time: 0,
    last_goal_ball_air_time: null,
  };
}

function defaultCorePlayerStats(): CorePlayerStats {
  return {
    ...defaultCoreTeamStats(),
    goals_conceded_while_last_defender: 0,
    goals_for_while_most_back: 0,
    goals_against_while_most_back: 0,
    goal_against_boost_sample_count: 0,
    cumulative_boost_on_goals_against: 0,
    last_boost_on_goal_against: null,
    goal_against_boost_leadup_sample_count: 0,
    cumulative_average_boost_in_goal_against_leadup: 0,
    cumulative_min_boost_in_goal_against_leadup: 0,
    last_average_boost_in_goal_against_leadup: null,
    last_min_boost_in_goal_against_leadup: null,
    goal_against_position_sample_count: 0,
    cumulative_goal_against_position_x: 0,
    cumulative_goal_against_position_y: 0,
    cumulative_goal_against_position_z: 0,
    last_goal_against_position: null,
    scoring_goal_last_touch_position_sample_count: 0,
    cumulative_scoring_goal_last_touch_position_x: 0,
    cumulative_scoring_goal_last_touch_position_y: 0,
    cumulative_scoring_goal_last_touch_position_z: 0,
    last_scoring_goal_last_touch_position: null,
  };
}

function sortCoreEvents<T extends { time: number; frame: number }>(events: readonly T[]): T[] {
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

function assignCorePlayerStats(target: CorePlayerStats, source: CorePlayerStats | undefined): void {
  Object.assign(target, source ?? defaultCorePlayerStats());
}

function assignCoreTeamStats(target: CoreTeamStats, source: CoreTeamStats): void {
  Object.assign(target, source);
}

function applyOptionalPositionDelta<T extends { x: number; y: number; z: number }>(
  current: T | null,
  delta: T | null,
): T | null {
  return delta == null ? current : { ...delta };
}

function applyCoreTeamDelta(stats: CoreTeamStats, delta: CoreTeamStats): void {
  stats.score += delta.score;
  stats.goals += delta.goals;
  stats.assists += delta.assists;
  stats.saves += delta.saves;
  stats.shots += delta.shots;
  stats.kickoff_goal_count += delta.kickoff_goal_count;
  stats.short_goal_count += delta.short_goal_count;
  stats.medium_goal_count += delta.medium_goal_count;
  stats.long_goal_count += delta.long_goal_count;
  stats.counter_attack_goal_count += delta.counter_attack_goal_count;
  stats.sustained_pressure_goal_count += delta.sustained_pressure_goal_count;
  stats.other_buildup_goal_count += delta.other_buildup_goal_count;
  stats.goal_ball_air_time_sample_count += delta.goal_ball_air_time_sample_count;
  stats.cumulative_goal_ball_air_time = addF32(
    stats.cumulative_goal_ball_air_time,
    delta.cumulative_goal_ball_air_time,
  );
  if (delta.last_goal_ball_air_time != null) {
    stats.last_goal_ball_air_time = delta.last_goal_ball_air_time;
  }
}

function applyCorePlayerDelta(stats: CorePlayerStats, delta: CorePlayerStats): void {
  applyCoreTeamDelta(stats, delta);
  stats.goals_conceded_while_last_defender += delta.goals_conceded_while_last_defender;
  stats.goals_for_while_most_back += delta.goals_for_while_most_back;
  stats.goals_against_while_most_back += delta.goals_against_while_most_back;
  stats.goal_against_boost_sample_count += delta.goal_against_boost_sample_count;
  stats.cumulative_boost_on_goals_against = addF32(
    stats.cumulative_boost_on_goals_against,
    delta.cumulative_boost_on_goals_against,
  );
  if (delta.last_boost_on_goal_against != null) {
    stats.last_boost_on_goal_against = delta.last_boost_on_goal_against;
  }
  stats.goal_against_boost_leadup_sample_count += delta.goal_against_boost_leadup_sample_count;
  stats.cumulative_average_boost_in_goal_against_leadup = addF32(
    stats.cumulative_average_boost_in_goal_against_leadup,
    delta.cumulative_average_boost_in_goal_against_leadup,
  );
  stats.cumulative_min_boost_in_goal_against_leadup = addF32(
    stats.cumulative_min_boost_in_goal_against_leadup,
    delta.cumulative_min_boost_in_goal_against_leadup,
  );
  if (delta.last_average_boost_in_goal_against_leadup != null) {
    stats.last_average_boost_in_goal_against_leadup =
      delta.last_average_boost_in_goal_against_leadup;
  }
  if (delta.last_min_boost_in_goal_against_leadup != null) {
    stats.last_min_boost_in_goal_against_leadup = delta.last_min_boost_in_goal_against_leadup;
  }
  stats.goal_against_position_sample_count += delta.goal_against_position_sample_count;
  stats.cumulative_goal_against_position_x = addF32(
    stats.cumulative_goal_against_position_x,
    delta.cumulative_goal_against_position_x,
  );
  stats.cumulative_goal_against_position_y = addF32(
    stats.cumulative_goal_against_position_y,
    delta.cumulative_goal_against_position_y,
  );
  stats.cumulative_goal_against_position_z = addF32(
    stats.cumulative_goal_against_position_z,
    delta.cumulative_goal_against_position_z,
  );
  stats.last_goal_against_position = applyOptionalPositionDelta(
    stats.last_goal_against_position,
    delta.last_goal_against_position,
  );
  stats.scoring_goal_last_touch_position_sample_count +=
    delta.scoring_goal_last_touch_position_sample_count;
  stats.cumulative_scoring_goal_last_touch_position_x = addF32(
    stats.cumulative_scoring_goal_last_touch_position_x,
    delta.cumulative_scoring_goal_last_touch_position_x,
  );
  stats.cumulative_scoring_goal_last_touch_position_y = addF32(
    stats.cumulative_scoring_goal_last_touch_position_y,
    delta.cumulative_scoring_goal_last_touch_position_y,
  );
  stats.cumulative_scoring_goal_last_touch_position_z = addF32(
    stats.cumulative_scoring_goal_last_touch_position_z,
    delta.cumulative_scoring_goal_last_touch_position_z,
  );
  stats.last_scoring_goal_last_touch_position = applyOptionalPositionDelta(
    stats.last_scoring_goal_last_touch_position,
    delta.last_scoring_goal_last_touch_position,
  );
}

export function applyCoreEventDerivedStats(
  timeline: MaterializedStatsTimeline,
): MaterializedStatsTimeline {
  const accumulator = createCoreEventDerivedStatsAccumulator(timeline);

  for (const frame of timeline.frames) {
    accumulator.applyFrame(frame);
  }

  return timeline;
}

export function createCoreEventDerivedStatsAccumulator(timeline: MaterializedStatsTimeline): {
  applyFrame(frame: StatsFrame): void;
} {
  const playerEvents = sortCoreEvents(timeline.events.core_player ?? []);
  const teamEvents = sortCoreEvents(timeline.events.core_team ?? []);

  let playerEventIndex = 0;
  let teamEventIndex = 0;
  const players = new Map<string, CorePlayerStats>();
  const teamZero = defaultCoreTeamStats();
  const teamOne = defaultCoreTeamStats();

  return {
    applyFrame(frame: StatsFrame): void {
      while (
        playerEventIndex < playerEvents.length &&
        playerEvents[playerEventIndex]!.frame <= frame.frame_number
      ) {
        const event = playerEvents[playerEventIndex] as CorePlayerStatsEvent;
        const playerKey = remoteIdKey(event.player);
        const stats = players.get(playerKey) ?? defaultCorePlayerStats();
        players.set(playerKey, stats);
        applyCorePlayerDelta(stats, event.delta);
        playerEventIndex += 1;
      }

      while (
        teamEventIndex < teamEvents.length &&
        teamEvents[teamEventIndex]!.frame <= frame.frame_number
      ) {
        const event = teamEvents[teamEventIndex] as CoreTeamStatsEvent;
        if (event.is_team_0) {
          applyCoreTeamDelta(teamZero, event.delta);
        } else {
          applyCoreTeamDelta(teamOne, event.delta);
        }
        teamEventIndex += 1;
      }

      assignCoreTeamStats(frame.team_zero.core, teamZero);
      assignCoreTeamStats(frame.team_one.core, teamOne);
      for (const player of frame.players) {
        assignCorePlayerStats(player.core, players.get(remoteIdKey(player.player_id)));
      }
    },
  };
}
