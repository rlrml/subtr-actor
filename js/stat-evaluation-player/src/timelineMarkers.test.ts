import test from "node:test";
import assert from "node:assert/strict";

import type { ReplayModel } from "subtr-actor-player";
import {
  buildBackboardTimelineEvents,
  buildBallCarryTimelineEvents,
  buildCeilingShotTimelineEvents,
  buildCenterTimelineEvents,
  buildDodgeResetTimelineEvents,
  buildDoubleTapTimelineEvents,
  buildFiftyFiftyTimelineEvents,
  buildGoalContextTimelineEvents,
  buildGoalTagTimelineEvents,
  buildHalfFlipTimelineEvents,
  buildHalfVolleyTimelineEvents,
  buildMechanicTimelineEvents,
  buildMustyFlickTimelineEvents,
  buildOneTimerTimelineEvents,
  buildPassTimelineEvents,
  buildPowerslideTimelineEvents,
  buildRushTimelineEvents,
  buildSpeedFlipTimelineEvents,
  buildTouchTimelineEvents,
  buildWavedashTimelineEvents,
  buildWallAerialTimelineEvents,
  buildWallAerialShotTimelineEvents,
  buildWhiffTimelineEvents,
  countEnabledTimelineEvents,
  filterReplayTimelineEvents,
  getReplayTimelineEventKinds,
} from "./timelineMarkers.ts";
import { createLegacyStatsTimeline } from "./testStatsTimeline.ts";

test("timeline defaults to goals and adds core and demo replay events when enabled", () => {
  assert.deepEqual(getReplayTimelineEventKinds([]), ["goal"]);
  assert.deepEqual(getReplayTimelineEventKinds(["core", "demo"]), [
    "goal",
    "save",
    "shot",
    "assist",
    "demo",
  ]);
});

test("filterReplayTimelineEvents keeps only goal markers by default", () => {
  const replay = {
    timelineEvents: [
      { kind: "goal", time: 10 },
      { kind: "save", time: 12 },
      { kind: "shot", time: 13 },
      { kind: "assist", time: 13.5 },
      { kind: "demo", time: 14 },
    ],
  } as ReplayModel;

  assert.deepEqual(
    filterReplayTimelineEvents(replay, []).map((event) => event.kind),
    ["goal"],
  );
  assert.deepEqual(
    filterReplayTimelineEvents(replay, ["core", "demo"]).map((event) => event.kind),
    ["goal", "save", "shot", "assist", "demo"],
  );
});

test("buildMechanicTimelineEvents maps endpoint mechanics to markers at the end frame", () => {
  const replay = {
    frames: Array.from({ length: 4 }, (_, time) => ({ time })),
    players: [
      {
        id: "Steam:blue-id",
        name: "Blue",
      },
    ],
  } as ReplayModel;

  const statsTimeline = createLegacyStatsTimeline({
    mechanic_events: [
      {
        id: "double_tap:1:3:0",
        kind: "double_tap",
        player_id: { Steam: "blue-id" },
        is_team_0: true,
        timing: {
          type: "span",
          start_frame: 1,
          end_frame: 3,
          start_time: 1,
          end_time: 3,
        },
        properties: [],
      },
    ],
  });

  assert.deepEqual(buildMechanicTimelineEvents(statsTimeline, replay, ["double_tap"]), [
    {
      id: "double_tap:1:3:0",
      time: 3,
      frame: 3,
      kind: "double_tap",
      label: "Blue double tap",
      shortLabel: "DT",
      playerId: "Steam:blue-id",
      playerName: "Blue",
      isTeamZero: true,
      color: "#3b82f6",
    },
  ]);
});

test("buildMechanicTimelineEvents skips range-only carry mechanics", () => {
  const replay = {
    frames: Array.from({ length: 4 }, (_, time) => ({ time })),
    players: [
      {
        id: "Steam:blue-id",
        name: "Blue",
      },
    ],
  } as ReplayModel;

  const statsTimeline = createLegacyStatsTimeline({
    mechanic_events: [
      {
        id: "ball_carry:1:3:0",
        kind: "ball_carry",
        player_id: { Steam: "blue-id" },
        is_team_0: true,
        timing: {
          type: "span",
          start_frame: 1,
          end_frame: 3,
          start_time: 1,
          end_time: 3,
        },
        properties: [],
      },
      {
        id: "air_dribble:1:3:0",
        kind: "air_dribble",
        player_id: { Steam: "blue-id" },
        is_team_0: true,
        timing: {
          type: "span",
          start_frame: 1,
          end_frame: 3,
          start_time: 1,
          end_time: 3,
        },
        properties: [],
      },
      {
        id: "double_tap:1:3:0",
        kind: "double_tap",
        player_id: { Steam: "blue-id" },
        is_team_0: true,
        timing: {
          type: "span",
          start_frame: 1,
          end_frame: 3,
          start_time: 1,
          end_time: 3,
        },
        properties: [],
      },
    ],
  });

  assert.deepEqual(
    buildMechanicTimelineEvents(statsTimeline, replay, [
      "air_dribble",
      "ball_carry",
      "double_tap",
    ]).map((event) => event.id),
    ["double_tap:1:3:0"],
  );
});

