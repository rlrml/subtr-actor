import test from "node:test";
import assert from "node:assert/strict";

import {
  formatIncidentXg,
  formatThreatProbability,
  normalizeThreatProbability,
} from "./scoreboardWindow.ts";

test("live threat probabilities render as bounded percentages", () => {
  assert.equal(formatThreatProbability(0.1264), "12.6%");
  assert.equal(formatThreatProbability(0), "0.0%");
  assert.equal(formatThreatProbability(1), "100.0%");
  assert.equal(formatThreatProbability(null), "--");
  assert.equal(formatThreatProbability(Number.NaN), "--");

  assert.equal(normalizeThreatProbability(-0.2), 0);
  assert.equal(normalizeThreatProbability(1.4), 1);
  assert.equal(normalizeThreatProbability(undefined), null);
});

test("incident xG renders as a goal-count total", () => {
  assert.equal(formatIncidentXg(2.345), "2.35");
  assert.equal(formatIncidentXg(0), "0.00");
  assert.equal(formatIncidentXg(null), "--");
  assert.equal(formatIncidentXg(Number.NaN), "--");
});
