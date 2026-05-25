import test from "node:test";
import assert from "node:assert/strict";

import { applyHalfVolleyEventDerivedStats } from "./halfVolleyEventDerivation.ts";
import { createStatsFrame, createStatsTimeline } from "./testStatsTimeline.ts";

const bluePlayer = { Steam: "blue-half-volley" } as Record<string, unknown>;
const orangePlayer = { Steam: "orange-half-volley" } as Record<string, unknown>;

test("half-volley event derivation can populate compacted player and team stats", () => {
  const timeline = createStatsTimeline({
    events: {
      half_volley: [
        {
          time: 2,
          frame: 20,
          player: bluePlayer,
          is_team_0: true,
          bounce_time: 1.8,
          bounce_frame: 18,
          bounce_to_touch_seconds: 0.2,
          ball_speed: 1400,
          goal_alignment: 0.8,
        },
        {
          time: 3,
          frame: 30,
          player: orangePlayer,
          is_team_0: false,
          bounce_time: 2.9,
          bounce_frame: 29,
          bounce_to_touch_seconds: 0.1,
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
    for (const key of Object.keys(frame.team_zero.half_volley)) {
      delete (frame.team_zero.half_volley as Record<string, unknown>)[key];
    }
    for (const key of Object.keys(frame.team_one.half_volley)) {
      delete (frame.team_one.half_volley as Record<string, unknown>)[key];
    }
    for (const player of frame.players) {
      for (const key of Object.keys(player.half_volley)) {
        delete (player.half_volley as Record<string, unknown>)[key];
      }
    }
  }

  applyHalfVolleyEventDerivedStats(timeline);

  assert.equal(timeline.frames[0]?.team_zero.half_volley.count, 1);
  assert.equal(timeline.frames[0]?.team_zero.half_volley.total_ball_speed, 1400);
  assert.equal(timeline.frames[0]?.players[0]?.half_volley.count, 1);
  assert.equal(timeline.frames[0]?.players[0]?.half_volley.is_last_half_volley, true);

  assert.equal(timeline.frames[1]?.players[0]?.half_volley.is_last_half_volley, false);
  assert.equal(timeline.frames[1]?.players[0]?.half_volley.frames_since_last_half_volley, 5);

  assert.equal(timeline.frames[2]?.team_one.half_volley.count, 1);
  assert.equal(timeline.frames[2]?.team_one.half_volley.fastest_ball_speed, 1600);
  assert.equal(timeline.frames[2]?.players[0]?.half_volley.is_last_half_volley, false);
  assert.equal(timeline.frames[2]?.players[1]?.half_volley.is_last_half_volley, true);
  assert.equal(timeline.frames[2]?.players[1]?.half_volley.frames_since_last_half_volley, 0);
});
