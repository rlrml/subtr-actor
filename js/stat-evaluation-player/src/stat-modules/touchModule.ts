import { renderTouchStats } from "../touchFormatting.ts";
import type { TouchBreakdownClass } from "../touchFormatting.ts";
import {
  isTouchOverlayColorMode,
  normalizeTouchOverlayColorModes,
  TouchEventOverlay,
} from "../touchOverlay.ts";
import type { TouchOverlayColorMode, TouchOverlayMode } from "../touchOverlay.ts";
import { TOUCH_COLOR_MODES_CHANGE_EVENT } from "../touchColorLegend.ts";
import { buildTouchTimelineEvents } from "../timelineMarkers.ts";
import { getStatsFrameForReplayFrame } from "../statsTimeline.ts";
import {
  getStatsPlayerSnapshot,
  renderGroupedPlayerCards,
  renderPlayerCard,
  type StatModule,
  type StatModuleRuntime,
} from "./types.ts";

export function createTouchModule(runtime: StatModuleRuntime): StatModule {
  let overlay: TouchEventOverlay | null = null;
  let decaySeconds = 5;
  let overlayMode: TouchOverlayMode = "advancement";
  let overlayColorModes: TouchOverlayColorMode[] = ["team"];
  let settingsEl: HTMLDivElement | null = null;
  let decayReadoutEl: HTMLElement | null = null;
  let overlayModeReadoutEl: HTMLElement | null = null;
  let breakdownReadoutEl: HTMLElement | null = null;
  let legendColorModeListener: ((event: Event) => void) | null = null;
  const activeBreakdownClasses = new Set<TouchBreakdownClass>();
  const orderedBreakdownClasses: TouchBreakdownClass[] = [
    "kind",
    "height_band",
    "surface",
    "dodge_state",
  ];

  return {
    id: "touch",
    label: "Touch",

    setup(ctx) {
      overlay = new TouchEventOverlay(
        ctx.player.sceneState,
        ctx.player.container,
        ctx.replay,
        ctx.statsTimeline,
        {
          mode: overlayMode,
          colorModes: overlayColorModes,
        },
      );
      overlay.setDecaySeconds(decaySeconds);
      legendColorModeListener = (event: Event) => {
        if (!(event instanceof CustomEvent)) {
          return;
        }
        const colorModes = (event.detail as { colorModes?: unknown } | null)?.colorModes;
        if (Array.isArray(colorModes)) {
          setOverlayColorModes(colorModes.filter(isTouchOverlayColorMode));
        }
      };
      window.addEventListener(TOUCH_COLOR_MODES_CHANGE_EVENT, legendColorModeListener);
      syncTouchSettingsUi();
    },

    teardown() {
      if (legendColorModeListener) {
        window.removeEventListener(TOUCH_COLOR_MODES_CHANGE_EVENT, legendColorModeListener);
        legendColorModeListener = null;
      }
      overlay?.dispose();
      overlay = null;
    },

    onBeforeRender(info) {
      overlay?.update(info.currentTime);
    },

    getTimelineEvents(ctx) {
      return buildTouchTimelineEvents(ctx.statsTimeline, ctx.replay);
    },

    getConfig() {
      return {
        decaySeconds,
        overlayMode,
        overlayColorModes,
        breakdownClasses: getActiveBreakdownClasses(),
      };
    },

    applyConfig(config) {
      if (config && typeof config === "object" && !Array.isArray(config)) {
        const record = config as Record<string, unknown>;
        if (typeof record.decaySeconds === "number" && Number.isFinite(record.decaySeconds)) {
          decaySeconds = Math.max(1, Math.min(10, record.decaySeconds));
          overlay?.setDecaySeconds(decaySeconds);
        }
        if (record.overlayMode === "markers" || record.overlayMode === "advancement") {
          overlayMode = record.overlayMode;
          overlay?.setMode(overlayMode);
        }
        if (Array.isArray(record.overlayColorModes)) {
          overlayColorModes = normalizeTouchOverlayColorModes(
            record.overlayColorModes.filter(isTouchOverlayColorMode),
          );
          overlay?.setColorModes(overlayColorModes);
        } else if (isTouchOverlayColorMode(record.overlayColorMode)) {
          overlayColorModes = normalizeTouchOverlayColorModes(record.overlayColorMode);
          overlay?.setColorModes(overlayColorModes);
        }
        activeBreakdownClasses.clear();
        if (Array.isArray(record.breakdownClasses)) {
          for (const className of record.breakdownClasses) {
            if (orderedBreakdownClasses.includes(className as TouchBreakdownClass)) {
              activeBreakdownClasses.add(className as TouchBreakdownClass);
            }
          }
        }
      }
      syncTouchSettingsUi();
      runtime.rerenderCurrentState();
    },

    renderStats(frameIndex, ctx) {
      const statsFrame = getStatsFrameForReplayFrame(ctx.statsFrameLookup, frameIndex);
      if (!statsFrame) return "";

      return renderGroupedPlayerCards(statsFrame.players, (player) =>
        renderPlayerCard(
          player.name,
          player.is_team_0,
          renderTouchStats(player.touch, {
            breakdownClasses: getActiveBreakdownClasses(),
          }),
          player.touch?.is_last_touch
            ? '<span class="role-indicator role-forward">Last Touch</span>'
            : "",
        ),
      );
    },

    renderFocusedPlayerStats(playerId, frameIndex, ctx) {
      const player = getStatsPlayerSnapshot(ctx, frameIndex, playerId);
      if (!player) return "";

      return renderTouchStats(player.touch, {
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
        eyebrow.textContent = "Touch markers";
        const title = document.createElement("h3");
        title.textContent = "Touch decay";
        text.append(eyebrow, title);

        decayReadoutEl = document.createElement("strong");
        decayReadoutEl.className = "metric-readout";
        header.append(text, decayReadoutEl);

        const label = document.createElement("label");
        const labelText = document.createElement("span");
        labelText.className = "label";
        labelText.textContent = "Keep each marker visible after the touch";

        const input = document.createElement("input");
        input.type = "range";
        input.min = "1";
        input.max = "10";
        input.step = "0.5";
        input.value = `${decaySeconds}`;
        input.addEventListener("input", () => {
          const nextValue = Number(input.value);
          decaySeconds = Number.isFinite(nextValue)
            ? Math.max(1, Math.min(10, nextValue))
            : decaySeconds;
          overlay?.setDecaySeconds(decaySeconds);
          syncTouchSettingsUi(decaySeconds);
          runtime.requestConfigSync?.();
        });

        label.append(labelText, input);

        const modeSection = document.createElement("div");
        modeSection.className = "module-settings-subgroup";

        const modeHeader = document.createElement("div");
        modeHeader.className = "module-settings-header";

        const modeText = document.createElement("div");
        const modeEyebrow = document.createElement("p");
        modeEyebrow.className = "module-settings-eyebrow";
        modeEyebrow.textContent = "Overlay";
        const modeTitle = document.createElement("h3");
        modeTitle.textContent = "Touch mode";
        modeText.append(modeEyebrow, modeTitle);

        overlayModeReadoutEl = document.createElement("strong");
        overlayModeReadoutEl.className = "metric-readout";
        modeHeader.append(modeText, overlayModeReadoutEl);

        const modeOptions = document.createElement("div");
        modeOptions.className = "module-settings-options";
        for (const option of [
          { mode: "markers", label: "Markers" },
          { mode: "advancement", label: "Advancement" },
        ] satisfies Array<{ mode: TouchOverlayMode; label: string }>) {
          const optionLabel = document.createElement("label");
          optionLabel.className = "toggle";

          const radio = document.createElement("input");
          radio.type = "radio";
          radio.name = "touch-overlay-mode";
          radio.dataset.overlayMode = option.mode;
          radio.addEventListener("change", () => {
            if (!radio.checked) {
              return;
            }
            overlayMode = option.mode;
            overlay?.setMode(overlayMode);
            syncTouchSettingsUi();
            runtime.requestConfigSync?.();
          });

          const optionText = document.createElement("span");
          optionText.textContent = option.label;
          optionLabel.append(radio, optionText);
          modeOptions.append(optionLabel);
        }
        modeSection.append(modeHeader, modeOptions);

        const breakdownSection = document.createElement("div");
        breakdownSection.className = "module-settings-subgroup";

        const breakdownHeader = document.createElement("div");
        breakdownHeader.className = "module-settings-header";

        const breakdownText = document.createElement("div");
        const breakdownEyebrow = document.createElement("p");
        breakdownEyebrow.className = "module-settings-eyebrow";
        breakdownEyebrow.textContent = "Stat display";
        const breakdownTitle = document.createElement("h3");
        breakdownTitle.textContent = "Touch breakdown";
        breakdownText.append(breakdownEyebrow, breakdownTitle);

        breakdownReadoutEl = document.createElement("strong");
        breakdownReadoutEl.className = "metric-readout";
        breakdownHeader.append(breakdownText, breakdownReadoutEl);

        const breakdownOptions = document.createElement("div");
        breakdownOptions.className = "module-settings-options";

        for (const option of [
          { className: "kind", label: "Kind" },
          { className: "height_band", label: "Height" },
          { className: "surface", label: "Surface" },
          { className: "dodge_state", label: "Dodge" },
        ] satisfies Array<{ className: TouchBreakdownClass; label: string }>) {
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
            syncTouchSettingsUi();
            runtime.rerenderCurrentState();
            runtime.requestConfigSync?.();
          });

          const optionText = document.createElement("span");
          optionText.textContent = option.label;
          optionLabel.append(checkbox, optionText);
          breakdownOptions.append(optionLabel);
        }

        breakdownSection.append(breakdownHeader, breakdownOptions);
        settingsEl.append(header, label, modeSection, breakdownSection);
      }

      syncTouchSettingsUi();
      return settingsEl;
    },
  };

  function syncTouchSettingsUi(nextValue?: number): void {
    if (!settingsEl) {
      return;
    }

    const value = nextValue ?? decaySeconds;
    const input = settingsEl.querySelector("input");
    if (input instanceof HTMLInputElement) {
      input.value = `${value}`;
    }
    if (decayReadoutEl) {
      decayReadoutEl.textContent = `${value.toFixed(1)}s`;
    }
    for (const radio of settingsEl.querySelectorAll<HTMLInputElement>("input[data-overlay-mode]")) {
      radio.checked = radio.dataset.overlayMode === overlayMode;
    }
    if (overlayModeReadoutEl) {
      overlayModeReadoutEl.textContent = overlayMode === "advancement" ? "Advancement" : "Markers";
    }
    for (const checkbox of settingsEl.querySelectorAll<HTMLInputElement>(
      "input[data-breakdown-class]",
    )) {
      const className = checkbox.dataset.breakdownClass as TouchBreakdownClass | undefined;
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
                    kind: "Kind",
                    height_band: "Height",
                    surface: "Surface",
                    dodge_state: "Dodge",
                  })[className],
              )
              .join(" + ")
          : "Total only";
    }
  }

  function getActiveBreakdownClasses(): TouchBreakdownClass[] {
    return orderedBreakdownClasses.filter((className) => activeBreakdownClasses.has(className));
  }

  function setOverlayColorModes(colorModes: readonly TouchOverlayColorMode[]): void {
    overlayColorModes = normalizeTouchOverlayColorModes(colorModes);
    overlay?.setColorModes(overlayColorModes);
    syncTouchSettingsUi();
    runtime.requestConfigSync?.();
  }
}
