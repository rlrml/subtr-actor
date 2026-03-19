import test from "node:test";
import assert from "node:assert/strict";

import type { ReplayModel } from "subtr-actor-player";
import type { DynamicStatsTimeline, StatsTimeline } from "./statsTimeline.ts";
import {
  buildPossessionTimelineRanges,
  buildPressureTimelineRanges,
  buildTimeInZoneTimelineRanges,
} from "./timelineRanges.ts";

test("buildPossessionTimelineRanges derives merged team and neutral control spans", () => {
  const timeline = {
    replay_meta: {},
    timeline_events: [],
    frames: [
      {
        frame_number: 1,
        time: 1,
        dt: 1,
        players: [],
      },
      {
        frame_number: 2,
        time: 2,
        dt: 1,
        players: [],
      },
      {
        frame_number: 3,
        time: 3,
        dt: 1,
        players: [],
      },
      {
        frame_number: 4,
        time: 4,
        dt: 1,
        players: [],
      },
    ],
  } as StatsTimeline;
  const dynamicTimeline = {
    replay_meta: {},
    timeline_events: [],
    frames: [
      {
        frame_number: 1,
        time: 1,
        dt: 1,
        possession: [
          {
            domain: "possession",
            name: "time",
            variant: "labeled",
            unit: "seconds",
            labels: [{ key: "possession_state", value: "team_zero" }],
            value_type: "float",
            value: 1,
          },
        ],
        players: [],
      },
      {
        frame_number: 2,
        time: 2,
        dt: 1,
        possession: [
          {
            domain: "possession",
            name: "time",
            variant: "labeled",
            unit: "seconds",
            labels: [{ key: "possession_state", value: "team_zero" }],
            value_type: "float",
            value: 2,
          },
        ],
        players: [],
      },
      {
        frame_number: 3,
        time: 3,
        dt: 1,
        possession: [
          {
            domain: "possession",
            name: "time",
            variant: "labeled",
            unit: "seconds",
            labels: [{ key: "possession_state", value: "team_zero" }],
            value_type: "float",
            value: 2,
          },
          {
            domain: "possession",
            name: "time",
            variant: "labeled",
            unit: "seconds",
            labels: [{ key: "possession_state", value: "neutral" }],
            value_type: "float",
            value: 1,
          },
        ],
        players: [],
      },
      {
        frame_number: 4,
        time: 4,
        dt: 1,
        possession: [
          {
            domain: "possession",
            name: "time",
            variant: "labeled",
            unit: "seconds",
            labels: [{ key: "possession_state", value: "team_zero" }],
            value_type: "float",
            value: 2,
          },
          {
            domain: "possession",
            name: "time",
            variant: "labeled",
            unit: "seconds",
            labels: [{ key: "possession_state", value: "team_one" }],
            value_type: "float",
            value: 1,
          },
          {
            domain: "possession",
            name: "time",
            variant: "labeled",
            unit: "seconds",
            labels: [{ key: "possession_state", value: "neutral" }],
            value_type: "float",
            value: 1,
          },
        ],
        players: [],
      },
    ],
  } as DynamicStatsTimeline;

  assert.deepEqual(buildPossessionTimelineRanges(timeline, dynamicTimeline), [
    {
      id: "possession:team_zero:0.000",
      startTime: 0,
      endTime: 2,
      lane: "possession",
      laneLabel: "Possession",
      label: "Blue possession",
      color: "rgba(59, 130, 246, 0.88)",
      isTeamZero: true,
    },
    {
      id: "possession:neutral:2.000",
      startTime: 2,
      endTime: 3,
      lane: "possession",
      laneLabel: "Possession",
      label: "Neutral possession",
      color: "rgba(209, 217, 224, 0.7)",
      isTeamZero: null,
    },
    {
      id: "possession:team_one:3.000",
      startTime: 3,
      endTime: 4,
      lane: "possession",
      laneLabel: "Possession",
      label: "Orange possession",
      color: "rgba(245, 158, 11, 0.88)",
      isTeamZero: false,
    },
  ]);
});

