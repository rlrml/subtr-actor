import test from "node:test";
import assert from "node:assert/strict";

import { formatPlaybackRate, snapPlaybackRate } from "./playbackRateControl.ts";

test("snapPlaybackRate sticks near common playback speeds", () => {
  assert.equal(snapPlaybackRate(0.27), 0.25);
  assert.equal(snapPlaybackRate(0.53), 0.5);
  assert.equal(snapPlaybackRate(0.97), 1);
  assert.equal(snapPlaybackRate(1.54), 1.5);
  assert.equal(snapPlaybackRate(1.96), 2);
});

test("snapPlaybackRate preserves non-notch playback speeds outside sticky zones", () => {
  assert.equal(snapPlaybackRate(1.25), 1.25);
  assert.equal(snapPlaybackRate(0.1), 0.25);
  assert.equal(snapPlaybackRate(2.5), 2);
});

test("formatPlaybackRate trims insignificant zeros", () => {
  assert.equal(formatPlaybackRate(0.25), "0.25x");
  assert.equal(formatPlaybackRate(1), "1x");
  assert.equal(formatPlaybackRate(1.5), "1.5x");
});
