import test from "node:test";
import assert from "node:assert/strict";

import type { ReplayModel } from "../../player/src/types.ts";
import type { StatsFrame, StatsTimeline } from "./statsTimeline.ts";
import {
  buildTouchMarkers,
  getLastTouchPlayer,
  getVisibleTouchMarkers,
} from "./touchOverlay.ts";

test("getLastTouchPlayer returns the player marked as the current last touch", () => {
  const statsFrame: StatsFrame = {
    frame_number: 42,
    time: 12.3,
    dt: 0.1,
    players: [
      {
        player_id: { Steam: "blue" },
        name: "Blue",
        is_team_0: true,
        touch: {
          touch_count: 2,
          is_last_touch: false,
        },
      },
      {
        player_id: { Steam: "orange" },
        name: "Orange",
        is_team_0: false,
        touch: {
          touch_count: 3,
          is_last_touch: true,
          time_since_last_touch: 0.4,
        },
      },
    ],
  };

  assert.equal(getLastTouchPlayer(statsFrame)?.name, "Orange");
});

test("buildTouchMarkers derives markers from touch stats and ball frames", () => {
  const replay = {
    ballFrames: [
      { position: { x: 0, y: 0, z: 0 } },
      { position: { x: 100, y: -250, z: 320 } },
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
    frames: [
      {
        frame_number: 0,
        time: 0,
        dt: 0.1,
        players: [
          {
            player_id: { Steam: "blue-id" },
            name: "Blue",
            is_team_0: true,
            touch: {
              touch_count: 0,
              is_last_touch: false,
            },
          },
        ],
      },
      {
        frame_number: 1,
        time: 1.25,
        dt: 0.1,
        players: [
          {
            player_id: { Steam: "blue-id" },
            name: "Blue",
            is_team_0: true,
            touch: {
              touch_count: 1,
              is_last_touch: true,
              last_touch_time: 1.25,
              last_touch_frame: 1,
            },
          },
        ],
      },
    ],
  } as StatsTimeline;

  assert.deepEqual(buildTouchMarkers(statsTimeline, replay), [
    {
      id: "touch-stat:1:Steam:blue-id:1",
      time: 1.25,
      frame: 1,
      isTeamZero: true,
      playerId: "Steam:blue-id",
      playerName: "Blue",
      position: { x: 100, y: -250, z: 320 },
    },
  ]);
});

test("buildTouchMarkers uses normalized replay frame time instead of raw stats time", () => {
  const replay = {
    ballFrames: [
      { position: { x: 0, y: 0, z: 0 } },
      { position: { x: 100, y: -250, z: 320 } },
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
    ],
  } as ReplayModel;

  const statsTimeline = {
    replay_meta: {},
    timeline_events: [],
    frames: [
      {
        frame_number: 1,
        time: 4.75,
        dt: 0.1,
        players: [
          {
            player_id: { Steam: "blue-id" },
            name: "Blue",
            is_team_0: true,
            touch: {
              touch_count: 1,
              is_last_touch: true,
              last_touch_time: 4.75,
              last_touch_frame: 1,
            },
          },
        ],
      },
    ],
  } as StatsTimeline;

  assert.equal(buildTouchMarkers(statsTimeline, replay)[0]?.time, 1.25);
});

test("getVisibleTouchMarkers returns markers inside the decay window", () => {
  const markers = [
    {
      id: "touch:1",
      time: 2,
      frame: 20,
      isTeamZero: true,
      playerId: "Steam:blue-id",
      playerName: "Blue",
      position: { x: 0, y: 0, z: 0 },
    },
    {
      id: "touch:2",
      time: 8,
      frame: 80,
      isTeamZero: false,
      playerId: "Steam:orange-id",
      playerName: "Orange",
      position: { x: 0, y: 0, z: 0 },
    },
  ];

  assert.deepEqual(
    getVisibleTouchMarkers(markers, 10, 5).map((marker) => marker.id),
    ["touch:2"],
  );
});
