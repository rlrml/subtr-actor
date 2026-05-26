import test from "node:test";
import assert from "node:assert/strict";

import { applyRotationEventDerivedStats } from "./rotationEventDerivation.ts";
import { createStatsFrame, createStatsTimeline } from "./testStatsTimeline.ts";

const playerId = { Steam: "rotation-player" } as Record<string, unknown>;

function assertClose(actual: number | undefined, expected: number): void {
  assert.ok(actual != null && Math.abs(actual - expected) < 1e-6, `${actual} != ${expected}`);
}

test("rotation event derivation populates compacted player and team stats", () => {
  const timeline = createStatsTimeline({
    events: {
      rotation_player: [
        {
          time: 1,
          frame: 10,
          player: playerId,
          is_team_0: true,
          active: true,
          became_first_man_count: 1,
          lost_first_man_count: 0,
          current_role_state: "first_man",
          current_depth_state: "ahead_of_play",
        },
        {
          time: 1.1,
          frame: 11,
          player: playerId,
          is_team_0: true,
          active: true,
          became_first_man_count: 0,
          lost_first_man_count: 1,
          current_role_state: "second_man",
          current_depth_state: "behind_play",
        },
      ],
      rotation_team: [
        {
          time: 1,
          frame: 10,
          is_team_0: true,
          first_man_changes_for_team: 1,
          rotation_count: 1,
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
        frame_number: 10,
        time: 1,
        dt: 0.1,
        players: [{ player_id: playerId, is_team_0: true, name: "Blue" }],
      }),
      createStatsFrame({
        frame_number: 11,
        time: 1.1,
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
  assertClose(derived.frames[1]!.players[0]!.rotation.tracked_time, 0.1);
  assertClose(derived.frames[1]!.players[0]!.rotation.time_first_man, 0.1);
  assert.equal(derived.frames[1]!.players[0]!.rotation.first_man_stint_count, 1);
  assert.equal(derived.frames[1]!.players[0]!.rotation.current_role_state, "first_man");
  assertClose(derived.frames[2]!.players[0]!.rotation.tracked_time, 0.3);
  assertClose(derived.frames[2]!.players[0]!.rotation.time_second_man, 0.2);
  assert.equal(derived.frames[2]!.players[0]!.rotation.lost_first_man_count, 1);
  assert.equal(derived.frames[2]!.players[0]!.rotation.current_depth_state, "behind_play");
});

test("rotation event derivation keeps first man stint through brief interruptions", () => {
  const frames = [10, 11, 12, 13, 14, 15, 16].map((frameNumber) =>
    createStatsFrame({
      frame_number: frameNumber,
      time: frameNumber / 10,
      dt: 0.1,
      players: [{ player_id: playerId, is_team_0: true, name: "Blue" }],
    }),
  );
  const timeline = createStatsTimeline({
    config: {
      rotation_first_man_debounce_seconds: 0.25,
    },
    events: {
      rotation_player: [
        {
          time: 1,
          frame: 10,
          player: playerId,
          is_team_0: true,
          active: true,
          became_first_man_count: 0,
          lost_first_man_count: 0,
          current_role_state: "first_man",
          current_depth_state: "level_with_play",
        },
        {
          time: 1.1,
          frame: 11,
          player: playerId,
          is_team_0: true,
          active: true,
          became_first_man_count: 0,
          lost_first_man_count: 0,
          current_role_state: "ambiguous",
          current_depth_state: "level_with_play",
        },
        {
          time: 1.2,
          frame: 12,
          player: playerId,
          is_team_0: true,
          active: true,
          became_first_man_count: 0,
          lost_first_man_count: 0,
          current_role_state: "first_man",
          current_depth_state: "level_with_play",
        },
        {
          time: 1.3,
          frame: 13,
          player: playerId,
          is_team_0: true,
          active: true,
          became_first_man_count: 0,
          lost_first_man_count: 0,
          current_role_state: "ambiguous",
          current_depth_state: "level_with_play",
        },
        {
          time: 1.6,
          frame: 16,
          player: playerId,
          is_team_0: true,
          active: true,
          became_first_man_count: 0,
          lost_first_man_count: 0,
          current_role_state: "first_man",
          current_depth_state: "level_with_play",
        },
      ],
    },
    frames,
  });

  const derived = applyRotationEventDerivedStats(timeline);

  assert.equal(derived.frames[0]!.players[0]!.rotation.first_man_stint_count, 1);
  assert.equal(derived.frames[2]!.players[0]!.rotation.first_man_stint_count, 1);
  assertClose(derived.frames[2]!.players[0]!.rotation.longest_first_man_stint_time, 0.2);
  assert.equal(derived.frames[6]!.players[0]!.rotation.first_man_stint_count, 2);
  assertClose(derived.frames[6]!.players[0]!.rotation.time_first_man, 0.3);
  assertClose(derived.frames[6]!.players[0]!.rotation.longest_first_man_stint_time, 0.2);
});