test("buildFiftyFiftyTimelineEvents maps 50/50 winners to timeline markers", () => {
  const replay = {
    frames: Array.from({ length: 41 }, (_, time) => ({ time })),
    players: [
      {
        id: "Steam:blue-id",
        name: "Blue",
      },
      {
        id: "Steam:orange-id",
        name: "Orange",
      },
    ],
  } as ReplayModel;

  const statsTimeline = createLegacyStatsTimeline({
    fifty_fifty_events: [
      {
        start_time: 1,
        start_frame: 20,
        resolve_time: 2,
        resolve_frame: 40,
        is_kickoff: false,
        team_zero_player: { Steam: "blue-id" },
        team_one_player: { Steam: "orange-id" },
        team_zero_position: [0, 0, 0],
        team_one_position: [10, 0, 0],
        midpoint: [5, 0, 0],
        plane_normal: [1, 0, 0],
        winning_team_is_team_0: true,
        possession_team_is_team_0: true,
      },
    ],
  });

  assert.deepEqual(buildFiftyFiftyTimelineEvents(statsTimeline, replay), [
    {
      id: "fifty-fifty:20:Steam:blue-id:Steam:orange-id",
      time: 20,
      frame: 20,
      kind: "fifty-fifty",
      label: "50/50: Blue vs Orange | blue win | blue poss",
      shortLabel: "50",
      isTeamZero: true,
      color: "#3b82f6",
    },
  ]);
});

test("buildCeilingShotTimelineEvents maps serialized ceiling shots to timeline markers", () => {
  const replay = {
    frames: [{ time: 0 }, { time: 1.5 }, { time: 2.25 }],
    players: [
      {
        id: "Steam:blue-id",
        name: "Blue",
      },
    ],
  } as ReplayModel;

  const statsTimeline = createLegacyStatsTimeline({
    ceiling_shot_events: [
      {
        time: 1.2,
        frame: 1,
        player: { Steam: "blue-id" },
        is_team_0: true,
        ceiling_contact_time: 0.9,
        ceiling_contact_frame: 0,
        time_since_ceiling_contact: 0.3,
        ceiling_contact_position: [0, -800, 1988],
        touch_position: [120, -690, 1580],
        local_ball_position: [70, 0, 40],
        separation_from_ceiling: 240,
        roof_alignment: 0.88,
        forward_alignment: 0.72,
        forward_approach_speed: 580,
        ball_speed_change: 610,
        confidence: 0.84,
      },
    ],
  });

  assert.deepEqual(buildCeilingShotTimelineEvents(statsTimeline, replay), [
    {
      id: "ceiling-shot:1:Steam:blue-id:840",
      time: 1.5,
      frame: 1,
      kind: "ceiling-shot",
      label: "Blue ceiling shot 84%",
      shortLabel: "CS",
      playerId: "Steam:blue-id",
      playerName: "Blue",
      isTeamZero: true,
      color: "#3b82f6",
    },
  ]);
});

test("buildWallAerialTimelineEvents maps serialized wall aerial events to timeline markers", () => {
  const replay = {
    frames: [{ time: 0 }, { time: 1.5 }, { time: 2.25 }],
    players: [
      {
        id: "Steam:blue-id",
        name: "Blue",
      },
    ],
  } as ReplayModel;

  const statsTimeline = createLegacyStatsTimeline({
    wall_aerial_events: [
      {
        time: 2.1,
        frame: 2,
        player: { Steam: "blue-id" },
        is_team_0: true,
        wall: "side",
        wall_contact_time: 1,
        wall_contact_frame: 1,
        takeoff_time: 1.4,
        takeoff_frame: 1,
        time_since_takeoff: 0.7,
        wall_contact_position: [4096, -400, 220],
        takeoff_position: [4090, -420, 280],
        player_position: [3200, -600, 760],
        ball_position: [3160, -650, 920],
        setup_start_time: 0.6,
        setup_start_frame: 0,
        setup_duration: 0.4,
        ball_speed: 1500,
        ball_speed_change: 320,
        goal_alignment: 0.72,
        confidence: 0.86,
      },
    ],
  });

  assert.deepEqual(buildWallAerialTimelineEvents(statsTimeline, replay), [
    {
      id: "wall-aerial:2:Steam:blue-id:0",
      time: 2.25,
      frame: 2,
      kind: "wall-aerial",
      label: "Blue wall aerial 86% | side wall",
      shortLabel: "WA",
      playerId: "Steam:blue-id",
      playerName: "Blue",
      isTeamZero: true,
      color: "#3b82f6",
    },
  ]);
});

test("buildWallAerialShotTimelineEvents maps serialized wall aerial shot events to timeline markers", () => {
  const replay = {
    frames: [{ time: 0 }, { time: 1.5 }, { time: 2.25 }],
    players: [
      {
        id: "Steam:blue-id",
        name: "Blue",
      },
    ],
  } as ReplayModel;

  const statsTimeline = createLegacyStatsTimeline({
    wall_aerial_shot_events: [
      {
        time: 2.1,
        frame: 2,
        player: { Steam: "blue-id" },
        is_team_0: true,
        wall: "side",
        wall_contact_time: 1,
        wall_contact_frame: 1,
        takeoff_time: 1.4,
        takeoff_frame: 1,
        time_since_takeoff: 0.7,
        wall_contact_position: [4096, -400, 220],
        takeoff_position: [4090, -420, 280],
        player_position: [3200, -600, 760],
        ball_position: [3160, -650, 920],
        ball_speed: 1500,
        goal_alignment: 0.72,
        confidence: 0.74,
      },
    ],
  });

  assert.deepEqual(buildWallAerialShotTimelineEvents(statsTimeline, replay), [
    {
      id: "wall-aerial-shot:2:Steam:blue-id:0",
      time: 2.25,
      frame: 2,
      kind: "wall-aerial-shot",
      label: "Blue wall aerial shot 74% | side wall",
      shortLabel: "WS",
      playerId: "Steam:blue-id",
      playerName: "Blue",
      isTeamZero: true,
      color: "#3b82f6",
    },
  ]);
});