test("buildPressureTimelineRanges derives half-control spans from labeled deltas including neutral", () => {
  const timeline = {
    replay_meta: {},
    timeline_events: [],
    frames: [
      {
        frame_number: 1,
        time: 0.5,
        dt: 0.5,
        players: [],
      },
      {
        frame_number: 2,
        time: 1,
        dt: 0.5,
        players: [],
      },
    ],
  } as StatsTimeline;
  const dynamicTimeline = {
    replay_meta: {},
    timeline_events: [],
    frames: [
      {
        frame_number: 1,
        time: 0.5,
        dt: 0.5,
        pressure: [
          {
            domain: "pressure",
            name: "time",
            variant: "labeled",
            unit: "seconds",
            labels: [{ key: "field_half", value: "team_zero_side" }],
            value_type: "float",
            value: 0.5,
          },
        ],
        players: [],
      },
      {
        frame_number: 2,
        time: 1,
        dt: 0.5,
        pressure: [
          {
            domain: "pressure",
            name: "time",
            variant: "labeled",
            unit: "seconds",
            labels: [{ key: "field_half", value: "team_zero_side" }],
            value_type: "float",
            value: 0.5,
          },
          {
            domain: "pressure",
            name: "time",
            variant: "labeled",
            unit: "seconds",
            labels: [{ key: "field_half", value: "neutral" }],
            value_type: "float",
            value: 0.5,
          },
        ],
        players: [],
      },
    ],
  } as DynamicStatsTimeline;

  assert.deepEqual(buildPressureTimelineRanges(timeline, dynamicTimeline), [
    {
      id: "half-control:team_zero_side:0.000",
      startTime: 0,
      endTime: 0.5,
      lane: "half-control",
      laneLabel: "Half Control",
      label: "Blue half control",
      color: "rgba(89, 195, 255, 0.76)",
      isTeamZero: true,
    },
    {
      id: "half-control:neutral:0.500",
      startTime: 0.5,
      endTime: 1,
      lane: "half-control",
      laneLabel: "Half Control",
      label: "Neutral half control",
      color: "rgba(209, 217, 224, 0.7)",
      isTeamZero: null,
    },
  ]);
});

test("buildPressureTimelineRanges uses replay centerline fallback for legacy half-control stats", () => {
  const timeline = {
    config: {
      most_back_forward_threshold_y: 400,
      pressure_neutral_zone_half_width_y: 200,
    },
    replay_meta: {},
    timeline_events: [],
    frames: [
      {
        frame_number: 1,
        time: 0.5,
        dt: 0.5,
        players: [],
      },
      {
        frame_number: 2,
        time: 1,
        dt: 0.5,
        players: [],
      },
    ],
  } as StatsTimeline;
  const dynamicTimeline = {
    config: {
      most_back_forward_threshold_y: 400,
      pressure_neutral_zone_half_width_y: 200,
    },
    replay_meta: {},
    timeline_events: [],
    frames: [
      {
        frame_number: 1,
        time: 0.5,
        dt: 0.5,
        pressure: [
          {
            domain: "pressure",
            name: "time",
            variant: "labeled",
            unit: "seconds",
            labels: [{ key: "field_half", value: "team_zero_side" }],
            value_type: "float",
            value: 0.5,
          },
        ],
        players: [],
      },
      {
        frame_number: 2,
        time: 1,
        dt: 0.5,
        pressure: [
          {
            domain: "pressure",
            name: "time",
            variant: "labeled",
            unit: "seconds",
            labels: [{ key: "field_half", value: "team_zero_side" }],
            value_type: "float",
            value: 0.5,
          },
          {
            domain: "pressure",
            name: "time",
            variant: "labeled",
            unit: "seconds",
            labels: [{ key: "field_half", value: "team_one_side" }],
            value_type: "float",
            value: 0.5,
          },
        ],
        players: [],
      },
    ],
  } as DynamicStatsTimeline;
  const replay = {
    frames: [
      { time: 0, secondsRemaining: 0, gameState: 0, kickoffCountdown: 0 },
      { time: 0.5, secondsRemaining: 0, gameState: 0, kickoffCountdown: 0 },
      { time: 1, secondsRemaining: 0, gameState: 0, kickoffCountdown: 0 },
    ],
    ballFrames: [
      { position: { x: 0, y: 0, z: 0 } },
      { position: { x: 0, y: -260, z: 0 } },
      { position: { x: 0, y: 0, z: 0 } },
    ],
  } as Partial<ReplayModel> as ReplayModel;

  assert.deepEqual(buildPressureTimelineRanges(timeline, dynamicTimeline, replay), [
    {
      id: "half-control:team_zero_side:0.000",
      startTime: 0,
      endTime: 0.5,
      lane: "half-control",
      laneLabel: "Half Control",
      label: "Blue half control",
      color: "rgba(89, 195, 255, 0.76)",
      isTeamZero: true,
    },
    {
      id: "half-control:neutral:0.500",
      startTime: 0.5,
      endTime: 1,
      lane: "half-control",
      laneLabel: "Half Control",
      label: "Neutral half control",
      color: "rgba(209, 217, 224, 0.7)",
      isTeamZero: null,
    },
  ]);
});

