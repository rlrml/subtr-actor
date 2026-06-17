import test from "node:test";
import assert from "node:assert/strict";

import type { ReplayModel } from "@rlrml/player";
import type { StatsTimeline } from "./statsTimeline.ts";
import {
  buildBoostPickupTimelineRanges,
  buildFiftyFiftyTimelineRanges,
  buildMechanicTimelineRanges,
  buildPossessionTimelineRanges,
  buildPowerslideTimelineRanges,
  buildBallHalfTimelineRanges,
  buildBallThirdTimelineRanges,
  buildRushTimelineRanges,
} from "./timelineRanges.ts";
import { createLegacyStatsTimeline, createStatsTimeline } from "./testStatsTimeline.ts";

test("buildMechanicTimelineRanges emits ranges for visible span mechanics", () => {
  const replay = {
    frames: Array.from({ length: 6 }, (_, time) => ({ time })),
    players: [
      {
        id: "Steam:blue-id",
        name: "Blue",
      },
    ],
  } as ReplayModel;
  const timeline = createLegacyStatsTimeline({
    mechanic_events: [
      {
        id: "wall_aerial:1:3:0",
        kind: "wall_aerial",
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
        id: "wall_aerial_shot:2:4:0",
        kind: "wall_aerial_shot",
        player_id: { Steam: "blue-id" },
        is_team_0: true,
        timing: {
          type: "span",
          start_frame: 2,
          end_frame: 4,
          start_time: 2,
          end_time: 4,
        },
        properties: [],
      },
      {
        id: "pass:2:4:0",
        kind: "pass",
        player_id: { Steam: "blue-id" },
        is_team_0: true,
        timing: {
          type: "span",
          start_frame: 2,
          end_frame: 4,
          start_time: 2,
          end_time: 4,
        },
        properties: [],
      },
      {
        id: "ball_carry:1:5:0",
        kind: "ball_carry",
        player_id: { Steam: "blue-id" },
        is_team_0: true,
        timing: {
          type: "span",
          start_frame: 1,
          end_frame: 5,
          start_time: 1,
          end_time: 5,
        },
        properties: [],
      },
      {
        id: "flick:3:5:0",
        kind: "flick",
        player_id: { Steam: "blue-id" },
        is_team_0: true,
        timing: {
          type: "span",
          start_frame: 3,
          end_frame: 5,
          start_time: 3,
          end_time: 5,
        },
        properties: [],
      },
      {
        id: "flip_reset:4:0",
        kind: "flip_reset",
        player_id: { Steam: "blue-id" },
        is_team_0: true,
        timing: {
          type: "moment",
          frame: 4,
          time: 4,
        },
        properties: [],
      },
    ],
  });

  assert.deepEqual(
    buildMechanicTimelineRanges(timeline, replay, [
      "wall_aerial",
      "wall_aerial_shot",
      "pass",
      "ball_carry",
      "flick",
      "flip_reset",
    ]).map((range) => range.id),
    [
      "ball_carry:1:5:0",
      "wall_aerial:1:3:0",
      "pass:2:4:0",
      "wall_aerial_shot:2:4:0",
      "flick:3:5:0",
    ],
  );
});

