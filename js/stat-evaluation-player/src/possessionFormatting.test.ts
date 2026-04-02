import test from "node:test";
import assert from "node:assert/strict";

import { renderPossessionStats } from "./possessionFormatting.ts";

test("renderPossessionStats shows possession-state breakdown rows when selected", () => {
  const html = renderPossessionStats(
    {
      tracked_time: 10,
      possession_time: 4,
      opponent_possession_time: 3,
      neutral_time: 3,
      labeled_time: {
        entries: [
          {
            labels: [
              { key: "possession_state", value: "own" },
              { key: "field_third", value: "defensive_third" },
            ],
            value: 4,
          },
          {
            labels: [
              { key: "possession_state", value: "opponent" },
              { key: "field_third", value: "offensive_third" },
            ],
            value: 3,
          },
          {
            labels: [
              { key: "possession_state", value: "neutral" },
              { key: "field_third", value: "neutral_third" },
            ],
            value: 3,
          },
        ],
      },
    },
    {
      labelPerspective: {
        kind: "team",
      },
      breakdownClasses: ["possession_state"],
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
      possession_time: 4,
      opponent_possession_time: 3,
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

test("renderPossessionStats can render possession by third in team perspective", () => {
  const html = renderPossessionStats(
    {
      tracked_time: 10,
      possession_time: 6,
      opponent_possession_time: 3,
      neutral_time: 1,
      labeled_time: {
        entries: [
          {
            labels: [
              { key: "possession_state", value: "own" },
              { key: "field_third", value: "defensive_third" },
            ],
            value: 2,
          },
          {
            labels: [
              { key: "possession_state", value: "own" },
              { key: "field_third", value: "neutral_third" },
            ],
            value: 4,
          },
          {
            labels: [
              { key: "possession_state", value: "neutral" },
              { key: "field_third", value: "offensive_third" },
            ],
            value: 1,
          },
          {
            labels: [
              { key: "possession_state", value: "opponent" },
              { key: "field_third", value: "offensive_third" },
            ],
            value: 3,
          },
        ],
      },
    },
    {
      labelPerspective: {
        kind: "team",
      },
      breakdownClasses: ["possession_state", "field_third"],
    },
  );

  assert.match(html, /Team control \/ Own third<\/span><span class="value">2\.0s \(20\.0%\)/);
  assert.match(html, /Team control \/ Neutral third<\/span><span class="value">4\.0s \(40\.0%\)/);
  assert.match(html, /Neutral \/ Opp third<\/span><span class="value">1\.0s \(10\.0%\)/);
  assert.match(html, /Opp control \/ Opp third<\/span><span class="value">3\.0s \(30\.0%\)/);
});

test("renderPossessionStats can render a field-third breakdown on its own", () => {
  const html = renderPossessionStats(
    {
      tracked_time: 10,
      possession_time: 4,
      opponent_possession_time: 4,
      neutral_time: 2,
      labeled_time: {
        entries: [
          {
            labels: [
              { key: "possession_state", value: "own" },
              { key: "field_third", value: "defensive_third" },
            ],
            value: 3,
          },
          {
            labels: [
              { key: "possession_state", value: "neutral" },
              { key: "field_third", value: "neutral_third" },
            ],
            value: 2,
          },
          {
            labels: [
              { key: "possession_state", value: "opponent" },
              { key: "field_third", value: "offensive_third" },
            ],
            value: 5,
          },
        ],
      },
    },
    {
      labelPerspective: {
        kind: "shared",
      },
      breakdownClasses: ["field_third"],
    },
  );

  assert.match(html, /Blue third<\/span><span class="value">3\.0s \(30\.0%\)/);
  assert.match(html, /Neutral third<\/span><span class="value">2\.0s \(20\.0%\)/);
  assert.match(html, /Orange third<\/span><span class="value">5\.0s \(50\.0%\)/);
});

test("renderPossessionStats omits breakdown rows when no classes are selected", () => {
  const html = renderPossessionStats(
    {
      tracked_time: 5,
      possession_time: 2,
      opponent_possession_time: 1,
      neutral_time: 2,
    },
    {
      labelPerspective: {
        kind: "team",
      },
    },
  );

  assert.doesNotMatch(html, /Team control<\/span>/);
  assert.doesNotMatch(html, /Opp control<\/span>/);
  assert.doesNotMatch(html, /Neutral<\/span>/);
});
