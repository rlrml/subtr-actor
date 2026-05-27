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
import { createStatModules, RELATIVE_POSITIONING_MODULE_ID } from "./statModules.ts";
import type { StatModule, StatModuleContext } from "./statModules.ts";
import {
  renderModuleSummaryView,
  type ModuleCapabilityKind,
} from "./moduleSummaryView.ts";
import { FloatingWindowController, mustElement } from "./floatingWindows.ts";
import { createBoostPickupFilterController } from "./boostPickupFilters.ts";
import { getStatsFrameForReplayFrame } from "./statsTimeline.ts";
import {
  applyConfigAdapterSnapshot,
  getConfigAdapterSnapshot,
  type StatsPlayerConfigAdapter,
} from "./configAdapters.ts";
import type {
  StatsFrame,
  StatsFrameLookup,
  StatsTimeline,
} from "./statsTimeline.ts";
import { createStatRegistry, type StatDefinition } from "./statRegistry.ts";
import {
  filterReplayTimelineEvents,
  getMechanicKinds,
  mechanicKindToModuleId,
} from "./timelineMarkers.ts";
import {
  formatReplayLoadProgress,
  loadReplayBundleInWorker,
  type ReplayLoadBundle,
} from "./replayLoader.ts";
import { getReplayFetchRequestFromSearch, type ReplayFetchRequest } from "./replayUrl.ts";
import {
  createFileReplaySource,
  createRemoteReplaySource,
  loadReplayBundleFromSource,
  type ReplayInputSource,
} from "./replayInputSources.ts";
import {
  createCameraControls,
  getCameraControlElements,
  type CameraControls,
} from "./cameraControls.ts";
import {
  createRecordingControls,
  getRecordingControlElements,
  type RecordingControls,
} from "./recordingControls.ts";
import { renderScoreboardWindow } from "./scoreboardWindow.ts";
import {
  createMechanicsReviewController,
  getMechanicsReviewElements,
  type MechanicsReviewController,
} from "./mechanicsReviewController.ts";
import { createStatsWindowsManager } from "./statsWindows.ts";
import { createEventWindowsManager } from "./eventWindows.ts";
import {
  getStatsPlayerConfigParamSnapshot,
  getStatsPlayerConfigFromLocation,
  isStatsPlayerConfigDebugEnabled,
  setStatsPlayerConfigOnUrl,
  STATS_PLAYER_CONFIG_VERSION,
  type PlayerCameraConfig,
  type PlayerPlaybackConfig,
  type SingletonWindowId,
  type StatsPlayerConfig,
  type StatsPlayerConfigParamSnapshot,
  type StatsWindowKind,
} from "./playerConfig.ts";

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
      statsWindowManager.render(state.frameIndex);
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
let scoreboardWindowBody!: HTMLDivElement;
let mechanicsTimelineWindowBody!: HTMLDivElement;
let eventPlaylistWindowBody!: HTMLDivElement;
let boostPickupFiltersWindowBody!: HTMLDivElement;
let touchControlsWindowBody!: HTMLDivElement;
let statsWindowLayer!: HTMLDivElement;
let togglePlayback!: HTMLButtonElement;
let playbackRate!: HTMLSelectElement;
let moduleSummaryEl!: HTMLDivElement;
let moduleSettingsEl!: HTMLDivElement;
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
let currentMountCleanup: (() => void) | null = null;
let statRegistry: StatDefinition[] = createStatRegistry(null);
let boostPadOverlayEnabled = true;
let loadedReplayName: string | null = null;
let initialUrlConfig: StatsPlayerConfig | null = null;
let isApplyingConfig = false;
let configUrlUpdateTimer: number | null = null;

const SINGLETON_WINDOW_IDS: SingletonWindowId[] = [
  "camera",
  "scoreboard",
  "playback",
  "recording",
  "mechanics",
  "event-playlist",
  "mechanics-review",
  "replay-loading",
  "boost-pickups",
  "touch-controls",
];
const floatingWindows = new FloatingWindowController(
  () => appRoot ?? document,
  scheduleConfigUrlUpdate,
);

let mechanicsReviewController: MechanicsReviewController | null = null;
let recordingControls: RecordingControls | null = null;
let cameraControls: CameraControls | null = null;

