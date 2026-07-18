import test from "node:test";
import assert from "node:assert/strict";

import {
  buildTimelineGraphPath,
  projectedRangeTimelineBounds,
  timelineEventSeekTime,
} from "../src/timeline-overlay";
import type { ReplayPlayerTimelineProjection } from "../src/types";

function projection(timelineTime: number, hiddenBySkip: boolean): ReplayPlayerTimelineProjection {
  return {
    replayTime: timelineTime,
    timelineTime,
    seekTime: timelineTime,
    hiddenBySkip,
  };
}

test("collapsed skipped ranges remain visible as timeline ticks", () => {
  assert.deepEqual(projectedRangeTimelineBounds(projection(10, true), projection(10, true), 120), {
    startTimelineTime: 10,
    endTimelineTime: 10.01,
  });
});

test("non-skipped projected ranges keep their original bounds", () => {
  assert.deepEqual(
    projectedRangeTimelineBounds(projection(10, false), projection(10.08, false), 120),
    {
      startTimelineTime: 10,
      endTimelineTime: 10.08,
    },
  );
});

test("timeline event seek targets include lead-in while preserving event timestamps", () => {
  assert.equal(timelineEventSeekTime({ kind: "goal", time: 12 }), 8);
  assert.equal(timelineEventSeekTime({ kind: "shot", time: 12 }), 10);
  assert.equal(timelineEventSeekTime({ kind: "demo", time: 1.25 }), 0);
});

test("timeline event seek targets honor explicit seek times", () => {
  assert.equal(timelineEventSeekTime({ kind: "shot", time: 12, seekTime: 6.5 }), 6.5);
});

test("timeline graph paths preserve stoppage gaps and clamp probabilities", () => {
  assert.equal(
    buildTimelineGraphPath(
      [
        { time: 0, value: 0 },
        { time: 5, value: 0.5 },
        { time: 6, value: null },
        { time: 10, value: 2 },
      ],
      10,
      0,
      1,
    ),
    "M0.00,100.00 L500.00,50.00 M1000.00,0.00",
  );
});

test("timeline graph paths use the rendered graph dimensions", () => {
  assert.equal(
    buildTimelineGraphPath(
      [
        { time: 0, value: 0 },
        { time: 5, value: 0.5 },
        { time: 10, value: 1 },
      ],
      10,
      0,
      1,
      1200,
      120,
    ),
    "M0.00,120.00 L600.00,60.00 L1200.00,0.00",
  );
});
