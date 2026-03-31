import { renderMovementStats } from "../movementFormatting.ts";
import type { MovementBreakdownClass } from "../movementFormatting.ts";
import { renderTouchStats } from "../touchFormatting.ts";
import type { TouchBreakdownClass } from "../touchFormatting.ts";
import { CeilingShotOverlay } from "../ceilingShotOverlay.ts";
import { TouchEventOverlay } from "../touchOverlay.ts";
import { SpeedFlipOverlay } from "../speedFlipOverlay.ts";
import {
  buildBackboardTimelineEvents,
  buildBallCarryTimelineEvents,
  buildCeilingShotTimelineEvents,
  buildDodgeResetTimelineEvents,
  buildDoubleTapTimelineEvents,
  buildMustyFlickTimelineEvents,
  buildPowerslideTimelineEvents,
  buildSpeedFlipTimelineEvents,
  buildTouchTimelineEvents,
} from "../timelineMarkers.ts";
import { getStatsFrameForReplayFrame } from "../statsTimeline.ts";
import { createPlayerStatsModule } from "./playerStatsModule.ts";
import {
  renderBackboardStats,
  renderBallCarryStats,
  renderBoostStats,
  renderCeilingShotStats,
  renderCoreStats,
  renderDemoStats,
  renderDodgeResetStats,
  renderDoubleTapStats,
  renderMustyFlickStats,
  renderPowerslideStats,
  renderSpeedFlipStats,
} from "./renderers.ts";
import {
  getStatsPlayerSnapshot,
  renderPlayerCard,
  type StatModule,
  type StatModuleRuntime,
} from "./types.ts";

export function createBoostModule(): StatModule {
  return {
    id: "boost",
    label: "Boost",

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
        renderBoostStats(player.boost),
      )).join("");
    },

    renderFocusedPlayerStats(playerId, frameIndex, ctx) {
      const player = getStatsPlayerSnapshot(ctx, frameIndex, playerId);
      if (!player) return "";

      return renderBoostStats(player.boost);
    },
  };
}

export function createCoreModule(): StatModule {
  return createPlayerStatsModule({
    id: "core",
    label: "Core",
    select: (player) => player.core,
    render: (core) => renderCoreStats(core),
  });
}

export function createBackboardModule(): StatModule {
  return createPlayerStatsModule({
    id: "backboard",
    label: "Backboard",
    select: (player) => player.backboard,
    render: (backboard) => renderBackboardStats(backboard),
    getTimelineEvents(ctx) {
      return buildBackboardTimelineEvents(ctx.statsTimeline, ctx.replay);
    },
  });
}

export function createCeilingShotModule(): StatModule {
  let overlay: CeilingShotOverlay | null = null;

  return {
    id: "ceiling-shot",
    label: "Ceiling Shot",

    setup(ctx) {
      overlay = new CeilingShotOverlay(
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
      return buildCeilingShotTimelineEvents(ctx.statsTimeline, ctx.replay);
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
        renderCeilingShotStats(player.ceiling_shot),
        player.ceiling_shot?.is_last_ceiling_shot
          ? '<span class="role-indicator role-forward">Last Ceiling Shot</span>'
          : "",
      )).join("");
    },

    renderFocusedPlayerStats(playerId, frameIndex, ctx) {
      const player = getStatsPlayerSnapshot(ctx, frameIndex, playerId);
      if (!player) return "";

      return renderCeilingShotStats(player.ceiling_shot);
    },
  };
}

export function createBallCarryModule(): StatModule {
  return createPlayerStatsModule({
    id: "ball-carry",
    label: "Ball Carry",
    select: (player) => player.ball_carry,
    render: (ballCarry) => renderBallCarryStats(ballCarry),
    getTimelineEvents(ctx) {
      return buildBallCarryTimelineEvents(ctx.statsTimeline, ctx.replay);
    },
  });
}

export function createDodgeResetModule(): StatModule {
  return createPlayerStatsModule({
    id: "dodge-reset",
    label: "Dodge Reset",
    select: (player) => player.dodge_reset,
    render: (dodgeReset) => renderDodgeResetStats(dodgeReset),
    getTimelineEvents(ctx) {
      return buildDodgeResetTimelineEvents(ctx.statsTimeline, ctx.replay);
    },
  });
}

export function createDoubleTapModule(): StatModule {
  return createPlayerStatsModule({
    id: "double-tap",
    label: "Double Tap",
    select: (player) => player.double_tap,
    render: (doubleTap) => renderDoubleTapStats(doubleTap),
    getTimelineEvents(ctx) {
      return buildDoubleTapTimelineEvents(ctx.statsTimeline, ctx.replay);
    },
  });
}

export function createMustyFlickModule(): StatModule {
  return {
    id: "musty-flick",
    label: "Musty Flick",

    setup() {},

    teardown() {},

    onBeforeRender() {},

    getTimelineEvents(ctx) {
      return buildMustyFlickTimelineEvents(ctx.statsTimeline, ctx.replay);
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
        renderMustyFlickStats(player.musty_flick),
        player.musty_flick?.is_last_musty
          ? '<span class="role-indicator role-forward">Last Musty</span>'
          : "",
      )).join("");
    },

    renderFocusedPlayerStats(playerId, frameIndex, ctx) {
      const player = getStatsPlayerSnapshot(ctx, frameIndex, playerId);
      if (!player) return "";

      return renderMustyFlickStats(player.musty_flick);
    },
  };
}

