import { getStatsFrameForReplayFrame } from "../statsTimeline.ts";
import type { PlayerStatsSnapshot } from "../statsTimeline.ts";
import {
  getStatsPlayerSnapshot,
  renderPlayerCard,
  type StatModule,
} from "./types.ts";

export function createPlayerStatsModule<T>(options: {
  id: string;
  label: string;
  select: (player: PlayerStatsSnapshot) => T | undefined;
  render: (stats: T | undefined, player: PlayerStatsSnapshot) => string;
}): StatModule {
  return {
    id: options.id,
    label: options.label,

    setup() {},

    teardown() {},

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
        options.render(options.select(player), player),
      )).join("");
    },

    renderFocusedPlayerStats(playerId, frameIndex, ctx) {
      const player = getStatsPlayerSnapshot(ctx, frameIndex, playerId);
      if (!player) return "";

      return options.render(options.select(player), player);
    },
  };
}
