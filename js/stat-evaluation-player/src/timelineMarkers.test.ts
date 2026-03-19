import test from "node:test";
import assert from "node:assert/strict";

import type { ReplayModel } from "subtr-actor-player";
import type { StatsTimeline } from "./statsTimeline.ts";
import {
  buildFiftyFiftyTimelineEvents,
  buildRushTimelineEvents,
  countEnabledTimelineEvents,
  filterReplayTimelineEvents,
  getReplayTimelineEventKinds,
} from "./timelineMarkers.ts";

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

  const statsTimeline = {
    replay_meta: {},
    timeline_events: [],
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
    frames: [],
  } as StatsTimeline;

  assert.deepEqual(buildFiftyFiftyTimelineEvents(statsTimeline, replay), [
    {
      id: "fifty-fifty:20:Steam:blue-id:Steam:orange-id",
      time: 2,
      kind: "fifty-fifty",
      label: "50/50: Blue vs Orange | blue win | blue poss",
      shortLabel: "50",
      isTeamZero: true,
      color: "#3b82f6",
    },
  ]);
});

test("buildRushTimelineEvents maps cumulative rush counts to discrete timeline markers", () => {
  const replay = {
    frames: [
      { time: 0 },
      { time: 1.5 },
      { time: 2.25 },
      { time: 3.5 },
    ],
  } as ReplayModel;

  const statsTimeline = {
    replay_meta: {},
    timeline_events: [],
    frames: [
      {
        frame_number: 1,
        time: 1,
        dt: 1,
        rush: {
          team_zero_count: 1,
          team_zero_two_v_one_count: 1,
          team_zero_two_v_two_count: 0,
          team_zero_two_v_three_count: 0,
          team_zero_three_v_one_count: 0,
          team_zero_three_v_two_count: 0,
          team_zero_three_v_three_count: 0,
          team_one_count: 0,
          team_one_two_v_one_count: 0,
          team_one_two_v_two_count: 0,
          team_one_two_v_three_count: 0,
          team_one_three_v_one_count: 0,
          team_one_three_v_two_count: 0,
          team_one_three_v_three_count: 0,
        },
        players: [],
      },
      {
        frame_number: 2,
        time: 2,
        dt: 1,
        rush: {
          team_zero_count: 1,
          team_zero_two_v_one_count: 1,
          team_zero_two_v_two_count: 0,
          team_zero_two_v_three_count: 0,
          team_zero_three_v_one_count: 0,
          team_zero_three_v_two_count: 0,
          team_zero_three_v_three_count: 0,
          team_one_count: 1,
          team_one_two_v_one_count: 0,
          team_one_two_v_two_count: 0,
          team_one_two_v_three_count: 0,
          team_one_three_v_one_count: 0,
          team_one_three_v_two_count: 1,
          team_one_three_v_three_count: 0,
        },
        players: [],
      },
      {
        frame_number: 3,
        time: 3,
        dt: 1,
        rush: {
          team_zero_count: 1,
          team_zero_two_v_one_count: 1,
          team_zero_two_v_two_count: 0,
          team_zero_two_v_three_count: 0,
          team_zero_three_v_one_count: 0,
          team_zero_three_v_two_count: 0,
          team_zero_three_v_three_count: 0,
          team_one_count: 1,
          team_one_two_v_one_count: 0,
          team_one_two_v_two_count: 0,
          team_one_two_v_three_count: 0,
          team_one_three_v_one_count: 0,
          team_one_three_v_two_count: 1,
          team_one_three_v_three_count: 0,
        },
        players: [],
      },
    ],
  } as StatsTimeline;

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

  const statsTimeline = {
    replay_meta: {},
    timeline_events: [],
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
    frames: [
      {
        frame_number: 1,
        time: 1.25,
        dt: 0.1,
        rush: {
          team_zero_count: 1,
          team_zero_two_v_one_count: 1,
          team_zero_two_v_two_count: 0,
          team_zero_two_v_three_count: 0,
          team_zero_three_v_one_count: 0,
          team_zero_three_v_two_count: 0,
          team_zero_three_v_three_count: 0,
          team_one_count: 0,
          team_one_two_v_one_count: 0,
          team_one_two_v_two_count: 0,
          team_one_two_v_three_count: 0,
          team_one_three_v_one_count: 0,
          team_one_three_v_two_count: 0,
          team_one_three_v_three_count: 0,
        },
        players: [],
      },
    ],
  } as StatsTimeline;

  assert.equal(countEnabledTimelineEvents([], replay, statsTimeline), 1);
  assert.equal(
    countEnabledTimelineEvents(["core", "fifty-fifty"], replay, statsTimeline),
    3,
  );
  assert.equal(
    countEnabledTimelineEvents(["core", "fifty-fifty", "rush"], replay, statsTimeline),
    4,
  );
});
