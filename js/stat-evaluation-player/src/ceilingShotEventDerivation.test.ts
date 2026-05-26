import test from "node:test";
import assert from "node:assert/strict";

import { applyCeilingShotEventDerivedStats } from "./ceilingShotEventDerivation.ts";
import { createStatsFrame, createStatsTimeline } from "./testStatsTimeline.ts";

const bluePlayer = { Steam: "blue-ceiling-shot" } as Record<string, unknown>;
const orangePlayer = { Steam: "orange-ceiling-shot" } as Record<string, unknown>;

function assertClose(actual: number | undefined, expected: number): void {
  assert.ok(actual != null && Math.abs(actual - expected) < 1e-6, `${actual} != ${expected}`);
}

test("ceiling-shot event derivation can populate compacted player stats", () => {
  const timeline = createStatsTimeline({
    events: {
      ceiling_shot: [
        {
          time: 2,
          frame: 20,
          player: bluePlayer,
          is_team_0: true,
          ceiling_contact_time: 1.2,
          ceiling_contact_frame: 12,
          time_since_ceiling_contact: 0.8,
          ceiling_contact_position: [0, 0, 2040],
          touch_position: [500, 100, 520],
          local_ball_position: [120, 10, 30],
          separation_from_ceiling: 460,
          roof_alignment: 0.9,
          forward_alignment: 0.8,
          forward_approach_speed: 700,
          ball_speed_change: 500,
          confidence: 0.82,
        },
        {
          time: 3,
          frame: 30,
          player: orangePlayer,
          is_team_0: false,
          ceiling_contact_time: 2.4,
          ceiling_contact_frame: 24,
          time_since_ceiling_contact: 0.6,
          ceiling_contact_position: [0, 0, 2040],
          touch_position: [-400, -200, 480],
          local_ball_position: [100, -20, 20],
          separation_from_ceiling: 520,
          roof_alignment: 0.85,
          forward_alignment: 0.7,
          forward_approach_speed: 650,
          ball_speed_change: 350,
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
      for (const key of Object.keys(player.ceiling_shot)) {
        delete (player.ceiling_shot as Record<string, unknown>)[key];
      }
    }
  }

  applyCeilingShotEventDerivedStats(timeline);

  assert.equal(timeline.frames[0]?.players[0]?.ceiling_shot.count, 1);
  assert.equal(timeline.frames[0]?.players[0]?.ceiling_shot.high_confidence_count, 1);
  assert.equal(timeline.frames[0]?.players[0]?.ceiling_shot.is_last_ceiling_shot, true);
  assertClose(timeline.frames[0]?.players[0]?.ceiling_shot.cumulative_confidence, 0.82);
  assert.equal(
    (timeline.frames[0]?.players[0]?.ceiling_shot as { labeled_event_counts?: unknown })
      .labeled_event_counts != null,
    true,
  );

  assert.equal(timeline.frames[1]?.players[0]?.ceiling_shot.is_last_ceiling_shot, true);
  assert.equal(timeline.frames[1]?.players[0]?.ceiling_shot.frames_since_last_ceiling_shot, 0);
  assert.equal(timeline.frames[1]?.players[0]?.ceiling_shot.time_since_last_ceiling_shot, 0);

  assert.equal(timeline.frames[2]?.players[0]?.ceiling_shot.is_last_ceiling_shot, false);
  assert.equal(timeline.frames[2]?.players[0]?.ceiling_shot.frames_since_last_ceiling_shot, 10);
  assert.equal(timeline.frames[2]?.players[1]?.ceiling_shot.count, 1);
  assert.equal(timeline.frames[2]?.players[1]?.ceiling_shot.high_confidence_count, 0);
  assert.equal(timeline.frames[2]?.players[1]?.ceiling_shot.is_last_ceiling_shot, true);
  assertClose(timeline.frames[2]?.players[1]?.ceiling_shot.cumulative_confidence, 0.7);
});
