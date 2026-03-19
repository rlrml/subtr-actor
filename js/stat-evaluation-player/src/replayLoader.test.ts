import test from "node:test";
import assert from "node:assert/strict";

import { getReplayLoadCompletion } from "./replayLoadProgress.ts";

function assertApproximatelyEqual(actual: number, expected: number): void {
  assert.ok(Math.abs(actual - expected) < 1e-9, `${actual} !== ${expected}`);
}

test("getReplayLoadCompletion maps replay loading stages to monotonic overall progress", () => {
  assertApproximatelyEqual(
    getReplayLoadCompletion({ stage: "validating", progress: 0 }),
    0.02,
  );
  assertApproximatelyEqual(
    getReplayLoadCompletion({ stage: "processing", progress: 0 }),
    0.05,
  );
  assertApproximatelyEqual(
    getReplayLoadCompletion({ stage: "processing", progress: 0.5 }),
    0.5,
  );
  assertApproximatelyEqual(
    getReplayLoadCompletion({ stage: "processing", progress: 1 }),
    0.95,
  );
  assertApproximatelyEqual(
    getReplayLoadCompletion({ stage: "stats-timeline", progress: 0 }),
    0.96,
  );
  assertApproximatelyEqual(
    getReplayLoadCompletion({ stage: "normalizing", progress: 0 }),
    0.99,
  );
});

test("getReplayLoadCompletion clamps processing progress into the expected range", () => {
  assertApproximatelyEqual(
    getReplayLoadCompletion({ stage: "processing", progress: -1 }),
    0.05,
  );
  assertApproximatelyEqual(
    getReplayLoadCompletion({ stage: "processing", progress: 2 }),
    0.95,
  );
});
