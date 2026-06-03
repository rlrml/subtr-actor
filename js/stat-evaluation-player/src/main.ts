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
import { createStatModules, getTeamClass, RELATIVE_POSITIONING_MODULE_ID } from "./statModules.ts";
import type { StatModule, StatModuleContext } from "./statModules.ts";
import { createBoostPickupFilterController } from "./boostPickupFilters.ts";
import { getStatsFrameForReplayFrame } from "./statsTimeline.ts";
import {
  applyConfigAdapterSnapshot,
  getConfigAdapterSnapshot,
  type StatsPlayerConfigAdapter,
} from "./configAdapters.ts";
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
  createEventPlaylistWindowController,
  type EventPlaylistWindowController,
  type SyncEventPlaylistTimelineOptions,
} from "./eventPlaylistWindow.ts";
import {
  formatReplayLoadProgress,
  loadReplayBundleInWorker,
  type ReplayLoadBundle,
  type ReplayLoadProgress,
} from "./replayLoader.ts";
import {
  createRecordingWindowController,
  type RecordingWindowController,
} from "./recordingWindow.ts";
import {
  formatMechanicsReviewBound,
  formatMechanicsReviewClipDetails,
  formatMechanicsReviewEventDetails,
  formatMechanicsReviewTime,
  getMechanicsReviewItemLabel,
  getMechanicsReviewMechanicLabel,
  getMechanicsReviewMechanicKind,
  getMechanicsReviewPlayerId,
  getMechanicsReviewReplayLabel,
  getMechanicsReviewReplayPath,
  getMechanicsReviewTargetNumber,
  getMechanicsReviewTargetTime,
  getMechanicsReviewUrlFromLocation,
  parseMechanicsReviewPlaylistJson,
  resolveMechanicsReviewUrl,
  type ActiveMechanicsReview,
  type MechanicsReviewItem,
  type MechanicsReviewPlaybackBound,
  type MechanicsReviewPlaylist,
  type MechanicsReviewReplay,
} from "./mechanicsReview.ts";
import {
  createMechanicsReviewReplayLoadsController,
  type MechanicsReviewReplayLoadsController,
} from "./mechanicsReviewReplayLoads.ts";
import { getReplayFetchRequestFromSearch, type ReplayFetchRequest } from "./replayUrl.ts";
import {
  getStatsPlayerConfigParamSnapshot,
  getStatsPlayerConfigFromLocation,
  isStatsPlayerConfigDebugEnabled,
  mapWindowPlacementToViewport,
  setStatsPlayerConfigOnUrl,
  STATS_PLAYER_CONFIG_VERSION,
  type ConfigViewportSize,
  type PlayerCameraConfig,
  type PlayerPlaybackConfig,
  type RecordingConfig,
  type SingletonWindowConfig,
  type SingletonWindowId,
  type StatsPlayerConfig,
  type StatsPlayerConfigParamSnapshot,
  type StatsWindowConfig,
  type StatsWindowKind,
  type WindowPlacementConfig,
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
let eventPlaylistWindowBody!: HTMLDivElement;
let mechanicsReviewFile!: HTMLInputElement;
let mechanicsReviewUrl!: HTMLInputElement;
let mechanicsReviewLoadUrl!: HTMLButtonElement;
let mechanicsReviewStatus!: HTMLElement;
let mechanicsReviewIndex!: HTMLElement;
let mechanicsReviewTitle!: HTMLElement;
let mechanicsReviewMechanic!: HTMLElement;
let mechanicsReviewPlayer!: HTMLElement;
let mechanicsReviewClip!: HTMLElement;
let mechanicsReviewEvent!: HTMLElement;
let mechanicsReviewReason!: HTMLElement;
let mechanicsReviewPrev!: HTMLButtonElement;
let mechanicsReviewReplay!: HTMLButtonElement;
let mechanicsReviewNext!: HTMLButtonElement;
let mechanicsReviewConfirm!: HTMLButtonElement;
let mechanicsReviewReject!: HTMLButtonElement;
let mechanicsReviewUncertain!: HTMLButtonElement;
let mechanicsReviewReplayLoadSummary!: HTMLElement;
let replayLoadingSummary!: HTMLElement;
let replayLoadingActive!: HTMLElement;
let replayLoadingList!: HTMLDivElement;
let mechanicsReviewCount!: HTMLElement;
let mechanicsReviewList!: HTMLDivElement;
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
let hitboxWireframes!: HTMLInputElement;
let currentMountCleanup: (() => void) | null = null;
let statRegistry: StatDefinition[] = createStatRegistry(null);
let cameraControlsController: CameraControlsController | null = null;
let recordingWindowController: RecordingWindowController | null = null;
let statsWindowsController: StatsWindowsController | null = null;
let eventPlaylistController: EventPlaylistWindowController | null = null;
let eventTimelineControlsController: EventTimelineControlsController | null = null;
let mechanicsReviewReplayLoadsController: MechanicsReviewReplayLoadsController | null = null;
let nextWindowZIndex = 30;
let boostPadOverlayEnabled = true;
let loadedReplayName: string | null = null;
let initialUrlConfig: StatsPlayerConfig | null = null;
let isApplyingConfig = false;
let configUrlUpdateTimer: number | null = null;

interface ReplayInputSource {
  name: string;
  preparingStatus: string;
  readBytes(): Promise<Uint8Array>;
}

type ModuleCapabilityKind = "events" | "ranges" | "effects";
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

let activeMechanicsReview: ActiveMechanicsReview | null = null;
let mechanicsReviewBoundaryGuard = false;

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

function getCurrentViewportSize(): ConfigViewportSize {
  return {
    width: Math.max(1, window.innerWidth),
    height: Math.max(1, window.innerHeight),
  };
}

function readWindowCoordinate(windowEl: HTMLElement, propertyName: string): number {
  const inlineValue = windowEl.style.getPropertyValue(propertyName).trim();
  const computedValue = getComputedStyle(windowEl).getPropertyValue(propertyName).trim();
  const rawValue = inlineValue || computedValue;
  const parsed = Number.parseFloat(rawValue);
  if (Number.isFinite(parsed)) {
    return parsed;
  }

  const rect = windowEl.getBoundingClientRect();
  return propertyName === "--window-y" ? rect.top : rect.left;
}

function readWindowPlacement(windowEl: HTMLElement): WindowPlacementConfig {
  const zIndex = Number.parseInt(windowEl.style.zIndex, 10);
  return {
    x: readWindowCoordinate(windowEl, "--window-x"),
    y: readWindowCoordinate(windowEl, "--window-y"),
    viewport: getCurrentViewportSize(),
    zIndex: Number.isFinite(zIndex) ? zIndex : undefined,
    visible: !windowEl.hidden,
  };
}

function applyWindowPlacement(windowEl: HTMLElement, placement: WindowPlacementConfig): void {
  const mapped = mapWindowPlacementToViewport(placement, getCurrentViewportSize());
  windowEl.style.setProperty("--window-x", `${mapped.x}px`);
  windowEl.style.setProperty("--window-y", `${mapped.y}px`);
  windowEl.hidden = !placement.visible;
  if (placement.zIndex !== undefined) {
    windowEl.style.zIndex = `${placement.zIndex}`;
    nextWindowZIndex = Math.max(nextWindowZIndex, placement.zIndex + 1);
  }
}

function getSingletonWindowConfigs(): SingletonWindowConfig[] {
  const configs: SingletonWindowConfig[] = [];
  const root = appRoot ?? document;
  for (const id of SINGLETON_WINDOW_IDS) {
    const element = root.querySelector<HTMLElement>(`[data-window-id="${id}"]`);
    if (element) {
      configs.push({
        id,
        placement: readWindowPlacement(element),
      });
    }
  }
  return configs;
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
  const state = replayPlayer?.getState();
  return {
    mode: state?.cameraViewMode,
    freePreset: cameraControlsController?.freeCameraPreset ?? null,
    attachedPlayerId: state?.attachedPlayerId,
    distanceScale: state?.cameraDistanceScale,
    ballCam: state?.ballCamEnabled ?? cameraControlsController?.ballCamChecked,
    customSettings: state?.customCameraSettings,
  };
}

function getRecordingConfigSnapshot(): RecordingConfig {
  return recordingWindowController?.getConfigSnapshot() ?? {};
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
      ...(initialUrlConfig?.overlays.pluginRenderEffects !== undefined
        ? { pluginRenderEffects: [...initialUrlConfig.overlays.pluginRenderEffects] }
        : {}),
      ...(initialUrlConfig?.overlays.pluginHudOverlay !== undefined
        ? { pluginHudOverlay: initialUrlConfig.overlays.pluginHudOverlay }
        : {}),
      followedPlayerHud: false,
      boostPads: boostPadOverlayEnabled,
      boostPickupAnimation: replayPlayer?.getState().boostPickupAnimationEnabled ?? false,
      hitboxWireframes: replayPlayer?.getState().hitboxWireframesEnabled ?? false,
    },
    recording: getRecordingConfigSnapshot(),
    singletonWindows: getSingletonWindowConfigs(),
    statsWindows: getStatsWindowConfigs(),
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
  const root = appRoot ?? document;
  for (const windowConfig of config.singletonWindows) {
    const element = root.querySelector<HTMLElement>(`[data-window-id="${windowConfig.id}"]`);
    if (element) {
      applyWindowPlacement(element, windowConfig.placement);
    }
  }
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
    hitboxWireframesEnabled: config.overlays.hitboxWireframes,
    skipPostGoalTransitionsEnabled: playback.skipPostGoalTransitions,
    skipKickoffsEnabled: playback.skipKickoffs,
  };
}

