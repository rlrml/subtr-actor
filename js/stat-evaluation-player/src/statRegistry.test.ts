import test from "node:test";
import assert from "node:assert/strict";
import { createStatRegistry } from "./statRegistry.ts";

test("stat registry exposes player and team stats before a replay is loaded", () => {
  const definitions = createStatRegistry(null);
  const ids = new Set(definitions.map((definition) => definition.id));

  assert.ok(ids.has("player:boost.amount_used"));
  assert.ok(ids.has("player:core.goals"));
  assert.ok(ids.has("team:core.goals"));
  assert.ok(ids.has("team:possession.possession_time"));
});
