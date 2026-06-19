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
import type { EventScope } from "./generated/EventScope.ts";
import {
  createPlayerStatsSnapshot,
  createTeamStatsSnapshot,
  merge,
  type DeepPartial,
} from "./statsSnapshotFactories.ts";
export { createPlayerStatsSnapshot, createTeamStatsSnapshot } from "./statsSnapshotFactories.ts";

export function createStatsEvents(overrides?: DeepPartial<StatsEvents>): StatsEvents {
  const merged = merge<StatsEvents>(
    {
      events: [],
    },
    overrides,
  );
  return {
    events: [
      ...(merged.events ?? []),
      ...legacyBucketEvents(overrides as Record<string, unknown> | undefined),
      ...legacyMechanicEvents(overrides as Record<string, unknown> | undefined),
    ],
  };
}

type PayloadList<K extends StatsEventPayloadKind> = Array<StatsEventPayload<K>>;

type LegacyEventBucket = {
  field: string;
  stream: string;
  kind: StatsEventPayloadKind;
};

const LEGACY_EVENT_BUCKETS: readonly LegacyEventBucket[] = [
  { field: "timeline", stream: "timeline", kind: "timeline" },
  { field: "core_player", stream: "core_player", kind: "core_player" },
  { field: "possession", stream: "possession", kind: "possession" },
  { field: "ball_half", stream: "ball_half", kind: "ball_half" },
  { field: "ball_third", stream: "ball_third", kind: "ball_third" },
  {
    field: "territorial_pressure",
    stream: "territorial_pressure",
    kind: "territorial_pressure",
  },
  { field: "movement", stream: "movement", kind: "movement" },
  {
    field: "player_activity",
    stream: "player_activity",
    kind: "player_activity",
  },
  { field: "field_third", stream: "field_third", kind: "field_third" },
  { field: "field_half", stream: "field_half", kind: "field_half" },
  { field: "ball_depth", stream: "ball_depth", kind: "ball_depth" },
  { field: "depth_role", stream: "depth_role", kind: "depth_role" },
  { field: "ball_proximity", stream: "ball_proximity", kind: "ball_proximity" },
  { field: "shadow_defense", stream: "shadow_defense", kind: "shadow_defense" },
  { field: "rotation_role", stream: "rotation_role", kind: "rotation_role" },
  { field: "first_man_change", stream: "first_man_change", kind: "first_man_change" },
  { field: "goal_context", stream: "goal_context", kind: "goal_context" },
  { field: "backboard", stream: "backboard", kind: "backboard" },
  { field: "ceiling_shot", stream: "ceiling_shot", kind: "ceiling_shot" },
  { field: "wall_aerial", stream: "wall_aerial", kind: "wall_aerial" },
  { field: "wall_aerial_shot", stream: "wall_aerial_shot", kind: "wall_aerial_shot" },
  { field: "center", stream: "center", kind: "center" },
  { field: "flick", stream: "flick", kind: "flick" },
  { field: "musty_flick", stream: "musty_flick", kind: "musty_flick" },
  { field: "dodge_reset", stream: "dodge_reset", kind: "dodge_reset" },
  { field: "double_tap", stream: "double_tap", kind: "double_tap" },
  { field: "fifty_fifty", stream: "fifty_fifty", kind: "fifty_fifty" },
  { field: "kickoff", stream: "kickoff", kind: "kickoff" },
  { field: "one_timer", stream: "one_timer", kind: "one_timer" },
  { field: "pass", stream: "pass", kind: "pass" },
  { field: "ball_carry", stream: "ball_carry", kind: "ball_carry" },
  { field: "controlled_play", stream: "controlled_play", kind: "controlled_play" },
  { field: "rush", stream: "rush", kind: "rush" },
  { field: "dodge", stream: "dodge", kind: "dodge" },
  { field: "speed_flip", stream: "speed_flip", kind: "speed_flip" },
  { field: "half_flip", stream: "half_flip", kind: "half_flip" },
  { field: "half_volley", stream: "half_volley", kind: "half_volley" },
  { field: "wavedash", stream: "wavedash", kind: "wavedash" },
  { field: "whiff", stream: "whiff", kind: "whiff" },
  { field: "powerslide", stream: "powerslide", kind: "powerslide" },
  { field: "touch", stream: "touch", kind: "touch" },
  { field: "boost_pickups", stream: "boost_pickups", kind: "boost_pickup" },
  { field: "boost_respawn", stream: "boost_respawn", kind: "respawn" },
  { field: "bump", stream: "bump", kind: "bump" },
  { field: "demolition", stream: "demolition", kind: "demolition" },
];