test("buildMustyFlickTimelineEvents maps cumulative musty counts to timeline markers", () => {
  const replay = {
    frames: [{ time: 0 }, { time: 1.5 }, { time: 2.25 }],
  } as ReplayModel;

  const statsTimeline = {
    replay_meta: {},
    timeline_events: [],
    frames: [
      {
        frame_number: 1,
        time: 1.5,
        dt: 1,
        players: [
          {
            player_id: { Steam: "blue-id" },
            name: "Blue",
            is_team_0: true,
            musty_flick: {
              count: 1,
              last_musty_frame: 1,
              last_musty_time: 1.5,
              is_last_musty: true,
            },
          },
        ],
      },
      {
        frame_number: 2,
        time: 2.25,
        dt: 1,
        players: [
          {
            player_id: { Steam: "blue-id" },
            name: "Blue",
            is_team_0: true,
            musty_flick: {
              count: 1,
              last_musty_frame: 1,
              last_musty_time: 1.5,
              is_last_musty: false,
            },
          },
        ],
      },
    ],
  } as StatsTimeline;

  assert.deepEqual(buildMustyFlickTimelineEvents(statsTimeline, replay), [
    {
      id: "musty-flick:1:Steam:blue-id:1",
      time: 1.5,
      frame: 1,
      kind: "musty-flick",
      label: "Blue musty flick",
      shortLabel: "M",
      playerId: "Steam:blue-id",
      playerName: "Blue",
      isTeamZero: true,
      color: "#3b82f6",
    },
  ]);
});

test("buildBackboardTimelineEvents maps serialized backboard events to timeline markers", () => {
  const replay = {
    frames: [{ time: 0 }, { time: 1.5 }],
    players: [
      {
        id: "Steam:blue-id",
        name: "Blue",
      },
    ],
  } as ReplayModel;

  const statsTimeline = createLegacyStatsTimeline({
    backboard_events: [
      {
        time: 1.2,
        frame: 1,
        player: { Steam: "blue-id" },
        is_team_0: true,
      },
    ],
  });

  assert.deepEqual(buildBackboardTimelineEvents(statsTimeline, replay), [
    {
      id: "backboard:1:Steam:blue-id:0",
      time: 1.5,
      frame: 1,
      kind: "backboard",
      label: "Blue backboard",
      shortLabel: "BB",
      playerId: "Steam:blue-id",
      playerName: "Blue",
      isTeamZero: true,
      color: "#3b82f6",
    },
  ]);
});

test("buildDoubleTapTimelineEvents maps serialized double tap events to timeline markers", () => {
  const replay = {
    frames: [{ time: 0 }, { time: 1.5 }],
    players: [
      {
        id: "Steam:blue-id",
        name: "Blue",
      },
    ],
  } as ReplayModel;

  const statsTimeline = createLegacyStatsTimeline({
    double_tap_events: [
      {
        time: 1.2,
        frame: 1,
        player: { Steam: "blue-id" },
        is_team_0: true,
        backboard_time: 1.0,
        backboard_frame: 0,
      },
    ],
  });

  assert.deepEqual(buildDoubleTapTimelineEvents(statsTimeline, replay), [
    {
      id: "double-tap:1:Steam:blue-id:0",
      time: 1.5,
      frame: 1,
      kind: "double-tap",
      label: "Blue double tap",
      shortLabel: "DT",
      playerId: "Steam:blue-id",
      playerName: "Blue",
      isTeamZero: true,
      color: "#3b82f6",
    },
  ]);
});

test("buildCenterTimelineEvents maps serialized center events to timeline markers", () => {
  const replay = {
    frames: [{ time: 0 }, { time: 1.5 }],
    players: [{ id: "Steam:blue-id", name: "Blue" }],
  } as ReplayModel;

  const statsTimeline = createLegacyStatsTimeline({
    center_events: [
      {
        time: 1.2,
        frame: 1,
        player: { Steam: "blue-id" },
        is_team_0: true,
        start_time: 0.9,
        start_frame: 0,
        duration: 0.3,
        start_ball_position: [0, 0, 100],
        end_ball_position: [200, -300, 110],
        ball_travel_distance: 360,
        ball_advance_distance: 220,
        lateral_centering_distance: 415.6,
      },
    ],
  });

  assert.deepEqual(buildCenterTimelineEvents(statsTimeline, replay), [
    {
      id: "center:1:Steam:blue-id:0",
      time: 1.5,
      frame: 1,
      kind: "center",
      label: "Blue center | 416uu lateral",
      shortLabel: "C",
      playerId: "Steam:blue-id",
      playerName: "Blue",
      isTeamZero: true,
      color: "#3b82f6",
    },
  ]);
});

test("buildPassTimelineEvents maps serialized passes to timeline markers", () => {
  const replay = {
    frames: [{ time: 0 }, { time: 1.5 }],
    players: [
      { id: "Steam:passer-id", name: "Passer" },
      { id: "Steam:receiver-id", name: "Receiver" },
    ],
  } as ReplayModel;

  const statsTimeline = createLegacyStatsTimeline({
    pass_events: [
      {
        time: 1.2,
        frame: 1,
        passer: { Steam: "passer-id" },
        receiver: { Steam: "receiver-id" },
        is_team_0: true,
        start_time: 0.8,
        start_frame: 0,
        duration: 0.4,
        ball_travel_distance: 1234.4,
        ball_advance_distance: 900,
        pass_kind: "fifty_fifty_backboard",
      },
    ],
  });

  assert.deepEqual(buildPassTimelineEvents(statsTimeline, replay), [
    {
      id: "pass:1:Steam:passer-id:Steam:receiver-id:0",
      time: 1.5,
      frame: 1,
      kind: "pass",
      label: "Passer to Receiver fifty fifty backboard pass | 1234uu",
      shortLabel: "P",
      playerId: "Steam:passer-id",
      playerName: "Passer",
      secondaryPlayerId: "Steam:receiver-id",
      secondaryPlayerName: "Receiver",
      isTeamZero: true,
      color: "#3b82f6",
    },
  ]);
});

