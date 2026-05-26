import test from "node:test";
import assert from "node:assert/strict";

import { applyRushEventDerivedStats } from "./rushEventDerivation.ts";
import { createStatsFrame, createStatsTimeline } from "./testStatsTimeline.ts";

test("rush event derivation can populate compacted team stats at retention threshold", () => {
  const timeline = createStatsTimeline({
    config: {
      rush_min_possession_retained_seconds: 0.75,
    },
    events: {
      rush: [
        {
          start_time: 1,
          start_frame: 10,
          end_time: 2,
          end_frame: 20,
          is_team_0: true,
          attackers: 2,
          defenders: 1,
        },
        {
          start_time: 3,
          start_frame: 30,
          end_time: 4,
          end_frame: 40,
          is_team_0: false,
          attackers: 3,
          defenders: 2,
        },
      ],
    },
    frames: [
      createStatsFrame({ frame_number: 10, time: 1 }),
      createStatsFrame({ frame_number: 17, time: 1.7 }),
      createStatsFrame({ frame_number: 18, time: 1.8 }),
      createStatsFrame({ frame_number: 37, time: 3.7 }),
      createStatsFrame({ frame_number: 38, time: 3.8 }),
    ],
  });

  for (const frame of timeline.frames) {
    for (const key of Object.keys(frame.team_zero.rush)) {
      delete (frame.team_zero.rush as Record<string, unknown>)[key];
    }
    for (const key of Object.keys(frame.team_one.rush)) {
      delete (frame.team_one.rush as Record<string, unknown>)[key];
    }
  }

  applyRushEventDerivedStats(timeline);

  assert.equal(timeline.frames[0]?.team_zero.rush.count, 0);
  assert.equal(timeline.frames[1]?.team_zero.rush.count, 0);
  assert.equal(timeline.frames[2]?.team_zero.rush.count, 1);
  assert.equal(timeline.frames[2]?.team_zero.rush.two_v_one_count, 1);

  assert.equal(timeline.frames[3]?.team_one.rush.count, 0);
  assert.equal(timeline.frames[4]?.team_one.rush.count, 1);
  assert.equal(timeline.frames[4]?.team_one.rush.three_v_two_count, 1);
});
