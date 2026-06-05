import test from "node:test";
import assert from "node:assert/strict";

import {
  resolveMechanicsReviewBoundTime,
  resolveMechanicsReviewTargetTime,
  type MechanicsReviewItem,
  type MechanicsReviewTimingReplay,
} from "./mechanicsReview.ts";

function replayWithFrame(frameIndex: number, time: number): MechanicsReviewTimingReplay {
  const frames = Array.from({ length: frameIndex + 1 }, (_, index) => ({ time: index / 30 }));
  frames[frameIndex] = { time };
  return {
    duration: 120,
    rawStartTime: 37,
    frames,
  };
}

test("raw replay review playlist times use the replay normalization offset", () => {
  const replay = replayWithFrame(3000, 63);
  const item: MechanicsReviewItem = {
    replay: "replay-id",
    start: { kind: "time", value: 96 },
    end: { kind: "time", value: 104 },
    meta: {
      target: {
        eventTime: 100,
      },
    },
  };

  assert.equal(resolveMechanicsReviewBoundTime(item, item.start, replay, "rawReplay"), 59);
  assert.equal(resolveMechanicsReviewBoundTime(item, item.end, replay, "rawReplay"), 67);
  assert.equal(resolveMechanicsReviewTargetTime(item, replay, "rawReplay"), 63);
});

test("legacy review playlist times are shifted from raw replay time into player playback time", () => {
  const replay = replayWithFrame(3000, 63);
  const item: MechanicsReviewItem = {
    replay: "replay-id",
    start: { kind: "time", value: 96 },
    end: { kind: "time", value: 104 },
    meta: {
      target: {
        eventTime: 100,
        eventFrame: 3000,
      },
    },
  };

  assert.equal(resolveMechanicsReviewBoundTime(item, item.start, replay), 59);
  assert.equal(resolveMechanicsReviewBoundTime(item, item.end, replay), 67);
  assert.equal(resolveMechanicsReviewTargetTime(item, replay), 63);
});

test("review playlist times are unchanged when they already match player playback time", () => {
  const replay = replayWithFrame(3000, 100);
  const item: MechanicsReviewItem = {
    replay: "replay-id",
    start: { kind: "time", value: 96 },
    end: { kind: "time", value: 104 },
    meta: {
      target: {
        eventTime: 100,
        eventFrame: 3000,
      },
    },
  };

  assert.equal(resolveMechanicsReviewBoundTime(item, item.start, replay, "playback"), 96);
  assert.equal(resolveMechanicsReviewBoundTime(item, item.end, replay, "playback"), 104);
  assert.equal(resolveMechanicsReviewTargetTime(item, replay, "playback"), 100);
});

test("review playlist frame bounds use replay frame playback time directly", () => {
  const replay = replayWithFrame(3000, 63);
  const item: MechanicsReviewItem = {
    replay: "replay-id",
    start: { kind: "frame", value: 3000 },
    end: { kind: "frame", value: 3000 },
    meta: {
      target: {
        eventTime: 100,
        eventFrame: 3000,
      },
    },
  };

  assert.equal(resolveMechanicsReviewBoundTime(item, item.start, replay), 63);
});
