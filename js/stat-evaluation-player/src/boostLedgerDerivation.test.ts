import test from "node:test";
import assert from "node:assert/strict";

import {
  applyBoostLedgerDerivedStats,
  findBoostLedgerDerivationMismatches,
} from "./boostLedgerDerivation.ts";
import { createStatsFrame, createStatsTimeline } from "./testStatsTimeline.ts";
import type { MaterializedStatsTimeline } from "./statsTimeline.ts";

const playerId = { Steam: "ledger-player" } as Record<string, unknown>;
const eventDerivedBoostFields = [
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

function ledgerTimeline(): MaterializedStatsTimeline {
  return createStatsTimeline({
    events: {
      boost_state: [
        {
          frame: 1,
          time: 1,
          player_id: playerId,
          is_team_0: true,
          boost_amount: 0,
          boost_before: null,
        },
        {
          frame: 2,
          time: 2,
          player_id: playerId,
          is_team_0: true,
          boost_amount: 32,
          boost_before: 0,
        },
        {
          frame: 3,
          time: 3,
          player_id: playerId,
          is_team_0: true,
          boost_amount: 27,
          boost_before: 32,
        },
      ],
      boost_ledger: [
        {
          frame: 1,
          time: 1,
          player_id: playerId,
          is_team_0: true,
          transaction: "respawn",
          amount: 33,
          count: 0,
          labels: [{ key: "transaction", value: "respawn" }],
          boost_before: 0,
          boost_after: 33,
        },
        {
          frame: 2,
          time: 2,
          player_id: playerId,
          is_team_0: true,
          transaction: "collected",
          amount: 12,
          count: 1,
          labels: [
            { key: "transaction", value: "collected" },
            { key: "pad_size", value: "small" },
            { key: "activity", value: "active" },
            { key: "field_half", value: "own" },
          ],
          boost_before: 20,
          boost_after: 32,
        },
        {
          frame: 3,
          time: 3,
          player_id: playerId,
          is_team_0: true,
          transaction: "used",
          amount: 5,
          count: 0,
          labels: [
            { key: "transaction", value: "used" },
            { key: "vertical_state", value: "grounded" },
            { key: "supersonic", value: "false" },
          ],
          boost_before: 32,
          boost_after: 27,
        },
        {
          frame: 3,
          time: 3,
          player_id: playerId,
          is_team_0: true,
          transaction: "used_allocation",
          amount: 5,
          count: 0,
          labels: [
            { key: "transaction", value: "used" },
            { key: "vertical_state", value: "grounded" },
            { key: "supersonic", value: "false" },
          ],
          boost_before: 32,
          boost_after: 27,
        },
      ],
    },
    frames: [
      createStatsFrame({
        frame_number: 1,
        time: 1,
        dt: 0.1,
        is_live_play: true,
        team_zero: {
          boost: {
            tracked_time: 0.1,
            time_zero_boost: 0.1,
            time_boost_0_25: 0.1,
            amount_respawned: 33,
          },
        },
        players: [
          {
            player_id: playerId,
            is_team_0: true,
            boost: {
              tracked_time: 0.1,
              time_zero_boost: 0.1,
              time_boost_0_25: 0.1,
              amount_respawned: 33,
            },
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
}

test("boost ledger derivation matches serialized boost partial sums", () => {
  assert.deepEqual(findBoostLedgerDerivationMismatches(ledgerTimeline()), []);
});

test("boost ledger derivation can populate boost partial sums for player rendering", () => {
  const timeline = ledgerTimeline();
  for (const frame of timeline.frames) {
    for (const field of eventDerivedBoostFields) {
      delete (frame.team_zero.boost as Partial<typeof frame.team_zero.boost>)[field];
    }
    for (const player of frame.players) {
      for (const field of eventDerivedBoostFields) {
        delete (player.boost as Partial<typeof player.boost>)[field];
      }
    }
  }

  applyBoostLedgerDerivedStats(timeline);

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
