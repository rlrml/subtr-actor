import test from "node:test";
import assert from "node:assert/strict";

import { applyDoubleTapEventDerivedStats } from "./doubleTapEventDerivation.ts";
import { createStatsFrame, createStatsTimeline } from "./testStatsTimeline.ts";

const bluePlayer = { Steam: "blue-double-tap" } as Record<string, unknown>;
const orangePlayer = { Steam: "orange-double-tap" } as Record<string, unknown>;

test("double-tap event derivation can populate compacted player and team stats", () => {
  const timeline = createStatsTimeline({
    events: {
      double_tap: [
        {
          time: 2,
          frame: 20,
          player: bluePlayer,
          is_team_0: true,
          backboard_time: 1.5,
          backboard_frame: 15,
        },
        {
          time: 2.2,
          frame: 22,
          player: orangePlayer,
          is_team_0: false,
          backboard_time: 1.8,
          backboard_frame: 18,
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
    for (const key of Object.keys(frame.team_zero.double_tap)) {
      delete (frame.team_zero.double_tap as Record<string, unknown>)[key];
    }
    for (const key of Object.keys(frame.team_one.double_tap)) {
      delete (frame.team_one.double_tap as Record<string, unknown>)[key];
    }
    for (const player of frame.players) {
      for (const key of Object.keys(player.double_tap)) {
        delete (player.double_tap as Record<string, unknown>)[key];
      }
    }
  }

  applyDoubleTapEventDerivedStats(timeline);

  assert.equal(timeline.frames[0]?.team_zero.double_tap.count, 1);
  assert.equal(timeline.frames[0]?.team_one.double_tap.count, 0);
  assert.equal(timeline.frames[0]?.players[0]?.double_tap.count, 1);
  assert.equal(timeline.frames[0]?.players[0]?.double_tap.is_last_double_tap, true);
  assert.equal(timeline.frames[0]?.players[1]?.double_tap.count, 0);

  assert.equal(timeline.frames[1]?.team_zero.double_tap.count, 1);
  assert.equal(timeline.frames[1]?.team_one.double_tap.count, 1);
  assert.equal(timeline.frames[1]?.players[0]?.double_tap.is_last_double_tap, false);
  assert.equal(timeline.frames[1]?.players[0]?.double_tap.frames_since_last_double_tap, 3);
  assert.equal(timeline.frames[1]?.players[1]?.double_tap.is_last_double_tap, true);
  assert.equal(timeline.frames[1]?.players[1]?.double_tap.frames_since_last_double_tap, 1);
});
