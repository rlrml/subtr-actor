import test from "node:test";
import assert from "node:assert/strict";

import { applyOneTimerEventDerivedStats } from "./oneTimerEventDerivation.ts";
import { createStatsFrame, createStatsTimeline } from "./testStatsTimeline.ts";

const bluePlayer = { Steam: "blue-one-timer" } as Record<string, unknown>;
const orangePlayer = { Steam: "orange-one-timer" } as Record<string, unknown>;
const passer = { Steam: "passer" } as Record<string, unknown>;

test("one-timer event derivation can populate compacted player and team stats", () => {
  const timeline = createStatsTimeline({
    events: {
      one_timer: [
        {
          time: 2,
          frame: 20,
          player: bluePlayer,
          passer,
          is_team_0: true,
          pass_start_time: 1.5,
          pass_start_frame: 15,
          pass_duration: 0.5,
          pass_travel_distance: 900,
          pass_advance_distance: 700,
          ball_speed: 1400,
          goal_alignment: 0.8,
        },
        {
          time: 3,
          frame: 30,
          player: orangePlayer,
          passer,
          is_team_0: false,
          pass_start_time: 2.5,
          pass_start_frame: 25,
          pass_duration: 0.5,
          pass_travel_distance: 1100,
          pass_advance_distance: 800,
          ball_speed: 1600,
          goal_alignment: 0.9,
        },
      ],
    },
    frames: [
      createStatsFrame({
        frame_number: 20,
        time: 2,
        is_live_play: true,
        players: [
          { player_id: bluePlayer, is_team_0: true },
          { player_id: orangePlayer, is_team_0: false },
        ],
      }),
      createStatsFrame({
        frame_number: 25,
        time: 2.5,
        is_live_play: false,
        players: [
          { player_id: bluePlayer, is_team_0: true },
          { player_id: orangePlayer, is_team_0: false },
        ],
      }),
      createStatsFrame({
        frame_number: 30,
        time: 3,
        is_live_play: true,
        players: [
          { player_id: bluePlayer, is_team_0: true },
          { player_id: orangePlayer, is_team_0: false },
        ],
      }),
    ],
  });

  for (const frame of timeline.frames) {
    for (const key of Object.keys(frame.team_zero.one_timer)) {
      delete (frame.team_zero.one_timer as Record<string, unknown>)[key];
    }
    for (const key of Object.keys(frame.team_one.one_timer)) {
      delete (frame.team_one.one_timer as Record<string, unknown>)[key];
    }
    for (const player of frame.players) {
      for (const key of Object.keys(player.one_timer)) {
        delete (player.one_timer as Record<string, unknown>)[key];
      }
    }
  }

  applyOneTimerEventDerivedStats(timeline);

  assert.equal(timeline.frames[0]?.team_zero.one_timer.count, 1);
  assert.equal(timeline.frames[0]?.team_zero.one_timer.total_ball_speed, 1400);
  assert.equal(timeline.frames[0]?.players[0]?.one_timer.count, 1);
  assert.equal(timeline.frames[0]?.players[0]?.one_timer.total_pass_distance, 900);
  assert.equal(timeline.frames[0]?.players[0]?.one_timer.is_last_one_timer, true);

  assert.equal(timeline.frames[1]?.players[0]?.one_timer.is_last_one_timer, false);
  assert.equal(timeline.frames[1]?.players[0]?.one_timer.frames_since_last_one_timer, 5);

  assert.equal(timeline.frames[2]?.team_one.one_timer.count, 1);
  assert.equal(timeline.frames[2]?.team_one.one_timer.fastest_ball_speed, 1600);
  assert.equal(timeline.frames[2]?.players[0]?.one_timer.is_last_one_timer, false);
  assert.equal(timeline.frames[2]?.players[1]?.one_timer.is_last_one_timer, true);
  assert.equal(timeline.frames[2]?.players[1]?.one_timer.frames_since_last_one_timer, 0);
});
