import type {
  PlayerStatsSnapshot,
  StatsEvents,
  StatsFrame,
  MaterializedStatsTimeline,
  TeamStatsSnapshot,
} from "./statsTimeline.ts";
import {
  createPlayerStatsSnapshot,
  createTeamStatsSnapshot,
  merge,
  type DeepPartial,
} from "./statsSnapshotFactories.ts";
export { createPlayerStatsSnapshot, createTeamStatsSnapshot } from "./statsSnapshotFactories.ts";

export function createStatsEvents(overrides?: DeepPartial<StatsEvents>): StatsEvents {
  return merge<StatsEvents>(
    {
      timeline: [],
      core_player: [],
      core_team: [],
      possession: [],
      pressure: [],
      territorial_pressure: [],
      movement: [],
      positioning: [],
      rotation_player: [],
      rotation_team: [],
      mechanics: [],
      goal_context: [],
      backboard: [],
      ceiling_shot: [],
      wall_aerial: [],
      wall_aerial_shot: [],
      center: [],
      flick: [],
      musty_flick: [],
      dodge_reset: [],
      double_tap: [],
      fifty_fifty: [],
      one_timer: [],
      pass: [],
      pass_last_completed: [],
      ball_carry: [],
      goal_tags: [],
      rush: [],
      speed_flip: [],
      half_flip: [],
      half_volley: [],
      wavedash: [],
      whiff: [],
      powerslide: [],
      touch: [],
      touch_ball_movement: [],
      touch_last_touch: [],
      boost_pickups: [],
      boost_ledger: [],
      boost_state: [],
      bump: [],
    },
    overrides,
  );
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
  const merged = merge<StatsFrame>(
    {
      frame_number: 0,
      time: 0,
      dt: 0,
      seconds_remaining: null,
      game_state: null,
      ball_has_been_hit: null,
      kickoff_countdown_time: null,
      gameplay_phase: "unknown",
      is_live_play: true,
      team_zero: createTeamStatsSnapshot(),
      team_one: createTeamStatsSnapshot(),
      players: [],
    },
    overrides,
  );

  merged.team_zero = createTeamStatsSnapshot(merged.team_zero as DeepPartial<TeamStatsSnapshot>);
  merged.team_one = createTeamStatsSnapshot(merged.team_one as DeepPartial<TeamStatsSnapshot>);
  merged.players = merged.players.map((player) =>
    createPlayerStatsSnapshot(player as DeepPartial<PlayerStatsSnapshot>),
  );
  return merged;
}

export function createStatsTimeline(
  overrides?: DeepPartial<MaterializedStatsTimeline>,
): MaterializedStatsTimeline {
  const merged = merge<MaterializedStatsTimeline>(
    {
      config: {
        most_back_forward_threshold_y: 0,
        level_ball_depth_margin: 0,
        pressure_neutral_zone_half_width_y: 0,
        territorial_pressure_neutral_zone_half_width_y: 0,
        territorial_pressure_min_establish_seconds: 0,
        territorial_pressure_min_establish_third_seconds: 0,
        territorial_pressure_relief_grace_seconds: 0,
        territorial_pressure_confirmed_relief_grace_seconds: 0,
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
        double_tap_goal_max_event_to_goal_seconds: 0,
        one_timer_goal_max_event_to_goal_seconds: 0,
        air_dribble_goal_max_end_to_goal_seconds: 0,
        flip_reset_goal_max_event_to_goal_seconds: 0,
        half_volley_max_bounce_to_touch_seconds: 0,
        half_volley_min_ball_speed: 0,
        half_volley_goal_max_touch_to_goal_seconds: 0,
        half_volley_goal_min_goal_alignment: 0,
      },
      replay_meta: {
        team_zero: [],
        team_one: [],
        all_headers: [],
      },
      events: createStatsEvents(),
      frames: [],
    },
    overrides,
  );

  merged.events = createStatsEvents(merged.events as DeepPartial<StatsEvents>);
  merged.frames = merged.frames.map((frame) => createStatsFrame(frame as DeepPartial<StatsFrame>));
  return merged;
}

