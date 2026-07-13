import test from "node:test";
import assert from "node:assert/strict";

import type { ExpectedGoalsTimelineTracks } from "./generated/ExpectedGoalsTimelineTracks.ts";
import {
  applyExpectedGoalsTrackDerivedStats,
  createExpectedGoalsTrackDerivedStatsAccumulator,
} from "./expectedGoalsTrackDerivation.ts";
import { createStatsTimeline } from "./testStatsTimeline.ts";

const playerId = { Steam: "expected-goals-player" };

function fixture() {
  const timeline = createStatsTimeline({
    frames: [0, 10, 20].map((frame) => ({
      frame_number: frame,
      time: frame / 10,
      dt: 1,
      players: [{ player_id: playerId, name: "Player", is_team_0: true }],
    })),
  });
  (
    timeline as typeof timeline & {
      expected_goals_tracks: ExpectedGoalsTimelineTracks;
    }
  ).expected_goals_tracks = {
    config: {
      episode_threshold: 0.15,
      episode_end_threshold: 0.05,
      goal_touch_exclusion_seconds: 0.5,
      incident_xg_calibration_factor: 0.629475,
    },
    teams: [
      {
        is_team_0: true,
        points: [
          {
            frame: 10,
            stats: {
              current_threat: 0.35,
              incident_xg: 0.19,
              xg: 0.25,
              episode_count: 1,
              goal_episode_count: 0,
            },
          },
          {
            frame: 20,
            stats: {
              current_threat: null,
              incident_xg: 0.19,
              xg: 0.6,
              episode_count: 1,
              goal_episode_count: 1,
            },
          },
        ],
      },
      {
        is_team_0: false,
        points: [
          {
            frame: 20,
            stats: {
              current_threat: 0.08,
              incident_xg: 0,
              xg: 0.1,
              episode_count: 0,
              goal_episode_count: 0,
            },
          },
        ],
      },
    ],
    episodes: [],
    players: [
      {
        player_id: playerId,
        is_team_0: true,
        points: [
          {
            frame: 10,
            stats: {
              threat_added: 0.4,
              xg: 0.2,
              credited_episode_count: 1,
              credited_goal_episode_count: 0,
            },
          },
          {
            frame: 20,
            stats: {
              threat_added: 0.4,
              xg: 0.2,
              credited_episode_count: 1,
              credited_goal_episode_count: 1,
            },
          },
        ],
      },
    ],
  };
  return timeline;
}

function assertHydrated(timeline: ReturnType<typeof fixture>): void {
  assert.equal(timeline.frames[0]?.team_zero.expected_goals.xg, 0);
  assert.equal(timeline.frames[0]?.players[0]?.expected_goals.threat_added, 0);

  assert.equal(timeline.frames[1]?.team_zero.expected_goals.xg, 0.25);
  assert.equal(timeline.frames[1]?.team_zero.expected_goals.current_threat, 0.35);
  assert.equal(timeline.frames[1]?.team_zero.expected_goals.incident_xg, 0.19);
  assert.equal(timeline.frames[1]?.team_zero.expected_goals.episode_count, 1);
  assert.equal(timeline.frames[1]?.players[0]?.expected_goals.threat_added, 0.4);
  assert.equal(timeline.frames[1]?.players[0]?.expected_goals.xg, 0.2);

  assert.equal(timeline.frames[2]?.team_zero.expected_goals.xg, 0.6);
  assert.equal(timeline.frames[2]?.team_zero.expected_goals.current_threat, null);
  assert.equal(timeline.frames[2]?.team_zero.expected_goals.incident_xg, 0.19);
  assert.equal(timeline.frames[2]?.team_zero.expected_goals.goal_episode_count, 1);
  assert.equal(timeline.frames[2]?.team_one.expected_goals.xg, 0.1);
  assert.equal(timeline.frames[2]?.team_one.expected_goals.current_threat, 0.08);
  assert.equal(timeline.frames[2]?.players[0]?.expected_goals.credited_goal_episode_count, 1);
}

test("expected-goals tracks hydrate full and sparse player/team snapshots", () => {
  const timeline = fixture();
  applyExpectedGoalsTrackDerivedStats(timeline);
  assertHydrated(timeline);
});

test("expected-goals tracks hydrate incrementally with held change-point values", () => {
  const timeline = fixture();
  const accumulator = createExpectedGoalsTrackDerivedStatsAccumulator(timeline);
  for (const frame of timeline.frames) accumulator.applyFrame(frame);
  assertHydrated(timeline);
});

test("missing expected-goals tracks preserve zero defaults for older compact payloads", () => {
  const timeline = createStatsTimeline({
    frames: [{ frame_number: 1, players: [{ player_id: playerId, is_team_0: true }] }],
  });
  applyExpectedGoalsTrackDerivedStats(timeline);
  assert.equal(timeline.frames[0]?.team_zero.expected_goals.xg, 0);
  assert.equal(timeline.frames[0]?.team_zero.expected_goals.current_threat, null);
  assert.equal(timeline.frames[0]?.players[0]?.expected_goals.xg, 0);
});
