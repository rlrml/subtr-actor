import test from "node:test";
import assert from "node:assert/strict";

import { renderTouchStats } from "./touchFormatting.ts";
import { createTouchStats } from "./testStatsTimeline.ts";

test("renderTouchStats shows only total and selected breakdown rows", () => {
  const html = renderTouchStats(createTouchStats({
    touch_count: 8,
    is_last_touch: true,
    labeled_touch_counts: {
      entries: [
        {
          labels: [
            { key: "kind", value: "dribble" },
            { key: "height_band", value: "ground" },
          ],
          count: 2,
        },
        {
          labels: [
            { key: "kind", value: "control" },
            { key: "height_band", value: "low_air" },
          ],
          count: 1,
        },
        {
          labels: [
            { key: "kind", value: "medium_hit" },
            { key: "height_band", value: "ground" },
          ],
          count: 3,
        },
        {
          labels: [
            { key: "kind", value: "hard_hit" },
            { key: "height_band", value: "high_air" },
          ],
          count: 2,
        },
      ],
    },
  }), {
    breakdownClasses: ["kind", "height_band"],
  });

  assert.match(html, /Touches<\/span><span class="value">8/);
  assert.match(html, /Dribble \/ Ground<\/span><span class="value">2/);
  assert.match(html, /Control \/ Low air<\/span><span class="value">1/);
  assert.match(html, /Medium \/ Ground<\/span><span class="value">3/);
  assert.match(html, /Hard \/ High air<\/span><span class="value">2/);
  assert.doesNotMatch(html, /Current<\/span>/);
});

test("renderTouchStats defaults to total touches when no classes are selected", () => {
  const html = renderTouchStats(createTouchStats({
    touch_count: 1,
    is_last_touch: false,
  }));

  assert.match(html, /Touches<\/span><span class="value">1/);
  assert.doesNotMatch(html, /Dribble<\/span>/);
  assert.doesNotMatch(html, /Ground<\/span>/);
});

test("renderTouchStats aggregates labeled rows by the selected class", () => {
  const html = renderTouchStats(createTouchStats({
    touch_count: 6,
    is_last_touch: false,
    labeled_touch_counts: {
      entries: [
        {
          labels: [
            { key: "kind", value: "dribble" },
            { key: "height_band", value: "ground" },
          ],
          count: 2,
        },
        {
          labels: [
            { key: "kind", value: "control" },
            { key: "height_band", value: "low_air" },
          ],
          count: 1,
        },
        {
          labels: [
            { key: "kind", value: "hard_hit" },
            { key: "height_band", value: "high_air" },
          ],
          count: 3,
        },
      ],
    },
  }), {
    breakdownClasses: ["height_band"],
  });

  assert.match(html, /Ground<\/span><span class="value">2/);
  assert.match(html, /Low air<\/span><span class="value">1/);
  assert.match(html, /High air<\/span><span class="value">3/);
});

test("renderTouchStats falls back to typed labeled counts for combined breakdowns", () => {
  const html = renderTouchStats(createTouchStats({
    touch_count: 4,
    is_last_touch: false,
    labeled_touch_counts: {
      entries: [
        {
          labels: [
            { key: "kind", value: "dribble" },
            { key: "height_band", value: "ground" },
          ],
          count: 1,
        },
        {
          labels: [
            { key: "kind", value: "control" },
            { key: "height_band", value: "low_air" },
          ],
          count: 1,
        },
        {
          labels: [
            { key: "kind", value: "hard_hit" },
            { key: "height_band", value: "high_air" },
          ],
          count: 2,
        },
      ],
    },
  }), {
    breakdownClasses: ["kind", "height_band"],
  });

  assert.match(html, /Dribble \/ Ground<\/span><span class="value">1/);
  assert.match(html, /Control \/ Low air<\/span><span class="value">1/);
  assert.match(html, /Hard \/ High air<\/span><span class="value">2/);
});

test("renderTouchStats falls back to legacy counts for single-class breakdowns", () => {
  const html = renderTouchStats(createTouchStats({
    touch_count: 7,
    dribble_touch_count: 2,
    control_touch_count: 1,
    medium_hit_count: 3,
    hard_hit_count: 1,
    aerial_touch_count: 2,
    high_aerial_touch_count: 1,
    is_last_touch: false,
  }), {
    breakdownClasses: ["height_band"],
  });

  assert.match(html, /Ground<\/span><span class="value">5/);
  assert.match(html, /Low air<\/span><span class="value">1/);
  assert.match(html, /High air<\/span><span class="value">1/);
});
