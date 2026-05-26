import test from "node:test";
import assert from "node:assert/strict";

import { applyBumpEventDerivedStats } from "./bumpEventDerivation.ts";
import { createStatsFrame, createStatsTimeline } from "./testStatsTimeline.ts";

const bluePlayer = { Steam: "blue-bumper" } as Record<string, unknown>;
const blueTeammate = { Steam: "blue-teammate-bump-victim" } as Record<string, unknown>;
const orangePlayer = { Steam: "orange-bump-victim" } as Record<string, unknown>;

test("bump event derivation can populate compacted player and team stats", () => {
  const timeline = createStatsTimeline({
    events: {
      bump: [
        {
          time: 2,
          frame: 20,
          initiator: bluePlayer,
          victim: orangePlayer,
          initiator_is_team_0: true,
          victim_is_team_0: false,
          is_team_bump: false,
          strength: 700,
          confidence: 0.7,
          contact_distance: 100,
          closing_speed: 500,
          victim_impulse: 110,
          initiator_position: [0, 0, 0],
          victim_position: [100, 0, 0],
        },
        {
          time: 3,
          frame: 30,
          initiator: bluePlayer,
          victim: blueTeammate,
          initiator_is_team_0: true,
          victim_is_team_0: true,
          is_team_bump: true,
          strength: 900,
          confidence: 0.8,
          contact_distance: 90,
          closing_speed: 600,
          victim_impulse: 120,
          initiator_position: [0, 0, 0],
          victim_position: [90, 0, 0],
        },
      ],
    },
    frames: [
      createStatsFrame({
        frame_number: 20,
        time: 2,
        players: [
          { player_id: bluePlayer, is_team_0: true },
          { player_id: blueTeammate, is_team_0: true },
          { player_id: orangePlayer, is_team_0: false },
        ],
      }),
      createStatsFrame({
        frame_number: 30,
        time: 3,
        players: [
          { player_id: bluePlayer, is_team_0: true },
          { player_id: blueTeammate, is_team_0: true },
          { player_id: orangePlayer, is_team_0: false },
        ],
      }),
    ],
  });

  for (const frame of timeline.frames) {
    for (const key of Object.keys(frame.team_zero.bump)) {
      delete (frame.team_zero.bump as Record<string, unknown>)[key];
    }
    for (const key of Object.keys(frame.team_one.bump)) {
      delete (frame.team_one.bump as Record<string, unknown>)[key];
    }
    for (const player of frame.players) {
      for (const key of Object.keys(player.bump)) {
        delete (player.bump as Record<string, unknown>)[key];
      }
    }
  }

  applyBumpEventDerivedStats(timeline);

  assert.equal(timeline.frames[0]?.team_zero.bump.bumps_inflicted, 1);
  assert.equal(timeline.frames[0]?.team_zero.bump.team_bumps_inflicted, 0);
  assert.equal(timeline.frames[0]?.players[0]?.bump.bumps_inflicted, 1);
  assert.equal(timeline.frames[0]?.players[0]?.bump.last_bump_strength, 700);
  assert.equal(timeline.frames[0]?.players[2]?.bump.bumps_taken, 1);

  assert.equal(timeline.frames[1]?.team_zero.bump.bumps_inflicted, 2);
  assert.equal(timeline.frames[1]?.team_zero.bump.team_bumps_inflicted, 1);
  assert.equal(timeline.frames[1]?.players[0]?.bump.bumps_inflicted, 2);
  assert.equal(timeline.frames[1]?.players[0]?.bump.team_bumps_inflicted, 1);
  assert.equal(timeline.frames[1]?.players[0]?.bump.max_bump_strength, 900);
  assert.equal(timeline.frames[1]?.players[0]?.bump.cumulative_bump_strength, 1600);
  assert.equal(timeline.frames[1]?.players[1]?.bump.team_bumps_taken, 1);
});
