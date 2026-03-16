export interface StatsTimeline {
  config?: {
    most_back_forward_threshold_y: number;
    [key: string]: unknown;
  };
  replay_meta: unknown;
  timeline_events: unknown[];
  frames: StatsFrame[];
}

export interface StatsFrame {
  frame_number: number;
  time: number;
  dt: number;
  possession?: {
    tracked_time: number;
    team_zero_time: number;
    team_one_time: number;
    [key: string]: unknown;
  };
  players: PlayerStatsSnapshot[];
  [key: string]: unknown;
}

export interface PlayerStatsSnapshot {
  player_id: Record<string, string>;
  name: string;
  is_team_0: boolean;
  core?: {
    score: number;
    goals: number;
    assists: number;
    saves: number;
    shots: number;
    goals_conceded_while_last_defender: number;
    [key: string]: unknown;
  };
  ball_carry?: {
    carry_count: number;
    total_carry_time: number;
    total_straight_line_distance: number;
    total_path_distance: number;
    longest_carry_time: number;
    furthest_carry_distance: number;
    fastest_carry_speed: number;
    carry_speed_sum: number;
    average_horizontal_gap_sum: number;
    average_vertical_gap_sum: number;
    [key: string]: unknown;
  };
  positioning?: {
    active_game_time: number;
    time_defensive_third: number;
    time_neutral_third: number;
    time_offensive_third: number;
    time_defensive_half: number;
    time_offensive_half: number;
    time_demolished: number;
    time_no_teammates: number;
    time_most_back: number;
    time_most_forward: number;
    time_other_role: number;
    time_even: number;
    [key: string]: unknown;
  };
  boost?: {
    amount_collected: number;
    amount_collected_big: number;
    amount_collected_small: number;
    amount_respawned: number;
    overfill_total: number;
    overfill_from_stolen: number;
    amount_used: number;
    amount_used_while_grounded: number;
    amount_used_while_airborne: number;
    amount_stolen: number;
    big_pads_collected: number;
    small_pads_collected: number;
    amount_used_while_supersonic: number;
    time_zero_boost: number;
    time_hundred_boost: number;
    boost_integral: number;
    tracked_time: number;
    [key: string]: unknown;
  };
  movement?: {
    tracked_time: number;
    total_distance: number;
    speed_integral: number;
    time_slow_speed: number;
    time_boost_speed: number;
    time_supersonic_speed: number;
    time_on_ground: number;
    time_low_air: number;
    time_high_air: number;
    [key: string]: unknown;
  };
  powerslide?: {
    total_duration: number;
    press_count: number;
    [key: string]: unknown;
  };
  demo?: {
    demos_inflicted: number;
    demos_taken: number;
    [key: string]: unknown;
  };
  [key: string]: unknown;
}

export function createStatsFrameLookup(statsTimeline: StatsTimeline): Map<number, StatsFrame> {
  return new Map(statsTimeline.frames.map((frame) => [frame.frame_number, frame]));
}

export function getStatsFrameForReplayFrame(
  statsFrameLookup: Map<number, StatsFrame>,
  replayFrameNumber: number,
): StatsFrame | null {
  return statsFrameLookup.get(replayFrameNumber) ?? null;
}
