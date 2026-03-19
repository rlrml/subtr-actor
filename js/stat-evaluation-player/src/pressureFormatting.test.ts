import test from "node:test";
import assert from "node:assert/strict";

import { renderPressureStats } from "./pressureFormatting.ts";

test("renderPressureStats shows field-half breakdown rows when selected", () => {
  const html = renderPressureStats(
    {
      tracked_time: 8,
      team_zero_side_time: 4,
      neutral_time: 1,
      team_one_side_time: 3,
    },
    {
      labelPerspective: {
        kind: "team",
        isTeamZero: true,
      },
      exportedStats: [
        {
          domain: "pressure",
          name: "time",
          variant: "labeled",
          unit: "seconds",
          labels: [{ key: "field_half", value: "neutral" }],
          value_type: "float",
          value: 1,
        },
        {
          domain: "pressure",
          name: "time",
          variant: "labeled",
          unit: "seconds",
          labels: [{ key: "field_half", value: "team_zero_side" }],
          value_type: "float",
          value: 4,
        },
        {
          domain: "pressure",
          name: "time",
          variant: "labeled",
          unit: "seconds",
          labels: [{ key: "field_half", value: "team_one_side" }],
          value_type: "float",
          value: 3,
        },
      ],
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
      team_zero_side_time: 4,
      neutral_time: 1,
      team_one_side_time: 3,
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
      team_zero_side_time: 2,
      neutral_time: 0,
      team_one_side_time: 2,
    },
    {
      labelPerspective: {
        kind: "team",
        isTeamZero: false,
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
      team_zero_side_time: 0,
      neutral_time: 0,
      team_one_side_time: 0,
    },
    {
      labelPerspective: {
        kind: "team",
        isTeamZero: false,
      },
    },
  );

  assert.match(html, /Tracked<\/span><span class="value">4\.0s/);
  assert.doesNotMatch(html, /Own half<\/span>/);
  assert.doesNotMatch(html, /Opp half<\/span>/);
});
