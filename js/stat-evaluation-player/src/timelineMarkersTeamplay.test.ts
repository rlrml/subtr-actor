import test from "node:test";
import assert from "node:assert/strict";

import type { ReplayModel } from "@rlrml/subtr-actor-player";
import {
  buildCenterTimelineEvents,
  buildGoalContextTimelineEvents,
  buildGoalTagTimelineEvents,
  buildHalfVolleyTimelineEvents,
  buildOneTimerTimelineEvents,
  buildPassTimelineEvents,
  buildRushTimelineEvents,
  buildTouchTimelineEvents,
} from "./timelineMarkers.ts";
import { createLegacyStatsTimeline } from "./testStatsTimeline.ts";

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
    players: [{ id: "Steam:blue-id", name: "Blue" }],
  } as ReplayModel;

  const statsTimeline = createLegacyStatsTimeline({
    events: {
      touch: [
        {
          time: 1.5,
          frame: 1,
          player: { Steam: "blue-id" },
          is_team_0: true,
          kind: "control",
          height_band: "ground",
          surface: "ground",
          dodge_state: "no_dodge",
          ball_speed_change: 0,
          sample_time: 1.5,
          sample_frame: 1,
        },
      ],
    },
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
  });

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