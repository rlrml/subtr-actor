import test from "node:test";
import assert from "node:assert/strict";

import { applyDemoEventDerivedStats } from "./demoEventDerivation.ts";
import { createStatsFrame, createStatsTimeline } from "./testStatsTimeline.ts";

const bluePlayer = { Steam: "blue-demo" } as Record<string, unknown>;
const orangePlayer = { Steam: "orange-demo" } as Record<string, unknown>;

test("demo event derivation can populate compacted player and team stats", () => {
  const timeline = createStatsTimeline({
    events: {
      demolition: [
        {
          time: 2,
          frame: 20,
          attacker: bluePlayer,
          victim: orangePlayer,
          attacker_is_team_0: true,
          victim_is_team_0: false,
        },
        {
          time: 3,
          frame: 30,
          attacker: orangePlayer,
          victim: bluePlayer,
          attacker_is_team_0: false,
          victim_is_team_0: true,
        },
      ],
    },
    frames: [
      createStatsFrame({
        frame_number: 19,
        time: 1.9,
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
    for (const key of Object.keys(frame.team_zero.demo)) {
      delete (frame.team_zero.demo as Record<string, unknown>)[key];
    }
    for (const key of Object.keys(frame.team_one.demo)) {
      delete (frame.team_one.demo as Record<string, unknown>)[key];
    }
    for (const player of frame.players) {
      for (const key of Object.keys(player.demo)) {
        delete (player.demo as Record<string, unknown>)[key];
      }
    }
  }

  applyDemoEventDerivedStats(timeline);

  assert.equal(timeline.frames[0]?.team_zero.demo.demos_inflicted, 0);
  assert.equal(timeline.frames[1]?.team_zero.demo.demos_inflicted, 1);
  assert.equal(timeline.frames[1]?.players[0]?.demo.demos_inflicted, 1);
  assert.equal(timeline.frames[1]?.players[1]?.demo.demos_taken, 1);
  assert.equal(timeline.frames[2]?.team_one.demo.demos_inflicted, 1);
  assert.equal(timeline.frames[2]?.players[0]?.demo.demos_taken, 1);
  assert.equal(timeline.frames[2]?.players[1]?.demo.demos_inflicted, 1);
});
