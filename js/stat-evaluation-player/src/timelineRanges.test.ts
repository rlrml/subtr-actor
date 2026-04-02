import test from "node:test";
import assert from "node:assert/strict";

import type { ReplayModel } from "subtr-actor-player";
import type { StatsTimeline } from "./statsTimeline.ts";
import {
  buildPossessionTimelineRanges,
  buildPressureTimelineRanges,
  buildRushTimelineRanges,
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
        possession: {
          tracked_time: 1,
          team_zero_time: 1,
          team_one_time: 0,
          neutral_time: 0,
        },
        players: [],
      },
      {
        frame_number: 2,
        time: 2,
        dt: 1,
        possession: {
          tracked_time: 2,
          team_zero_time: 2,
          team_one_time: 0,
          neutral_time: 0,
        },
        players: [],
      },
      {
        frame_number: 3,
        time: 3,
        dt: 1,
        possession: {
          tracked_time: 3,
          team_zero_time: 2,
          team_one_time: 0,
          neutral_time: 1,
        },
        players: [],
      },
      {
        frame_number: 4,
        time: 4,
        dt: 1,
        possession: {
          tracked_time: 4,
          team_zero_time: 2,
          team_one_time: 1,
          neutral_time: 1,
        },
        players: [],
      },
    ],
  } as StatsTimeline;

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

test("buildPressureTimelineRanges derives half-control spans from labeled deltas including neutral", () => {
  const timeline = {
    replay_meta: {},
    timeline_events: [],
    frames: [
      {
        frame_number: 1,
        time: 0.5,
        dt: 0.5,
        pressure: {
          tracked_time: 0.5,
          team_zero_side_time: 0.5,
          team_one_side_time: 0,
          neutral_time: 0,
        },
        players: [],
      },
      {
        frame_number: 2,
        time: 1,
        dt: 0.5,
        pressure: {
          tracked_time: 1,
          team_zero_side_time: 0.5,
          team_one_side_time: 0,
          neutral_time: 0.5,
        },
        players: [],
      },
    ],
  } as StatsTimeline;

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

test("buildRushTimelineRanges maps serialized rush spans to replay timeline ranges", () => {
  const replay = {
    frames: [
      { time: 0 },
      { time: 1.5 },
      { time: 2.25 },
      { time: 3.5 },
    ],
  } as ReplayModel;
  const timeline = {
    replay_meta: {},
    timeline_events: [],
    rush_events: [
      {
        start_time: 1.1,
        start_frame: 1,
        end_time: 2.0,
        end_frame: 2,
        is_team_0: true,
        attackers: 2,
        defenders: 1,
      },
      {
        start_time: 2.4,
        start_frame: 2,
        end_time: 3.0,
        end_frame: 3,
        is_team_0: false,
        attackers: 3,
        defenders: 2,
      },
    ],
    frames: [],
  } as StatsTimeline;

  assert.deepEqual(buildRushTimelineRanges(timeline, replay), [
    {
      id: "rush-range:1:2:0",
      startTime: 1.5,
      endTime: 2.25,
      lane: "rush",
      laneLabel: "Rush",
      label: "Blue rush 2v1",
      color: "rgba(59, 130, 246, 0.4)",
      isTeamZero: true,
    },
    {
      id: "rush-range:2:3:1",
      startTime: 2.25,
      endTime: 3.5,
      lane: "rush",
      laneLabel: "Rush",
      label: "Orange rush 3v2",
      color: "rgba(245, 158, 11, 0.4)",
      isTeamZero: false,
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
        pressure: {
          tracked_time: 0.5,
          team_zero_side_time: 0.5,
          team_one_side_time: 0,
          neutral_time: 0,
        },
        players: [],
      },
      {
        frame_number: 2,
        time: 1,
        dt: 0.5,
        pressure: {
          tracked_time: 1,
          team_zero_side_time: 0.5,
          team_one_side_time: 0.5,
          neutral_time: 0,
        },
        players: [],
      },
    ],
  } as StatsTimeline;
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

  assert.deepEqual(buildPressureTimelineRanges(timeline, replay), [
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

test("buildTimeInZoneTimelineRanges merges continuous spans independently per player lane", () => {
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
          {
            player_id: { Epic: "orange-id" },
            name: "Orange",
            is_team_0: false,
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
              time_defensive_zone: 2,
              time_neutral_zone: 0,
              time_offensive_zone: 0,
              time_defensive_half: 2,
              time_offensive_half: 0,
              time_demolished: 0,
              time_no_teammates: 0,
              time_most_back: 0,
              time_most_forward: 0,
              time_mid_role: 0,
              time_other_role: 0,
            },
          },
          {
            player_id: { Epic: "orange-id" },
            name: "Orange",
            is_team_0: false,
            positioning: {
              active_game_time: 2,
              time_defensive_zone: 2,
              time_neutral_zone: 0,
              time_offensive_zone: 0,
              time_defensive_half: 2,
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
              time_defensive_zone: 3,
              time_neutral_zone: 0,
              time_offensive_zone: 0,
              time_defensive_half: 3,
              time_offensive_half: 0,
              time_demolished: 0,
              time_no_teammates: 0,
              time_most_back: 0,
              time_most_forward: 0,
              time_mid_role: 0,
              time_other_role: 0,
            },
          },
          {
            player_id: { Epic: "orange-id" },
            name: "Orange",
            is_team_0: false,
            positioning: {
              active_game_time: 3,
              time_defensive_zone: 3,
              time_neutral_zone: 0,
              time_offensive_zone: 0,
              time_defensive_half: 3,
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
    ],
  } as StatsTimeline;

  assert.deepEqual(buildTimeInZoneTimelineRanges(timeline), [
    {
      id: "time-in-zone:Steam:blue-id:time_defensive_third:0.000",
      startTime: 0,
      endTime: 3,
      lane: "time-in-zone:Steam:blue-id",
      laneLabel: "Blue",
      label: "Def third",
      color: "rgba(89, 195, 255, 0.74)",
      isTeamZero: true,
    },
    {
      id: "time-in-zone:Epic:orange-id:time_defensive_third:0.000",
      startTime: 0,
      endTime: 3,
      lane: "time-in-zone:Epic:orange-id",
      laneLabel: "Orange",
      label: "Def third",
      color: "rgba(255, 193, 92, 0.78)",
      isTeamZero: false,
    },
  ]);
});

test("buildTimeInZoneTimelineRanges uses player-relative colors for orange players", () => {
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
            player_id: { Epic: "orange-id" },
            name: "Orange",
            is_team_0: false,
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
            player_id: { Epic: "orange-id" },
            name: "Orange",
            is_team_0: false,
            positioning: {
              active_game_time: 2,
              time_defensive_zone: 1,
              time_neutral_zone: 0,
              time_offensive_zone: 1,
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
    ],
  } as StatsTimeline;

  assert.deepEqual(buildTimeInZoneTimelineRanges(timeline), [
    {
      id: "time-in-zone:Epic:orange-id:time_defensive_third:0.000",
      startTime: 0,
      endTime: 1,
      lane: "time-in-zone:Epic:orange-id",
      laneLabel: "Orange",
      label: "Def third",
      color: "rgba(255, 193, 92, 0.78)",
      isTeamZero: false,
    },
    {
      id: "time-in-zone:Epic:orange-id:time_offensive_third:1.000",
      startTime: 1,
      endTime: 2,
      lane: "time-in-zone:Epic:orange-id",
      laneLabel: "Orange",
      label: "Off third",
      color: "rgba(89, 195, 255, 0.74)",
      isTeamZero: false,
    },
  ]);
});
