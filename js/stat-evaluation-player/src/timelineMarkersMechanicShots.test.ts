import test from "node:test";
import assert from "node:assert/strict";

import type { ReplayModel } from "@rlrml/subtr-actor-player";
import {
  buildBackboardTimelineEvents,
  buildCeilingShotTimelineEvents,
  buildDoubleTapTimelineEvents,
  buildFiftyFiftyTimelineEvents,
  buildMustyFlickTimelineEvents,
  buildWallAerialShotTimelineEvents,
  buildWallAerialTimelineEvents,
} from "./timelineMarkers.ts";
import { createLegacyStatsTimeline } from "./testStatsTimeline.ts";

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