const statsWindowManager = createStatsWindowsManager({
  getDefaultFrameIndex() {
    return replayPlayer?.getState().frameIndex ?? 0;
  },
  getReplayPlayer() {
    return replayPlayer;
  },
  getStatsFrame(frameIndex) {
    return getCurrentStatsFrame(frameIndex);
  },
  getStatsTimeline() {
    return statsTimeline;
  },
  getStatRegistry() {
    return statRegistry;
  },
  getWindowLayer() {
    return statsWindowLayer;
  },
  applyWindowPlacement: (windowEl, placement) =>
    floatingWindows.applyWindowPlacement(windowEl, placement),
  bringWindowToFront: (windowEl) => floatingWindows.bringWindowToFront(windowEl),
  cueGoalReplay(time) {
    replayPlayer?.setState({
      currentTime: Math.max(0, time - GOAL_WATCH_LEAD_SECONDS),
      playing: false,
      skipPostGoalTransitionsEnabled: false,
      skipKickoffsEnabled: false,
    });
    skipPostGoalTransitions.checked = false;
    skipKickoffs.checked = false;
    scheduleConfigUrlUpdate();
  },
  formatTime,
  readWindowPlacement: (windowEl) => floatingWindows.readWindowPlacement(windowEl),
  scheduleConfigUrlUpdate,
  setLauncherOpen,
  watchGoalReplay,
});

