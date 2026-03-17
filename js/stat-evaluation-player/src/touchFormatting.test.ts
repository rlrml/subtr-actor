import test from "node:test";
import assert from "node:assert/strict";

import { renderTouchStats } from "./touchFormatting.ts";

test("renderTouchStats includes touch classification and momentum rows", () => {
  const html = renderTouchStats({
    touch_count: 8,
    dribble_touch_count: 2,
    control_touch_count: 1,
    medium_hit_count: 3,
    hard_hit_count: 2,
    aerial_touch_count: 4,
    high_aerial_touch_count: 1,
    is_last_touch: true,
    last_touch_time: 12.34,
    last_touch_frame: 456,
    time_since_last_touch: 0.75,
    frames_since_last_touch: 9,
    last_ball_speed_change: 321.4,
    average_ball_speed_change: 512.2,
    max_ball_speed_change: 911.9,
  });

  assert.match(html, /Dribbles<\/span><span class="value">2/);
  assert.match(html, /Control<\/span><span class="value">1/);
  assert.match(html, /Medium<\/span><span class="value">3/);
  assert.match(html, /Hard<\/span><span class="value">2/);
  assert.match(html, /Aerials<\/span><span class="value">4/);
  assert.match(html, /High aerials<\/span><span class="value">1/);
  assert.match(html, /Last change<\/span><span class="value">321\.4/);
  assert.match(html, /Avg change<\/span><span class="value">512\.2/);
  assert.match(html, /Max change<\/span><span class="value">911\.9/);
});

test("renderTouchStats falls back to unknowns for missing touch classification values", () => {
  const html = renderTouchStats({
    touch_count: 1,
    is_last_touch: false,
  });

  assert.match(html, /Dribbles<\/span><span class="value">\?/);
  assert.match(html, /Aerials<\/span><span class="value">\?/);
  assert.match(html, /Last change<\/span><span class="value">\?/);
});
