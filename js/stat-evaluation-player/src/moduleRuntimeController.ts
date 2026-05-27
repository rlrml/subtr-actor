import { createBoostPickupFilterController } from "./boostPickupFilters.ts";
import {
  applyConfigAdapterSnapshot,
  getConfigAdapterSnapshot,
  type StatsPlayerConfigAdapter,
} from "./configAdapters.ts";
import { renderModuleSummaryView, type ModuleCapabilityKind } from "./moduleSummaryView.ts";
import { createStatModules, RELATIVE_POSITIONING_MODULE_ID } from "./statModules.ts";
import type { StatModule, StatModuleContext } from "./statModules.ts";
import {
  getMechanicKinds,
  mechanicKindToModuleId,
} from "./timelineMarkers.ts";
import type {
  BoostPickupAnimationPickup,
  ReplayPlayer,
  ReplayTimelineEvent,
  TimelineOverlayPlugin,
} from "@rlrml/subtr-actor-player";
import type { EventWindowsManager } from "./eventWindows.ts";
import type { StatsFrameLookup, StatsTimeline } from "./statsTimeline.ts";

const RENDER_EFFECT_MODULE_IDS = new Set([
  "ceiling-shot",
  "fifty-fifty",
  "pressure",
  RELATIVE_POSITIONING_MODULE_ID,
  "absolute-positioning",
  "speed-flip",
  "touch",
]);
const TOUCH_MODULE_ID = "touch";

export interface ModuleRuntimeControllerOptions {
  getEventWindowsManager: () => EventWindowsManager;
  getReplayPlayer: () => ReplayPlayer | null;
  getStatsFrameLookup: () => StatsFrameLookup | null;
  getStatsTimeline: () => StatsTimeline | null;
  getTimelineOverlay: () => TimelineOverlayPlugin | null;
  renderTimelineEvent: (event: ReplayTimelineEvent) => ReplayTimelineEvent;
  rerenderStatsWindow: () => void;
  renderModuleRuntimeViews: () => void;
  renderTimelineEventCountValue: (value: string) => void;
  requestConfigSync: () => void;
}

export interface ModuleRuntimeController {
  readonly modules: readonly StatModule[];
  applyModuleConfigSnapshot(configs: Record<string, unknown>): void;
  clearRenderCaches(): void;
  clearTimelineEventSources(): void;
  clearTimelineRangeSources(): void;
  getActiveCapabilityIds(kind: ModuleCapabilityKind): Set<string>;
  getActiveMechanicTimelineKinds(): Set<string>;
  getActiveTimelineEventSourceIds(): Set<string>;
  getContext(): StatModuleContext | null;
  getModuleConfigSnapshot(): Record<string, unknown>;
  getOverlayConfigSnapshot(): {
    timelineEvents: string[];
    timelineRanges: string[];
    mechanics: string[];
    renderEffects: string[];
  };
  includeBoostPickupAnimationPickup(pickup: BoostPickupAnimationPickup): boolean;
  migrateMechanicBackedTimelineEventSelections(): void;
  renderBoostPickupFiltersWindow(container: HTMLElement | null): void;
  renderModuleSettings(container: HTMLElement, touchContainer: HTMLElement | null): void;
  renderModuleSummary(
    container: HTMLElement,
    options: {
      boostPadOverlayEnabled: boolean;
      toggleBoostPadOverlay: () => void;
    },
  ): void;
  renderTimelineEventCount(): void;
  reset(): void;
  setMechanicTimelineKind(kind: string, enabled: boolean): void;
  setOverlayConfig(config: {
    timelineEvents: string[];
    timelineRanges: string[];
    mechanics: string[];
    renderEffects: string[];
  }): void;
  setupActiveModules(): void;
  syncTimelineEvents(): void;
  syncTimelineRanges(): void;
  teardownActiveModules(): void;
  toggleCapability(id: string, kind: ModuleCapabilityKind, enabled: boolean): void;
}

