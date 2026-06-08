import test from "node:test";
import assert from "node:assert/strict";

import { applyPositioningEventDerivedStats } from "./positioningEventDerivation.ts";
import { createStatsFrame, createStatsTimeline } from "./testStatsTimeline.ts";

const playerId = { Steam: "positioning-player" } as Record<string, unknown>;
const opponentId = { Steam: "opponent-positioning-player" } as Record<string, unknown>;

function assertClose(actual: number | undefined, expected: number): void {
  assert.ok(actual != null && Math.abs(actual - expected) < 1e-6, `${actual} != ${expected}`);
}

test("positioning event derivation prorates spans across the playhead", () => {
  const timeline = createStatsTimeline({
    events: {
      // Tracked for the first half of the window, demolished for the second half, so the
      // running snapshot should accrue each span in time rather than crediting it whole at
      // its start frame.
      positioning_activity: [
        {
          time: 1,
          frame: 10,
          end_time: 1.5,
          end_frame: 15,
          duration: 0.5,
          player: playerId,
          is_team_0: true,
          active: true,
          tracked: true,
          demolished: false,
        },
        {
          time: 1.5,
          frame: 15,
          end_time: 2,
          end_frame: 20,
          duration: 0.5,
          player: playerId,
          is_team_0: true,
          active: true,
          tracked: false,
          demolished: true,
        },
      ],
      positioning_possession: [
        {
          time: 1,
          frame: 10,
          end_time: 2,
          end_frame: 20,
          duration: 1,
          player: playerId,
          is_team_0: true,
          possession_state: "has_possession",
        },
      ],
      positioning_field_zone: [
        {
          time: 1,
          frame: 10,
          end_time: 2,
          end_frame: 20,
          duration: 1,
          player: playerId,
          is_team_0: true,
          defensive_zone_fraction: 0.5,
          neutral_zone_fraction: 0.5,
          offensive_zone_fraction: 0,
          defensive_half_fraction: 0.8,
          offensive_half_fraction: 0.2,
        },
      ],
      positioning_ball_proximity: [
        {
          time: 1,
          frame: 10,
          end_time: 2,
          end_frame: 20,
          duration: 1,
          player: playerId,
          is_team_0: true,
          closest_to_ball_team: true,
          closest_to_ball_absolute: true,
          farthest_from_ball: false,
        },
      ],
      positioning_goal_context: [
        {
          time: 1.5,
          frame: 15,
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
        frame_number: 15,
        time: 1.5,
        players: [{ player_id: playerId, is_team_0: true, name: "Blue" }],
      }),
      createStatsFrame({
        frame_number: 20,
        time: 2,
        players: [{ player_id: playerId, is_team_0: true, name: "Blue" }],
      }),
    ],
  });

  // Distance is a continuous magnitude computed over the whole match and shipped once as a
  // per-player summary (not per frame, not events). Possession, by contrast, is categorical
  // and comes from the positioning_possession event above.
  for (const frame of timeline.frames) {
    Object.keys(frame.players[0]!.positioning).forEach((key) => {
      delete (frame.players[0]!.positioning as Record<string, unknown>)[key];
    });
  }
  (timeline as unknown as { positioning_summary: unknown }).positioning_summary = [
    {
      player_id: playerId,
      is_team_0: true,
      distance: {
        sum_distance_to_teammates: 1000,
        sum_distance_to_ball: 2000,
        sum_distance_to_ball_has_possession: 2000,
        sum_distance_to_ball_no_possession: 0,
      },
    },
  ];

  const derived = applyPositioningEventDerivedStats(timeline);
  const positioningAt = (frameIndex: number) => derived.frames[frameIndex]!.players[0]!.positioning;

  // Distance is the whole-match total, reported constant at every playhead.
  assert.equal(positioningAt(0).tracked_time, 0);
  assertClose(positioningAt(0).sum_distance_to_ball, 2000);
  assertClose(positioningAt(1).sum_distance_to_ball, 2000);
  assertClose(positioningAt(3).sum_distance_to_teammates, 1000);

  // Frame 10: categorical spans have just begun, so no time has elapsed inside them yet.
  assertClose(positioningAt(1).tracked_time, 0);
  assert.equal(positioningAt(1).times_caught_ahead_of_play_on_conceded_goals, 0);

  // Frame 15 (halfway): tracked span complete (0.5), demolished span just started (0), and the
  // categorical full-window spans (zone, possession, proximity) are half elapsed.
  assertClose(positioningAt(2).tracked_time, 0.5);
  assertClose(positioningAt(2).active_game_time, 0.5);
  assertClose(positioningAt(2).time_demolished, 0);
  assertClose(positioningAt(2).time_defensive_third, 0.25);
  assertClose(positioningAt(2).time_closest_to_ball, 0.5);
  assertClose(positioningAt(2).time_has_possession, 0.5);
  assert.equal(positioningAt(2).times_caught_ahead_of_play_on_conceded_goals, 1);

  // Frame 20 (end): every categorical span is fully credited.
  assertClose(positioningAt(3).tracked_time, 0.5);
  assertClose(positioningAt(3).time_demolished, 0.5);
  assertClose(positioningAt(3).active_game_time, 1);
  assertClose(positioningAt(3).time_has_possession, 1);
  assertClose(positioningAt(3).time_defensive_third, 0.5);
  assertClose(positioningAt(3).time_closest_to_ball, 1);
  assertClose(positioningAt(3).time_closest_to_ball_absolute, 1);
});

