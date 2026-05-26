import test from "node:test";
import assert from "node:assert/strict";

import { applyWhiffEventDerivedStats } from "./whiffEventDerivation.ts";
import { createStatsFrame, createStatsTimeline } from "./testStatsTimeline.ts";

const playerId = { Steam: "whiff-player" } as Record<string, unknown>;

test("whiff event derivation can populate compacted whiff stats", () => {
  const timeline = createStatsTimeline({
    events: {
      whiff: [
        {
          kind: "whiff",
          time: 2,
          frame: 20,
          resolved_time: 3.5,
          resolved_frame: 35,
          player: playerId,
          is_team_0: true,
          closest_approach_distance: 128,
          forward_alignment: 0.8,
          approach_speed: 1300,
          dodge_active: true,
          aerial: true,
        },
        {
          kind: "beaten_to_ball",
          time: 4,
          frame: 40,
          resolved_time: 4,
          resolved_frame: 40,
          player: playerId,
          is_team_0: true,
          closest_approach_distance: 160,
          forward_alignment: 0.7,
          approach_speed: 1000,
          dodge_active: false,
          aerial: false,
        },
      ],
    },
    frames: [
      createStatsFrame({
        frame_number: 20,
        time: 2,
        gameplay_phase: "active_play",
        players: [{ player_id: playerId, is_team_0: true }],
      }),
      createStatsFrame({
        frame_number: 35,
        time: 3.5,
        gameplay_phase: "active_play",
        players: [{ player_id: playerId, is_team_0: true }],
      }),
      createStatsFrame({
        frame_number: 36,
        time: 3.6,
        gameplay_phase: "post_goal",
        is_live_play: false,
        players: [{ player_id: playerId, is_team_0: true }],
      }),
      createStatsFrame({
        frame_number: 40,
        time: 4,
        gameplay_phase: "active_play",
        players: [{ player_id: playerId, is_team_0: true }],
      }),
    ],
  });

  for (const frame of timeline.frames) {
    for (const player of frame.players) {
      for (const key of Object.keys(player.whiff)) {
        delete (player.whiff as Record<string, unknown>)[key];
      }
    }
  }

  applyWhiffEventDerivedStats(timeline);

  assert.equal(timeline.frames[0]?.players[0]?.whiff.whiff_count, 0);
  assert.equal(timeline.frames[1]?.players[0]?.whiff.whiff_count, 1);
  assert.equal(timeline.frames[1]?.players[0]?.whiff.aerial_whiff_count, 1);
  assert.equal(timeline.frames[1]?.players[0]?.whiff.dodge_whiff_count, 1);
  assert.equal(timeline.frames[1]?.players[0]?.whiff.last_whiff_frame, 20);
  assert.equal(timeline.frames[1]?.players[0]?.whiff.frames_since_last_whiff, 15);
  assert.equal(timeline.frames[1]?.players[0]?.whiff.is_last_whiff, true);
  assert.equal(timeline.frames[2]?.players[0]?.whiff.frames_since_last_whiff, 15);
  assert.equal(timeline.frames[3]?.players[0]?.whiff.frames_since_last_whiff, 20);
  assert.equal(timeline.frames[3]?.players[0]?.whiff.beaten_to_ball_count, 1);
  assert.equal(timeline.frames[3]?.players[0]?.whiff.whiff_count, 1);
  assert.equal(timeline.frames[3]?.players[0]?.whiff.best_closest_approach_distance, 128);
});
