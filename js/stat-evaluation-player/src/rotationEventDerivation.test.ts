import test from "node:test";
import assert from "node:assert/strict";

import { applyRotationEventDerivedStats } from "./rotationEventDerivation.ts";
import { createStatsFrame, createStatsTimeline } from "./testStatsTimeline.ts";

const playerId = { Steam: "rotation-player" } as Record<string, unknown>;

function assertClose(actual: number | undefined, expected: number): void {
  assert.ok(actual != null && Math.abs(actual - expected) < 1e-9, `${actual} != ${expected}`);
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
          active_game_time: 0.1,
          tracked_time: 0.1,
          time_first_man: 0.1,
          time_second_man: 0,
          time_third_man: 0,
          time_ambiguous_role: 0,
          time_behind_play: 0,
          time_level_with_play: 0.04,
          time_ahead_of_play: 0.06,
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
          active_game_time: 0.2,
          tracked_time: 0.2,
          time_first_man: 0,
          time_second_man: 0.2,
          time_third_man: 0,
          time_ambiguous_role: 0,
          time_behind_play: 0.2,
          time_level_with_play: 0,
          time_ahead_of_play: 0,
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
        players: [{ player_id: playerId, is_team_0: true, name: "Blue" }],
      }),
      createStatsFrame({
        frame_number: 11,
        time: 1.1,
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
  assert.equal(derived.frames[1]!.players[0]!.rotation.current_role_state, "first_man");
  assertClose(derived.frames[2]!.players[0]!.rotation.tracked_time, 0.3);
  assertClose(derived.frames[2]!.players[0]!.rotation.time_second_man, 0.2);
  assert.equal(derived.frames[2]!.players[0]!.rotation.lost_first_man_count, 1);
  assert.equal(derived.frames[2]!.players[0]!.rotation.current_depth_state, "behind_play");
});