test("buildPossessionTimelineRanges derives merged team and neutral control spans", () => {
  const timeline = {
    replay_meta: {},
    timeline_events: [],
    frames: [
      {
        frame_number: 1,
        time: 1,
        dt: 1,
        team_zero: {
          possession: {
            tracked_time: 1,
            possession_time: 1,
            opponent_possession_time: 0,
            neutral_time: 0,
          },
        },
        team_one: {
          possession: {
            tracked_time: 1,
            possession_time: 0,
            opponent_possession_time: 1,
            neutral_time: 0,
          },
        },
        players: [],
      },
      {
        frame_number: 2,
        time: 2,
        dt: 1,
        team_zero: {
          possession: {
            tracked_time: 2,
            possession_time: 2,
            opponent_possession_time: 0,
            neutral_time: 0,
          },
        },
        team_one: {
          possession: {
            tracked_time: 2,
            possession_time: 0,
            opponent_possession_time: 2,
            neutral_time: 0,
          },
        },
        players: [],
      },
      {
        frame_number: 3,
        time: 3,
        dt: 1,
        team_zero: {
          possession: {
            tracked_time: 3,
            possession_time: 2,
            opponent_possession_time: 0,
            neutral_time: 1,
          },
        },
        team_one: {
          possession: {
            tracked_time: 3,
            possession_time: 0,
            opponent_possession_time: 2,
            neutral_time: 1,
          },
        },
        players: [],
      },
      {
        frame_number: 4,
        time: 4,
        dt: 1,
        team_zero: {
          possession: {
            tracked_time: 4,
            possession_time: 2,
            opponent_possession_time: 1,
            neutral_time: 1,
          },
        },
        team_one: {
          possession: {
            tracked_time: 4,
            possession_time: 1,
            opponent_possession_time: 2,
            neutral_time: 1,
          },
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

test("buildPossessionTimelineRanges derives spans from compact event timelines", () => {
  const timeline = createStatsTimeline({
    events: {
      possession: [
        {
          frame: 1,
          time: 1,
          active: true,
          possession_state: "team_zero",
          field_third: "team_zero_third",
        },
        {
          frame: 3,
          time: 3,
          active: true,
          possession_state: "neutral",
          field_third: "neutral_third",
        },
      ],
    },
    frames: [
      { frame_number: 1, time: 1, dt: 1, team_zero: {}, team_one: {}, players: [] },
      { frame_number: 2, time: 2, dt: 1, team_zero: {}, team_one: {}, players: [] },
      { frame_number: 3, time: 3, dt: 1, team_zero: {}, team_one: {}, players: [] },
    ],
  });

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
  ]);
});

test("buildBallHalfTimelineRanges derives half-control spans from labeled deltas including neutral", () => {
  const timeline = {
    replay_meta: {},
    timeline_events: [],
    frames: [
      {
        frame_number: 1,
        time: 0.5,
        dt: 0.5,
        team_zero: {
          ball_half: {
            tracked_time: 0.5,
            defensive_half_time: 0.5,
            offensive_half_time: 0,
            neutral_time: 0,
          },
        },
        team_one: {
          ball_half: {
            tracked_time: 0.5,
            defensive_half_time: 0,
            offensive_half_time: 0.5,
            neutral_time: 0,
          },
        },
        players: [],
      },
      {
        frame_number: 2,
        time: 1,
        dt: 0.5,
        team_zero: {
          ball_half: {
            tracked_time: 1,
            defensive_half_time: 0.5,
            offensive_half_time: 0,
            neutral_time: 0.5,
          },
        },
        team_one: {
          ball_half: {
            tracked_time: 1,
            defensive_half_time: 0,
            offensive_half_time: 0.5,
            neutral_time: 0.5,
          },
        },
        players: [],
      },
    ],
  } as StatsTimeline;

  assert.deepEqual(buildBallHalfTimelineRanges(timeline), [
    {
      id: "half-control:team_zero_side:0.000",
      startTime: 0,
      endTime: 0.5,
      lane: "half-control",
      laneLabel: "Half Control",
      label: "Blue half control",
      color: "#3b82f6",
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

test("buildBallHalfTimelineRanges derives spans from compact event timelines", () => {
  const timeline = createStatsTimeline({
    events: {
      ball_half: [
        { frame: 1, time: 0.5, active: true, field_half: "team_zero_side" },
        { frame: 2, time: 1, active: true, field_half: "team_one_side" },
        { frame: 3, time: 1.5, active: true, field_half: "neutral" },
      ],
    },
    frames: [
      { frame_number: 1, time: 0.5, dt: 0.5, team_zero: {}, team_one: {}, players: [] },
      { frame_number: 2, time: 1, dt: 0.5, team_zero: {}, team_one: {}, players: [] },
      { frame_number: 3, time: 1.5, dt: 0.5, team_zero: {}, team_one: {}, players: [] },
    ],
  });

  assert.deepEqual(buildBallHalfTimelineRanges(timeline), [
    {
      id: "half-control:team_zero_side:0.000",
      startTime: 0,
      endTime: 0.5,
      lane: "half-control",
      laneLabel: "Half Control",
      label: "Blue half control",
      color: "#3b82f6",
      isTeamZero: true,
    },
    {
      id: "half-control:team_one_side:0.500",
      startTime: 0.5,
      endTime: 1,
      lane: "half-control",
      laneLabel: "Half Control",
      label: "Orange half control",
      color: "#f59e0b",
      isTeamZero: false,
    },
    {
      id: "half-control:neutral:1.000",
      startTime: 1,
      endTime: 1.5,
      lane: "half-control",
      laneLabel: "Half Control",
      label: "Neutral half control",
      color: "rgba(209, 217, 224, 0.7)",
      isTeamZero: null,
    },
  ]);
});

test("buildBallThirdTimelineRanges derives third-control spans from labeled deltas including neutral", () => {
  const timeline = {
    replay_meta: {},
    timeline_events: [],
    frames: [
      {
        frame_number: 1,
        time: 0.5,
        dt: 0.5,
        team_zero: {
          ball_third: {
            tracked_time: 0.5,
            defensive_third_time: 0.5,
            neutral_third_time: 0,
            offensive_third_time: 0,
          },
        },
        team_one: {
          ball_third: {
            tracked_time: 0.5,
            defensive_third_time: 0,
            neutral_third_time: 0,
            offensive_third_time: 0.5,
          },
        },
        players: [],
      },
      {
        frame_number: 2,
        time: 1,
        dt: 0.5,
        team_zero: {
          ball_third: {
            tracked_time: 1,
            defensive_third_time: 0.5,
            neutral_third_time: 0.5,
            offensive_third_time: 0,
          },
        },
        team_one: {
          ball_third: {
            tracked_time: 1,
            defensive_third_time: 0,
            neutral_third_time: 0.5,
            offensive_third_time: 0.5,
          },
        },
        players: [],
      },
    ],
  } as StatsTimeline;

  assert.deepEqual(buildBallThirdTimelineRanges(timeline), [
    {
      id: "third-control:team_zero_third:0.000",
      startTime: 0,
      endTime: 0.5,
      lane: "third-control",
      laneLabel: "Third Control",
      label: "Blue third control",
      color: "#3b82f6",
      isTeamZero: true,
    },
    {
      id: "third-control:neutral_third:0.500",
      startTime: 0.5,
      endTime: 1,
      lane: "third-control",
      laneLabel: "Third Control",
      label: "Neutral third control",
      color: "rgba(209, 217, 224, 0.7)",
      isTeamZero: null,
    },
  ]);
});

test("buildBallThirdTimelineRanges derives spans from compact event timelines", () => {
  const timeline = createStatsTimeline({
    events: {
      ball_third: [
        { frame: 1, time: 0.5, active: true, field_third: "team_zero_third" },
        { frame: 2, time: 1, active: true, field_third: "team_one_third" },
        { frame: 3, time: 1.5, active: true, field_third: "neutral_third" },
      ],
    },
    frames: [
      { frame_number: 1, time: 0.5, dt: 0.5, team_zero: {}, team_one: {}, players: [] },
      { frame_number: 2, time: 1, dt: 0.5, team_zero: {}, team_one: {}, players: [] },
      { frame_number: 3, time: 1.5, dt: 0.5, team_zero: {}, team_one: {}, players: [] },
    ],
  });

  assert.deepEqual(buildBallThirdTimelineRanges(timeline), [
    {
      id: "third-control:team_zero_third:0.000",
      startTime: 0,
      endTime: 0.5,
      lane: "third-control",
      laneLabel: "Third Control",
      label: "Blue third control",
      color: "#3b82f6",
      isTeamZero: true,
    },
    {
      id: "third-control:team_one_third:0.500",
      startTime: 0.5,
      endTime: 1,
      lane: "third-control",
      laneLabel: "Third Control",
      label: "Orange third control",
      color: "#f59e0b",
      isTeamZero: false,
    },
    {
      id: "third-control:neutral_third:1.000",
      startTime: 1,
      endTime: 1.5,
      lane: "third-control",
      laneLabel: "Third Control",
      label: "Neutral third control",
      color: "rgba(209, 217, 224, 0.7)",
      isTeamZero: null,
    },
  ]);
});

test("buildRushTimelineRanges maps serialized rush spans to replay timeline ranges", () => {
  const replay = {
    frames: [{ time: 0 }, { time: 1.5 }, { time: 2.25 }, { time: 3.5 }],
  } as ReplayModel;
  const timeline = createLegacyStatsTimeline({
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
  });

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

test("buildFiftyFiftyTimelineRanges maps contest resolution windows to ranges", () => {
  const replay = {
    frames: Array.from({ length: 6 }, (_, time) => ({ time })),
  } as ReplayModel;
  const timeline = createStatsTimeline({
    events: {
      fifty_fifty: [
        {
          start_time: 1,
          start_frame: 1,
          resolve_time: 4,
          resolve_frame: 4,
          is_kickoff: true,
          team_zero_player: null,
          team_one_player: null,
          team_zero_touch_time: null,
          team_zero_touch_frame: null,
          team_zero_dodge_contact: false,
          team_one_touch_time: null,
          team_one_touch_frame: null,
          team_one_dodge_contact: false,
          team_zero_position: [0, 0, 0],
          team_one_position: [0, 0, 0],
          midpoint: [0, 0, 0],
          plane_normal: [0, 1, 0],
          winning_team_is_team_0: true,
          possession_team_is_team_0: true,
        },
      ],
    },
  });

  assert.deepEqual(buildFiftyFiftyTimelineRanges(timeline, replay), [
    {
      id: "fifty-fifty:1:4:0",
      startTime: 1,
      endTime: 4,
      lane: "fifty-fifty",
      laneLabel: "50/50",
      label: "Blue win kickoff 50/50",
      shortLabel: "KO",
      color: "rgba(59, 130, 246, 0.48)",
      isTeamZero: true,
    },
  ]);
});

test("buildPowerslideTimelineRanges maps active edges to per-player ranges", () => {
  const replay = {
    duration: 10,
    frames: Array.from({ length: 5 }, (_, time) => ({ time })),
    players: [{ id: "Steam:blue-id", name: "Blue" }],
  } as ReplayModel;
  const timeline = createStatsTimeline({
    events: {
      powerslide: [
        {
          time: 1,
          frame: 1,
          player: { Steam: "blue-id" },
          player_position: null,
          is_team_0: true,
          active: true,
        },
        {
          time: 3,
          frame: 3,
          player: { Steam: "blue-id" },
          player_position: null,
          is_team_0: true,
          active: false,
        },
      ],
    },
  });

  assert.deepEqual(buildPowerslideTimelineRanges(timeline, replay), [
    {
      id: "powerslide:1:3:Steam:blue-id",
      startTime: 1,
      endTime: 3,
      lane: "powerslide:Steam:blue-id",
      laneLabel: "Blue",
      label: "Blue powerslide",
      shortLabel: "PS",
      color: "#3b82f6",
      isTeamZero: true,
    },
  ]);
});

test("buildBoostPickupTimelineRanges maps pad pickups to a separate size-filtered lane", () => {
  const replay = {
    frames: Array.from({ length: 51 }, (_, index) => ({
      time: index === 10 ? 1.25 : index === 20 ? 2.5 : index / 10,
    })),
    players: [
      {
        id: "Steam:blue-id",
        name: "Blue",
        isTeamZero: true,
        cameraSettings: {},
        frames: [],
      },
    ],
    boostPads: [
      {
        index: 3,
        padId: "Boost_TA_3",
        size: "small",
        position: { x: 0, y: 0, z: 70 },
        events: [
          {
            time: 1,
            frame: 10,
            available: false,
            playerId: "Steam:blue-id",
            playerName: "Blue",
          },
          {
            time: 5,
            frame: 50,
            available: true,
          },
        ],
      },
      {
        index: 9,
        padId: "Boost_TA_9",
        size: "big",
        position: { x: 0, y: 0, z: 73 },
        events: [
          {
            time: 2,
            frame: 20,
            available: false,
            playerId: null,
            playerName: null,
          },
        ],
      },
    ],
  } as Partial<ReplayModel> as ReplayModel;

  const legacyTimeline = createLegacyStatsTimeline();

  assert.deepEqual(
    buildBoostPickupTimelineRanges(legacyTimeline, replay, {
      sizes: ["small"],
    }),
    [
      {
        id: "boost-pickup:3:10:0",
        startTime: 1.25,
        endTime: 1.33,
        lane: "boost-pickups",
        laneLabel: "Boost Pickups",
        label: "Blue picked up small boost pad 3",
        shortLabel: "12",
        color: "#3b82f6",
        isTeamZero: true,
      },
    ],
  );

  assert.deepEqual(
    buildBoostPickupTimelineRanges(legacyTimeline, replay).map((range) => range.id),
    ["boost-pickup:3:10:0", "boost-pickup:9:20:0"],
  );

  assert.deepEqual(
    buildBoostPickupTimelineRanges(legacyTimeline, replay, {
      detections: [],
    }),
    [],
  );
});

test("buildBoostPickupTimelineRanges uses tagged boost pickup detection events", () => {
  const timeline = createLegacyStatsTimeline({
    boost_pickups: [
      {
        detection: "both",
        frame: 10,
        time: 1,
        player_id: { Steam: "blue-id" },
        is_team_0: true,
        pad_type: "big",
        field_half: "own",
        activity: "active",
        is_steal: false,
        collected_amount: 100,
        overfill_amount: 0,
        boost_before: 0,
        boost_after: 100,
      },
      {
        detection: "inferred_only",
        frame: 20,
        time: 2,
        player_id: { Steam: "orange-id" },
        is_team_0: false,
        pad_type: "small",
        field_half: "opponent",
        activity: "active",
        is_steal: true,
        collected_amount: 12,
        overfill_amount: 0,
        boost_before: null,
        boost_after: null,
      },
    ],
  });
  const replay = {
    frames: Array.from({ length: 21 }, (_, index) => ({
      time: index === 20 ? 2.25 : index / 10,
    })),
    players: [
      {
        id: "Steam:blue-id",
        name: "Blue",
        isTeamZero: true,
        cameraSettings: {},
        frames: [],
      },
      {
        id: "Steam:orange-id",
        name: "Orange",
        isTeamZero: false,
        cameraSettings: {},
        frames: [],
      },
    ],
    boostPads: [],
  } as Partial<ReplayModel> as ReplayModel;

  assert.deepEqual(
    buildBoostPickupTimelineRanges(timeline, replay, {
      detections: ["inferred_only"],
    }),
    [
      {
        id: "boost-pickup:inferred_only:20:Steam:orange-id:0",
        startTime: 2.25,
        endTime: 2.33,
        lane: "boost-pickups",
        laneLabel: "Boost Pickups",
        label: "Orange inferred small boost pickup",
        shortLabel: "I",
        color: "#f59e0b",
        isTeamZero: false,
      },
    ],
  );

  assert.deepEqual(
    buildBoostPickupTimelineRanges(timeline, replay, {
      padTypes: ["big"],
      detections: ["inferred_only"],
      playerIds: ["Steam:orange-id"],
    }),
    [],
  );

  assert.deepEqual(
    buildBoostPickupTimelineRanges(timeline, replay, {
      padTypes: ["small"],
      detections: ["inferred_only"],
      playerIds: ["Steam:orange-id"],
    }).map((range) => range.id),
    ["boost-pickup:inferred_only:20:Steam:orange-id:0"],
  );
});

test("buildBallHalfTimelineRanges uses replay centerline fallback for legacy half-control stats", () => {
  const timeline = {
    config: {
      most_back_forward_threshold_y: 400,
      ball_half_neutral_zone_half_width_y: 200,
    },
    replay_meta: {},
    timeline_events: [],
    frames: [
      {
        frame_number: 1,
        time: 0.5,
        dt: 0.5,
        team_zero: {
          ball_half: {
            tracked_time: 0.5,
            defensive_half_time: 0.5,
            offensive_half_time: 0,
            neutral_time: 0,
          },
        },
        team_one: {
          ball_half: {
            tracked_time: 0.5,
            defensive_half_time: 0,
            offensive_half_time: 0.5,
            neutral_time: 0,
          },
        },
        players: [],
      },
      {
        frame_number: 2,
        time: 1,
        dt: 0.5,
        team_zero: {
          ball_half: {
            tracked_time: 1,
            defensive_half_time: 0.5,
            offensive_half_time: 0.5,
            neutral_time: 0,
          },
        },
        team_one: {
          ball_half: {
            tracked_time: 1,
            defensive_half_time: 0.5,
            offensive_half_time: 0.5,
            neutral_time: 0,
          },
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

  assert.deepEqual(buildBallHalfTimelineRanges(timeline, replay), [
    {
      id: "half-control:team_zero_side:0.000",
      startTime: 0,
      endTime: 0.5,
      lane: "half-control",
      laneLabel: "Half Control",
      label: "Blue half control",
      color: "#3b82f6",
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
