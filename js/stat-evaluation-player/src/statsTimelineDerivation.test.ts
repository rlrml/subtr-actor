import test from "node:test";
import assert from "node:assert/strict";

import {
  createEventDerivedStatsFrameLookup,
  STATS_TIMELINE_EVENT_DERIVED_APPLIERS,
} from "./statsTimelineDerivation.ts";
import { createStatsTimeline } from "./testStatsTimeline.ts";
import type { ReplayLoadProgress } from "./replayLoadProgress.ts";

test("event-derived stats frame lookup defers derivation until first frame access", () => {
  const timeline = createStatsTimeline({
    frames: Array.from({ length: 10 }, (_, index) => ({
      frame_number: index * 10,
      time: index,
      dt: 0.1,
      players: [],
    })),
  });
  const progress: ReplayLoadProgress[] = [];

  const lookup = createEventDerivedStatsFrameLookup(
    timeline,
    (entry) => {
      progress.push(entry);
    },
    { materializationChunkSize: 3 },
  );

  assert.deepEqual(progress, []);
  assert.equal(lookup.get(10)?.frame_number, 10);
  assert.ok(progress.some((entry) => entry.stage === "deriving-stats"));
  assert.ok(progress.every((entry) => entry.processedFrames === 3 && entry.totalFrames === 10));
  const progressCount = progress.length;
  assert.equal(lookup.get(10)?.frame_number, 10);
  assert.equal(progress.length, progressCount);

  assert.equal(lookup.get(80)?.frame_number, 80);
  assert.ok(progress.slice(progressCount).every((entry) => entry.processedFrames === 9));
  assert.equal(lookup.get(90)?.frame_number, 90);
  assert.ok(progress.slice(progressCount).some((entry) => entry.processedFrames === 10));
});

test("event-derived stats frame lookup does not clone scaffold payloads before frame access", () => {
  const timeline = createStatsTimeline({
    frames: [
      {
        frame_number: 0,
        time: 0,
        dt: 0,
      },
    ],
  });
  Object.defineProperty(timeline.frames[0]!, "players", {
    get() {
      throw new Error("players should not be read until the scaffold frame is materialized");
    },
  });

  assert.doesNotThrow(() => createEventDerivedStatsFrameLookup(timeline));
});

test("event-derived stats frame lookup expands later materialization chunks", () => {
  const timeline = createStatsTimeline({
    frames: Array.from({ length: 20 }, (_, index) => ({
      frame_number: index,
      time: index,
      dt: 0.1,
      players: [],
    })),
  });
  const progress: ReplayLoadProgress[] = [];

  const lookup = createEventDerivedStatsFrameLookup(
    timeline,
    (entry) => {
      progress.push(entry);
    },
    { materializationChunkSize: 2, maxMaterializationChunkSize: 4 },
  );

  lookup.get(0);
  lookup.get(2);
  lookup.get(6);

  const materializedFrameCounts = [
    ...new Set(
      progress
        .filter((entry) => entry.stage === "deriving-stats")
        .map((entry) => entry.processedFrames),
    ),
  ];
  assert.deepEqual(materializedFrameCounts, [2, 6, 10]);
});

test("event-derived stats frame lookup applies converted modules incrementally", () => {
  const playerA = { Steam: "player-a" };
  const playerB = { Steam: "player-b" };
  const timeline = createStatsTimeline({
    events: {
      timeline: [
        { time: 0.5, kind: "Kill", player_id: playerA, is_team_0: true },
        { time: 0.5, kind: "Death", player_id: playerB, is_team_0: false },
      ],
      bump: [
        {
          time: 2,
          frame: 2,
          initiator: playerA,
          victim: playerB,
          initiator_is_team_0: true,
          victim_is_team_0: false,
          is_team_bump: false,
          strength: 12,
          confidence: 1,
          contact_distance: 0,
          closing_speed: 0,
          victim_impulse: 0,
          initiator_position: [0, 0, 0],
          victim_position: [0, 0, 0],
        },
      ],
      dodge_reset: [
        { time: 1, frame: 1, player: playerA, is_team_0: true, counter_value: 1, on_ball: true },
      ],
      powerslide: [
        { time: 1, frame: 1, player: playerA, is_team_0: true, active: true },
        { time: 2, frame: 2, player: playerA, is_team_0: true, active: false },
      ],
      rush: [
        {
          start_time: 1,
          start_frame: 1,
          end_time: 2,
          end_frame: 2,
          is_team_0: false,
          attackers: 2,
          defenders: 1,
        },
      ],
    },
    frames: [0, 1, 2].map((frameNumber) => ({
      frame_number: frameNumber,
      time: frameNumber,
      dt: 1,
      gameplay_phase: "active_play",
      players: [
        { player_id: playerA, is_team_0: true },
        { player_id: playerB, is_team_0: false },
      ],
    })),
  });

  const incrementalApplierIds = STATS_TIMELINE_EVENT_DERIVED_APPLIERS.filter(
    (applier) => applier.createFrameAccumulator,
  ).map((applier) => applier.id);
  assert.deepEqual(incrementalApplierIds, [
    "event-counts",
    "boost-ledger",
    "core",
    "possession",
    "pressure",
    "territorial-pressure",
    "movement",
    "positioning",
    "rotation",
    "mechanics",
    "whiff",
    "backboard",
    "double-tap",
    "demo",
    "fifty-fifty",
    "kickoff",
    "bump",
    "rush",
    "pass",
    "one-timer",
    "ball-carry",
    "wall-aerial",
    "wall-aerial-shot",
    "flick",
    "ceiling-shot",
    "musty-flick",
    "dodge-reset",
    "powerslide",
    "touch",
    "half-volley",
  ]);
  assert.deepEqual(
    STATS_TIMELINE_EVENT_DERIVED_APPLIERS.filter((applier) => !applier.createFrameAccumulator).map(
      (applier) => applier.id,
    ),
    [],
  );

  const lookup = createEventDerivedStatsFrameLookup(timeline, undefined, {
    materializationChunkSize: 1,
    maxMaterializationChunkSize: 1,
  });

  assert.equal(lookup.get(0)?.players[0]?.demo.demos_inflicted, 0);
  const frameOne = lookup.get(1);
  assert.equal(frameOne?.team_zero.event_counts?.dodge_reset, 1);
  assert.equal(frameOne?.players[0]?.event_counts?.dodge_reset, 1);
  assert.equal(frameOne?.team_zero.demo.demos_inflicted, 1);
  assert.equal(frameOne?.players[0]?.demo.demos_inflicted, 1);
  assert.equal(frameOne?.players[1]?.demo.demos_taken, 1);
  assert.equal(frameOne?.team_one.rush.count, 1);
  assert.equal(frameOne?.team_one.rush.two_v_one_count, 1);
  assert.equal(frameOne?.players[0]?.dodge_reset.count, 1);
  assert.equal(frameOne?.players[0]?.dodge_reset.on_ball_count, 1);
  assert.equal(frameOne?.team_zero.powerslide.press_count, 1);
  assert.equal(frameOne?.players[0]?.powerslide.press_count, 1);
  assert.equal(frameOne?.players[0]?.powerslide.total_duration, 1);

  const frameTwo = lookup.get(2);
  assert.equal(frameTwo?.team_zero.bump.bumps_inflicted, 1);
  assert.equal(frameTwo?.players[0]?.bump.bumps_inflicted, 1);
  assert.equal(frameTwo?.players[0]?.bump.cumulative_bump_strength, 12);
  assert.equal(frameTwo?.players[1]?.bump.bumps_taken, 1);
  assert.equal(frameTwo?.players[0]?.powerslide.total_duration, 1);
});
