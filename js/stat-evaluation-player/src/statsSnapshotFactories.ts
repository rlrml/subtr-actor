import type {
  PlayerStatsSnapshot,
  TeamStatsSnapshot,
} from "./statsTimeline.ts";

export type DeepPartial<T> = {
  [K in keyof T]?: T[K] extends Array<infer U>
    ? Array<DeepPartial<U>>
    : T[K] extends object
      ? DeepPartial<T[K]>
      : T[K];
};

export function merge<T>(base: T, overrides: DeepPartial<T> | undefined): T {
  if (!overrides) {
    return base;
  }

  const result: Record<string, unknown> = { ...base as Record<string, unknown> };
  for (const [key, value] of Object.entries(overrides)) {
    if (Array.isArray(value)) {
      result[key] = value;
      continue;
    }

    const baseValue = result[key];
    if (
      value &&
      typeof value === "object" &&
      baseValue &&
      typeof baseValue === "object" &&
      !Array.isArray(baseValue)
    ) {
      result[key] = merge(
        baseValue as Record<string, unknown>,
        value as Record<string, unknown>,
      );
      continue;
    }

    result[key] = value;
  }

  return result as T;
}

export function createTeamStatsSnapshot(
  overrides?: DeepPartial<TeamStatsSnapshot>,
): TeamStatsSnapshot {
  return merge<TeamStatsSnapshot>({
    fifty_fifty: {
      count: 0,
      wins: 0,
      losses: 0,
      neutral_outcomes: 0,
      kickoff_count: 0,
      kickoff_wins: 0,
      kickoff_losses: 0,
      kickoff_neutral_outcomes: 0,
      possession_after_count: 0,
      opponent_possession_after_count: 0,
      neutral_possession_after_count: 0,
      kickoff_possession_after_count: 0,
      kickoff_opponent_possession_after_count: 0,
      kickoff_neutral_possession_after_count: 0,
    },
    possession: {
      tracked_time: 0,
      possession_time: 0,
      opponent_possession_time: 0,
      neutral_time: 0,
      labeled_time: { entries: [] },
    },
    pressure: {
      tracked_time: 0,
      defensive_half_time: 0,
      offensive_half_time: 0,
      neutral_time: 0,
      labeled_time: { entries: [] },
    },
    rush: {
      count: 0,
      two_v_one_count: 0,
      two_v_two_count: 0,
      two_v_three_count: 0,
      three_v_one_count: 0,
      three_v_two_count: 0,
      three_v_three_count: 0,
    },
    core: {
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
    },
    backboard: { count: 0 },
    double_tap: { count: 0 },
    ball_carry: {
      carry_count: 0,
      total_carry_time: 0,
      total_straight_line_distance: 0,
      total_path_distance: 0,
      longest_carry_time: 0,
      furthest_carry_distance: 0,
      fastest_carry_speed: 0,
      carry_speed_sum: 0,
      average_horizontal_gap_sum: 0,
      average_vertical_gap_sum: 0,
    },
    boost: {
      tracked_time: 0,
      boost_integral: 0,
      time_zero_boost: 0,
      time_hundred_boost: 0,
      time_boost_0_25: 0,
      time_boost_25_50: 0,
      time_boost_50_75: 0,
      time_boost_75_100: 0,
      amount_collected: 0,
      amount_collected_inactive: 0,
      big_pads_collected_inactive: 0,
      small_pads_collected_inactive: 0,
      amount_stolen: 0,
      big_pads_collected: 0,
      small_pads_collected: 0,
      big_pads_stolen: 0,
      small_pads_stolen: 0,
      amount_collected_big: 0,
      amount_stolen_big: 0,
      amount_collected_small: 0,
      amount_stolen_small: 0,
      amount_respawned: 0,
      overfill_total: 0,
      overfill_from_stolen: 0,
      amount_used: 0,
      amount_used_while_grounded: 0,
      amount_used_while_airborne: 0,
      amount_used_while_supersonic: 0,
    },
    movement: {
      tracked_time: 0,
      total_distance: 0,
      speed_integral: 0,
      time_slow_speed: 0,
      time_boost_speed: 0,
      time_supersonic_speed: 0,
      time_on_ground: 0,
      time_low_air: 0,
      time_high_air: 0,
      labeled_tracked_time: { entries: [] },
    },
    powerslide: {
      total_duration: 0,
      press_count: 0,
    },
    demo: {
      demos_inflicted: 0,
    },
  }, overrides);
}

