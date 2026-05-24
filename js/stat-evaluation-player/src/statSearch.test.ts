import test from "node:test";
import assert from "node:assert/strict";
import { getStatDefinitionSearchMatches } from "./statSearch.ts";
import type { StatDefinition } from "./statRegistry.ts";

function stat(id: string, label: string, category: string): StatDefinition {
  const [, pathText = label] = id.split(":");
  return {
    id,
    label,
    category,
    scope: id.startsWith("team:") ? "team" : "player",
    path: pathText.split("."),
    read: () => 0,
    format: (value) => `${value}`,
  };
}

test("stat definition search treats space-separated terms as independent filters", () => {
  const definitions = [
    stat("player:boost.amount_used", "boost.amount_used", "boost"),
    stat("player:movement.time_airborne", "movement.time_airborne", "movement"),
    stat("player:boost.amount_used_while_airborne", "boost.amount_used_while_airborne", "boost"),
  ];

  assert.deepEqual(
    getStatDefinitionSearchMatches(definitions, "boost air").map((definition) => definition.id),
    ["player:boost.amount_used_while_airborne"],
  );
});

test("stat definition search requires the typed string for each term", () => {
  const definitions = [
    stat("player:boost.amount_collected", "boost.amount_collected", "boost"),
    stat("player:boost.amount_used_while_airborne", "boost.amount_used_while_airborne", "boost"),
  ];

  assert.deepEqual(
    getStatDefinitionSearchMatches(definitions, "bst air usd").map((definition) => definition.id),
    [],
  );
  assert.deepEqual(
    getStatDefinitionSearchMatches(definitions, "boost air used").map(
      (definition) => definition.id,
    ),
    ["player:boost.amount_used_while_airborne"],
  );
});
