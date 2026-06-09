import type { CorePlayerStats } from "./generated/CorePlayerStats.ts";
import type { CorePlayerScoreboardEvent } from "./generated/CorePlayerScoreboardEvent.ts";
import type { GoalContextEvent } from "./generated/GoalContextEvent.ts";
import type { GoalContextPosition } from "./generated/GoalContextPosition.ts";
import type { GoalPlayerContext } from "./generated/GoalPlayerContext.ts";
import type { CoreTeamStats } from "./generated/CoreTeamStats.ts";
import type { StatsFrame, MaterializedStatsTimeline } from "./statsTimeline.ts";
import { statsEventPayloads } from "./statsTimeline.ts";

const GOAL_CAUGHT_AHEAD_MAX_BALL_Y = -1200;
const GOAL_CAUGHT_AHEAD_MIN_PLAYER_Y = -250;
const GOAL_CAUGHT_AHEAD_MIN_BALL_DELTA_Y = 2200;

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
    caught_ahead_of_play_on_conceded_goals: 0,
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

function applyCorePlayerScoreboardEvent(
  stats: CorePlayerStats,
  event: CorePlayerScoreboardEvent,
): void {
  stats.score += event.score_delta;
  stats.goals += event.goals_delta;
  stats.assists += event.assists_delta;
  stats.saves += event.saves_delta;
  stats.shots += event.shots_delta;
}

type ScoringGoalContext = Pick<
  GoalContextEvent,
  "time_after_kickoff" | "goal_buildup" | "ball_air_time_before_goal"
>;

function applyScoringGoalContextToTeam(
  stats: CoreTeamStats,
  event: ScoringGoalContext,
): void {
  if (event.time_after_kickoff != null) {
    const time = Math.max(0, event.time_after_kickoff);
    if (time < 10) {
      stats.kickoff_goal_count += 1;
    } else if (time < 20) {
      stats.short_goal_count += 1;
    } else if (time < 40) {
      stats.medium_goal_count += 1;
    } else {
      stats.long_goal_count += 1;
    }
  }
  if (event.goal_buildup === "counter_attack") {
    stats.counter_attack_goal_count += 1;
  } else if (event.goal_buildup === "sustained_pressure") {
    stats.sustained_pressure_goal_count += 1;
  } else if (event.goal_buildup != null) {
    stats.other_buildup_goal_count += 1;
  }
  if (event.ball_air_time_before_goal != null) {
    const airTime = Math.max(0, event.ball_air_time_before_goal);
    stats.goal_ball_air_time_sample_count += 1;
    stats.cumulative_goal_ball_air_time = addF32(stats.cumulative_goal_ball_air_time, airTime);
    stats.last_goal_ball_air_time = airTime;
  }
}

function samePlayer(left: unknown, right: unknown): boolean {
  return left != null && right != null && remoteIdKey(left) === remoteIdKey(right);
}

function normalizedY(isTeam0: boolean, position: GoalContextPosition): number {
  return isTeam0 ? position.y : -position.y;
}

function isCaughtAheadOnConcededGoal(
  goal: GoalContextEvent,
  player: GoalPlayerContext,
): boolean {
  if (goal.ball_position == null || player.position == null) {
    return false;
  }
  const defendingTeamIsTeam0 = !goal.scoring_team_is_team_0;
  const normalizedBallY = normalizedY(defendingTeamIsTeam0, goal.ball_position);
  if (normalizedBallY > GOAL_CAUGHT_AHEAD_MAX_BALL_Y) {
    return false;
  }
  const normalizedPlayerY = normalizedY(defendingTeamIsTeam0, player.position);
  if (normalizedPlayerY < GOAL_CAUGHT_AHEAD_MIN_PLAYER_Y) {
    return false;
  }
  return normalizedPlayerY - normalizedBallY >= GOAL_CAUGHT_AHEAD_MIN_BALL_DELTA_Y;
}

