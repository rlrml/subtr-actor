import test from "node:test";
import assert from "node:assert/strict";

import type { ReplayModel } from "subtr-actor-player";
import {
  buildSpeedFlipMarkers,
  getVisibleSpeedFlipMarkers,
} from "./speedFlipOverlay.ts";
import { createLegacyStatsTimeline } from "./testStatsTimeline.ts";

test("buildSpeedFlipMarkers uses replay-normalized frame time and player names", () => {
  const replay = {
    frameCount: 4,
    duration: 4,
    frames: [{ time: 0 }, { time: 1.5 }, { time: 2.5 }, { time: 3.5 }],
    ballFrames: [],
    boostPads: [],
    players: [
      {
        id: "Steam:blue-id",
        name: "Blue",
      },
    ],
    timelineEvents: [],
    teamZeroNames: ["Blue"],
    teamOneNames: [],
  } as unknown as ReplayModel;

  const statsTimeline = createLegacyStatsTimeline({
    speed_flip_events: [
      {
        time: 9.2,
        frame: 2,
        player: { Steam: "blue-id" },
        is_team_0: true,
        time_since_kickoff_start: 0.41,
        start_position: [100, -250, 17],
        end_position: [920, -40, 17],
        start_speed: 1230,
        max_speed: 1815,
        best_alignment: 0.94,
        diagonal_score: 0.82,
        cancel_score: 0.91,
        speed_score: 0.88,
        confidence: 0.86,
      },
    ],
  });

  assert.deepEqual(buildSpeedFlipMarkers(statsTimeline, replay), [
    {
      id: "speed-flip:2:Steam:blue-id:860",
      time: 2.5,
      frame: 2,
      isTeamZero: true,
      playerId: "Steam:blue-id",
      playerName: "Blue",
      position: { x: 100, y: -250, z: 17 },
      quality: 0.86,
      qualityLabel: "86%",
    },
  ]);
});

test("getVisibleSpeedFlipMarkers filters markers outside the decay window", () => {
  const markers = [
    {
      id: "speed-flip:1",
      time: 2,
      frame: 1,
      isTeamZero: true,
      playerId: "Steam:blue-id",
      playerName: "Blue",
      position: { x: 0, y: 0, z: 17 },
      quality: 0.8,
      qualityLabel: "80%",
    },
    {
      id: "speed-flip:2",
      time: 8.5,
      frame: 2,
      isTeamZero: false,
      playerId: "Steam:orange-id",
      playerName: "Orange",
      position: { x: 0, y: 0, z: 17 },
      quality: 0.62,
      qualityLabel: "62%",
    },
  ];

  assert.deepEqual(
    getVisibleSpeedFlipMarkers(markers, 10, 3).map((marker) => marker.id),
    ["speed-flip:2"],
  );
});