export function createSpeedFlipModule(): StatModule {
  let overlay: SpeedFlipOverlay | null = null;

  return {
    id: "speed-flip",
    label: "Speed Flip",

    setup(ctx) {
      overlay = new SpeedFlipOverlay(
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
      return buildSpeedFlipTimelineEvents(ctx.statsTimeline, ctx.replay);
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
        renderSpeedFlipStats(player.speed_flip),
        player.speed_flip?.is_last_speed_flip
          ? '<span class="role-indicator role-forward">Last Speed Flip</span>'
          : "",
      )).join("");
    },

    renderFocusedPlayerStats(playerId, frameIndex, ctx) {
      const player = getStatsPlayerSnapshot(ctx, frameIndex, playerId);
      if (!player) return "";

      return renderSpeedFlipStats(player.speed_flip);
    },
  };
}

export function createTouchModule(runtime: StatModuleRuntime): StatModule {
  let overlay: TouchEventOverlay | null = null;
  let settingsEl: HTMLDivElement | null = null;
  let decayReadoutEl: HTMLElement | null = null;
  let breakdownReadoutEl: HTMLElement | null = null;
  const activeBreakdownClasses = new Set<TouchBreakdownClass>();
  const orderedBreakdownClasses: TouchBreakdownClass[] = [
    "kind",
    "height_band",
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
      );
      syncTouchSettingsUi();
    },

    teardown() {
      overlay?.dispose();
      overlay = null;
    },

    onBeforeRender(info) {
      overlay?.update(info.currentTime);
    },

    getTimelineEvents(ctx) {
      return buildTouchTimelineEvents(ctx.statsTimeline, ctx.replay);
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
        renderTouchStats(player.touch, {
          breakdownClasses: getActiveBreakdownClasses(),
        }),
        player.touch?.is_last_touch
          ? '<span class="role-indicator role-forward">Last Touch</span>'
          : "",
      )).join("");
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
        input.value = `${overlay?.getDecaySeconds() ?? 5}`;
        input.addEventListener("input", () => {
          const nextValue = Number(input.value);
          overlay?.setDecaySeconds(nextValue);
          syncTouchSettingsUi(nextValue);
        });

        label.append(labelText, input);
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
          });

          const optionText = document.createElement("span");
          optionText.textContent = option.label;
          optionLabel.append(checkbox, optionText);
          breakdownOptions.append(optionLabel);
        }

        breakdownSection.append(breakdownHeader, breakdownOptions);
        settingsEl.append(header, label, breakdownSection);
      }

      syncTouchSettingsUi();
      return settingsEl;
    },
  };

  function syncTouchSettingsUi(nextValue?: number): void {
    if (!settingsEl) {
      return;
    }

    const value = nextValue ?? overlay?.getDecaySeconds() ?? 5;
    const input = settingsEl.querySelector("input");
    if (input instanceof HTMLInputElement) {
      input.value = `${value}`;
    }
    if (decayReadoutEl) {
      decayReadoutEl.textContent = `${value.toFixed(1)}s`;
    }
    for (const checkbox of settingsEl.querySelectorAll<HTMLInputElement>(
      "input[data-breakdown-class]",
    )) {
      const className = checkbox.dataset
        .breakdownClass as TouchBreakdownClass | undefined;
      checkbox.checked = className
        ? activeBreakdownClasses.has(className)
        : false;
    }
    if (breakdownReadoutEl) {
      const active = getActiveBreakdownClasses();
      breakdownReadoutEl.textContent = active.length > 0
        ? active.map((className) => ({
          kind: "Kind",
          height_band: "Height",
        }[className])).join(" + ")
        : "Total only";
    }
  }

  function getActiveBreakdownClasses(): TouchBreakdownClass[] {
    return orderedBreakdownClasses.filter((className) =>
      activeBreakdownClasses.has(className)
    );
  }
}

export function createMovementModule(runtime: StatModuleRuntime): StatModule {
  let settingsEl: HTMLDivElement | null = null;
  let breakdownReadoutEl: HTMLElement | null = null;
  const activeBreakdownClasses = new Set<MovementBreakdownClass>();
  const orderedBreakdownClasses: MovementBreakdownClass[] = [
    "speed_band",
    "height_band",
  ];

  return {
    id: "movement",
    label: "Movement",

    setup() {
      syncMovementSettingsUi();
    },

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
        renderMovementStats(player.movement, {
          breakdownClasses: getActiveBreakdownClasses(),
        }),
      )).join("");
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
      const className = checkbox.dataset
        .breakdownClass as MovementBreakdownClass | undefined;
      checkbox.checked = className
        ? activeBreakdownClasses.has(className)
        : false;
    }

    if (breakdownReadoutEl) {
      const active = getActiveBreakdownClasses();
      breakdownReadoutEl.textContent = active.length > 0
        ? active.map((className) => ({
          speed_band: "Speed band",
          height_band: "Height band",
        }[className])).join(" + ")
        : "Total only";
    }
  }

  function getActiveBreakdownClasses(): MovementBreakdownClass[] {
    return orderedBreakdownClasses.filter((className) =>
      activeBreakdownClasses.has(className)
    );
  }
}

export function createPowerslideModule(): StatModule {
  return createPlayerStatsModule({
    id: "powerslide",
    label: "Powerslide",
    select: (player) => player.powerslide,
    render: (powerslide) => renderPowerslideStats(powerslide),
    getTimelineEvents(ctx) {
      return buildPowerslideTimelineEvents(ctx.statsTimeline, ctx.replay);
    },
  });
}

export function createDemoModule(): StatModule {
  return createPlayerStatsModule({
    id: "demo",
    label: "Demo",
    select: (player) => player.demo,
    render: (demo) => renderDemoStats(demo),
  });
}
