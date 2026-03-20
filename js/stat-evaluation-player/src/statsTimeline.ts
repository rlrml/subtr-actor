export interface StatsTimeline {
  config?: {
    most_back_forward_threshold_y: number;
    pressure_neutral_zone_half_width_y?: number;
    rush_max_start_y?: number;
    rush_attack_support_distance_y?: number;
    rush_defender_distance_y?: number;
    rush_min_possession_retained_seconds?: number;
    [key: string]: unknown;
  };
  replay_meta: unknown;
  timeline_events: unknown[];
  fifty_fifty_events?: FiftyFiftyEvent[];
  rush_events?: RushEvent[];
  speed_flip_events?: SpeedFlipEvent[];
  frames: StatsFrame[];
}

export interface SpeedFlipEvent {
  time: number;
  frame: number;
  player?: Record<string, string>;
  is_team_0: boolean;
  time_since_kickoff_start: number;
  start_position: [number, number, number];
  end_position: [number, number, number];
  start_speed: number;
  max_speed: number;
  best_alignment: number;
  diagonal_score: number;
  cancel_score: number;
  speed_score: number;
  confidence: number;
}

export interface FiftyFiftyEvent {
  start_time: number;
  start_frame: number;
  resolve_time: number;
  resolve_frame: number;
  is_kickoff: boolean;
  team_zero_player?: Record<string, string>;
  team_one_player?: Record<string, string>;
  team_zero_position: [number, number, number];
  team_one_position: [number, number, number];
  midpoint: [number, number, number];
  plane_normal: [number, number, number];
  winning_team_is_team_0?: boolean;
  possession_team_is_team_0?: boolean;
}

export interface RushEvent {
  start_time: number;
  start_frame: number;
  end_time: number;
  end_frame: number;
  is_team_0: boolean;
  attackers: number;
  defenders: number;
}

export interface LabeledCountEntry {
  labels: StatLabel[];
  count: number;
}

export interface LabeledCounts {
  entries: LabeledCountEntry[];
}

export interface LabeledFloatSumEntry {
  labels: StatLabel[];
  value: number;
}

export interface LabeledFloatSums {
  entries: LabeledFloatSumEntry[];
}

