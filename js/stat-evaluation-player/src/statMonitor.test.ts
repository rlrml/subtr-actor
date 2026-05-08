import test from "node:test";
import assert from "node:assert/strict";
import {
  buildStatDescriptors,
  formatStatMonitorValue,
  getFuzzyStatMatches,
} from "./statMonitor.ts";
import { createStatsFrame } from "./testStatsTimeline.ts";

test("buildStatDescriptors exposes scoped primitive player and team stats", () => {
  const descriptors = buildStatDescriptors(createStatsFrame({
    players: [{
      name: "Blue One",
      boost: {
        amount_used_while_airborne: 42,
      },
      touch: {
        labeled_touch_counts: {
          entries: [{
            labels: [{ key: "kind", value: "ground" }],
            count: 2,
          }],
        },
      },
    }],
    team_zero: {
      rush: {
        two_v_one_count: 3,
      },
    },
  }));

  assert.ok(descriptors.some((descriptor) =>
    descriptor.id === "player:boost.amount_used_while_airborne" &&
    descriptor.selectorLabel === "Players / Boost / Amount Used While Airborne"
  ));
  assert.ok(descriptors.some((descriptor) =>
    descriptor.id === "team:rush.two_v_one_count" &&
    descriptor.selectorLabel === "Teams / Rush / Two V One Count"
  ));
  assert.equal(
    descriptors.some((descriptor) => descriptor.id === "player:name"),
    false,
  );
  assert.equal(
    descriptors.some((descriptor) =>
      descriptor.id.startsWith("player:touch.labeled_touch_counts.entries")
    ),
    false,
  );
});

test("getFuzzyStatMatches handles incomplete stat queries", () => {
  const descriptors = buildStatDescriptors(createStatsFrame({
    players: [{
      boost: {
        amount_used_while_airborne: 42,
      },
    }],
  }));

  const [match] = getFuzzyStatMatches(descriptors, "player bst air usd", 1);

  assert.equal(match?.id, "player:boost.amount_used_while_airborne");
});

test("formatStatMonitorValue formats common stat units", () => {
  assert.equal(formatStatMonitorValue(12.345, ["boost", "tracked_time"]), "12.3s");
  assert.equal(formatStatMonitorValue(87.6, ["speed_flip", "last_quality"]), "88%");
  assert.equal(formatStatMonitorValue(4, ["touch", "touch_count"]), "4");
  assert.equal(formatStatMonitorValue(false, ["touch", "is_last_touch"]), "No");
  assert.equal(formatStatMonitorValue(null, ["touch", "last_touch_time"]), "--");
});
