import test from "node:test";
import assert from "node:assert/strict";

import { applyPowerslideEventDerivedStats } from "./powerslideEventDerivation.ts";
import { createStatsFrame, createStatsTimeline } from "./testStatsTimeline.ts";

const bluePlayer = { Steam: "blue-powerslide" } as Record<string, unknown>;
const orangePlayer = { Steam: "orange-powerslide" } as Record<string, unknown>;

function assertClose(actual: number | undefined, expected: number): void {
  assert.ok(actual != null && Math.abs(actual - expected) < 1e-9, `${actual} != ${expected}`);
}

test("powerslide event derivation can populate compacted player and team stats", () => {
  const timeline = createStatsTimeline({
    events: {
      powerslide: [
        {
          time: 1,
          frame: 10,
          player: bluePlayer,
          is_team_0: true,
          active: true,
        },
        {
          time: 1.2,
          frame: 12,
          player: orangePlayer,
          is_team_0: false,
          active: true,
        },
        {
          time: 2,
          frame: 20,
          player: bluePlayer,
          is_team_0: true,
          active: false,
        },
      ],
    },
    frames: [
      createStatsFrame({
        frame_number: 10,
        time: 1,
        dt: 0.1,
        gameplay_phase: "active_play",
        is_live_play: true,
        players: [
          { player_id: bluePlayer, is_team_0: true },
          { player_id: orangePlayer, is_team_0: false },
        ],
      }),
      createStatsFrame({
        frame_number: 12,
        time: 1.2,
        dt: 0.2,
        gameplay_phase: "kickoff_waiting_for_touch",
        is_live_play: false,
        players: [
          { player_id: bluePlayer, is_team_0: true },
          { player_id: orangePlayer, is_team_0: false },
        ],
      }),
      createStatsFrame({
        frame_number: 20,
        time: 2,
        dt: 0.3,
        gameplay_phase: "active_play",
        is_live_play: true,
        players: [
          { player_id: bluePlayer, is_team_0: true },
          { player_id: orangePlayer, is_team_0: false },
        ],
      }),
    ],
  });

  for (const frame of timeline.frames) {
    for (const stats of [
      frame.team_zero.powerslide,
      frame.team_one.powerslide,
      ...frame.players.map((player) => player.powerslide),
    ]) {
      for (const key of Object.keys(stats)) {
        delete (stats as Record<string, unknown>)[key];
      }
    }
  }

  applyPowerslideEventDerivedStats(timeline);

  assert.equal(timeline.frames[0]?.players[0]?.powerslide.press_count, 1);
  assertClose(timeline.frames[0]?.players[0]?.powerslide.total_duration, 0.1);
  assert.equal(timeline.frames[0]?.team_zero.powerslide.press_count, 1);
  assertClose(timeline.frames[0]?.team_zero.powerslide.total_duration, 0.1);

  assert.equal(timeline.frames[1]?.players[0]?.powerslide.press_count, 1);
  assertClose(timeline.frames[1]?.players[0]?.powerslide.total_duration, 0.3);
  assert.equal(timeline.frames[1]?.players[1]?.powerslide.press_count, 1);
  assertClose(timeline.frames[1]?.players[1]?.powerslide.total_duration, 0.2);
  assert.equal(timeline.frames[1]?.team_one.powerslide.press_count, 1);
  assertClose(timeline.frames[1]?.team_one.powerslide.total_duration, 0.2);

  assert.equal(timeline.frames[2]?.players[0]?.powerslide.press_count, 1);
  assertClose(timeline.frames[2]?.players[0]?.powerslide.total_duration, 0.3);
  assert.equal(timeline.frames[2]?.players[1]?.powerslide.press_count, 1);
  assertClose(timeline.frames[2]?.players[1]?.powerslide.total_duration, 0.5);
  assertClose(timeline.frames[2]?.team_zero.powerslide.total_duration, 0.3);
  assertClose(timeline.frames[2]?.team_one.powerslide.total_duration, 0.5);
});
