import test from "node:test";
import assert from "node:assert/strict";

import {
  formatReplayLoadProgress,
  getReplayLoadCompletion,
  getReplayLoadPhase,
  getReplayLoadPhaseStates,
  listReplayLoadPhases,
} from "./replayLoadProgress.ts";

function assertApproximatelyEqual(actual: number, expected: number): void {
  assert.ok(Math.abs(actual - expected) < 1e-9, `${actual} !== ${expected}`);
}

test("replay loading exposes each user-visible work phase", () => {
  assert.deepEqual(
    listReplayLoadPhases().map((phase) => ({
      stage: phase.stage,
      index: phase.index,
      total: phase.total,
      label: phase.label,
    })),
    [
      {
        stage: "validating",
        index: 1,
        total: 8,
        label: "Parse replay",
      },
      {
        stage: "processing",
        index: 2,
        total: 8,
        label: "Process replay frames",
      },
      {
        stage: "building-stats",
        index: 3,
        total: 8,
        label: "Build stats snapshots",
      },
      {
        stage: "serializing-replay",
        index: 4,
        total: 8,
        label: "Serialize replay data",
      },
      {
        stage: "serializing-stats",
        index: 5,
        total: 8,
        label: "Serialize stats timeline",
      },
      {
        stage: "decoding-replay",
        index: 6,
        total: 8,
        label: "Decode replay data",
      },
      {
        stage: "decoding-stats",
        index: 7,
        total: 8,
        label: "Decode stats chunks",
      },
      {
        stage: "normalizing",
        index: 8,
        total: 8,
        label: "Normalize replay model",
      },
    ],
  );
});

test("getReplayLoadCompletion maps replay loading stages to monotonic overall progress", () => {
  assertApproximatelyEqual(
    getReplayLoadCompletion({ stage: "validating", progress: 0 }),
    0.04,
  );
  assertApproximatelyEqual(
    getReplayLoadCompletion({ stage: "processing", progress: 0 }),
    0.08,
  );
  assertApproximatelyEqual(
    getReplayLoadCompletion({ stage: "processing", progress: 0.5 }),
    0.35,
  );
  assertApproximatelyEqual(
    getReplayLoadCompletion({ stage: "processing", progress: 1 }),
    0.62,
  );
  assertApproximatelyEqual(
    getReplayLoadCompletion({ stage: "building-stats", progress: 0.5 }),
    0.66,
  );
  assertApproximatelyEqual(
    getReplayLoadCompletion({ stage: "serializing-replay", progress: 0.5 }),
    0.73,
  );
  assertApproximatelyEqual(
    getReplayLoadCompletion({ stage: "serializing-stats", progress: 0.5 }),
    0.81,
  );
  assertApproximatelyEqual(
    getReplayLoadCompletion({ stage: "decoding-replay", progress: 1 }),
    0.89,
  );
  assertApproximatelyEqual(
    getReplayLoadCompletion({ stage: "decoding-stats", progress: 1 }),
    0.94,
  );
  assertApproximatelyEqual(
    getReplayLoadCompletion({ stage: "normalizing", progress: 0.6 }),
    0.97,
  );
});

test("getReplayLoadCompletion clamps processing progress into the expected range", () => {
  assertApproximatelyEqual(
    getReplayLoadCompletion({ stage: "processing", progress: -1 }),
    0.08,
  );
  assertApproximatelyEqual(
    getReplayLoadCompletion({ stage: "processing", progress: 2 }),
    0.62,
  );
});

test("legacy stats timeline progress maps to the newer concrete phases", () => {
  assert.deepEqual(
    getReplayLoadPhase({ stage: "stats-timeline", progress: 0.25 }),
    {
      stage: "building-stats",
      index: 3,
      total: 8,
      label: "Build stats snapshots",
    },
  );
  assert.deepEqual(
    getReplayLoadPhase({ stage: "stats-timeline", progress: 0.42 }),
    {
      stage: "serializing-replay",
      index: 4,
      total: 8,
      label: "Serialize replay data",
    },
  );
  assert.deepEqual(
    getReplayLoadPhase({ stage: "stats-timeline", progress: 0.75 }),
    {
      stage: "serializing-stats",
      index: 5,
      total: 8,
      label: "Serialize stats timeline",
    },
  );
});

test("getReplayLoadPhaseStates marks prior detailed phases complete", () => {
  const states = getReplayLoadPhaseStates({
    stage: "decoding-stats",
    processedChunks: 2,
    totalChunks: 4,
    progress: 0.5,
  });

  assert.deepEqual(
    states.map((state) => ({
      stage: state.stage,
      state: state.state,
      completion: state.completion,
      indeterminate: state.indeterminate,
    })),
    [
      {
        stage: "validating",
        state: "complete",
        completion: 1,
        indeterminate: false,
      },
      {
        stage: "processing",
        state: "complete",
        completion: 1,
        indeterminate: false,
      },
      {
        stage: "building-stats",
        state: "complete",
        completion: 1,
        indeterminate: false,
      },
      {
        stage: "serializing-replay",
        state: "complete",
        completion: 1,
        indeterminate: false,
      },
      {
        stage: "serializing-stats",
        state: "complete",
        completion: 1,
        indeterminate: false,
      },
      {
        stage: "decoding-replay",
        state: "complete",
        completion: 1,
        indeterminate: false,
      },
      {
        stage: "decoding-stats",
        state: "active",
        completion: 0.5,
        indeterminate: false,
      },
      {
        stage: "normalizing",
        state: "pending",
        completion: 0,
        indeterminate: false,
      },
    ],
  );
});

test("getReplayLoadPhaseStates keeps legacy stats timeline indeterminate without progress", () => {
  const states = getReplayLoadPhaseStates({ stage: "stats-timeline" });

  assert.equal(states[2]?.stage, "building-stats");
  assert.equal(states[2]?.state, "active");
  assert.equal(states[2]?.completion, 1);
  assert.equal(states[2]?.indeterminate, true);
  assert.equal(states[3]?.state, "pending");
});

test("formatReplayLoadProgress reports detailed stats and decode work", () => {
  assert.equal(
    formatReplayLoadProgress({
      stage: "building-stats",
      processedFrames: 3,
      totalFrames: 10,
      progress: 0.3,
    }),
    "Building stats snapshots... 30% (3/10)",
  );
  assert.equal(
    formatReplayLoadProgress({ stage: "stats-timeline", progress: 0.42 }),
    "Serializing replay data... 35%",
  );
  assert.equal(
    formatReplayLoadProgress({
      stage: "decoding-stats",
      processedChunks: 2,
      totalChunks: 4,
      progress: 0.5,
    }),
    "Decoding stats chunks... 50% (2/4)",
  );
  assert.equal(
    formatReplayLoadProgress({ stage: "normalizing", progress: 1 }),
    "Normalizing replay model... 100%",
  );
});