test("buildOneTimerTimelineEvents maps serialized one-timers to timeline markers", () => {
  const replay = {
    frames: [{ time: 0 }, { time: 1.5 }],
    players: [
      { id: "Steam:shooter-id", name: "Shooter" },
      { id: "Steam:passer-id", name: "Passer" },
    ],
  } as ReplayModel;

  const statsTimeline = createLegacyStatsTimeline({
    one_timer_events: [
      {
        time: 1.2,
        frame: 1,
        player: { Steam: "shooter-id" },
        passer: { Steam: "passer-id" },
        is_team_0: true,
        pass_start_time: 0.8,
        pass_start_frame: 0,
        pass_duration: 0.4,
        pass_travel_distance: 1200,
        pass_advance_distance: 700,
        ball_speed: 1800.7,
        goal_alignment: 0.9,
      },
    ],
  });

  assert.deepEqual(buildOneTimerTimelineEvents(statsTimeline, replay), [
    {
      id: "one-timer:1:Steam:passer-id:Steam:shooter-id:0",
      time: 1.5,
      frame: 1,
      kind: "one-timer",
      label: "Shooter one-timer from Passer | 1801uu/s",
      shortLabel: "OT",
      playerId: "Steam:shooter-id",
      playerName: "Shooter",
      secondaryPlayerId: "Steam:passer-id",
      secondaryPlayerName: "Passer",
      isTeamZero: true,
      color: "#3b82f6",
    },
  ]);
});

test("buildHalfVolleyTimelineEvents maps serialized half volleys to timeline markers", () => {
  const replay = {
    frames: [{ time: 0 }, { time: 1.5 }],
    players: [{ id: "Steam:blue-id", name: "Blue" }],
  } as ReplayModel;

  const statsTimeline = createLegacyStatsTimeline({
    half_volley_events: [
      {
        time: 1.2,
        frame: 1,
        player: { Steam: "blue-id" },
        is_team_0: true,
        bounce_time: 1.0,
        bounce_frame: 0,
        bounce_to_touch_seconds: 0.2,
        ball_speed: 1650.4,
        goal_alignment: 0.8,
      },
    ],
  });

  assert.deepEqual(buildHalfVolleyTimelineEvents(statsTimeline, replay), [
    {
      id: "half-volley:1:Steam:blue-id:0",
      time: 1.5,
      frame: 1,
      kind: "half-volley",
      label: "Blue half volley | 1650uu/s",
      shortLabel: "HV",
      playerId: "Steam:blue-id",
      playerName: "Blue",
      isTeamZero: true,
      color: "#3b82f6",
    },
  ]);
});

test("buildRushTimelineEvents maps serialized rush spans to end-time markers", () => {
  const replay = {
    frames: Array.from({ length: 6 }, (_, time) => ({ time })),
    players: [],
  } as ReplayModel;

  const statsTimeline = createLegacyStatsTimeline({
    rush_events: [
      {
        start_time: 1,
        start_frame: 1,
        end_time: 4,
        end_frame: 4,
        is_team_0: true,
        attackers: 2,
        defenders: 1,
      },
    ],
  });

  assert.deepEqual(buildRushTimelineEvents(statsTimeline, replay), [
    {
      id: "rush:1:4:0",
      time: 4,
      frame: 4,
      kind: "rush",
      label: "Blue rush 2v1",
      shortLabel: "R",
      playerId: null,
      playerName: null,
      isTeamZero: true,
      color: "#3b82f6",
    },
  ]);
});

test("buildGoalTagTimelineEvents and buildGoalContextTimelineEvents map goal analysis events", () => {
  const replay = {
    frames: [{ time: 0 }, { time: 1.5 }],
    players: [{ id: "Steam:blue-id", name: "Blue" }],
  } as ReplayModel;

  const statsTimeline = createLegacyStatsTimeline({
    goal_tag_events: [
      {
        goal_index: 0,
        time: 1.2,
        frame: 1,
        kind: "half_volley_goal",
        scoring_team_is_team_0: true,
        scorer: { Steam: "blue-id" },
        confidence: 0.88,
        modifiers: [],
        evidence: [],
      },
    ],
    events: {
      goal_context: [
        {
          time: 1.2,
          frame: 1,
          scoring_team_is_team_0: true,
          scorer: { Steam: "blue-id" },
          scoring_team_most_back_player: null,
          defending_team_most_back_player: null,
          ball_position: null,
          ball_air_time_before_goal: null,
          scorer_last_touch: null,
          players: [],
        },
      ],
    },
  });

  assert.deepEqual(buildGoalTagTimelineEvents(statsTimeline, replay), [
    {
      id: "goal-tag:0:half_volley_goal:0",
      time: 1.5,
      frame: 1,
      kind: "goal-tag",
      label: "Blue half volley goal 88%",
      shortLabel: "GT",
      playerId: "Steam:blue-id",
      playerName: "Blue",
      isTeamZero: true,
      color: "#3b82f6",
    },
  ]);
  assert.deepEqual(buildGoalContextTimelineEvents(statsTimeline, replay), [
    {
      id: "goal-context:1:Steam:blue-id:0",
      time: 1.5,
      frame: 1,
      kind: "goal-context",
      label: "Blue goal context",
      shortLabel: "GC",
      playerId: "Steam:blue-id",
      playerName: "Blue",
      isTeamZero: true,
      color: "#3b82f6",
    },
  ]);
});

