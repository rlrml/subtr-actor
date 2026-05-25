import test from "node:test";
import assert from "node:assert/strict";

import { applyBallCarryEventDerivedStats } from "./ballCarryEventDerivation.ts";
import { createStatsFrame, createStatsTimeline } from "./testStatsTimeline.ts";

const bluePlayer = { Steam: "blue-ball-carry" } as Record<string, unknown>;
const orangePlayer = { Steam: "orange-air-dribble" } as Record<string, unknown>;

test("ball-carry event derivation can populate compacted carry and air-dribble stats", () => {
  const timeline = createStatsTimeline({
    events: {
      ball_carry: [
        {
          player_id: bluePlayer,
          is_team_0: true,
          kind: "carry",
          start_frame: 5,
          end_frame: 10,
          start_time: 0.5,
          end_time: 1,
          duration: 0.5,
          straight_line_distance: 600,
          path_distance: 650,
          average_horizontal_gap: 80,
          average_vertical_gap: 40,
          average_speed: 1300,
          touch_count: 2,
          air_touch_count: 0,
          air_dribble_origin: null,
        },
        {
          player_id: orangePlayer,
          is_team_0: false,
          kind: "air_dribble",
          start_frame: 8,
          end_frame: 20,
          start_time: 0.8,
          end_time: 2,
          duration: 1.2,
          straight_line_distance: 1400,
          path_distance: 1500,
          average_horizontal_gap: 120,
          average_vertical_gap: 90,
          average_speed: 1500,
          touch_count: 4,
          air_touch_count: 3,
          air_dribble_origin: "wall_to_air",
        },
      ],
    },
    frames: [
      createStatsFrame({
        frame_number: 10,
        time: 1,
        players: [
          { player_id: bluePlayer, is_team_0: true },
          { player_id: orangePlayer, is_team_0: false },
        ],
      }),
      createStatsFrame({
        frame_number: 11,
        time: 1.1,
        players: [
          { player_id: bluePlayer, is_team_0: true },
          { player_id: orangePlayer, is_team_0: false },
        ],
      }),
      createStatsFrame({
        frame_number: 21,
        time: 2.1,
        players: [
          { player_id: bluePlayer, is_team_0: true },
          { player_id: orangePlayer, is_team_0: false },
        ],
      }),
    ],
  });

  for (const frame of timeline.frames) {
    for (const stats of [
      frame.team_zero.ball_carry,
      frame.team_one.ball_carry,
      frame.team_zero.air_dribble,
      frame.team_one.air_dribble,
      ...frame.players.flatMap((player) => [player.ball_carry, player.air_dribble]),
    ]) {
      for (const key of Object.keys(stats)) {
        delete (stats as Record<string, unknown>)[key];
      }
    }
  }

  applyBallCarryEventDerivedStats(timeline);

  assert.equal(timeline.frames[0]?.team_zero.ball_carry.carry_count, 0);
  assert.equal(timeline.frames[0]?.players[0]?.ball_carry.carry_count, 0);

  assert.equal(timeline.frames[1]?.team_zero.ball_carry.carry_count, 1);
  assert.equal(timeline.frames[1]?.team_zero.ball_carry.total_carry_time, 0.5);
  assert.equal(timeline.frames[1]?.team_zero.ball_carry.furthest_carry_distance, 600);
  assert.equal(timeline.frames[1]?.players[0]?.ball_carry.fastest_carry_speed, 1300);
  assert.equal(
    (timeline.frames[1]?.players[0]?.ball_carry as { labeled_event_counts?: unknown })
      .labeled_event_counts != null,
    true,
  );

  assert.equal(timeline.frames[2]?.team_one.air_dribble.count, 1);
  assert.equal(timeline.frames[2]?.team_one.air_dribble.wall_to_air_count, 1);
  assert.equal(timeline.frames[2]?.team_one.air_dribble.total_touch_count, 4);
  assert.equal(timeline.frames[2]?.team_one.air_dribble.fastest_speed, 1500);
  assert.equal(timeline.frames[2]?.players[1]?.air_dribble.max_touch_count, 4);
  assert.equal(
    (timeline.frames[2]?.players[1]?.air_dribble as { labeled_event_counts?: unknown })
      .labeled_event_counts != null,
    true,
  );
});
