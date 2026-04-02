import test from "node:test";
import assert from "node:assert/strict";

import type { ReplayModel } from "subtr-actor-player";
import {
  buildBackboardTimelineEvents,
  buildBallCarryTimelineEvents,
  buildCeilingShotTimelineEvents,
  buildDodgeResetTimelineEvents,
  buildDoubleTapTimelineEvents,
  buildFiftyFiftyTimelineEvents,
  buildMustyFlickTimelineEvents,
  buildPowerslideTimelineEvents,
  buildRushTimelineEvents,
  buildSpeedFlipTimelineEvents,
  buildTouchTimelineEvents,
  countEnabledTimelineEvents,
  filterReplayTimelineEvents,
  getReplayTimelineEventKinds,
} from "./timelineMarkers.ts";
import { createLegacyStatsTimeline } from "./testStatsTimeline.ts";

test("timeline defaults to goals and adds core and demo replay events when enabled", () => {
  assert.deepEqual(getReplayTimelineEventKinds([]), ["goal"]);
  assert.deepEqual(
    getReplayTimelineEventKinds(["core", "demo"]),
    ["goal", "save", "shot", "demo"],
  );
});

test("filterReplayTimelineEvents keeps only goal markers by default", () => {
  const replay = {
    timelineEvents: [
      { kind: "goal", time: 10 },
      { kind: "save", time: 12 },
      { kind: "shot", time: 13 },
      { kind: "demo", time: 14 },
    ],
  } as ReplayModel;

  assert.deepEqual(
    filterReplayTimelineEvents(replay, []).map((event) => event.kind),
    ["goal"],
  );
  assert.deepEqual(
    filterReplayTimelineEvents(replay, ["core", "demo"]).map((event) => event.kind),
    ["goal", "save", "shot", "demo"],
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

test("buildRushTimelineEvents anchors rush markers to serialized rush event starts", () => {
  const replay = {
    frames: [
      { time: 0 },
      { time: 1.5 },
      { time: 2.25 },
      { time: 3.5 },
    ],
  } as ReplayModel;

  const statsTimeline = createLegacyStatsTimeline({
    rush_events: [
      {
        start_time: 1.1,
        start_frame: 1,
        end_time: 1.9,
        end_frame: 2,
        is_team_0: true,
        attackers: 2,
        defenders: 1,
      },
      {
        start_time: 2.1,
        start_frame: 2,
        end_time: 2.8,
        end_frame: 3,
        is_team_0: false,
        attackers: 3,
        defenders: 2,
      },
    ],
    frames: [
      {
        frame_number: 1,
        time: 1,
        dt: 1,
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
        players: [],
      },
      {
        frame_number: 2,
        time: 2,
        dt: 1,
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
            count: 1,
            two_v_one_count: 0,
            two_v_two_count: 0,
            two_v_three_count: 0,
            three_v_one_count: 0,
            three_v_two_count: 1,
            three_v_three_count: 0,
          },
        },
        players: [],
      },
      {
        frame_number: 3,
        time: 3,
        dt: 1,
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
            count: 1,
            two_v_one_count: 0,
            two_v_two_count: 0,
            two_v_three_count: 0,
            three_v_one_count: 0,
            three_v_two_count: 1,
            three_v_three_count: 0,
          },
        },
        players: [],
      },
    ],
  });

  assert.deepEqual(buildRushTimelineEvents(statsTimeline, replay), [
    {
      id: "rush:1:team_zero:0:2v1",
      time: 1.5,
      frame: 1,
      kind: "rush",
      label: "Blue rush 2v1",
      shortLabel: "2v1",
      isTeamZero: true,
      color: "#3b82f6",
    },
    {
      id: "rush:2:team_one:0:3v2",
      time: 2.25,
      frame: 2,
      kind: "rush",
      label: "Orange rush 3v2",
      shortLabel: "3v2",
      isTeamZero: false,
      color: "#f59e0b",
    },
  ]);
});

test("buildCeilingShotTimelineEvents maps serialized ceiling shots to timeline markers", () => {
  const replay = {
    frames: [
      { time: 0 },
      { time: 1.5 },
      { time: 2.25 },
    ],
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

test("buildMustyFlickTimelineEvents maps cumulative musty counts to timeline markers", () => {
  const replay = {
    frames: [
      { time: 0 },
      { time: 1.5 },
      { time: 2.25 },
    ],
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
    frames: [
      { time: 0 },
      { time: 1.5 },
    ],
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
    frames: [
      { time: 0 },
      { time: 1.5 },
    ],
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

test("buildTouchTimelineEvents maps touch overlay markers to timeline markers", () => {
  const replay = {
    frames: [
      { time: 0 },
      { time: 1.5 },
    ],
    ballFrames: [
      { position: { x: 0, y: 0, z: 0 } },
      { position: { x: 10, y: 20, z: 30 } },
    ],
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

test("buildDodgeResetTimelineEvents distinguishes on-ball resets", () => {
  const replay = {
    frames: [
      { time: 0 },
      { time: 1.5 },
      { time: 2.25 },
    ],
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
      id: "dodge-reset:1:Steam:blue-id:1:ball",
      time: 1.5,
      frame: 1,
      kind: "dodge-reset",
      label: "Blue ball reset",
      shortLabel: "BR",
      playerId: "Steam:blue-id",
      playerName: "Blue",
      isTeamZero: true,
      color: "#3b82f6",
    },
    {
      id: "dodge-reset:2:Steam:blue-id:2:air",
      time: 2.25,
      frame: 2,
      kind: "dodge-reset",
      label: "Blue dodge reset",
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
    frames: [
      { time: 0 },
      { time: 1.5 },
    ],
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
    frames: [
      { time: 0 },
      { time: 1.5 },
    ],
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
    frames: [
      { time: 0 },
      { time: 1.5 },
      { time: 2.25 },
    ],
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

test("countEnabledTimelineEvents includes enabled custom module markers", () => {
  const replay = {
    timelineEvents: [
      { kind: "goal", time: 10 },
      { kind: "save", time: 12 },
    ],
    frames: [
      { time: 0 },
      { time: 1.25 },
    ],
    ballFrames: [
      { position: { x: 0, y: 0, z: 0 } },
      { position: { x: 10, y: 20, z: 30 } },
    ],
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
  assert.equal(
    countEnabledTimelineEvents(["core", "fifty-fifty"], replay, statsTimeline),
    3,
  );
  assert.equal(
    countEnabledTimelineEvents(["core", "fifty-fifty", "rush"], replay, statsTimeline),
    4,
  );
  assert.equal(
    countEnabledTimelineEvents(
      ["core", "fifty-fifty", "rush", "musty-flick"],
      replay,
      statsTimeline,
    ),
    5,
  );
  assert.equal(
    countEnabledTimelineEvents(
      ["core", "fifty-fifty", "rush", "musty-flick", "ceiling-shot"],
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
        "double-tap",
        "touch",
        "dodge-reset",
        "ball-carry",
        "powerslide",
        "speed-flip",
      ],
      replay,
      statsTimeline,
    ),
    13,
  );
});
