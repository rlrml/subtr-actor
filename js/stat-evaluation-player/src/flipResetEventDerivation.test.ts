import test from "node:test";
import assert from "node:assert/strict";

import { applyFlipResetEventDerivedStats } from "./flipResetEventDerivation.ts";
import { createStatsFrame, createStatsTimeline } from "./testStatsTimeline.ts";

const bluePlayer = { Steam: "blue-flip-reset" } as Record<string, unknown>;
const orangePlayer = { Steam: "orange-flip-reset" } as Record<string, unknown>;

test("flip-reset event derivation can populate compacted confirmed player stats", () => {
  const timeline = createStatsTimeline({
    events: {
      flip_reset: [
        {
          time: 1.4,
          frame: 14,
          reset_time: 1,
          reset_frame: 10,
          player: bluePlayer,
          is_team_0: true,
          counter_value: 1,
          time_since_reset: 0.4,
        },
        {
          time: 3.25,
          frame: 32,
          reset_time: 3,
          reset_frame: 30,
          player: orangePlayer,
          is_team_0: false,
          counter_value: 1,
          time_since_reset: 0.25,
        },
      ],
    },
    frames: [
      createStatsFrame({
        frame_number: 10,
        time: 1,
        players: [
          { player_id: bluePlayer, is_team_0: true },
          { player_id: orangePlayer, is_team_0: false },
        ],
      }),
      createStatsFrame({
        frame_number: 14,
        time: 1.4,
        players: [
          { player_id: bluePlayer, is_team_0: true },
          { player_id: orangePlayer, is_team_0: false },
        ],
      }),
      createStatsFrame({
        frame_number: 32,
        time: 3.25,
        players: [
          { player_id: bluePlayer, is_team_0: true },
          { player_id: orangePlayer, is_team_0: false },
        ],
      }),
    ],
  });

  for (const frame of timeline.frames) {
    for (const player of frame.players) {
      for (const key of Object.keys(player.flip_reset)) {
        delete (player.flip_reset as Record<string, unknown>)[key];
      }
    }
  }

  applyFlipResetEventDerivedStats(timeline);

  assert.equal(timeline.frames[0]?.players[0]?.flip_reset.count, 0);
  assert.equal(timeline.frames[1]?.players[0]?.flip_reset.count, 1);
  assert.equal(timeline.frames[1]?.players[0]?.flip_reset.total_time_to_use, 0.4);
  assert.equal(timeline.frames[1]?.players[0]?.flip_reset.min_time_to_use, 0.4);
  assert.equal(timeline.frames[2]?.players[1]?.flip_reset.count, 1);
  assert.equal(timeline.frames[2]?.players[1]?.flip_reset.total_time_to_use, 0.25);
  assert.equal(timeline.frames[2]?.players[1]?.flip_reset.min_time_to_use, 0.25);
});
