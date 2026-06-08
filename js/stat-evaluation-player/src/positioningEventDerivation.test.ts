import test from "node:test";
import assert from "node:assert/strict";

import { applyPositioningEventDerivedStats } from "./positioningEventDerivation.ts";
import { createStatsFrame, createStatsTimeline } from "./testStatsTimeline.ts";

const playerId = { Steam: "positioning-player" } as Record<string, unknown>;
const opponentId = { Steam: "opponent-positioning-player" } as Record<string, unknown>;

function assertClose(actual: number | undefined, expected: number): void {
  assert.ok(actual != null && Math.abs(actual - expected) < 1e-6, `${actual} != ${expected}`);
}

test("positioning event derivation populates compacted player stats", () => {
  const timeline = createStatsTimeline({
    events: {
      positioning_activity: [
        {
          time: 1,
          frame: 10,
          end_time: 1.1,
          end_frame: 11,
          duration: 0.1,
          player: playerId,
          is_team_0: true,
          active: true,
          tracked: true,
          demolished: false,
        },
        {
          time: 1.1,
          frame: 11,
          end_time: 1.3,
          end_frame: 13,
          duration: 0.2,
          player: playerId,
          is_team_0: true,
          active: true,
          tracked: false,
          demolished: true,
        },
      ],
      positioning_distance: [
        {
          time: 1,
          frame: 10,
          end_time: 1.1,
          end_frame: 11,
          duration: 0.1,
          player: playerId,
          is_team_0: true,
          distance_to_teammates: 1000,
          distance_to_ball: 2000,
          possession_state: "has_possession",
        },
      ],
      positioning_field_zone: [
        {
          time: 1,
          frame: 10,
          end_time: 1.1,
          end_frame: 11,
          duration: 0.1,
          player: playerId,
          is_team_0: true,
          defensive_zone_fraction: 0.5,
          neutral_zone_fraction: 0.5,
          offensive_zone_fraction: 0,
          defensive_half_fraction: 0.8,
          offensive_half_fraction: 0.2,
        },
      ],
      positioning_ball_depth: [
        {
          time: 1,
          frame: 10,
          end_time: 1.1,
          end_frame: 11,
          duration: 0.1,
          player: playerId,
          is_team_0: true,
          behind_ball_fraction: 0.6,
          level_with_ball_fraction: 0.4,
          in_front_of_ball_fraction: 0,
        },
      ],
      positioning_teammate_role: [
        {
          time: 1,
          frame: 10,
          end_time: 1.1,
          end_frame: 11,
          duration: 0.1,
          player: playerId,
          is_team_0: true,
          teammate_role: "most_back",
        },
      ],
      positioning_ball_proximity: [
        {
          time: 1,
          frame: 10,
          end_time: 1.1,
          end_frame: 11,
          duration: 0.1,
          player: playerId,
          is_team_0: true,
          closest_to_ball_team: true,
          closest_to_ball_absolute: true,
          farthest_from_ball: false,
        },
      ],
      positioning_goal_context: [
        {
          time: 1.1,
          frame: 11,
          player: playerId,
          is_team_0: true,
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
  assertClose(derived.frames[1]!.players[0]!.positioning.time_closest_to_ball_team, 0.1);
  assertClose(derived.frames[1]!.players[0]!.positioning.time_closest_to_ball_absolute, 0.1);
  assertClose(derived.frames[2]!.players[0]!.positioning.active_game_time, 0.3);
  assertClose(derived.frames[2]!.players[0]!.positioning.time_demolished, 0.2);
  assert.equal(
    derived.frames[2]!.players[0]!.positioning.times_caught_ahead_of_play_on_conceded_goals,
    1,
  );
});

test("positioning event derivation populates team closest-to-ball stats", () => {
  const timeline = createStatsTimeline({
    events: {
      positioning_ball_proximity: [
        {
          time: 1,
          frame: 10,
          end_time: 1.1,
          end_frame: 11,
          duration: 0.1,
          player: playerId,
          is_team_0: true,
          closest_to_ball_team: true,
          closest_to_ball_absolute: true,
          farthest_from_ball: false,
        },
        {
          time: 1.1,
          frame: 11,
          end_time: 1.3,
          end_frame: 13,
          duration: 0.2,
          player: opponentId,
          is_team_0: false,
          closest_to_ball_team: true,
          closest_to_ball_absolute: true,
          farthest_from_ball: false,
        },
        {
          time: 1.2,
          frame: 12,
          end_time: 1.5,
          end_frame: 15,
          duration: 0.3,
          player: playerId,
          is_team_0: true,
          closest_to_ball_team: true,
          closest_to_ball_absolute: false,
          farthest_from_ball: true,
        },
      ],
    },
    frames: [
      createStatsFrame({ frame_number: 9, time: 0.9 }),
      createStatsFrame({ frame_number: 10, time: 1 }),
      createStatsFrame({ frame_number: 11, time: 1.1 }),
      createStatsFrame({ frame_number: 12, time: 1.2 }),
    ],
  });

  const derived = applyPositioningEventDerivedStats(timeline);

  assertClose(derived.frames[0]!.team_zero.positioning.tracked_time, 0);
  assertClose(derived.frames[1]!.team_zero.positioning.tracked_time, 0.1);
  assertClose(derived.frames[1]!.team_zero.positioning.time_closest_to_ball, 0.1);
  assertClose(derived.frames[1]!.team_zero.positioning.time_closest_to_ball_team, 0.1);
  assertClose(derived.frames[1]!.team_zero.positioning.time_closest_to_ball_absolute, 0.1);
  assertClose(derived.frames[2]!.team_one.positioning.tracked_time, 0.2);
  assertClose(derived.frames[2]!.team_one.positioning.time_closest_to_ball, 0.2);
  assertClose(derived.frames[2]!.team_one.positioning.time_closest_to_ball_absolute, 0.2);
  assertClose(derived.frames[3]!.team_zero.positioning.tracked_time, 0.4);
  assertClose(derived.frames[3]!.team_zero.positioning.time_closest_to_ball, 0.4);
  assertClose(derived.frames[3]!.team_zero.positioning.time_closest_to_ball_team, 0.4);
  assertClose(derived.frames[3]!.team_zero.positioning.time_closest_to_ball_absolute, 0.1);
});
