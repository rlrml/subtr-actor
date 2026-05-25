import test from "node:test";
import assert from "node:assert/strict";

import { applyPassEventDerivedStats } from "./passEventDerivation.ts";
import { createStatsFrame, createStatsTimeline } from "./testStatsTimeline.ts";

const passer = { Steam: "passer" } as Record<string, unknown>;
const receiver = { Steam: "receiver" } as Record<string, unknown>;
const orangePasser = { Steam: "orange-passer" } as Record<string, unknown>;

test("pass event derivation can populate compacted player and team stats", () => {
  const timeline = createStatsTimeline({
    events: {
      pass: [
        {
          time: 2,
          frame: 20,
          passer,
          receiver,
          is_team_0: true,
          start_time: 1.5,
          start_frame: 15,
          duration: 0.5,
          ball_travel_distance: 900,
          ball_advance_distance: 700,
          pass_kind: "direct",
        },
        {
          time: 3,
          frame: 30,
          passer: orangePasser,
          receiver,
          is_team_0: false,
          start_time: 2.5,
          start_frame: 25,
          duration: 0.5,
          ball_travel_distance: 1100,
          ball_advance_distance: 800,
          pass_kind: "backboard",
        },
      ],
    },
    frames: [
      createStatsFrame({
        frame_number: 20,
        time: 2,
        is_live_play: true,
        players: [
          { player_id: passer, is_team_0: true },
          { player_id: receiver, is_team_0: true },
          { player_id: orangePasser, is_team_0: false },
        ],
      }),
      createStatsFrame({
        frame_number: 25,
        time: 2.5,
        is_live_play: false,
        players: [
          { player_id: passer, is_team_0: true },
          { player_id: receiver, is_team_0: true },
          { player_id: orangePasser, is_team_0: false },
        ],
      }),
      createStatsFrame({
        frame_number: 30,
        time: 3,
        is_live_play: true,
        players: [
          { player_id: passer, is_team_0: true },
          { player_id: receiver, is_team_0: true },
          { player_id: orangePasser, is_team_0: false },
        ],
      }),
    ],
  });

  for (const frame of timeline.frames) {
    for (const key of Object.keys(frame.team_zero.pass)) {
      delete (frame.team_zero.pass as Record<string, unknown>)[key];
    }
    for (const key of Object.keys(frame.team_one.pass)) {
      delete (frame.team_one.pass as Record<string, unknown>)[key];
    }
    for (const player of frame.players) {
      for (const key of Object.keys(player.pass)) {
        delete (player.pass as Record<string, unknown>)[key];
      }
    }
  }

  applyPassEventDerivedStats(timeline);

  assert.equal(timeline.frames[0]?.team_zero.pass.completed_pass_count, 1);
  assert.equal(timeline.frames[0]?.team_zero.pass.total_pass_distance, 900);
  assert.equal(timeline.frames[0]?.players[0]?.pass.completed_pass_count, 1);
  assert.equal(timeline.frames[0]?.players[0]?.pass.is_last_completed_pass, true);
  assert.equal(timeline.frames[0]?.players[1]?.pass.received_pass_count, 1);

  assert.equal(timeline.frames[1]?.players[0]?.pass.is_last_completed_pass, false);
  assert.equal(timeline.frames[1]?.players[0]?.pass.frames_since_last_completed_pass, 5);

  assert.equal(timeline.frames[2]?.team_one.pass.completed_pass_count, 1);
  assert.equal(timeline.frames[2]?.team_one.pass.longest_pass_distance, 1100);
  assert.equal(timeline.frames[2]?.players[0]?.pass.is_last_completed_pass, false);
  assert.equal(timeline.frames[2]?.players[1]?.pass.received_pass_count, 2);
  assert.equal(timeline.frames[2]?.players[2]?.pass.is_last_completed_pass, true);
  assert.equal(timeline.frames[2]?.players[2]?.pass.frames_since_last_completed_pass, 0);
});
