import test from "node:test";
import assert from "node:assert/strict";

import { renderRushStats } from "./rushFormatting.ts";

test("renderRushStats shows the selected team's matchup counts", () => {
  const html = renderRushStats({
    team_zero_count: 7,
    team_zero_two_v_one_count: 2,
    team_zero_two_v_two_count: 1,
    team_zero_two_v_three_count: 0,
    team_zero_three_v_one_count: 1,
    team_zero_three_v_two_count: 2,
    team_zero_three_v_three_count: 1,
    team_one_count: 4,
    team_one_two_v_one_count: 1,
    team_one_two_v_two_count: 1,
    team_one_two_v_three_count: 1,
    team_one_three_v_one_count: 0,
    team_one_three_v_two_count: 0,
    team_one_three_v_three_count: 1,
  }, true);

  assert.match(html, /Rushes<\/span><span class="value">7<\/span>/);
  assert.match(html, /2v1<\/span><span class="value">2<\/span>/);
  assert.match(html, /3v2<\/span><span class="value">2<\/span>/);
  assert.match(html, /3v3<\/span><span class="value">1<\/span>/);
});

test("renderRushStats flips to the other team perspective", () => {
  const html = renderRushStats({
    team_zero_count: 7,
    team_zero_two_v_one_count: 2,
    team_zero_two_v_two_count: 1,
    team_zero_two_v_three_count: 0,
    team_zero_three_v_one_count: 1,
    team_zero_three_v_two_count: 2,
    team_zero_three_v_three_count: 1,
    team_one_count: 4,
    team_one_two_v_one_count: 1,
    team_one_two_v_two_count: 1,
    team_one_two_v_three_count: 1,
    team_one_three_v_one_count: 0,
    team_one_three_v_two_count: 0,
    team_one_three_v_three_count: 1,
  }, false);

  assert.match(html, /Rushes<\/span><span class="value">4<\/span>/);
  assert.match(html, /2v3<\/span><span class="value">1<\/span>/);
  assert.match(html, /3v1<\/span><span class="value">0<\/span>/);
  assert.match(html, /3v3<\/span><span class="value">1<\/span>/);
});
