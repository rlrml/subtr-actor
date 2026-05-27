import { renderMovementStats } from "../movementFormatting.ts";
import type { MovementBreakdownClass } from "../movementFormatting.ts";
import { getStatsFrameForReplayFrame } from "../statsTimeline.ts";
import {
  getStatsPlayerSnapshot,
  renderGroupedPlayerCards,
  renderPlayerCard,
  type StatModule,
  type StatModuleRuntime,
} from "./types.ts";

export function createMovementModule(runtime: StatModuleRuntime): StatModule {
  let settingsEl: HTMLDivElement | null = null;
  let breakdownReadoutEl: HTMLElement | null = null;
  const activeBreakdownClasses = new Set<MovementBreakdownClass>();
  const orderedBreakdownClasses: MovementBreakdownClass[] = ["speed_band", "height_band"];

  return {
    id: "movement",
    label: "Movement",

    setup() {
      syncMovementSettingsUi();
    },

    teardown() {},

    onBeforeRender() {},

    getConfig() {
      return {
        breakdownClasses: getActiveBreakdownClasses(),
      };
    },

    applyConfig(config) {
      activeBreakdownClasses.clear();
      if (config && typeof config === "object" && !Array.isArray(config)) {
        const classes = (config as Record<string, unknown>).breakdownClasses;
        if (Array.isArray(classes)) {
          for (const className of classes) {
            if (orderedBreakdownClasses.includes(className as MovementBreakdownClass)) {
              activeBreakdownClasses.add(className as MovementBreakdownClass);
            }
          }
        }
      }
      syncMovementSettingsUi();
      runtime.rerenderCurrentState();
    },

    renderStats(frameIndex, ctx) {
      const statsFrame = getStatsFrameForReplayFrame(ctx.statsFrameLookup, frameIndex);
      if (!statsFrame) return "";

      return renderGroupedPlayerCards(statsFrame.players, (player) =>
        renderPlayerCard(
          player.name,
          player.is_team_0,
          renderMovementStats(player.movement, {
            breakdownClasses: getActiveBreakdownClasses(),
          }),
        ),
      );
    },

    renderFocusedPlayerStats(playerId, frameIndex, ctx) {
      const player = getStatsPlayerSnapshot(ctx, frameIndex, playerId);
      if (!player) return "";

      return renderMovementStats(player.movement, {
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
        title.textContent = "Movement breakdown";
        text.append(eyebrow, title);

        breakdownReadoutEl = document.createElement("strong");
        breakdownReadoutEl.className = "metric-readout";
        header.append(text, breakdownReadoutEl);

        const options = document.createElement("div");
        options.className = "module-settings-options";

        for (const option of [
          { className: "speed_band", label: "Speed band" },
          { className: "height_band", label: "Height band" },
        ] satisfies Array<{
          className: MovementBreakdownClass;
          label: string;
        }>) {
          const optionLabel = document.createElement("label");
          optionLabel.className = "toggle";

          const checkbox = document.createElement("input");
          checkbox.type = "checkbox";
          checkbox.dataset.breakdownClass = option.className;
          checkbox.addEventListener("change", () => {
            if (checkbox.checked) {
              activeBreakdownClasses.add(option.className);
            } else {
              activeBreakdownClasses.delete(option.className);
            }
            syncMovementSettingsUi();
            runtime.rerenderCurrentState();
            runtime.requestConfigSync?.();
          });

          const optionText = document.createElement("span");
          optionText.textContent = option.label;
          optionLabel.append(checkbox, optionText);
          options.append(optionLabel);
        }

        settingsEl.append(header, options);
      }

      syncMovementSettingsUi();
      return settingsEl;
    },
  };

  function syncMovementSettingsUi(): void {
    if (!settingsEl) {
      return;
    }

    for (const checkbox of settingsEl.querySelectorAll<HTMLInputElement>(
      "input[data-breakdown-class]",
    )) {
      const className = checkbox.dataset.breakdownClass as MovementBreakdownClass | undefined;
      checkbox.checked = className ? activeBreakdownClasses.has(className) : false;
    }

    if (breakdownReadoutEl) {
      const active = getActiveBreakdownClasses();
      breakdownReadoutEl.textContent =
        active.length > 0
          ? active
              .map(
                (className) =>
                  ({
                    speed_band: "Speed band",
                    height_band: "Height band",
                  })[className],
              )
              .join(" + ")
          : "Total only";
    }
  }

  function getActiveBreakdownClasses(): MovementBreakdownClass[] {
    return orderedBreakdownClasses.filter((className) => activeBreakdownClasses.has(className));
  }
}
