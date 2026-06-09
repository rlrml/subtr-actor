import test from "node:test";
import assert from "node:assert/strict";

import { applyBallHalfEventDerivedStats } from "./ballHalfEventDerivation.ts";
import { createStatsFrame, createStatsTimeline } from "./testStatsTimeline.ts";

function assertClose(actual: number | undefined, expected: number): void {
  assert.ok(actual != null && Math.abs(actual - expected) < 1e-6, `${actual} != ${expected}`);
}

test("pressure event derivation populates compacted team stats", () => {
  const timeline = createStatsTimeline({
    events: {
      ball_half: [
        { time: 1, frame: 10, active: true, field_half: "team_zero_side" },
        { time: 1.1, frame: 11, active: true, field_half: "neutral" },
        { time: 1.2, frame: 12, active: true, field_half: "team_one_side" },
      ],
    },
    frames: [
      createStatsFrame({ frame_number: 9, time: 0.9 }),
      createStatsFrame({ frame_number: 10, time: 1, dt: 0.1 }),
      createStatsFrame({ frame_number: 11, time: 1.1, dt: 0.2 }),
      createStatsFrame({ frame_number: 12, time: 1.2, dt: 0.3 }),
    ],
  });

  for (const frame of timeline.frames) {
    Object.keys(frame.team_zero.ball_half).forEach((key) => {
      delete (frame.team_zero.ball_half as Record<string, unknown>)[key];
    });
    Object.keys(frame.team_one.ball_half).forEach((key) => {
      delete (frame.team_one.ball_half as Record<string, unknown>)[key];
    });
  }

  const derived = applyBallHalfEventDerivedStats(timeline);

  assert.equal(derived.frames[0]!.team_zero.ball_half.tracked_time, 0);
  assertClose(derived.frames[3]!.team_zero.ball_half.tracked_time, 0.6);
  assertClose(derived.frames[3]!.team_zero.ball_half.defensive_half_time, 0.1);
  assertClose(derived.frames[3]!.team_zero.ball_half.neutral_time, 0.2);
  assertClose(derived.frames[3]!.team_zero.ball_half.offensive_half_time, 0.3);
  assertClose(derived.frames[3]!.team_one.ball_half.defensive_half_time, 0.3);
  assertClose(derived.frames[3]!.team_one.ball_half.offensive_half_time, 0.1);

  const blueDefensive = derived.frames[3]!.team_zero.ball_half.labeled_time?.entries.find((entry) =>
    entry.labels.some((label) => label.key === "field_half" && label.value === "defensive_half"),
  );
  assertClose(blueDefensive?.value, 0.1);

  const orangeDefensive = derived.frames[3]!.team_one.ball_half.labeled_time?.entries.find((entry) =>
    entry.labels.some((label) => label.key === "field_half" && label.value === "defensive_half"),
  );
  assertClose(orangeDefensive?.value, 0.3);
});