export interface StatsFrame {
  frame_number: number;
  time: number;
  dt: number;
  fifty_fifty?: {
    count: number;
    team_zero_wins: number;
    team_one_wins: number;
    neutral_outcomes: number;
    kickoff_count: number;
    kickoff_team_zero_wins: number;
    kickoff_team_one_wins: number;
    kickoff_neutral_outcomes: number;
    team_zero_possession_after_count: number;
    team_one_possession_after_count: number;
    neutral_possession_after_count: number;
    kickoff_team_zero_possession_after_count?: number;
    kickoff_team_one_possession_after_count?: number;
    kickoff_neutral_possession_after_count?: number;
    [key: string]: unknown;
  };
  possession?: {
    tracked_time: number;
    team_zero_time: number;
    team_one_time: number;
    neutral_time?: number;
    labeled_time?: LabeledFloatSums;
    [key: string]: unknown;
  };
  pressure?: {
    tracked_time: number;
    team_zero_side_time: number;
    team_one_side_time: number;
    neutral_time?: number;
    labeled_time?: LabeledFloatSums;
    [key: string]: unknown;
  } | null;
  rush?: {
    team_zero_count: number;
    team_zero_two_v_one_count: number;
    team_zero_two_v_two_count: number;
    team_zero_two_v_three_count: number;
    team_zero_three_v_one_count: number;
    team_zero_three_v_two_count: number;
    team_zero_three_v_three_count: number;
    team_one_count: number;
    team_one_two_v_one_count: number;
    team_one_two_v_two_count: number;
    team_one_two_v_three_count: number;
    team_one_three_v_one_count: number;
    team_one_three_v_two_count: number;
    team_one_three_v_three_count: number;
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
  fifty_fifty?: {
    count: number;
    wins: number;
    losses: number;
    neutral_outcomes: number;
    kickoff_count: number;
    kickoff_wins: number;
    kickoff_losses: number;
    kickoff_neutral_outcomes: number;
    possession_after_count: number;
    kickoff_possession_after_count: number;
    [key: string]: unknown;
  };
  speed_flip?: {
    count: number;
    high_confidence_count: number;
    is_last_speed_flip: boolean;
    last_speed_flip_time?: number;
    last_speed_flip_frame?: number;
    time_since_last_speed_flip?: number;
    frames_since_last_speed_flip?: number;
    last_quality?: number;
    average_quality?: number;
    best_quality?: number;
    [key: string]: unknown;
  };
  touch?: {
    touch_count: number;
    dribble_touch_count?: number;
    control_touch_count?: number;
    medium_hit_count?: number;
    hard_hit_count?: number;
    aerial_touch_count?: number;
    high_aerial_touch_count?: number;
    labeled_touch_counts?: LabeledCounts;
    is_last_touch: boolean;
    last_touch_time?: number;
    last_touch_frame?: number;
    time_since_last_touch?: number;
    frames_since_last_touch?: number;
    last_ball_speed_change?: number;
    average_ball_speed_change?: number;
    max_ball_speed_change?: number;
    [key: string]: unknown;
  };
  musty_flick?: {
    count: number;
    aerial_count?: number;
    high_confidence_count?: number;
    is_last_musty: boolean;
    last_musty_time?: number;
    last_musty_frame?: number;
    time_since_last_musty?: number;
    frames_since_last_musty?: number;
    last_confidence?: number;
    average_confidence?: number;
    best_confidence?: number;
    [key: string]: unknown;
  };
  dodge_reset?: {
    count: number;
    on_ball_count: number;
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
    time_defensive_third?: number;
    time_neutral_third?: number;
    time_offensive_third?: number;
    time_defensive_zone?: number;
    time_neutral_zone?: number;
    time_offensive_zone?: number;
    time_defensive_half: number;
    time_offensive_half: number;
    time_demolished: number;
    time_no_teammates: number;
    time_most_back: number;
    time_most_forward: number;
    time_mid_role: number;
    time_other_role: number;
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
    labeled_tracked_time?: LabeledFloatSums;
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

export interface StatLabel {
  key: string;
  value: string;
}

export interface ExportedStat {
  domain: string;
  name: string;
  variant: string;
  unit: string;
  labels?: StatLabel[];
  value_type: "float" | "unsigned" | "signed";
  value: number;
}

export interface DynamicStatsFrame {
  frame_number: number;
  time: number;
  dt: number;
  possession?: ExportedStat[];
  pressure?: ExportedStat[];
  rush?: ExportedStat[];
  players: DynamicPlayerStatsSnapshot[];
  [key: string]: unknown;
}

export interface DynamicPlayerStatsSnapshot {
  player_id: Record<string, string>;
  name: string;
  is_team_0: boolean;
  stats: ExportedStat[];
  [key: string]: unknown;
}

export interface DynamicStatsTimeline {
  config?: {
    most_back_forward_threshold_y: number;
    pressure_neutral_zone_half_width_y?: number;
    rush_max_start_y?: number;
    rush_attack_support_distance_y?: number;
    rush_defender_distance_y?: number;
    rush_min_possession_retained_seconds?: number;
    [key: string]: unknown;
  };
  replay_meta: unknown;
  timeline_events: unknown[];
  rush_events?: RushEvent[];
  frames: DynamicStatsFrame[];
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

export function createDynamicStatsFrameLookup(
  statsTimeline: DynamicStatsTimeline,
): Map<number, DynamicStatsFrame> {
  return new Map(statsTimeline.frames.map((frame) => [frame.frame_number, frame]));
}

export function getDynamicStatsFrameForReplayFrame(
  statsFrameLookup: Map<number, DynamicStatsFrame>,
  replayFrameNumber: number,
): DynamicStatsFrame | null {
  return statsFrameLookup.get(replayFrameNumber) ?? null;
}