test("buildTimeInZoneTimelineRanges derives per-player third occupancy spans", () => {
  const timeline = {
    replay_meta: {},
    timeline_events: [],
    frames: [
      {
        frame_number: 1,
        time: 1,
        dt: 1,
        players: [
          {
            player_id: { Steam: "blue-id" },
            name: "Blue",
            is_team_0: true,
            positioning: {
              active_game_time: 1,
              time_defensive_zone: 1,
              time_neutral_zone: 0,
              time_offensive_zone: 0,
              time_defensive_half: 1,
              time_offensive_half: 0,
              time_demolished: 0,
              time_no_teammates: 0,
              time_most_back: 0,
              time_most_forward: 0,
              time_mid_role: 0,
              time_other_role: 0,
            },
          },
        ],
      },
      {
        frame_number: 2,
        time: 2,
        dt: 1,
        players: [
          {
            player_id: { Steam: "blue-id" },
            name: "Blue",
            is_team_0: true,
            positioning: {
              active_game_time: 2,
              time_defensive_zone: 1,
              time_neutral_zone: 1,
              time_offensive_zone: 0,
              time_defensive_half: 1,
              time_offensive_half: 1,
              time_demolished: 0,
              time_no_teammates: 0,
              time_most_back: 0,
              time_most_forward: 0,
              time_mid_role: 0,
              time_other_role: 0,
            },
          },
        ],
      },
      {
        frame_number: 3,
        time: 3,
        dt: 1,
        players: [
          {
            player_id: { Steam: "blue-id" },
            name: "Blue",
            is_team_0: true,
            positioning: {
              active_game_time: 3,
              time_defensive_zone: 1,
              time_neutral_zone: 1,
              time_offensive_zone: 1,
              time_defensive_half: 1,
              time_offensive_half: 2,
              time_demolished: 0,
              time_no_teammates: 0,
              time_most_back: 0,
              time_most_forward: 0,
              time_mid_role: 0,
              time_other_role: 0,
            },
          },
        ],
      },
    ],
  } as StatsTimeline;

  assert.deepEqual(buildTimeInZoneTimelineRanges(timeline), [
    {
      id: "time-in-zone:Steam:blue-id:time_defensive_third:0.000",
      startTime: 0,
      endTime: 1,
      lane: "time-in-zone:Steam:blue-id",
      laneLabel: "Blue",
      label: "Def third",
      color: "rgba(89, 195, 255, 0.74)",
      isTeamZero: true,
    },
    {
      id: "time-in-zone:Steam:blue-id:time_neutral_third:1.000",
      startTime: 1,
      endTime: 2,
      lane: "time-in-zone:Steam:blue-id",
      laneLabel: "Blue",
      label: "Neutral third",
      color: "rgba(209, 217, 224, 0.68)",
      isTeamZero: true,
    },
    {
      id: "time-in-zone:Steam:blue-id:time_offensive_third:2.000",
      startTime: 2,
      endTime: 3,
      lane: "time-in-zone:Steam:blue-id",
      laneLabel: "Blue",
      label: "Off third",
      color: "rgba(255, 193, 92, 0.78)",
      isTeamZero: true,
    },
  ]);
});
