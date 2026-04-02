import { HalfFieldOverlay } from "../overlays.ts";
import type { ReplayModel } from "subtr-actor-player";
import { renderPlayerFiftyFiftyStats, renderFiftyFiftySummary } from "../fiftyFiftyFormatting.ts";
import { renderPossessionStats } from "../possessionFormatting.ts";
import type { PossessionBreakdownClass } from "../possessionFormatting.ts";
import { renderPressureStats } from "../pressureFormatting.ts";
import { renderRushStats } from "../rushFormatting.ts";
import { FiftyFiftyOverlay } from "../fiftyFiftyOverlay.ts";
import { buildFiftyFiftyTimelineEvents, buildRushTimelineEvents } from "../timelineMarkers.ts";
import {
  buildPossessionTimelineRanges,
  buildPressureTimelineRanges,
  buildRushTimelineRanges,
} from "../timelineRanges.ts";
import { getStatsFrameForReplayFrame } from "../statsTimeline.ts";
import {
  getStatsPlayerSnapshot,
  renderPlayerCard,
  renderSharedCard,
  type StatModule,
  type StatModuleRuntime,
} from "./types.ts";

export function createPossessionModule(runtime: StatModuleRuntime): StatModule {
  let settingsEl: HTMLDivElement | null = null;
  let breakdownReadoutEl: HTMLElement | null = null;
  const activeBreakdownClasses = new Set<PossessionBreakdownClass>();
  const orderedBreakdownClasses: PossessionBreakdownClass[] = [
    "possession_state",
    "field_third",
  ];

  return {
    id: "possession",
    label: "Possession",

    setup() {
      syncPossessionSettingsUi();
    },

    teardown() {},

    onBeforeRender() {},

    getTimelineRanges(ctx) {
      return buildPossessionTimelineRanges(ctx.statsTimeline, ctx.replay);
    },

    renderStats(frameIndex, ctx) {
      const statsFrame = getStatsFrameForReplayFrame(
        ctx.statsFrameLookup,
        frameIndex,
      );
      const possession = statsFrame?.team_zero?.possession;
      if (!possession) return "";

      return renderSharedCard(
        "Control State",
        renderPossessionStats(possession, {
          labelPerspective: {
            kind: "shared",
          },
          breakdownClasses: getActiveBreakdownClasses(),
        }),
      );
    },

    renderFocusedPlayerStats(playerId, frameIndex, ctx) {
      const statsFrame = getStatsFrameForReplayFrame(
        ctx.statsFrameLookup,
        frameIndex,
      );
      const player = getStatsPlayerSnapshot(ctx, frameIndex, playerId);
      const possession = player?.is_team_0
        ? statsFrame?.team_zero?.possession
        : statsFrame?.team_one?.possession;
      if (!possession || !player) return "";

      return renderPossessionStats(possession, {
        labelPerspective: {
          kind: "team",
        },
        breakdownClasses: getActiveBreakdownClasses(),
      });
    },

    renderSettings() {
      if (!settingsEl) {
        settingsEl = document.createElement("div");
        settingsEl.className = "module-settings-card";

        const header = document.createElement("div");
        header.className = "module-settings-header";

        const text = document.createElement("div");
        const eyebrow = document.createElement("p");
        eyebrow.className = "module-settings-eyebrow";
        eyebrow.textContent = "Stat display";
        const title = document.createElement("h3");
        title.textContent = "Possession breakdown";
        text.append(eyebrow, title);

        breakdownReadoutEl = document.createElement("strong");
        breakdownReadoutEl.className = "metric-readout";
        header.append(text, breakdownReadoutEl);

        const options = document.createElement("div");
        options.className = "module-settings-options";

        const optionLabel = document.createElement("label");
        optionLabel.className = "toggle";

        const checkbox = document.createElement("input");
        checkbox.type = "checkbox";
        checkbox.dataset.breakdownClass = "possession_state";
        checkbox.addEventListener("change", () => {
          if (checkbox.checked) {
            activeBreakdownClasses.add("possession_state");
          } else {
            activeBreakdownClasses.delete("possession_state");
          }
          syncPossessionSettingsUi();
          runtime.rerenderCurrentState();
        });

        const optionText = document.createElement("span");
        optionText.textContent = "Control";
        optionLabel.append(checkbox, optionText);
        options.append(optionLabel);

        const thirdOptionLabel = document.createElement("label");
        thirdOptionLabel.className = "toggle";

        const thirdCheckbox = document.createElement("input");
        thirdCheckbox.type = "checkbox";
        thirdCheckbox.dataset.breakdownClass = "field_third";
        thirdCheckbox.addEventListener("change", () => {
          if (thirdCheckbox.checked) {
            activeBreakdownClasses.add("field_third");
          } else {
            activeBreakdownClasses.delete("field_third");
          }
          syncPossessionSettingsUi();
          runtime.rerenderCurrentState();
        });

        const thirdOptionText = document.createElement("span");
        thirdOptionText.textContent = "Third";
        thirdOptionLabel.append(thirdCheckbox, thirdOptionText);
        options.append(thirdOptionLabel);

        settingsEl.append(header, options);
      }

      syncPossessionSettingsUi();
      return settingsEl;
    },
  };

  function syncPossessionSettingsUi(): void {
    if (!settingsEl) {
      return;
    }

    for (const checkbox of settingsEl.querySelectorAll<HTMLInputElement>(
      "input[data-breakdown-class]",
    )) {
      const className = checkbox.dataset
        .breakdownClass as PossessionBreakdownClass | undefined;
      checkbox.checked = className
        ? activeBreakdownClasses.has(className)
        : false;
    }

    if (breakdownReadoutEl) {
      const enabled = orderedBreakdownClasses.filter((className) =>
        activeBreakdownClasses.has(className)
      );
      breakdownReadoutEl.textContent = enabled.length === 0
        ? "Total only"
        : enabled.map((className) =>
          className === "possession_state" ? "Control" : "Third"
        ).join(" x ");
    }
  }

  function getActiveBreakdownClasses(): PossessionBreakdownClass[] {
    return orderedBreakdownClasses.filter((className) =>
      activeBreakdownClasses.has(className)
    );
  }
}

