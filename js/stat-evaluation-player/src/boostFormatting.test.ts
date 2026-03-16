import test from "node:test";
import assert from "node:assert/strict";

import {
  formatCollectedWithRespawnBound,
  formatBoostDisplayAmount,
  toBoostDisplayUnits,
} from "./boostFormatting.ts";

test("converts raw replay boost units to 0-100 display units", () => {
  assert.equal(toBoostDisplayUnits(255), 100);
  assert.equal(toBoostDisplayUnits(510), 200);
  assert.equal(formatBoostDisplayAmount(30.6), "12");
});

test("formats missing boost amounts as unknown", () => {
  assert.equal(formatBoostDisplayAmount(undefined), "?");
  assert.equal(formatBoostDisplayAmount(null), "?");
});

test("formats collected amount with respawn-inclusive bound in parentheses", () => {
  assert.equal(formatCollectedWithRespawnBound(255, 127.5), "100 (150)");
});

test("falls back to the collected amount when respawn amount is unavailable", () => {
  assert.equal(formatCollectedWithRespawnBound(255, undefined), "100");
});
