import test from "node:test";
import assert from "node:assert/strict";

import {
  computeTeamBandDescriptors,
  getTeamLaneBounds,
} from "./overlays.ts";

test("blue and orange team lanes stay disjoint so team bands do not fully overlap", () => {
  const blueLane = getTeamLaneBounds(true);
  const orangeLane = getTeamLaneBounds(false);

  assert.ok(blueLane.maxX < orangeLane.minX);
  assert.ok(blueLane.width > 0);
  assert.ok(orangeLane.width > 0);
});

test("team zero back and forward bands point toward opposite goals", () => {
  const descriptors = computeTeamBandDescriptors([-3200, -1800, -600], true, 236);

  assert.deepEqual(descriptors, [
    {
      kind: "back",
      centerY: -3200,
      halfDepth: 236,
      directions: [-1],
    },
    {
      kind: "forward",
      centerY: -600,
      halfDepth: 236,
      directions: [1],
    },
  ]);
});

test("team one back and forward bands reverse direction in world space", () => {
  const descriptors = computeTeamBandDescriptors([-3200, -1800, -600], false, 236);

  assert.deepEqual(descriptors, [
    {
      kind: "back",
      centerY: -600,
      halfDepth: 236,
      directions: [1],
    },
    {
      kind: "forward",
      centerY: -3200,
      halfDepth: 236,
      directions: [-1],
    },
  ]);
});

test("other bands keep both team-relative directions", () => {
  const descriptors = computeTeamBandDescriptors([-1400, -1280, -1200], false, 236);

  assert.deepEqual(descriptors, [{
    kind: "other",
    centerY: -1300,
    halfDepth: 136,
    directions: [1, -1],
  }]);
});
