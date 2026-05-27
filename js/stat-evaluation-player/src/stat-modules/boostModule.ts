import { createBoostPickupFilterController } from "../boostPickupFilters.ts";
import type { BoostPickupFilterController } from "../boostPickupFilters.ts";
import { buildBoostPickupTimelineRanges } from "../timelineRanges.ts";
import { getStatsFrameForReplayFrame } from "../statsTimeline.ts";
import { renderBoostStats } from "./renderers.ts";
import {
  getStatsPlayerSnapshot,
  renderGroupedPlayerCards,
  renderPlayerCard,
  type StatModule,
  type StatModuleRuntime,
} from "./types.ts";

export function createBoostModule(
  runtime: StatModuleRuntime,
  pickupFilters: BoostPickupFilterController = createBoostPickupFilterController({
    refreshTimelineRanges: runtime.refreshTimelineRanges,
    rerenderCurrentState: runtime.rerenderCurrentState,
  }),
): StatModule {
  return {
    id: "boost",
    label: "Boost",

    setup(ctx) {
      pickupFilters.setup(ctx);
    },

    teardown() {
      pickupFilters.teardown();
    },

    onBeforeRender() {},

    getTimelineRanges(ctx) {
      return buildBoostPickupTimelineRanges(
        ctx.statsTimeline,
        ctx.replay,
        pickupFilters.getTimelineRangeOptions(),
      );
    },

    getConfig() {
      return pickupFilters.getConfig();
    },

    applyConfig(config) {
      pickupFilters.applyConfig(config);
    },

    includeBoostPickupAnimationPickup(pickup) {
      return pickupFilters.includePickup(pickup);
    },

    renderStats(frameIndex, ctx) {
      const statsFrame = getStatsFrameForReplayFrame(ctx.statsFrameLookup, frameIndex);
      if (!statsFrame) return "";

      return renderGroupedPlayerCards(statsFrame.players, (player) =>
        renderPlayerCard(player.name, player.is_team_0, renderBoostStats(player.boost)),
      );
    },

    renderFocusedPlayerStats(playerId, frameIndex, ctx) {
      const player = getStatsPlayerSnapshot(ctx, frameIndex, playerId);
      if (!player) return "";

      return renderBoostStats(player.boost);
    },

    renderSettings(ctx) {
      return pickupFilters.renderSettings(ctx, {
        showHeader: true,
      });
    },
  };
}