test("positioning event derivation prorates team closest-to-ball stats", () => {
  const timeline = createStatsTimeline({
    events: {
      positioning_ball_proximity: [
        {
          time: 1,
          frame: 10,
          end_time: 1.5,
          end_frame: 15,
          duration: 0.5,
          player: playerId,
          is_team_0: true,
          closest_to_ball_team: true,
          closest_to_ball_absolute: true,
          farthest_from_ball: false,
        },
        {
          time: 1.5,
          frame: 15,
          end_time: 2,
          end_frame: 20,
          duration: 0.5,
          player: opponentId,
          is_team_0: false,
          closest_to_ball_team: true,
          closest_to_ball_absolute: true,
          farthest_from_ball: false,
        },
      ],
    },
    frames: [
      createStatsFrame({ frame_number: 9, time: 0.9 }),
      createStatsFrame({ frame_number: 10, time: 1 }),
      createStatsFrame({ frame_number: 15, time: 1.5 }),
      createStatsFrame({ frame_number: 20, time: 2 }),
    ],
  });

  const derived = applyPositioningEventDerivedStats(timeline);

  // Before either span has elapsed any time.
  assertClose(derived.frames[0]!.team_zero.positioning.tracked_time, 0);
  assertClose(derived.frames[1]!.team_zero.positioning.tracked_time, 0);

  // Team zero's span completes by frame 15; team one's has only just started.
  assertClose(derived.frames[2]!.team_zero.positioning.tracked_time, 0.5);
  assertClose(derived.frames[2]!.team_zero.positioning.time_closest_to_ball, 0.5);
  assertClose(derived.frames[2]!.team_zero.positioning.time_closest_to_ball_team, 0.5);
  assertClose(derived.frames[2]!.team_zero.positioning.time_closest_to_ball_absolute, 0.5);
  assertClose(derived.frames[2]!.team_one.positioning.tracked_time, 0);

  // By frame 20 team one's span is complete and team zero's totals are unchanged.
  assertClose(derived.frames[3]!.team_one.positioning.tracked_time, 0.5);
  assertClose(derived.frames[3]!.team_one.positioning.time_closest_to_ball, 0.5);
  assertClose(derived.frames[3]!.team_one.positioning.time_closest_to_ball_absolute, 0.5);
  assertClose(derived.frames[3]!.team_zero.positioning.tracked_time, 0.5);
  assertClose(derived.frames[3]!.team_zero.positioning.time_closest_to_ball, 0.5);
});
