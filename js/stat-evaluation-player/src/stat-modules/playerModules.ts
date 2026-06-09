import { renderMovementStats } from "../movementFormatting.ts";
import type { MovementBreakdownClass } from "../movementFormatting.ts";
import { CeilingShotOverlay } from "../ceilingShotOverlay.ts";
import { DodgeImpulseOverlay } from "../dodgeImpulseOverlay.ts";
import { SpeedFlipOverlay } from "../speedFlipOverlay.ts";
import { createBoostPickupFilterController } from "../boostPickupFilters.ts";
import type { BoostPickupFilterController } from "../boostPickupFilters.ts";
import {
  buildBackboardTimelineEvents,
  buildBumpTimelineEvents,
  buildDodgeTimelineEvents,
  buildPowerslideTimelineEvents,
  buildWavedashTimelineEvents,
  buildWhiffTimelineEvents,
} from "../timelineMarkers.ts";
import { buildBoostPickupTimelineRanges } from "../timelineRanges.ts";
import { getStatsFrameForReplayFrame } from "../statsTimeline.ts";
import { playerIdToString } from "../touchOverlay.ts";
import { createPlayerStatsModule } from "./playerStatsModule.ts";
import {
  renderBackboardStats,
  renderAirDribbleStats,
  renderBallCarryStats,
  renderBoostStats,
  renderBumpStats,
  renderCeilingShotStats,
  renderCoreStats,
  renderDemoStats,
  renderDodgeResetStats,
  renderDoubleTapStats,
  renderFlickStats,
  renderHalfFlipStats,
  renderMustyFlickStats,
  renderOneTimerStats,
  renderPassStats,
  renderPowerslideStats,
  renderRotationStats,
  renderSpeedFlipStats,
  renderWavedashStats,
  renderWallAerialStats,
  renderWallAerialShotStats,
  renderWhiffStats,
} from "./renderers.ts";
import {
  getStatsPlayerSnapshot,
  getTeamClass,
  renderGroupedPlayerCards,
  renderPlayerCard,
  type StatModule,
  type StatModuleRuntime,
} from "./types.ts";
export { createTouchModule } from "./touchModule.ts";

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

    renderStats(frameIndex, ctx) {
      const statsFrame = getStatsFrameForReplayFrame(ctx.statsFrameLookup, frameIndex);
      if (!statsFrame) return "";

      return renderGroupedPlayerCards(statsFrame.players, (player) =>
        renderPlayerCard(
          player.name,
          player.is_team_0,
          renderCeilingShotStats(player.ceiling_shot),
          player.ceiling_shot?.is_last_ceiling_shot
            ? '<span class="role-indicator role-forward">Last Ceiling Shot</span>'
            : "",
        ),
      );
    },

    renderFocusedPlayerStats(playerId, frameIndex, ctx) {
      const player = getStatsPlayerSnapshot(ctx, frameIndex, playerId);
      if (!player) return "";

      return renderCeilingShotStats(player.ceiling_shot);
    },
  };
}

export function createWallAerialModule(): StatModule {
  return {
    id: "wall-aerial",
    label: "Wall-to-Air Setup",

    setup() {},

    teardown() {},

    onBeforeRender() {},

    renderStats(frameIndex, ctx) {
      const statsFrame = getStatsFrameForReplayFrame(ctx.statsFrameLookup, frameIndex);
      if (!statsFrame) return "";

      return renderGroupedPlayerCards(statsFrame.players, (player) =>
        renderPlayerCard(
          player.name,
          player.is_team_0,
          renderWallAerialStats(player.wall_aerial),
          player.wall_aerial?.is_last_wall_aerial
            ? '<span class="role-indicator role-forward">Last Wall-to-Air Setup</span>'
            : "",
        ),
      );
    },

    renderFocusedPlayerStats(playerId, frameIndex, ctx) {
      const player = getStatsPlayerSnapshot(ctx, frameIndex, playerId);
      if (!player) return "";

      return renderWallAerialStats(player.wall_aerial);
    },
  };
}

export function createWallAerialShotModule(): StatModule {
  return {
    id: "wall-aerial-shot",
    label: "Wall Aerial Shot",

    setup() {},

    teardown() {},

    onBeforeRender() {},

    renderStats(frameIndex, ctx) {
      const statsFrame = getStatsFrameForReplayFrame(ctx.statsFrameLookup, frameIndex);
      if (!statsFrame) return "";

      return renderGroupedPlayerCards(statsFrame.players, (player) =>
        renderPlayerCard(
          player.name,
          player.is_team_0,
          renderWallAerialShotStats(player.wall_aerial_shot),
          player.wall_aerial_shot?.is_last_wall_aerial_shot
            ? '<span class="role-indicator role-forward">Last Wall Aerial Shot</span>'
            : "",
        ),
      );
    },

    renderFocusedPlayerStats(playerId, frameIndex, ctx) {
      const player = getStatsPlayerSnapshot(ctx, frameIndex, playerId);
      if (!player) return "";

      return renderWallAerialShotStats(player.wall_aerial_shot);
    },
  };
}

