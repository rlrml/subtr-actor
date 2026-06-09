import test from "node:test";
import assert from "node:assert/strict";

import {
  applyBoostTrackDerivedStats,
  findBoostTrackDerivationMismatches,
} from "./boostTrackDerivation.ts";
import { createStatsFrame, createStatsTimeline } from "./testStatsTimeline.ts";
import type { AccumulationTrack } from "./generated/AccumulationTrack.ts";
import type { MaterializedStatsTimeline } from "./statsTimeline.ts";

const playerId = { Steam: "ledger-player" } as Record<string, unknown>;
const derivedBoostFields = [
  "tracked_time",
  "boost_integral",
  "time_zero_boost",
  "time_boost_0_25",
  "amount_respawned",
  "amount_collected",
  "amount_collected_small",
  "small_pads_collected",
  "amount_used",
  "amount_used_while_grounded",
] as const;

function assertClose(actual: number | undefined, expected: number): void {
  assert.ok(actual != null && Math.abs(actual - expected) < 0.000001);
}

// The boost calculator now ships pickup/respawn events plus compressed per-frame accumulation
// tracks instead of ledger/state events. The continuous integral / time-in-band fields are
// reconstructed from the boost-amount track, so the expected partial sums match the former model.
function trackTimeline(): MaterializedStatsTimeline {
  const timeline = createStatsTimeline({
    events: {
      boost_pickups: [
        {
          frame: 2,
          time: 2,
          player_id: playerId,
          is_team_0: true,
          pad_type: "small",
          field_half: "own",
          activity: "active",
          detection: "both",
          is_steal: false,
          collected_amount: 12,
          overfill_amount: 0,
          boost_before: 20,
          boost_after: 32,
        },
      ],
      boost_respawn: [
        {
          frame: 1,
          time: 1,
          player_id: playerId,
          is_team_0: true,
          kind: "kickoff",
          boost_granted: 33,
        },
      ],
    } as never,
    frames: [
      createStatsFrame({
        frame_number: 1,
        time: 1,
        dt: 0.1,
        is_live_play: true,
        team_zero: {
          boost: { tracked_time: 0.1, time_zero_boost: 0.1, time_boost_0_25: 0.1, amount_respawned: 33 },
        },
        players: [
          {
            player_id: playerId,
            is_team_0: true,
            boost: { tracked_time: 0.1, time_zero_boost: 0.1, time_boost_0_25: 0.1, amount_respawned: 33 },
          },
        ],
      }),
      createStatsFrame({
        frame_number: 2,
        time: 2,
        dt: 0.1,
        is_live_play: true,
        team_zero: {
          boost: {
            tracked_time: 0.2,
            boost_integral: 1.6,
            time_zero_boost: 0.103125,
            time_boost_0_25: 0.2,
            amount_respawned: 33,
            amount_collected: 12,
            amount_collected_small: 12,
            small_pads_collected: 1,
          },
        },
        players: [
          {
            player_id: playerId,
            is_team_0: true,
            boost: {
              tracked_time: 0.2,
              boost_integral: 1.6,
              time_zero_boost: 0.103125,
              time_boost_0_25: 0.2,
              amount_respawned: 33,
              amount_collected: 12,
              amount_collected_small: 12,
              small_pads_collected: 1,
            },
          },
        ],
      }),
      createStatsFrame({
        frame_number: 3,
        time: 3,
        dt: 0.1,
        is_live_play: true,
        team_zero: {
          boost: {
            tracked_time: 0.3,
            boost_integral: 4.55,
            time_zero_boost: 0.103125,
            time_boost_0_25: 0.3,
            amount_respawned: 33,
            amount_collected: 12,
            amount_collected_small: 12,
            small_pads_collected: 1,
            amount_used: 5,
            amount_used_while_grounded: 5,
          },
        },
        players: [
          {
            player_id: playerId,
            is_team_0: true,
            boost: {
              tracked_time: 0.3,
              boost_integral: 4.55,
              time_zero_boost: 0.103125,
              time_boost_0_25: 0.3,
              amount_respawned: 33,
              amount_collected: 12,
              amount_collected_small: 12,
              small_pads_collected: 1,
              amount_used: 5,
              amount_used_while_grounded: 5,
            },
          },
        ],
      }),
    ],
  });

  const tracks: AccumulationTrack[] = [
    {
      player_id: playerId as never,
      is_team_0: true,
      quantity: "boost_amount",
      points: [
        { frame: 1, value: 0 },
        { frame: 2, value: 32 },
        { frame: 3, value: 27 },
      ],
    },
    {
      player_id: playerId as never,
      is_team_0: true,
      quantity: "boost_used",
      points: [{ frame: 3, value: 5 }],
    },
    {
      player_id: playerId as never,
      is_team_0: true,
      quantity: "boost_used_grounded",
      points: [{ frame: 3, value: 5 }],
    },
  ];
  (timeline as { accumulation_tracks?: AccumulationTrack[] }).accumulation_tracks = tracks;
  return timeline;
}

test("boost track derivation matches serialized boost partial sums", () => {
  assert.deepEqual(findBoostTrackDerivationMismatches(trackTimeline()), []);
});

test("boost track derivation can populate boost partial sums for player rendering", () => {
  const timeline = trackTimeline();
  for (const frame of timeline.frames) {
    for (const field of derivedBoostFields) {
      delete (frame.team_zero.boost as Partial<typeof frame.team_zero.boost>)[field];
    }
    for (const player of frame.players) {
      for (const field of derivedBoostFields) {
        delete (player.boost as Partial<typeof player.boost>)[field];
      }
    }
  }

  applyBoostTrackDerivedStats(timeline);

  assert.equal(timeline.frames[2]?.players[0]?.boost.amount_respawned, 33);
  assertClose(timeline.frames[2]?.players[0]?.boost.tracked_time, 0.3);
  assertClose(timeline.frames[2]?.players[0]?.boost.time_zero_boost, 0.103125);
  assertClose(timeline.frames[2]?.players[0]?.boost.time_boost_0_25, 0.3);
  assert.equal(timeline.frames[2]?.players[0]?.boost.amount_collected, 12);
  assert.equal(timeline.frames[2]?.players[0]?.boost.amount_collected_small, 12);
  assert.equal(timeline.frames[2]?.players[0]?.boost.small_pads_collected, 1);
  assert.equal(timeline.frames[2]?.players[0]?.boost.amount_used, 5);
  assert.equal(timeline.frames[2]?.players[0]?.boost.amount_used_while_grounded, 5);
  assert.equal(timeline.frames[2]?.team_zero.boost.amount_used, 5);
});
