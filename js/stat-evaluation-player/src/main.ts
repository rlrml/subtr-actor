import "./styles.css";
import {
  createBallchasingOverlayPlugin,
  createBoostPadOverlayPlugin,
  createBoostPickupAnimationPlugin,
  createCanvasRecorderPlugin,
  createTimelineOverlayPlugin,
  timelineEventSeekTime,
  ReplayPlayer,
} from "@rlrml/player";
import type {
  BoostPickupAnimationPickup,
  CanvasRecorderPlugin,
  ReplayTimelineEvent,
  ReplayPlayerState,
  TimelineOverlayPlugin,
} from "@rlrml/player";
import { getAppTemplate } from "./appTemplate.ts";
import { createReplayLoadModal } from "./replayLoadModal.ts";
import type { ReplayLoadModalController } from "./replayLoadModal.ts";
import { createCameraControlsController, type CameraControlsController } from "./cameraControls.ts";
import { createStatModules } from "./statModules.ts";
import type { StatModule, StatModuleContext } from "./statModules.ts";
import { createBoostPickupFilterController } from "./boostPickupFilters.ts";
import { applyConfigAdapterSnapshot } from "./configAdapters.ts";
import type { StatsFrameLookup, StatsTimeline } from "./statsTimeline.ts";
import { createStatRegistry, type StatDefinition } from "./statRegistry.ts";
import {
  createStatsWindowsController,
  formatTime,
  type RenderStatsWindowsOptions,
  type StatsWindowsController,
} from "./statsWindows.ts";
import {
  filterReplayTimelineEvents,
  getMechanicKinds,
  mechanicKindToModuleId,
} from "./timelineMarkers.ts";
import { getEventPlaylistSources as getEventPlaylistSourcesFromTimelineSources } from "./eventTimelineSources.ts";
import {
  createEventTimelineControlsController,
  type EventTimelineControlsController,
} from "./eventTimelineControls.ts";
import {
  createModuleControlsController,
  type ModuleCapabilityKind,
  type ModuleControlsController,
} from "./moduleControls.ts";
import {
  createFloatingWindowController,
  type FloatingWindowController,
} from "./floatingWindows.ts";
import {
  createEventPlaylistWindowController,
  type EventPlaylistWindowController,
  type SyncEventPlaylistTimelineOptions,
} from "./eventPlaylistWindow.ts";
import { formatReplayLoadProgress, type ReplayLoadBundle } from "./replayLoader.ts";
import {
  createFileReplaySource,
  createRemoteReplaySource,
  loadReplayBundleFromSource,
  type ReplayInputSource,
} from "./replaySources.ts";
import {
  createRecordingWindowController,
  type RecordingWindowController,
} from "./recordingWindow.ts";
import {
  createScoreboardWindowController,
  type ScoreboardWindowController,
} from "./scoreboardWindow.ts";
import {
  getMechanicsReviewMechanicKind,
  getMechanicsReviewUrlFromLocation,
  type MechanicsReviewItem,
} from "./mechanicsReview.ts";
import { createMechanicsReviewReplayLoadsController } from "./mechanicsReviewReplayLoads.ts";
import {
  createMechanicsReviewWindowController,
  type MechanicsReviewWindowController,
} from "./mechanicsReviewWindow.ts";
import { getReplayFetchRequestFromSearch, type ReplayFetchRequest } from "./replayUrl.ts";
import {
  getStatsPlayerConfigParamSnapshot,
  getStatsPlayerConfigFromLocation,
  isStatsPlayerConfigDebugEnabled,
  setStatsPlayerConfigOnUrl,
  type PlayerCameraConfig,
  type PlayerPlaybackConfig,
  type RecordingConfig,
  type SingletonWindowId,
  type StatsPlayerConfig,
  type StatsPlayerConfigParamSnapshot,
  type StatsWindowConfig,
  type StatsWindowKind,
  type WindowPlacementConfig,
} from "./playerConfig.ts";
import {
  getCameraConfigSnapshot as getCameraConfigSnapshotFromRuntime,
  getConfigAdapters,
  getModuleConfigSnapshot as getModuleConfigSnapshotFromRuntime,
  getPlaybackConfigSnapshot as getPlaybackConfigSnapshotFromRuntime,
  getReplayPlayerStatePatchFromConfig,
  getStatsPlayerConfigSnapshot as getStatsPlayerConfigSnapshotFromRuntime,
  logStatsPlayerConfigLoadDebug,
} from "./playerConfigRuntime.ts";

const DEFAULT_CAMERA_DISTANCE_SCALE = 2.25;
const GOAL_WATCH_LEAD_SECONDS = 4;
const PLAYING_SNAPSHOT_UI_INTERVAL_MS = 100;

let replayPlayer: ReplayPlayer | null = null;
let timelineOverlay: TimelineOverlayPlugin | null = null;
let canvasRecorder: CanvasRecorderPlugin | null = null;
let statsTimeline: StatsTimeline | null = null;
let statsFrameLookup: StatsFrameLookup | null = null;
let unsubscribe: (() => void) | null = null;
let removeRenderHook: (() => void) | null = null;
let lastPlayingSnapshotUiUpdateAt = 0;

const timelineSourceRemovers = new Map<string, () => void>();
const timelineRangeSourceRemovers = new Map<string, () => void>();
const standalonePluginRemovers = new Map<string, () => void>();

const boostPickupFilters = createBoostPickupFilterController({
  refreshTimelineRanges() {
    syncTimelineRanges();
  },
  rerenderCurrentState() {
    if (!replayPlayer) {
      return;
    }
    replayPlayer.setBoostPickupAnimationEnabled(
      replayPlayer.getState().boostPickupAnimationEnabled,
    );
  },
  requestConfigSync() {
    scheduleConfigUrlUpdate();
  },
});

