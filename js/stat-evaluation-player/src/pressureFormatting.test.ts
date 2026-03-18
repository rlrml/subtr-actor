import test from "node:test";
import assert from "node:assert/strict";

import { renderPressureStats } from "./pressureFormatting.ts";

test("renderPressureStats shows field-half breakdown rows when selected", () => {
  const html = renderPressureStats(
    {
      tracked_time: 8,
      team_zero_side_time: 5,
      team_one_side_time: 3,
    },
    {
      isTeamZero: true,
      breakdownClasses: ["field_half"],
      exportedStats: [
        {
          domain: "pressure",
          name: "time",
          variant: "labeled",
          unit: "seconds",
          labels: [{ key: "field_half", value: "team_zero_side" }],
          value_type: "float",
          value: 5,
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

  assert.match(html, /Tracked<\/span><span class="value">8\.0s/);
  assert.match(html, /Own half<\/span><span class="value">5\.0s \(62\.5%\)/);
  assert.match(html, /Opp half<\/span><span class="value">3\.0s \(37\.5%\)/);
});

test("renderPressureStats omits breakdown rows when no classes are selected", () => {
  const html = renderPressureStats(
    {
      tracked_time: 4,
      team_zero_side_time: 2,
      team_one_side_time: 2,
    },
    {
      isTeamZero: false,
    },
  );

  assert.doesNotMatch(html, /Own half<\/span>/);
  assert.doesNotMatch(html, /Opp half<\/span>/);
});