export function createFiftyFiftyModule(): StatModule {
  let overlay: FiftyFiftyOverlay | null = null;

  return {
    id: "fifty-fifty",
    label: "50/50",

    setup(ctx) {
      overlay = new FiftyFiftyOverlay(
        ctx.player.sceneState,
        ctx.player.container,
        ctx.replay,
        ctx.statsTimeline,
      );
    },

    teardown() {
      overlay?.dispose();
      overlay = null;
    },

    onBeforeRender(info) {
      overlay?.update(info.currentTime);
    },

    getTimelineEvents(ctx) {
      return buildFiftyFiftyTimelineEvents(ctx.statsTimeline, ctx.replay);
    },

    renderStats(frameIndex, ctx) {
      const statsFrame = getStatsFrameForReplayFrame(
        ctx.statsFrameLookup,
        frameIndex,
      );
      if (!statsFrame) return "";

      const summary = renderSharedCard(
        "Challenge Summary",
        renderFiftyFiftySummary(statsFrame.team_zero?.fifty_fifty, {
          kind: "shared",
        }),
      );

      const players = statsFrame.players.map((player) => renderPlayerCard(
        player.name,
        player.is_team_0,
        renderPlayerFiftyFiftyStats(player.fifty_fifty),
      )).join("");

      return summary + players;
    },

    renderFocusedPlayerStats(playerId, frameIndex, ctx) {
      const player = getStatsPlayerSnapshot(ctx, frameIndex, playerId);
      if (!player) return "";

      return renderPlayerFiftyFiftyStats(player.fifty_fifty);
    },
  };
}

export function createPressureModule(): StatModule {
  let halfFieldOverlay: HalfFieldOverlay | null = null;
  let replay: ReplayModel | null = null;

  return {
    id: "pressure",
    label: "Half Control",

    setup(ctx) {
      replay = ctx.replay;
      halfFieldOverlay = new HalfFieldOverlay(
        ctx.player.sceneState.scene,
        ctx.fieldScale,
      );
    },

    teardown() {
      halfFieldOverlay?.dispose();
      halfFieldOverlay = null;
      replay = null;
    },

    onBeforeRender(info) {
      const ballFrame = replay?.ballFrames[info.frameIndex];
      halfFieldOverlay?.update(ballFrame?.position?.y ?? null);
    },

    getTimelineRanges(ctx) {
      return buildPressureTimelineRanges(ctx.statsTimeline, ctx.replay);
    },

    renderStats(frameIndex, ctx) {
      const statsFrame = getStatsFrameForReplayFrame(
        ctx.statsFrameLookup,
        frameIndex,
      );
      const pressure = statsFrame?.team_zero?.pressure;
      if (!pressure) return "";

      return renderSharedCard(
        "Field State",
        renderPressureStats(pressure, {
          labelPerspective: {
            kind: "shared",
          },
        }),
      );
    },

    renderFocusedPlayerStats(playerId, frameIndex, ctx) {
      const statsFrame = getStatsFrameForReplayFrame(
        ctx.statsFrameLookup,
        frameIndex,
      );
      const player = getStatsPlayerSnapshot(ctx, frameIndex, playerId);
      const pressure = player?.is_team_0
        ? statsFrame?.team_zero?.pressure
        : statsFrame?.team_one?.pressure;
      if (!pressure || !player) return "";

      return renderPressureStats(pressure, {
        labelPerspective: {
          kind: "team",
        },
      });
    },
  };
}

export function createRushModule(): StatModule {
  return {
    id: "rush",
    label: "Rush",

    setup() {},

    teardown() {},

    onBeforeRender() {},

    getTimelineEvents(ctx) {
      return buildRushTimelineEvents(ctx.statsTimeline, ctx.replay);
    },

    getTimelineRanges(ctx) {
      return buildRushTimelineRanges(ctx.statsTimeline, ctx.replay);
    },

    renderStats(frameIndex, ctx) {
      const statsFrame = getStatsFrameForReplayFrame(
        ctx.statsFrameLookup,
        frameIndex,
      );
      const teamZeroRush = statsFrame?.team_zero?.rush;
      const teamOneRush = statsFrame?.team_one?.rush;
      if (!teamZeroRush || !teamOneRush) return "";

      return [
        renderPlayerCard(
          "Blue Team",
          true,
          renderRushStats(teamZeroRush),
        ),
        renderPlayerCard(
          "Orange Team",
          false,
          renderRushStats(teamOneRush),
        ),
      ].join("");
    },

    renderFocusedPlayerStats(playerId, frameIndex, ctx) {
      const statsFrame = getStatsFrameForReplayFrame(
        ctx.statsFrameLookup,
        frameIndex,
      );
      const player = getStatsPlayerSnapshot(ctx, frameIndex, playerId);
      const rush = player?.is_team_0
        ? statsFrame?.team_zero?.rush
        : statsFrame?.team_one?.rush;
      if (!rush || !player) return "";

      return renderRushStats(rush);
    },
  };
}
