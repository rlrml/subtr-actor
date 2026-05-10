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
    backboard: [],
    ceiling_shot: [],
    double_tap: [],
    fifty_fifty: [],
    rush: [],
    speed_flip: [],
    boost_pickups: [],
  }, overrides);
}

export function createTouchStats(
  overrides?: DeepPartial<PlayerStatsSnapshot["touch"]>,
): PlayerStatsSnapshot["touch"] {
  return createPlayerStatsSnapshot({ touch: overrides }).touch;
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
      rush_max_start_y: 0,
      rush_attack_support_distance_y: 0,
      rush_defender_distance_y: 0,
      rush_min_possession_retained_seconds: 0,
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
  rush_events?: StatsEvents["rush"];
  speed_flip_events?: StatsEvents["speed_flip"];
  boost_pickups?: StatsEvents["boost_pickups"];
} = {}): StatsTimeline {
  return createStatsTimeline({
    ...overrides,
    events: {
      ...(overrides.events ?? {}),
      timeline: overrides.timeline_events ?? overrides.events?.timeline ?? [],
      backboard: overrides.backboard_events ?? overrides.events?.backboard ?? [],
      ceiling_shot: overrides.ceiling_shot_events ?? overrides.events?.ceiling_shot ?? [],
      double_tap: overrides.double_tap_events ?? overrides.events?.double_tap ?? [],
      fifty_fifty: overrides.fifty_fifty_events ?? overrides.events?.fifty_fifty ?? [],
      rush: overrides.rush_events ?? overrides.events?.rush ?? [],
      speed_flip: overrides.speed_flip_events ?? overrides.events?.speed_flip ?? [],
      boost_pickups: overrides.boost_pickups ?? overrides.events?.boost_pickups ?? [],
    },
  });
}
