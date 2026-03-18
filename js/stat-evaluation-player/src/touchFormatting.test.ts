import test from "node:test";
import assert from "node:assert/strict";

import { renderTouchStats } from "./touchFormatting.ts";

test("renderTouchStats includes touch classification and momentum rows", () => {
  const html = renderTouchStats({
    touch_count: 8,
    is_last_touch: true,
    last_touch_time: 12.34,
    last_touch_frame: 456,
    time_since_last_touch: 0.75,
    frames_since_last_touch: 9,
    last_ball_speed_change: 321.4,
    average_ball_speed_change: 512.2,
    max_ball_speed_change: 911.9,
  }, {
    breakdownClasses: ["kind", "aerial"],
    exportedStats: [
      {
        domain: "touch",
        name: "touch_count",
        variant: "labeled",
        unit: "count",
        labels: [
          { key: "kind", value: "dribble" },
          { key: "aerial", value: "false" },
          { key: "high_aerial", value: "false" },
        ],
        value_type: "unsigned",
        value: 2,
      },
      {
        domain: "touch",
        name: "touch_count",
        variant: "labeled",
        unit: "count",
        labels: [
          { key: "kind", value: "control" },
          { key: "aerial", value: "true" },
          { key: "high_aerial", value: "false" },
        ],
        value_type: "unsigned",
        value: 1,
      },
      {
        domain: "touch",
        name: "touch_count",
        variant: "labeled",
        unit: "count",
        labels: [
          { key: "kind", value: "medium_hit" },
          { key: "aerial", value: "false" },
          { key: "high_aerial", value: "false" },
        ],
        value_type: "unsigned",
        value: 3,
      },
      {
        domain: "touch",
        name: "touch_count",
        variant: "labeled",
        unit: "count",
        labels: [
          { key: "kind", value: "hard_hit" },
          { key: "aerial", value: "true" },
          { key: "high_aerial", value: "true" },
        ],
        value_type: "unsigned",
        value: 2,
      },
    ],
  });

  assert.match(html, /Touches<\/span><span class="value">8/);
  assert.match(html, /Dribble \/ Ground<\/span><span class="value">2/);
  assert.match(html, /Control \/ Aerial<\/span><span class="value">1/);
  assert.match(html, /Medium \/ Ground<\/span><span class="value">3/);
  assert.match(html, /Hard \/ Aerial<\/span><span class="value">2/);
  assert.match(html, /Last change<\/span><span class="value">321\.4/);
  assert.match(html, /Avg change<\/span><span class="value">512\.2/);
  assert.match(html, /Max change<\/span><span class="value">911\.9/);
});

test("renderTouchStats falls back to unknowns for missing momentum values", () => {
  const html = renderTouchStats({
    touch_count: 1,
    is_last_touch: false,
  });

  assert.doesNotMatch(html, /Dribbles<\/span>/);
  assert.doesNotMatch(html, /Aerials<\/span>/);
  assert.match(html, /Last change<\/span><span class="value">\?/);
});

test("renderTouchStats aggregates labeled rows by the selected class", () => {
  const html = renderTouchStats({
    touch_count: 6,
    is_last_touch: false,
  }, {
    breakdownClasses: ["aerial"],
    exportedStats: [
      {
        domain: "touch",
        name: "touch_count",
        variant: "labeled",
        unit: "count",
        labels: [
          { key: "kind", value: "dribble" },
          { key: "aerial", value: "false" },
          { key: "high_aerial", value: "false" },
        ],
        value_type: "unsigned",
        value: 2,
      },
      {
        domain: "touch",
        name: "touch_count",
        variant: "labeled",
        unit: "count",
        labels: [
          { key: "kind", value: "control" },
          { key: "aerial", value: "true" },
          { key: "high_aerial", value: "false" },
        ],
        value_type: "unsigned",
        value: 1,
      },
      {
        domain: "touch",
        name: "touch_count",
        variant: "labeled",
        unit: "count",
        labels: [
          { key: "kind", value: "hard_hit" },
          { key: "aerial", value: "true" },
          { key: "high_aerial", value: "true" },
        ],
        value_type: "unsigned",
        value: 3,
      },
    ],
  });

  assert.match(html, /Aerial<\/span><span class="value">4/);
  assert.match(html, /Ground<\/span><span class="value">2/);
});