function watchGoalReplay(time: number, scorerId: string | null): void {
  if (!replayPlayer || !Number.isFinite(time)) {
    return;
  }

  if (activeMechanicsReview) {
    activeMechanicsReview.currentClip = null;
  }

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

  if (activeMechanicsReview) {
    activeMechanicsReview.currentClip = null;
  }

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

  if (activeMechanicsReview) {
    activeMechanicsReview.currentClip = null;
  }

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
  windowEl.style.zIndex = `${nextWindowZIndex++}`;
}

function showWindow(id: SingletonWindowId): void {
  const windowEl = mustElement<HTMLElement>(appRoot ?? document, `[data-window-id="${id}"]`);
  windowEl.hidden = false;
  bringWindowToFront(windowEl);
  scheduleConfigUrlUpdate();
}

function toggleWindow(id: SingletonWindowId): void {
  const windowEl = mustElement<HTMLElement>(appRoot ?? document, `[data-window-id="${id}"]`);
  windowEl.hidden = !windowEl.hidden;
  if (!windowEl.hidden) {
    bringWindowToFront(windowEl);
  }
  scheduleConfigUrlUpdate();
}

function hideWindow(id: string): void {
  const windowEl = mustElement<HTMLElement>(appRoot ?? document, `[data-window-id="${id}"]`);
  windowEl.hidden = true;
  scheduleConfigUrlUpdate();
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

function isInteractiveDragTarget(target: EventTarget | null): boolean {
  return (
    target instanceof Element &&
    Boolean(target.closest("button, input, select, textarea, option, label, a, [data-no-drag]"))
  );
}

function installWindowDragging(root: HTMLElement, signal: AbortSignal): void {
  root.addEventListener(
    "pointerdown",
    (event) => {
      if (!(event.target instanceof HTMLElement) || isInteractiveDragTarget(event.target)) {
        return;
      }

      const windowEl = event.target.closest<HTMLElement>("[data-window-id]");
      if (!windowEl || windowEl.hidden) {
        return;
      }

      bringWindowToFront(windowEl);
      const startX = event.clientX;
      const startY = event.clientY;
      const rect = windowEl.getBoundingClientRect();
      const pointerId = event.pointerId;

      windowEl.setPointerCapture(pointerId);
      event.preventDefault();

      const onPointerMove = (moveEvent: PointerEvent) => {
        const nextX = Math.max(
          8,
          Math.min(window.innerWidth - 120, rect.left + moveEvent.clientX - startX),
        );
        const nextY = Math.max(
          8,
          Math.min(window.innerHeight - 100, rect.top + moveEvent.clientY - startY),
        );
        windowEl.style.setProperty("--window-x", `${nextX}px`);
        windowEl.style.setProperty("--window-y", `${nextY}px`);
      };

      const onPointerUp = () => {
        windowEl.releasePointerCapture(pointerId);
        windowEl.removeEventListener("pointermove", onPointerMove);
        windowEl.removeEventListener("pointerup", onPointerUp);
        windowEl.removeEventListener("pointercancel", onPointerUp);
        scheduleConfigUrlUpdate();
      };

      windowEl.addEventListener("pointermove", onPointerMove);
      windowEl.addEventListener("pointerup", onPointerUp);
      windowEl.addEventListener("pointercancel", onPointerUp);
    },
    { signal },
  );
}

function renderModuleSummary(): void {
  moduleSummaryEl.replaceChildren();

  const timelineToggles: HTMLButtonElement[] = [];
  const inGameVisualizationToggles: HTMLButtonElement[] = [];

  for (const mod of MODULES) {
    const hasRenderEffect = RENDER_EFFECT_MODULE_IDS.has(mod.id);
    if (!mod.getTimelineEvents && !mod.getTimelineRanges && !hasRenderEffect) {
      continue;
    }

    if (mod.getTimelineEvents) {
      timelineToggles.push(
        renderCapabilityToggle(mod.id, getCapabilityLabel(mod, "events"), "events"),
      );
    }
    if (mod.getTimelineRanges) {
      timelineToggles.push(
        renderCapabilityToggle(mod.id, getCapabilityLabel(mod, "ranges"), "ranges"),
      );
    }
    if (hasRenderEffect) {
      inGameVisualizationToggles.push(
        renderCapabilityToggle(mod.id, getCapabilityLabel(mod, "effects"), "effects"),
      );
    }
  }

  const boostAnimationActive = replayPlayer?.getState().boostPickupAnimationEnabled ?? false;
  const boostAnimation = document.createElement("button");
  boostAnimation.type = "button";
  boostAnimation.className = "module-summary-item";
  boostAnimation.dataset.active = boostAnimationActive ? "true" : "false";
  boostAnimation.setAttribute("aria-pressed", boostAnimationActive ? "true" : "false");
  boostAnimation.addEventListener("click", () => {
    const next = !(replayPlayer?.getState().boostPickupAnimationEnabled ?? false);
    replayPlayer?.setBoostPickupAnimationEnabled(next);
    setupActiveModules();
    renderModuleSummary();
    renderModuleSettings();
    scheduleConfigUrlUpdate();
  });
  const boostName = document.createElement("span");
  boostName.textContent = "Boost pickup animation";
  const boostState = document.createElement("strong");
  boostState.textContent = boostAnimationActive ? "On" : "Off";
  boostAnimation.append(boostName, boostState);
  inGameVisualizationToggles.push(boostAnimation);

  const boostPadOverlay = document.createElement("button");
  boostPadOverlay.type = "button";
  boostPadOverlay.className = "module-summary-item";
  boostPadOverlay.dataset.active = boostPadOverlayEnabled ? "true" : "false";
  boostPadOverlay.setAttribute("aria-pressed", boostPadOverlayEnabled ? "true" : "false");
  boostPadOverlay.addEventListener("click", toggleBoostPadOverlay);
  const boostPadName = document.createElement("span");
  boostPadName.textContent = "Boost pad locations";
  const boostPadState = document.createElement("strong");
  boostPadState.textContent = boostPadOverlayEnabled ? "On" : "Off";
  boostPadOverlay.append(boostPadName, boostPadState);
  inGameVisualizationToggles.push(boostPadOverlay);

  moduleSummaryEl.append(
    renderModuleSummaryGroup("Timeline visualizations", timelineToggles),
    renderModuleSummaryGroup("In-game visualizations", inGameVisualizationToggles),
  );
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

function getMechanicsReviewBoundTime(bound: MechanicsReviewPlaybackBound): number {
  if (bound.kind === "time") {
    return bound.value;
  }
  const frameIndex = Math.max(0, Math.trunc(bound.value));
  return (
    replayPlayer?.replay.frames[frameIndex]?.time ?? replayPlayer?.replay.frames.at(-1)?.time ?? 0
  );
}

function getMechanicsReviewPlayerName(item: MechanicsReviewItem): string {
  if (typeof item.meta?.playerName === "string" && item.meta.playerName.trim()) {
    return item.meta.playerName;
  }
  const playerId = getMechanicsReviewPlayerId(item);
  return playerId
    ? (replayPlayer?.replay.players.find((player) => player.id === playerId)?.name ?? playerId)
    : "--";
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

function formatMechanicsReviewStatus(value: unknown): string {
  return typeof value === "string" && value.trim() ? value.replaceAll("_", " ") : "unreviewed";
}

function getMechanicsReviewDecisionEndpoint(item: MechanicsReviewItem | null): string | null {
  if (!item) {
    return null;
  }
  if (typeof item.meta?.reviewEndpoint === "string" && item.meta.reviewEndpoint) {
    return item.meta.reviewEndpoint;
  }
  const eventId =
    typeof item.meta?.eventId === "string" && item.meta.eventId ? item.meta.eventId : item.id;
  return eventId ? `/api/v1/mechanics/events/${encodeURIComponent(eventId)}/reviews` : null;
}

function mechanicsReviewAuthHeaders(): Record<string, string> {
  const params = new URLSearchParams(window.location.search);
  const token =
    params.get("reviewToken") ??
    params.get("token") ??
    window.localStorage.getItem("rocket_sense_access_token");
  return token ? { Authorization: `Bearer ${token}` } : {};
}

function setMechanicsReviewStatus(message: string): void {
  if (mechanicsReviewStatus) {
    mechanicsReviewStatus.textContent = message;
  }
}

function getMechanicsReviewReplayLoadsController(): MechanicsReviewReplayLoadsController {
  if (!mechanicsReviewReplayLoadsController) {
    throw new Error("Mechanics review replay loads are not initialized.");
  }
  return mechanicsReviewReplayLoadsController;
}

function createMechanicsReviewReplaySource(
  item: MechanicsReviewItem,
  review: ActiveMechanicsReview,
  signal?: AbortSignal,
): ReplayInputSource {
  return getMechanicsReviewReplayLoadsController().createReplaySource(item, review, signal);
}

function initializeMechanicsReviewReplayLoadStates(review: ActiveMechanicsReview): void {
  getMechanicsReviewReplayLoadsController().initialize(review);
}

function renderMechanicsReviewReplayLoads(review: ActiveMechanicsReview | null): void {
  mechanicsReviewReplayLoadsController?.render(review);
}

function preloadMechanicsReviewReplays(
  review: ActiveMechanicsReview,
  currentReplayId: string,
): void {
  getMechanicsReviewReplayLoadsController().preload(review, currentReplayId);
}

function loadMechanicsReviewReplayBundle(
  item: MechanicsReviewItem,
  review: ActiveMechanicsReview,
): Promise<ReplayLoadBundle> {
  return getMechanicsReviewReplayLoadsController().loadBundle(item, review);
}

function renderMechanicsReviewWindow(): void {
  if (!mechanicsReviewList) {
    return;
  }

  const review = activeMechanicsReview;
  const items = review?.manifest.items ?? [];
  const item = review ? (items[review.currentIndex] ?? null) : null;
  const hasItems = items.length > 0;

  mechanicsReviewCount.textContent = `${items.length} item${items.length === 1 ? "" : "s"}`;
  mechanicsReviewIndex.textContent =
    hasItems && review ? `${review.currentIndex + 1} / ${items.length}` : "0 / 0";
  mechanicsReviewTitle.textContent = item
    ? getMechanicsReviewItemLabel(item, review?.currentIndex ?? 0)
    : "No candidate selected";
  mechanicsReviewMechanic.textContent = item ? getMechanicsReviewMechanicLabel(item) : "--";
  mechanicsReviewPlayer.textContent = item ? getMechanicsReviewPlayerName(item) : "--";
  mechanicsReviewClip.textContent = item ? formatMechanicsReviewClipDetails(item) : "--";
  mechanicsReviewEvent.textContent = item ? formatMechanicsReviewEventDetails(item) : "--";
  mechanicsReviewReason.textContent = item?.meta?.reason ?? "--";
  mechanicsReviewPrev.disabled = !review || review.loading || review.currentIndex <= 0;
  mechanicsReviewReplay.disabled = !review || review.loading || !review.currentClip;
  mechanicsReviewNext.disabled =
    !review || review.loading || review.currentIndex >= items.length - 1;
  const decisionDisabled =
    !review || review.loading || getMechanicsReviewDecisionEndpoint(item) === null;
  mechanicsReviewConfirm.disabled = decisionDisabled;
  mechanicsReviewReject.disabled = decisionDisabled;
  mechanicsReviewUncertain.disabled = decisionDisabled;
  renderMechanicsReviewReplayLoads(review);

  mechanicsReviewList.replaceChildren();
  if (!review || items.length === 0) {
    const empty = document.createElement("p");
    empty.className = "stat-window-empty";
    empty.textContent = "No review playlist loaded.";
    mechanicsReviewList.append(empty);
    return;
  }

  items.forEach((candidate, index) => {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "mechanics-review-item";
    button.dataset.active = index === review.currentIndex ? "true" : "false";
    button.disabled = review.loading;
    button.addEventListener("click", () => {
      void activateMechanicsReviewItem(index);
    });

    const title = document.createElement("span");
    title.textContent = getMechanicsReviewItemLabel(candidate, index);

    const meta = document.createElement("strong");
    meta.textContent =
      [
        getMechanicsReviewMechanicLabel(candidate),
        getMechanicsReviewPlayerName(candidate),
        formatMechanicsReviewStatus(candidate.meta?.reviewStatus),
      ]
        .filter((part) => part && part !== "--")
        .join(" · ") || "--";

    button.append(title, meta);
    mechanicsReviewList.append(button);
  });
}

async function loadMechanicsReviewPlaylist(
  manifest: MechanicsReviewPlaylist,
  sourceUrl: string | null,
): Promise<void> {
  const replaysById = new Map<string, MechanicsReviewReplay>();
  for (const replay of manifest.replays ?? []) {
    replaysById.set(replay.id, replay);
  }

  activeMechanicsReview = {
    manifest,
    sourceUrl,
    replaysById,
    replayLoadStates: new Map(),
    replayLoadCache: new Map(),
    currentIndex: 0,
    loading: false,
    preloading: false,
    currentReplayId: null,
    currentClip: null,
  };
  initializeMechanicsReviewReplayLoadStates(activeMechanicsReview);
  showWindow("replay-loading");
  setMechanicsReviewStatus(
    manifest.label ? `Loaded ${manifest.label}.` : `Loaded review playlist.`,
  );
  renderMechanicsReviewWindow();

  if (manifest.items.length > 0) {
    await activateMechanicsReviewItem(0);
  }
}

async function loadMechanicsReviewPlaylistFromUrl(urlText: string): Promise<void> {
  if (!urlText) {
    setMechanicsReviewStatus("Enter a review playlist URL.");
    return;
  }
  const url = resolveMechanicsReviewUrl(urlText, window.location.href);
  setMechanicsReviewStatus("Loading review playlist...");
  const response = await fetch(url);
  if (!response.ok) {
    const statusText = response.statusText ? ` ${response.statusText}` : "";
    throw new Error(
      `Failed to fetch review playlist from ${url} (${response.status}${statusText})`,
    );
  }
  const manifest = parseMechanicsReviewPlaylistJson(await response.text());
  await loadMechanicsReviewPlaylist(manifest, response.url || url);
}

async function activateMechanicsReviewItem(index: number): Promise<void> {
  const review = activeMechanicsReview;
  const item = review?.manifest.items[index];
  if (!review || !item || review.loading) {
    return;
  }

  review.loading = true;
  review.currentIndex = index;
  renderMechanicsReviewWindow();
  setMechanicsReviewStatus(`Loading ${getMechanicsReviewItemLabel(item, index)}...`);

  try {
    if (!replayPlayer || review.currentReplayId !== item.replay) {
      const source = createMechanicsReviewReplaySource(item, review);
      const replayBundlePromise = loadMechanicsReviewReplayBundle(item, review);
      await loadReplayBundleForDisplay(source, replayBundlePromise);
      review.currentReplayId = item.replay;
    }
    preloadMechanicsReviewReplays(review, item.replay);

    const startTime = Math.max(0, getMechanicsReviewBoundTime(item.start));
    const endTime = Math.min(
      replayPlayer?.getState().duration ?? Number.POSITIVE_INFINITY,
      Math.max(startTime, getMechanicsReviewBoundTime(item.end)),
    );
    if (!Number.isFinite(startTime) || !Number.isFinite(endTime) || endTime <= startTime) {
      throw new Error("Review item has an empty playback range.");
    }

    const playerId = getMechanicsReviewPlayerId(item);
    if (playerId && replayPlayer?.replay.players.some((player) => player.id === playerId)) {
      replayPlayer.setAttachedPlayer(playerId);
      replayPlayer.setCameraViewMode("follow");
      if (cameraControlsController) {
        cameraControlsController.freeCameraPreset = null;
      }
    }

    skipPostGoalTransitions.checked = false;
    skipKickoffs.checked = false;
    const targetTime = getMechanicsReviewTargetTime(item);
    review.currentClip = { startTime, endTime, targetTime };
    activateMechanicsReviewTimelineSource(item);
    replayPlayer?.setState({
      currentTime: startTime,
      playing: true,
      skipPostGoalTransitionsEnabled: false,
      skipKickoffsEnabled: false,
    });
    setMechanicsReviewStatus(
      targetTime === null
        ? `Playing ${startTime.toFixed(2)}s to ${endTime.toFixed(2)}s`
        : `Playing ${startTime.toFixed(2)}s to ${endTime.toFixed(2)}s; target ${targetTime.toFixed(2)}s`,
    );
  } catch (error) {
    console.error("Failed to activate mechanics review item:", error);
    review.currentClip = null;
    setMechanicsReviewStatus(error instanceof Error ? error.message : "Failed to load review item");
  } finally {
    review.loading = false;
    renderMechanicsReviewWindow();
  }
}

function replayMechanicsReviewClip(): void {
  const clip = activeMechanicsReview?.currentClip;
  if (!clip || !replayPlayer) {
    return;
  }
  replayPlayer.setState({
    currentTime: clip.startTime,
    playing: true,
    skipPostGoalTransitionsEnabled: false,
    skipKickoffsEnabled: false,
  });
}

async function submitMechanicsReviewDecision(
  status: "confirmed" | "rejected" | "uncertain",
): Promise<void> {
  const review = activeMechanicsReview;
  const item = review?.manifest.items[review.currentIndex] ?? null;
  const endpoint = getMechanicsReviewDecisionEndpoint(item);
  if (!review || !item || !endpoint) {
    setMechanicsReviewStatus("Current review item has no review endpoint.");
    return;
  }

  setMechanicsReviewStatus(`Submitting ${formatMechanicsReviewStatus(status)}...`);
  const response = await fetch(endpoint, {
    method: "POST",
    headers: {
      "content-type": "application/json",
      ...mechanicsReviewAuthHeaders(),
    },
    credentials: "same-origin",
    body: JSON.stringify({ status }),
  });
  if (!response.ok) {
    let message = `${response.status}${response.statusText ? ` ${response.statusText}` : ""}`;
    try {
      const body = (await response.json()) as { error?: unknown };
      if (typeof body.error === "string") {
        message = body.error;
      }
    } catch {
      // Keep the HTTP status fallback.
    }
    setMechanicsReviewStatus(`Review failed: ${message}`);
    return;
  }

  item.meta = item.meta ?? {};
  item.meta.reviewStatus = status;
  setMechanicsReviewStatus(`Marked ${formatMechanicsReviewStatus(status)}.`);
  renderMechanicsReviewWindow();
}

function enforceMechanicsReviewClipBoundary(state: ReplayPlayerState): boolean {
  const clip = activeMechanicsReview?.currentClip;
  if (!clip || !replayPlayer || mechanicsReviewBoundaryGuard) {
    return false;
  }

  const beforeStart = state.currentTime < clip.startTime - 0.1;
  const atOrPastEnd = state.playing && state.currentTime >= clip.endTime - 0.025;
  if (!beforeStart && !atOrPastEnd) {
    return false;
  }

  mechanicsReviewBoundaryGuard = true;
  try {
    replayPlayer.setState({
      currentTime: beforeStart ? clip.startTime : clip.endTime,
      playing: false,
      skipPostGoalTransitionsEnabled: false,
      skipKickoffsEnabled: false,
    });
    if (atOrPastEnd) {
      setMechanicsReviewStatus(`Finished clip at ${clip.endTime.toFixed(2)}s`);
    }
  } finally {
    mechanicsReviewBoundaryGuard = false;
  }
  return true;
}

function renderModuleSummaryGroup(title: string, items: HTMLButtonElement[]): HTMLElement {
  const group = document.createElement("section");
  group.className = "module-summary-group";

  const heading = document.createElement("h3");
  heading.textContent = title;

  const list = document.createElement("div");
  list.className = "module-list";
  list.append(...items);

  group.append(heading, list);
  return group;
}

function getCapabilityLabel(mod: StatModule, kind: ModuleCapabilityKind): string {
  const timelineLabels: Record<string, string> = {
    "absolute-positioning:ranges": "Position zones",
    "backboard:events": "Backboard",
    "ball-carry:events": "Ball carry",
    "boost:ranges": "Boost pickup timeline",
    "bump:events": "Bump",
    "ceiling-shot:events": "Ceiling shot",
    "demo:events": "Demo",
    "dodge-reset:events": "Dodge refresh",
    "double-tap:events": "Double tap",
    "fifty-fifty:events": "50/50",
    "half-flip:events": "Half flip",
    "musty-flick:events": "Musty flick",
    "possession:ranges": "Possession",
    "powerslide:events": "Powerslide",
    "pressure:ranges": "Half control",
    "rush:ranges": "Rush",
    "speed-flip:events": "Speed flip",
    "touch:events": "Touch",
    "wavedash:events": "Wavedash",
  };
  const inGameVisualizationLabels: Record<string, string> = {
    "absolute-positioning": "Position zones",
    "ceiling-shot": "Ceiling shot labels",
    "fifty-fifty": "50/50 labels",
    pressure: "Half control",
    "relative-positioning": "Player roles",
    "speed-flip": "Speed flip labels",
    touch: "Touch labels",
  };

  if (kind === "effects") {
    return inGameVisualizationLabels[mod.id] ?? mod.label;
  }

  return timelineLabels[`${mod.id}:${kind}`] ?? `${mod.label} timeline`;
}

function renderCapabilityToggle(
  moduleId: string,
  label: string,
  kind: ModuleCapabilityKind,
): HTMLButtonElement {
  const activeIds = getActiveCapabilityIds(kind);
  const active = activeIds.has(moduleId);
  const item = document.createElement("button");
  item.type = "button";
  item.className = "module-summary-item";
  item.dataset.active = active ? "true" : "false";
  item.setAttribute("aria-pressed", active ? "true" : "false");
  item.addEventListener("click", () => {
    toggleCapability(moduleId, kind, !activeIds.has(moduleId));
  });

  const name = document.createElement("span");
  name.textContent = label;

  const state = document.createElement("strong");
  state.textContent = active ? "On" : "Off";

  item.append(name, state);
  return item;
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

function formatScoreboardInteger(value: number | null | undefined): string {
  return typeof value === "number" && Number.isFinite(value) ? `${Math.round(value)}` : "--";
}

function renderScoreboard(frameIndex = replayPlayer?.getState().frameIndex ?? 0): void {
  if (!scoreboardWindowBody) {
    return;
  }

  scoreboardWindowBody.replaceChildren();
  const frame = statsFrameLookup ? getStatsFrameForReplayFrame(statsFrameLookup, frameIndex) : null;
  const replay = replayPlayer?.replay ?? null;
  if (!frame || !replay) {
    const empty = document.createElement("p");
    empty.className = "scoreboard-empty";
    empty.textContent = "Load a replay to show the scoreboard.";
    scoreboardWindowBody.append(empty);
    return;
  }

  const header = document.createElement("div");
  header.className = "scoreboard-scoreline";
  header.append(
    createScoreboardGoalValue(frame.team_zero?.core.goals, true),
    createScoreboardDivider(),
    createScoreboardGoalValue(frame.team_one?.core.goals, false),
  );

  scoreboardWindowBody.append(header);
}

function createScoreboardDivider(): HTMLElement {
  const divider = document.createElement("span");
  divider.className = "scoreboard-divider";
  divider.textContent = "-";
  return divider;
}

function createScoreboardGoalValue(
  goals: number | null | undefined,
  isTeamZero: boolean,
): HTMLElement {
  const score = document.createElement("strong");
  score.className = `scoreboard-goal-value ${getTeamClass(isTeamZero)}`;
  score.textContent = formatScoreboardInteger(goals);
  return score;
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

function createFileReplaySource(file: File): ReplayInputSource {
  return {
    name: file.name,
    preparingStatus: "Preparing replay...",
    async readBytes() {
      return new Uint8Array(await file.arrayBuffer());
    },
  };
}

function createRemoteReplaySource(
  request: ReplayFetchRequest,
  signal: AbortSignal,
): ReplayInputSource {
  return {
    name: request.name,
    preparingStatus: "Fetching replay...",
    async readBytes() {
      const response = await fetch(request.url, {
        ...request.fetchInit,
        signal,
      });
      if (!response.ok) {
        const statusText = response.statusText ? ` ${response.statusText}` : "";
        const authHint =
          request.kind === "ballchasing" && [401, 403, 404].includes(response.status)
            ? ". The replay may be private, unavailable, or not downloadable without a Ballchasing session"
            : "";
        throw new Error(
          `Failed to fetch replay from ${request.url.href} (${response.status}${statusText})${authHint}`,
        );
      }
      return new Uint8Array(await response.arrayBuffer());
    },
  };
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

async function loadReplayBundleFromSource(
  source: ReplayInputSource,
  onProgress?: (progress: ReplayLoadProgress) => void,
): Promise<ReplayLoadBundle> {
  const bytes = await source.readBytes();
  return loadReplayBundleInWorker(bytes, {
    reportEveryNFrames: 100,
    onProgress,
  });
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

  fileInput = mustElement<HTMLInputElement>(root, "#replay-file");
  viewport = mustElement<HTMLDivElement>(root, "#viewport");
  emptyState = mustElement<HTMLDivElement>(root, "#empty-state");
  emptyLoadReplay = mustElement<HTMLButtonElement>(root, "#empty-load-replay");
  launcherToggle = mustElement<HTMLButtonElement>(root, "#launcher-toggle");
  launcherMenu = mustElement<HTMLDivElement>(root, "#launcher-menu");
  loadReplayAction = mustElement<HTMLButtonElement>(root, "#load-replay-action");
  floatingWindowLayer = mustElement<HTMLDivElement>(root, "#floating-window-layer");
  scoreboardWindowBody = mustElement<HTMLDivElement>(root, "#scoreboard-window-body");
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
  mechanicsReviewFile = mustElement<HTMLInputElement>(root, "#mechanics-review-file");
  mechanicsReviewUrl = mustElement<HTMLInputElement>(root, "#mechanics-review-url");
  mechanicsReviewLoadUrl = mustElement<HTMLButtonElement>(root, "#mechanics-review-load-url");
  mechanicsReviewStatus = mustElement<HTMLElement>(root, "#mechanics-review-status");
  mechanicsReviewIndex = mustElement<HTMLElement>(root, "#mechanics-review-index");
  mechanicsReviewTitle = mustElement<HTMLElement>(root, "#mechanics-review-title");
  mechanicsReviewMechanic = mustElement<HTMLElement>(root, "#mechanics-review-mechanic");
  mechanicsReviewPlayer = mustElement<HTMLElement>(root, "#mechanics-review-player");
  mechanicsReviewClip = mustElement<HTMLElement>(root, "#mechanics-review-clip");
  mechanicsReviewEvent = mustElement<HTMLElement>(root, "#mechanics-review-event");
  mechanicsReviewReason = mustElement<HTMLElement>(root, "#mechanics-review-reason");
  mechanicsReviewPrev = mustElement<HTMLButtonElement>(root, "#mechanics-review-prev");
  mechanicsReviewReplay = mustElement<HTMLButtonElement>(root, "#mechanics-review-replay");
  mechanicsReviewNext = mustElement<HTMLButtonElement>(root, "#mechanics-review-next");
  mechanicsReviewConfirm = mustElement<HTMLButtonElement>(root, "#mechanics-review-confirm");
  mechanicsReviewReject = mustElement<HTMLButtonElement>(root, "#mechanics-review-reject");
  mechanicsReviewUncertain = mustElement<HTMLButtonElement>(root, "#mechanics-review-uncertain");
  mechanicsReviewReplayLoadSummary = mustElement<HTMLElement>(
    root,
    "#mechanics-review-replay-load-summary",
  );
  replayLoadingSummary = mustElement<HTMLElement>(root, "#replay-loading-summary");
  replayLoadingActive = mustElement<HTMLElement>(root, "#replay-loading-active");
  replayLoadingList = mustElement<HTMLDivElement>(root, "#replay-loading-list");
  mechanicsReviewReplayLoadsController = createMechanicsReviewReplayLoadsController({
    elements: {
      reviewSummary: mechanicsReviewReplayLoadSummary,
      loadingSummary: replayLoadingSummary,
      loadingActive: replayLoadingActive,
      loadingList: replayLoadingList,
    },
    isActiveReview(review) {
      return activeMechanicsReview === review;
    },
    onActiveLoadProgress(progress) {
      statusReadout.textContent = formatReplayLoadProgress(progress);
      replayLoadModal?.update(progress);
    },
  });
  mechanicsReviewCount = mustElement<HTMLElement>(root, "#mechanics-review-count");
  mechanicsReviewList = mustElement<HTMLDivElement>(root, "#mechanics-review-list");
  boostPickupFiltersWindowBody = mustElement<HTMLDivElement>(
    root,
    "#boost-pickup-filters-window-body",
  );
  touchControlsWindowBody = mustElement<HTMLDivElement>(root, "#touch-controls-window-body");
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
    activeMechanicsReview = null;
    mechanicsReviewBoundaryGuard = false;
    mechanicsReviewReplayLoadsController = null;
    boostPadOverlayEnabled = true;
    loadedReplayName = null;
    cameraControlsController = null;
    recordingWindowController = null;
    initialUrlConfig = null;
    if (configUrlUpdateTimer !== null) {
      window.clearTimeout(configUrlUpdateTimer);
      configUrlUpdateTimer = null;
    }
    isApplyingConfig = false;
    nextWindowZIndex = 30;
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
        if (activeMechanicsReview) {
          activeMechanicsReview.currentClip = null;
          activeMechanicsReview.currentReplayId = null;
          renderMechanicsReviewWindow();
        }
        await loadReplay(createFileReplaySource(file));
      } catch (error) {
        console.error("Failed to load replay:", error);
        statusReadout.textContent =
          error instanceof Error ? error.message : "Failed to load replay";
      }
    },
    { signal: listeners.signal },
  );

  mechanicsReviewFile.addEventListener(
    "change",
    async () => {
      const file = mechanicsReviewFile.files?.[0];
      if (!file) return;

      try {
        const manifest = parseMechanicsReviewPlaylistJson(await file.text());
        await loadMechanicsReviewPlaylist(manifest, null);
      } catch (error) {
        console.error("Failed to load mechanics review playlist:", error);
        setMechanicsReviewStatus(
          error instanceof Error ? error.message : "Failed to load mechanics review playlist",
        );
      } finally {
        mechanicsReviewFile.value = "";
      }
    },
    { signal: listeners.signal },
  );

  mechanicsReviewLoadUrl.addEventListener(
    "click",
    () => {
      void loadMechanicsReviewPlaylistFromUrl(mechanicsReviewUrl.value.trim()).catch((error) => {
        console.error("Failed to load mechanics review playlist URL:", error);
        setMechanicsReviewStatus(
          error instanceof Error ? error.message : "Failed to load mechanics review playlist URL",
        );
      });
    },
    { signal: listeners.signal },
  );

  mechanicsReviewPrev.addEventListener(
    "click",
    () => {
      const review = activeMechanicsReview;
      if (review) {
        void activateMechanicsReviewItem(Math.max(0, review.currentIndex - 1));
      }
    },
    { signal: listeners.signal },
  );

  mechanicsReviewReplay.addEventListener("click", replayMechanicsReviewClip, {
    signal: listeners.signal,
  });

  mechanicsReviewNext.addEventListener(
    "click",
    () => {
      const review = activeMechanicsReview;
      if (review) {
        void activateMechanicsReviewItem(
          Math.min(review.manifest.items.length - 1, review.currentIndex + 1),
        );
      }
    },
    { signal: listeners.signal },
  );

  mechanicsReviewConfirm.addEventListener(
    "click",
    () => {
      void submitMechanicsReviewDecision("confirmed");
    },
    { signal: listeners.signal },
  );

  mechanicsReviewReject.addEventListener(
    "click",
    () => {
      void submitMechanicsReviewDecision("rejected");
    },
    { signal: listeners.signal },
  );

  mechanicsReviewUncertain.addEventListener(
    "click",
    () => {
      void submitMechanicsReviewDecision("uncertain");
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
  renderMechanicsReviewWindow();
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
    mechanicsReviewUrl.value = reviewUrl;
    showWindow("mechanics-review");
    void loadMechanicsReviewPlaylistFromUrl(reviewUrl).catch((error) => {
      if (listeners.signal.aborted) {
        return;
      }
      console.error("Failed to load mechanics review playlist from URL:", error);
      setMechanicsReviewStatus(
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
