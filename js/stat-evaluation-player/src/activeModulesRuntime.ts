import type { ReplayTimelineEvent, TimelineOverlayPlugin } from "@rlrml/player";
import type { StatsReplayPlayer } from "./statsReplayPlayer.ts";
import type { BoostPickupFilterController } from "./boostPickupFilters.ts";
import type { EventTimelineSource } from "./eventTimelineSources.ts";
import type { ModuleCapabilityKind } from "./moduleControls.ts";
import type { StatModule, StatModuleContext } from "./statModules.ts";

export interface ActiveModulesRuntimeOptions {
  readonly modules: readonly StatModule[];
  readonly boostPickupFilters: BoostPickupFilterController;
  getContext(): StatModuleContext | null;
  getReplayPlayer(): StatsReplayPlayer | null;
  getTimelineOverlay(): TimelineOverlayPlugin | null;
  getEventTimelineSources(ctx: StatModuleContext | null): EventTimelineSource[];
  withTimelineEventSeekTimes(events: ReplayTimelineEvent[]): ReplayTimelineEvent[];
  renderModuleSummary(): void;
  renderModuleSettings(): void;
  renderStatsWindows(): void;
  renderTimelineEventCount(): void;
  requestConfigSync(): void;
}

export class ActiveModulesRuntime {
  private activeModules: StatModule[] = [];
  private activeTimelineEventSourceIds = new Set<string>();
  private activeTimelineRangeModuleIds = new Set<string>();
  private activeMechanicTimelineKinds = new Set<string>();
  private activeRenderEffectModuleIds = new Set<string>();
  private removeRenderHook: (() => void) | null = null;
  private readonly timelineSourceRemovers = new Map<string, () => void>();
  private readonly timelineRangeSourceRemovers = new Map<string, () => void>();

  constructor(private readonly options: ActiveModulesRuntimeOptions) {}

  getActiveModules(): readonly StatModule[] {
    return this.activeModules;
  }

  getActiveTimelineEventSourceIds(): ReadonlySet<string> {
    return this.activeTimelineEventSourceIds;
  }

  getActiveTimelineRangeModuleIds(): ReadonlySet<string> {
    return this.activeTimelineRangeModuleIds;
  }

  getActiveMechanicTimelineKinds(): ReadonlySet<string> {
    return this.activeMechanicTimelineKinds;
  }

  getActiveRenderEffectModuleIds(): ReadonlySet<string> {
    return this.activeRenderEffectModuleIds;
  }

  getActiveCapabilityIds(kind: ModuleCapabilityKind): ReadonlySet<string> {
    return kind === "events"
      ? this.activeTimelineEventSourceIds
      : kind === "ranges"
        ? this.activeTimelineRangeModuleIds
        : this.activeRenderEffectModuleIds;
  }

  getBoostPadOverlayEnabled(): boolean {
    return true;
  }

  getTimelineEventSourceIds(): string[] {
    return [...this.activeTimelineEventSourceIds];
  }

  getTimelineRangeModuleIds(): string[] {
    return [...this.activeTimelineRangeModuleIds];
  }

  getMechanicTimelineKinds(): string[] {
    return [...this.activeMechanicTimelineKinds];
  }

  getRenderEffectModuleIds(): string[] {
    return [...this.activeRenderEffectModuleIds];
  }

  applyOverlayConfig({
    timelineEvents,
    timelineRanges,
    mechanics,
    renderEffects,
    boostPads,
  }: {
    timelineEvents: readonly string[];
    timelineRanges: readonly string[];
    mechanics: readonly string[];
    renderEffects: readonly string[];
    boostPads: boolean;
  }): void {
    this.activeTimelineEventSourceIds = new Set(timelineEvents);
    this.activeTimelineRangeModuleIds = new Set(timelineRanges);
    this.activeMechanicTimelineKinds = new Set(mechanics);
    this.activeRenderEffectModuleIds = new Set(renderEffects);
    void boostPads;
  }

  reset(): void {
    this.teardownActiveModules();
    this.clearStandalonePlugins();
    this.activeModules = [];
    this.activeTimelineEventSourceIds = new Set<string>();
    this.activeTimelineRangeModuleIds = new Set<string>();
    this.activeMechanicTimelineKinds = new Set<string>();
    this.activeRenderEffectModuleIds = new Set<string>();
    this.removeRenderHook = null;
  }

  setupActiveModules(): void {
    this.teardownActiveModules();

    const ctx = this.options.getContext();
    if (!ctx) return;

    const activeSourceIds = this.getActiveModuleIds();
    this.activeModules = this.options.modules.filter((mod) => activeSourceIds.has(mod.id));
    this.options.boostPickupFilters.setup(ctx);

    for (const mod of this.activeModules) {
      mod.setup(ctx);
    }

    this.removeRenderHook = ctx.player.onBeforeRender((info) => {
      for (const mod of this.activeModules) {
        if (this.activeRenderEffectModuleIds.has(mod.id)) {
          mod.onBeforeRender(info);
        }
      }
    });

    this.syncTimelineEvents();
    this.syncTimelineRanges();
  }

