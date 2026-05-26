import test from "node:test";
import assert from "node:assert/strict";

import { applyDodgeResetEventDerivedStats } from "./dodgeResetEventDerivation.ts";
import { createStatsFrame, createStatsTimeline } from "./testStatsTimeline.ts";

const bluePlayer = { Steam: "blue-dodge-reset" } as Record<string, unknown>;
const orangePlayer = { Steam: "orange-dodge-reset" } as Record<string, unknown>;

test("dodge-reset event derivation can populate compacted player stats", () => {
  const timeline = createStatsTimeline({
    events: {
      dodge_reset: [
        {
          time: 1,
          frame: 10,
          player: bluePlayer,
          is_team_0: true,
          counter_value: 1,
          on_ball: true,
        },
        {
          time: 2,
          frame: 20,
          player: bluePlayer,
          is_team_0: true,
          counter_value: 2,
          on_ball: false,
        },
        {
          time: 3,
          frame: 30,
          player: orangePlayer,
          is_team_0: false,
          counter_value: 1,
          on_ball: false,
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
        frame_number: 20,
        time: 2,
        players: [
          { player_id: bluePlayer, is_team_0: true },
          { player_id: orangePlayer, is_team_0: false },
        ],
      }),
      createStatsFrame({
        frame_number: 30,
        time: 3,
        players: [
          { player_id: bluePlayer, is_team_0: true },
          { player_id: orangePlayer, is_team_0: false },
        ],
      }),
    ],
  });

  for (const frame of timeline.frames) {
    for (const player of frame.players) {
      for (const key of Object.keys(player.dodge_reset)) {
        delete (player.dodge_reset as Record<string, unknown>)[key];
      }
    }
  }

  applyDodgeResetEventDerivedStats(timeline);

  assert.equal(timeline.frames[0]?.players[0]?.dodge_reset.count, 1);
  assert.equal(timeline.frames[0]?.players[0]?.dodge_reset.on_ball_count, 1);
  assert.equal(timeline.frames[0]?.players[1]?.dodge_reset.count, 0);

  assert.equal(timeline.frames[1]?.players[0]?.dodge_reset.count, 2);
  assert.equal(timeline.frames[1]?.players[0]?.dodge_reset.on_ball_count, 1);

  assert.equal(timeline.frames[2]?.players[0]?.dodge_reset.count, 2);
  assert.equal(timeline.frames[2]?.players[1]?.dodge_reset.count, 1);
  assert.equal(timeline.frames[2]?.players[1]?.dodge_reset.on_ball_count, 0);
});
