import test from "node:test";
import assert from "node:assert/strict";

import { applyMovementEventDerivedStats } from "./movementEventDerivation.ts";
import { createStatsFrame, createStatsTimeline } from "./testStatsTimeline.ts";

const bluePlayer = { Steam: "blue-movement" } as Record<string, unknown>;
const orangePlayer = { Steam: "orange-movement" } as Record<string, unknown>;

function assertClose(actual: number | undefined, expected: number): void {
  assert.ok(actual != null && Math.abs(actual - expected) < 1e-6, `${actual} != ${expected}`);
}

test("movement event derivation populates compacted player and team stats", () => {
  const timeline = createStatsTimeline({
    events: {
      movement: [
        {
          time: 1,
          frame: 10,
          player: bluePlayer,
          is_team_0: true,
          dt: 0.1,
          speed: 1000,
          distance: 100,
          speed_band: "slow",
          height_band: "ground",
        },
        {
          time: 1.1,
          frame: 11,
          player: bluePlayer,
          is_team_0: true,
          dt: 0.2,
          speed: 1500,
          distance: 300,
          speed_band: "boost",
          height_band: "low_air",
        },
        {
          time: 1.1,
          frame: 11,
          player: orangePlayer,
          is_team_0: false,
          dt: 0.2,
          speed: 2300,
          distance: 460,
          speed_band: "supersonic",
          height_band: "high_air",
        },
      ],
    },
    frames: [
      createStatsFrame({
        frame_number: 9,
        time: 0.9,
        players: [
          { player_id: bluePlayer, is_team_0: true, name: "Blue" },
          { player_id: orangePlayer, is_team_0: false, name: "Orange" },
        ],
      }),
      createStatsFrame({
        frame_number: 10,
        time: 1,
        players: [
          { player_id: bluePlayer, is_team_0: true, name: "Blue" },
          { player_id: orangePlayer, is_team_0: false, name: "Orange" },
        ],
      }),
      createStatsFrame({
        frame_number: 11,
        time: 1.1,
        players: [
          { player_id: bluePlayer, is_team_0: true, name: "Blue" },
          { player_id: orangePlayer, is_team_0: false, name: "Orange" },
        ],
      }),
    ],
  });

  for (const frame of timeline.frames) {
    for (const movement of [
      frame.team_zero.movement,
      frame.team_one.movement,
      frame.players[0]!.movement,
      frame.players[1]!.movement,
    ]) {
      Object.keys(movement).forEach((key) => {
        delete (movement as Record<string, unknown>)[key];
      });
    }
  }

  const derived = applyMovementEventDerivedStats(timeline);

  assert.equal(derived.frames[0]!.team_zero.movement.tracked_time, 0);
  assertClose(derived.frames[2]!.team_zero.movement.tracked_time, 0.3);
  assertClose(derived.frames[2]!.team_zero.movement.total_distance, 400);
  assertClose(derived.frames[2]!.team_zero.movement.speed_integral, 400);
  assertClose(derived.frames[2]!.team_zero.movement.time_slow_speed, 0.1);
  assertClose(derived.frames[2]!.team_zero.movement.time_boost_speed, 0.2);
  assertClose(derived.frames[2]!.team_zero.movement.time_on_ground, 0.1);
  assertClose(derived.frames[2]!.team_zero.movement.time_low_air, 0.2);
  assertClose(derived.frames[2]!.players[0]!.movement.total_distance, 400);
  assertClose(derived.frames[2]!.team_one.movement.time_supersonic_speed, 0.2);
  assertClose(derived.frames[2]!.players[1]!.movement.time_high_air, 0.2);

  const boostLowAir = derived.frames[2]!.players[0]!.movement.labeled_tracked_time?.entries.find(
    (entry) =>
      entry.labels.some((label) => label.key === "speed_band" && label.value === "boost") &&
      entry.labels.some((label) => label.key === "height_band" && label.value === "low_air"),
  );
  assertClose(boostLowAir?.value, 0.2);
});
