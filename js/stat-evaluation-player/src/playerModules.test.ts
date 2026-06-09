import assert from "node:assert/strict";
import test from "node:test";
import { hasBoostPickupAnimationTimelineMatch } from "./statModules.ts";

function pickup(frame: number, size: "big" | "small" = "small") {
  return {
    pad: { size },
    event: { frame },
    player: { id: "Steam:player-1" },
  };
}

function timeline(boostPickups: unknown[]) {
  return {
    events: {
      boost_pickups: boostPickups,
    },
  };
}

test("boost pickup animation fallback keeps legacy raw pickups without comparison events", () => {
  assert.equal(hasBoostPickupAnimationTimelineMatch(pickup(120), timeline([]) as never), true);
});

test("boost pickup animation fallback matches counted timeline pickups", () => {
  assert.equal(
    hasBoostPickupAnimationTimelineMatch(
      pickup(120),
      timeline([
        {
          detection: "both",
          frame: 120,
          player_id: { Steam: "player-1" },
          pad_type: "small",
        },
      ]) as never,
    ),
    true,
  );
});

test("boost pickup animation fallback rejects inferred-only timeline pickups", () => {
  assert.equal(
    hasBoostPickupAnimationTimelineMatch(
      pickup(120),
      timeline([
        {
          detection: "inferred_only",
          frame: 120,
          player_id: { Steam: "player-1" },
          pad_type: "small",
        },
      ]) as never,
    ),
    false,
  );
});

test("boost pickup animation fallback rejects unmatched raw pickup events", () => {
  assert.equal(
    hasBoostPickupAnimationTimelineMatch(
      pickup(122),
      timeline([
        {
          detection: "both",
          frame: 120,
          player_id: { Steam: "player-1" },
          pad_type: "small",
        },
      ]) as never,
    ),
    false,
  );
});
