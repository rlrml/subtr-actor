import test from "node:test";
import assert from "node:assert/strict";

import type { DynamicStatsTimeline } from "./statsTimeline.ts";
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
        possession: [
          {
            domain: "possession",
            name: "time",
            variant: "labeled",
            unit: "seconds",
            value_type: "float",
            value: 1,
            labels: [{ key: "possession_state", value: "team_zero" }],
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
            value_type: "float",
            value: 2,
            labels: [{ key: "possession_state", value: "team_zero" }],
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
            value_type: "float",
            value: 2,
            labels: [{ key: "possession_state", value: "team_zero" }],
          },
          {
            domain: "possession",
            name: "time",
            variant: "labeled",
            unit: "seconds",
            value_type: "float",
            value: 1,
            labels: [{ key: "possession_state", value: "neutral" }],
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
            value_type: "float",
            value: 2,
            labels: [{ key: "possession_state", value: "team_zero" }],
          },
          {
            domain: "possession",
            name: "time",
            variant: "labeled",
            unit: "seconds",
            value_type: "float",
            value: 1,
            labels: [{ key: "possession_state", value: "neutral" }],
          },
          {
            domain: "possession",
            name: "time",
            variant: "labeled",
            unit: "seconds",
            value_type: "float",
            value: 1,
            labels: [{ key: "possession_state", value: "team_one" }],
          },
        ],
        players: [],
      },
    ],
  } as DynamicStatsTimeline;

  assert.deepEqual(buildPossessionTimelineRanges(timeline), [
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

test("buildPressureTimelineRanges derives half-control spans from labeled deltas", () => {
  const timeline = {
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
            value_type: "float",
            value: 0.5,
            labels: [{ key: "field_half", value: "team_zero_side" }],
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
            value_type: "float",
            value: 0.5,
            labels: [{ key: "field_half", value: "team_zero_side" }],
          },
          {
            domain: "pressure",
            name: "time",
            variant: "labeled",
            unit: "seconds",
            value_type: "float",
            value: 0.5,
            labels: [{ key: "field_half", value: "team_one_side" }],
          },
        ],
        players: [],
      },
    ],
  } as DynamicStatsTimeline;

  assert.deepEqual(buildPressureTimelineRanges(timeline), [
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
      id: "half-control:team_one_side:0.500",
      startTime: 0.5,
      endTime: 1,
      lane: "half-control",
      laneLabel: "Half Control",
      label: "Orange half control",
      color: "rgba(255, 193, 92, 0.76)",
      isTeamZero: false,
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
            stats: [
              {
                domain: "positioning",
                name: "time_defensive_third",
                variant: "total",
                unit: "seconds",
                value_type: "float",
                value: 1,
              },
            ],
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
            stats: [
              {
                domain: "positioning",
                name: "time_defensive_third",
                variant: "total",
                unit: "seconds",
                value_type: "float",
                value: 1,
              },
              {
                domain: "positioning",
                name: "time_neutral_third",
                variant: "total",
                unit: "seconds",
                value_type: "float",
                value: 1,
              },
            ],
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
            stats: [
              {
                domain: "positioning",
                name: "time_defensive_third",
                variant: "total",
                unit: "seconds",
                value_type: "float",
                value: 1,
              },
              {
                domain: "positioning",
                name: "time_neutral_third",
                variant: "total",
                unit: "seconds",
                value_type: "float",
                value: 1,
              },
              {
                domain: "positioning",
                name: "time_offensive_third",
                variant: "total",
                unit: "seconds",
                value_type: "float",
                value: 1,
              },
            ],
          },
        ],
      },
    ],
  } as DynamicStatsTimeline;

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
