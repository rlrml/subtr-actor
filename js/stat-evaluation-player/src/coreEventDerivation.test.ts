import test from "node:test";
import assert from "node:assert/strict";

import { applyCoreEventDerivedStats } from "./coreEventDerivation.ts";
import { createStatsFrame, createStatsTimeline } from "./testStatsTimeline.ts";

const bluePlayer = { Steam: "blue-core" } as Record<string, unknown>;

test("core event derivation populates compacted player and team stats", () => {
  const timeline = createStatsTimeline({
    events: {
      core_player: [
        {
          time: 1,
          frame: 10,
          player: bluePlayer,
          is_team_0: true,
          score_delta: 110,
          goals_delta: 1,
          assists_delta: 0,
          saves_delta: 0,
          shots_delta: 1,
        },
      ],
      goal_context: [
        {
          time: 1,
          frame: 10,
          scoring_team_is_team_0: true,
          scorer: bluePlayer,
          scoring_team_most_back_player: bluePlayer,
          defending_team_most_back_player: null,
          ball_position: null,
          ball_air_time_before_goal: 0.5,
          time_after_kickoff: 1,
          goal_buildup: "counter_attack",
          scorer_last_touch: {
            time: 0.8,
            frame: 8,
            player: bluePlayer,
            is_team_0: true,
            ball_position: { x: 10, y: 20, z: 30 },
            player_position: null,
            players: [],
          },
          players: [
            {
              player: bluePlayer,
              is_team_0: true,
              position: null,
              boost_amount: null,
              average_boost_in_leadup: null,
              min_boost_in_leadup: null,
              is_most_back: true,
            },
          ],
          tags: [],
        },
      ],
    },
    frames: [
      createStatsFrame({
        frame_number: 9,
        time: 0.9,
        players: [{ player_id: bluePlayer, is_team_0: true, name: "Blue" }],
      }),
      createStatsFrame({
        frame_number: 10,
        time: 1,
        players: [{ player_id: bluePlayer, is_team_0: true, name: "Blue" }],
      }),
      createStatsFrame({
        frame_number: 11,
        time: 1.1,
        players: [{ player_id: bluePlayer, is_team_0: true, name: "Blue" }],
      }),
    ],
  });

  for (const frame of timeline.frames) {
    Object.keys(frame.team_zero.core).forEach((key) => {
      delete (frame.team_zero.core as Record<string, unknown>)[key];
    });
    Object.keys(frame.players[0]!.core).forEach((key) => {
      delete (frame.players[0]!.core as Record<string, unknown>)[key];
    });
  }

  const derived = applyCoreEventDerivedStats(timeline);

  assert.equal(derived.frames[0]!.team_zero.core.goals, 0);
  assert.equal(derived.frames[0]!.players[0]!.core.goals, 0);
  assert.equal(derived.frames[1]!.team_zero.core.goals, 1);
  assert.equal(derived.frames[1]!.team_zero.core.counter_attack_goal_count, 1);
  assert.equal(derived.frames[1]!.players[0]!.core.goals, 1);
  assert.equal(derived.frames[1]!.players[0]!.core.goals_for_while_most_back, 1);
  assert.deepEqual(derived.frames[1]!.players[0]!.core.last_scoring_goal_last_touch_position, {
    x: 10,
    y: 20,
    z: 30,
  });
  assert.equal(derived.frames[2]!.players[0]!.core.goal_ball_air_time_sample_count, 1);
});