export function createModuleRuntimeController(
  options: ModuleRuntimeControllerOptions,
): ModuleRuntimeController {
  const timelineSourceRemovers = new Map<string, () => void>();
  const timelineRangeSourceRemovers = new Map<string, () => void>();

  const boostPickupFilters = createBoostPickupFilterController({
    refreshTimelineRanges() {
      controller.syncTimelineRanges();
    },
    rerenderCurrentState() {
      const replayPlayer = options.getReplayPlayer();
      if (!replayPlayer) {
        return;
      }
      replayPlayer.setBoostPickupAnimationEnabled(
        replayPlayer.getState().boostPickupAnimationEnabled,
      );
    },
    requestConfigSync() {
      options.requestConfigSync();
    },
  });

  const modules = createStatModules(
    {
      rerenderCurrentState() {
        options.rerenderStatsWindow();
      },
      refreshTimelineRanges() {
        controller.syncTimelineRanges();
      },
      requestConfigSync() {
        options.requestConfigSync();
      },
    },
    {
      boostPickupFilters,
    },
  );

  let activeModules: StatModule[] = [];
  let activeTimelineEventSourceIds = new Set<string>();
  let activeTimelineRangeModuleIds = new Set<string>();
  let activeMechanicTimelineKinds = new Set<string>();
  let activeRenderEffectModuleIds = new Set<string>();
  let removeRenderHook: (() => void) | null = null;

  function getActiveModuleIds(): Set<string> {
    return new Set([
      ...activeTimelineEventSourceIds,
      ...activeTimelineRangeModuleIds,
      ...activeRenderEffectModuleIds,
    ]);
  }

  function getConfigAdapters(): StatsPlayerConfigAdapter[] {
    return modules.filter((mod) => mod.getConfig || mod.applyConfig).map((mod) => {
      const adapter: StatsPlayerConfigAdapter = {
        id: mod.id,
      };
      if (mod.id === "boost") {
        adapter.aliases = ["boost-pickup-animation"];
      }
      if (mod.getConfig) {
        adapter.getConfig = () => mod.getConfig?.();
      }
      if (mod.applyConfig) {
        adapter.applyConfig = (config: unknown) => mod.applyConfig?.(config);
      }
      return adapter;
    });
  }

  const controller: ModuleRuntimeController = {
    modules,
    applyModuleConfigSnapshot(configs) {
      applyConfigAdapterSnapshot(getConfigAdapters(), configs);
    },
    clearRenderCaches() {},
    clearTimelineEventSources() {
      for (const removeSource of timelineSourceRemovers.values()) {
        removeSource();
      }
      timelineSourceRemovers.clear();
    },
    clearTimelineRangeSources() {
      for (const removeSource of timelineRangeSourceRemovers.values()) {
        removeSource();
      }
      timelineRangeSourceRemovers.clear();
    },
    getActiveCapabilityIds(kind) {
      return kind === "events"
        ? activeTimelineEventSourceIds
        : kind === "ranges"
          ? activeTimelineRangeModuleIds
          : activeRenderEffectModuleIds;
    },
    getActiveMechanicTimelineKinds() {
      return activeMechanicTimelineKinds;
    },
    getActiveTimelineEventSourceIds() {
      return activeTimelineEventSourceIds;
    },
    getContext() {
      const replayPlayer = options.getReplayPlayer();
      const statsTimeline = options.getStatsTimeline();
      const statsFrameLookup = options.getStatsFrameLookup();
      if (!replayPlayer || !statsTimeline || !statsFrameLookup) {
        return null;
      }

      return {
        player: replayPlayer,
        replay: replayPlayer.replay,
        statsTimeline,
        statsFrameLookup,
        fieldScale: replayPlayer.options.fieldScale ?? 1,
      };
    },
    getModuleConfigSnapshot() {
      return getConfigAdapterSnapshot(getConfigAdapters());
    },
    getOverlayConfigSnapshot() {
      return {
        timelineEvents: [...activeTimelineEventSourceIds],
        timelineRanges: [...activeTimelineRangeModuleIds],
        mechanics: [...activeMechanicTimelineKinds],
        renderEffects: [...activeRenderEffectModuleIds],
      };
    },
    includeBoostPickupAnimationPickup(pickup) {
      return boostPickupFilters.includePickup(pickup);
    },
    migrateMechanicBackedTimelineEventSelections() {
      for (const kind of getMechanicKinds(options.getStatsTimeline())) {
        const moduleId = mechanicKindToModuleId(kind);
        if (activeTimelineEventSourceIds.delete(moduleId)) {
          activeMechanicTimelineKinds.add(kind);
        }
      }
    },
    renderBoostPickupFiltersWindow(container) {
      if (!container) {
        return;
      }

      const panel = boostPickupFilters.renderSettings(controller.getContext(), {
        showHeader: false,
      });
      container.replaceChildren(panel);
    },
    renderModuleSettings(container, touchContainer) {
      container.replaceChildren();

      const ctx = controller.getContext();
      const panels = activeModules
        .filter((mod) => mod.id !== "boost" && mod.id !== TOUCH_MODULE_ID)
        .map((mod) => mod.renderSettings?.(ctx) ?? null)
        .filter((panel): panel is HTMLElement => panel instanceof HTMLElement);

      if (panels.length === 0) {
        container.hidden = true;
      } else {
        container.hidden = false;
        container.append(...panels);
      }

      if (touchContainer) {
        const touchModule = modules.find((mod) => mod.id === TOUCH_MODULE_ID);
        const panel = touchModule?.renderSettings?.(ctx) ?? null;
        touchContainer.replaceChildren();
        if (panel instanceof HTMLElement) {
          touchContainer.append(panel);
        }
      }
    },
    renderModuleSummary(container, summaryOptions) {
      renderModuleSummaryView({
        container,
        modules,
        renderEffectModuleIds: RENDER_EFFECT_MODULE_IDS,
        getActiveCapabilityIds: controller.getActiveCapabilityIds,
        toggleCapability: controller.toggleCapability,
        boostPickupAnimationEnabled:
          options.getReplayPlayer()?.getState().boostPickupAnimationEnabled ?? false,
        toggleBoostPickupAnimation() {
          const replayPlayer = options.getReplayPlayer();
          const next = !(replayPlayer?.getState().boostPickupAnimationEnabled ?? false);
          replayPlayer?.setBoostPickupAnimationEnabled(next);
          controller.setupActiveModules();
          options.renderModuleRuntimeViews();
          options.requestConfigSync();
        },
        boostPadOverlayEnabled: summaryOptions.boostPadOverlayEnabled,
        toggleBoostPadOverlay: summaryOptions.toggleBoostPadOverlay,
      });
    },
    renderTimelineEventCount() {
      const ctx = controller.getContext();
      options.renderTimelineEventCountValue(
        ctx ? `${options.getEventWindowsManager().countVisibleTimelineSources(ctx)}` : "--",
      );
    },
    reset() {
      controller.teardownActiveModules();
      activeTimelineEventSourceIds = new Set<string>();
      activeTimelineRangeModuleIds = new Set<string>();
      activeMechanicTimelineKinds = new Set<string>();
      activeRenderEffectModuleIds = new Set<string>();
    },
    setMechanicTimelineKind(kind, enabled) {
      if (enabled) {
        activeMechanicTimelineKinds.add(kind);
      } else {
        activeMechanicTimelineKinds.delete(kind);
      }
    },
    setOverlayConfig(config) {
      activeTimelineEventSourceIds = new Set(config.timelineEvents);
      activeTimelineRangeModuleIds = new Set(config.timelineRanges);
      activeMechanicTimelineKinds = new Set(config.mechanics);
      controller.migrateMechanicBackedTimelineEventSelections();
      activeRenderEffectModuleIds = new Set(config.renderEffects);
    },
    setupActiveModules() {
      controller.teardownActiveModules();

      const ctx = controller.getContext();
      if (!ctx) return;

      const activeSourceIds = getActiveModuleIds();
      activeModules = modules.filter((mod) => activeSourceIds.has(mod.id));
      boostPickupFilters.setup(ctx);

      for (const mod of activeModules) {
        mod.setup(ctx);
      }

      removeRenderHook = ctx.player.onBeforeRender((info) => {
        for (const mod of activeModules) {
          if (activeRenderEffectModuleIds.has(mod.id)) {
            mod.onBeforeRender(info);
          }
        }
      });

      controller.syncTimelineEvents();
      controller.syncTimelineRanges();
      controller.clearRenderCaches();
    },
    syncTimelineEvents() {
      controller.clearTimelineEventSources();

      const ctx = controller.getContext();
      const timelineOverlay = options.getTimelineOverlay();
      if (!timelineOverlay || !ctx) {
        return;
      }

      for (const source of options.getEventWindowsManager().getTimelineSources(ctx)) {
        if (!source.active) {
          continue;
        }
        const events = source.buildTimelineEvents();
        if (events.length === 0) continue;

        timelineSourceRemovers.set(
          source.timelineKey,
          timelineOverlay.addEventSource(events.map(options.renderTimelineEvent), {
            id: source.timelineId,
            label: source.label,
          }),
        );
      }

      timelineOverlay.refreshEvents();
    },
    syncTimelineRanges() {
      controller.clearTimelineRangeSources();

      const ctx = controller.getContext();
      const timelineOverlay = options.getTimelineOverlay();
      if (!timelineOverlay || !ctx) {
        return;
      }

      for (const mod of activeModules) {
        if (!activeTimelineRangeModuleIds.has(mod.id) || !mod.getTimelineRanges) {
          continue;
        }

        timelineRangeSourceRemovers.set(
          mod.id,
          timelineOverlay.addRangeSource(() => mod.getTimelineRanges?.(ctx) ?? []),
        );
      }

      for (const source of options.getEventWindowsManager().getTimelineSources(ctx)) {
        if (!source.active || !source.buildTimelineRanges) {
          continue;
        }
        const ranges = source.buildTimelineRanges();
        if (ranges.length === 0) continue;
        timelineRangeSourceRemovers.set(source.timelineKey, timelineOverlay.addRangeSource(ranges));
      }

      timelineOverlay.refreshRanges();
    },
    teardownActiveModules() {
      removeRenderHook?.();
      removeRenderHook = null;
      controller.clearTimelineEventSources();
      controller.clearTimelineRangeSources();

      for (const mod of activeModules) {
        mod.teardown();
      }
      activeModules = [];
      controller.clearRenderCaches();
    },
    toggleCapability(id, kind, enabled) {
      const activeIds = controller.getActiveCapabilityIds(kind);
      if (enabled) {
        activeIds.add(id);
      } else {
        activeIds.delete(id);
      }

      controller.setupActiveModules();
      options.renderModuleRuntimeViews();
      options.rerenderStatsWindow();
      controller.renderTimelineEventCount();
      options.requestConfigSync();
    },
  };

  return controller;
}
