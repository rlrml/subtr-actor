import test from "node:test";
import assert from "node:assert/strict";

import { STATS_TIMELINE_EVENT_DERIVED_APPLIERS } from "./replayLoader.ts";
import { createPlayerStatsSnapshot, createTeamStatsSnapshot } from "./testStatsTimeline.ts";

const PLAYER_IDENTITY_FIELDS = new Set(["player_id", "name", "is_team_0"]);

function sorted(values: Iterable<string>): string[] {
  return [...values].sort((left, right) => left.localeCompare(right));
}

test("loader event-derived appliers cover the transferred player and team stats shape", () => {
  const playerModules = Object.keys(createPlayerStatsSnapshot()).filter(
    (module) => !PLAYER_IDENTITY_FIELDS.has(module),
  );
  const teamModules = Object.keys(createTeamStatsSnapshot());
  const derivedPlayerModules = new Set(
    STATS_TIMELINE_EVENT_DERIVED_APPLIERS.flatMap((applier) => applier.playerModules),
  );
  const derivedTeamModules = new Set(
    STATS_TIMELINE_EVENT_DERIVED_APPLIERS.flatMap((applier) => applier.teamModules),
  );
  const applierIds = STATS_TIMELINE_EVENT_DERIVED_APPLIERS.map((applier) => applier.id);

  assert.deepEqual(sorted(derivedPlayerModules), sorted(playerModules));
  assert.deepEqual(sorted(derivedTeamModules), sorted(teamModules));
  assert.deepEqual(sorted(applierIds), sorted(new Set(applierIds)));
});
