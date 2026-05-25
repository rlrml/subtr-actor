import test from "node:test";
import assert from "node:assert/strict";

import { applyPositioningEventDerivedStats } from "./positioningEventDerivation.ts";
import { createStatsFrame, createStatsTimeline } from "./testStatsTimeline.ts";

const playerId = { Steam: "positioning-player" } as Record<string, unknown>;

function assertClose(actual: number | undefined, expected: number): void {
  assert.ok(actual != null && Math.abs(actual - expected) < 1e-9, `${actual} != ${expected}`);
}

test("positioning event derivation populates compacted player stats", () => {
  const timeline = createStatsTimeline({
    events: {
      positioning: [
        {
          time: 1,
          frame: 10,
          player: playerId,
          is_team_0: true,
          active_game_time: 0.1,
          tracked_time: 0.1,
          sum_distance_to_teammates: 100,
          sum_distance_to_ball: 200,
          sum_distance_to_ball_has_possession: 200,
          time_has_possession: 0.1,
          sum_distance_to_ball_no_possession: 0,
          time_no_possession: 0,
          time_demolished: 0,
          time_no_teammates: 0,
          time_most_back: 0.1,
          time_most_forward: 0,
          time_mid_role: 0,
          time_other_role: 0,
          time_defensive_third: 0.05,
          time_neutral_third: 0.05,
          time_offensive_third: 0,
          time_defensive_half: 0.08,
          time_offensive_half: 0.02,
          time_closest_to_ball: 0.1,
          time_farthest_from_ball: 0,
          time_behind_ball: 0.06,
          time_level_with_ball: 0.04,
          time_in_front_of_ball: 0,
          times_caught_ahead_of_play_on_conceded_goals: 0,
        },
        {
          time: 1.1,
          frame: 11,
          player: playerId,
          is_team_0: true,
          active_game_time: 0.2,
          tracked_time: 0,
          sum_distance_to_teammates: 0,
          sum_distance_to_ball: 0,
          sum_distance_to_ball_has_possession: 0,
          time_has_possession: 0,
          sum_distance_to_ball_no_possession: 0,
          time_no_possession: 0,
          time_demolished: 0.2,
          time_no_teammates: 0,
          time_most_back: 0,
          time_most_forward: 0,
          time_mid_role: 0,
          time_other_role: 0,
          time_defensive_third: 0,
          time_neutral_third: 0,
          time_offensive_third: 0,
          time_defensive_half: 0,
          time_offensive_half: 0,
          time_closest_to_ball: 0,
          time_farthest_from_ball: 0,
          time_behind_ball: 0,
          time_level_with_ball: 0,
          time_in_front_of_ball: 0,
          times_caught_ahead_of_play_on_conceded_goals: 1,
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
    Object.keys(frame.players[0]!.positioning).forEach((key) => {
      delete (frame.players[0]!.positioning as Record<string, unknown>)[key];
    });
  }

  const derived = applyPositioningEventDerivedStats(timeline);

  assert.equal(derived.frames[0]!.players[0]!.positioning.tracked_time, 0);
  assertClose(derived.frames[1]!.players[0]!.positioning.active_game_time, 0.1);
  assertClose(derived.frames[1]!.players[0]!.positioning.sum_distance_to_ball, 200);
  assertClose(derived.frames[1]!.players[0]!.positioning.time_defensive_third, 0.05);
  assertClose(derived.frames[1]!.players[0]!.positioning.time_closest_to_ball, 0.1);
  assertClose(derived.frames[2]!.players[0]!.positioning.active_game_time, 0.3);
  assertClose(derived.frames[2]!.players[0]!.positioning.time_demolished, 0.2);
  assert.equal(
    derived.frames[2]!.players[0]!.positioning.times_caught_ahead_of_play_on_conceded_goals,
    1,
  );
});