export function createBallCarryModule(): StatModule {
  return createPlayerStatsModule({
    id: "ball-carry",
    label: "Ball Carry",
    select: (player) => player.ball_carry,
    render: (ballCarry) => renderBallCarryStats(ballCarry),
  });
}

export function createAirDribbleModule(): StatModule {
  return createPlayerStatsModule({
    id: "air-dribble",
    label: "Air Dribble",
    select: (player) => player.air_dribble,
    render: (airDribble) => renderAirDribbleStats(airDribble),
  });
}

export function createDodgeResetModule(): StatModule {
  return createPlayerStatsModule({
    id: "dodge-reset",
    label: "Dodge Refresh",
    select: (player) => player.dodge_reset,
    render: (dodgeReset) => renderDodgeResetStats(dodgeReset),
  });
}

export function createDoubleTapModule(): StatModule {
  return createPlayerStatsModule({
    id: "double-tap",
    label: "Double Tap",
    select: (player) => player.double_tap,
    render: (doubleTap) => renderDoubleTapStats(doubleTap),
  });
}

export function createPassModule(): StatModule {
  return createPlayerStatsModule({
    id: "pass",
    label: "Pass",
    select: (player) => player.pass,
    render: (pass) => renderPassStats(pass),
  });
}

export function createOneTimerModule(): StatModule {
  return createPlayerStatsModule({
    id: "one-timer",
    label: "One-timer",
    select: (player) => player.one_timer,
    render: (oneTimer) => renderOneTimerStats(oneTimer),
  });
}

export function createMustyFlickModule(): StatModule {
  return {
    id: "musty-flick",
    label: "Musty Flick",

    setup() {},

    teardown() {},

    onBeforeRender() {},

    renderStats(frameIndex, ctx) {
      const statsFrame = getStatsFrameForReplayFrame(ctx.statsFrameLookup, frameIndex);
      if (!statsFrame) return "";

      return renderGroupedPlayerCards(statsFrame.players, (player) =>
        renderPlayerCard(
          player.name,
          player.is_team_0,
          renderMustyFlickStats(player.musty_flick),
          player.musty_flick?.is_last_musty
            ? '<span class="role-indicator role-forward">Last Musty</span>'
            : "",
        ),
      );
    },

    renderFocusedPlayerStats(playerId, frameIndex, ctx) {
      const player = getStatsPlayerSnapshot(ctx, frameIndex, playerId);
      if (!player) return "";

      return renderMustyFlickStats(player.musty_flick);
    },
  };
}

