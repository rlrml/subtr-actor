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
          delta: {
            score: 110,
            goals: 1,
            assists: 0,
            saves: 0,
            shots: 1,
            goals_conceded_while_last_defender: 0,
            goals_for_while_most_back: 1,
            goals_against_while_most_back: 0,
            goal_against_boost_sample_count: 0,
            cumulative_boost_on_goals_against: 0,
            last_boost_on_goal_against: null,
            goal_against_boost_leadup_sample_count: 0,
            cumulative_average_boost_in_goal_against_leadup: 0,
            cumulative_min_boost_in_goal_against_leadup: 0,
            last_average_boost_in_goal_against_leadup: null,
            last_min_boost_in_goal_against_leadup: null,
            goal_against_position_sample_count: 0,
            cumulative_goal_against_position_x: 0,
            cumulative_goal_against_position_y: 0,
            cumulative_goal_against_position_z: 0,
            last_goal_against_position: null,
            scoring_goal_last_touch_position_sample_count: 1,
            cumulative_scoring_goal_last_touch_position_x: 10,
            cumulative_scoring_goal_last_touch_position_y: 20,
            cumulative_scoring_goal_last_touch_position_z: 30,
            last_scoring_goal_last_touch_position: { x: 10, y: 20, z: 30 },
            kickoff_goal_count: 1,
            short_goal_count: 0,
            medium_goal_count: 0,
            long_goal_count: 0,
            counter_attack_goal_count: 1,
            sustained_pressure_goal_count: 0,
            other_buildup_goal_count: 0,
            goal_ball_air_time_sample_count: 1,
            cumulative_goal_ball_air_time: 0.5,
            last_goal_ball_air_time: 0.5,
          },
        },
      ],
      core_team: [
        {
          time: 1,
          frame: 10,
          is_team_0: true,
          delta: {
            score: 110,
            goals: 1,
            assists: 0,
            saves: 0,
            shots: 1,
            kickoff_goal_count: 1,
            short_goal_count: 0,
            medium_goal_count: 0,
            long_goal_count: 0,
            counter_attack_goal_count: 1,
            sustained_pressure_goal_count: 0,
            other_buildup_goal_count: 0,
            goal_ball_air_time_sample_count: 1,
            cumulative_goal_ball_air_time: 0.5,
            last_goal_ball_air_time: 0.5,
          },
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