  teardownActiveModules(): void {
    this.removeRenderHook?.();
    this.removeRenderHook = null;
    this.clearTimelineEventSources();
    this.clearTimelineRangeSources();

    for (const mod of this.activeModules) {
      mod.teardown();
    }
    this.activeModules = [];
  }

  toggleCapability(id: string, kind: ModuleCapabilityKind, enabled: boolean): void {
    const activeIds = this.getMutableActiveCapabilityIds(kind);
    if (enabled) {
      activeIds.add(id);
    } else {
      activeIds.delete(id);
    }

    this.setupActiveModules();
    this.options.renderModuleSummary();
    this.options.renderModuleSettings();
    this.options.renderStatsWindows();
    this.options.renderTimelineEventCount();
    this.options.requestConfigSync();
  }

  setMechanicTimelineKind(kind: string, enabled: boolean): void {
    if (enabled) {
      this.activeMechanicTimelineKinds.add(kind);
    } else {
      this.activeMechanicTimelineKinds.delete(kind);
    }
    this.options.requestConfigSync();
  }

  activateMechanicTimelineKind(kind: string): void {
    this.activeMechanicTimelineKinds.add(kind);
    this.syncTimelineEvents();
    this.syncTimelineRanges();
    this.options.renderTimelineEventCount();
    this.options.requestConfigSync();
  }

  clearTimelineEventSources(): void {
    for (const removeSource of this.timelineSourceRemovers.values()) {
      removeSource();
    }
    this.timelineSourceRemovers.clear();
  }

  clearTimelineRangeSources(): void {
    for (const removeSource of this.timelineRangeSourceRemovers.values()) {
      removeSource();
    }
    this.timelineRangeSourceRemovers.clear();
  }

  clearStandalonePlugins(): void {
    // Boost pad rendering is baseline player scene content now; no standalone
    // render plugins are managed by the stats module runtime.
  }

  syncBoostPadOverlayPlugin(): void {
    // Kept as a compatibility no-op for existing wiring.
  }

  toggleBoostPadOverlay(): void {
    // Boost pad locations are always rendered by the player.
    this.options.renderModuleSummary();
    this.options.requestConfigSync();
  }

  syncTimelineEvents(): void {
    this.clearTimelineEventSources();

    const ctx = this.options.getContext();
    const timelineOverlay = this.options.getTimelineOverlay();
    if (!timelineOverlay || !ctx) {
      return;
    }

    for (const source of this.options.getEventTimelineSources(ctx)) {
      if (!source.active) {
        continue;
      }
      const events = source.buildTimelineEvents();
      if (events.length === 0) continue;

      this.timelineSourceRemovers.set(
        source.timelineKey,
        timelineOverlay.addEventSource(this.options.withTimelineEventSeekTimes(events), {
          id: source.timelineId,
          label: source.label,
        }),
      );
    }

    timelineOverlay.refreshEvents();
  }

  syncTimelineRanges(): void {
    this.clearTimelineRangeSources();

    const ctx = this.options.getContext();
    const timelineOverlay = this.options.getTimelineOverlay();
    if (!timelineOverlay || !ctx) {
      return;
    }

    for (const mod of this.activeModules) {
      if (!this.activeTimelineRangeModuleIds.has(mod.id) || !mod.getTimelineRanges) {
        continue;
      }

      this.timelineRangeSourceRemovers.set(
        mod.id,
        timelineOverlay.addRangeSource(() => mod.getTimelineRanges?.(ctx) ?? []),
      );
    }

    for (const source of this.options.getEventTimelineSources(ctx)) {
      if (!source.active || !source.buildTimelineRanges) {
        continue;
      }
      const ranges = source.buildTimelineRanges();
      if (ranges.length === 0) continue;
      this.timelineRangeSourceRemovers.set(
        source.timelineKey,
        timelineOverlay.addRangeSource(ranges),
      );
    }

    timelineOverlay.refreshRanges();
  }

  private getActiveModuleIds(): Set<string> {
    return new Set([
      ...this.activeTimelineEventSourceIds,
      ...this.activeTimelineRangeModuleIds,
      ...this.activeRenderEffectModuleIds,
    ]);
  }

  private getMutableActiveCapabilityIds(kind: ModuleCapabilityKind): Set<string> {
    return kind === "events"
      ? this.activeTimelineEventSourceIds
      : kind === "ranges"
        ? this.activeTimelineRangeModuleIds
        : this.activeRenderEffectModuleIds;
  }
}

export function createActiveModulesRuntime(
  options: ActiveModulesRuntimeOptions,
): ActiveModulesRuntime {
  return new ActiveModulesRuntime(options);
}
