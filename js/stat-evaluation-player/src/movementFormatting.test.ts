import test from "node:test";
import assert from "node:assert/strict";

import { renderMovementStats } from "./movementFormatting.ts";

test("renderMovementStats shows movement-time breakdowns for selected classes", () => {
  const html = renderMovementStats({
    tracked_time: 6,
    total_distance: 1234,
    speed_integral: 6000,
    time_slow_speed: 0,
    time_boost_speed: 0,
    time_supersonic_speed: 0,
    time_on_ground: 0,
    time_low_air: 0,
    time_high_air: 0,
    labeled_tracked_time: {
      entries: [
        {
          labels: [
            { key: "speed_band", value: "slow" },
            { key: "height_band", value: "ground" },
          ],
          value: 2,
        },
        {
          labels: [
            { key: "speed_band", value: "boost" },
            { key: "height_band", value: "low_air" },
          ],
          value: 1.5,
        },
        {
          labels: [
            { key: "speed_band", value: "supersonic" },
            { key: "height_band", value: "high_air" },
          ],
          value: 2.5,
        },
      ],
    },
  }, {
    breakdownClasses: ["speed_band", "height_band"],
  });

  assert.match(html, /Tracked<\/span><span class="value">6\.0s/);
  assert.match(html, /Distance<\/span><span class="value">1234 uu/);
  assert.match(html, /Avg speed<\/span><span class="value">1000 uu\/s/);
  assert.match(html, /Slow \/ Ground<\/span><span class="value">2\.0s \(33\.3%\)/);
  assert.match(html, /Boost \/ Low air<\/span><span class="value">1\.5s \(25\.0%\)/);
  assert.match(html, /Supersonic \/ High air<\/span><span class="value">2\.5s \(41\.7%\)/);
});

test("renderMovementStats omits breakdown rows when no classes are selected", () => {
  const html = renderMovementStats({
    tracked_time: 2,
    total_distance: 500,
    speed_integral: 1600,
    time_slow_speed: 0,
    time_boost_speed: 0,
    time_supersonic_speed: 0,
    time_on_ground: 0,
    time_low_air: 0,
    time_high_air: 0,
  });

  assert.doesNotMatch(html, /Slow<\/span>/);
  assert.doesNotMatch(html, /Ground<\/span>/);
});
