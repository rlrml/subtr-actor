import test from "node:test";
import assert from "node:assert/strict";

import { applyBallThirdEventDerivedStats } from "./ballThirdEventDerivation.ts";
import { createStatsFrame, createStatsTimeline } from "./testStatsTimeline.ts";

function assertClose(actual: number | undefined, expected: number): void {
  assert.ok(actual != null && Math.abs(actual - expected) < 1e-6, `${actual} != ${expected}`);
}

test("ball third event derivation populates compacted team stats", () => {
  const timeline = createStatsTimeline({
    events: {
      ball_third: [
        { time: 1, frame: 10, active: true, field_third: "team_zero_third" },
        { time: 1.1, frame: 11, active: true, field_third: "neutral_third" },
        { time: 1.2, frame: 12, active: true, field_third: "team_one_third" },
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
    Object.keys(frame.team_zero.ball_third).forEach((key) => {
      delete (frame.team_zero.ball_third as Record<string, unknown>)[key];
    });
    Object.keys(frame.team_one.ball_third).forEach((key) => {
      delete (frame.team_one.ball_third as Record<string, unknown>)[key];
    });
  }

  const derived = applyBallThirdEventDerivedStats(timeline);

  assert.equal(derived.frames[0]!.team_zero.ball_third.tracked_time, 0);
  assertClose(derived.frames[3]!.team_zero.ball_third.tracked_time, 0.6);
  assertClose(derived.frames[3]!.team_zero.ball_third.defensive_third_time, 0.1);
  assertClose(derived.frames[3]!.team_zero.ball_third.neutral_third_time, 0.2);
  assertClose(derived.frames[3]!.team_zero.ball_third.offensive_third_time, 0.3);
  assertClose(derived.frames[3]!.team_one.ball_third.defensive_third_time, 0.3);
  assertClose(derived.frames[3]!.team_one.ball_third.offensive_third_time, 0.1);

  const blueDefensive = derived.frames[3]!.team_zero.ball_third.labeled_time?.entries.find(
    (entry) =>
      entry.labels.some(
        (label) => label.key === "field_third" && label.value === "defensive_third",
      ),
  );
  assertClose(blueDefensive?.value, 0.1);

  const orangeDefensive = derived.frames[3]!.team_one.ball_third.labeled_time?.entries.find(
    (entry) =>
      entry.labels.some(
        (label) => label.key === "field_third" && label.value === "defensive_third",
      ),
  );
  assertClose(orangeDefensive?.value, 0.3);
});
