import test from "node:test";
import assert from "node:assert/strict";

import { projectedRangeTimelineBounds } from "../src/timeline-overlay";
import type { ReplayPlayerTimelineProjection } from "../src/types";

function projection(
  timelineTime: number,
  hiddenBySkip: boolean,
): ReplayPlayerTimelineProjection {
  return {
    replayTime: timelineTime,
    timelineTime,
    seekTime: timelineTime,
    hiddenBySkip,
  };
}

test("collapsed skipped ranges remain visible as timeline ticks", () => {
  assert.deepEqual(
    projectedRangeTimelineBounds(
      projection(10, true),
      projection(10, true),
      120,
    ),
    {
      startTimelineTime: 10,
      endTimelineTime: 10.01,
    },
  );
});

test("non-skipped projected ranges keep their original bounds", () => {
  assert.deepEqual(
    projectedRangeTimelineBounds(
      projection(10, false),
      projection(10.08, false),
      120,
    ),
    {
      startTimelineTime: 10,
      endTimelineTime: 10.08,
    },
  );
});