const MODULES = createStatModules(
  {
    rerenderCurrentState() {
      if (!replayPlayer) {
        return;
      }

      const state = replayPlayer.getState();
      renderStatsWindows(state.frameIndex);
    },
    refreshTimelineRanges() {
      syncTimelineRanges();
    },
    requestConfigSync() {
      scheduleConfigUrlUpdate();
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

export interface StatEvaluationPlayerHandle {
  readonly root: HTMLElement;
  destroy(): void;
}

export interface StatEvaluationPlayerMountOptions {
  initialBundle?: ReplayLoadBundle | Promise<ReplayLoadBundle>;
  initialConfig?: StatsPlayerConfig | null;
  initialReplayName?: string;
  loadFromLocation?: boolean;
}

let appRoot: HTMLElement | null = null;
let fileInput!: HTMLInputElement;
let viewport!: HTMLDivElement;
let emptyState!: HTMLDivElement;
let emptyLoadReplay!: HTMLButtonElement;
let launcherToggle!: HTMLButtonElement;
let launcherMenu!: HTMLDivElement;
let loadReplayAction!: HTMLButtonElement;
let floatingWindowLayer!: HTMLDivElement;
let eventPlaylistWindowBody!: HTMLDivElement;
let replayLoadingSummary!: HTMLElement;
let replayLoadingActive!: HTMLElement;
let replayLoadingList!: HTMLDivElement;
let statsWindowLayer!: HTMLDivElement;
let togglePlayback!: HTMLButtonElement;
let playbackRate!: HTMLSelectElement;
let timeReadout!: HTMLElement;
let frameReadout!: HTMLElement;
let durationReadout!: HTMLElement;
let playbackStatusReadout!: HTMLElement;
let statusReadout!: HTMLElement;
let playersReadout!: HTMLElement;
let framesReadout!: HTMLElement;
let eventsReadout!: HTMLElement;
let skipPostGoalTransitions!: HTMLInputElement;
let replayLoadModal: ReplayLoadModalController | null = null;
let skipKickoffs!: HTMLInputElement;
let hitboxWireframes!: HTMLInputElement;
let currentMountCleanup: (() => void) | null = null;
let statRegistry: StatDefinition[] = createStatRegistry(null);
let cameraControlsController: CameraControlsController | null = null;
let recordingWindowController: RecordingWindowController | null = null;
let statsWindowsController: StatsWindowsController | null = null;
let eventPlaylistController: EventPlaylistWindowController | null = null;
let eventTimelineControlsController: EventTimelineControlsController | null = null;
let moduleControlsController: ModuleControlsController | null = null;
let mechanicsReviewController: MechanicsReviewWindowController | null = null;
let floatingWindowController: FloatingWindowController | null = null;
let scoreboardWindowController: ScoreboardWindowController | null = null;
let boostPadOverlayEnabled = true;
let loadedReplayName: string | null = null;
let initialUrlConfig: StatsPlayerConfig | null = null;
let isApplyingConfig = false;
let configUrlUpdateTimer: number | null = null;

function getActiveModuleIds(): Set<string> {
  return new Set([
    ...activeTimelineEventSourceIds,
    ...activeTimelineRangeModuleIds,
    ...activeRenderEffectModuleIds,
  ]);
}

function getActiveCapabilityIds(kind: ModuleCapabilityKind): Set<string> {
  return kind === "events"
    ? activeTimelineEventSourceIds
    : kind === "ranges"
      ? activeTimelineRangeModuleIds
      : activeRenderEffectModuleIds;
}

function clearRenderCaches(): void {}

function getModuleContext(): StatModuleContext | null {
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
}

function setupActiveModules(): void {
  teardownActiveModules();

  const ctx = getModuleContext();
  if (!ctx) return;

  const activeSourceIds = getActiveModuleIds();
  activeModules = MODULES.filter((mod) => activeSourceIds.has(mod.id));
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

  syncTimelineEvents();
  syncTimelineRanges();
  clearRenderCaches();
}

function migrateMechanicBackedTimelineEventSelections(): void {
  for (const kind of getMechanicKinds(statsTimeline)) {
    const moduleId = mechanicKindToModuleId(kind);
    if (activeTimelineEventSourceIds.delete(moduleId)) {
      activeMechanicTimelineKinds.add(kind);
    }
  }
}

function teardownActiveModules(): void {
  removeRenderHook?.();
  removeRenderHook = null;
  clearTimelineEventSources();
  clearTimelineRangeSources();

  for (const mod of activeModules) {
    mod.teardown();
  }
  activeModules = [];
  clearRenderCaches();
}

function toggleCapability(id: string, kind: ModuleCapabilityKind, enabled: boolean): void {
  const activeIds = getActiveCapabilityIds(kind);
  if (enabled) {
    activeIds.add(id);
  } else {
    activeIds.delete(id);
  }

  setupActiveModules();
  renderModuleSummary();
  renderModuleSettings();
  if (replayPlayer) {
    const state = replayPlayer.getState();
    renderStatsWindows(state.frameIndex);
  }
  renderTimelineEventCount();
  scheduleConfigUrlUpdate();
}

function clearTimelineEventSources(): void {
  for (const removeSource of timelineSourceRemovers.values()) {
    removeSource();
  }
  timelineSourceRemovers.clear();
}

function clearTimelineRangeSources(): void {
  for (const removeSource of timelineRangeSourceRemovers.values()) {
    removeSource();
  }
  timelineRangeSourceRemovers.clear();
}

function clearStandalonePlugins(): void {
  for (const removePlugin of standalonePluginRemovers.values()) {
    removePlugin();
  }
  standalonePluginRemovers.clear();
}

function syncBoostPadOverlayPlugin(): void {
  standalonePluginRemovers.get("boost-pad-overlay")?.();
  standalonePluginRemovers.delete("boost-pad-overlay");

  if (!replayPlayer || !boostPadOverlayEnabled) {
    return;
  }

  standalonePluginRemovers.set(
    "boost-pad-overlay",
    replayPlayer.addPlugin(createBoostPadOverlayPlugin()),
  );
}

function toggleBoostPadOverlay(): void {
  boostPadOverlayEnabled = !boostPadOverlayEnabled;
  syncBoostPadOverlayPlugin();
  renderModuleSummary();
  scheduleConfigUrlUpdate();
}

function syncTimelineEvents(): void {
  clearTimelineEventSources();

  const ctx = getModuleContext();
  if (!timelineOverlay || !ctx) {
    return;
  }

  for (const source of getEventTimelineSources(ctx)) {
    if (!source.active) {
      continue;
    }
    const events = source.buildTimelineEvents();
    if (events.length === 0) continue;

    timelineSourceRemovers.set(
      source.timelineKey,
      timelineOverlay.addEventSource(withTimelineEventSeekTimes(events), {
        id: source.timelineId,
        label: source.label,
      }),
    );
  }

  timelineOverlay.refreshEvents();
}

function syncTimelineRanges(): void {
  clearTimelineRangeSources();

  const ctx = getModuleContext();
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

  for (const source of getEventTimelineSources(ctx)) {
    if (!source.active || !source.buildTimelineRanges) {
      continue;
    }
    const ranges = source.buildTimelineRanges();
    if (ranges.length === 0) continue;
    timelineRangeSourceRemovers.set(source.timelineKey, timelineOverlay.addRangeSource(ranges));
  }

  timelineOverlay.refreshRanges();
}

function renderTimelineEventCount(): void {
  const ctx = getModuleContext();
  if (!ctx) {
    eventsReadout.textContent = "--";
    return;
  }

  eventsReadout.textContent = `${countVisibleTimelineSources(ctx)}`;
}

function countVisibleTimelineSources(ctx: StatModuleContext): number {
  return eventTimelineControlsController?.countVisibleSources(ctx) ?? 0;
}

function mustElement<T extends HTMLElement>(root: ParentNode, selector: string): T {
  const element = root.querySelector(selector);
  if (!(element instanceof HTMLElement)) {
    throw new Error(`Missing element for selector: ${selector}`);
  }

  return element as T;
}

function getElementWindowId(element: HTMLElement): string | null {
  return element.closest<HTMLElement>("[data-window-id]")?.dataset.windowId ?? null;
}

function readWindowPlacement(windowEl: HTMLElement): WindowPlacementConfig {
  if (!floatingWindowController) {
    throw new Error("Floating windows are not initialized.");
  }
  return floatingWindowController.readPlacement(windowEl);
}

function applyWindowPlacement(windowEl: HTMLElement, placement: WindowPlacementConfig): void {
  floatingWindowController?.applyPlacement(windowEl, placement);
}

function getSingletonWindowConfigs() {
  return floatingWindowController?.getSingletonConfigs() ?? [];
}

function getModuleConfigSnapshot(): Record<string, unknown> {
  return getModuleConfigSnapshotFromRuntime(MODULES);
}

function applyModuleConfigSnapshot(configs: Record<string, unknown>): void {
  applyConfigAdapterSnapshot(getConfigAdapters(MODULES), configs);
}

function getStatsWindowConfigs(): StatsWindowConfig[] {
  return statsWindowsController?.getConfigs() ?? [];
}

function renderStatsWindows(
  frameIndex = replayPlayer?.getState().frameIndex ?? 0,
  options: RenderStatsWindowsOptions = {},
): void {
  statsWindowsController?.render(frameIndex, options);
}

function createStatsWindow(kind: StatsWindowKind): void {
  statsWindowsController?.create(kind);
}

function replaceStatsWindowsFromConfig(configs: readonly StatsWindowConfig[]): void {
  statsWindowsController?.replaceFromConfig(configs);
}

function clearStatsWindows(): void {
  statsWindowsController?.clear();
}

function getPlaybackConfigSnapshot(): PlayerPlaybackConfig {
  return getPlaybackConfigSnapshotFromRuntime({
    replayPlayer,
    playbackRate,
    skipPostGoalTransitions,
    skipKickoffs,
  });
}

function getCameraConfigSnapshot(): PlayerCameraConfig {
  return getCameraConfigSnapshotFromRuntime({
    replayPlayer,
    cameraControlsController,
  });
}

function getRecordingConfigSnapshot(): RecordingConfig {
  return recordingWindowController?.getConfigSnapshot() ?? {};
}

function getStatsPlayerConfigSnapshot(): StatsPlayerConfig {
  return getStatsPlayerConfigSnapshotFromRuntime({
    playback: getPlaybackConfigSnapshot(),
    camera: getCameraConfigSnapshot(),
    activeTimelineEventSourceIds,
    activeTimelineRangeModuleIds,
    activeMechanicTimelineKinds,
    activeRenderEffectModuleIds,
    initialConfig: initialUrlConfig,
    replayPlayer,
    boostPadOverlayEnabled,
    recording: getRecordingConfigSnapshot(),
    singletonWindows: getSingletonWindowConfigs(),
    statsWindows: getStatsWindowConfigs(),
    moduleConfigs: getModuleConfigSnapshot(),
  });
}

function scheduleConfigUrlUpdate(): void {
  if (isApplyingConfig) {
    return;
  }
  if (configUrlUpdateTimer !== null) {
    window.clearTimeout(configUrlUpdateTimer);
  }
  configUrlUpdateTimer = window.setTimeout(() => {
    configUrlUpdateTimer = null;
    const nextUrl = setStatsPlayerConfigOnUrl(
      new URL(window.location.href),
      getStatsPlayerConfigSnapshot(),
    );
    window.history.replaceState(window.history.state, "", nextUrl);
  }, 150);
}

function applyConfigToExistingWindows(config: StatsPlayerConfig): void {
  floatingWindowController?.applySingletonConfigs(config.singletonWindows);
}

function applyConfigToStaticControls(config: StatsPlayerConfig): void {
  activeTimelineEventSourceIds = new Set(config.overlays.timelineEvents);
  activeTimelineRangeModuleIds = new Set(config.overlays.timelineRanges);
  activeMechanicTimelineKinds = new Set(config.overlays.mechanics);
  migrateMechanicBackedTimelineEventSelections();
  activeRenderEffectModuleIds = new Set(config.overlays.renderEffects);
  boostPadOverlayEnabled = config.overlays.boostPads;
  skipPostGoalTransitions.checked =
    config.playback.skipPostGoalTransitions ?? skipPostGoalTransitions.checked;
  skipKickoffs.checked = config.playback.skipKickoffs ?? skipKickoffs.checked;
  hitboxWireframes.checked = config.overlays.hitboxWireframes;
  if (config.playback.rate !== undefined) {
    playbackRate.value = `${config.playback.rate}`;
  }
  recordingWindowController?.applyConfig(config.recording);
  applyModuleConfigSnapshot(config.moduleConfigs);
  applyConfigToExistingWindows(config);
  replaceStatsWindowsFromConfig(config.statsWindows);
  renderModuleSummary();
  renderModuleSettings();
  renderTimelineEventCount();
}

function watchGoalReplay(time: number, scorerId: string | null): void {
  if (!replayPlayer || !Number.isFinite(time)) {
    return;
  }

  mechanicsReviewController?.clearCurrentClip();

  const canFollowScorer =
    scorerId !== null && replayPlayer.replay.players.some((player) => player.id === scorerId);
  if (canFollowScorer) {
    replayPlayer.setAttachedPlayer(scorerId);
    replayPlayer.setCameraViewMode("follow");
    if (cameraControlsController) {
      cameraControlsController.freeCameraPreset = null;
    }
  }

  skipPostGoalTransitions.checked = false;
  skipKickoffs.checked = false;
  replayPlayer.setState({
    currentTime: Math.max(0, time - GOAL_WATCH_LEAD_SECONDS),
    playing: true,
    skipPostGoalTransitionsEnabled: false,
    skipKickoffsEnabled: false,
  });
  scheduleConfigUrlUpdate();
}

function cueGoalReplay(time: number): void {
  if (!replayPlayer || !Number.isFinite(time)) {
    return;
  }

  mechanicsReviewController?.clearCurrentClip();

  skipPostGoalTransitions.checked = false;
  skipKickoffs.checked = false;
  replayPlayer.setState({
    currentTime: Math.max(0, time - GOAL_WATCH_LEAD_SECONDS),
    playing: false,
    skipPostGoalTransitionsEnabled: false,
    skipKickoffsEnabled: false,
  });
  scheduleConfigUrlUpdate();
}

function cueTimelineEvent(event: ReplayTimelineEvent): void {
  if (!replayPlayer) {
    return;
  }

  mechanicsReviewController?.clearCurrentClip();

  skipPostGoalTransitions.checked = false;
  skipKickoffs.checked = false;
  replayPlayer.setState({
    currentTime: timelineEventSeekTime(event),
    skipPostGoalTransitionsEnabled: false,
    skipKickoffsEnabled: false,
  });
  scheduleConfigUrlUpdate();
}

function withTimelineEventSeekTimes(events: ReplayTimelineEvent[]): ReplayTimelineEvent[] {
  return events.map((event) => ({
    ...event,
    seekTime: timelineEventSeekTime(event),
  }));
}

function applyConfigToReplayPlayer(config: StatsPlayerConfig): void {
  if (!replayPlayer) {
    return;
  }
  replayPlayer.setState(
    getReplayPlayerStatePatchFromConfig(config.playback, config.camera, config),
  );
  if (cameraControlsController) {
    cameraControlsController.freeCameraPreset = config.camera.freePreset ?? null;
  }
  if (config.camera.mode === "free" && config.camera.freePreset) {
    replayPlayer.setFreeCameraPreset(config.camera.freePreset);
  }
  syncBoostPadOverlayPlugin();
  setupActiveModules();
  renderModuleSummary();
  renderModuleSettings();
  renderStatsWindows(replayPlayer.getState().frameIndex);
}

function bringWindowToFront(windowEl: HTMLElement): void {
  floatingWindowController?.bringToFront(windowEl);
}

function showWindow(id: SingletonWindowId): void {
  floatingWindowController?.show(id);
}

function toggleWindow(id: SingletonWindowId): void {
  floatingWindowController?.toggle(id);
}

function hideWindow(id: string): void {
  floatingWindowController?.hide(id);
}

function setLauncherOpen(open: boolean): void {
  launcherMenu.hidden = !open;
  launcherToggle.setAttribute("aria-label", open ? "Close menu" : "Open menu");
  launcherToggle.setAttribute("aria-expanded", open ? "true" : "false");
}

function openReplayFilePicker(): void {
  fileInput.click();
  setLauncherOpen(false);
}

function installWindowDragging(root: HTMLElement, signal: AbortSignal): void {
  floatingWindowController?.installDragging(root, signal);
}

function renderModuleSummary(): void {
  moduleControlsController?.renderSummary();
}

function renderMechanicsTimelineControls(): void {
  eventTimelineControlsController?.render();
}

function getEventTimelineSources(ctx: StatModuleContext | null) {
  return eventTimelineControlsController?.getSources(ctx) ?? [];
}

function getEventPlaylistSourcesForWindow() {
  const ctx = getModuleContext();
  return getEventPlaylistSourcesFromTimelineSources(ctx, getEventTimelineSources(ctx));
}

function renderEventPlaylistWindow(): void {
  eventPlaylistController?.render();
}

function syncEventPlaylistTimeline(
  state: ReplayPlayerState,
  options: SyncEventPlaylistTimelineOptions = {},
): void {
  eventPlaylistController?.syncTimeline(state, options);
}

function resetEventPlaylistWindow(): void {
  eventPlaylistController?.reset();
}

function activateMechanicsReviewTimelineSource(item: MechanicsReviewItem): void {
  const mechanic = getMechanicsReviewMechanicKind(item);
  if (!mechanic) {
    return;
  }

  activeMechanicTimelineKinds.add(mechanic);
  syncTimelineEvents();
  syncTimelineRanges();
  renderMechanicsTimelineControls();
  renderTimelineEventCount();
  scheduleConfigUrlUpdate();
}

function enforceMechanicsReviewClipBoundary(state: ReplayPlayerState): boolean {
  return mechanicsReviewController?.enforceClipBoundary(state) ?? false;
}

function renderModuleSettings(): void {
  moduleControlsController?.renderSettings();
}

function renderScoreboard(frameIndex = replayPlayer?.getState().frameIndex ?? 0): void {
  scoreboardWindowController?.render(frameIndex);
}

function setTransportEnabled(enabled: boolean): void {
  togglePlayback.disabled = !enabled;
  playbackRate.disabled = !enabled;
  skipPostGoalTransitions.disabled = !enabled;
  skipKickoffs.disabled = !enabled;
  hitboxWireframes.disabled = !enabled;
  cameraControlsController?.setTransportEnabled(enabled, replayPlayer?.getState());
}

function syncRecordingWindow(status = canvasRecorder?.getStatus() ?? null): void {
  recordingWindowController?.sync(status);
}

function renderSnapshot(state: ReplayPlayerState): void {
  if (enforceMechanicsReviewClipBoundary(state)) {
    return;
  }

  const now = performance.now();
  if (state.playing && now - lastPlayingSnapshotUiUpdateAt < PLAYING_SNAPSHOT_UI_INTERVAL_MS) {
    return;
  }
  lastPlayingSnapshotUiUpdateAt = now;

  timeReadout.textContent = `${state.currentTime.toFixed(2)}s`;
  frameReadout.textContent = `${state.frameIndex}`;
  durationReadout.textContent = `${state.duration.toFixed(2)}s`;
  playbackStatusReadout.textContent = state.playing ? "Playing" : "Paused";
  togglePlayback.textContent = state.playing ? "Pause" : "Play";
  playbackRate.value = `${state.speed}`;
  cameraControlsController?.syncState(state);
  skipPostGoalTransitions.checked = state.skipPostGoalTransitionsEnabled;
  skipKickoffs.checked = state.skipKickoffsEnabled;
  hitboxWireframes.checked = state.hitboxWireframesEnabled;
  emptyState.hidden = true;

  renderStatsWindows(state.frameIndex, { preserveOpenPickers: true });
  renderScoreboard(state.frameIndex);
  syncEventPlaylistTimeline(state);
}

function includeBoostPickupAnimationPickup(pickup: BoostPickupAnimationPickup): boolean {
  return boostPickupFilters.includePickup(pickup);
}

async function loadReplay(source: ReplayInputSource): Promise<void> {
  await loadReplayBundleForDisplay(
    source,
    Promise.resolve().then(() =>
      loadReplayBundleFromSource(source, (progress) => {
        statusReadout.textContent = formatReplayLoadProgress(progress);
        replayLoadModal?.update(progress);
      }),
    ),
  );
}

async function loadReplayBundleForDisplay(
  source: ReplayInputSource,
  bundlePromise: Promise<ReplayLoadBundle>,
): Promise<void> {
  statusReadout.textContent = source.preparingStatus;
  fileInput.disabled = true;
  replayLoadModal?.show(source.name, source.preparingStatus);
  setTransportEnabled(false);
  cameraControlsController?.syncAvailability();
  emptyState.hidden = false;

  if (unsubscribe) {
    unsubscribe();
    unsubscribe = null;
  }

  teardownActiveModules();
  replayPlayer?.destroy();
  replayPlayer = null;
  canvasRecorder = null;
  loadedReplayName = null;
  timelineOverlay = null;
  statsTimeline = null;
  statsFrameLookup = null;
  statRegistry = createStatRegistry(null);
  clearTimelineEventSources();
  clearTimelineRangeSources();
  clearStandalonePlugins();
  clearRenderCaches();
  resetEventPlaylistWindow();
  renderScoreboard();
  renderTimelineEventCount();
  renderMechanicsTimelineControls();
  renderEventPlaylistWindow();
  renderModuleSettings();
  syncRecordingWindow();

  try {
    statusReadout.textContent = "Parsing replay...";
    replayLoadModal?.show(source.name, "Parsing replay...");
    const loadedReplay = await bundlePromise;
    const { replay } = loadedReplay;
    statsTimeline = loadedReplay.statsTimeline;
    statsFrameLookup = loadedReplay.statsFrameLookup;
    statRegistry = createStatRegistry(null);
    migrateMechanicBackedTimelineEventSelections();

    timelineOverlay = createTimelineOverlayPlugin({
      replayEventsLabel: "Replay",
      replayEvents: (context) =>
        withTimelineEventSeekTimes(
          filterReplayTimelineEvents(context.replay, activeTimelineEventSourceIds),
        ),
    });
    const recorder = createCanvasRecorderPlugin({
      onStatusChange: syncRecordingWindow,
    });
    canvasRecorder = recorder;
    const config = initialUrlConfig;

    replayPlayer = new ReplayPlayer(viewport, replay, {
      initialPlaybackRate: config?.playback.rate,
      initialCameraDistanceScale: config?.camera.distanceScale ?? DEFAULT_CAMERA_DISTANCE_SCALE,
      initialCustomCameraSettings: config?.camera.customSettings ?? null,
      initialAttachedPlayerId: config?.camera.attachedPlayerId ?? null,
      initialCameraViewMode: config?.camera.mode,
      initialBallCamEnabled: config?.camera.ballCam ?? false,
      initialBoostPickupAnimationEnabled: config?.overlays.boostPickupAnimation ?? false,
      initialHitboxWireframesEnabled: config?.overlays.hitboxWireframes ?? hitboxWireframes.checked,
      initialSkipPostGoalTransitionsEnabled: skipPostGoalTransitions.checked,
      initialSkipKickoffsEnabled: skipKickoffs.checked,
      plugins: [
        createBallchasingOverlayPlugin(),
        createBoostPickupAnimationPlugin({
          includePickup: includeBoostPickupAnimationPickup,
        }),
        recorder,
        timelineOverlay,
      ],
    });
    syncBoostPadOverlayPlugin();

    setupActiveModules();
    unsubscribe = replayPlayer.subscribe(renderSnapshot);
    if (config) {
      isApplyingConfig = true;
      try {
        applyConfigToReplayPlayer(config);
      } finally {
        isApplyingConfig = false;
      }
    }

    cameraControlsController?.populateAttachedPlayerOptions(replay.players);
    emptyState.hidden = true;
    statusReadout.textContent = `Loaded ${source.name}`;
    loadedReplayName = source.name;
    playersReadout.textContent = replay.players.map((player) => player.name).join(", ");
    framesReadout.textContent = `${replay.frameCount}`;
    renderTimelineEventCount();
    renderMechanicsTimelineControls();
    resetEventPlaylistWindow();
    renderEventPlaylistWindow();
    setTransportEnabled(true);
    cameraControlsController?.syncAvailability(replayPlayer.getState());
    renderSnapshot(replayPlayer.getState());
    renderStatsWindows(replayPlayer.getState().frameIndex);
    renderScoreboard(replayPlayer.getState().frameIndex);
    syncEventPlaylistTimeline(replayPlayer.getState(), { forceScroll: true });
    renderModuleSettings();
    syncRecordingWindow();
    replayLoadModal?.hide();
  } catch (error) {
    replayLoadModal?.hide();
    replayPlayer?.destroy();
    replayPlayer = null;
    canvasRecorder = null;
    syncRecordingWindow();
    throw error;
  } finally {
    fileInput.disabled = false;
  }
}

function loadReplayFromLocation(signal: AbortSignal): void {
  let replayRequest: ReplayFetchRequest | null;
  try {
    replayRequest = getReplayFetchRequestFromSearch(window.location.search, window.location.href);
  } catch (error) {
    console.error("Invalid replay URL:", error);
    statusReadout.textContent = error instanceof Error ? error.message : "Invalid replay URL";
    return;
  }

  if (!replayRequest) {
    return;
  }

  void loadReplay(createRemoteReplaySource(replayRequest, signal)).catch((error) => {
    if (signal.aborted) {
      return;
    }
    console.error("Failed to load replay URL:", error);
    statusReadout.textContent =
      error instanceof Error ? error.message : "Failed to load replay URL";
  });
}

export function mountStatEvaluationPlayer(
  root: HTMLElement,
  options: StatEvaluationPlayerMountOptions = {},
): StatEvaluationPlayerHandle {
  currentMountCleanup?.();

  root.innerHTML = getAppTemplate(DEFAULT_CAMERA_DISTANCE_SCALE);
  appRoot = root;
  replayLoadModal = createReplayLoadModal(root);
  floatingWindowController = createFloatingWindowController({
    getRoot: () => appRoot ?? document,
    requestConfigSync: scheduleConfigUrlUpdate,
  });

  fileInput = mustElement<HTMLInputElement>(root, "#replay-file");
  viewport = mustElement<HTMLDivElement>(root, "#viewport");
  emptyState = mustElement<HTMLDivElement>(root, "#empty-state");
  emptyLoadReplay = mustElement<HTMLButtonElement>(root, "#empty-load-replay");
  launcherToggle = mustElement<HTMLButtonElement>(root, "#launcher-toggle");
  launcherMenu = mustElement<HTMLDivElement>(root, "#launcher-menu");
  loadReplayAction = mustElement<HTMLButtonElement>(root, "#load-replay-action");
  floatingWindowLayer = mustElement<HTMLDivElement>(root, "#floating-window-layer");
  scoreboardWindowController = createScoreboardWindowController({
    body: mustElement<HTMLDivElement>(root, "#scoreboard-window-body"),
    getReplayPlayer: () => replayPlayer,
    getStatsFrameLookup: () => statsFrameLookup,
  });
  const mechanicsTimelineWindowBody = mustElement<HTMLDivElement>(
    root,
    "#mechanics-timeline-window-body",
  );
  eventTimelineControlsController = createEventTimelineControlsController({
    body: mechanicsTimelineWindowBody,
    modules: MODULES,
    getContext: getModuleContext,
    getActiveTimelineEventSourceIds: () => activeTimelineEventSourceIds,
    getActiveMechanicTimelineKinds: () => activeMechanicTimelineKinds,
    toggleEventSource(id, enabled) {
      toggleCapability(id, "events", enabled);
    },
    setMechanicTimelineKind(kind, enabled) {
      if (enabled) {
        activeMechanicTimelineKinds.add(kind);
      } else {
        activeMechanicTimelineKinds.delete(kind);
      }
      scheduleConfigUrlUpdate();
    },
    setupActiveModules,
    syncTimelineEvents,
    syncTimelineRanges,
    renderModuleSummary,
    renderModuleSettings,
    renderTimelineEventCount,
    requestConfigSync: scheduleConfigUrlUpdate,
  });
  eventPlaylistWindowBody = mustElement<HTMLDivElement>(root, "#event-playlist-window-body");
  eventPlaylistController = createEventPlaylistWindowController({
    body: eventPlaylistWindowBody,
    getReplayPlayer: () => replayPlayer,
    getSources: getEventPlaylistSourcesForWindow,
    cueTimelineEvent,
    formatTime,
  });
  replayLoadingSummary = mustElement<HTMLElement>(root, "#replay-loading-summary");
  replayLoadingActive = mustElement<HTMLElement>(root, "#replay-loading-active");
  replayLoadingList = mustElement<HTMLDivElement>(root, "#replay-loading-list");
  const mechanicsReviewReplayLoadsController = createMechanicsReviewReplayLoadsController({
    elements: {
      reviewSummary: mustElement<HTMLElement>(root, "#mechanics-review-replay-load-summary"),
      loadingSummary: replayLoadingSummary,
      loadingActive: replayLoadingActive,
      loadingList: replayLoadingList,
    },
    isActiveReview(review) {
      return mechanicsReviewController?.review === review;
    },
    onActiveLoadProgress(progress) {
      statusReadout.textContent = formatReplayLoadProgress(progress);
      replayLoadModal?.update(progress);
    },
  });
  mechanicsReviewController = createMechanicsReviewWindowController({
    elements: {
      file: mustElement<HTMLInputElement>(root, "#mechanics-review-file"),
      url: mustElement<HTMLInputElement>(root, "#mechanics-review-url"),
      loadUrl: mustElement<HTMLButtonElement>(root, "#mechanics-review-load-url"),
      status: mustElement<HTMLElement>(root, "#mechanics-review-status"),
      index: mustElement<HTMLElement>(root, "#mechanics-review-index"),
      title: mustElement<HTMLElement>(root, "#mechanics-review-title"),
      mechanic: mustElement<HTMLElement>(root, "#mechanics-review-mechanic"),
      player: mustElement<HTMLElement>(root, "#mechanics-review-player"),
      clip: mustElement<HTMLElement>(root, "#mechanics-review-clip"),
      event: mustElement<HTMLElement>(root, "#mechanics-review-event"),
      reason: mustElement<HTMLElement>(root, "#mechanics-review-reason"),
      previous: mustElement<HTMLButtonElement>(root, "#mechanics-review-prev"),
      replay: mustElement<HTMLButtonElement>(root, "#mechanics-review-replay"),
      next: mustElement<HTMLButtonElement>(root, "#mechanics-review-next"),
      confirm: mustElement<HTMLButtonElement>(root, "#mechanics-review-confirm"),
      reject: mustElement<HTMLButtonElement>(root, "#mechanics-review-reject"),
      uncertain: mustElement<HTMLButtonElement>(root, "#mechanics-review-uncertain"),
      count: mustElement<HTMLElement>(root, "#mechanics-review-count"),
      list: mustElement<HTMLDivElement>(root, "#mechanics-review-list"),
    },
    replayLoads: mechanicsReviewReplayLoadsController,
    getReplayPlayer: () => replayPlayer,
    clearFreeCameraPreset() {
      if (cameraControlsController) {
        cameraControlsController.freeCameraPreset = null;
      }
    },
    resetReplayTransitionControls() {
      skipPostGoalTransitions.checked = false;
      skipKickoffs.checked = false;
    },
    activateTimelineSource: activateMechanicsReviewTimelineSource,
    loadReplayBundleForDisplay,
    showReplayLoadingWindow() {
      showWindow("replay-loading");
    },
  });
  const boostPickupFiltersWindowBody = mustElement<HTMLDivElement>(
    root,
    "#boost-pickup-filters-window-body",
  );
  const touchControlsWindowBody = mustElement<HTMLDivElement>(root, "#touch-controls-window-body");
  statsWindowLayer = mustElement<HTMLDivElement>(root, "#stats-window-layer");
  statsWindowsController = createStatsWindowsController({
    layer: statsWindowLayer,
    getReplayPlayer: () => replayPlayer,
    getStatsTimeline: () => statsTimeline,
    getStatsFrameLookup: () => statsFrameLookup,
    getStatRegistry: () => statRegistry,
    readWindowPlacement,
    applyWindowPlacement,
    bringWindowToFront,
    setLauncherOpen,
    requestConfigSync: scheduleConfigUrlUpdate,
    watchGoalReplay,
    cueGoalReplay,
  });
  togglePlayback = mustElement<HTMLButtonElement>(root, "#toggle-playback");
  playbackRate = mustElement<HTMLSelectElement>(root, "#playback-rate");
  cameraControlsController = createCameraControlsController({
    elements: {
      attachedPlayer: mustElement<HTMLSelectElement>(root, "#attached-player"),
      cameraViewFreeButton: mustElement<HTMLButtonElement>(root, "#camera-view-free"),
      cameraViewFollowButton: mustElement<HTMLButtonElement>(root, "#camera-view-follow"),
      cameraViewOverheadButton: mustElement<HTMLButtonElement>(root, "#camera-view-overhead"),
      cameraViewSideButton: mustElement<HTMLButtonElement>(root, "#camera-view-side"),
      cameraDistance: mustElement<HTMLInputElement>(root, "#camera-distance"),
      cameraDistanceReadout: mustElement<HTMLElement>(root, "#camera-distance-readout"),
      customCameraSettings: mustElement<HTMLInputElement>(root, "#custom-camera-settings"),
      cameraSettingsControls: mustElement<HTMLDivElement>(root, "#camera-settings-controls"),
      customCameraFov: mustElement<HTMLInputElement>(root, "#custom-camera-fov"),
      customCameraHeight: mustElement<HTMLInputElement>(root, "#custom-camera-height"),
      customCameraPitch: mustElement<HTMLInputElement>(root, "#custom-camera-pitch"),
      customCameraDistance: mustElement<HTMLInputElement>(root, "#custom-camera-distance"),
      customCameraStiffness: mustElement<HTMLInputElement>(root, "#custom-camera-stiffness"),
      customCameraSwivelSpeed: mustElement<HTMLInputElement>(root, "#custom-camera-swivel-speed"),
      customCameraTransitionSpeed: mustElement<HTMLInputElement>(
        root,
        "#custom-camera-transition-speed",
      ),
      customCameraFovReadout: mustElement<HTMLElement>(root, "#custom-camera-fov-readout"),
      customCameraHeightReadout: mustElement<HTMLElement>(root, "#custom-camera-height-readout"),
      customCameraPitchReadout: mustElement<HTMLElement>(root, "#custom-camera-pitch-readout"),
      customCameraDistanceReadout: mustElement<HTMLElement>(
        root,
        "#custom-camera-distance-readout",
      ),
      customCameraStiffnessReadout: mustElement<HTMLElement>(
        root,
        "#custom-camera-stiffness-readout",
      ),
      customCameraSwivelSpeedReadout: mustElement<HTMLElement>(
        root,
        "#custom-camera-swivel-speed-readout",
      ),
      customCameraTransitionSpeedReadout: mustElement<HTMLElement>(
        root,
        "#custom-camera-transition-speed-readout",
      ),
      ballCam: mustElement<HTMLInputElement>(root, "#ball-cam"),
      cameraProfileReadout: mustElement<HTMLElement>(root, "#camera-profile-readout"),
      cameraFovReadout: mustElement<HTMLElement>(root, "#camera-fov-readout"),
      cameraHeightReadout: mustElement<HTMLElement>(root, "#camera-height-readout"),
      cameraPitchReadout: mustElement<HTMLElement>(root, "#camera-pitch-readout"),
      cameraBaseDistanceReadout: mustElement<HTMLElement>(root, "#camera-base-distance-readout"),
      cameraStiffnessReadout: mustElement<HTMLElement>(root, "#camera-stiffness-readout"),
    },
    getReplayPlayer: () => replayPlayer,
    requestConfigSync: scheduleConfigUrlUpdate,
  });
  moduleControlsController = createModuleControlsController({
    elements: {
      summary: mustElement<HTMLDivElement>(root, "#module-summary"),
      settings: mustElement<HTMLDivElement>(root, "#module-settings"),
      boostPickupFilters: boostPickupFiltersWindowBody,
      touchControls: touchControlsWindowBody,
    },
    modules: MODULES,
    boostPickupFilters,
    getContext: getModuleContext,
    getActiveModules: () => activeModules,
    getActiveCapabilityIds,
    getBoostPickupAnimationEnabled: () =>
      replayPlayer?.getState().boostPickupAnimationEnabled ?? false,
    getBoostPadOverlayEnabled: () => boostPadOverlayEnabled,
    toggleCapability,
    toggleBoostPickupAnimation() {
      const next = !(replayPlayer?.getState().boostPickupAnimationEnabled ?? false);
      replayPlayer?.setBoostPickupAnimationEnabled(next);
      setupActiveModules();
      renderModuleSummary();
      renderModuleSettings();
      scheduleConfigUrlUpdate();
    },
    toggleBoostPadOverlay,
  });
  timeReadout = mustElement<HTMLElement>(root, "#time-readout");
  frameReadout = mustElement<HTMLElement>(root, "#frame-readout");
  durationReadout = mustElement<HTMLElement>(root, "#duration-readout");
  playbackStatusReadout = mustElement<HTMLElement>(root, "#playback-status-readout");
  statusReadout = mustElement<HTMLElement>(root, "#status-readout");
  playersReadout = mustElement<HTMLElement>(root, "#players-readout");
  framesReadout = mustElement<HTMLElement>(root, "#frames-readout");
  eventsReadout = mustElement<HTMLElement>(root, "#events-readout");
  skipPostGoalTransitions = mustElement<HTMLInputElement>(root, "#skip-post-goal-transitions");
  skipKickoffs = mustElement<HTMLInputElement>(root, "#skip-kickoffs");
  hitboxWireframes = mustElement<HTMLInputElement>(root, "#hitbox-wireframes");
  recordingWindowController = createRecordingWindowController({
    elements: {
      fps: mustElement<HTMLInputElement>(root, "#recording-fps"),
      playbackRate: mustElement<HTMLSelectElement>(root, "#recording-playback-rate"),
      start: mustElement<HTMLButtonElement>(root, "#recording-start"),
      fullReplay: mustElement<HTMLButtonElement>(root, "#recording-full-replay"),
      stop: mustElement<HTMLButtonElement>(root, "#recording-stop"),
      download: mustElement<HTMLButtonElement>(root, "#recording-download"),
      clear: mustElement<HTMLButtonElement>(root, "#recording-clear"),
      status: mustElement<HTMLElement>(root, "#recording-status"),
      elapsed: mustElement<HTMLElement>(root, "#recording-elapsed"),
      size: mustElement<HTMLElement>(root, "#recording-size"),
      type: mustElement<HTMLElement>(root, "#recording-type"),
    },
    getCanvasRecorder: () => canvasRecorder,
    getReplayPlayer: () => replayPlayer,
    getLoadedReplayName: () => loadedReplayName,
    setStatus(message) {
      statusReadout.textContent = message;
    },
    requestConfigSync: scheduleConfigUrlUpdate,
  });

  const configParamSnapshot = getStatsPlayerConfigParamSnapshot(window.location);
  const configDebugEnabled = isStatsPlayerConfigDebugEnabled(window.location);
  let configLoadError: unknown = null;
  if (options.initialConfig !== undefined) {
    initialUrlConfig = options.initialConfig;
  } else {
    try {
      initialUrlConfig = getStatsPlayerConfigFromLocation(window.location);
    } catch (error) {
      configLoadError = error;
      console.error("Invalid stats player config:", error);
      statusReadout.textContent =
        error instanceof Error ? error.message : "Invalid stats player config";
      initialUrlConfig = null;
    }
    if (configDebugEnabled) {
      logStatsPlayerConfigLoadDebug(configParamSnapshot, initialUrlConfig, configLoadError);
    }
  }

  const listeners = new AbortController();
  installWindowDragging(floatingWindowLayer, listeners.signal);
  installWindowDragging(statsWindowLayer, listeners.signal);
  const cleanup = () => {
    listeners.abort();
    unsubscribe?.();
    unsubscribe = null;
    teardownActiveModules();
    replayPlayer?.destroy();
    replayPlayer = null;
    canvasRecorder = null;
    timelineOverlay = null;
    statsTimeline = null;
    statsFrameLookup = null;
    statRegistry = createStatRegistry(null);
    clearStatsWindows();
    statsWindowsController = null;
    clearTimelineEventSources();
    clearTimelineRangeSources();
    clearStandalonePlugins();
    clearRenderCaches();
    activeModules = [];
    replayLoadModal?.destroy();
    replayLoadModal = null;
    activeTimelineEventSourceIds = new Set<string>();
    activeTimelineRangeModuleIds = new Set<string>();
    activeMechanicTimelineKinds = new Set<string>();
    activeRenderEffectModuleIds = new Set<string>();
    resetEventPlaylistWindow();
    eventTimelineControlsController = null;
    eventPlaylistController = null;
    mechanicsReviewController?.reset();
    mechanicsReviewController = null;
    boostPadOverlayEnabled = true;
    loadedReplayName = null;
    cameraControlsController = null;
    recordingWindowController = null;
    moduleControlsController = null;
    scoreboardWindowController = null;
    initialUrlConfig = null;
    if (configUrlUpdateTimer !== null) {
      window.clearTimeout(configUrlUpdateTimer);
      configUrlUpdateTimer = null;
    }
    isApplyingConfig = false;
    floatingWindowController?.reset();
    floatingWindowController = null;
    removeRenderHook = null;
    if (appRoot === root) {
      appRoot = null;
      root.replaceChildren();
    }
    if (currentMountCleanup === cleanup) {
      currentMountCleanup = null;
    }
  };
  currentMountCleanup = cleanup;

  if (initialUrlConfig) {
    isApplyingConfig = true;
    try {
      applyConfigToStaticControls(initialUrlConfig);
    } finally {
      isApplyingConfig = false;
    }
  }

  launcherToggle.addEventListener(
    "click",
    () => {
      setLauncherOpen(launcherMenu.hidden);
    },
    { signal: listeners.signal },
  );

  root.addEventListener(
    "click",
    (event) => {
      if (!(event.target instanceof Element)) {
        return;
      }
      if (!event.target.closest(".top-chrome")) {
        setLauncherOpen(false);
      }
    },
    { signal: listeners.signal },
  );

  loadReplayAction.addEventListener("click", openReplayFilePicker, {
    signal: listeners.signal,
  });
  emptyLoadReplay.addEventListener("click", openReplayFilePicker, {
    signal: listeners.signal,
  });

  root.querySelectorAll<HTMLElement>("[data-window-toggle]").forEach((button) => {
    button.addEventListener(
      "click",
      () => {
        const id = button.dataset.windowToggle as SingletonWindowId | undefined;
        if (id) {
          toggleWindow(id);
          setLauncherOpen(false);
        }
      },
      { signal: listeners.signal },
    );
  });

  root.querySelectorAll<HTMLElement>("[data-window-hide]").forEach((button) => {
    button.addEventListener(
      "click",
      () => {
        const id = button.dataset.windowHide ?? getElementWindowId(button);
        if (id) {
          hideWindow(id);
        }
      },
      { signal: listeners.signal },
    );
  });

  root.querySelectorAll<HTMLElement>("[data-create-stats-window]").forEach((button) => {
    button.addEventListener(
      "click",
      () => {
        createStatsWindow(button.dataset.createStatsWindow as StatsWindowKind);
      },
      { signal: listeners.signal },
    );
  });

  fileInput.addEventListener(
    "change",
    async () => {
      const file = fileInput.files?.[0];
      if (!file) return;

      try {
        mechanicsReviewController?.clearCurrentClip({ resetReplayId: true, render: true });
        await loadReplay(createFileReplaySource(file));
      } catch (error) {
        console.error("Failed to load replay:", error);
        statusReadout.textContent =
          error instanceof Error ? error.message : "Failed to load replay";
      }
    },
    { signal: listeners.signal },
  );

  togglePlayback.addEventListener(
    "click",
    () => {
      replayPlayer?.togglePlayback();
      scheduleConfigUrlUpdate();
    },
    { signal: listeners.signal },
  );

  playbackRate.addEventListener(
    "change",
    () => {
      replayPlayer?.setPlaybackRate(Number(playbackRate.value));
      scheduleConfigUrlUpdate();
    },
    { signal: listeners.signal },
  );

  mechanicsReviewController?.installEventListeners(listeners.signal);
  recordingWindowController?.installEventListeners(listeners.signal);
  cameraControlsController?.installEventListeners(listeners.signal);

  skipPostGoalTransitions.addEventListener(
    "change",
    () => {
      replayPlayer?.setSkipPostGoalTransitionsEnabled(skipPostGoalTransitions.checked);
      scheduleConfigUrlUpdate();
    },
    { signal: listeners.signal },
  );

  skipKickoffs.addEventListener(
    "change",
    () => {
      replayPlayer?.setSkipKickoffsEnabled(skipKickoffs.checked);
      scheduleConfigUrlUpdate();
    },
    { signal: listeners.signal },
  );

  hitboxWireframes.addEventListener(
    "change",
    () => {
      replayPlayer?.setHitboxWireframesEnabled(hitboxWireframes.checked);
      scheduleConfigUrlUpdate();
    },
    { signal: listeners.signal },
  );

  renderModuleSummary();
  renderModuleSettings();
  renderScoreboard();
  cameraControlsController?.renderProfile();
  cameraControlsController?.syncModeButtons();
  syncRecordingWindow();
  renderTimelineEventCount();
  mechanicsReviewController?.render();
  renderEventPlaylistWindow();
  if (options.initialBundle) {
    void loadReplayBundleForDisplay(
      {
        name: options.initialReplayName ?? "replay",
        preparingStatus: "Preparing replay...",
        async readBytes() {
          throw new Error("Replay bytes are not available for this preloaded replay");
        },
      },
      Promise.resolve(options.initialBundle),
    ).catch((error) => {
      if (listeners.signal.aborted) {
        return;
      }
      console.error("Failed to load preprocessed replay bundle:", error);
      statusReadout.textContent =
        error instanceof Error ? error.message : "Failed to load preprocessed replay bundle";
    });
  } else if (options.loadFromLocation !== false) {
    loadReplayFromLocation(listeners.signal);
  }

  const reviewUrl = getMechanicsReviewUrlFromLocation();
  if (reviewUrl) {
    mechanicsReviewController?.setUrl(reviewUrl);
    showWindow("mechanics-review");
    void mechanicsReviewController?.loadPlaylistFromUrl(reviewUrl).catch((error) => {
      if (listeners.signal.aborted) {
        return;
      }
      console.error("Failed to load mechanics review playlist from URL:", error);
      mechanicsReviewController?.setStatus(
        error instanceof Error
          ? error.message
          : "Failed to load mechanics review playlist from URL",
      );
    });
  }

  return {
    root,
    destroy: cleanup,
  };
}
