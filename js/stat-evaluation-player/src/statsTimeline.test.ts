import test from "node:test";
import assert from "node:assert/strict";

import {
  createStatsFrameLookup,
  getStatsFrameForReplayFrame,
  isCompactStatsTimeline,
  type StatsTimeline,
} from "./statsTimeline.ts";
import { createStatsTimeline } from "./testStatsTimeline.ts";

test("stats frame lookup uses replay frame_number instead of array index", () => {
  const statsTimeline: StatsTimeline = createStatsTimeline({
    frames: [
      {
        frame_number: 10,
        time: 0,
        dt: 0,
      },
      {
        frame_number: 11,
        time: 0.1,
        dt: 0.1,
      },
      {
        frame_number: 15,
        time: 0.2,
        dt: 0.1,
      },
    ],
  });

  const lookup = createStatsFrameLookup(statsTimeline);

  assert.equal(statsTimeline.frames[1]?.frame_number, 11);
  assert.equal(statsTimeline.frames[2]?.frame_number, 15);
  assert.equal(getStatsFrameForReplayFrame(lookup, 2), null);
  assert.equal(getStatsFrameForReplayFrame(lookup, 15), statsTimeline.frames[2]);
});

test("stats frame lookup adds event counts to materialized frames", () => {
  const player = { Steam: "player-a" };
  const statsTimeline: StatsTimeline = createStatsTimeline({
    events: {
      mechanics: [
        {
          id: "flip_reset:1:0",
          kind: "flip_reset",
          player_id: player,
          is_team_0: true,
          timing: { type: "moment", frame: 1, time: 1 },
        },
      ],
    },
    frames: [
      {
        frame_number: 0,
        time: 0,
        dt: 0,
        players: [{ player_id: player, name: "A", is_team_0: true }],
      },
      {
        frame_number: 1,
        time: 1,
        dt: 1,
        players: [{ player_id: player, name: "A", is_team_0: true }],
      },
    ],
  });

  const lookup = createStatsFrameLookup(statsTimeline);

  assert.equal(getStatsFrameForReplayFrame(lookup, 0)?.players[0]?.event_counts?.flip_reset, 0);
  assert.equal(getStatsFrameForReplayFrame(lookup, 1)?.players[0]?.event_counts?.flip_reset, 1);
  assert.equal(getStatsFrameForReplayFrame(lookup, 1)?.team_zero.event_counts?.flip_reset, 1);
});

test("stats frame lookup materializes compact scaffold frames from events", () => {
  const playerA = { Steam: "player-a" };
  const playerB = { Steam: "player-b" };
  const statsTimeline: StatsTimeline = createStatsTimeline({
    events: {
      demolition: [
        {
          time: 0.5,
          frame: 0,
          attacker: playerA,
          victim: playerB,
          attacker_is_team_0: true,
          victim_is_team_0: false,
        },
      ],
    },
    frames: [0, 1].map((frameNumber) => ({
      frame_number: frameNumber,
      time: frameNumber,
      dt: 1,
      players: [
        { player_id: playerA, name: "A", is_team_0: true },
        { player_id: playerB, name: "B", is_team_0: false },
      ],
    })),
  });

  for (const frame of statsTimeline.frames) {
    frame.team_zero = {};
    frame.team_one = {};
    frame.players = frame.players.map(({ player_id, name, is_team_0 }) => ({
      player_id,
      name,
      is_team_0,
    }));
  }

  assert.equal(isCompactStatsTimeline(statsTimeline), true);
  const lookup = createStatsFrameLookup(statsTimeline);
  const frameOne = getStatsFrameForReplayFrame(lookup, 1);

  assert.equal(frameOne?.team_zero.demo.demos_inflicted, 1);
  assert.equal(frameOne?.team_one.demo.demos_inflicted, 0);
  assert.equal(frameOne?.players[0]?.demo.demos_inflicted, 1);
  assert.equal(frameOne?.players[1]?.demo.demos_taken, 1);
});

test("non-compact stats frame lookup reports immediate deriving-stats completion", () => {
  const player = { Steam: "player-a" };
  const statsTimeline: StatsTimeline = createStatsTimeline({
    frames: [
      {
        frame_number: 0,
        time: 0,
        dt: 0,
        players: [{ player_id: player, name: "A", is_team_0: true }],
      },
      {
        frame_number: 1,
        time: 1,
        dt: 1,
        players: [{ player_id: player, name: "A", is_team_0: true }],
      },
    ],
  });

  assert.equal(isCompactStatsTimeline(statsTimeline), false);
  const progress: import("./replayLoadProgress.ts").ReplayLoadProgress[] = [];
  const lookup = createStatsFrameLookup(statsTimeline, (entry) => {
    progress.push(entry);
  });

  assert.equal(lookup.materializeNextChunk(), false);
  assert.deepEqual(progress, [
    { stage: "deriving-stats", processedFrames: 2, totalFrames: 2, progress: 1 },
  ]);
  // Idempotent: a second drive step does not emit again.
  assert.equal(lookup.materializeNextChunk(), false);
  assert.equal(progress.length, 1);
  // Lazy get() still works after driving.
  assert.equal(getStatsFrameForReplayFrame(lookup, 1)?.frame_number, 1);
});

test("stats frame lookup rejects mixed compact and materialized frame payloads", () => {
  const player = { Steam: "player-a" };
  const statsTimeline: StatsTimeline = createStatsTimeline({
    frames: [
      {
        frame_number: 0,
        time: 0,
        dt: 0,
        team_zero: {},
        team_one: {},
        players: [{ player_id: player, name: "A", is_team_0: true }],
      },
      {
        frame_number: 1,
        time: 1,
        dt: 1,
        players: [{ player_id: player, name: "A", is_team_0: true }],
      },
    ],
  });

  statsTimeline.frames[0]!.team_zero = {};
  statsTimeline.frames[0]!.team_one = {};
  statsTimeline.frames[0]!.players = [{ player_id: player, name: "A", is_team_0: true }];

  assert.equal(isCompactStatsTimeline(statsTimeline), false);
  assert.throws(
    () => createStatsFrameLookup(statsTimeline),
    /all compact scaffolds or all materialized snapshots/,
  );
});
