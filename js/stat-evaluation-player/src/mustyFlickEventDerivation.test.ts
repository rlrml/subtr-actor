import test from "node:test";
import assert from "node:assert/strict";

import { applyMustyFlickEventDerivedStats } from "./mustyFlickEventDerivation.ts";
import { createStatsFrame, createStatsTimeline } from "./testStatsTimeline.ts";

const bluePlayer = { Steam: "blue-musty" } as Record<string, unknown>;
const orangePlayer = { Steam: "orange-musty" } as Record<string, unknown>;

function assertClose(actual: number | undefined, expected: number): void {
  assert.ok(actual != null && Math.abs(actual - expected) < 1e-6, `${actual} != ${expected}`);
}

test("musty-flick event derivation can populate compacted player stats", () => {
  const timeline = createStatsTimeline({
    events: {
      musty_flick: [
        {
          time: 2,
          frame: 20,
          sample_time: 2,
          sample_frame: 20,
          player: bluePlayer,
          is_team_0: true,
          aerial: true,
          dodge_time: 1.9,
          dodge_frame: 19,
          time_since_dodge: 0.1,
          confidence: 0.85,
          local_ball_position: [10, 20, 30],
          rear_alignment: 0.7,
          top_alignment: 0.6,
          forward_approach_speed: 650,
          pitch_rate: 5,
          ball_speed_change: 500,
        },
        {
          time: 3,
          frame: 30,
          sample_time: 3,
          sample_frame: 30,
          player: orangePlayer,
          is_team_0: false,
          aerial: false,
          dodge_time: 2.85,
          dodge_frame: 28,
          time_since_dodge: 0.15,
          confidence: 0.7,
          local_ball_position: [15, -10, 25],
          rear_alignment: 0.6,
          top_alignment: 0.5,
          forward_approach_speed: 500,
          pitch_rate: 4,
          ball_speed_change: 350,
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
      for (const key of Object.keys(player.musty_flick)) {
        delete (player.musty_flick as Record<string, unknown>)[key];
      }
    }
  }

  applyMustyFlickEventDerivedStats(timeline);

  assert.equal(timeline.frames[0]?.players[0]?.musty_flick.count, 1);
  assert.equal(timeline.frames[0]?.players[0]?.musty_flick.aerial_count, 1);
  assert.equal(timeline.frames[0]?.players[0]?.musty_flick.high_confidence_count, 1);
  assert.equal(timeline.frames[0]?.players[0]?.musty_flick.is_last_musty, true);
  assertClose(timeline.frames[0]?.players[0]?.musty_flick.cumulative_confidence, 0.85);
  assert.equal(
    (timeline.frames[0]?.players[0]?.musty_flick as { labeled_event_counts?: unknown })
      .labeled_event_counts != null,
    true,
  );

  assert.equal(timeline.frames[1]?.players[0]?.musty_flick.is_last_musty, true);
  assert.equal(timeline.frames[1]?.players[0]?.musty_flick.frames_since_last_musty, 0);
  assert.equal(timeline.frames[1]?.players[0]?.musty_flick.time_since_last_musty, 0);

  assert.equal(timeline.frames[2]?.players[0]?.musty_flick.is_last_musty, false);
  assert.equal(timeline.frames[2]?.players[0]?.musty_flick.frames_since_last_musty, 10);
  assert.equal(timeline.frames[2]?.players[1]?.musty_flick.count, 1);
  assert.equal(timeline.frames[2]?.players[1]?.musty_flick.aerial_count, 0);
  assert.equal(timeline.frames[2]?.players[1]?.musty_flick.high_confidence_count, 0);
  assert.equal(timeline.frames[2]?.players[1]?.musty_flick.is_last_musty, true);
  assertClose(timeline.frames[2]?.players[1]?.musty_flick.cumulative_confidence, 0.7);
});
