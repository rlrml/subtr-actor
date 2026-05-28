import test from "node:test";
import assert from "node:assert/strict";

import type { ReplayModel } from "@rlrml/player";
import {
  buildMechanicPlaylistEvents,
  buildMechanicTimelineEvents,
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
