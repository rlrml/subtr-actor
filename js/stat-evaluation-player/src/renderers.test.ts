import test from "node:test";
import assert from "node:assert/strict";

import {
  renderAbsolutePositioningStats,
  renderRelativePositioningStats,
} from "./stat-modules/renderers.ts";
import type { PlayerStatsSnapshot } from "./statsTimeline.ts";

test("relative positioning renderer derives percentages from accumulated times", () => {
  const positioning = {
    active_game_time: 4,
    tracked_time: 4,
    time_defensive_half: 0,
    time_offensive_half: 0,
    time_demolished: 0,
    time_no_teammates: 0,
    time_most_back: 1,
    time_most_forward: 2,
    time_mid_role: 1,
    time_other_role: 0,
    time_closest_to_ball: 3,
    time_farthest_from_ball: 1,
    time_behind_ball: 2,
    time_in_front_of_ball: 2,
  } as PlayerStatsSnapshot["positioning"];

  const html = renderRelativePositioningStats(positioning);

  assert.match(html, /Most back.*25%/s);
  assert.match(html, /Most forward.*50%/s);
  assert.match(html, /Mid role.*25%/s);
  assert.match(html, /Other role.*0%/s);
  assert.match(html, /Closest to ball.*75%/s);
  assert.match(html, /Farthest from ball.*25%/s);
  assert.match(html, /Behind ball.*50%/s);
  assert.match(html, /In front of ball.*50%/s);
});

test("absolute positioning renderer derives averages from accumulated sums", () => {
  const positioning = {
    active_game_time: 4,
    tracked_time: 4,
    time_demolished: 0,
    time_no_teammates: 0,
    time_most_back: 0,
    time_most_forward: 0,
    time_mid_role: 0,
    time_other_role: 0,
    time_defensive_zone: 1.5,
    time_neutral_zone: 1,
    time_offensive_zone: 1.5,
    time_defensive_half: 2.5,
    time_offensive_half: 1.5,
    sum_distance_to_teammates: 420,
    sum_distance_to_ball: 820,
  } as PlayerStatsSnapshot["positioning"];

  const html = renderAbsolutePositioningStats(positioning);

  assert.match(html, /Defensive zone.*1\.5s/s);
  assert.match(html, /Neutral zone.*1\.0s/s);
  assert.match(html, /Offensive zone.*1\.5s/s);
  assert.match(html, /To teammates.*105/s);
  assert.match(html, /To ball.*205/s);
});
