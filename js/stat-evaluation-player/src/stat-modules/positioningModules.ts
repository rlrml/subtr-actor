import { ThresholdZoneOverlay } from "../overlays.ts";
import { getStatsFrameForReplayFrame } from "../statsTimeline.ts";
import { buildTimeInZoneTimelineRanges } from "../timelineRanges.ts";
import { playerIdToString } from "../touchOverlay.ts";
import {
  renderAbsolutePositioningStats,
  renderRelativePositioningStats,
  zoneBoundaryOverlayManager,
} from "./renderers.ts";
import {
  getCurrentRole,
  getStatsPlayerSnapshot,
  RELATIVE_POSITIONING_MODULE_ID,
  renderPlayerCard,
  ROLE_LABELS,
  type StatModule,
} from "./types.ts";

export function createRelativePositioningModule(): StatModule {
  let thresholdZoneOverlay: ThresholdZoneOverlay | null = null;
  let fieldScale = 1;

  return {
    id: RELATIVE_POSITIONING_MODULE_ID,
    label: "Relative Positioning",

    setup(ctx) {
      fieldScale = ctx.fieldScale;
      thresholdZoneOverlay = new ThresholdZoneOverlay(
        ctx.player.sceneState.scene,
        ctx.replay,
        fieldScale,
      );
    },

    teardown() {
      thresholdZoneOverlay?.dispose();
      thresholdZoneOverlay = null;
    },

    onBeforeRender(info) {
      thresholdZoneOverlay?.update(info, fieldScale);
    },

    renderStats(frameIndex, ctx) {
      const statsFrame = getStatsFrameForReplayFrame(
        ctx.statsFrameLookup,
        frameIndex,
      );
      if (!statsFrame) return "";

      return statsFrame.players.map((player) => {
        const role = getCurrentRole(
          ctx.replay,
          playerIdToString(player.player_id),
          frameIndex,
        );
        return renderPlayerCard(
          player.name,
          player.is_team_0,
          renderRelativePositioningStats(player.positioning),
          `<span class="role-indicator role-${role}">${ROLE_LABELS[role]}</span>`,
        );
      }).join("");
    },

    renderFocusedPlayerStats(playerId, frameIndex, ctx) {
      const player = getStatsPlayerSnapshot(ctx, frameIndex, playerId);
      if (!player) return "";

      return renderRelativePositioningStats(player.positioning);
    },
  };
}

export function createAbsolutePositioningModule(): StatModule {
  return {
    id: "absolute-positioning",
    label: "Absolute Positioning",

    setup(ctx) {
      zoneBoundaryOverlayManager.acquire(ctx);
    },

    teardown() {
      zoneBoundaryOverlayManager.release();
    },

    onBeforeRender() {},

    renderStats(frameIndex, ctx) {
      const statsFrame = getStatsFrameForReplayFrame(
        ctx.statsFrameLookup,
        frameIndex,
      );
      if (!statsFrame) return "";

      return statsFrame.players.map((player) => renderPlayerCard(
        player.name,
        player.is_team_0,
        renderAbsolutePositioningStats(player.positioning),
      )).join("");
    },

    renderFocusedPlayerStats(playerId, frameIndex, ctx) {
      const player = getStatsPlayerSnapshot(ctx, frameIndex, playerId);
      if (!player) return "";

      return renderAbsolutePositioningStats(player.positioning);
    },
  };
}

export function createTimeInZoneModule(): StatModule {
  return {
    id: "time-in-zone",
    label: "Time In Zone",

    setup(ctx) {
      zoneBoundaryOverlayManager.acquire(ctx);
    },

    teardown() {
      zoneBoundaryOverlayManager.release();
    },

    onBeforeRender() {},

    getTimelineRanges(ctx) {
      return buildTimeInZoneTimelineRanges(ctx.statsTimeline, ctx.replay);
    },

    renderStats(frameIndex, ctx) {
      const statsFrame = getStatsFrameForReplayFrame(
        ctx.statsFrameLookup,
        frameIndex,
      );
      if (!statsFrame) return "";

      return statsFrame.players.map((player) => renderPlayerCard(
        player.name,
        player.is_team_0,
        renderAbsolutePositioningStats(player.positioning),
      )).join("");
    },

    renderFocusedPlayerStats(playerId, frameIndex, ctx) {
      const player = getStatsPlayerSnapshot(ctx, frameIndex, playerId);
      if (!player) return "";

      return renderAbsolutePositioningStats(player.positioning);
    },
  };
}