const eventWindowsManager = createEventWindowsManager({
  cueTimelineEvent,
  formatTime,
  getActiveMechanicTimelineKinds() {
    return activeMechanicTimelineKinds;
  },
  getActiveTimelineEventSourceIds() {
    return activeTimelineEventSourceIds;
  },
  getModuleContext,
  getModules() {
    return MODULES;
  },
  getPlaylistWindowBody() {
    return eventPlaylistWindowBody;
  },
  getReplayPlayer() {
    return replayPlayer;
  },
  getTimelineWindowBody() {
    return mechanicsTimelineWindowBody;
  },
  renderModuleSettings,
  renderModuleSummary,
  renderTimelineEventCount,
  scheduleConfigUrlUpdate,
  setMechanicTimelineKind(kind, enabled) {
    if (enabled) {
      activeMechanicTimelineKinds.add(kind);
    } else {
      activeMechanicTimelineKinds.delete(kind);
    }
  },
  setupActiveModules,
  syncTimelineEvents,
  syncTimelineRanges,
  toggleCapability,
});

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
    statsWindowManager.render(state.frameIndex);
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

  for (const source of eventWindowsManager.getTimelineSources(ctx)) {
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

  for (const source of eventWindowsManager.getTimelineSources(ctx)) {
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

  eventsReadout.textContent = `${eventWindowsManager.countVisibleTimelineSources(ctx)}`;
}

function getSingletonWindowConfigs() {
  return floatingWindows.getSingletonWindowConfigs(SINGLETON_WINDOW_IDS);
}

function getConfigAdapters(): StatsPlayerConfigAdapter[] {
  return MODULES.filter((mod) => mod.getConfig || mod.applyConfig).map((mod) => {
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

function getModuleConfigSnapshot(): Record<string, unknown> {
  return getConfigAdapterSnapshot(getConfigAdapters());
}

function applyModuleConfigSnapshot(configs: Record<string, unknown>): void {
  applyConfigAdapterSnapshot(getConfigAdapters(), configs);
}

function getPlaybackConfigSnapshot(): PlayerPlaybackConfig {
  const state = replayPlayer?.getState();
  return {
    currentTime: state?.currentTime,
    playing: state?.playing,
    rate: state?.speed ?? Number(playbackRate?.value ?? 1),
    skipPostGoalTransitions: replayPlayer
      ? state?.skipPostGoalTransitionsEnabled
      : skipPostGoalTransitions.checked,
    skipKickoffs: replayPlayer ? state?.skipKickoffsEnabled : skipKickoffs.checked,
  };
}

function getCameraConfigSnapshot(): PlayerCameraConfig {
  return cameraControls?.getConfigSnapshot() ?? {};
}

function getStatsPlayerConfigSnapshot(): StatsPlayerConfig {
  return {
    version: STATS_PLAYER_CONFIG_VERSION,
    playback: getPlaybackConfigSnapshot(),
    camera: getCameraConfigSnapshot(),
    overlays: {
      timelineEvents: [...activeTimelineEventSourceIds],
      timelineRanges: [...activeTimelineRangeModuleIds],
      mechanics: [...activeMechanicTimelineKinds],
      renderEffects: [...activeRenderEffectModuleIds],
      followedPlayerHud: false,
      boostPads: boostPadOverlayEnabled,
      boostPickupAnimation: replayPlayer?.getState().boostPickupAnimationEnabled ?? false,
    },
    recording: recordingControls?.getConfigSnapshot() ?? {},
    singletonWindows: getSingletonWindowConfigs(),
    statsWindows: statsWindowManager.getConfigs(),
    moduleConfigs: getModuleConfigSnapshot(),
  };
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

function logStatsPlayerConfigLoadDebug(
  snapshot: StatsPlayerConfigParamSnapshot,
  config: StatsPlayerConfig | null,
  error: unknown,
): void {
  console.groupCollapsed("[subtr-actor] stats player cfg load");
  console.log("location.href", window.location.href);
  console.log("location.search", snapshot.search || "(empty)");
  console.log("location.hash", snapshot.hash || "(empty)");
  console.table([
    ...snapshot.searchParams.map(([name, value]) => ({
      source: "search",
      name,
      value,
    })),
    ...snapshot.hashParams.map(([name, value]) => ({
      source: "hash",
      name,
      value,
    })),
  ]);
  console.log("cfg selected source", snapshot.selectedSource ?? "(none)");
  console.log("cfg selected raw text", snapshot.selectedValue ?? "(none)");
  console.log("cfg selected raw length", snapshot.selectedValue?.length ?? 0);
  console.log("cfg search values", snapshot.searchValues);
  console.log("cfg hash values", snapshot.hashValues);
  if (snapshot.hashValues.length > 0 && snapshot.searchValues.length > 0) {
    console.warn("Both hash and search contain cfg; hash cfg is used.");
  }
  if (config) {
    console.log("cfg normalized JSON", JSON.stringify(config, null, 2));
    console.log("cfg normalized object", config);
  }
  if (error) {
    console.error("cfg decode/apply error", error);
  }
  console.groupEnd();
}

function applyConfigToExistingWindows(config: StatsPlayerConfig): void {
  floatingWindows.applyWindowConfigs(config.singletonWindows);
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
  if (config.playback.rate !== undefined) {
    playbackRate.value = `${config.playback.rate}`;
  }
  recordingControls?.applyConfig(config.recording);
  applyModuleConfigSnapshot(config.moduleConfigs);
  applyConfigToExistingWindows(config);
  statsWindowManager.replaceFromConfig(config.statsWindows);
  renderModuleSummary();
  renderModuleSettings();
  renderTimelineEventCount();
}

function getReplayPlayerStatePatchFromConfig(
  playback: PlayerPlaybackConfig,
  camera: PlayerCameraConfig,
  config: StatsPlayerConfig,
): Parameters<ReplayPlayer["setState"]>[0] {
  return {
    currentTime: playback.currentTime,
    playing: playback.playing,
    speed: playback.rate,
    cameraDistanceScale: camera.distanceScale,
    customCameraSettings: camera.customSettings,
    cameraViewMode: camera.mode,
    attachedPlayerId: camera.attachedPlayerId,
    ballCamEnabled: camera.ballCam,
    boostPickupAnimationEnabled: config.overlays.boostPickupAnimation,
    skipPostGoalTransitionsEnabled: playback.skipPostGoalTransitions,
    skipKickoffsEnabled: playback.skipKickoffs,
  };
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
    cameraControls?.clearFreePreset();
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
  cameraControls?.applyReplayConfig(config.camera);
  syncBoostPadOverlayPlugin();
  setupActiveModules();
  renderModuleSummary();
  renderModuleSettings();
  statsWindowManager.render(replayPlayer.getState().frameIndex);
}

function showWindow(id: SingletonWindowId): void {
  floatingWindows.showWindow(id);
}

function toggleWindow(id: SingletonWindowId): void {
  floatingWindows.toggleWindow(id);
}

function hideWindow(id: string): void {
  floatingWindows.hideWindow(id);
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
  floatingWindows.installDragging(root, signal);
}

function renderModuleSummary(): void {
  renderModuleSummaryView({
    container: moduleSummaryEl,
    modules: MODULES,
    renderEffectModuleIds: RENDER_EFFECT_MODULE_IDS,
    getActiveCapabilityIds,
    toggleCapability,
    boostPickupAnimationEnabled: replayPlayer?.getState().boostPickupAnimationEnabled ?? false,
    toggleBoostPickupAnimation() {
      const next = !(replayPlayer?.getState().boostPickupAnimationEnabled ?? false);
      replayPlayer?.setBoostPickupAnimationEnabled(next);
      setupActiveModules();
      renderModuleSummary();
      renderModuleSettings();
      scheduleConfigUrlUpdate();
    },
    boostPadOverlayEnabled,
    toggleBoostPadOverlay,
  });
}

function renderModuleSettings(): void {
  moduleSettingsEl.replaceChildren();

  const ctx = getModuleContext();
  const panels = activeModules
    .filter((mod) => mod.id !== "boost" && mod.id !== TOUCH_MODULE_ID)
    .map((mod) => mod.renderSettings?.(ctx) ?? null)
    .filter((panel): panel is HTMLElement => panel instanceof HTMLElement);

  if (panels.length === 0) {
    moduleSettingsEl.hidden = true;
    renderBoostPickupFiltersWindow();
    renderTouchControlsWindow();
    return;
  }

  moduleSettingsEl.hidden = false;
  moduleSettingsEl.append(...panels);
  renderBoostPickupFiltersWindow();
  renderTouchControlsWindow();
}

function renderBoostPickupFiltersWindow(): void {
  if (!boostPickupFiltersWindowBody) {
    return;
  }

  const ctx = getModuleContext();
  const panel = boostPickupFilters.renderSettings(ctx, {
    showHeader: false,
  });
  boostPickupFiltersWindowBody.replaceChildren(panel);
}

function renderScoreboard(frameIndex = replayPlayer?.getState().frameIndex ?? 0): void {
  if (!scoreboardWindowBody) {
    return;
  }

  renderScoreboardWindow(
    scoreboardWindowBody,
    getCurrentStatsFrame(frameIndex),
    replayPlayer !== null,
  );
}

function renderTouchControlsWindow(): void {
  if (!touchControlsWindowBody) {
    return;
  }

  const ctx = getModuleContext();
  const touchModule = MODULES.find((mod) => mod.id === TOUCH_MODULE_ID);
  const panel = touchModule?.renderSettings?.(ctx) ?? null;
  touchControlsWindowBody.replaceChildren();
  if (panel instanceof HTMLElement) {
    touchControlsWindowBody.append(panel);
  }
}

function getStatById(statId: string): StatDefinition | null {
  return statRegistry.find((definition) => definition.id === statId) ?? null;
}

function getCurrentStatsFrame(frameIndex: number): StatsFrame | null {
  return statsFrameLookup ? getStatsFrameForReplayFrame(statsFrameLookup, frameIndex) : null;
}

function formatTime(seconds: number): string {
  if (!Number.isFinite(seconds)) {
    return "--";
  }
  const minutes = Math.floor(Math.max(0, seconds) / 60);
  const remainingSeconds = Math.max(0, seconds) - minutes * 60;
  return `${minutes}:${remainingSeconds.toFixed(1).padStart(4, "0")}`;
}

function setTransportEnabled(enabled: boolean): void {
  togglePlayback.disabled = !enabled;
  playbackRate.disabled = !enabled;
  skipPostGoalTransitions.disabled = !enabled;
  skipKickoffs.disabled = !enabled;
  cameraControls?.setEnabled(enabled);
}

function renderSnapshot(state: ReplayPlayerState): void {
  if (mechanicsReviewController?.enforceClipBoundary(state)) {
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
  skipPostGoalTransitions.checked = state.skipPostGoalTransitionsEnabled;
  skipKickoffs.checked = state.skipKickoffsEnabled;
  emptyState.hidden = true;

  cameraControls?.syncSnapshot(state);
  statsWindowManager.render(state.frameIndex, { preserveOpenPickers: true });
  renderScoreboard(state.frameIndex);
  eventWindowsManager.syncPlaylistTimeline(state);
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
  cameraControls?.syncAvailability();
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
  eventWindowsManager.resetPlaylistState();
  renderScoreboard();
  renderTimelineEventCount();
  eventWindowsManager.renderTimelineControls();
  eventWindowsManager.renderPlaylistWindow();
  renderModuleSettings();
  recordingControls?.sync();

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
      onStatusChange: (status) => recordingControls?.sync(status),
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

    cameraControls?.populateAttachedPlayers(replay.players);
    emptyState.hidden = true;
    statusReadout.textContent = `Loaded ${source.name}`;
    loadedReplayName = source.name;
    playersReadout.textContent = replay.players.map((player) => player.name).join(", ");
    framesReadout.textContent = `${replay.frameCount}`;
    renderTimelineEventCount();
    eventWindowsManager.renderTimelineControls();
    eventWindowsManager.resetPlaylistState();
    eventWindowsManager.renderPlaylistWindow();
    setTransportEnabled(true);
    cameraControls?.syncAvailability(replayPlayer.getState());
    renderSnapshot(replayPlayer.getState());
    statsWindowManager.render(replayPlayer.getState().frameIndex);
    renderScoreboard(replayPlayer.getState().frameIndex);
    eventWindowsManager.syncPlaylistTimeline(replayPlayer.getState(), { forceScroll: true });
    renderModuleSettings();
    recordingControls?.sync();
    replayLoadModal?.hide();
  } catch (error) {
    replayLoadModal?.hide();
    replayPlayer?.destroy();
    replayPlayer = null;
    canvasRecorder = null;
    recordingControls?.sync();
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

  fileInput = mustElement<HTMLInputElement>(root, "#replay-file");
  viewport = mustElement<HTMLDivElement>(root, "#viewport");
  emptyState = mustElement<HTMLDivElement>(root, "#empty-state");
  emptyLoadReplay = mustElement<HTMLButtonElement>(root, "#empty-load-replay");
  launcherToggle = mustElement<HTMLButtonElement>(root, "#launcher-toggle");
  launcherMenu = mustElement<HTMLDivElement>(root, "#launcher-menu");
  loadReplayAction = mustElement<HTMLButtonElement>(root, "#load-replay-action");
  floatingWindowLayer = mustElement<HTMLDivElement>(root, "#floating-window-layer");
  scoreboardWindowBody = mustElement<HTMLDivElement>(root, "#scoreboard-window-body");
  mechanicsTimelineWindowBody = mustElement<HTMLDivElement>(
    root,
    "#mechanics-timeline-window-body",
  );
  eventPlaylistWindowBody = mustElement<HTMLDivElement>(root, "#event-playlist-window-body");
  boostPickupFiltersWindowBody = mustElement<HTMLDivElement>(
    root,
    "#boost-pickup-filters-window-body",
  );
  touchControlsWindowBody = mustElement<HTMLDivElement>(root, "#touch-controls-window-body");
  statsWindowLayer = mustElement<HTMLDivElement>(root, "#stats-window-layer");
  togglePlayback = mustElement<HTMLButtonElement>(root, "#toggle-playback");
  playbackRate = mustElement<HTMLSelectElement>(root, "#playback-rate");
  moduleSummaryEl = mustElement<HTMLDivElement>(root, "#module-summary");
  moduleSettingsEl = mustElement<HTMLDivElement>(root, "#module-settings");
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
  recordingControls = createRecordingControls({
    elements: getRecordingControlElements(root),
    getRecorder() {
      return canvasRecorder;
    },
    getLoadedReplayName() {
      return loadedReplayName;
    },
    hasReplayPlayer() {
      return replayPlayer !== null;
    },
    scheduleConfigUrlUpdate,
    setStatus(message) {
      statusReadout.textContent = message;
    },
  });

  cameraControls = createCameraControls({
    elements: getCameraControlElements(root),
    getReplayPlayer() {
      return replayPlayer;
    },
    scheduleConfigUrlUpdate,
  });

  mechanicsReviewController = createMechanicsReviewController({
    elements: getMechanicsReviewElements(root),
    getReplayPlayer() {
      return replayPlayer;
    },
    loadReplayBundleForDisplay,
    resetTransitionSkipControls() {
      skipPostGoalTransitions.checked = false;
      skipKickoffs.checked = false;
    },
    clearFreeCameraPreset() {
      cameraControls?.clearFreePreset();
    },
    showWindow,
    setStatusReadout(message) {
      statusReadout.textContent = message;
    },
    updateReplayLoadModal(progress) {
      replayLoadModal?.update(progress);
    },
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
    statsWindowManager.clear();
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
    eventWindowsManager.resetPlaylistState();
    mechanicsReviewController?.reset();
    mechanicsReviewController = null;
    recordingControls = null;
    cameraControls = null;
    boostPadOverlayEnabled = true;
    loadedReplayName = null;
    initialUrlConfig = null;
    if (configUrlUpdateTimer !== null) {
      window.clearTimeout(configUrlUpdateTimer);
      configUrlUpdateTimer = null;
    }
    isApplyingConfig = false;
    floatingWindows.resetZIndex();
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
        const id = button.dataset.windowHide ?? floatingWindows.getElementWindowId(button);
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
        statsWindowManager.create(button.dataset.createStatsWindow as StatsWindowKind);
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
        mechanicsReviewController?.clearCurrentReplay();
        await loadReplay(createFileReplaySource(file));
      } catch (error) {
        console.error("Failed to load replay:", error);
        statusReadout.textContent =
          error instanceof Error ? error.message : "Failed to load replay";
      }
    },
    { signal: listeners.signal },
  );

  mechanicsReviewController?.installListeners(listeners.signal);

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

  recordingControls?.installListeners(listeners.signal);
  cameraControls?.installListeners(listeners.signal);

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

  renderModuleSummary();
  renderModuleSettings();
  renderScoreboard();
  cameraControls?.syncAvailability();
  recordingControls?.sync();
  renderTimelineEventCount();
  mechanicsReviewController?.render();
  eventWindowsManager.renderPlaylistWindow();
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

  mechanicsReviewController?.loadFromLocation(listeners.signal);

  return {
    root,
    destroy: cleanup,
  };
}