export function createFlickModule(): StatModule {
  return {
    id: "flick",
    label: "Flick",

    setup() {},

    teardown() {},

    onBeforeRender() {},

    renderStats(frameIndex, ctx) {
      const statsFrame = getStatsFrameForReplayFrame(ctx.statsFrameLookup, frameIndex);
      if (!statsFrame) return "";

      return renderGroupedPlayerCards(statsFrame.players, (player) =>
        renderPlayerCard(
          player.name,
          player.is_team_0,
          renderFlickStats(player.flick),
          player.flick?.is_last_flick
            ? '<span class="role-indicator role-forward">Last Flick</span>'
            : "",
        ),
      );
    },

    renderFocusedPlayerStats(playerId, frameIndex, ctx) {
      const player = getStatsPlayerSnapshot(ctx, frameIndex, playerId);
      if (!player) return "";

      return renderFlickStats(player.flick);
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

    renderStats(frameIndex, ctx) {
      const statsFrame = getStatsFrameForReplayFrame(ctx.statsFrameLookup, frameIndex);
      if (!statsFrame) return "";

      return renderGroupedPlayerCards(statsFrame.players, (player) =>
        renderPlayerCard(
          player.name,
          player.is_team_0,
          renderSpeedFlipStats(player.speed_flip),
          player.speed_flip?.is_last_speed_flip
            ? '<span class="role-indicator role-forward">Last Speed Flip</span>'
            : "",
        ),
      );
    },

    renderFocusedPlayerStats(playerId, frameIndex, ctx) {
      const player = getStatsPlayerSnapshot(ctx, frameIndex, playerId);
      if (!player) return "";

      return renderSpeedFlipStats(player.speed_flip);
    },
  };
}

export function createDodgeModule(): StatModule {
  let overlay: DodgeImpulseOverlay | null = null;

  return {
    id: "dodge",
    label: "Dodge",

    setup(ctx) {
      overlay = new DodgeImpulseOverlay(
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
      return buildDodgeTimelineEvents(ctx.statsTimeline, ctx.replay);
    },

    renderStats(_frameIndex, ctx) {
      const eventsByPlayer = new Map<
        string,
        { name: string; isTeamZero: boolean; count: number }
      >();
      for (const replayPlayer of ctx.replay.players) {
        eventsByPlayer.set(replayPlayer.id, {
          name: replayPlayer.name,
          isTeamZero: replayPlayer.isTeamZero,
          count: 0,
        });
      }
      for (const event of ctx.statsTimeline.events.dodge ?? []) {
        const id = playerIdToString(event.player);
        const player = ctx.replay.players.find((candidate) => candidate.id === id) ?? null;
        const entry = eventsByPlayer.get(id) ?? {
          name: player?.name ?? id,
          isTeamZero: event.is_team_0,
          count: 0,
        };
        entry.count += 1;
        eventsByPlayer.set(id, entry);
      }

      const entries = [...eventsByPlayer.values()].filter((entry) => entry.count > 0);
      if (entries.length === 0) {
        return "";
      }

      return `<div class="player-team-stack">${([true, false] as const)
        .map((isTeamZero) => {
          const teamEntries = entries.filter((entry) => entry.isTeamZero === isTeamZero);
          if (teamEntries.length === 0) {
            return "";
          }
          const teamName = isTeamZero ? "Blue" : "Orange";
          return `<section class="player-team-group ${getTeamClass(isTeamZero)}">
            <div class="player-team-header">
              <h3>${teamName} team</h3>
              <span>${teamEntries.length} player${teamEntries.length === 1 ? "" : "s"}</span>
            </div>
            <div class="player-stats-grid">
              ${teamEntries
                .map((entry) =>
                  renderPlayerCard(
                    entry.name,
                    entry.isTeamZero,
                    `<div class="stat-grid"><div class="stat-row"><span class="label">Events</span><span class="value">${entry.count}</span></div></div>`,
                  ),
                )
                .join("")}
            </div>
          </section>`;
        })
        .join("")}</div>`;
    },

    renderFocusedPlayerStats(playerId, _frameIndex, ctx) {
      const count = (ctx.statsTimeline.events.dodge ?? []).filter((event) => {
        return playerIdToString(event.player) === playerId;
      }).length;
      return `<div class="stat-grid"><div class="stat-row"><span class="label">Events</span><span class="value">${count}</span></div></div>`;
    },
  };
}

export function createHalfFlipModule(): StatModule {
  return {
    id: "half-flip",
    label: "Half Flip",

    setup() {},

    teardown() {},

    onBeforeRender() {},

    renderStats(frameIndex, ctx) {
      const statsFrame = getStatsFrameForReplayFrame(ctx.statsFrameLookup, frameIndex);
      if (!statsFrame) return "";

      return renderGroupedPlayerCards(statsFrame.players, (player) =>
        renderPlayerCard(
          player.name,
          player.is_team_0,
          renderHalfFlipStats(player.half_flip),
          player.half_flip?.is_last_half_flip
            ? '<span class="role-indicator role-forward">Last Half Flip</span>'
            : "",
        ),
      );
    },

    renderFocusedPlayerStats(playerId, frameIndex, ctx) {
      const player = getStatsPlayerSnapshot(ctx, frameIndex, playerId);
      if (!player) return "";

      return renderHalfFlipStats(player.half_flip);
    },
  };
}

export function createWavedashModule(): StatModule {
  return {
    id: "wavedash",
    label: "Wavedash",

    setup() {},

    teardown() {},

    onBeforeRender() {},

    getTimelineEvents(ctx) {
      return buildWavedashTimelineEvents(ctx.statsTimeline, ctx.replay);
    },

    renderStats(frameIndex, ctx) {
      const statsFrame = getStatsFrameForReplayFrame(ctx.statsFrameLookup, frameIndex);
      if (!statsFrame) return "";

      return renderGroupedPlayerCards(statsFrame.players, (player) =>
        renderPlayerCard(
          player.name,
          player.is_team_0,
          renderWavedashStats(player.wavedash),
          player.wavedash?.is_last_wavedash
            ? '<span class="role-indicator role-forward">Last Wavedash</span>'
            : "",
        ),
      );
    },

    renderFocusedPlayerStats(playerId, frameIndex, ctx) {
      const player = getStatsPlayerSnapshot(ctx, frameIndex, playerId);
      if (!player) return "";

      return renderWavedashStats(player.wavedash);
    },
  };
}

export function createWhiffModule(): StatModule {
  return createPlayerStatsModule({
    id: "whiff",
    label: "Whiff",
    select: (player) => player.whiff,
    render: (whiff) => renderWhiffStats(whiff),
    getTimelineEvents(ctx) {
      return buildWhiffTimelineEvents(ctx.statsTimeline, ctx.replay);
    },
  });
}

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

export function createRotationModule(): StatModule {
  return createPlayerStatsModule({
    id: "rotation",
    label: "Rotation",
    select: (player) => player.rotation,
    render: (rotation) => renderRotationStats(rotation),
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

export function createBumpModule(): StatModule {
  return createPlayerStatsModule({
    id: "bump",
    label: "Bump",
    select: (player) => player.bump,
    render: (bump) => renderBumpStats(bump),
    getTimelineEvents(ctx) {
      return buildBumpTimelineEvents(ctx.statsTimeline, ctx.replay);
    },
  });
}
