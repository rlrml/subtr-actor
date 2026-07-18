import { buildExpectedGoalsTimelineGraphs } from "../expectedGoalsTimelineGraph.ts";
import type { StatModule } from "./types.ts";

export function createExpectedGoalsModule(): StatModule {
  return {
    id: "expected_goals",
    label: "Expected goals",

    setup() {},
    teardown() {},
    onBeforeRender() {},

    getTimelineGraphs(ctx) {
      return buildExpectedGoalsTimelineGraphs(ctx.statsTimeline);
    },

    renderStats() {
      return "";
    },

    renderFocusedPlayerStats() {
      return "";
    },
  };
}