function legacyBucketEvents(record: Record<string, unknown> | undefined): Event[] {
  if (!record) {
    return [];
  }
  return LEGACY_EVENT_BUCKETS.flatMap(({ field, stream, kind }) => {
    const values = record[field];
    if (!Array.isArray(values)) {
      return [];
    }
    return values.map((event, index) =>
      payloadEvent(stream, kind, event as StatsEventPayload<typeof kind>, index),
    );
  });
}

function eventStreamScope(stream: string): EventScope {
  if (stream === "timeline" || stream === "goal_context" || stream === "core_player") {
    return "match";
  }
  if (
    stream === "possession" ||
    stream === "ball_half" ||
    stream === "territorial_pressure" ||
    stream === "controlled_play" ||
    stream === "fifty_fifty" ||
    stream === "rush"
  ) {
    return "team";
  }
  return "player";
}

function legacyMechanicEvents(record: Record<string, unknown> | undefined): Event[] {
  if (!record) {
    return [];
  }
  const values = [
    ...(Array.isArray(record.mechanics) ? record.mechanics : []),
    ...(Array.isArray(record.mechanic_events) ? record.mechanic_events : []),
  ];
  return values.map((event, index) => {
    const record = event as Record<string, unknown>;
    const stream = typeof record.kind === "string" ? record.kind : "mechanics";
    const timing = eventTiming(record);
    return {
      meta: {
        id: typeof record.id === "string" ? record.id : `${stream}:${index}`,
        stream,
        label: titleCaseStream(stream),
        scope: eventStreamScope(stream),
        timing,
        primary_player:
          (record.player as Event["meta"]["primary_player"]) ??
          (record.player_id as Event["meta"]["primary_player"]),
        secondary_player: record.secondary_player as Event["meta"]["secondary_player"],
        player_position: record.player_position as Event["meta"]["player_position"],
        ball_position: record.ball_position as Event["meta"]["ball_position"],
        team_is_team_0: record.is_team_0 as Event["meta"]["team_is_team_0"],
        confidence: record.confidence as Event["meta"]["confidence"],
        properties: Array.isArray(record.properties)
          ? (record.properties as Event["meta"]["properties"])
          : [],
      },
      payload: {
        kind: "timeline",
        payload: {
          time: timing.type === "span" ? timing.end_time : timing.time,
          frame: timing.type === "span" ? timing.end_frame : timing.frame,
          kind: stream,
        },
      } as EventPayload,
    };
  });
}

function titleCaseStream(stream: string): string {
  return stream
    .split("_")
    .filter(Boolean)
    .map((part) => `${part.slice(0, 1).toUpperCase()}${part.slice(1)}`)
    .join(" ");
}

