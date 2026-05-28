import test from "node:test";
import assert from "node:assert/strict";

import { applyFiftyFiftyEventDerivedStats } from "./fiftyFiftyEventDerivation.ts";
import { createStatsFrame, createStatsTimeline } from "./testStatsTimeline.ts";

const bluePlayer = { Steam: "blue-fifty" } as Record<string, unknown>;
const orangePlayer = { Steam: "orange-fifty" } as Record<string, unknown>;

test("fifty-fifty event derivation can populate compacted player and team stats", () => {
  const timeline = createStatsTimeline({
    events: {
      fifty_fifty: [
        {
          start_time: 1,
          start_frame: 10,
          resolve_time: 2,
          resolve_frame: 20,
          is_kickoff: true,
          team_zero_player: bluePlayer,
          team_one_player: orangePlayer,
          team_zero_touch_time: 1,
          team_zero_touch_frame: 10,
          team_zero_dodge_contact: true,
          team_one_touch_time: 1,
          team_one_touch_frame: 10,
          team_one_dodge_contact: false,
          team_zero_position: [0, -100, 0],
          team_one_position: [0, 100, 0],
          midpoint: [0, 0, 0],
          plane_normal: [0, 1, 0],
          winning_team_is_team_0: true,
          possession_team_is_team_0: false,
        },
        {
          start_time: 3,
          start_frame: 30,
          resolve_time: 4,
          resolve_frame: 40,
          is_kickoff: false,
          team_zero_player: bluePlayer,
          team_one_player: orangePlayer,
          team_zero_touch_time: 3,
          team_zero_touch_frame: 30,
          team_zero_dodge_contact: false,
          team_one_touch_time: 3,
          team_one_touch_frame: 30,
          team_one_dodge_contact: true,
          team_zero_position: [0, -100, 0],
          team_one_position: [0, 100, 0],
          midpoint: [0, 0, 0],
          plane_normal: [0, 1, 0],
          winning_team_is_team_0: null,
          possession_team_is_team_0: true,
        },
      ],
    },
    frames: [
      createStatsFrame({
        frame_number: 19,
        time: 1.9,
        players: [
          { player_id: bluePlayer, is_team_0: true },
          { player_id: orangePlayer, is_team_0: false },
        ],
      }),
      createStatsFrame({
        frame_number: 20,
        time: 2,
        players: [
          { player_id: bluePlayer, is_team_0: true },
          { player_id: orangePlayer, is_team_0: false },
        ],
      }),
      createStatsFrame({
        frame_number: 40,
        time: 4,
        players: [
          { player_id: bluePlayer, is_team_0: true },
          { player_id: orangePlayer, is_team_0: false },
        ],
      }),
    ],
  });

  for (const frame of timeline.frames) {
    for (const key of Object.keys(frame.team_zero.fifty_fifty)) {
      delete (frame.team_zero.fifty_fifty as Record<string, unknown>)[key];
    }
    for (const key of Object.keys(frame.team_one.fifty_fifty)) {
      delete (frame.team_one.fifty_fifty as Record<string, unknown>)[key];
    }
    for (const player of frame.players) {
      for (const key of Object.keys(player.fifty_fifty)) {
        delete (player.fifty_fifty as Record<string, unknown>)[key];
      }
    }
  }

  applyFiftyFiftyEventDerivedStats(timeline);

  assert.equal(timeline.frames[0]?.team_zero.fifty_fifty.count, 0);
  assert.equal(timeline.frames[1]?.team_zero.fifty_fifty.count, 1);
  assert.equal(timeline.frames[1]?.team_zero.fifty_fifty.wins, 1);
  assert.equal(timeline.frames[1]?.team_one.fifty_fifty.losses, 1);
  assert.equal(timeline.frames[1]?.team_one.fifty_fifty.possession_after_count, 1);
  assert.equal(timeline.frames[1]?.players[0]?.fifty_fifty.kickoff_wins, 1);
  assert.equal(timeline.frames[1]?.players[1]?.fifty_fifty.kickoff_losses, 1);

  assert.equal(timeline.frames[2]?.team_zero.fifty_fifty.count, 2);
  assert.equal(timeline.frames[2]?.team_zero.fifty_fifty.neutral_outcomes, 1);
  assert.equal(timeline.frames[2]?.players[0]?.fifty_fifty.possession_after_count, 1);
  assert.equal(
    (timeline.frames[2]?.players[0]?.fifty_fifty as { labeled_event_counts?: unknown })
      .labeled_event_counts != null,
    true,
  );
  assert.equal(
    timeline.frames[1]?.players[0]?.fifty_fifty.labeled_event_counts?.entries.some((entry) =>
      entry.labels.some((label) => label.key === "dodge_state" && label.value === "dodge"),
    ),
    true,
  );
  assert.equal(
    timeline.frames[2]?.players[1]?.fifty_fifty.labeled_event_counts?.entries.some((entry) =>
      entry.labels.some((label) => label.key === "dodge_state" && label.value === "dodge"),
    ),
    true,
  );
});
