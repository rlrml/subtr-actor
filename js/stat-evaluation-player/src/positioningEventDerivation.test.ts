import test from "node:test";
import assert from "node:assert/strict";

import { applyPositioningEventDerivedStats } from "./positioningEventDerivation.ts";
import { createStatsFrame, createStatsTimeline } from "./testStatsTimeline.ts";

const playerId = { Steam: "positioning-player" } as Record<string, unknown>;

function assertClose(actual: number | undefined, expected: number): void {
  assert.ok(actual != null && Math.abs(actual - expected) < 1e-6, `${actual} != ${expected}`);
}

test("positioning event derivation populates compacted player stats", () => {
  const timeline = createStatsTimeline({
    events: {
      positioning: [
        {
          time: 1,
          frame: 10,
          duration: 0.1,
          player: playerId,
          is_team_0: true,
          active: true,
          tracked: true,
          distance_to_teammates: 1000,
          distance_to_ball: 2000,
          possession_state: "has_possession",
          demolished: false,
          no_teammates: false,
          teammate_role: "most_back",
          defensive_zone_fraction: 0.5,
          neutral_zone_fraction: 0.5,
          offensive_zone_fraction: 0,
          defensive_half_fraction: 0.8,
          offensive_half_fraction: 0.2,
          closest_to_ball: true,
          farthest_from_ball: false,
          behind_ball_fraction: 0.6,
          level_with_ball_fraction: 0.4,
          in_front_of_ball_fraction: 0,
          caught_ahead_of_play_on_conceded_goal: false,
        },
        {
          time: 1.1,
          frame: 11,
          duration: 0.2,
          player: playerId,
          is_team_0: true,
          active: true,
          tracked: false,
          possession_state: "neutral",
          demolished: true,
          no_teammates: false,
          teammate_role: "unknown",
          defensive_zone_fraction: 0,
          neutral_zone_fraction: 0,
          offensive_zone_fraction: 0,
          defensive_half_fraction: 0,
          offensive_half_fraction: 0,
          closest_to_ball: false,
          farthest_from_ball: false,
          behind_ball_fraction: 0,
          level_with_ball_fraction: 0,
          in_front_of_ball_fraction: 0,
          caught_ahead_of_play_on_conceded_goal: true,
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
