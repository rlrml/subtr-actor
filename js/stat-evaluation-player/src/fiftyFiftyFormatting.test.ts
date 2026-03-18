import test from "node:test";
import assert from "node:assert/strict";

import {
  renderFiftyFiftySummary,
  renderPlayerFiftyFiftyStats,
} from "./fiftyFiftyFormatting.ts";

test("renderFiftyFiftySummary renders team-perspective win and possession counts", () => {
  const html = renderFiftyFiftySummary({
    count: 6,
    team_zero_wins: 4,
    team_one_wins: 1,
    neutral_outcomes: 1,
    kickoff_count: 3,
    kickoff_team_zero_wins: 2,
    kickoff_team_one_wins: 1,
    kickoff_neutral_outcomes: 0,
    team_zero_possession_after_count: 5,
    team_one_possession_after_count: 1,
    neutral_possession_after_count: 0,
    kickoff_team_zero_possession_after_count: 2,
    kickoff_team_one_possession_after_count: 1,
    kickoff_neutral_possession_after_count: 0,
  }, true);

  assert.match(html, /50s<\/span><span class="value">6<\/span>/);
  assert.match(html, /Wins<\/span><span class="value">4 \(66\.7%\)<\/span>/);
  assert.match(html, /Poss after<\/span><span class="value">5<\/span>/);
  assert.match(html, /Kickoff wins<\/span><span class="value">2<\/span>/);
});

test("renderPlayerFiftyFiftyStats renders player totals and kickoff split", () => {
  const html = renderPlayerFiftyFiftyStats({
    count: 5,
    wins: 3,
    losses: 1,
    neutral_outcomes: 1,
    kickoff_count: 2,
    kickoff_wins: 1,
    kickoff_losses: 1,
    kickoff_neutral_outcomes: 0,
    possession_after_count: 4,
    kickoff_possession_after_count: 1,
  });

  assert.match(html, /50s<\/span><span class="value">5<\/span>/);
  assert.match(html, /Wins<\/span><span class="value">3 \(60\.0%\)<\/span>/);
  assert.match(html, /Kickoff 50s<\/span><span class="value">2<\/span>/);
  assert.match(html, /Kickoff poss<\/span><span class="value">1<\/span>/);
});
