import type {
  Event,
  EventPayload,
  PlayerStatsSnapshot,
  StatsEventPayload,
  StatsEventPayloadKind,
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
      events: [],
    },
    overrides,
  );
}

type PayloadList<K extends StatsEventPayloadKind> = Array<StatsEventPayload<K>>;

function titleCaseStream(stream: string): string {
  return stream
    .split("_")
    .filter(Boolean)
    .map((part) => `${part.slice(0, 1).toUpperCase()}${part.slice(1)}`)
    .join(" ");
}

function eventTiming(payload: Record<string, unknown>): Event["meta"]["timing"] {
  const startFrame = payload.start_frame;
  const endFrame = payload.end_frame ?? payload.resolve_frame;
  const startTime = payload.start_time;
  const endTime = payload.end_time ?? payload.resolve_time;
  if (
    typeof startFrame === "number" &&
    typeof endFrame === "number" &&
    typeof startTime === "number" &&
    typeof endTime === "number"
  ) {
    return {
      type: "span",
      start_frame: startFrame,
      end_frame: endFrame,
      start_time: startTime,
      end_time: endTime,
    };
  }
  return {
    type: "moment",
    frame: typeof payload.frame === "number" ? payload.frame : 0,
    time: typeof payload.time === "number" ? payload.time : 0,
  };
}

