import test from "node:test";
import assert from "node:assert/strict";

import { EVENT_DEFINITION_CATALOG } from "./generated/eventDefinitionCatalog.generated.ts";
import {
  STATS_EVENT_STREAM_COUNT_TYPES,
  STATS_MECHANIC_EVENT_COUNT_TYPES,
} from "./eventCountDerivation.ts";

// `EVENT_DEFINITION_CATALOG` is generated from the subtr-actor Rust event
// registry (the single source of truth). This test fails if a registry event is
// not represented anywhere in the player, so adding an event in Rust forces it to
// be wired into the player (or explicitly excepted here) instead of silently
// going missing — the TypeScript mirror of Rust's
// `every_produced_event_has_a_registered_definition`.

// The player keys events off analysis-graph *stream* names. Those mostly equal
// the registry's definition ids; these few legitimately differ.
const DEFINITION_ID_TO_STREAM: Record<string, string> = {
  backboard_bounce: "backboard",
  core_player_scoreboard: "core_player",
};

// Native scoreboard stats are surfaced collectively by the "core" event source
// (see coreEventDerivation.ts), not as individual stream toggles.
const CORE_AGGREGATED = new Set(["assist", "goal", "save", "shot"]);

// Registry entries that are intentionally not surfaced as their own player
// timeline stream.
const NOT_SURFACED = new Set([
  "event", // the generic timeline envelope, not a concrete event type
  // Expected-goals events remain hidden until the threat-curve timeline UI is
  // implemented; the Rust registry still documents them in the meantime.
  "threat_episode",
  "threat_touch",
]);

test("every registry event is represented in the player (or explicitly excepted)", () => {
  const viewerStreams = new Set<string>([
    ...STATS_EVENT_STREAM_COUNT_TYPES,
    ...STATS_MECHANIC_EVENT_COUNT_TYPES,
  ]);

  const missing = EVENT_DEFINITION_CATALOG.filter((entry) => {
    if (CORE_AGGREGATED.has(entry.key) || NOT_SURFACED.has(entry.key)) {
      return false;
    }
    const streamId = DEFINITION_ID_TO_STREAM[entry.key] ?? entry.key;
    return !viewerStreams.has(streamId);
  }).map((entry) => entry.key);

  assert.deepEqual(
    missing,
    [],
    `Registry events missing from the player: ${missing.join(", ")}. Add the stream id to ` +
      `STATS_EVENT_STREAM_COUNT_TYPES / STATS_MECHANIC_EVENT_COUNT_TYPES in eventCountDerivation.ts, ` +
      `or extend the alias / exception sets in this test.`,
  );
});

test("event catalog parity alias and exception keys still exist in the registry", () => {
  const catalogKeys = new Set(EVENT_DEFINITION_CATALOG.map((entry) => entry.key));
  for (const key of [
    ...Object.keys(DEFINITION_ID_TO_STREAM),
    ...CORE_AGGREGATED,
    ...NOT_SURFACED,
  ]) {
    assert.ok(
      catalogKeys.has(key),
      `Stale parity exception "${key}" is no longer in the registry catalog; remove it from this test.`,
    );
  }
});
