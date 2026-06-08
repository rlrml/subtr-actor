import test from "node:test";
import assert from "node:assert/strict";

import { applyControlledPlayEventDerivedStats } from "./controlledPlayEventDerivation.ts";
import { createStatsFrame, createStatsTimeline } from "./testStatsTimeline.ts";

const bluePlayer = { Steam: "blue-controlled-play" } as Record<string, unknown>;
const orangePlayer = { Steam: "orange-controlled-play" } as Record<string, unknown>;

test("controlled-play event derivation can populate compacted player and team stats", () => {
  const timeline = createStatsTimeline({
    events: {
      controlled_play: [
        {
          player_id: bluePlayer,
          is_team_0: true,
          start_frame: 5,
          end_frame: 10,
          start_time: 0.5,
          end_time: 1,
          duration: 0.5,
          first_touch_frame: 5,
          last_touch_frame: 10,
          first_touch_time: 0.5,
          last_touch_time: 1,
          touch_count: 2,
          close_duration: 0.45,
          total_advance_distance: 600,
        },
        {
          player_id: orangePlayer,
          is_team_0: false,
          start_frame: 15,
          end_frame: 20,
          start_time: 1.5,
          end_time: 2,
          duration: 0.5,
          first_touch_frame: 15,
          last_touch_frame: 20,
          first_touch_time: 1.5,
          last_touch_time: 2,
          touch_count: 3,
          close_duration: 0.5,
          total_advance_distance: 700,
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
      frame.team_zero.controlled_play,
      frame.team_one.controlled_play,
      ...frame.players.map((player) => player.controlled_play),
    ]) {
      for (const key of Object.keys(stats)) {
        delete (stats as Record<string, unknown>)[key];
      }
    }
  }

  applyControlledPlayEventDerivedStats(timeline);

  assert.equal(timeline.frames[0]?.team_zero.controlled_play.count, 0);
  assert.equal(timeline.frames[0]?.players[0]?.controlled_play.count, 0);

  assert.equal(timeline.frames[1]?.team_zero.controlled_play.count, 1);
  assert.equal(timeline.frames[1]?.team_zero.controlled_play.total_time, Math.fround(0.5));
  assert.equal(timeline.frames[1]?.team_zero.controlled_play.touch_count, 2);
  assert.equal(timeline.frames[1]?.players[0]?.controlled_play.total_advance_distance, 600);

  assert.equal(timeline.frames[2]?.team_one.controlled_play.count, 1);
  assert.equal(timeline.frames[2]?.team_one.controlled_play.touch_count, 3);
  assert.equal(timeline.frames[2]?.players[1]?.controlled_play.longest_time, 0.5);
  assert.equal(timeline.frames[2]?.players[1]?.controlled_play.total_advance_distance, 700);
});
