import test from "node:test";
import assert from "node:assert/strict";

import { applyRotationEventDerivedStats } from "./rotationEventDerivation.ts";
import { createStatsFrame, createStatsTimeline } from "./testStatsTimeline.ts";

const playerId = { Steam: "rotation-player" } as Record<string, unknown>;
const nextPlayerId = { Steam: "rotation-next-player" } as Record<string, unknown>;

function assertClose(actual: number | undefined, expected: number): void {
  assert.ok(actual != null && Math.abs(actual - expected) < 1e-6, `${actual} != ${expected}`);
}

test("rotation event derivation populates compacted player and team stats", () => {
  const timeline = createStatsTimeline({
    events: {
      rotation_role: [
        {
          time: 1,
          frame: 10,
          end_time: 1.1,
          end_frame: 11,
          duration: 0.1,
          player: playerId,
          is_team_0: true,
          state: "first_man",
        },
        {
          time: 1.1,
          frame: 11,
          end_time: 1.3,
          end_frame: 13,
          duration: 0.2,
          player: playerId,
          is_team_0: true,
          state: "second_man",
        },
      ],
      first_man_change: [
        {
          time: 1.1,
          frame: 11,
          is_team_0: true,
          previous_first_man: playerId,
          next_first_man: nextPlayerId,
        },
      ],
    },
    frames: [
      createStatsFrame({
        frame_number: 9,
        time: 0.9,
        players: [{ player_id: playerId, is_team_0: true, name: "Blue" }],
      }),
      createStatsFrame({
        frame_number: 11,
        time: 1.1,
        dt: 0.1,
        players: [{ player_id: playerId, is_team_0: true, name: "Blue" }],
      }),
      createStatsFrame({
        frame_number: 13,
        time: 1.3,
        dt: 0.2,
        players: [{ player_id: playerId, is_team_0: true, name: "Blue" }],
      }),
    ],
  });

  for (const frame of timeline.frames) {
    Object.keys(frame.team_zero.rotation).forEach((key) => {
      delete (frame.team_zero.rotation as Record<string, unknown>)[key];
    });
    Object.keys(frame.team_one.rotation).forEach((key) => {
      delete (frame.team_one.rotation as Record<string, unknown>)[key];
    });
    Object.keys(frame.players[0]!.rotation).forEach((key) => {
      delete (frame.players[0]!.rotation as Record<string, unknown>)[key];
    });
  }

  const derived = applyRotationEventDerivedStats(timeline);

  assert.equal(derived.frames[0]!.team_zero.rotation.rotation_count, 0);
  assert.equal(derived.frames[1]!.team_zero.rotation.rotation_count, 1);
  assert.equal(derived.frames[2]!.team_zero.rotation.first_man_changes_for_team, 1);
  assertClose(derived.frames[1]!.players[0]!.rotation.active_game_time, 0.1);
  assertClose(derived.frames[1]!.players[0]!.rotation.time_first_man, 0.1);
  assert.equal(derived.frames[1]!.players[0]!.rotation.first_man_stint_count, 1);
  assert.equal(derived.frames[1]!.players[0]!.rotation.current_role_state, "first_man");
  assertClose(derived.frames[2]!.players[0]!.rotation.active_game_time, 0.3);
  assertClose(derived.frames[2]!.players[0]!.rotation.time_second_man, 0.2);
  assert.equal(derived.frames[2]!.players[0]!.rotation.lost_first_man_count, 1);
  assert.equal(derived.frames[2]!.players[0]!.rotation.current_role_state, "second_man");
});

test("rotation event derivation keeps first man stint through brief interruptions", () => {
  const frames = [10, 11, 12, 13, 14, 15, 16, 17].map((frameNumber) =>
    createStatsFrame({
      frame_number: frameNumber,
      time: frameNumber / 10,
      dt: 0.1,
      players: [{ player_id: playerId, is_team_0: true, name: "Blue" }],
    }),
  );
  const roleSpan = (
    startFrame: number,
    endFrame: number,
    state: "first_man" | "ambiguous",
  ): Record<string, unknown> => ({
    time: startFrame / 10,
    frame: startFrame,
    end_time: endFrame / 10,
    end_frame: endFrame,
    duration: (endFrame - startFrame) / 10,
    player: playerId,
    is_team_0: true,
    state,
  });
  const timeline = createStatsTimeline({
    config: {
      rotation_first_man_stint_end_grace_seconds: 0.25,
    },
    events: {
      rotation_role: [
        // A short ambiguous gap (0.1s <= 0.25s grace) keeps the stint alive; the
        // longer 0.3s gap before the final span starts a new stint.
        roleSpan(10, 11, "first_man"),
        roleSpan(11, 12, "ambiguous"),
        roleSpan(12, 13, "first_man"),
        roleSpan(13, 16, "ambiguous"),
        roleSpan(16, 17, "first_man"),
      ],
    },
    frames,
  });

  const derived = applyRotationEventDerivedStats(timeline);

  assert.equal(derived.frames[1]!.players[0]!.rotation.first_man_stint_count, 1);
  assert.equal(derived.frames[3]!.players[0]!.rotation.first_man_stint_count, 1);
  assertClose(derived.frames[3]!.players[0]!.rotation.longest_first_man_stint_time, 0.2);
  assert.equal(derived.frames[7]!.players[0]!.rotation.first_man_stint_count, 2);
  assertClose(derived.frames[7]!.players[0]!.rotation.time_first_man, 0.3);
  assertClose(derived.frames[7]!.players[0]!.rotation.longest_first_man_stint_time, 0.2);
});
