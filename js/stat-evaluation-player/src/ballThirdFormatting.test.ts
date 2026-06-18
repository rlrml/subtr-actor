import test from "node:test";
import assert from "node:assert/strict";

import { renderBallThirdStats } from "./ballThirdFormatting.ts";

test("renderBallThirdStats shows field-third breakdown rows when selected", () => {
  const html = renderBallThirdStats(
    {
      tracked_time: 8,
      defensive_third_time: 4,
      neutral_third_time: 1,
      offensive_third_time: 3,
      labeled_time: {
        entries: [
          {
            labels: [{ key: "field_third", value: "neutral_third" }],
            value: 1,
          },
          {
            labels: [{ key: "field_third", value: "defensive_third" }],
            value: 4,
          },
          {
            labels: [{ key: "field_third", value: "offensive_third" }],
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
  assert.match(html, /Own third<\/span><span class="value">4\.0s \(50\.0%\)/);
  assert.match(html, /Neutral third<\/span><span class="value">1\.0s \(12\.5%\)/);
  assert.match(html, /Opp third<\/span><span class="value">3\.0s \(37\.5%\)/);
});

test("renderBallThirdStats can render a shared third-control breakdown", () => {
  const html = renderBallThirdStats(
    {
      tracked_time: 8,
      defensive_third_time: 4,
      neutral_third_time: 1,
      offensive_third_time: 3,
    },
    {
      labelPerspective: {
        kind: "shared",
      },
    },
  );

  assert.match(html, /Blue third<\/span><span class="value">4\.0s \(50\.0%\)/);
  assert.match(html, /Neutral third<\/span><span class="value">1\.0s \(12\.5%\)/);
  assert.match(html, /Orange third<\/span><span class="value">3\.0s \(37\.5%\)/);
  assert.doesNotMatch(html, /Own third<\/span>/);
  assert.doesNotMatch(html, /Opp third<\/span>/);
});

test("renderBallThirdStats uses snapshot totals without labeled exports", () => {
  const html = renderBallThirdStats(
    {
      tracked_time: 4,
      defensive_third_time: 2,
      neutral_third_time: 0,
      offensive_third_time: 2,
    },
    {
      labelPerspective: {
        kind: "team",
      },
    },
  );

  assert.doesNotMatch(html, /Tracked<\/span>/);
  assert.match(html, /Own third<\/span><span class="value">2\.0s \(50\.0%\)/);
  assert.match(html, /Opp third<\/span><span class="value">2\.0s \(50\.0%\)/);
});

test("renderBallThirdStats falls back to tracked time when no breakdown data exists", () => {
  const html = renderBallThirdStats(
    {
      tracked_time: 4,
      defensive_third_time: 0,
      neutral_third_time: 0,
      offensive_third_time: 0,
    },
    {
      labelPerspective: {
        kind: "team",
      },
    },
  );

  assert.match(html, /Tracked<\/span><span class="value">4\.0s/);
  assert.doesNotMatch(html, /Own third<\/span>/);
  assert.doesNotMatch(html, /Opp third<\/span>/);
});
