import test from "node:test";
import assert from "node:assert/strict";

import { applyBackboardEventDerivedStats } from "./backboardEventDerivation.ts";
import { createStatsFrame, createStatsTimeline } from "./testStatsTimeline.ts";

const bluePlayer = { Steam: "blue-backboard" } as Record<string, unknown>;
const orangePlayer = { Steam: "orange-backboard" } as Record<string, unknown>;

test("backboard event derivation can populate compacted player and team stats", () => {
  const timeline = createStatsTimeline({
    events: {
      backboard: [
        {
          time: 2,
          frame: 20,
          player: bluePlayer,
          is_team_0: true,
        },
        {
          time: 2.2,
          frame: 22,
          player: orangePlayer,
          is_team_0: false,
        },
      ],
    },
    frames: [
      createStatsFrame({
        frame_number: 20,
        time: 2,
        players: [
          { player_id: bluePlayer, is_team_0: true },
          { player_id: orangePlayer, is_team_0: false },
        ],
      }),
      createStatsFrame({
        frame_number: 23,
        time: 2.3,
        players: [
          { player_id: bluePlayer, is_team_0: true },
          { player_id: orangePlayer, is_team_0: false },
        ],
      }),
    ],
  });

  for (const frame of timeline.frames) {
    for (const key of Object.keys(frame.team_zero.backboard)) {
      delete (frame.team_zero.backboard as Record<string, unknown>)[key];
    }
    for (const key of Object.keys(frame.team_one.backboard)) {
      delete (frame.team_one.backboard as Record<string, unknown>)[key];
    }
    for (const player of frame.players) {
      for (const key of Object.keys(player.backboard)) {
        delete (player.backboard as Record<string, unknown>)[key];
      }
    }
  }

  applyBackboardEventDerivedStats(timeline);

  assert.equal(timeline.frames[0]?.team_zero.backboard.count, 1);
  assert.equal(timeline.frames[0]?.team_one.backboard.count, 0);
  assert.equal(timeline.frames[0]?.players[0]?.backboard.count, 1);
  assert.equal(timeline.frames[0]?.players[0]?.backboard.is_last_backboard, true);
  assert.equal(timeline.frames[0]?.players[1]?.backboard.count, 0);

  assert.equal(timeline.frames[1]?.team_zero.backboard.count, 1);
  assert.equal(timeline.frames[1]?.team_one.backboard.count, 1);
  assert.equal(timeline.frames[1]?.players[0]?.backboard.is_last_backboard, false);
  assert.equal(timeline.frames[1]?.players[0]?.backboard.frames_since_last_backboard, 3);
  assert.equal(timeline.frames[1]?.players[1]?.backboard.is_last_backboard, true);
  assert.equal(timeline.frames[1]?.players[1]?.backboard.frames_since_last_backboard, 1);
});
