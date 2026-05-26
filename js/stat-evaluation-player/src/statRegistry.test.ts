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

test("stat registry hides live playback bookkeeping stats", () => {
  const definitions = createStatRegistry(null);
  const ids = new Set(definitions.map((definition) => definition.id));

  assert.ok(ids.has("player:backboard.count"));
  assert.ok(!ids.has("player:backboard.frames_since_last_backboard"));
  assert.ok(!ids.has("player:backboard.is_last_backboard"));
  assert.ok(!ids.has("player:backboard.last_backboard_frame"));
  assert.ok(!ids.has("player:backboard.last_backboard_time"));
  assert.ok(!ids.has("player:backboard.time_since_last_backboard"));
  assert.ok(!ids.has("player:touch.is_last_touch"));
  assert.ok(!ids.has("player:pass.time_since_last_completed_pass"));
  assert.ok(ids.has("player:core.last_goal_ball_air_time"));
});
