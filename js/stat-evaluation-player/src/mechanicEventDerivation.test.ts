import test from "node:test";
import assert from "node:assert/strict";

import { applyMechanicEventDerivedStats } from "./mechanicEventDerivation.ts";
import { createStatsFrame, createStatsTimeline } from "./testStatsTimeline.ts";

const playerId = { Steam: "mechanic-player" } as Record<string, unknown>;

test("mechanic event derivation can populate compacted half-flip and wavedash stats", () => {
  const timeline = createStatsTimeline({
    events: {
      speed_flip: [
        {
          time: 2,
          frame: 20,
          resolved_time: 3.5,
          resolved_frame: 35,
          player: playerId,
          is_team_0: true,
          start_position: [0, 0, 0],
          end_position: [1, 0, 0],
          start_speed: 900,
          max_speed: 1800,
          best_alignment: 0.9,
          diagonal_score: 0.9,
          cancel_score: 0.9,
          speed_score: 0.9,
          confidence: 0.76,
        },
      ],
      half_flip: [
        {
          time: 2,
          frame: 20,
          player: playerId,
          is_team_0: true,
          start_position: [0, 0, 0],
          end_position: [1, 0, 0],
          start_speed: 300,
          end_speed: 600,
          start_backward_alignment: 0.8,
          best_reorientation_alignment: 0.9,
          best_forward_reversal: 0.85,
          max_forward_vertical: 0.4,
          confidence: 0.8,
        },
      ],
      wavedash: [
        {
          time: 3,
          frame: 30,
          player: playerId,
          is_team_0: true,
          dodge_time: 2.8,
          dodge_frame: 28,
          time_since_dodge: 0.2,
          dodge_position: [0, 0, 30],
          landing_position: [1, 0, 17],
          start_speed: 500,
          landing_speed: 900,
          horizontal_speed_gain: 400,
          landing_uprightness: 0.9,
          confidence: 0.7,
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
        frame_number: 25,
        time: 2.5,
        gameplay_phase: "post_goal",
        is_live_play: false,
        players: [{ player_id: playerId, is_team_0: true }],
      }),
      createStatsFrame({
        frame_number: 35,
        time: 3.5,
        gameplay_phase: "active_play",
        players: [{ player_id: playerId, is_team_0: true }],
      }),
    ],
  });

  for (const frame of timeline.frames) {
    for (const player of frame.players) {
      for (const key of Object.keys(player.half_flip)) {
        delete (player.half_flip as Record<string, unknown>)[key];
      }
      for (const key of Object.keys(player.speed_flip)) {
        delete (player.speed_flip as Record<string, unknown>)[key];
      }
      for (const key of Object.keys(player.wavedash)) {
        delete (player.wavedash as Record<string, unknown>)[key];
      }
    }
  }

  applyMechanicEventDerivedStats(timeline);

  assert.equal(timeline.frames[0]?.players[0]?.half_flip.count, 1);
  assert.equal(timeline.frames[0]?.players[0]?.half_flip.high_confidence_count, 1);
  assert.equal(timeline.frames[0]?.players[0]?.half_flip.is_last_half_flip, true);
  assert.equal(timeline.frames[1]?.players[0]?.half_flip.frames_since_last_half_flip, 0);
  assert.equal(timeline.frames[1]?.players[0]?.half_flip.time_since_last_half_flip, 0);
  assert.equal(timeline.frames[1]?.players[0]?.half_flip.is_last_half_flip, true);
  assert.equal(timeline.frames[2]?.players[0]?.half_flip.frames_since_last_half_flip, 15);
  assert.equal(timeline.frames[2]?.players[0]?.half_flip.time_since_last_half_flip, 1.5);
  assert.equal(timeline.frames[2]?.players[0]?.half_flip.is_last_half_flip, false);

  assert.equal(timeline.frames[0]?.players[0]?.speed_flip.count, 0);
  assert.equal(timeline.frames[1]?.players[0]?.speed_flip.count, 0);
  assert.equal(timeline.frames[2]?.players[0]?.speed_flip.count, 1);
  assert.equal(timeline.frames[2]?.players[0]?.speed_flip.high_confidence_count, 1);
  assert.equal(timeline.frames[2]?.players[0]?.speed_flip.last_speed_flip_frame, 20);
  assert.equal(timeline.frames[2]?.players[0]?.speed_flip.frames_since_last_speed_flip, 0);
  assert.equal(timeline.frames[2]?.players[0]?.speed_flip.is_last_speed_flip, true);

  assert.equal(timeline.frames[0]?.players[0]?.wavedash.count, 0);
  assert.equal(timeline.frames[1]?.players[0]?.wavedash.count, 0);
  assert.equal(timeline.frames[2]?.players[0]?.wavedash.count, 1);
  assert.equal(timeline.frames[2]?.players[0]?.wavedash.high_confidence_count, 0);
  assert.equal(timeline.frames[2]?.players[0]?.wavedash.last_quality, 0.7);
  assert.equal(timeline.frames[2]?.players[0]?.wavedash.is_last_wavedash, true);
});