test("buildTouchTimelineEvents maps touch overlay markers to timeline markers", () => {
  const replay = {
    frames: [{ time: 0 }, { time: 1.5 }],
    ballFrames: [{ position: { x: 0, y: 0, z: 0 } }, { position: { x: 10, y: 20, z: 30 } }],
  } as ReplayModel;

  const statsTimeline = {
    replay_meta: {},
    timeline_events: [],
    frames: [
      {
        frame_number: 1,
        time: 1.5,
        dt: 1,
        players: [
          {
            player_id: { Steam: "blue-id" },
            name: "Blue",
            is_team_0: true,
            touch: {
              touch_count: 1,
              last_touch_frame: 1,
              last_touch_time: 1.5,
              is_last_touch: true,
            },
          },
        ],
      },
    ],
  } as StatsTimeline;

  assert.deepEqual(buildTouchTimelineEvents(statsTimeline, replay), [
    {
      id: "touch-stat:1:Steam:blue-id:1",
      time: 1.5,
      frame: 1,
      kind: "touch",
      label: "Blue touch",
      shortLabel: "T",
      playerId: "Steam:blue-id",
      playerName: "Blue",
      isTeamZero: true,
      color: "#3b82f6",
    },
  ]);
});

test("buildDodgeResetTimelineEvents emits non-flip-reset dodge refreshes", () => {
  const replay = {
    frames: [{ time: 0 }, { time: 1.5 }, { time: 2.25 }],
  } as ReplayModel;

  const statsTimeline = {
    replay_meta: {},
    timeline_events: [],
    frames: [
      {
        frame_number: 1,
        time: 1.5,
        dt: 1,
        players: [
          {
            player_id: { Steam: "blue-id" },
            name: "Blue",
            is_team_0: true,
            dodge_reset: {
              count: 1,
              on_ball_count: 1,
            },
          },
        ],
      },
      {
        frame_number: 2,
        time: 2.25,
        dt: 1,
        players: [
          {
            player_id: { Steam: "blue-id" },
            name: "Blue",
            is_team_0: true,
            dodge_reset: {
              count: 2,
              on_ball_count: 1,
            },
          },
        ],
      },
    ],
  } as StatsTimeline;

  assert.deepEqual(buildDodgeResetTimelineEvents(statsTimeline, replay), [
    {
      id: "dodge-reset:2:Steam:blue-id:2:air",
      time: 2.25,
      frame: 2,
      kind: "dodge-reset",
      label: "Blue dodge refresh",
      shortLabel: "DR",
      playerId: "Steam:blue-id",
      playerName: "Blue",
      isTeamZero: true,
      color: "#3b82f6",
    },
  ]);
});

test("buildBallCarryTimelineEvents maps carry completions to timeline markers", () => {
  const replay = {
    frames: [{ time: 0 }, { time: 1.5 }],
  } as ReplayModel;

  const statsTimeline = {
    replay_meta: {},
    timeline_events: [],
    frames: [
      {
        frame_number: 1,
        time: 1.5,
        dt: 1,
        players: [
          {
            player_id: { Steam: "orange-id" },
            name: "Orange",
            is_team_0: false,
            ball_carry: {
              carry_count: 1,
              total_carry_time: 0,
              total_straight_line_distance: 0,
              total_path_distance: 0,
              longest_carry_time: 0,
              furthest_carry_distance: 0,
              fastest_carry_speed: 0,
              carry_speed_sum: 0,
              average_horizontal_gap_sum: 0,
              average_vertical_gap_sum: 0,
              air_dribble_count: 0,
              total_air_dribble_time: 0,
              total_air_dribble_straight_line_distance: 0,
              total_air_dribble_path_distance: 0,
              longest_air_dribble_time: 0,
              furthest_air_dribble_distance: 0,
              fastest_air_dribble_speed: 0,
              air_dribble_speed_sum: 0,
              average_air_dribble_horizontal_gap_sum: 0,
              average_air_dribble_vertical_gap_sum: 0,
            },
          },
        ],
      },
    ],
  } as StatsTimeline;

  assert.deepEqual(buildBallCarryTimelineEvents(statsTimeline, replay), [
    {
      id: "ball-carry:1:Steam:orange-id:1",
      time: 1.5,
      frame: 1,
      kind: "ball-carry",
      label: "Orange ball carry",
      shortLabel: "BC",
      playerId: "Steam:orange-id",
      playerName: "Orange",
      isTeamZero: false,
      color: "#f59e0b",
    },
  ]);
});

test("buildPowerslideTimelineEvents maps powerslide presses to timeline markers", () => {
  const replay = {
    frames: [{ time: 0 }, { time: 1.5 }],
  } as ReplayModel;

  const statsTimeline = {
    replay_meta: {},
    timeline_events: [],
    frames: [
      {
        frame_number: 1,
        time: 1.5,
        dt: 1,
        players: [
          {
            player_id: { Steam: "orange-id" },
            name: "Orange",
            is_team_0: false,
            powerslide: {
              total_duration: 0,
              press_count: 1,
            },
          },
        ],
      },
    ],
  } as StatsTimeline;

  assert.deepEqual(buildPowerslideTimelineEvents(statsTimeline, replay), [
    {
      id: "powerslide:1:Steam:orange-id:1",
      time: 1.5,
      frame: 1,
      kind: "powerslide",
      label: "Orange powerslide",
      shortLabel: "PS",
      playerId: "Steam:orange-id",
      playerName: "Orange",
      isTeamZero: false,
      color: "#f59e0b",
    },
  ]);
});

test("buildSpeedFlipTimelineEvents maps serialized speed flips to timeline markers", () => {
  const replay = {
    frames: [{ time: 0 }, { time: 1.5 }, { time: 2.25 }],
    players: [
      {
        id: "Steam:blue-id",
        name: "Blue",
      },
    ],
  } as ReplayModel;

  const statsTimeline = createLegacyStatsTimeline({
    speed_flip_events: [
      {
        time: 1.2,
        frame: 1,
        player: { Steam: "blue-id" },
        is_team_0: true,
        time_since_kickoff_start: 0.4,
        start_position: [0, 0, 0],
        end_position: [100, 0, 0],
        start_speed: 1200,
        max_speed: 1600,
        best_alignment: 0.91,
        diagonal_score: 0.8,
        cancel_score: 0.78,
        speed_score: 0.74,
        confidence: 0.86,
      },
    ],
  });

  assert.deepEqual(buildSpeedFlipTimelineEvents(statsTimeline, replay), [
    {
      id: "speed-flip:1:Steam:blue-id:860",
      time: 1.5,
      frame: 1,
      kind: "speed-flip",
      label: "Blue speed flip 86%",
      shortLabel: "SF",
      playerId: "Steam:blue-id",
      playerName: "Blue",
      isTeamZero: true,
      color: "#3b82f6",
    },
  ]);
});

