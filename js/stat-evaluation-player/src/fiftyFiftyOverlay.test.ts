import test from "node:test";
import assert from "node:assert/strict";

import type { ReplayModel } from "subtr-actor-player";
import {
  buildFiftyFiftyMarkers,
  getVisibleFiftyFiftyMarkers,
} from "./fiftyFiftyOverlay.ts";
import { createLegacyStatsTimeline } from "./testStatsTimeline.ts";

test("buildFiftyFiftyMarkers anchors 50/50 markers to the challenge start", () => {
  const replay = {
    frameCount: 41,
    duration: 40,
    frames: Array.from({ length: 41 }, (_, time) => ({ time })),
    ballFrames: [],
    boostPads: [],
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
    timelineEvents: [],
    teamZeroNames: ["Blue"],
    teamOneNames: ["Orange"],
  } as unknown as ReplayModel;

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

  const [marker] = buildFiftyFiftyMarkers(statsTimeline, replay);

  assert.ok(marker);
  assert.equal(marker.id, "fifty-fifty:20:Steam:blue-id:Steam:orange-id");
  assert.equal(marker.time, 20);
  assert.equal(marker.frame, 20);
  assert.equal(marker.label, "50/50: Blue vs Orange | blue win | blue poss");
  assert.equal(marker.labelClassName, "sap-fifty-fifty-overlay-label-blue");
  assert.deepEqual(marker.axisStart.toArray(), [0, 0, 0]);
  assert.deepEqual(marker.axisEnd.toArray(), [10, 0, 0]);
  assert.deepEqual(marker.midpoint.toArray(), [5, 0, 0]);
  assert.equal(marker.winnerIsTeamZero, true);
});

test("buildFiftyFiftyMarkers falls back to stats timeline start_time when frame time is missing", () => {
  const replay = {
    frameCount: 0,
    duration: 0,
    frames: [],
    ballFrames: [],
    boostPads: [],
    players: [],
    timelineEvents: [],
    teamZeroNames: [],
    teamOneNames: [],
  } as unknown as ReplayModel;

  const statsTimeline = createLegacyStatsTimeline({
    fifty_fifty_events: [
      {
        start_time: 1.25,
        start_frame: 20,
        resolve_time: 2,
        resolve_frame: 40,
        is_kickoff: false,
        team_zero_position: [0, 0, 0],
        team_one_position: [10, 0, 0],
        midpoint: [5, 0, 0],
        plane_normal: [1, 0, 0],
      },
    ],
  });

  assert.equal(buildFiftyFiftyMarkers(statsTimeline, replay)[0]?.time, 1.25);
});

test("getVisibleFiftyFiftyMarkers makes markers visible from their start time", () => {
  const visible = getVisibleFiftyFiftyMarkers(
    [
      {
        id: "fifty-fifty:20",
        time: 2,
        frame: 20,
        label: "marker",
        labelClassName: "sap-fifty-fifty-overlay-label-neutral",
        axisStart: {} as never,
        axisEnd: {} as never,
        midpoint: {} as never,
        winnerIsTeamZero: null,
      },
    ],
    2.1,
    1,
  );

  assert.equal(visible.length, 1);
});