function payloadEvent<K extends StatsEventPayloadKind>(
  stream: string,
  kind: K,
  payload: StatsEventPayload<K>,
  index: number,
): Event {
  const record = payload as Record<string, unknown>;
  const timing = eventTiming(record);
  const frameId =
    timing.type === "span" ? `${timing.start_frame}:${timing.end_frame}` : `${timing.frame}`;
  return {
    meta: {
      id: `${stream}:${frameId}:${index}`,
      stream,
      label: titleCaseStream(stream),
      timing,
      primary_player:
        (record.player as Event["meta"]["primary_player"]) ??
        (record.player_id as Event["meta"]["primary_player"]) ??
        (record.scorer as Event["meta"]["primary_player"]),
      secondary_player:
        (record.receiver as Event["meta"]["secondary_player"]) ??
        (record.victim as Event["meta"]["secondary_player"]),
      player_position:
        (record.player_position as Event["meta"]["player_position"]) ??
        (record.end_position as Event["meta"]["player_position"]),
      ball_position:
        (record.ball_position as Event["meta"]["ball_position"]) ??
        (record.end_ball_position as Event["meta"]["ball_position"]),
      team_is_team_0:
        (record.is_team_0 as Event["meta"]["team_is_team_0"]) ??
        (record.scoring_team_is_team_0 as Event["meta"]["team_is_team_0"]) ??
        (record.initiator_is_team_0 as Event["meta"]["team_is_team_0"]),
      confidence: record.confidence as Event["meta"]["confidence"],
      properties: [],
    },
    payload: { kind, payload } as EventPayload,
  };
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
        closest_to_ball_switch_margin: 0,
        closest_to_ball_switch_min_seconds: 0,
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
        ceiling_shot_goal_max_event_to_goal_seconds: 0,
        double_tap_goal_max_event_to_goal_seconds: 0,
        one_timer_goal_max_event_to_goal_seconds: 0,
        air_dribble_goal_max_end_to_goal_seconds: 0,
        flip_reset_goal_max_event_to_goal_seconds: 0,
        bump_goal_max_event_to_goal_seconds: 0,
        demo_goal_max_event_to_goal_seconds: 0,
        half_volley_max_bounce_to_touch_seconds: 0,
        half_volley_min_ball_speed: 0,
        half_volley_goal_max_touch_to_goal_seconds: 0,
        half_volley_goal_min_goal_alignment: 0,
      },
      replay_meta: {
        team_zero: [],
        team_one: [],
        all_headers: [],
        game_type: {
          game_type: "Unknown",
          header_match_type: null,
          playlist_id: null,
          match_type_class: null,
        },
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
    timeline_events?: PayloadList<"timeline">;
    backboard_events?: PayloadList<"backboard">;
    ceiling_shot_events?: PayloadList<"ceiling_shot">;
    wall_aerial_events?: PayloadList<"wall_aerial">;
    wall_aerial_shot_events?: PayloadList<"wall_aerial_shot">;
    center_events?: PayloadList<"center">;
    flick_events?: PayloadList<"flick">;
    musty_flick_events?: PayloadList<"musty_flick">;
    dodge_reset_events?: PayloadList<"dodge_reset">;
    double_tap_events?: PayloadList<"double_tap">;
    fifty_fifty_events?: PayloadList<"fifty_fifty">;
    one_timer_events?: PayloadList<"one_timer">;
    pass_events?: PayloadList<"pass">;
    ball_carry_events?: PayloadList<"ball_carry">;
    controlled_play_events?: PayloadList<"controlled_play">;
    rush_events?: PayloadList<"rush">;
    speed_flip_events?: PayloadList<"speed_flip">;
    half_flip_events?: PayloadList<"half_flip">;
    half_volley_events?: PayloadList<"half_volley">;
    wavedash_events?: PayloadList<"wavedash">;
    whiff_events?: PayloadList<"whiff">;
    powerslide_events?: PayloadList<"powerslide">;
    positioning_activity_events?: PayloadList<"positioning_activity">;
    positioning_possession_events?: PayloadList<"positioning_possession">;
    positioning_field_zone_events?: PayloadList<"positioning_field_zone">;
    positioning_ball_depth_events?: PayloadList<"positioning_ball_depth">;
    positioning_teammate_role_events?: PayloadList<"positioning_teammate_role">;
    positioning_ball_proximity_events?: PayloadList<"positioning_ball_proximity">;
    positioning_goal_context_events?: PayloadList<"positioning_goal_context">;
    rotation_player_events?: PayloadList<"rotation_player">;
    rotation_role_span_events?: PayloadList<"rotation_role_span">;
    rotation_depth_span_events?: PayloadList<"rotation_depth_span">;
    rotation_first_man_stint_events?: PayloadList<"rotation_first_man_stint">;
    rotation_team_events?: PayloadList<"rotation_team">;
    boost_pickups?: PayloadList<"boost_pickup">;
    boost_ledger?: PayloadList<"boost_ledger">;
    boost_bucket?: PayloadList<"boost_bucket">;
    boost_state?: PayloadList<"boost_state">;
    bump_events?: PayloadList<"bump">;
  } = {},
): MaterializedStatsTimeline {
  const events = [
    ...(overrides.events?.events ?? []),
    ...(overrides.timeline_events ?? []).map((event, index) =>
      payloadEvent("timeline", "timeline", event, index),
    ),
    ...(overrides.backboard_events ?? []).map((event, index) =>
      payloadEvent("backboard", "backboard", event, index),
    ),
    ...(overrides.ceiling_shot_events ?? []).map((event, index) =>
      payloadEvent("ceiling_shot", "ceiling_shot", event, index),
    ),
    ...(overrides.wall_aerial_events ?? []).map((event, index) =>
      payloadEvent("wall_aerial", "wall_aerial", event, index),
    ),
    ...(overrides.wall_aerial_shot_events ?? []).map((event, index) =>
      payloadEvent("wall_aerial_shot", "wall_aerial_shot", event, index),
    ),
    ...(overrides.center_events ?? []).map((event, index) =>
      payloadEvent("center", "center", event, index),
    ),
    ...(overrides.flick_events ?? []).map((event, index) =>
      payloadEvent("flick", "flick", event, index),
    ),
    ...(overrides.musty_flick_events ?? []).map((event, index) =>
      payloadEvent("musty_flick", "musty_flick", event, index),
    ),
    ...(overrides.dodge_reset_events ?? []).map((event, index) =>
      payloadEvent("dodge_reset", "dodge_reset", event, index),
    ),
    ...(overrides.double_tap_events ?? []).map((event, index) =>
      payloadEvent("double_tap", "double_tap", event, index),
    ),
    ...(overrides.fifty_fifty_events ?? []).map((event, index) =>
      payloadEvent("fifty_fifty", "fifty_fifty", event, index),
    ),
    ...(overrides.one_timer_events ?? []).map((event, index) =>
      payloadEvent("one_timer", "one_timer", event, index),
    ),
    ...(overrides.pass_events ?? []).map((event, index) =>
      payloadEvent("pass", "pass", event, index),
    ),
    ...(overrides.ball_carry_events ?? []).map((event, index) =>
      payloadEvent("ball_carry", "ball_carry", event, index),
    ),
    ...(overrides.controlled_play_events ?? []).map((event, index) =>
      payloadEvent("controlled_play", "controlled_play", event, index),
    ),
    ...(overrides.rush_events ?? []).map((event, index) =>
      payloadEvent("rush", "rush", event, index),
    ),
    ...(overrides.speed_flip_events ?? []).map((event, index) =>
      payloadEvent("speed_flip", "speed_flip", event, index),
    ),
    ...(overrides.half_flip_events ?? []).map((event, index) =>
      payloadEvent("half_flip", "half_flip", event, index),
    ),
    ...(overrides.half_volley_events ?? []).map((event, index) =>
      payloadEvent("half_volley", "half_volley", event, index),
    ),
    ...(overrides.wavedash_events ?? []).map((event, index) =>
      payloadEvent("wavedash", "wavedash", event, index),
    ),
    ...(overrides.whiff_events ?? []).map((event, index) =>
      payloadEvent("whiff", "whiff", event, index),
    ),
    ...(overrides.powerslide_events ?? []).map((event, index) =>
      payloadEvent("powerslide", "powerslide", event, index),
    ),
    ...(overrides.positioning_activity_events ?? []).map((event, index) =>
      payloadEvent("positioning_activity", "positioning_activity", event, index),
    ),
    ...(overrides.positioning_possession_events ?? []).map((event, index) =>
      payloadEvent("positioning_possession", "positioning_possession", event, index),
    ),
    ...(overrides.positioning_field_zone_events ?? []).map((event, index) =>
      payloadEvent("positioning_field_zone", "positioning_field_zone", event, index),
    ),
    ...(overrides.positioning_ball_depth_events ?? []).map((event, index) =>
      payloadEvent("positioning_ball_depth", "positioning_ball_depth", event, index),
    ),
    ...(overrides.positioning_teammate_role_events ?? []).map((event, index) =>
      payloadEvent("positioning_teammate_role", "positioning_teammate_role", event, index),
    ),
    ...(overrides.positioning_ball_proximity_events ?? []).map((event, index) =>
      payloadEvent("positioning_ball_proximity", "positioning_ball_proximity", event, index),
    ),
    ...(overrides.positioning_goal_context_events ?? []).map((event, index) =>
      payloadEvent("positioning_goal_context", "positioning_goal_context", event, index),
    ),
    ...(overrides.rotation_player_events ?? []).map((event, index) =>
      payloadEvent("rotation_player", "rotation_player", event, index),
    ),
    ...(overrides.rotation_role_span_events ?? []).map((event, index) =>
      payloadEvent("rotation_role_span", "rotation_role_span", event, index),
    ),
    ...(overrides.rotation_depth_span_events ?? []).map((event, index) =>
      payloadEvent("rotation_depth_span", "rotation_depth_span", event, index),
    ),
    ...(overrides.rotation_first_man_stint_events ?? []).map((event, index) =>
      payloadEvent("rotation_first_man_stint", "rotation_first_man_stint", event, index),
    ),
    ...(overrides.rotation_team_events ?? []).map((event, index) =>
      payloadEvent("rotation_team", "rotation_team", event, index),
    ),
    ...(overrides.boost_pickups ?? []).map((event, index) =>
      payloadEvent("boost_pickups", "boost_pickup", event, index),
    ),
    ...(overrides.boost_ledger ?? []).map((event, index) =>
      payloadEvent("boost_ledger", "boost_ledger", event, index),
    ),
    ...(overrides.boost_bucket ?? []).map((event, index) =>
      payloadEvent("boost_bucket", "boost_bucket", event, index),
    ),
    ...(overrides.boost_state ?? []).map((event, index) =>
      payloadEvent("boost_state", "boost_state", event, index),
    ),
    ...(overrides.bump_events ?? []).map((event, index) =>
      payloadEvent("bump", "bump", event, index),
    ),
  ];
  return createStatsTimeline({
    ...overrides,
    events: { events },
  });
}
