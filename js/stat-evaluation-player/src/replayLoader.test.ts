import test from "node:test";
import assert from "node:assert/strict";

import {
  formatReplayLoadProgress,
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
    0.425,
  );
  assertApproximatelyEqual(
    getReplayLoadCompletion({ stage: "processing", progress: 1 }),
    0.75,
  );
  assertApproximatelyEqual(
    getReplayLoadCompletion({ stage: "stats-timeline", progress: 0 }),
    0.75,
  );
  assertApproximatelyEqual(
    getReplayLoadCompletion({ stage: "stats-timeline" }),
    0.825,
  );
  assertApproximatelyEqual(
    getReplayLoadCompletion({ stage: "stats-timeline", progress: 1 }),
    0.9,
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
    0.75,
  );
});

test("getReplayLoadPhase exposes explicit phase metadata for the modal", () => {
  assert.deepEqual(
    getReplayLoadPhase({ stage: "processing", progress: 0 }),
    {
      stage: "processing",
      index: 2,
      total: 4,
      label: "Process replay frames",
    },
  );
});

test("getReplayLoadPhaseStates keeps processing and stats as separate bars", () => {
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
        label: "Process replay frames",
        state: "active",
        completion: 0.25,
        indeterminate: false,
      },
      {
        stage: "stats-timeline",
        index: 3,
        total: 4,
        label: "Build stats timeline",
        state: "pending",
        completion: 0,
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

test("getReplayLoadPhaseStates keeps stats timeline indeterminate without progress", () => {
  assert.deepEqual(
    getReplayLoadPhaseStates({ stage: "stats-timeline" }),
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
        label: "Process replay frames",
        state: "complete",
        completion: 1,
        indeterminate: false,
      },
      {
        stage: "stats-timeline",
        index: 3,
        total: 4,
        label: "Build stats timeline",
        state: "active",
        completion: 1,
        indeterminate: true,
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

test("getReplayLoadPhaseStates shows determinate stats timeline progress", () => {
  assert.deepEqual(
    getReplayLoadPhaseStates({ stage: "stats-timeline", progress: 0.42 }),
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
        label: "Process replay frames",
        state: "complete",
        completion: 1,
        indeterminate: false,
      },
      {
        stage: "stats-timeline",
        index: 3,
        total: 4,
        label: "Build stats timeline",
        state: "active",
        completion: 0.42,
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

test("formatReplayLoadProgress shows stats timeline percentages when available", () => {
  assert.equal(
    formatReplayLoadProgress({ stage: "stats-timeline", progress: 0.42 }),
    "Building stats timeline... 42%",
  );
  assert.equal(
    formatReplayLoadProgress({ stage: "stats-timeline" }),
    "Building stats timeline...",
  );
});

test("getReplayLoadPhaseStates shows determinate progress during normalization", () => {
  assert.deepEqual(
    getReplayLoadPhaseStates({ stage: "normalizing", progress: 0.6 }),
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
        label: "Process replay frames",
        state: "complete",
        completion: 1,
        indeterminate: false,
      },
      {
        stage: "stats-timeline",
        index: 3,
        total: 4,
        label: "Build stats timeline",
        state: "complete",
        completion: 1,
        indeterminate: false,
      },
      {
        stage: "normalizing",
        index: 4,
        total: 4,
        label: "Normalize replay data",
        state: "active",
        completion: 0.6,
        indeterminate: false,
      },
    ],
  );
});
