import test from "node:test";
import assert from "node:assert/strict";

import {
  getReplayLoadCompletion,
  getReplayLoadPhase,
  getReplayLoadPhaseStates,
} from "./replayLoadProgress.ts";

function assertApproximatelyEqual(actual: number, expected: number): void {
  assert.ok(Math.abs(actual - expected) < 1e-9, `${actual} !== ${expected}`);
}

test("getReplayLoadCompletion maps replay loading stages to monotonic overall progress", () => {
  assertApproximatelyEqual(
    getReplayLoadCompletion({ stage: "validating", progress: 0 }),
    0.05,
  );
  assertApproximatelyEqual(
    getReplayLoadCompletion({ stage: "processing", progress: 0 }),
    0.1,
  );
  assertApproximatelyEqual(
    getReplayLoadCompletion({ stage: "processing", progress: 0.5 }),
    0.5,
  );
  assertApproximatelyEqual(
    getReplayLoadCompletion({ stage: "processing", progress: 1 }),
    0.9,
  );
  assertApproximatelyEqual(
    getReplayLoadCompletion({ stage: "stats-timeline", progress: 0 }),
    0.1,
  );
  assertApproximatelyEqual(
    getReplayLoadCompletion({ stage: "normalizing", progress: 0 }),
    0.9,
  );
  assertApproximatelyEqual(
    getReplayLoadCompletion({ stage: "normalizing", progress: 0.6 }),
    0.954,
  );
});

test("getReplayLoadCompletion clamps processing progress into the expected range", () => {
  assertApproximatelyEqual(
    getReplayLoadCompletion({ stage: "processing", progress: -1 }),
    0.1,
  );
  assertApproximatelyEqual(
    getReplayLoadCompletion({ stage: "processing", progress: 2 }),
    0.9,
  );
});

test("getReplayLoadPhase exposes explicit phase metadata for the modal", () => {
  assert.deepEqual(
    getReplayLoadPhase({ stage: "processing", progress: 0 }),
    {
      stage: "processing",
      index: 2,
      total: 3,
      label: "Process replay frames and stats",
    },
  );
});

test("getReplayLoadPhaseStates collapses processing and stats into one bar", () => {
  assert.deepEqual(
    getReplayLoadPhaseStates({ stage: "processing", progress: 0.25 }),
    [
      {
        stage: "validating",
        index: 1,
        total: 3,
        label: "Parse replay",
        state: "complete",
        completion: 1,
        indeterminate: false,
      },
      {
        stage: "processing",
        index: 2,
        total: 3,
        label: "Process replay frames and stats",
        state: "active",
        completion: 0.25,
        indeterminate: false,
      },
      {
        stage: "normalizing",
        index: 3,
        total: 3,
        label: "Normalize replay data",
        state: "pending",
        completion: 0,
        indeterminate: false,
      },
    ],
  );
});

test("getReplayLoadPhaseStates shows determinate progress during normalization", () => {
  assert.deepEqual(
    getReplayLoadPhaseStates({ stage: "normalizing", progress: 0.6 }),
    [
      {
        stage: "validating",
        index: 1,
        total: 3,
        label: "Parse replay",
        state: "complete",
        completion: 1,
        indeterminate: false,
      },
      {
        stage: "processing",
        index: 2,
        total: 3,
        label: "Process replay frames and stats",
        state: "complete",
        completion: 1,
        indeterminate: false,
      },
      {
        stage: "normalizing",
        index: 3,
        total: 3,
        label: "Normalize replay data",
        state: "active",
        completion: 0.6,
        indeterminate: false,
      },
    ],
  );
});
