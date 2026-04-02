import test from "node:test";
import assert from "node:assert/strict";

import type { ReplayModel } from "subtr-actor-player";
import {
  buildCeilingShotMarkers,
  getVisibleCeilingShotMarkers,
} from "./ceilingShotOverlay.ts";
import { createLegacyStatsTimeline } from "./testStatsTimeline.ts";

test("buildCeilingShotMarkers uses replay-normalized frame time and player names", () => {
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
    ceiling_shot_events: [
      {
        time: 9.2,
        frame: 2,
        player: { Steam: "blue-id" },
        is_team_0: true,
        ceiling_contact_time: 8.8,
        ceiling_contact_frame: 1,
        time_since_ceiling_contact: 0.4,
        ceiling_contact_position: [0, -900, 1988],
        touch_position: [120, -720, 1580],
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

  assert.deepEqual(buildCeilingShotMarkers(statsTimeline, replay), [
    {
      id: "ceiling-shot:2:Steam:blue-id:840",
      time: 2.5,
      frame: 2,
      isTeamZero: true,
      playerId: "Steam:blue-id",
      playerName: "Blue",
      ceilingContactPosition: { x: 0, y: -900, z: 1988 },
      touchPosition: { x: 120, y: -720, z: 1580 },
      quality: 0.84,
      qualityLabel: "84%",
    },
  ]);
});

test("getVisibleCeilingShotMarkers filters markers outside the decay window", () => {
  const markers = [
    {
      id: "ceiling-shot:1",
      time: 2,
      frame: 1,
      isTeamZero: true,
      playerId: "Steam:blue-id",
      playerName: "Blue",
      ceilingContactPosition: { x: 0, y: -900, z: 1988 },
      touchPosition: { x: 120, y: -720, z: 1580 },
      quality: 0.8,
      qualityLabel: "80%",
    },
    {
      id: "ceiling-shot:2",
      time: 8.5,
      frame: 2,
      isTeamZero: false,
      playerId: "Steam:orange-id",
      playerName: "Orange",
      ceilingContactPosition: { x: 0, y: 900, z: 1988 },
      touchPosition: { x: -120, y: 720, z: 1580 },
      quality: 0.62,
      qualityLabel: "62%",
    },
  ];

  assert.deepEqual(
    getVisibleCeilingShotMarkers(markers, 10, 3).map((marker) => marker.id),
    ["ceiling-shot:2"],
  );
});
