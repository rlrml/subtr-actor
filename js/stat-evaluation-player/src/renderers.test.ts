import test from "node:test";
import assert from "node:assert/strict";

import {
  renderBoostStats,
  renderAbsolutePositioningStats,
  renderRelativePositioningStats,
} from "./stat-modules/renderers.ts";
import {
  createPlayerStatsSnapshot,
  createPositioningStats,
} from "./testStatsTimeline.ts";

test("relative positioning renderer shows times and derives percentages from accumulated times", () => {
  const positioning = createPositioningStats({
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
    time_behind_ball: 1,
    time_level_with_ball: 1,
    time_in_front_of_ball: 2,
  });

  const html = renderRelativePositioningStats(positioning);

  assert.match(html, /Most back.*1\.0s \(25%\)/s);
  assert.match(html, /Most forward.*2\.0s \(50%\)/s);
  assert.match(html, /Mid role.*1\.0s \(25%\)/s);
  assert.match(html, /Other role.*0\.0s \(0%\)/s);
  assert.match(html, /Closest to ball.*3\.0s \(75%\)/s);
  assert.match(html, /Farthest from ball.*1\.0s \(25%\)/s);
  assert.match(html, /Behind ball.*1\.0s \(25%\)/s);
  assert.match(html, /Level with ball.*1\.0s \(25%\)/s);
  assert.match(html, /In front of ball.*2\.0s \(50%\)/s);
});

test("absolute positioning renderer shows time shares and derives averages from accumulated sums", () => {
  const positioning = createPositioningStats({
    active_game_time: 4,
    tracked_time: 4,
    time_demolished: 0,
    time_no_teammates: 0,
    time_most_back: 0,
    time_most_forward: 0,
    time_mid_role: 0,
    time_other_role: 0,
    time_defensive_third: 1.5,
    time_neutral_third: 1,
    time_offensive_third: 1.5,
    time_defensive_half: 2.5,
    time_offensive_half: 1.5,
    sum_distance_to_teammates: 420,
    sum_distance_to_ball: 820,
  });

  const html = renderAbsolutePositioningStats(positioning);

  assert.match(html, /Defensive zone.*1\.5s \(38%\)/s);
  assert.match(html, /Neutral zone.*1\.0s \(25%\)/s);
  assert.match(html, /Offensive zone.*1\.5s \(38%\)/s);
  assert.match(html, /Defensive half.*2\.5s \(63%\)/s);
  assert.match(html, /Offensive half.*1\.5s \(38%\)/s);
  assert.match(html, /To teammates.*105/s);
  assert.match(html, /To ball.*205/s);
});

test("boost renderer shows tracked-time shares for all boost time buckets", () => {
  const boost = createPlayerStatsSnapshot({
    boost: {
      tracked_time: 8,
      boost_integral: 0,
      time_zero_boost: 1,
      time_hundred_boost: 2,
      time_boost_0_25: 1.5,
      time_boost_25_50: 2,
      time_boost_50_75: 1,
      time_boost_75_100: 0.5,
      amount_collected: 0,
      amount_stolen: 0,
      big_pads_collected: 0,
      small_pads_collected: 0,
      big_pads_stolen: 0,
      small_pads_stolen: 0,
      amount_collected_big: 0,
      amount_stolen_big: 0,
      amount_collected_small: 0,
      amount_stolen_small: 0,
      amount_respawned: 0,
      overfill_total: 0,
      overfill_from_stolen: 0,
      amount_used: 0,
      amount_used_while_grounded: 0,
      amount_used_while_airborne: 0,
      amount_used_while_supersonic: 0,
    },
  }).boost;

  const html = renderBoostStats(boost);

  assert.match(html, /Time @ 0.*1\.0s \(13%\)/s);
  assert.match(html, /Time 0-25.*1\.5s \(19%\)/s);
  assert.match(html, /Time 25-50.*2\.0s \(25%\)/s);
  assert.match(html, /Time 50-75.*1\.0s \(13%\)/s);
  assert.match(html, /Time 75-100.*0\.5s \(6%\)/s);
  assert.match(html, /Time @ 100.*2\.0s \(25%\)/s);
});