function applyGoalPlayerContext(
  stats: CorePlayerStats,
  goal: GoalContextEvent,
  player: GoalPlayerContext,
): void {
  const isScoringTeam = player.is_team_0 === goal.scoring_team_is_team_0;
  const isDefendingMostBack = samePlayer(player.player, goal.defending_team_most_back_player);
  const isScoringMostBack = samePlayer(player.player, goal.scoring_team_most_back_player);
  if (isDefendingMostBack) {
    stats.goals_conceded_while_last_defender += 1;
  }
  if (isScoringMostBack) {
    stats.goals_for_while_most_back += 1;
  }
  if (isDefendingMostBack) {
    stats.goals_against_while_most_back += 1;
  }
  if (!isScoringTeam && isCaughtAheadOnConcededGoal(goal, player)) {
    stats.caught_ahead_of_play_on_conceded_goals += 1;
  }
  if (!isScoringTeam && player.boost_amount != null) {
    stats.goal_against_boost_sample_count += 1;
    stats.cumulative_boost_on_goals_against = addF32(
      stats.cumulative_boost_on_goals_against,
      player.boost_amount,
    );
    stats.last_boost_on_goal_against = player.boost_amount;
  }
  if (
    !isScoringTeam &&
    player.average_boost_in_leadup != null &&
    player.min_boost_in_leadup != null
  ) {
    stats.goal_against_boost_leadup_sample_count += 1;
    stats.cumulative_average_boost_in_goal_against_leadup = addF32(
      stats.cumulative_average_boost_in_goal_against_leadup,
      player.average_boost_in_leadup,
    );
    stats.cumulative_min_boost_in_goal_against_leadup = addF32(
      stats.cumulative_min_boost_in_goal_against_leadup,
      player.min_boost_in_leadup,
    );
    stats.last_average_boost_in_goal_against_leadup = player.average_boost_in_leadup;
    stats.last_min_boost_in_goal_against_leadup = player.min_boost_in_leadup;
  }
  if (!isScoringTeam && player.position != null) {
    stats.goal_against_position_sample_count += 1;
    stats.cumulative_goal_against_position_x = addF32(
      stats.cumulative_goal_against_position_x,
      player.position.x,
    );
    stats.cumulative_goal_against_position_y = addF32(
      stats.cumulative_goal_against_position_y,
      player.position.y,
    );
    stats.cumulative_goal_against_position_z = addF32(
      stats.cumulative_goal_against_position_z,
      player.position.z,
    );
    stats.last_goal_against_position = { ...player.position };
  }
  const isScorer = samePlayer(player.player, goal.scorer);
  const scoringGoalLastTouchPosition = isScorer ? goal.scorer_last_touch?.ball_position : null;
  if (scoringGoalLastTouchPosition != null) {
    stats.scoring_goal_last_touch_position_sample_count += 1;
    stats.cumulative_scoring_goal_last_touch_position_x = addF32(
      stats.cumulative_scoring_goal_last_touch_position_x,
      scoringGoalLastTouchPosition.x,
    );
    stats.cumulative_scoring_goal_last_touch_position_y = addF32(
      stats.cumulative_scoring_goal_last_touch_position_y,
      scoringGoalLastTouchPosition.y,
    );
    stats.cumulative_scoring_goal_last_touch_position_z = addF32(
      stats.cumulative_scoring_goal_last_touch_position_z,
      scoringGoalLastTouchPosition.z,
    );
    stats.last_scoring_goal_last_touch_position = { ...scoringGoalLastTouchPosition };
  }
  if (isScorer) {
    applyScoringGoalContextToTeam(stats, goal);
  }
}

function applyGoalContextEvent(
  players: Map<string, CorePlayerStats>,
  teamZero: CoreTeamStats,
  teamOne: CoreTeamStats,
  goal: GoalContextEvent,
): void {
  applyScoringGoalContextToTeam(goal.scoring_team_is_team_0 ? teamZero : teamOne, goal);
  for (const player of goal.players) {
    const playerKey = remoteIdKey(player.player);
    const stats = players.get(playerKey) ?? defaultCorePlayerStats();
    players.set(playerKey, stats);
    applyGoalPlayerContext(stats, goal, player);
  }
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
  const playerEvents = sortCoreEvents(statsEventPayloads(timeline, "core_player"));
  const goalContextEvents = sortCoreEvents(statsEventPayloads(timeline, "goal_context"));

  let playerEventIndex = 0;
  let goalContextEventIndex = 0;
  const players = new Map<string, CorePlayerStats>();
  const teamZero = defaultCoreTeamStats();
  const teamOne = defaultCoreTeamStats();

  return {
    applyFrame(frame: StatsFrame): void {
      while (
        playerEventIndex < playerEvents.length &&
        playerEvents[playerEventIndex]!.frame <= frame.frame_number
      ) {
        const event = playerEvents[playerEventIndex] as CorePlayerScoreboardEvent;
        const playerKey = remoteIdKey(event.player);
        const stats = players.get(playerKey) ?? defaultCorePlayerStats();
        players.set(playerKey, stats);
        applyCorePlayerScoreboardEvent(stats, event);
        const teamStats = event.is_team_0 ? teamZero : teamOne;
        applyCorePlayerScoreboardEvent(teamStats as CorePlayerStats, event);
        playerEventIndex += 1;
      }

      while (
        goalContextEventIndex < goalContextEvents.length &&
        goalContextEvents[goalContextEventIndex]!.frame <= frame.frame_number
      ) {
        const event = goalContextEvents[goalContextEventIndex] as GoalContextEvent;
        applyGoalContextEvent(players, teamZero, teamOne, event);
        goalContextEventIndex += 1;
      }

      assignCoreTeamStats(frame.team_zero.core, teamZero);
      assignCoreTeamStats(frame.team_one.core, teamOne);
      for (const player of frame.players) {
        assignCorePlayerStats(player.core, players.get(remoteIdKey(player.player_id)));
      }
    },
  };
}
