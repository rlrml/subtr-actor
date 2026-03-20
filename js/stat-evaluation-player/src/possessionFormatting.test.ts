import test from "node:test";
import assert from "node:assert/strict";

import { renderPossessionStats } from "./possessionFormatting.ts";

test("renderPossessionStats shows possession-state breakdown rows when selected", () => {
  const html = renderPossessionStats(
    {
      tracked_time: 10,
      team_zero_time: 4,
      team_one_time: 3,
      neutral_time: 3,
    },
    {
      labelPerspective: {
        kind: "team",
        isTeamZero: true,
      },
      breakdownClasses: ["possession_state"],
      exportedStats: [
        {
          domain: "possession",
          name: "time",
          variant: "labeled",
          unit: "seconds",
          labels: [{ key: "possession_state", value: "team_zero" }],
          value_type: "float",
          value: 4,
        },
        {
          domain: "possession",
          name: "time",
          variant: "labeled",
          unit: "seconds",
          labels: [{ key: "possession_state", value: "team_one" }],
          value_type: "float",
          value: 3,
        },
        {
          domain: "possession",
          name: "time",
          variant: "labeled",
          unit: "seconds",
          labels: [{ key: "possession_state", value: "neutral" }],
          value_type: "float",
          value: 3,
        },
      ],
    },
  );

  assert.match(html, /Tracked<\/span><span class="value">10\.0s/);
  assert.match(html, /Team control<\/span><span class="value">4\.0s \(40\.0%\)/);
  assert.match(html, /Opp control<\/span><span class="value">3\.0s \(30\.0%\)/);
  assert.match(html, /Neutral<\/span><span class="value">3\.0s \(30\.0%\)/);
});

test("renderPossessionStats can render a shared control breakdown", () => {
  const html = renderPossessionStats(
    {
      tracked_time: 10,
      team_zero_time: 4,
      team_one_time: 3,
      neutral_time: 3,
    },
    {
      labelPerspective: {
        kind: "shared",
      },
      breakdownClasses: ["possession_state"],
    },
  );

  assert.match(html, /Blue control<\/span><span class="value">4\.0s \(40\.0%\)/);
  assert.match(html, /Neutral<\/span><span class="value">3\.0s \(30\.0%\)/);
  assert.match(html, /Orange control<\/span><span class="value">3\.0s \(30\.0%\)/);
  assert.doesNotMatch(html, /Team control<\/span>/);
  assert.doesNotMatch(html, /Opp control<\/span>/);
});

test("renderPossessionStats omits breakdown rows when no classes are selected", () => {
  const html = renderPossessionStats(
    {
      tracked_time: 5,
      team_zero_time: 2,
      team_one_time: 1,
      neutral_time: 2,
    },
    {
      labelPerspective: {
        kind: "team",
        isTeamZero: false,
      },
    },
  );

  assert.doesNotMatch(html, /Team control<\/span>/);
  assert.doesNotMatch(html, /Opp control<\/span>/);
  assert.doesNotMatch(html, /Neutral<\/span>/);
});
