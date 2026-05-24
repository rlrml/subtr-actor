import type {
  PlayerStatsSnapshot,
  StatsEvents,
  StatsFrame,
  StatsTimeline,
  TeamStatsSnapshot,
} from "./statsTimeline.ts";
import {
  createPlayerStatsSnapshot,
  createTeamStatsSnapshot,
  merge,
  type DeepPartial,
} from "./statsSnapshotFactories.ts";
export {
  createPlayerStatsSnapshot,
  createTeamStatsSnapshot,
} from "./statsSnapshotFactories.ts";

export function createStatsEvents(overrides?: DeepPartial<StatsEvents>): StatsEvents {
  return merge<StatsEvents>({
    timeline: [],
    mechanics: [],
    goal_context: [],
    backboard: [],
    ceiling_shot: [],
    double_tap: [],
    fifty_fifty: [],
    one_timer: [],
    pass: [],
    goal_tags: [],
    rush: [],
    speed_flip: [],
    half_flip: [],
    wavedash: [],
    whiff: [],
    boost_pickups: [],
    bump: [],
  }, overrides);
}

export function createTouchStats(
  overrides?: DeepPartial<PlayerStatsSnapshot["touch"]>,
): PlayerStatsSnapshot["touch"] {
  return createPlayerStatsSnapshot({ touch: overrides }).touch;
}

export function createWhiffStats(
  overrides?: DeepPartial<PlayerStatsSnapshot["whiff"]>,
): PlayerStatsSnapshot["whiff"] {
  return createPlayerStatsSnapshot({ whiff: overrides }).whiff;
}

export function createPositioningStats(
  overrides?: DeepPartial<PlayerStatsSnapshot["positioning"]>,
): PlayerStatsSnapshot["positioning"] {
  return createPlayerStatsSnapshot({ positioning: overrides }).positioning;
}

export function createStatsFrame(overrides?: DeepPartial<StatsFrame>): StatsFrame {
  const merged = merge<StatsFrame>({
    frame_number: 0,
    time: 0,
    dt: 0,
    seconds_remaining: null,
    game_state: null,
    gameplay_phase: "unknown",
    is_live_play: true,
    team_zero: createTeamStatsSnapshot(),
    team_one: createTeamStatsSnapshot(),
    players: [],
  }, overrides);

  merged.team_zero = createTeamStatsSnapshot(merged.team_zero as DeepPartial<TeamStatsSnapshot>);
  merged.team_one = createTeamStatsSnapshot(merged.team_one as DeepPartial<TeamStatsSnapshot>);
  merged.players = merged.players.map((player) =>
    createPlayerStatsSnapshot(player as DeepPartial<PlayerStatsSnapshot>));
  return merged;
}

export function createStatsTimeline(overrides?: DeepPartial<StatsTimeline>): StatsTimeline {
  const merged = merge<StatsTimeline>({
    config: {
      most_back_forward_threshold_y: 0,
      level_ball_depth_margin: 0,
      pressure_neutral_zone_half_width_y: 0,
      rotation_role_depth_margin: 0,
      rotation_first_man_ambiguity_margin: 0,
      rotation_first_man_debounce_seconds: 0,
      rush_max_start_y: 0,
      rush_attack_support_distance_y: 0,
      rush_defender_distance_y: 0,
      rush_min_possession_retained_seconds: 0,
      aerial_goal_min_ball_z: 0,
      high_aerial_goal_min_ball_z: 0,
      long_distance_goal_max_attacking_y: 0,
      own_half_goal_max_attacking_y: 0,
      empty_net_min_defender_y_margin: 0,
      empty_net_min_defender_distance: 0,
      empty_net_max_touch_attacking_y: 0,
      flick_goal_max_event_to_goal_seconds: 0,
      one_timer_goal_max_event_to_goal_seconds: 0,
      air_dribble_goal_max_end_to_goal_seconds: 0,
      flip_reset_goal_max_event_to_goal_seconds: 0,
    },
    replay_meta: {
      team_zero: [],
      team_one: [],
      all_headers: [],
    },
    events: createStatsEvents(),
    frames: [],
  }, overrides);

  merged.events = createStatsEvents(merged.events as DeepPartial<StatsEvents>);
  merged.frames = merged.frames.map((frame) => createStatsFrame(frame as DeepPartial<StatsFrame>));
  return merged;
}

export function createLegacyStatsTimeline(overrides: DeepPartial<StatsTimeline> & {
  timeline_events?: StatsEvents["timeline"];
  backboard_events?: StatsEvents["backboard"];
  ceiling_shot_events?: StatsEvents["ceiling_shot"];
  double_tap_events?: StatsEvents["double_tap"];
  fifty_fifty_events?: StatsEvents["fifty_fifty"];
  one_timer_events?: StatsEvents["one_timer"];
  pass_events?: StatsEvents["pass"];
  goal_tag_events?: StatsEvents["goal_tags"];
  mechanic_events?: StatsEvents["mechanics"];
  rush_events?: StatsEvents["rush"];
  speed_flip_events?: StatsEvents["speed_flip"];
  half_flip_events?: StatsEvents["half_flip"];
  wavedash_events?: StatsEvents["wavedash"];
  whiff_events?: StatsEvents["whiff"];
  boost_pickups?: StatsEvents["boost_pickups"];
} = {}): StatsTimeline {
  return createStatsTimeline({
    ...overrides,
    events: {
      ...(overrides.events ?? {}),
      timeline: overrides.timeline_events ?? overrides.events?.timeline ?? [],
      mechanics: overrides.mechanic_events ?? overrides.events?.mechanics ?? [],
      backboard: overrides.backboard_events ?? overrides.events?.backboard ?? [],
      ceiling_shot: overrides.ceiling_shot_events ?? overrides.events?.ceiling_shot ?? [],
      double_tap: overrides.double_tap_events ?? overrides.events?.double_tap ?? [],
      fifty_fifty: overrides.fifty_fifty_events ?? overrides.events?.fifty_fifty ?? [],
      one_timer: overrides.one_timer_events ?? overrides.events?.one_timer ?? [],
      pass: overrides.pass_events ?? overrides.events?.pass ?? [],
      goal_tags: overrides.goal_tag_events ?? overrides.events?.goal_tags ?? [],
      rush: overrides.rush_events ?? overrides.events?.rush ?? [],
      speed_flip: overrides.speed_flip_events ?? overrides.events?.speed_flip ?? [],
      half_flip: overrides.half_flip_events ?? overrides.events?.half_flip ?? [],
      wavedash: overrides.wavedash_events ?? overrides.events?.wavedash ?? [],
      whiff: overrides.whiff_events ?? overrides.events?.whiff ?? [],
      boost_pickups: overrides.boost_pickups ?? overrides.events?.boost_pickups ?? [],
    },
  });
}
