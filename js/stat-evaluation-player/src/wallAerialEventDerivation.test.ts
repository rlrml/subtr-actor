import test from "node:test";
import assert from "node:assert/strict";

import { applyWallAerialEventDerivedStats } from "./wallAerialEventDerivation.ts";
import { createStatsFrame, createStatsTimeline } from "./testStatsTimeline.ts";

const bluePlayer = { Steam: "blue-wall-aerial" } as Record<string, unknown>;
const orangePlayer = { Steam: "orange-wall-aerial" } as Record<string, unknown>;

test("wall-aerial event derivation can populate compacted player stats", () => {
  const timeline = createStatsTimeline({
    events: {
      wall_aerial: [
        {
          time: 2,
          frame: 20,
          sample_time: 2,
          sample_frame: 20,
          player: bluePlayer,
          is_team_0: true,
          wall: "side",
          wall_contact_time: 1,
          wall_contact_frame: 10,
          takeoff_time: 1.4,
          takeoff_frame: 14,
          time_since_takeoff: 0.6,
          wall_contact_position: [4096, 0, 180],
          takeoff_position: [3900, 0, 240],
          player_position: [1200, 500, 420],
          ball_position: [1250, 520, 500],
          setup_start_time: 0.8,
          setup_start_frame: 8,
          setup_duration: 0.6,
          ball_speed: 1600,
          ball_speed_change: 200,
          goal_alignment: 0.5,
          confidence: 0.8,
        },
        {
          time: 3,
          frame: 30,
          sample_time: 3,
          sample_frame: 30,
          player: orangePlayer,
          is_team_0: false,
          wall: "back",
          wall_contact_time: 2.2,
          wall_contact_frame: 22,
          takeoff_time: 2.5,
          takeoff_frame: 25,
          time_since_takeoff: 0.5,
          wall_contact_position: [1200, -5120, 200],
          takeoff_position: [1200, -4800, 250],
          player_position: [500, -3000, 380],
          ball_position: [520, -2950, 460],
          setup_start_time: 2,
          setup_start_frame: 20,
          setup_duration: 0.5,
          ball_speed: 1400,
          ball_speed_change: 180,
          goal_alignment: 0.4,
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
      for (const key of Object.keys(player.wall_aerial)) {
        delete (player.wall_aerial as Record<string, unknown>)[key];
      }
    }
  }

  applyWallAerialEventDerivedStats(timeline);

  assert.equal(timeline.frames[0]?.players[0]?.wall_aerial.count, 1);
  assert.equal(timeline.frames[0]?.players[0]?.wall_aerial.high_confidence_count, 1);
  assert.equal(timeline.frames[0]?.players[0]?.wall_aerial.is_last_wall_aerial, true);
  assert.equal(timeline.frames[0]?.players[0]?.wall_aerial.cumulative_touch_height, 420);

  assert.equal(timeline.frames[1]?.players[0]?.wall_aerial.is_last_wall_aerial, false);
  assert.equal(timeline.frames[1]?.players[0]?.wall_aerial.frames_since_last_wall_aerial, 5);
  assert.equal(timeline.frames[1]?.players[0]?.wall_aerial.time_since_last_wall_aerial, 0.5);

  assert.equal(timeline.frames[2]?.players[0]?.wall_aerial.is_last_wall_aerial, false);
  assert.equal(timeline.frames[2]?.players[1]?.wall_aerial.count, 1);
  assert.equal(timeline.frames[2]?.players[1]?.wall_aerial.high_confidence_count, 0);
  assert.equal(timeline.frames[2]?.players[1]?.wall_aerial.is_last_wall_aerial, true);
  assert.equal(timeline.frames[2]?.players[1]?.wall_aerial.cumulative_setup_duration, 0.5);
});
