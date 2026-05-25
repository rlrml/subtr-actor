import test from "node:test";
import assert from "node:assert/strict";

import { applyPressureEventDerivedStats } from "./pressureEventDerivation.ts";
import { createStatsFrame, createStatsTimeline } from "./testStatsTimeline.ts";

function assertClose(actual: number | undefined, expected: number): void {
  assert.ok(actual != null && Math.abs(actual - expected) < 1e-9, `${actual} != ${expected}`);
}

test("pressure event derivation populates compacted team stats", () => {
  const timeline = createStatsTimeline({
    events: {
      pressure: [
        { time: 1, frame: 10, dt: 0.1, field_half: "team_zero_side" },
        { time: 1.1, frame: 11, dt: 0.2, field_half: "neutral" },
        { time: 1.2, frame: 12, dt: 0.3, field_half: "team_one_side" },
      ],
    },
    frames: [
      createStatsFrame({ frame_number: 9, time: 0.9 }),
      createStatsFrame({ frame_number: 10, time: 1 }),
      createStatsFrame({ frame_number: 11, time: 1.1 }),
      createStatsFrame({ frame_number: 12, time: 1.2 }),
    ],
  });

  for (const frame of timeline.frames) {
    Object.keys(frame.team_zero.pressure).forEach((key) => {
      delete (frame.team_zero.pressure as Record<string, unknown>)[key];
    });
    Object.keys(frame.team_one.pressure).forEach((key) => {
      delete (frame.team_one.pressure as Record<string, unknown>)[key];
    });
  }

  const derived = applyPressureEventDerivedStats(timeline);

  assert.equal(derived.frames[0]!.team_zero.pressure.tracked_time, 0);
  assertClose(derived.frames[3]!.team_zero.pressure.tracked_time, 0.6);
  assertClose(derived.frames[3]!.team_zero.pressure.defensive_half_time, 0.1);
  assertClose(derived.frames[3]!.team_zero.pressure.neutral_time, 0.2);
  assertClose(derived.frames[3]!.team_zero.pressure.offensive_half_time, 0.3);
  assertClose(derived.frames[3]!.team_one.pressure.defensive_half_time, 0.3);
  assertClose(derived.frames[3]!.team_one.pressure.offensive_half_time, 0.1);

  const blueDefensive = derived.frames[3]!.team_zero.pressure.labeled_time?.entries.find((entry) =>
    entry.labels.some((label) => label.key === "field_half" && label.value === "defensive_half"),
  );
  assertClose(blueDefensive?.value, 0.1);

  const orangeDefensive = derived.frames[3]!.team_one.pressure.labeled_time?.entries.find((entry) =>
    entry.labels.some((label) => label.key === "field_half" && label.value === "defensive_half"),
  );
  assertClose(orangeDefensive?.value, 0.3);
});
