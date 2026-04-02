import test from "node:test";
import assert from "node:assert/strict";

import { renderPressureStats } from "./pressureFormatting.ts";

test("renderPressureStats shows field-half breakdown rows when selected", () => {
  const html = renderPressureStats(
    {
      tracked_time: 8,
      defensive_half_time: 4,
      neutral_time: 1,
      offensive_half_time: 3,
      labeled_time: {
        entries: [
          {
            labels: [{ key: "field_half", value: "neutral" }],
            value: 1,
          },
          {
            labels: [{ key: "field_half", value: "defensive_half" }],
            value: 4,
          },
          {
            labels: [{ key: "field_half", value: "offensive_half" }],
            value: 3,
          },
        ],
      },
    },
    {
      labelPerspective: {
        kind: "team",
      },
    },
  );

  assert.doesNotMatch(html, /Tracked<\/span>/);
  assert.match(html, /Own half<\/span><span class="value">4\.0s \(50\.0%\)/);
  assert.match(html, /Neutral zone<\/span><span class="value">1\.0s \(12\.5%\)/);
  assert.match(html, /Opp half<\/span><span class="value">3\.0s \(37\.5%\)/);
});

test("renderPressureStats can render a shared half-control breakdown", () => {
  const html = renderPressureStats(
    {
      tracked_time: 8,
      defensive_half_time: 4,
      neutral_time: 1,
      offensive_half_time: 3,
    },
    {
      labelPerspective: {
        kind: "shared",
      },
    },
  );

  assert.match(html, /Blue side<\/span><span class="value">4\.0s \(50\.0%\)/);
  assert.match(html, /Neutral zone<\/span><span class="value">1\.0s \(12\.5%\)/);
  assert.match(html, /Orange side<\/span><span class="value">3\.0s \(37\.5%\)/);
  assert.doesNotMatch(html, /Own half<\/span>/);
  assert.doesNotMatch(html, /Opp half<\/span>/);
});

test("renderPressureStats uses snapshot pressure totals without labeled exports", () => {
  const html = renderPressureStats(
    {
      tracked_time: 4,
      defensive_half_time: 2,
      neutral_time: 0,
      offensive_half_time: 2,
    },
    {
      labelPerspective: {
        kind: "team",
      },
    },
  );

  assert.doesNotMatch(html, /Tracked<\/span>/);
  assert.match(html, /Own half<\/span><span class="value">2\.0s \(50\.0%\)/);
  assert.match(html, /Opp half<\/span><span class="value">2\.0s \(50\.0%\)/);
});

test("renderPressureStats falls back to tracked time when no breakdown data exists", () => {
  const html = renderPressureStats(
    {
      tracked_time: 4,
      defensive_half_time: 0,
      neutral_time: 0,
      offensive_half_time: 0,
    },
    {
      labelPerspective: {
        kind: "team",
      },
    },
  );

  assert.match(html, /Tracked<\/span><span class="value">4\.0s/);
  assert.doesNotMatch(html, /Own half<\/span>/);
  assert.doesNotMatch(html, /Opp half<\/span>/);
});