function eventTiming(payload: Record<string, unknown>): Event["meta"]["timing"] {
  const explicitTiming = payload.timing;
  if (
    explicitTiming &&
    typeof explicitTiming === "object" &&
    !Array.isArray(explicitTiming) &&
    "type" in explicitTiming
  ) {
    return explicitTiming as Event["meta"]["timing"];
  }
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
  if (
    typeof payload.frame === "number" &&
    typeof endFrame === "number" &&
    typeof payload.time === "number" &&
    typeof endTime === "number"
  ) {
    return {
      type: "span",
      start_frame: payload.frame,
      end_frame: endFrame,
      start_time: payload.time,
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
      scope: eventStreamScope(stream),
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
        (record.team_is_team_0 as Event["meta"]["team_is_team_0"]) ??
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
        shadow_defense_max_ball_y: 0,
        shadow_defense_min_goal_side_y: 0,
        shadow_defense_min_gap: 0,
        shadow_defense_max_gap: 0,
        shadow_defense_max_lateral_gap: 0,
        shadow_defense_min_retreat_speed: 0,
        shadow_defense_max_speed_delta: 0,
        ball_half_neutral_zone_half_width_y: 0,
        ball_third_boundary_y: 0,
        territorial_pressure_neutral_zone_half_width_y: 0,
        territorial_pressure_min_establish_seconds: 0,
        territorial_pressure_min_establish_third_seconds: 0,
        territorial_pressure_relief_grace_seconds: 0,
        territorial_pressure_confirmed_relief_grace_seconds: 0,
        rotation_role_depth_margin: 0,
        rotation_first_man_ambiguity_margin: 0,
        rotation_first_man_debounce_seconds: 0,
        rotation_first_man_stint_end_grace_seconds: 0,
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
        flip_into_ball_goal_max_touch_to_goal_seconds: 0,
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
        season: null,
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
    player_activity_events?: PayloadList<"player_activity">;
    field_third_events?: PayloadList<"field_third">;
    field_half_events?: PayloadList<"field_half">;
    ball_depth_events?: PayloadList<"ball_depth">;
    depth_role_events?: PayloadList<"depth_role">;
    ball_proximity_events?: PayloadList<"ball_proximity">;
    shadow_defense_events?: PayloadList<"shadow_defense">;
    rotation_role_events?: PayloadList<"rotation_role">;
    first_man_change_events?: PayloadList<"first_man_change">;
    boost_pickups?: PayloadList<"boost_pickup">;
    boost_respawn?: PayloadList<"respawn">;
    bump_events?: PayloadList<"bump">;
  } = {},
): MaterializedStatsTimeline {
  const events = [
    ...(overrides.events?.events ?? []),
    ...legacyBucketEvents(overrides.events as Record<string, unknown> | undefined),
    ...legacyMechanicEvents(overrides.events as Record<string, unknown> | undefined),
    ...legacyMechanicEvents(overrides as Record<string, unknown> | undefined),
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
    ...(overrides.player_activity_events ?? []).map((event, index) =>
      payloadEvent("player_activity", "player_activity", event, index),
    ),
    ...(overrides.field_third_events ?? []).map((event, index) =>
      payloadEvent("field_third", "field_third", event, index),
    ),
    ...(overrides.field_half_events ?? []).map((event, index) =>
      payloadEvent("field_half", "field_half", event, index),
    ),
    ...(overrides.ball_depth_events ?? []).map((event, index) =>
      payloadEvent("ball_depth", "ball_depth", event, index),
    ),
    ...(overrides.depth_role_events ?? []).map((event, index) =>
      payloadEvent("depth_role", "depth_role", event, index),
    ),
    ...(overrides.ball_proximity_events ?? []).map((event, index) =>
      payloadEvent("ball_proximity", "ball_proximity", event, index),
    ),
    ...(overrides.shadow_defense_events ?? []).map((event, index) =>
      payloadEvent("shadow_defense", "shadow_defense", event, index),
    ),
    ...(overrides.rotation_role_events ?? []).map((event, index) =>
      payloadEvent("rotation_role", "rotation_role", event, index),
    ),
    ...(overrides.first_man_change_events ?? []).map((event, index) =>
      payloadEvent("first_man_change", "first_man_change", event, index),
    ),
    ...(overrides.boost_pickups ?? []).map((event, index) =>
      payloadEvent("boost_pickups", "boost_pickup", event, index),
    ),
    ...(overrides.boost_respawn ?? []).map((event, index) =>
      payloadEvent("boost_respawn", "respawn", event, index),
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