test("buildHalfFlipTimelineEvents maps serialized half flips to timeline markers", () => {
  const replay = {
    frames: [{ time: 0 }, { time: 1.5 }, { time: 2.25 }],
    players: [
      {
        id: "Steam:blue-id",
        name: "Blue",
      },
    ],
  } as ReplayModel;

  const statsTimeline = createLegacyStatsTimeline({
    half_flip_events: [
      {
        time: 1.2,
        frame: 1,
        player: { Steam: "blue-id" },
        is_team_0: true,
        start_position: [0, 0, 20],
        end_position: [-200, 0, 20],
        start_speed: 600,
        end_speed: 1180,
        start_backward_alignment: 0.94,
        best_reorientation_alignment: 0.91,
        best_forward_reversal: 0.89,
        max_forward_vertical: 0.73,
        confidence: 0.82,
      },
    ],
  });

  assert.deepEqual(buildHalfFlipTimelineEvents(statsTimeline, replay), [
    {
      id: "half-flip:1:Steam:blue-id:0",
      time: 1.5,
      frame: 1,
      kind: "half-flip",
      label: "Blue half flip 82% | +580uu/s",
      shortLabel: "HF",
      playerId: "Steam:blue-id",
      playerName: "Blue",
      isTeamZero: true,
      color: "#3b82f6",
    },
  ]);
});

test("buildWavedashTimelineEvents maps serialized wavedashes to their own timeline markers", () => {
  const replay = {
    frames: [{ time: 0 }, { time: 1.5 }, { time: 2.25 }],
    players: [
      {
        id: "Steam:blue-id",
        name: "Blue",
      },
    ],
  } as ReplayModel;

  const statsTimeline = createLegacyStatsTimeline({
    wavedash_events: [
      {
        time: 1.2,
        frame: 1,
        player: { Steam: "blue-id" },
        is_team_0: true,
        dodge_time: 1.05,
        dodge_frame: 0,
        time_since_dodge: 0.15,
        dodge_position: [0, 0, 70],
        landing_position: [120, 0, 17],
        start_speed: 700,
        landing_speed: 1240,
        horizontal_speed_gain: 540,
        landing_uprightness: 0.92,
        confidence: 0.81,
      },
    ],
  });

  assert.deepEqual(buildWavedashTimelineEvents(statsTimeline, replay), [
    {
      id: "wavedash:1:Steam:blue-id:0",
      time: 1.5,
      frame: 1,
      kind: "wavedash",
      label: "Blue wavedash 81% | +540uu/s",
      shortLabel: "WD",
      playerId: "Steam:blue-id",
      playerName: "Blue",
      isTeamZero: true,
      color: "#3b82f6",
    },
  ]);
});

test("buildWhiffTimelineEvents maps serialized whiffs to timeline markers", () => {
  const replay = {
    frames: [{ time: 0 }, { time: 1.5 }],
    players: [
      {
        id: "Steam:blue-id",
        name: "Blue",
      },
    ],
  } as ReplayModel;

  const statsTimeline = createLegacyStatsTimeline({
    whiff_events: [
      {
        kind: "whiff",
        time: 1.2,
        frame: 1,
        player: { Steam: "blue-id" },
        is_team_0: true,
        closest_approach_distance: 128.4,
        forward_alignment: 0.72,
        approach_speed: 1310.6,
        dodge_active: true,
        aerial: true,
      },
    ],
  });

  assert.deepEqual(buildWhiffTimelineEvents(statsTimeline, replay), [
    {
      id: "whiff:1:Steam:blue-id:0",
      time: 1.5,
      frame: 1,
      kind: "whiff",
      label: "Blue aerial dodge whiff | 128uu closest, 1311uu/s",
      shortLabel: "DW",
      playerId: "Steam:blue-id",
      playerName: "Blue",
      isTeamZero: true,
      color: "#3b82f6",
    },
  ]);
});

test("buildWhiffTimelineEvents labels beaten-to-ball events separately", () => {
  const replay = {
    frames: [{ time: 0 }, { time: 1.5 }],
    players: [
      {
        id: "Steam:blue-id",
        name: "Blue",
      },
    ],
  } as ReplayModel;

  const statsTimeline = createLegacyStatsTimeline({
    whiff_events: [
      {
        kind: "beaten_to_ball",
        time: 1.2,
        frame: 1,
        player: { Steam: "blue-id" },
        is_team_0: true,
        closest_approach_distance: 128.4,
        forward_alignment: 0.72,
        approach_speed: 1310.6,
        dodge_active: true,
        aerial: true,
      },
    ],
  });

  assert.deepEqual(buildWhiffTimelineEvents(statsTimeline, replay), [
    {
      id: "whiff:1:Steam:blue-id:0",
      time: 1.5,
      frame: 1,
      kind: "whiff",
      label: "Blue aerial dodge beaten to ball | 128uu closest, 1311uu/s",
      shortLabel: "BT",
      playerId: "Steam:blue-id",
      playerName: "Blue",
      isTeamZero: true,
      color: "#3b82f6",
    },
  ]);
});

