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
    0.325,
  );
  assertApproximatelyEqual(
    getReplayLoadCompletion({ stage: "processing", progress: 1 }),
    0.55,
  );
  assertApproximatelyEqual(
    getReplayLoadCompletion({ stage: "stats-timeline", progress: 0 }),
    0.325,
  );
  assertApproximatelyEqual(
    getReplayLoadCompletion({ stage: "normalizing", progress: 0 }),
    0.945,
  );
});

test("getReplayLoadCompletion clamps processing progress into the expected range", () => {
  assertApproximatelyEqual(
    getReplayLoadCompletion({ stage: "processing", progress: -1 }),
    0.1,
  );
  assertApproximatelyEqual(
    getReplayLoadCompletion({ stage: "processing", progress: 2 }),
    0.55,
  );
});

test("getReplayLoadPhase exposes explicit phase metadata for the modal", () => {
  assert.deepEqual(
    getReplayLoadPhase({ stage: "processing", progress: 0 }),
    {
      stage: "processing",
      index: 2,
      total: 4,
      label: "Process replay frames and stats",
    },
  );
});

test("getReplayLoadPhaseStates drives one bar per phase", () => {
  assert.deepEqual(
    getReplayLoadPhaseStates({ stage: "processing", progress: 0.25 }),
    [
      {
        stage: "validating",
        index: 1,
        total: 4,
        label: "Parse replay",
        state: "complete",
        completion: 1,
        indeterminate: false,
      },
      {
        stage: "processing",
        index: 2,
        total: 4,
        label: "Extract replay frames",
        state: "active",
        completion: 0.25,
        indeterminate: false,
      },
      {
        stage: "stats-timeline",
        index: 3,
        total: 4,
        label: "Build stats timeline",
        state: "active",
        completion: 0.25,
        indeterminate: false,
      },
      {
        stage: "normalizing",
        index: 4,
        total: 4,
        label: "Normalize replay data",
        state: "pending",
        completion: 0,
        indeterminate: false,
      },
    ],
  );
});
