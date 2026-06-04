import test from "node:test";
import assert from "node:assert/strict";

import type { ReplayModel } from "@rlrml/player";
import {
  buildBackboardTimelineEvents,
  buildCeilingShotTimelineEvents,
  buildCenterTimelineEvents,
  buildDoubleTapTimelineEvents,
  buildFiftyFiftyTimelineEvents,
  buildGoalContextTimelineEvents,
  buildGoalTagTimelineEvents,
  buildHalfVolleyTimelineEvents,
  buildMechanicPlaylistEvents,
  buildMechanicTimelineEvents,
  buildMustyFlickTimelineEvents,
  buildOneTimerTimelineEvents,
  buildPassTimelineEvents,
  buildRushTimelineEvents,
  buildWallAerialTimelineEvents,
  buildWallAerialShotTimelineEvents,
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

test("buildMechanicTimelineEvents skips span mechanics", () => {
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
      {
        id: "flick:1:3:0",
        kind: "flick",
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

  assert.deepEqual(buildMechanicTimelineEvents(statsTimeline, replay, ["double_tap", "flick"]), []);
});

test("buildMechanicTimelineEvents maps flip reset mechanics to moment markers", () => {
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
        id: "flip_reset:2:0",
        kind: "flip_reset",
        player_id: { Steam: "blue-id" },
        is_team_0: true,
        timing: {
          type: "moment",
          frame: 2,
          time: 2,
        },
        properties: [],
      },
    ],
  });

  assert.deepEqual(buildMechanicTimelineEvents(statsTimeline, replay, ["flip_reset"]), [
    {
      id: "flip_reset:2:0",
      time: 2,
      frame: 2,
      kind: "flip_reset",
      label: "Blue flip reset",
      shortLabel: "FR",
      playerId: "Steam:blue-id",
      playerName: "Blue",
      isTeamZero: true,
      color: "#3b82f6",
    },
  ]);
});

test("buildMechanicTimelineEvents skips all span mechanics regardless kind", () => {
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
    [],
  );
});

test("buildMechanicPlaylistEvents includes span mechanics at their end time", () => {
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

  assert.deepEqual(buildMechanicPlaylistEvents(statsTimeline, replay, ["double_tap"]), [
    {
      id: "double_tap:1:3:0:playlist",
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
        team_zero_touch_time: 1,
        team_zero_touch_frame: 20,
        team_zero_dodge_contact: false,
        team_one_touch_time: 1,
        team_one_touch_frame: 20,
        team_one_dodge_contact: false,
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
        sample_time: 1.2,
        sample_frame: 1,
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
        sample_time: 2.1,
        sample_frame: 2,
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
      label: "Blue wall-to-air setup 86% | side wall",
      shortLabel: "W2A",
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
    players: [{ id: "Steam:blue-id", name: "Blue" }],
  } as ReplayModel;

  const statsTimeline = createLegacyStatsTimeline({
    musty_flick_events: [
      {
        time: 1.5,
        frame: 1,
        sample_time: 1.5,
        sample_frame: 1,
        player: { Steam: "blue-id" },
        is_team_0: true,
        aerial: false,
        dodge_time: 1.4,
        dodge_frame: 1,
        time_since_dodge: 0.1,
        confidence: 0.8,
        local_ball_position: [0, 0, 0],
        rear_alignment: 0.9,
        top_alignment: 0.7,
        forward_approach_speed: 1200,
        pitch_rate: 2,
        ball_speed_change: 300,
      },
    ],
  });

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
        sample_time: 1.2,
        sample_frame: 1,
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
        sample_time: 1.2,
        sample_frame: 1,
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
          tags: [
            {
              kind: "half_volley_goal",
              metadata: {
                confidence: 0.88,
                modifiers: [],
                related_events: [],
                evidence: [],
              },
            },
          ],
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
