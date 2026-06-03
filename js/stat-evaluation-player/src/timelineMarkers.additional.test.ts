import test from "node:test";
import assert from "node:assert/strict";

import type { ReplayModel } from "@rlrml/player";
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
  buildMechanicPlaylistEvents,
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

test("buildDodgeResetTimelineEvents emits non-flip-reset dodge refreshes", () => {
  const replay = {
    frames: [{ time: 0 }, { time: 1.5 }, { time: 2.25 }],
    players: [{ id: "Steam:blue-id", name: "Blue" }],
  } as ReplayModel;

  const statsTimeline = createLegacyStatsTimeline({
    dodge_reset_events: [
      {
        time: 1.5,
        frame: 1,
        player: { Steam: "blue-id" },
        is_team_0: true,
        counter_value: 1,
        on_ball: true,
      },
      {
        time: 2.25,
        frame: 2,
        player: { Steam: "blue-id" },
        is_team_0: true,
        counter_value: 2,
        on_ball: false,
      },
    ],
  });

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
    players: [{ id: "Steam:orange-id", name: "Orange" }],
  } as ReplayModel;

  const statsTimeline = createLegacyStatsTimeline({
    ball_carry_events: [
      {
        player_id: { Steam: "orange-id" },
        is_team_0: false,
        kind: "carry",
        start_frame: 0,
        end_frame: 1,
        start_time: 0,
        end_time: 1.5,
        duration: 1.5,
        straight_line_distance: 0,
        path_distance: 0,
        average_horizontal_gap: 0,
        average_vertical_gap: 0,
        average_speed: 0,
        touch_count: 1,
        air_touch_count: 0,
        air_dribble_origin: null,
      },
    ],
  });

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
    players: [{ id: "Steam:orange-id", name: "Orange" }],
  } as ReplayModel;

  const statsTimeline = createLegacyStatsTimeline({
    powerslide_events: [
      {
        time: 1.5,
        frame: 1,
        player: { Steam: "orange-id" },
        is_team_0: false,
        active: true,
      },
    ],
  });

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