test("countEnabledTimelineEvents includes enabled custom module markers", () => {
  const replay = {
    timelineEvents: [
      { kind: "goal", time: 10 },
      { kind: "save", time: 12 },
    ],
    frames: [{ time: 0 }, { time: 1.25 }],
    ballFrames: [{ position: { x: 0, y: 0, z: 0 } }, { position: { x: 10, y: 20, z: 30 } }],
    players: [
      {
        id: "Steam:blue-id",
        name: "Blue",
      },
      {
        id: "Steam:orange-id",
        name: "Orange",
      },
    ],
  } as ReplayModel;

  const statsTimeline = createLegacyStatsTimeline({
    fifty_fifty_events: [
      {
        start_time: 1,
        start_frame: 20,
        resolve_time: 2,
        resolve_frame: 40,
        is_kickoff: false,
        team_zero_player: { Steam: "blue-id" },
        team_one_player: { Steam: "orange-id" },
        team_zero_position: [0, 0, 0],
        team_one_position: [10, 0, 0],
        midpoint: [5, 0, 0],
        plane_normal: [1, 0, 0],
        winning_team_is_team_0: true,
        possession_team_is_team_0: true,
      },
    ],
    backboard_events: [
      {
        time: 1.0,
        frame: 1,
        player: { Steam: "blue-id" },
        is_team_0: true,
      },
    ],
    ceiling_shot_events: [
      {
        time: 1.15,
        frame: 1,
        player: { Steam: "blue-id" },
        is_team_0: true,
        ceiling_contact_time: 0.9,
        ceiling_contact_frame: 0,
        time_since_ceiling_contact: 0.25,
        ceiling_contact_position: [0, -800, 1988],
        touch_position: [120, -690, 1580],
        local_ball_position: [70, 0, 40],
        separation_from_ceiling: 240,
        roof_alignment: 0.88,
        forward_alignment: 0.72,
        forward_approach_speed: 580,
        ball_speed_change: 610,
        confidence: 0.84,
      },
    ],
    wall_aerial_events: [
      {
        time: 1.2,
        frame: 1,
        player: { Steam: "blue-id" },
        is_team_0: true,
        wall: "side",
        wall_contact_time: 0.8,
        wall_contact_frame: 0,
        takeoff_time: 1,
        takeoff_frame: 1,
        time_since_takeoff: 0.2,
        wall_contact_position: [4096, -300, 220],
        takeoff_position: [4070, -330, 300],
        player_position: [3600, -500, 820],
        ball_position: [3700, -520, 900],
        setup_start_time: 0.4,
        setup_start_frame: 0,
        setup_duration: 0.4,
        ball_speed: 1400,
        ball_speed_change: 260,
        goal_alignment: 0.69,
        confidence: 0.74,
      },
    ],
    wall_aerial_shot_events: [
      {
        time: 1.2,
        frame: 1,
        player: { Steam: "blue-id" },
        is_team_0: true,
        wall: "side",
        wall_contact_time: 0.8,
        wall_contact_frame: 0,
        takeoff_time: 1,
        takeoff_frame: 1,
        time_since_takeoff: 0.2,
        wall_contact_position: [4096, -300, 220],
        takeoff_position: [4070, -330, 300],
        player_position: [3600, -500, 820],
        ball_position: [3700, -520, 900],
        ball_speed: 1400,
        goal_alignment: 0.69,
        confidence: 0.74,
      },
    ],
    double_tap_events: [
      {
        time: 1.1,
        frame: 1,
        player: { Steam: "blue-id" },
        is_team_0: true,
        backboard_time: 1.0,
        backboard_frame: 1,
      },
    ],
    center_events: [
      {
        time: 1.1,
        frame: 1,
        player: { Steam: "blue-id" },
        is_team_0: true,
        start_time: 0.8,
        start_frame: 0,
        duration: 0.3,
        start_ball_position: [0, 0, 100],
        end_ball_position: [100, 200, 100],
        ball_travel_distance: 300,
        ball_advance_distance: 200,
        lateral_centering_distance: 240,
      },
    ],
    one_timer_events: [
      {
        time: 1.1,
        frame: 1,
        player: { Steam: "blue-id" },
        passer: { Steam: "orange-id" },
        is_team_0: true,
        pass_start_time: 0.7,
        pass_start_frame: 0,
        pass_duration: 0.4,
        pass_travel_distance: 1200,
        pass_advance_distance: 800,
        ball_speed: 1800,
        goal_alignment: 0.9,
      },
    ],
    pass_events: [
      {
        time: 1.1,
        frame: 1,
        passer: { Steam: "blue-id" },
        receiver: { Steam: "orange-id" },
        is_team_0: true,
        start_time: 0.7,
        start_frame: 0,
        duration: 0.4,
        ball_travel_distance: 1200,
        ball_advance_distance: 800,
        pass_kind: "direct",
      },
    ],
    goal_tag_events: [
      {
        goal_index: 0,
        time: 1.1,
        frame: 1,
        kind: "one_timer_goal",
        scoring_team_is_team_0: true,
        scorer: { Steam: "blue-id" },
        confidence: 0.8,
        modifiers: [],
        evidence: [],
      },
    ],
    speed_flip_events: [
      {
        time: 1.1,
        frame: 1,
        player: { Steam: "blue-id" },
        is_team_0: true,
        time_since_kickoff_start: 0.35,
        start_position: [0, 0, 0],
        end_position: [120, 0, 0],
        start_speed: 1180,
        max_speed: 1650,
        best_alignment: 0.93,
        diagonal_score: 0.84,
        cancel_score: 0.82,
        speed_score: 0.79,
        confidence: 0.88,
      },
    ],
    half_flip_events: [
      {
        time: 1.1,
        frame: 1,
        player: { Steam: "blue-id" },
        is_team_0: true,
        start_position: [0, 0, 20],
        end_position: [-140, 0, 20],
        start_speed: 620,
        end_speed: 1180,
        start_backward_alignment: 0.95,
        best_reorientation_alignment: 0.9,
        best_forward_reversal: 0.88,
        max_forward_vertical: 0.7,
        confidence: 0.81,
      },
    ],
    half_volley_events: [
      {
        time: 1.1,
        frame: 1,
        player: { Steam: "blue-id" },
        is_team_0: true,
        bounce_time: 0.9,
        bounce_frame: 0,
        bounce_to_touch_seconds: 0.2,
        ball_speed: 1600,
        goal_alignment: 0.8,
      },
    ],
    wavedash_events: [
      {
        time: 1.1,
        frame: 1,
        player: { Steam: "blue-id" },
        is_team_0: true,
        dodge_time: 1.0,
        dodge_frame: 0,
        time_since_dodge: 0.1,
        dodge_position: [0, 0, 80],
        landing_position: [100, 0, 17],
        start_speed: 720,
        landing_speed: 1260,
        horizontal_speed_gain: 540,
        landing_uprightness: 0.9,
        confidence: 0.82,
      },
    ],
    whiff_events: [
      {
        kind: "whiff",
        time: 1.1,
        frame: 1,
        player: { Steam: "blue-id" },
        is_team_0: true,
        closest_approach_distance: 126,
        forward_alignment: 0.7,
        approach_speed: 1280,
        dodge_active: false,
        aerial: false,
      },
    ],
    bump_events: [
      {
        time: 1.1,
        frame: 1,
        initiator: { Steam: "blue-id" },
        victim: { Steam: "orange-id" },
        initiator_is_team_0: true,
        victim_is_team_0: false,
        is_team_bump: false,
        strength: 500,
        confidence: 0.8,
        contact_distance: 30,
        closing_speed: 900,
        victim_impulse: 1000,
        initiator_position: [0, 0, 0],
        victim_position: [10, 0, 0],
      },
    ],
    events: {
      goal_context: [
        {
          time: 1.1,
          frame: 1,
          scoring_team_is_team_0: true,
          scorer: { Steam: "blue-id" },
          scoring_team_most_back_player: null,
          defending_team_most_back_player: null,
          ball_position: null,
          ball_air_time_before_goal: null,
          scorer_last_touch: null,
          players: [],
        },
      ],
    },
    frames: [
      {
        frame_number: 1,
        time: 1.25,
        dt: 0.1,
        team_zero: {
          rush: {
            count: 1,
            two_v_one_count: 1,
            two_v_two_count: 0,
            two_v_three_count: 0,
            three_v_one_count: 0,
            three_v_two_count: 0,
            three_v_three_count: 0,
          },
        },
        team_one: {
          rush: {
            count: 0,
            two_v_one_count: 0,
            two_v_two_count: 0,
            two_v_three_count: 0,
            three_v_one_count: 0,
            three_v_two_count: 0,
            three_v_three_count: 0,
          },
        },
        players: [
          {
            player_id: { Steam: "blue-id" },
            name: "Blue",
            is_team_0: true,
            musty_flick: {
              count: 1,
              last_musty_frame: 1,
              last_musty_time: 1.25,
              is_last_musty: true,
            },
            ceiling_shot: {
              count: 1,
              high_confidence_count: 1,
              last_ceiling_shot_frame: 1,
              last_ceiling_shot_time: 1.25,
              is_last_ceiling_shot: true,
            },
            touch: {
              touch_count: 1,
              last_touch_frame: 1,
              last_touch_time: 1.25,
              is_last_touch: true,
            },
            dodge_reset: {
              count: 1,
              on_ball_count: 1,
            },
            ball_carry: {
              carry_count: 1,
              total_carry_time: 0,
              total_straight_line_distance: 0,
              total_path_distance: 0,
              longest_carry_time: 0,
              furthest_carry_distance: 0,
              fastest_carry_speed: 0,
              carry_speed_sum: 0,
              average_horizontal_gap_sum: 0,
              average_vertical_gap_sum: 0,
              air_dribble_count: 0,
              total_air_dribble_time: 0,
              total_air_dribble_straight_line_distance: 0,
              total_air_dribble_path_distance: 0,
              longest_air_dribble_time: 0,
              furthest_air_dribble_distance: 0,
              fastest_air_dribble_speed: 0,
              air_dribble_speed_sum: 0,
              average_air_dribble_horizontal_gap_sum: 0,
              average_air_dribble_vertical_gap_sum: 0,
            },
            powerslide: {
              total_duration: 0,
              press_count: 1,
            },
          },
        ],
      },
    ],
  });

  assert.equal(countEnabledTimelineEvents([], replay, statsTimeline), 1);
  assert.equal(countEnabledTimelineEvents(["core", "fifty-fifty"], replay, statsTimeline), 3);
  assert.equal(
    countEnabledTimelineEvents(["core", "fifty-fifty", "rush"], replay, statsTimeline),
    3,
  );
  assert.equal(
    countEnabledTimelineEvents(
      ["core", "fifty-fifty", "rush", "musty-flick"],
      replay,
      statsTimeline,
    ),
    4,
  );
  assert.equal(
    countEnabledTimelineEvents(
      ["core", "fifty-fifty", "rush", "musty-flick", "ceiling-shot", "wall-aerial"],
      replay,
      statsTimeline,
    ),
    6,
  );
  assert.equal(
    countEnabledTimelineEvents(
      [
        "core",
        "fifty-fifty",
        "rush",
        "musty-flick",
        "backboard",
        "ceiling-shot",
        "wall-aerial",
        "wall-aerial-shot",
        "center",
        "double-tap",
        "goal-tags",
        "one-timer",
        "pass",
        "touch",
        "dodge-reset",
        "ball-carry",
        "powerslide",
        "speed-flip",
        "half-flip",
        "half-volley",
        "wavedash",
        "whiff",
        "bump",
      ],
      replay,
      statsTimeline,
    ),
    22,
  );
});
