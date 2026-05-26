import test from "node:test";
import assert from "node:assert/strict";

import { applyFlickEventDerivedStats } from "./flickEventDerivation.ts";
import { createStatsFrame, createStatsTimeline } from "./testStatsTimeline.ts";

const bluePlayer = { Steam: "blue-flick" } as Record<string, unknown>;
const orangePlayer = { Steam: "orange-flick" } as Record<string, unknown>;

function assertClose(actual: number | undefined, expected: number): void {
  assert.ok(actual != null && Math.abs(actual - expected) < 1e-6, `${actual} != ${expected}`);
}

test("flick event derivation can populate compacted player stats", () => {
  const timeline = createStatsTimeline({
    events: {
      flick: [
        {
          time: 2,
          frame: 20,
          sample_time: 2,
          sample_frame: 20,
          player: bluePlayer,
          is_team_0: true,
          dodge_time: 1.8,
          dodge_frame: 18,
          time_since_dodge: 0.2,
          setup_start_time: 1.3,
          setup_start_frame: 13,
          setup_duration: 0.7,
          setup_touch_count: 2,
          average_horizontal_gap: 80,
          average_vertical_gap: 40,
          ball_speed_change: 500,
          ball_impulse: [300, 200, 100],
          impulse_away_alignment: 0.9,
          vertical_impulse: 100,
          confidence: 0.85,
        },
        {
          time: 3,
          frame: 30,
          sample_time: 3,
          sample_frame: 30,
          player: orangePlayer,
          is_team_0: false,
          dodge_time: 2.8,
          dodge_frame: 28,
          time_since_dodge: 0.2,
          setup_start_time: 2.5,
          setup_start_frame: 25,
          setup_duration: 0.5,
          setup_touch_count: 1,
          average_horizontal_gap: 100,
          average_vertical_gap: 45,
          ball_speed_change: 350,
          ball_impulse: [220, -120, 80],
          impulse_away_alignment: 0.7,
          vertical_impulse: 80,
          confidence: 0.7,
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
    for (const player of frame.players) {
      for (const key of Object.keys(player.flick)) {
        delete (player.flick as Record<string, unknown>)[key];
      }
    }
  }

  applyFlickEventDerivedStats(timeline);

  assert.equal(timeline.frames[0]?.players[0]?.flick.count, 1);
  assert.equal(timeline.frames[0]?.players[0]?.flick.high_confidence_count, 1);
  assert.equal(timeline.frames[0]?.players[0]?.flick.is_last_flick, true);
  assertClose(timeline.frames[0]?.players[0]?.flick.cumulative_confidence, 0.85);
  assertClose(timeline.frames[0]?.players[0]?.flick.cumulative_setup_duration, 0.7);
  assert.equal(timeline.frames[0]?.players[0]?.flick.cumulative_ball_speed_change, 500);
  assert.equal(
    (timeline.frames[0]?.players[0]?.flick as { labeled_event_counts?: unknown })
      .labeled_event_counts != null,
    true,
  );

  assert.equal(timeline.frames[1]?.players[0]?.flick.is_last_flick, true);
  assert.equal(timeline.frames[1]?.players[0]?.flick.frames_since_last_flick, 0);
  assert.equal(timeline.frames[1]?.players[0]?.flick.time_since_last_flick, 0);

  assert.equal(timeline.frames[2]?.players[0]?.flick.is_last_flick, false);
  assert.equal(timeline.frames[2]?.players[0]?.flick.frames_since_last_flick, 10);
  assert.equal(timeline.frames[2]?.players[1]?.flick.count, 1);
  assert.equal(timeline.frames[2]?.players[1]?.flick.high_confidence_count, 0);
  assert.equal(timeline.frames[2]?.players[1]?.flick.is_last_flick, true);
  assertClose(timeline.frames[2]?.players[1]?.flick.cumulative_setup_duration, 0.5);
  assert.equal(timeline.frames[2]?.players[1]?.flick.cumulative_ball_speed_change, 350);
});
