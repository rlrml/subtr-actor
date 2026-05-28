import test from "node:test";
import assert from "node:assert/strict";
import { applyEventCountDerivedStats } from "./eventCountDerivation.ts";
import { createStatsTimeline } from "./testStatsTimeline.ts";

const playerId = { Steam: "player-1" };

test("event count derivation exposes flip reset counts on player and team snapshots", () => {
  const timeline = createStatsTimeline({
    events: {
      mechanics: [
        {
          id: "flip_reset:10:0",
          kind: "flip_reset",
          player_id: playerId,
          is_team_0: true,
          timing: { type: "moment", frame: 10, time: 1 },
        },
      ],
      possession: [
        {
          time: 1.5,
          frame: 15,
          active: true,
          possession_state: "team_zero",
          field_third: "offensive",
        },
      ],
    },
    frames: [
      {
        frame_number: 9,
        time: 0.9,
        team_zero: {},
        players: [{ player_id: playerId, name: "Player 1", is_team_0: true }],
      },
      {
        frame_number: 10,
        time: 1,
        team_zero: {},
        players: [{ player_id: playerId, name: "Player 1", is_team_0: true }],
      },
      {
        frame_number: 15,
        time: 1.5,
        team_zero: {},
        players: [{ player_id: playerId, name: "Player 1", is_team_0: true }],
      },
    ],
  });

  applyEventCountDerivedStats(timeline);

  assert.equal(timeline.frames[0]?.players[0]?.event_counts?.flip_reset, 0);
  assert.equal(timeline.frames[0]?.team_zero.event_counts?.flip_reset, 0);
  assert.equal(timeline.frames[1]?.players[0]?.event_counts?.flip_reset, 1);
  assert.equal(timeline.frames[1]?.team_zero.event_counts?.flip_reset, 1);
  assert.equal(timeline.frames[1]?.players[0]?.event_counts?.mechanics, 1);
  assert.equal(timeline.frames[1]?.team_zero.event_counts?.mechanics, 1);
  assert.equal(timeline.frames[2]?.players[0]?.event_counts?.possession, 0);
  assert.equal(timeline.frames[2]?.team_zero.event_counts?.possession, 0);
});