export function createLegacyStatsTimeline(
  overrides: DeepPartial<MaterializedStatsTimeline> & {
    timeline_events?: StatsEvents["timeline"];
    backboard_events?: StatsEvents["backboard"];
    ceiling_shot_events?: StatsEvents["ceiling_shot"];
    wall_aerial_events?: StatsEvents["wall_aerial"];
    wall_aerial_shot_events?: StatsEvents["wall_aerial_shot"];
    center_events?: StatsEvents["center"];
    flick_events?: StatsEvents["flick"];
    musty_flick_events?: StatsEvents["musty_flick"];
    dodge_reset_events?: StatsEvents["dodge_reset"];
    double_tap_events?: StatsEvents["double_tap"];
    fifty_fifty_events?: StatsEvents["fifty_fifty"];
    one_timer_events?: StatsEvents["one_timer"];
    pass_events?: StatsEvents["pass"];
    ball_carry_events?: StatsEvents["ball_carry"];
    goal_tag_events?: StatsEvents["goal_tags"];
    mechanic_events?: StatsEvents["mechanics"];
    rush_events?: StatsEvents["rush"];
    speed_flip_events?: StatsEvents["speed_flip"];
    half_flip_events?: StatsEvents["half_flip"];
    half_volley_events?: StatsEvents["half_volley"];
    wavedash_events?: StatsEvents["wavedash"];
    whiff_events?: StatsEvents["whiff"];
    powerslide_events?: StatsEvents["powerslide"];
    positioning_events?: StatsEvents["positioning"];
    rotation_player_events?: StatsEvents["rotation_player"];
    rotation_team_events?: StatsEvents["rotation_team"];
    boost_pickups?: StatsEvents["boost_pickups"];
    boost_ledger?: StatsEvents["boost_ledger"];
    boost_state?: StatsEvents["boost_state"];
    bump_events?: StatsEvents["bump"];
  } = {},
): MaterializedStatsTimeline {
  return createStatsTimeline({
    ...overrides,
    events: {
      ...(overrides.events ?? {}),
      timeline: overrides.timeline_events ?? overrides.events?.timeline ?? [],
      mechanics: overrides.mechanic_events ?? overrides.events?.mechanics ?? [],
      backboard: overrides.backboard_events ?? overrides.events?.backboard ?? [],
      ceiling_shot: overrides.ceiling_shot_events ?? overrides.events?.ceiling_shot ?? [],
      wall_aerial: overrides.wall_aerial_events ?? overrides.events?.wall_aerial ?? [],
      wall_aerial_shot:
        overrides.wall_aerial_shot_events ?? overrides.events?.wall_aerial_shot ?? [],
      center: overrides.center_events ?? overrides.events?.center ?? [],
      flick: overrides.flick_events ?? overrides.events?.flick ?? [],
      musty_flick: overrides.musty_flick_events ?? overrides.events?.musty_flick ?? [],
      dodge_reset: overrides.dodge_reset_events ?? overrides.events?.dodge_reset ?? [],
      double_tap: overrides.double_tap_events ?? overrides.events?.double_tap ?? [],
      fifty_fifty: overrides.fifty_fifty_events ?? overrides.events?.fifty_fifty ?? [],
      one_timer: overrides.one_timer_events ?? overrides.events?.one_timer ?? [],
      pass: overrides.pass_events ?? overrides.events?.pass ?? [],
      ball_carry: overrides.ball_carry_events ?? overrides.events?.ball_carry ?? [],
      goal_tags: overrides.goal_tag_events ?? overrides.events?.goal_tags ?? [],
      rush: overrides.rush_events ?? overrides.events?.rush ?? [],
      speed_flip: overrides.speed_flip_events ?? overrides.events?.speed_flip ?? [],
      half_flip: overrides.half_flip_events ?? overrides.events?.half_flip ?? [],
      half_volley: overrides.half_volley_events ?? overrides.events?.half_volley ?? [],
      wavedash: overrides.wavedash_events ?? overrides.events?.wavedash ?? [],
      whiff: overrides.whiff_events ?? overrides.events?.whiff ?? [],
      powerslide: overrides.powerslide_events ?? overrides.events?.powerslide ?? [],
      positioning: overrides.positioning_events ?? overrides.events?.positioning ?? [],
      rotation_player: overrides.rotation_player_events ?? overrides.events?.rotation_player ?? [],
      rotation_team: overrides.rotation_team_events ?? overrides.events?.rotation_team ?? [],
      boost_pickups: overrides.boost_pickups ?? overrides.events?.boost_pickups ?? [],
      boost_ledger: overrides.boost_ledger ?? overrides.events?.boost_ledger ?? [],
      boost_state: overrides.boost_state ?? overrides.events?.boost_state ?? [],
      bump: overrides.bump_events ?? overrides.events?.bump ?? [],
    },
  });
}
