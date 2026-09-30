import test from "node:test";
import assert from "node:assert/strict";

import { applyKickoffEventDerivedStats } from "./kickoffEventDerivation.ts";
import { createStatsFrame, createStatsTimeline } from "./testStatsTimeline.ts";

for (const winnerIsTeamZero of [true, false]) {
  test(`kickoff team stats attribute taker samples and wins independently (${winnerIsTeamZero})`, () => {
    const timeline = createStatsTimeline({
      events: {
        kickoff: [
          {
            end_frame: 10,
            end_time: 1,
            outcome: winnerIsTeamZero ? "team_zero_win" : "team_one_win",
            win_strength: 0.5,
            kickoff_possession_outcome: "contested",
            team_zero_taker: {
              player: { Steam: "1" },
              is_team_0: true,
              outcome: "fake",
              boost_after: 11,
            },
            team_one_taker: {
              player: { Steam: "2" },
              is_team_0: false,
              outcome: "missed",
              boost_after: 40,
            },
            team_zero_non_takers: [],
            team_one_non_takers: [],
          },
          {
            end_frame: 20,
            end_time: 2,
            outcome: winnerIsTeamZero ? "team_zero_win" : "team_one_win",
            win_strength: 0.25,
            kickoff_possession_outcome: "contested",
            team_zero_taker: {
              player: { Steam: "1" },
              is_team_0: true,
              outcome: "missed",
              boost_after: null,
            },
            team_one_taker: {
              player: { Steam: "2" },
              is_team_0: false,
              outcome: "fake",
              boost_after: 0,
            },
            team_zero_non_takers: [],
            team_one_non_takers: [],
          },
        ],
      },
      frames: [0, 10, 15, 20].map((frame_number) => createStatsFrame({ frame_number })),
    });

    applyKickoffEventDerivedStats(timeline);

    assert.equal(timeline.frames[0]!.team_zero.kickoff.count, 0);
    const first = timeline.frames[1]!;
    assert.equal(first.team_zero.kickoff.fake_count, 1);
    assert.equal(first.team_zero.kickoff.missed_count, 0);
    assert.equal(first.team_one.kickoff.fake_count, 0);
    assert.equal(first.team_one.kickoff.missed_count, 1);
    assert.equal(first.team_zero.kickoff.cumulative_boost_after, 11);
    assert.equal(first.team_one.kickoff.cumulative_boost_after, 40);
    assert.deepEqual(timeline.frames[2]!.team_zero.kickoff, first.team_zero.kickoff);
    assert.deepEqual(timeline.frames[2]!.team_one.kickoff, first.team_one.kickoff);
    const last = timeline.frames[3]!;
    for (const [isTeamZero, stats] of [
      [true, last.team_zero.kickoff],
      [false, last.team_one.kickoff],
    ] as const) {
      assert.equal(stats.fake_count, 1);
      assert.equal(stats.missed_count, 1);
      assert.equal(stats.boost_after_sample_count, isTeamZero ? 1 : 2);
      assert.equal(stats.cumulative_boost_after, isTeamZero ? 11 : 40);
      assert.equal(stats.win_strength_sample_count, isTeamZero === winnerIsTeamZero ? 2 : 0);
      assert.equal(stats.cumulative_win_strength, isTeamZero === winnerIsTeamZero ? 0.75 : 0);
    }
  });
}

test("kickoff team stats ignore unassigned strength and absent samples", () => {
  const timeline = createStatsTimeline({
    events: {
      kickoff: ["neutral", "unknown", "team_zero_win", "team_one_win"].map((outcome) => ({
        end_frame: 10,
        end_time: 1,
        outcome,
        win_strength: outcome === "neutral" || outcome === "unknown" ? 0.5 : null,
        kickoff_possession_outcome: "contested",
        team_zero_taker: null,
        team_one_taker: null,
        team_zero_non_takers: [],
        team_one_non_takers: [],
      })),
    },
    frames: [createStatsFrame({ frame_number: 10 })],
  });

  applyKickoffEventDerivedStats(timeline);

  for (const stats of [
    timeline.frames[0]!.team_zero.kickoff,
    timeline.frames[0]!.team_one.kickoff,
  ]) {
    assert.equal(stats.count, 4);
    assert.equal(stats.wins, 1);
    assert.equal(stats.losses, 1);
    assert.equal(stats.win_strength_sample_count, 0);
    assert.equal(stats.cumulative_win_strength, 0);
    assert.equal(stats.boost_after_sample_count, 0);
    assert.equal(stats.cumulative_boost_after, 0);
    assert.equal(stats.fake_count, 0);
    assert.equal(stats.missed_count, 0);
  }
});
