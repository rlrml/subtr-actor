import type { CorePlayerStats } from "./generated/CorePlayerStats.ts";
import type { CorePlayerStatsEvent } from "./generated/CorePlayerStatsEvent.ts";
import type { CoreTeamStats } from "./generated/CoreTeamStats.ts";
import type { CoreTeamStatsEvent } from "./generated/CoreTeamStatsEvent.ts";
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

function assignCorePlayerStats(
  target: CorePlayerStats,
  source: CorePlayerStats | undefined,
): void {
  Object.assign(target, source ?? defaultCorePlayerStats());
}

function assignCoreTeamStats(target: CoreTeamStats, source: CoreTeamStats): void {
  Object.assign(target, source);
}

export function applyCoreEventDerivedStats(timeline: StatsTimeline): StatsTimeline {
  const playerEvents = sortCoreEvents(timeline.events.core_player ?? []);
  const teamEvents = sortCoreEvents(timeline.events.core_team ?? []);

  let playerEventIndex = 0;
  let teamEventIndex = 0;
  const players = new Map<string, CorePlayerStats>();
  let teamZero = defaultCoreTeamStats();
  let teamOne = defaultCoreTeamStats();

  for (const frame of timeline.frames) {
    while (
      playerEventIndex < playerEvents.length &&
      playerEvents[playerEventIndex]!.frame <= frame.frame_number
    ) {
      const event = playerEvents[playerEventIndex] as CorePlayerStatsEvent;
      players.set(remoteIdKey(event.player), event.stats);
      playerEventIndex += 1;
    }

    while (
      teamEventIndex < teamEvents.length &&
      teamEvents[teamEventIndex]!.frame <= frame.frame_number
    ) {
      const event = teamEvents[teamEventIndex] as CoreTeamStatsEvent;
      if (event.is_team_0) {
        teamZero = event.stats;
      } else {
        teamOne = event.stats;
      }
      teamEventIndex += 1;
    }

    assignCoreTeamStats(frame.team_zero.core, teamZero);
    assignCoreTeamStats(frame.team_one.core, teamOne);
    for (const player of frame.players) {
      assignCorePlayerStats(player.core, players.get(remoteIdKey(player.player_id)));
    }
  }

  return timeline;
}
