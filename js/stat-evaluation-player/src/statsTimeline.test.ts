import test from "node:test";
import assert from "node:assert/strict";

import {
  createStatsFrameLookup,
  getStatsFrameForReplayFrame,
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
