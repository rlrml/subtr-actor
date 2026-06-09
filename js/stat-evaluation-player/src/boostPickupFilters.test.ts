import assert from "node:assert/strict";
import test from "node:test";
import { createBoostPickupFilterController } from "./boostPickupFilters.ts";

const STATS_TIMELINE = {
  events: {
    boost_pickups: [],
  },
};

function replay(id: string) {
  return {
    frameCount: 0,
    duration: 0,
    frames: [],
    ballFrames: [],
    boostPads: [],
    players: [{ id, name: id, isTeamZero: true }],
    timelineEvents: [],
    teamZeroNames: [],
    teamOneNames: [],
  };
}

test("boost pickup player filters restored from config survive initial replay setup", () => {
  const controller = createBoostPickupFilterController();
  controller.applyConfig({
    playerIds: ["Steam:player-1"],
    padTypes: ["big"],
    detections: ["both"],
    activities: ["active"],
    fieldHalves: ["own"],
  });

  controller.setup({
    replay: replay("Steam:player-1") as never,
    statsTimeline: STATS_TIMELINE as never,
  });

  assert.deepEqual(controller.getConfig().playerIds, ["Steam:player-1"]);
  assert.deepEqual([...(controller.getTimelineRangeOptions().playerIds ?? [])], ["Steam:player-1"]);
});

test("boost pickup player filters still reset across later replay changes", () => {
  const controller = createBoostPickupFilterController();
  controller.setup({
    replay: replay("Steam:first") as never,
    statsTimeline: STATS_TIMELINE as never,
  });
  controller.applyConfig({
    playerIds: ["Steam:first"],
    padTypes: ["big"],
    detections: ["both"],
    activities: ["active"],
    fieldHalves: ["own"],
  });

  controller.setup({
    replay: replay("Steam:second") as never,
    statsTimeline: STATS_TIMELINE as never,
  });

  assert.equal(controller.getConfig().playerIds, null);
  assert.equal(controller.getTimelineRangeOptions().playerIds, undefined);
});