export function createPlayerStatsSnapshot(
  overrides?: DeepPartial<PlayerStatsSnapshot>,
): PlayerStatsSnapshot {
  return merge<PlayerStatsSnapshot>({
    player_id: { Steam: "test-player" },
    name: "Test Player",
    is_team_0: true,
    core: {
      score: 0,
      goals: 0,
      assists: 0,
      saves: 0,
      shots: 0,
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
      kickoff_goal_count: 0,
      short_goal_count: 0,
      medium_goal_count: 0,
      long_goal_count: 0,
      counter_attack_goal_count: 0,
      sustained_pressure_goal_count: 0,
      other_buildup_goal_count: 0,
    },
    backboard: {
      count: 0,
      is_last_backboard: false,
      last_backboard_time: null,
      last_backboard_frame: null,
      time_since_last_backboard: null,
      frames_since_last_backboard: null,
    },
    ceiling_shot: {
      count: 0,
      high_confidence_count: 0,
      is_last_ceiling_shot: false,
      last_ceiling_shot_time: null,
      last_ceiling_shot_frame: null,
      time_since_last_ceiling_shot: null,
      frames_since_last_ceiling_shot: null,
      last_confidence: null,
      best_confidence: 0,
      cumulative_confidence: 0,
    },
    double_tap: {
      count: 0,
      is_last_double_tap: false,
      last_double_tap_time: null,
      last_double_tap_frame: null,
      time_since_last_double_tap: null,
      frames_since_last_double_tap: null,
    },
    fifty_fifty: {
      count: 0,
      wins: 0,
      losses: 0,
      neutral_outcomes: 0,
      kickoff_count: 0,
      kickoff_wins: 0,
      kickoff_losses: 0,
      kickoff_neutral_outcomes: 0,
      possession_after_count: 0,
      kickoff_possession_after_count: 0,
    },
    speed_flip: {
      count: 0,
      high_confidence_count: 0,
      is_last_speed_flip: false,
      last_speed_flip_time: null,
      last_speed_flip_frame: null,
      time_since_last_speed_flip: null,
      frames_since_last_speed_flip: null,
      last_quality: null,
      best_quality: 0,
      cumulative_quality: 0,
    },
    touch: {
      touch_count: 0,
      dribble_touch_count: 0,
      control_touch_count: 0,
      medium_hit_count: 0,
      hard_hit_count: 0,
      aerial_touch_count: 0,
      high_aerial_touch_count: 0,
      is_last_touch: false,
      last_touch_time: null,
      last_touch_frame: null,
      time_since_last_touch: null,
      frames_since_last_touch: null,
      last_ball_speed_change: null,
      max_ball_speed_change: 0,
      cumulative_ball_speed_change: 0,
      labeled_touch_counts: { entries: [] },
    },
    whiff: {
      whiff_count: 0,
      grounded_whiff_count: 0,
      aerial_whiff_count: 0,
      dodge_whiff_count: 0,
      is_last_whiff: false,
      last_whiff_time: null,
      last_whiff_frame: null,
      time_since_last_whiff: null,
      frames_since_last_whiff: null,
      last_closest_approach_distance: null,
      best_closest_approach_distance: null,
      cumulative_closest_approach_distance: 0,
    },
    flick: {
      count: 0,
      high_confidence_count: 0,
      is_last_flick: false,
      last_flick_time: null,
      last_flick_frame: null,
      time_since_last_flick: null,
      frames_since_last_flick: null,
      last_confidence: null,
      best_confidence: 0,
      cumulative_confidence: 0,
      cumulative_setup_duration: 0,
      cumulative_ball_speed_change: 0,
    },
    musty_flick: {
      count: 0,
      aerial_count: 0,
      high_confidence_count: 0,
      is_last_musty: false,
      last_musty_time: null,
      last_musty_frame: null,
      time_since_last_musty: null,
      frames_since_last_musty: null,
      last_confidence: null,
      best_confidence: 0,
      cumulative_confidence: 0,
    },
    dodge_reset: {
      count: 0,
      on_ball_count: 0,
    },
    ball_carry: {
      carry_count: 0,
      total_carry_time: 0,
      total_straight_line_distance: 0,
      total_path_distance: 0,
      longest_carry_time: 0,
      furthest_carry_distance: 0,
      fastest_carry_speed: 0,
      carry_speed_sum: 0,
      average_horizontal_gap_sum: 0,
      average_vertical_gap_sum: 0,
    },
    boost: createTeamStatsSnapshot().boost,
    movement: createTeamStatsSnapshot().movement,
    positioning: {
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
    },
    powerslide: {
      total_duration: 0,
      press_count: 0,
    },
    demo: {
      demos_inflicted: 0,
      demos_taken: 0,
    },
  }, overrides);
}
