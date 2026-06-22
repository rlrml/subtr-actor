import test from "node:test";
import assert from "node:assert/strict";

import { applyTouchEventDerivedStats } from "./touchEventDerivation.ts";
import { createStatsFrame, createStatsTimeline } from "./testStatsTimeline.ts";

const bluePlayer = { Steam: "blue-touch" } as Record<string, unknown>;

function frameWithBlue(frameNumber: number, time: number, isLivePlay = true) {
  return createStatsFrame({
    frame_number: frameNumber,
    time,
    dt: 0.1,
    gameplay_phase: isLivePlay ? "active_play" : "post_goal",
    is_live_play: isLivePlay,
    players: [{ player_id: bluePlayer, is_team_0: true, name: "Blue" }],
  });
}

function assertClose(actual: number | undefined, expected: number): void {
  assert.ok(actual != null && Math.abs(actual - expected) < 1e-6, `${actual} != ${expected}`);
}

test("touch event derivation uses sample frame for accumulation and touch frame for last-touch fields", () => {
  const timeline = createStatsTimeline({
    events: {
      touch: [
        {
          time: 1,
          frame: 10,
          sample_time: 1.2,
          sample_frame: 12,
          player: bluePlayer,
          is_team_0: true,
          tags: [
            { group: "kind", value: "hard_hit" },
            { group: "height_band", value: "high_air" },
            { group: "surface", value: "wall" },
            { group: "dodge_state", value: "dodge" },
          ],
          ball_speed_change: 950,
          ball_movement: {
            start_time: 1.2,
            start_frame: 12,
            end_time: 1.2,
            end_frame: 12,
            duration: 0.1,
            travel_distance: 100,
            advance_distance: 60,
            retreat_distance: 0,
            finalized: true,
          },
        },
      ],
    },
    frames: [
      frameWithBlue(10, 1),
      frameWithBlue(11, 1.1),
      frameWithBlue(12, 1.2),
      frameWithBlue(13, 1.3, false),
      frameWithBlue(14, 1.4),
    ],
  });

  const derived = applyTouchEventDerivedStats(timeline);

  assert.equal(derived.frames[0]!.players[0]!.touch.touch_count, 0);
  assert.equal(derived.frames[1]!.players[0]!.touch.touch_count, 0);

  const touchFrame = derived.frames[2]!.players[0]!.touch;
  assert.equal(touchFrame.touch_count, 1);
  assert.equal(touchFrame.hard_hit_count, 1);
  assert.equal(touchFrame.aerial_touch_count, 1);
  assert.equal(touchFrame.high_aerial_touch_count, 1);
  assert.equal(touchFrame.wall_touch_count, 1);
  assert.equal(touchFrame.is_last_touch, true);
  assert.equal(touchFrame.last_touch_frame, 10);
  assert.equal(touchFrame.last_touch_time, 1);
  assertClose(touchFrame.time_since_last_touch ?? undefined, 0.2);
  assert.equal(touchFrame.frames_since_last_touch, 2);
  assert.equal(touchFrame.last_ball_speed_change, 950);
  assert.equal(touchFrame.max_ball_speed_change, 950);
  assert.equal(touchFrame.cumulative_ball_speed_change, 950);
  assert.equal(touchFrame.total_ball_travel_distance, 100);
  assert.equal(touchFrame.total_ball_advance_distance, 60);
  const labeledTouch = touchFrame.labeled_touch_counts?.entries.find(
    (entry) =>
      entry.labels.some((label) => label.key === "kind" && label.value === "hard_hit") &&
      entry.labels.some((label) => label.key === "height_band" && label.value === "high_air") &&
      entry.labels.some((label) => label.key === "surface" && label.value === "wall") &&
      entry.labels.some((label) => label.key === "dodge_state" && label.value === "dodge"),
  );
  assert.equal(labeledTouch?.count, 1);

  assert.equal(derived.frames[3]!.players[0]!.touch.is_last_touch, true);
  assert.equal(derived.frames[4]!.players[0]!.touch.is_last_touch, false);
  assertClose(derived.frames[4]!.players[0]!.touch.time_since_last_touch ?? undefined, 0.4);
});
