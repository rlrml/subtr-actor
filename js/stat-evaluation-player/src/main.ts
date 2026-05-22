import "./styles.css";
import {
  createBallchasingOverlayPlugin,
  createBoostPadOverlayPlugin,
  createBoostPickupAnimationPlugin,
  createCanvasRecorderPlugin,
  createTimelineOverlayPlugin,
  ReplayPlayer,
} from "subtr-actor-player";
import type {
  BoostPickupAnimationPickup,
  CanvasRecorderPlugin,
  CanvasRecorderStatus,
  CameraSettings,
  ReplayCameraViewMode,
  ReplayFreeCameraPreset,
  ReplayPlayerState,
  ReplayPlayerTrack,
  TimelineOverlayPlugin,
} from "subtr-actor-player";
import { getAppTemplate } from "./appTemplate.ts";
import { createReplayLoadModal } from "./replayLoadModal.ts";
import type { ReplayLoadModalController } from "./replayLoadModal.ts";
import {
  createStatModules,
  getTeamClass,
  RELATIVE_POSITIONING_MODULE_ID,
} from "./statModules.ts";
import type { StatModule, StatModuleContext } from "./statModules.ts";
import { createBoostPickupFilterController } from "./boostPickupFilters.ts";
import {
  createStatsFrameLookup,
  getStatsFrameForReplayFrame,
} from "./statsTimeline.ts";
import {
  applyConfigAdapterSnapshot,
  getConfigAdapterSnapshot,
  type StatsPlayerConfigAdapter,
} from "./configAdapters.ts";
import type {
  PlayerStatsSnapshot,
  StatsFrame,
  StatsTimeline,
  TeamStatsSnapshot,
} from "./statsTimeline.ts";
import {
  createStatRegistry,
  type StatDefinition,
  type StatScopeKind,
} from "./statRegistry.ts";
import { getStatDefinitionSearchMatches } from "./statSearch.ts";
import {
  countEnabledTimelineEvents,
  buildMechanicTimelineEvents,
  filterReplayTimelineEvents,
  formatMechanicKind,
  getMechanicKinds,
} from "./timelineMarkers.ts";
import { buildMechanicTimelineRanges } from "./timelineRanges.ts";
import {
  formatReplayLoadProgress,
  loadReplayBundleInWorker,
} from "./replayLoader.ts";
import {
  getReplayFetchRequestFromSearch,
  type ReplayFetchRequest,
} from "./replayUrl.ts";
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
  type TeamScope,
  type WindowPlacementConfig,
} from "./playerConfig.ts";
import { playerIdToString } from "./touchOverlay.ts";

const DEFAULT_CAMERA_DISTANCE_SCALE = 2.25;
const CAMERA_VIEW_MODES: ReplayCameraViewMode[] = [
  "free",
  "follow",
];

let replayPlayer: ReplayPlayer | null = null;
let timelineOverlay: TimelineOverlayPlugin | null = null;
let canvasRecorder: CanvasRecorderPlugin | null = null;
let statsTimeline: StatsTimeline | null = null;
let statsFrameLookup: Map<number, StatsFrame> | null = null;
let unsubscribe: (() => void) | null = null;
let removeRenderHook: (() => void) | null = null;

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

const MODULES = createStatModules({
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
}, {
  boostPickupFilters,
});

let activeModules: StatModule[] = [];
let activeTimelineEventModuleIds = new Set<string>();
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
const MECHANIC_EVENT_SOURCE_ID = "mechanics:events";
const MECHANIC_RANGE_SOURCE_ID = "mechanics:ranges";

export interface StatEvaluationPlayerHandle {
  readonly root: HTMLElement;
  destroy(): void;
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
let mechanicsTimelineWindowBody!: HTMLDivElement;
let mechanicsReviewFile!: HTMLInputElement;
let mechanicsReviewUrl!: HTMLInputElement;
let mechanicsReviewLoadUrl!: HTMLButtonElement;
let mechanicsReviewStatus!: HTMLElement;
let mechanicsReviewIndex!: HTMLElement;
let mechanicsReviewTitle!: HTMLElement;
let mechanicsReviewMechanic!: HTMLElement;
let mechanicsReviewPlayer!: HTMLElement;
let mechanicsReviewReason!: HTMLElement;
let mechanicsReviewPrev!: HTMLButtonElement;
let mechanicsReviewReplay!: HTMLButtonElement;
let mechanicsReviewNext!: HTMLButtonElement;
let mechanicsReviewCount!: HTMLElement;
let mechanicsReviewList!: HTMLDivElement;
let boostPickupFiltersWindowBody!: HTMLDivElement;
let touchControlsWindowBody!: HTMLDivElement;
let statsWindowLayer!: HTMLDivElement;
let togglePlayback!: HTMLButtonElement;
let playbackRate!: HTMLSelectElement;
let attachedPlayer!: HTMLSelectElement;
let cameraViewFreeButton!: HTMLButtonElement;
let cameraViewFollowButton!: HTMLButtonElement;
let cameraViewOverheadButton!: HTMLButtonElement;
let cameraViewSideButton!: HTMLButtonElement;
let cameraDistance!: HTMLInputElement;
let cameraDistanceReadout!: HTMLElement;
let customCameraSettings!: HTMLInputElement;
let cameraSettingsControls!: HTMLDivElement;
let customCameraFov!: HTMLInputElement;
let customCameraHeight!: HTMLInputElement;
let customCameraPitch!: HTMLInputElement;
let customCameraDistance!: HTMLInputElement;
let customCameraStiffness!: HTMLInputElement;
let customCameraSwivelSpeed!: HTMLInputElement;
let customCameraTransitionSpeed!: HTMLInputElement;
let customCameraFovReadout!: HTMLElement;
let customCameraHeightReadout!: HTMLElement;
let customCameraPitchReadout!: HTMLElement;
let customCameraDistanceReadout!: HTMLElement;
let customCameraStiffnessReadout!: HTMLElement;
let customCameraSwivelSpeedReadout!: HTMLElement;
let customCameraTransitionSpeedReadout!: HTMLElement;
let ballCam!: HTMLInputElement;
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
let cameraProfileReadout!: HTMLElement;
let cameraFovReadout!: HTMLElement;
let cameraHeightReadout!: HTMLElement;
let cameraPitchReadout!: HTMLElement;
let cameraBaseDistanceReadout!: HTMLElement;
let cameraStiffnessReadout!: HTMLElement;
let skipPostGoalTransitions!: HTMLInputElement;
let replayLoadModal: ReplayLoadModalController | null = null;
let skipKickoffs!: HTMLInputElement;
let recordingFps!: HTMLInputElement;
let recordingPlaybackRate!: HTMLSelectElement;
let recordingStart!: HTMLButtonElement;
let recordingFullReplay!: HTMLButtonElement;
let recordingStop!: HTMLButtonElement;
let recordingDownload!: HTMLButtonElement;
let recordingClear!: HTMLButtonElement;
let recordingStatus!: HTMLElement;
let recordingElapsed!: HTMLElement;
let recordingSize!: HTMLElement;
let recordingType!: HTMLElement;
let currentMountCleanup: (() => void) | null = null;
let statRegistry: StatDefinition[] = createStatRegistry(null);
let nextWindowZIndex = 30;
let nextStatsWindowId = 1;
let boostPadOverlayEnabled = true;
let loadedReplayName: string | null = null;
let lastFreeCameraPreset: ReplayFreeCameraPreset | null = null;
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
  "playback",
  "recording",
  "mechanics",
  "mechanics-review",
  "boost-pickups",
  "touch-controls",
];

type MechanicsReviewPlaybackBound =
  | { kind: "time"; value: number }
  | { kind: "frame"; value: number };

interface MechanicsReviewReplay {
  id: string;
  path?: string;
  label?: string;
  locator?: Record<string, unknown>;
  meta?: Record<string, unknown>;
}

interface MechanicsReviewItemMeta {
  confidence?: number | null;
  mechanic?: string;
  mechanicLabel?: string;
  playerId?: string;
  playerName?: string | null;
  reason?: string;
  target?: Record<string, unknown>;
  followupGoal?: unknown;
  [key: string]: unknown;
}

interface MechanicsReviewItem {
  id?: string;
  replay: string;
  start: MechanicsReviewPlaybackBound;
  end: MechanicsReviewPlaybackBound;
  label?: string;
  meta?: MechanicsReviewItemMeta;
}

interface MechanicsReviewPlaylist {
  label?: string;
  replays?: MechanicsReviewReplay[];
  items: MechanicsReviewItem[];
  playback?: unknown;
  meta?: unknown;
}

interface ActiveMechanicsReview {
  manifest: MechanicsReviewPlaylist;
  sourceUrl: string | null;
  replaysById: Map<string, MechanicsReviewReplay>;
  currentIndex: number;
  loading: boolean;
  currentReplayId: string | null;
  currentClip: { startTime: number; endTime: number } | null;
}

interface SelectedStatEntry {
  key: string;
  statId: string;
  targetId?: string;
}

interface StatsWindowState {
  readonly id: string;
  readonly kind: StatsWindowKind;
  readonly entries: SelectedStatEntry[];
  playerId: string | null;
  team: TeamScope | null;
  pickerOpen: boolean;
  query: string;
  element: HTMLElement;
  body: HTMLElement;
}

const statsWindows = new Map<string, StatsWindowState>();
let activeMechanicsReview: ActiveMechanicsReview | null = null;
let mechanicsReviewBoundaryGuard = false;

function getActiveModuleIds(): Set<string> {
  return new Set([
    ...activeTimelineEventModuleIds,
    ...activeTimelineRangeModuleIds,
    ...activeRenderEffectModuleIds,
  ]);
}

function getActiveModuleSignature(): string {
  return [
    `events=${[...activeTimelineEventModuleIds].sort().join(",")}`,
    `ranges=${[...activeTimelineRangeModuleIds].sort().join(",")}`,
    `mechanics=${[...activeMechanicTimelineKinds].sort().join(",")}`,
    `effects=${[...activeRenderEffectModuleIds].sort().join(",")}`,
  ].join("|");
}

function getActiveCapabilityIds(kind: ModuleCapabilityKind): Set<string> {
  return kind === "events"
    ? activeTimelineEventModuleIds
    : kind === "ranges"
      ? activeTimelineRangeModuleIds
      : activeRenderEffectModuleIds;
}

function clearRenderCaches(): void {
}

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

  const activeModuleIds = getActiveModuleIds();
  activeModules = MODULES.filter((mod) => activeModuleIds.has(mod.id));
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

function toggleCapability(
  id: string,
  kind: ModuleCapabilityKind,
  enabled: boolean,
): void {
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

  for (const mod of activeModules) {
    if (!activeTimelineEventModuleIds.has(mod.id)) {
      continue;
    }
    const events = mod.getTimelineEvents?.(ctx);
    if (!events || events.length === 0) {
      continue;
    }

    timelineSourceRemovers.set(mod.id, timelineOverlay.addEventSource(events));
  }

  const mechanicEvents = buildMechanicTimelineEvents(
    ctx.statsTimeline,
    ctx.replay,
    activeMechanicTimelineKinds,
  );
  if (mechanicEvents.length > 0) {
    timelineSourceRemovers.set(
      MECHANIC_EVENT_SOURCE_ID,
      timelineOverlay.addEventSource(mechanicEvents),
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

  const mechanicRanges = buildMechanicTimelineRanges(
    ctx.statsTimeline,
    ctx.replay,
    activeMechanicTimelineKinds,
  );
  if (mechanicRanges.length > 0) {
    timelineRangeSourceRemovers.set(
      MECHANIC_RANGE_SOURCE_ID,
      timelineOverlay.addRangeSource(mechanicRanges),
    );
  }

  timelineOverlay.refreshRanges();
}

function renderTimelineEventCount(): void {
  if (!replayPlayer || !statsTimeline) {
    eventsReadout.textContent = "--";
    return;
  }

  const mechanicEventCount = buildMechanicTimelineEvents(
    statsTimeline,
    replayPlayer.replay,
    activeMechanicTimelineKinds,
  ).length;
  const mechanicRangeCount = buildMechanicTimelineRanges(
    statsTimeline,
    replayPlayer.replay,
    activeMechanicTimelineKinds,
  ).length;

  eventsReadout.textContent = `${countEnabledTimelineEvents(
    activeTimelineEventModuleIds,
    replayPlayer.replay,
    statsTimeline,
  ) + mechanicEventCount + mechanicRangeCount}`;
}

function mustElement<T extends HTMLElement>(
  root: ParentNode,
  selector: string,
): T {
  const element = root.querySelector(selector);
  if (!(element instanceof HTMLElement)) {
    throw new Error(`Missing element for selector: ${selector}`);
  }

  return element as T;
}

function getElementWindowId(element: HTMLElement): string | null {
  return element.closest<HTMLElement>("[data-window-id]")?.dataset.windowId ??
    null;
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

function applyWindowPlacement(
  windowEl: HTMLElement,
  placement: WindowPlacementConfig,
): void {
  const mapped = mapWindowPlacementToViewport(
    placement,
    getCurrentViewportSize(),
  );
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
  return MODULES
    .filter((mod) => mod.getConfig || mod.applyConfig)
    .map((mod) => {
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

function getStatsWindowConfig(statsWindow: StatsWindowState): StatsWindowConfig {
  return {
    id: statsWindow.id,
    kind: statsWindow.kind,
    placement: readWindowPlacement(statsWindow.element),
    playerId: statsWindow.playerId,
    team: statsWindow.team,
    entries: statsWindow.entries.map((entry) => ({
      statId: entry.statId,
      targetId: entry.targetId,
    })),
  };
}

function getPlaybackConfigSnapshot(): PlayerPlaybackConfig {
  const state = replayPlayer?.getState();
  return {
    currentTime: state?.currentTime,
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
    freePreset: lastFreeCameraPreset,
    attachedPlayerId: state?.attachedPlayerId,
    distanceScale: state?.cameraDistanceScale,
    ballCam: state?.ballCamEnabled,
    customSettings: state?.customCameraSettings,
  };
}

function getRecordingConfigSnapshot(): RecordingConfig {
  return {
    fps: Number(recordingFps?.value),
    playbackRate: Number(recordingPlaybackRate?.value),
  };
}

function getStatsPlayerConfigSnapshot(): StatsPlayerConfig {
  return {
    version: STATS_PLAYER_CONFIG_VERSION,
    playback: getPlaybackConfigSnapshot(),
    camera: getCameraConfigSnapshot(),
    overlays: {
      timelineEvents: [...activeTimelineEventModuleIds],
      timelineRanges: [...activeTimelineRangeModuleIds],
      mechanics: [...activeMechanicTimelineKinds],
      renderEffects: [...activeRenderEffectModuleIds],
      followedPlayerHud: false,
      boostPads: boostPadOverlayEnabled,
      boostPickupAnimation: replayPlayer?.getState().boostPickupAnimationEnabled ?? false,
    },
    recording: getRecordingConfigSnapshot(),
    singletonWindows: getSingletonWindowConfigs(),
    statsWindows: [...statsWindows.values()].map(getStatsWindowConfig),
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
    const element = root.querySelector<HTMLElement>(
      `[data-window-id="${windowConfig.id}"]`,
    );
    if (element) {
      applyWindowPlacement(element, windowConfig.placement);
    }
  }
}

function applyConfigToStaticControls(config: StatsPlayerConfig): void {
  activeTimelineEventModuleIds = new Set(config.overlays.timelineEvents);
  activeTimelineRangeModuleIds = new Set(config.overlays.timelineRanges);
  activeMechanicTimelineKinds = new Set(config.overlays.mechanics);
  activeRenderEffectModuleIds = new Set(config.overlays.renderEffects);
  boostPadOverlayEnabled = config.overlays.boostPads;
  skipPostGoalTransitions.checked =
    config.playback.skipPostGoalTransitions ?? skipPostGoalTransitions.checked;
  skipKickoffs.checked = config.playback.skipKickoffs ?? skipKickoffs.checked;
  if (config.playback.rate !== undefined) {
    playbackRate.value = `${config.playback.rate}`;
  }
  if (config.recording.fps !== undefined) {
    recordingFps.value = `${config.recording.fps}`;
  }
  if (config.recording.playbackRate !== undefined) {
    recordingPlaybackRate.value = `${config.recording.playbackRate}`;
  }
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

function applyConfigToReplayPlayer(config: StatsPlayerConfig): void {
  if (!replayPlayer) {
    return;
  }
  replayPlayer.setState(
    getReplayPlayerStatePatchFromConfig(config.playback, config.camera, config),
  );
  lastFreeCameraPreset = config.camera.freePreset ?? null;
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
  const windowEl = mustElement<HTMLElement>(
    appRoot ?? document,
    `[data-window-id="${id}"]`,
  );
  windowEl.hidden = false;
  bringWindowToFront(windowEl);
  scheduleConfigUrlUpdate();
}

function toggleWindow(id: SingletonWindowId): void {
  const windowEl = mustElement<HTMLElement>(
    appRoot ?? document,
    `[data-window-id="${id}"]`,
  );
  windowEl.hidden = !windowEl.hidden;
  if (!windowEl.hidden) {
    bringWindowToFront(windowEl);
  }
  scheduleConfigUrlUpdate();
}

function hideWindow(id: string): void {
  const windowEl = mustElement<HTMLElement>(
    appRoot ?? document,
    `[data-window-id="${id}"]`,
  );
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
  return target instanceof Element && Boolean(target.closest(
    "button, input, select, textarea, option, label, a, [data-no-drag]",
  ));
}

function installWindowDragging(root: HTMLElement, signal: AbortSignal): void {
  root.addEventListener("pointerdown", (event) => {
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
  }, { signal });
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
      timelineToggles.push(renderCapabilityToggle(
        mod.id,
        getCapabilityLabel(mod, "events"),
        "events",
      ));
    }
    if (mod.getTimelineRanges) {
      timelineToggles.push(renderCapabilityToggle(
        mod.id,
        getCapabilityLabel(mod, "ranges"),
        "ranges",
      ));
    }
    if (hasRenderEffect) {
      inGameVisualizationToggles.push(renderCapabilityToggle(
        mod.id,
        getCapabilityLabel(mod, "effects"),
        "effects",
      ));
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
  mechanicsTimelineWindowBody.replaceChildren();

  const kinds = getMechanicKinds(statsTimeline);
  const counts = new Map<string, number>();
  for (const event of statsTimeline?.events.mechanics ?? []) {
    counts.set(event.kind, (counts.get(event.kind) ?? 0) + 1);
  }

  if (kinds.length === 0) {
    const empty = document.createElement("p");
    empty.className = "stat-window-empty";
    empty.textContent = "No mechanic events loaded.";
    mechanicsTimelineWindowBody.append(empty);
    return;
  }

  const actions = document.createElement("div");
  actions.className = "mechanics-actions";

  const allButton = document.createElement("button");
  allButton.type = "button";
  allButton.className = "module-summary-item";
  allButton.addEventListener("click", () => {
    activeMechanicTimelineKinds = new Set(kinds);
    syncTimelineEvents();
    syncTimelineRanges();
    renderMechanicsTimelineControls();
    renderTimelineEventCount();
    scheduleConfigUrlUpdate();
  });
  const allName = document.createElement("span");
  allName.textContent = "All mechanics";
  const allCount = document.createElement("strong");
  allCount.textContent = `${kinds.length}`;
  allButton.append(allName, allCount);

  const noneButton = document.createElement("button");
  noneButton.type = "button";
  noneButton.className = "module-summary-item";
  noneButton.addEventListener("click", () => {
    activeMechanicTimelineKinds.clear();
    syncTimelineEvents();
    syncTimelineRanges();
    renderMechanicsTimelineControls();
    renderTimelineEventCount();
    scheduleConfigUrlUpdate();
  });
  const noneName = document.createElement("span");
  noneName.textContent = "No mechanics";
  const noneState = document.createElement("strong");
  noneState.textContent = "Off";
  noneButton.append(noneName, noneState);

  actions.append(allButton, noneButton);

  const list = document.createElement("div");
  list.className = "module-list mechanics-list";
  for (const kind of kinds) {
    const active = activeMechanicTimelineKinds.has(kind);
    const item = document.createElement("button");
    item.type = "button";
    item.className = "module-summary-item";
    item.dataset.active = active ? "true" : "false";
    item.setAttribute("aria-pressed", active ? "true" : "false");
    item.addEventListener("click", () => {
      if (activeMechanicTimelineKinds.has(kind)) {
        activeMechanicTimelineKinds.delete(kind);
      } else {
        activeMechanicTimelineKinds.add(kind);
      }
      syncTimelineEvents();
      syncTimelineRanges();
      renderMechanicsTimelineControls();
      renderTimelineEventCount();
      scheduleConfigUrlUpdate();
    });

    const name = document.createElement("span");
    name.textContent = formatMechanicKind(kind);

    const state = document.createElement("strong");
    state.textContent = `${active ? "On" : "Off"} ${counts.get(kind) ?? 0}`;

    item.append(name, state);
    list.append(item);
  }

  mechanicsTimelineWindowBody.append(actions, list);
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function parseMechanicsReviewBound(value: unknown): MechanicsReviewPlaybackBound | null {
  if (!isRecord(value)) {
    return null;
  }
  if (
    (value.kind === "time" || value.kind === "frame") &&
    typeof value.value === "number" &&
    Number.isFinite(value.value)
  ) {
    return {
      kind: value.kind,
      value: value.value,
    };
  }
  return null;
}

function parseMechanicsReviewPlaylist(value: unknown): MechanicsReviewPlaylist {
  if (!isRecord(value) || !Array.isArray(value.items)) {
    throw new Error("Review playlist must contain an items array.");
  }

  const items = value.items.map((rawItem, index): MechanicsReviewItem => {
    if (!isRecord(rawItem) || typeof rawItem.replay !== "string") {
      throw new Error(`Invalid review item at index ${index}.`);
    }
    const start = parseMechanicsReviewBound(rawItem.start);
    const end = parseMechanicsReviewBound(rawItem.end);
    if (!start || !end) {
      throw new Error(`Review item ${index + 1} has invalid start or end.`);
    }
    return {
      id: typeof rawItem.id === "string" ? rawItem.id : undefined,
      replay: rawItem.replay,
      start,
      end,
      label: typeof rawItem.label === "string" ? rawItem.label : undefined,
      meta: isRecord(rawItem.meta) ? rawItem.meta : undefined,
    };
  });

  const replays = Array.isArray(value.replays)
    ? value.replays
      .map((rawReplay): MechanicsReviewReplay | null => {
        if (!isRecord(rawReplay) || typeof rawReplay.id !== "string") {
          return null;
        }
        return {
          id: rawReplay.id,
          path: typeof rawReplay.path === "string" ? rawReplay.path : undefined,
          label: typeof rawReplay.label === "string" ? rawReplay.label : undefined,
          locator: isRecord(rawReplay.locator) ? rawReplay.locator : undefined,
          meta: isRecord(rawReplay.meta) ? rawReplay.meta : undefined,
        };
      })
      .filter((replay): replay is MechanicsReviewReplay => replay !== null)
    : undefined;

  return {
    label: typeof value.label === "string" ? value.label : undefined,
    replays,
    items,
    playback: value.playback,
    meta: value.meta,
  };
}

function parseMechanicsReviewPlaylistJson(text: string): MechanicsReviewPlaylist {
  let parsed: unknown;
  try {
    parsed = JSON.parse(text);
  } catch (error) {
    throw new Error(
      `Invalid review playlist JSON: ${
        error instanceof Error ? error.message : String(error)
      }`,
    );
  }
  return parseMechanicsReviewPlaylist(parsed);
}

function getMechanicsReviewUrlFromLocation(): string | null {
  const params = new URLSearchParams(window.location.search);
  return params.get("reviewPlaylist")?.trim() ||
    params.get("review")?.trim() ||
    params.get("playlistUrl")?.trim() ||
    null;
}

function resolveMechanicsReviewUrl(value: string, sourceUrl: string | null): string {
  const path = value.startsWith("path:") ? value.slice("path:".length) : value;
  if (/^https?:\/\//i.test(path) || path.startsWith("/@fs/")) {
    return path;
  }
  if (path.startsWith("/")) {
    return `/@fs${path}`;
  }
  return sourceUrl ? new URL(path, sourceUrl).href : path;
}

function getMechanicsReviewReplayPath(
  item: MechanicsReviewItem,
  review: ActiveMechanicsReview,
): string {
  const replay = review.replaysById.get(item.replay);
  if (replay?.path) {
    return replay.path;
  }
  if (isRecord(replay?.locator) && replay.locator.kind === "path" &&
    typeof replay.locator.path === "string") {
    return replay.locator.path;
  }
  if (/^https?:\/\//i.test(item.replay) || item.replay.startsWith("/") ||
    item.replay.startsWith("/@fs/") || item.replay.startsWith("path:")) {
    return item.replay;
  }
  throw new Error(`Review replay "${item.replay}" does not include a loadable path.`);
}

function getMechanicsReviewReplayLabel(
  item: MechanicsReviewItem,
  review: ActiveMechanicsReview,
): string {
  const replay = review.replaysById.get(item.replay);
  const rawPath = replay?.path ?? getMechanicsReviewReplayPath(item, review);
  const fileName = rawPath.replace(/^path:/, "").split("/").filter(Boolean).pop();
  return replay?.label ?? fileName ?? "review replay";
}

function createMechanicsReviewReplaySource(
  item: MechanicsReviewItem,
  review: ActiveMechanicsReview,
  signal?: AbortSignal,
): ReplayInputSource {
  const replayPath = getMechanicsReviewReplayPath(item, review);
  const url = resolveMechanicsReviewUrl(replayPath, review.sourceUrl);
  return {
    name: getMechanicsReviewReplayLabel(item, review),
    preparingStatus: "Loading review replay...",
    async readBytes() {
      const response = await fetch(url, { signal });
      if (!response.ok) {
        const statusText = response.statusText ? ` ${response.statusText}` : "";
        throw new Error(
          `Failed to fetch review replay from ${url} (${response.status}${statusText})`,
        );
      }
      return new Uint8Array(await response.arrayBuffer());
    },
  };
}

function getMechanicsReviewBoundTime(bound: MechanicsReviewPlaybackBound): number {
  if (bound.kind === "time") {
    return bound.value;
  }
  const frameIndex = Math.max(0, Math.trunc(bound.value));
  return replayPlayer?.replay.frames[frameIndex]?.time ??
    replayPlayer?.replay.frames.at(-1)?.time ??
    0;
}

function getMechanicsReviewItemLabel(item: MechanicsReviewItem, index: number): string {
  return item.label ?? item.meta?.mechanicLabel ?? `Review item ${index + 1}`;
}

function getMechanicsReviewPlayerId(item: MechanicsReviewItem): string | null {
  if (typeof item.meta?.playerId === "string") {
    return item.meta.playerId;
  }
  if (isRecord(item.meta?.target) && typeof item.meta.target.playerId === "string") {
    return item.meta.target.playerId;
  }
  return null;
}

function getMechanicsReviewPlayerName(item: MechanicsReviewItem): string {
  if (typeof item.meta?.playerName === "string" && item.meta.playerName.trim()) {
    return item.meta.playerName;
  }
  const playerId = getMechanicsReviewPlayerId(item);
  return playerId
    ? replayPlayer?.replay.players.find((player) => player.id === playerId)?.name ??
      playerId
    : "--";
}

function getMechanicsReviewMechanicLabel(item: MechanicsReviewItem): string {
  if (typeof item.meta?.mechanicLabel === "string" && item.meta.mechanicLabel.trim()) {
    return item.meta.mechanicLabel;
  }
  return typeof item.meta?.mechanic === "string"
    ? formatMechanicKind(item.meta.mechanic)
    : "--";
}

function setMechanicsReviewStatus(message: string): void {
  if (mechanicsReviewStatus) {
    mechanicsReviewStatus.textContent = message;
  }
}

function renderMechanicsReviewWindow(): void {
  if (!mechanicsReviewList) {
    return;
  }

  const review = activeMechanicsReview;
  const items = review?.manifest.items ?? [];
  const item = review ? items[review.currentIndex] ?? null : null;
  const hasItems = items.length > 0;

  mechanicsReviewCount.textContent = `${items.length} item${items.length === 1 ? "" : "s"}`;
  mechanicsReviewIndex.textContent = hasItems && review
    ? `${review.currentIndex + 1} / ${items.length}`
    : "0 / 0";
  mechanicsReviewTitle.textContent = item
    ? getMechanicsReviewItemLabel(item, review?.currentIndex ?? 0)
    : "No candidate selected";
  mechanicsReviewMechanic.textContent = item
    ? getMechanicsReviewMechanicLabel(item)
    : "--";
  mechanicsReviewPlayer.textContent = item
    ? getMechanicsReviewPlayerName(item)
    : "--";
  mechanicsReviewReason.textContent = item?.meta?.reason ?? "--";
  mechanicsReviewPrev.disabled = !review || review.loading || review.currentIndex <= 0;
  mechanicsReviewReplay.disabled = !review || review.loading || !review.currentClip;
  mechanicsReviewNext.disabled = !review || review.loading ||
    review.currentIndex >= items.length - 1;

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
    meta.textContent = getMechanicsReviewMechanicLabel(candidate);

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
    currentIndex: 0,
    loading: false,
    currentReplayId: null,
    currentClip: null,
  };
  setMechanicsReviewStatus(
    manifest.label
      ? `Loaded ${manifest.label}.`
      : `Loaded review playlist.`,
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
      await loadReplay(createMechanicsReviewReplaySource(item, review));
      review.currentReplayId = item.replay;
    }

    const startTime = Math.max(0, getMechanicsReviewBoundTime(item.start));
    const endTime = Math.min(
      replayPlayer?.getState().duration ?? Number.POSITIVE_INFINITY,
      Math.max(startTime, getMechanicsReviewBoundTime(item.end)),
    );
    if (!Number.isFinite(startTime) || !Number.isFinite(endTime) ||
      endTime <= startTime) {
      throw new Error("Review item has an empty playback range.");
    }

    const playerId = getMechanicsReviewPlayerId(item);
    if (playerId && replayPlayer?.replay.players.some((player) => player.id === playerId)) {
      replayPlayer.setAttachedPlayer(playerId);
      replayPlayer.setCameraViewMode("follow");
      lastFreeCameraPreset = null;
    }

    skipPostGoalTransitions.checked = false;
    review.currentClip = { startTime, endTime };
    replayPlayer?.setState({
      currentTime: startTime,
      playing: false,
      skipPostGoalTransitionsEnabled: false,
    });
    setMechanicsReviewStatus(
      `${startTime.toFixed(2)}s to ${endTime.toFixed(2)}s`,
    );
  } catch (error) {
    console.error("Failed to activate mechanics review item:", error);
    review.currentClip = null;
    setMechanicsReviewStatus(
      error instanceof Error ? error.message : "Failed to load review item",
    );
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
  });
}

function enforceMechanicsReviewClipBoundary(state: ReplayPlayerState): boolean {
  const clip = activeMechanicsReview?.currentClip;
  if (!clip || !replayPlayer || mechanicsReviewBoundaryGuard) {
    return false;
  }

  const beforeStart = state.currentTime < clip.startTime - 0.1;
  const atOrPastEnd = state.currentTime >= clip.endTime - 0.025;
  if (!beforeStart && !atOrPastEnd) {
    return false;
  }

  mechanicsReviewBoundaryGuard = true;
  try {
    replayPlayer.setState({
      currentTime: beforeStart ? clip.startTime : clip.endTime,
      playing: false,
      skipPostGoalTransitionsEnabled: false,
    });
  } finally {
    mechanicsReviewBoundaryGuard = false;
  }
  return true;
}

function renderModuleSummaryGroup(
  title: string,
  items: HTMLButtonElement[],
): HTMLElement {
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

function getCapabilityLabel(
  mod: StatModule,
  kind: ModuleCapabilityKind,
): string {
  const timelineLabels: Record<string, string> = {
    "absolute-positioning:ranges": "Position zones",
    "backboard:events": "Backboard",
    "ball-carry:events": "Ball carry",
    "boost:ranges": "Boost pickup timeline",
    "ceiling-shot:events": "Ceiling shot",
    "demo:events": "Demo",
    "dodge-reset:events": "Dodge reset",
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
  return statsFrameLookup
    ? getStatsFrameForReplayFrame(statsFrameLookup, frameIndex)
    : null;
}

function getTeamSnapshot(
  frame: StatsFrame,
  team: TeamScope,
): TeamStatsSnapshot | null {
  return team === "blue" ? frame.team_zero ?? null : frame.team_one ?? null;
}

function getTeamLabel(team: TeamScope): string {
  return team === "blue" ? "Blue" : "Orange";
}

function getPlayerLabel(playerId: string | null): string {
  if (!playerId || !replayPlayer) {
    return "Select player";
  }
  return replayPlayer.replay.players.find((player) => player.id === playerId)
    ?.name ?? "Unknown player";
}

function getPlayerTeamClass(playerId: string | null | undefined): string | null {
  const player = replayPlayer?.replay.players.find((candidate) =>
    candidate.id === playerId
  );
  return player ? getTeamClass(player.isTeamZero) : null;
}

function getTeamScopeClass(team: TeamScope): string {
  return getTeamClass(team === "blue");
}

function appendGroupedPlayerOptions(
  select: HTMLSelectElement,
  selectedPlayerId: string | null | undefined,
): void {
  const players = replayPlayer?.replay.players ?? [];
  for (const team of ["blue", "orange"] as const) {
    const teamPlayers = players.filter((player) =>
      player.isTeamZero === (team === "blue")
    );
    if (teamPlayers.length === 0) {
      continue;
    }

    const group = document.createElement("optgroup");
    group.label = `${getTeamLabel(team)} team`;
    for (const player of teamPlayers) {
      group.append(
        new Option(
          player.name,
          player.id,
          player.id === selectedPlayerId,
          player.id === selectedPlayerId,
        ),
      );
    }
    select.append(group);
  }
}

function getStatsWindowScopeTeamClass(statsWindow: StatsWindowState): string | null {
  if (statsWindow.kind === "player") {
    return getPlayerTeamClass(statsWindow.playerId);
  }
  if (statsWindow.kind === "team") {
    return getTeamScopeClass(statsWindow.team ?? "blue");
  }
  return null;
}

function getStatTargetTeamClass(
  definition: StatDefinition,
  targetId: string | undefined,
): string | null {
  if (definition.scope === "player") {
    return getPlayerTeamClass(targetId);
  }
  return getTeamScopeClass(targetId === "orange" ? "orange" : "blue");
}

function getStatsWindowTitle(kind: StatsWindowKind): string {
  switch (kind) {
    case "player":
      return "Player stats";
    case "team":
      return "Team stats";
    case "all-players":
      return "All players stats";
    case "all-teams":
      return "All teams stats";
    case "ad-hoc":
      return "Ad hoc stats";
  }
}

function hasStatsWindowScopeSelector(kind: StatsWindowKind): boolean {
  return kind === "player" || kind === "team";
}

function getStatsWindowAllowedScope(kind: StatsWindowKind): StatScopeKind | null {
  switch (kind) {
    case "player":
    case "all-players":
      return "player";
    case "team":
    case "all-teams":
      return "team";
    case "ad-hoc":
      return null;
  }
}

function getStatsWindowDefaultPosition(): { x: number; y: number } {
  const offset = statsWindows.size * 18;
  return {
    x: Math.max(12, Math.min(window.innerWidth - 360, 96 + offset)),
    y: Math.max(64, Math.min(window.innerHeight - 240, 96 + offset)),
  };
}

function renderStatsWindows(
  frameIndex = replayPlayer?.getState().frameIndex ?? 0,
  options: { preserveOpenPickers?: boolean } = {},
): void {
  for (const statsWindow of statsWindows.values()) {
    if (
      options.preserveOpenPickers &&
      (statsWindow.pickerOpen ||
        statsWindow.element.contains(document.activeElement))
    ) {
      continue;
    }
    renderStatsWindow(statsWindow, frameIndex);
  }
}

function createStatsWindow(
  kind: StatsWindowKind,
  config?: StatsWindowConfig,
): StatsWindowState {
  const id = config?.id ?? `stats-${nextStatsWindowId++}`;
  const idNumber = Number.parseInt(id.replace(/^stats-/, ""), 10);
  if (Number.isFinite(idNumber)) {
    nextStatsWindowId = Math.max(nextStatsWindowId, idNumber + 1);
  }
  const { x, y } = getStatsWindowDefaultPosition();
  const element = document.createElement("section");
  element.className = "stats-window";
  element.dataset.windowId = id;
  element.style.setProperty("--window-x", `${x}px`);
  element.style.setProperty("--window-y", `${y}px`);
  if (config) {
    applyWindowPlacement(element, config.placement);
  }

  const header = document.createElement("header");
  header.className = "stats-window-header";

  const actions = document.createElement("div");
  actions.className = "stats-window-actions";
  const hideButton = document.createElement("button");
  hideButton.type = "button";
  hideButton.className = "stats-window-action";
  hideButton.textContent = "Hide";
  actions.append(hideButton);
  if (hasStatsWindowScopeSelector(kind)) {
    header.classList.add("stats-window-header-actions-only");
    header.append(actions);
  } else {
    const title = document.createElement("h2");
    title.textContent = getStatsWindowTitle(kind);
    header.append(title, actions);
  }

  const body = document.createElement("div");
  body.className = "stats-window-body";
  element.append(header, body);
  statsWindowLayer.append(element);

  const state: StatsWindowState = {
    id,
    kind,
    entries: config?.entries.map((entry) => ({
      key: `${id}:${entry.statId}:${entry.targetId ?? "scope"}`,
      statId: entry.statId,
      targetId: entry.targetId,
    })) ?? [],
    playerId: config?.playerId ?? replayPlayer?.replay.players[0]?.id ?? null,
    team: config?.team ?? "blue",
    pickerOpen: false,
    query: "",
    element,
    body,
  };

  hideButton.addEventListener("click", () => {
    element.hidden = true;
    scheduleConfigUrlUpdate();
  });

  statsWindows.set(id, state);
  if (!config) {
    bringWindowToFront(element);
  }
  setLauncherOpen(false);
  renderStatsWindow(state);
  scheduleConfigUrlUpdate();
  return state;
}

function replaceStatsWindowsFromConfig(configs: readonly StatsWindowConfig[]): void {
  for (const statsWindow of statsWindows.values()) {
    statsWindow.element.remove();
  }
  statsWindows.clear();
  nextStatsWindowId = 1;
  for (const config of configs) {
    createStatsWindow(config.kind, config);
  }
}

function renderStatsWindow(
  statsWindow: StatsWindowState,
  frameIndex = replayPlayer?.getState().frameIndex ?? 0,
): void {
  const activeElement = document.activeElement;
  const searchFocused = activeElement instanceof HTMLInputElement &&
    activeElement.dataset.statsWindowSearch === statsWindow.id;
  const searchSelectionStart = searchFocused ? activeElement.selectionStart : null;
  const searchSelectionEnd = searchFocused ? activeElement.selectionEnd : null;
  const searchSelectionDirection = searchFocused ? activeElement.selectionDirection : null;

  statsWindow.body.replaceChildren();

  renderStatsWindowScope(statsWindow);
  renderStatsWindowAddControl(statsWindow);
  renderStatsWindowPicker(statsWindow);
  renderStatsWindowEntries(statsWindow, frameIndex);

  if (searchFocused) {
    const searchInput = statsWindow.body.querySelector<HTMLInputElement>(
      `input[data-stats-window-search="${statsWindow.id}"]`,
    );
    searchInput?.focus({ preventScroll: true });
    if (
      searchInput &&
      searchSelectionStart !== null &&
      searchSelectionEnd !== null
    ) {
      searchInput.setSelectionRange(
        searchSelectionStart,
        searchSelectionEnd,
        searchSelectionDirection ?? "none",
      );
    }
  }
}

function renderStatsWindowScope(statsWindow: StatsWindowState): void {
  if (statsWindow.kind !== "player" && statsWindow.kind !== "team") {
    return;
  }

  const row = document.createElement("div");
  row.className = "stats-window-scope-row";

  const select = document.createElement("select");
  select.className = "stats-window-scope-select";
  const teamClass = getStatsWindowScopeTeamClass(statsWindow);
  if (teamClass) {
    select.classList.add(teamClass);
  }
  select.setAttribute(
    "aria-label",
    statsWindow.kind === "player" ? "Player stats target" : "Team stats target",
  );
  if (statsWindow.kind === "player") {
    appendGroupedPlayerOptions(select, statsWindow.playerId);
    select.value = statsWindow.playerId ?? "";
    select.addEventListener("change", () => {
      statsWindow.playerId = select.value || null;
      renderStatsWindow(statsWindow);
      scheduleConfigUrlUpdate();
    });
  } else {
    select.append(
      new Option("Blue", "blue", statsWindow.team === "blue", statsWindow.team === "blue"),
      new Option(
        "Orange",
        "orange",
        statsWindow.team === "orange",
        statsWindow.team === "orange",
      ),
    );
    select.value = statsWindow.team ?? "blue";
    select.addEventListener("change", () => {
      statsWindow.team = select.value === "orange" ? "orange" : "blue";
      renderStatsWindow(statsWindow);
      scheduleConfigUrlUpdate();
    });
  }

  row.append(select);
  statsWindow.body.append(row);
}

function renderStatsWindowAddControl(statsWindow: StatsWindowState): void {
  const button = document.createElement("button");
  button.type = "button";
  button.className = "stats-window-add-button";
  button.textContent = "+";
  button.title = "Add stat";
  button.setAttribute("aria-label", "Add stat");
  button.setAttribute("aria-expanded", String(statsWindow.pickerOpen));
  activateButton(button, () => {
    statsWindow.pickerOpen = !statsWindow.pickerOpen;
    renderStatsWindow(statsWindow);
  });

  if (hasStatsWindowScopeSelector(statsWindow.kind)) {
    const scopeRow = statsWindow.body.querySelector(".stats-window-scope-row");
    scopeRow?.append(button);
    return;
  }

  const toolbar = document.createElement("div");
  toolbar.className = "stats-window-toolbar";
  toolbar.append(button);
  statsWindow.body.append(toolbar);
}

function activateButton(button: HTMLButtonElement, callback: () => void): void {
  let pointerActivated = false;
  button.addEventListener("pointerdown", (event) => {
    if (button.disabled) {
      return;
    }
    pointerActivated = true;
    event.preventDefault();
    callback();
  });
  button.addEventListener("click", () => {
    if (pointerActivated) {
      pointerActivated = false;
      return;
    }
    if (!button.disabled) {
      callback();
    }
  });
}

function renderStatsWindowPicker(statsWindow: StatsWindowState): void {
  const picker = document.createElement("div");
  picker.className = "stats-window-picker";
  picker.hidden = !statsWindow.pickerOpen;
  if (picker.hidden) {
    statsWindow.body.append(picker);
    return;
  }

  const allowedScope = getStatsWindowAllowedScope(statsWindow.kind);
  const queryInput = document.createElement("input");
  queryInput.type = "search";
  queryInput.placeholder = "Search stats";
  queryInput.value = statsWindow.query;
  queryInput.dataset.statsWindowSearch = statsWindow.id;

  const list = document.createElement("div");
  list.className = "stats-window-picker-list";
  queryInput.addEventListener("input", () => {
    statsWindow.query = queryInput.value;
    renderStatsWindowPickerList(statsWindow, list, allowedScope);
  });

  renderStatsWindowPickerList(statsWindow, list, allowedScope);

  picker.append(queryInput, list);
  statsWindow.body.append(picker);
}

function renderStatsWindowPickerList(
  statsWindow: StatsWindowState,
  list: HTMLElement,
  allowedScope: StatScopeKind | null,
): void {
  list.replaceChildren();

  const scopeDefinitions = allowedScope
    ? statRegistry.filter((definition) => definition.scope === allowedScope)
    : statRegistry;
  const definitions = getStatDefinitionSearchMatches(
    scopeDefinitions,
    statsWindow.query,
  );

  const groupByCategory = new Map<string, StatDefinition[]>();
  for (const definition of definitions) {
    const group = groupByCategory.get(definition.category) ?? [];
    group.push(definition);
    groupByCategory.set(definition.category, group);
  }

  for (const [category, group] of groupByCategory) {
    if (group.length < 2) continue;
    const addGroup = document.createElement("button");
    addGroup.type = "button";
    addGroup.className = "stats-window-picker-item";
    addGroup.innerHTML = `<span>Add all ${category}</span><strong>${group.length}</strong>`;
    activateButton(addGroup, () => {
      for (const definition of group) {
        addStatToWindow(statsWindow, definition);
      }
      renderStatsWindow(statsWindow);
      scheduleConfigUrlUpdate();
    });
    list.append(addGroup);
  }

  for (const definition of definitions) {
    const item = document.createElement("button");
    item.type = "button";
    item.className = "stats-window-picker-item";
    item.innerHTML = `<span>${definition.label}</span><strong>${definition.scope}</strong>`;
    item.disabled = statsWindow.kind !== "ad-hoc" &&
      statsWindow.entries.some((entry) => entry.statId === definition.id);
    activateButton(item, () => {
      addStatToWindow(statsWindow, definition);
      renderStatsWindow(statsWindow);
      scheduleConfigUrlUpdate();
    });
    list.append(item);
  }

  if (definitions.length === 0) {
    const empty = document.createElement("p");
    empty.className = "stat-window-empty";
    empty.textContent = statRegistry.length === 0
      ? "No stats available."
      : "No matching stats.";
    list.append(empty);
  }
}

function addStatToWindow(
  statsWindow: StatsWindowState,
  definition: StatDefinition,
): void {
  const targetId = statsWindow.kind === "ad-hoc"
    ? getDefaultAdHocTargetId(definition)
    : undefined;
  if (statsWindow.entries.some((entry) =>
    entry.statId === definition.id && entry.targetId === targetId
  )) {
    return;
  }
  statsWindow.entries.push({
    key: `${statsWindow.id}:${definition.id}:${targetId ?? "scope"}`,
    statId: definition.id,
    targetId,
  });
}

function getDefaultAdHocTargetId(definition: StatDefinition): string {
  if (definition.scope === "player") {
    return replayPlayer?.replay.players[0]?.id ?? "";
  }
  return "blue";
}

function removeStatFromWindow(statsWindow: StatsWindowState, entryKey: string): void {
  const index = statsWindow.entries.findIndex((entry) => entry.key === entryKey);
  if (index >= 0) {
    statsWindow.entries.splice(index, 1);
  }
}

function renderStatsWindowEntries(
  statsWindow: StatsWindowState,
  frameIndex: number,
): void {
  const frame = getCurrentStatsFrame(frameIndex);
  const allowedScope = getStatsWindowAllowedScope(statsWindow.kind);
  const entries = statsWindow.entries
    .map((entry) => ({ entry, definition: getStatById(entry.statId) }))
    .filter((item): item is { entry: SelectedStatEntry; definition: StatDefinition } =>
      item.definition !== null &&
      (!allowedScope || item.definition.scope === allowedScope)
    );

  if (entries.length === 0) {
    const empty = document.createElement("p");
    empty.className = "stat-window-empty";
    empty.textContent = "No stats added.";
    statsWindow.body.append(empty);
    return;
  }

  if (!frame) {
    const empty = document.createElement("p");
    empty.className = "stat-window-empty";
    empty.textContent = "Load a replay to show stats.";
    statsWindow.body.append(empty);
    return;
  }

  if (statsWindow.kind === "all-players") {
    renderAllPlayersStats(statsWindow, frame, entries);
    return;
  }
  if (statsWindow.kind === "all-teams") {
    renderAllTeamsStats(statsWindow, frame, entries);
    return;
  }
  if (statsWindow.kind === "player") {
    const player = statsWindow.playerId
      ? frame.players.find((candidate) =>
        playerIdToString(candidate.player_id) === statsWindow.playerId
      ) ?? null
      : null;
    renderScopedStatList(statsWindow, player, entries);
    return;
  }
  if (statsWindow.kind === "team") {
    renderScopedStatList(
      statsWindow,
      getTeamSnapshot(frame, statsWindow.team ?? "blue"),
      entries,
    );
    return;
  }
  if (statsWindow.kind === "ad-hoc") {
    renderAdHocStats(statsWindow, frame, entries);
  }
}

function renderScopedStatList(
  statsWindow: StatsWindowState,
  target: PlayerStatsSnapshot | TeamStatsSnapshot | null,
  entries: Array<{ entry: SelectedStatEntry; definition: StatDefinition }>,
): void {
  const list = document.createElement("div");
  list.className = "stats-window-stat-list";
  for (const { entry, definition } of entries) {
    list.append(renderStatRow(
      statsWindow,
      entry,
      definition,
      target ? definition.format(definition.read(target)) : "--",
    ));
  }
  statsWindow.body.append(list);
}

function renderAllPlayersStats(
  statsWindow: StatsWindowState,
  frame: StatsFrame,
  entries: Array<{ entry: SelectedStatEntry; definition: StatDefinition }>,
): void {
  const list = document.createElement("div");
  list.className = "stats-window-team-list";
  for (const team of ["blue", "orange"] as const) {
    const players = frame.players.filter((player) =>
      player.is_team_0 === (team === "blue")
    );
    if (players.length === 0) {
      continue;
    }

    const teamSection = document.createElement("section");
    teamSection.className = `stats-window-team-group ${getTeamScopeClass(team)}`;

    const teamHeader = document.createElement("header");
    teamHeader.className = "stats-window-team-header";
    const teamTitle = document.createElement("h3");
    teamTitle.textContent = `${getTeamLabel(team)} team`;
    const teamMeta = document.createElement("span");
    teamMeta.textContent = `${players.length} player${players.length === 1 ? "" : "s"}`;
    teamHeader.append(teamTitle, teamMeta);
    teamSection.append(teamHeader);

    const playerList = document.createElement("div");
    playerList.className = "stats-window-entity-list";
    for (const player of players) {
      const section = document.createElement("section");
      section.className = `stats-window-entity ${getTeamClass(player.is_team_0)}`;
      const title = document.createElement("h4");
      title.className = "stats-window-entity-title";
      title.textContent = player.name;
      section.append(title);
      for (const { entry, definition } of entries) {
        section.append(renderStatRow(
          statsWindow,
          entry,
          definition,
          definition.format(definition.read(player)),
        ));
      }
      playerList.append(section);
    }
    teamSection.append(playerList);
    list.append(teamSection);
  }
  statsWindow.body.append(list);
}

function renderAllTeamsStats(
  statsWindow: StatsWindowState,
  frame: StatsFrame,
  entries: Array<{ entry: SelectedStatEntry; definition: StatDefinition }>,
): void {
  const list = document.createElement("div");
  list.className = "stats-window-entity-list";
  for (const team of ["blue", "orange"] as const) {
    const snapshot = getTeamSnapshot(frame, team);
    const section = document.createElement("section");
    section.className = `stats-window-entity ${getTeamScopeClass(team)}`;
    const title = document.createElement("h3");
    title.className = "stats-window-entity-title";
    title.textContent = getTeamLabel(team);
    section.append(title);
    for (const { entry, definition } of entries) {
      section.append(renderStatRow(
        statsWindow,
        entry,
        definition,
        snapshot ? definition.format(definition.read(snapshot)) : "--",
      ));
    }
    list.append(section);
  }
  statsWindow.body.append(list);
}

function renderAdHocStats(
  statsWindow: StatsWindowState,
  frame: StatsFrame,
  entries: Array<{ entry: SelectedStatEntry; definition: StatDefinition }>,
): void {
  const list = document.createElement("div");
  list.className = "stats-window-stat-list";
  for (const { entry, definition } of entries) {
    const target = getAdHocTarget(frame, definition, entry.targetId);
    list.append(renderStatRow(
      statsWindow,
      entry,
      definition,
      target ? definition.format(definition.read(target)) : "--",
    ));
  }
  statsWindow.body.append(list);
}

function getAdHocTarget(
  frame: StatsFrame,
  definition: StatDefinition,
  targetId: string | undefined,
): PlayerStatsSnapshot | TeamStatsSnapshot | null {
  if (definition.scope === "player") {
    return frame.players.find((player) =>
      playerIdToString(player.player_id) === targetId
    ) ?? frame.players[0] ?? null;
  }
  return getTeamSnapshot(frame, targetId === "orange" ? "orange" : "blue");
}

function renderStatRow(
  statsWindow: StatsWindowState,
  entry: SelectedStatEntry,
  definition: StatDefinition,
  value: string,
): HTMLElement {
  const row = document.createElement("div");
  row.className = "stats-window-stat-row";
  const name = document.createElement("span");
  name.className = "stats-window-stat-name";
  name.textContent = definition.label;
  if (statsWindow.kind === "ad-hoc") {
    const targetSelect = document.createElement("select");
    targetSelect.className = "stats-window-stat-target";
    const teamClass = getStatTargetTeamClass(definition, entry.targetId);
    if (teamClass) {
      targetSelect.classList.add(teamClass);
    }
    if (definition.scope === "player") {
      appendGroupedPlayerOptions(targetSelect, entry.targetId);
    } else {
      targetSelect.append(
        new Option("Blue", "blue", entry.targetId === "blue", entry.targetId === "blue"),
        new Option(
          "Orange",
          "orange",
          entry.targetId === "orange",
          entry.targetId === "orange",
        ),
      );
    }
    targetSelect.value = entry.targetId ?? "";
    targetSelect.addEventListener("change", () => {
      const nextTargetId = targetSelect.value;
      if (statsWindow.entries.some((candidate) =>
        candidate !== entry &&
        candidate.statId === entry.statId &&
        candidate.targetId === nextTargetId
      )) {
        renderStatsWindow(statsWindow);
        return;
      }
      const index = statsWindow.entries.findIndex((candidate) =>
        candidate.key === entry.key
      );
      if (index >= 0) {
        statsWindow.entries[index] = {
          key: `${statsWindow.id}:${entry.statId}:${nextTargetId}`,
          statId: entry.statId,
          targetId: nextTargetId,
        };
      }
      renderStatsWindow(statsWindow);
      scheduleConfigUrlUpdate();
    });
    name.append(" ", targetSelect);
  }
  const valueEl = document.createElement("span");
  valueEl.className = "stats-window-stat-value";
  valueEl.textContent = value;
  const remove = document.createElement("button");
  remove.type = "button";
  remove.className = "stats-window-stat-remove";
  remove.textContent = "x";
  remove.addEventListener("click", () => {
    removeStatFromWindow(statsWindow, entry.key);
    renderStatsWindow(statsWindow);
    scheduleConfigUrlUpdate();
  });
  row.append(name, valueEl, remove);
  return row;
}

function formatSetting(
  value: number | undefined,
  suffix = "",
  digits = 0,
): string {
  if (value === undefined || Number.isNaN(value)) {
    return "--";
  }

  return `${value.toFixed(digits)}${suffix}`;
}

function getFallbackCameraSettings(): Required<CameraSettings> {
  return {
    fov: 110,
    height: 100,
    pitch: -4,
    distance: 270,
    stiffness: 0,
    swivelSpeed: 1,
    transitionSpeed: 1,
  };
}

function getAttachedPlayerCameraSettings(
  attachedPlayerId: string | null,
): CameraSettings | null {
  if (!replayPlayer || attachedPlayerId === null) {
    return null;
  }

  return replayPlayer.replay.players.find(
    (candidate) => candidate.id === attachedPlayerId,
  )?.cameraSettings ?? null;
}

function getEffectiveCameraSettings(state: ReplayPlayerState): CameraSettings {
  return {
    ...getFallbackCameraSettings(),
    ...(getAttachedPlayerCameraSettings(state.attachedPlayerId) ?? {}),
    ...(state.customCameraSettings ?? {}),
  };
}

function readCustomCameraSettings(): CameraSettings {
  return {
    fov: Number(customCameraFov.value),
    height: Number(customCameraHeight.value),
    pitch: Number(customCameraPitch.value),
    distance: Number(customCameraDistance.value),
    stiffness: Number(customCameraStiffness.value),
    swivelSpeed: Number(customCameraSwivelSpeed.value),
    transitionSpeed: Number(customCameraTransitionSpeed.value),
  };
}

function setCameraSettingControlsEnabled(enabled: boolean): void {
  cameraSettingsControls.hidden = !customCameraSettings.checked;
  customCameraFov.disabled = !enabled;
  customCameraHeight.disabled = !enabled;
  customCameraPitch.disabled = !enabled;
  customCameraDistance.disabled = !enabled;
  customCameraStiffness.disabled = !enabled;
  customCameraSwivelSpeed.disabled = !enabled;
  customCameraTransitionSpeed.disabled = !enabled;
}

function syncCustomCameraSettingControls(settings: CameraSettings): void {
  const fallback = getFallbackCameraSettings();
  const fov = settings.fov ?? fallback.fov;
  const height = settings.height ?? fallback.height;
  const pitch = settings.pitch ?? fallback.pitch;
  const distance = settings.distance ?? fallback.distance;
  const stiffness = settings.stiffness ?? fallback.stiffness;
  const swivelSpeed = settings.swivelSpeed ?? fallback.swivelSpeed;
  const transitionSpeed = settings.transitionSpeed ?? fallback.transitionSpeed;

  customCameraFov.value = `${fov}`;
  customCameraHeight.value = `${height}`;
  customCameraPitch.value = `${pitch}`;
  customCameraDistance.value = `${distance}`;
  customCameraStiffness.value = `${stiffness}`;
  customCameraSwivelSpeed.value = `${swivelSpeed}`;
  customCameraTransitionSpeed.value = `${transitionSpeed}`;

  customCameraFovReadout.textContent = formatSetting(fov, "", 0);
  customCameraHeightReadout.textContent = formatSetting(height, "", 0);
  customCameraPitchReadout.textContent = formatSetting(pitch, "", 0);
  customCameraDistanceReadout.textContent = formatSetting(distance, "", 0);
  customCameraStiffnessReadout.textContent = formatSetting(stiffness, "", 2);
  customCameraSwivelSpeedReadout.textContent = formatSetting(swivelSpeed, "", 1);
  customCameraTransitionSpeedReadout.textContent = formatSetting(
    transitionSpeed,
    "",
    2,
  );
}

function setTransportEnabled(enabled: boolean): void {
  togglePlayback.disabled = !enabled;
  playbackRate.disabled = !enabled;
  attachedPlayer.disabled = !enabled;
  skipPostGoalTransitions.disabled = !enabled;
  skipKickoffs.disabled = !enabled;
  syncCameraModeButtons(enabled ? replayPlayer?.getState() : undefined);
}

function getCameraViewButton(
  mode: ReplayCameraViewMode,
): HTMLButtonElement {
  switch (mode) {
    case "free":
      return cameraViewFreeButton;
    case "follow":
      return cameraViewFollowButton;
  }
}

function syncCameraModeButtons(state?: ReplayPlayerState): void {
  const activeMode = state?.cameraViewMode ?? "free";
  const hasReplay = replayPlayer !== null && state !== undefined;
  const canFollow = (state?.attachedPlayerId ?? null) !== null;

  for (const mode of CAMERA_VIEW_MODES) {
    const button = getCameraViewButton(mode);
    button.disabled = !hasReplay || (mode === "follow" && !canFollow);
    const isActive = mode === activeMode;
    button.dataset.active = isActive ? "true" : "false";
    button.setAttribute("aria-pressed", isActive ? "true" : "false");
  }

  cameraViewOverheadButton.disabled = !hasReplay;
  cameraViewSideButton.disabled = !hasReplay;
  cameraViewOverheadButton.dataset.active = "false";
  cameraViewSideButton.dataset.active = "false";
  cameraViewOverheadButton.setAttribute("aria-pressed", "false");
  cameraViewSideButton.setAttribute("aria-pressed", "false");
}

function syncCameraControlAvailability(state?: ReplayPlayerState): void {
  syncCameraModeButtons(state);
  const hasAttachedCamera = replayPlayer !== null &&
    state?.cameraViewMode === "follow" &&
    (state.attachedPlayerId ?? null) !== null;
  cameraDistance.disabled = !hasAttachedCamera;
  customCameraSettings.disabled = !hasAttachedCamera;
  setCameraSettingControlsEnabled(
    hasAttachedCamera && state?.customCameraSettings !== null,
  );
  ballCam.disabled = !hasAttachedCamera;
}

function populateAttachedPlayerOptions(players: ReplayPlayerTrack[]): void {
  attachedPlayer.replaceChildren();
  attachedPlayer.append(new Option("Free camera", ""));

  for (const player of players) {
    attachedPlayer.append(
      new Option(
        `${player.name} (${player.isTeamZero ? "Blue" : "Orange"})`,
        player.id,
      ),
    );
  }
}

function formatBytes(bytes: number): string {
  if (bytes <= 0) {
    return "--";
  }
  const units = ["B", "KB", "MB", "GB"];
  let value = bytes;
  let unitIndex = 0;
  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex += 1;
  }
  const precision = unitIndex === 0 ? 0 : value >= 10 ? 1 : 2;
  return `${value.toFixed(precision)} ${units[unitIndex]}`;
}

function recordingLabel(status: CanvasRecorderStatus | null): string {
  if (!status) {
    return "No replay";
  }
  if (status.error) {
    return status.error;
  }
  switch (status.state) {
    case "idle":
      return "Idle";
    case "recording":
      return "Recording";
    case "stopping":
      return "Stopping";
    case "ready":
      return "Ready";
    case "error":
      return "Error";
  }
}

function getRecordingOptions(): { fps: number; playbackRate: number } {
  const fps = Number(recordingFps.value);
  const playbackRate = Number(recordingPlaybackRate.value);
  return {
    fps: Number.isFinite(fps) ? Math.max(1, Math.min(120, Math.trunc(fps))) : 60,
    playbackRate: Number.isFinite(playbackRate) ? Math.max(0.1, playbackRate) : 1,
  };
}

function syncRecordingWindow(status = canvasRecorder?.getStatus() ?? null): void {
  const hasRecorder = canvasRecorder !== null && replayPlayer !== null;
  const state = status?.state ?? "idle";
  const isRecording = state === "recording" || state === "stopping";
  const hasRecording = (canvasRecorder?.getRecording() ?? null) !== null;

  recordingStatus.textContent = recordingLabel(status);
  recordingElapsed.textContent = `${(status?.elapsedSeconds ?? 0).toFixed(1)}s`;
  recordingSize.textContent = formatBytes(status?.sizeBytes ?? 0);
  recordingType.textContent = status?.mimeType || "WebM";
  recordingStart.disabled = !hasRecorder || isRecording;
  recordingFullReplay.disabled = !hasRecorder || isRecording;
  recordingStop.disabled = !hasRecorder || !isRecording;
  recordingDownload.disabled = !hasRecording || isRecording;
  recordingClear.disabled = !hasRecording || isRecording;
  recordingFps.disabled = isRecording;
  recordingPlaybackRate.disabled = isRecording;
}

function recordingFileName(): string {
  const source = loadedReplayName?.replace(/\.replay$/i, "") || "replay";
  const safeSource = source.replace(/[^a-zA-Z0-9._-]+/g, "-").replace(/^-+|-+$/g, "");
  const timestamp = new Date().toISOString().replace(/[:.]/g, "-");
  return `${safeSource || "replay"}-${timestamp}.webm`;
}

function downloadRecording(blob: Blob): void {
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = recordingFileName();
  document.body.append(link);
  link.click();
  link.remove();
  window.setTimeout(() => URL.revokeObjectURL(url), 0);
}

function renderCameraProfile(state?: ReplayPlayerState): void {
  const attachedPlayerId = state?.attachedPlayerId ?? null;
  if (!replayPlayer || state?.cameraViewMode !== "follow" || attachedPlayerId === null) {
    cameraProfileReadout.textContent = "Free camera";
    cameraFovReadout.textContent = "--";
    cameraHeightReadout.textContent = "--";
    cameraPitchReadout.textContent = "--";
    cameraBaseDistanceReadout.textContent = "--";
    cameraStiffnessReadout.textContent = "--";
    return;
  }

  const player = replayPlayer.replay.players.find(
    (candidate) => candidate.id === attachedPlayerId,
  );
  if (!player) {
    cameraProfileReadout.textContent = "Unknown";
    cameraFovReadout.textContent = "--";
    cameraHeightReadout.textContent = "--";
    cameraPitchReadout.textContent = "--";
    cameraBaseDistanceReadout.textContent = "--";
    cameraStiffnessReadout.textContent = "--";
    return;
  }

  const cameraSettings = getEffectiveCameraSettings(state);
  cameraProfileReadout.textContent = state.customCameraSettings === null
    ? player.name
    : `${player.name} custom`;
  cameraFovReadout.textContent = formatSetting(cameraSettings.fov, "", 0);
  cameraHeightReadout.textContent = formatSetting(cameraSettings.height, "", 0);
  cameraPitchReadout.textContent = formatSetting(cameraSettings.pitch, "", 0);
  cameraBaseDistanceReadout.textContent = formatSetting(
    cameraSettings.distance,
    "",
    0,
  );
  cameraStiffnessReadout.textContent = formatSetting(
    cameraSettings.stiffness,
    "",
    2,
  );
}

function renderSnapshot(state: ReplayPlayerState): void {
  if (enforceMechanicsReviewClipBoundary(state)) {
    return;
  }

  timeReadout.textContent = `${state.currentTime.toFixed(2)}s`;
  frameReadout.textContent = `${state.frameIndex}`;
  durationReadout.textContent = `${state.duration.toFixed(2)}s`;
  playbackStatusReadout.textContent = state.playing ? "Playing" : "Paused";
  togglePlayback.textContent = state.playing ? "Pause" : "Play";
  playbackRate.value = `${state.speed}`;
  cameraDistance.value = `${state.cameraDistanceScale}`;
  cameraDistanceReadout.textContent = `${state.cameraDistanceScale.toFixed(2)}x`;
  customCameraSettings.checked = state.customCameraSettings !== null;
  cameraSettingsControls.hidden = !customCameraSettings.checked;
  syncCustomCameraSettingControls(getEffectiveCameraSettings(state));
  ballCam.checked = state.ballCamEnabled;
  attachedPlayer.value = state.attachedPlayerId ?? "";
  skipPostGoalTransitions.checked = state.skipPostGoalTransitionsEnabled;
  skipKickoffs.checked = state.skipKickoffsEnabled;
  emptyState.hidden = true;

  syncCameraControlAvailability(state);
  renderCameraProfile(state);
  renderStatsWindows(state.frameIndex, { preserveOpenPickers: true });
}

function includeBoostPickupAnimationPickup(
  pickup: BoostPickupAnimationPickup,
): boolean {
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
        const statusText = response.statusText
          ? ` ${response.statusText}`
          : "";
        const authHint =
          request.kind === "ballchasing" &&
            [401, 403, 404].includes(response.status)
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
  statusReadout.textContent = source.preparingStatus;
  fileInput.disabled = true;
  replayLoadModal?.show(source.name, source.preparingStatus);
  setTransportEnabled(false);
  syncCameraControlAvailability();
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
  renderTimelineEventCount();
  renderMechanicsTimelineControls();
  renderModuleSettings();
  syncRecordingWindow();

  try {
    const bytes = await source.readBytes();
    statusReadout.textContent = "Parsing replay...";
    replayLoadModal?.show(source.name, "Parsing replay...");
    const loadedReplay = await loadReplayBundleInWorker(bytes, {
      reportEveryNFrames: 100,
      onProgress(progress) {
        statusReadout.textContent = formatReplayLoadProgress(progress);
        replayLoadModal?.update(progress);
      },
    });
    const { replay } = loadedReplay;
    statsTimeline = loadedReplay.statsTimeline;
    statsFrameLookup = createStatsFrameLookup(statsTimeline);
    statRegistry = createStatRegistry(statsTimeline.frames[0] ?? null);
    if (activeMechanicTimelineKinds.size === 0) {
      activeMechanicTimelineKinds = new Set(getMechanicKinds(statsTimeline));
    }

    timelineOverlay = createTimelineOverlayPlugin({
      replayEvents: (context) =>
        filterReplayTimelineEvents(context.replay, activeTimelineEventModuleIds),
    });
    const recorder = createCanvasRecorderPlugin({
      onStatusChange: syncRecordingWindow,
    });
    canvasRecorder = recorder;
    const config = initialUrlConfig;

    replayPlayer = new ReplayPlayer(viewport, replay, {
      initialPlaybackRate: config?.playback.rate,
      initialCameraDistanceScale:
        config?.camera.distanceScale ?? DEFAULT_CAMERA_DISTANCE_SCALE,
      initialCustomCameraSettings: config?.camera.customSettings ?? null,
      initialAttachedPlayerId: config?.camera.attachedPlayerId ?? null,
      initialCameraViewMode: config?.camera.mode,
      initialBallCamEnabled: config?.camera.ballCam ?? false,
      initialBoostPickupAnimationEnabled:
        config?.overlays.boostPickupAnimation ?? false,
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

    populateAttachedPlayerOptions(replay.players);
    emptyState.hidden = true;
    statusReadout.textContent = `Loaded ${source.name}`;
    loadedReplayName = source.name;
    playersReadout.textContent = replay.players.map((player) => player.name)
      .join(", ");
    framesReadout.textContent = `${replay.frameCount}`;
    renderTimelineEventCount();
    renderMechanicsTimelineControls();
    setTransportEnabled(true);
    syncCameraControlAvailability(replayPlayer.getState());
    renderSnapshot(replayPlayer.getState());
    renderStatsWindows(replayPlayer.getState().frameIndex);
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
    replayRequest = getReplayFetchRequestFromSearch(
      window.location.search,
      window.location.href,
    );
  } catch (error) {
    console.error("Invalid replay URL:", error);
    statusReadout.textContent = error instanceof Error
      ? error.message
      : "Invalid replay URL";
    return;
  }

  if (!replayRequest) {
    return;
  }

  void loadReplay(createRemoteReplaySource(replayRequest, signal)).catch(
    (error) => {
      if (signal.aborted) {
        return;
      }
      console.error("Failed to load replay URL:", error);
      statusReadout.textContent = error instanceof Error
        ? error.message
        : "Failed to load replay URL";
    },
  );
}

export function mountStatEvaluationPlayer(
  root: HTMLElement,
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
  mechanicsTimelineWindowBody = mustElement<HTMLDivElement>(
    root,
    "#mechanics-timeline-window-body",
  );
  mechanicsReviewFile = mustElement<HTMLInputElement>(
    root,
    "#mechanics-review-file",
  );
  mechanicsReviewUrl = mustElement<HTMLInputElement>(
    root,
    "#mechanics-review-url",
  );
  mechanicsReviewLoadUrl = mustElement<HTMLButtonElement>(
    root,
    "#mechanics-review-load-url",
  );
  mechanicsReviewStatus = mustElement<HTMLElement>(
    root,
    "#mechanics-review-status",
  );
  mechanicsReviewIndex = mustElement<HTMLElement>(
    root,
    "#mechanics-review-index",
  );
  mechanicsReviewTitle = mustElement<HTMLElement>(
    root,
    "#mechanics-review-title",
  );
  mechanicsReviewMechanic = mustElement<HTMLElement>(
    root,
    "#mechanics-review-mechanic",
  );
  mechanicsReviewPlayer = mustElement<HTMLElement>(
    root,
    "#mechanics-review-player",
  );
  mechanicsReviewReason = mustElement<HTMLElement>(
    root,
    "#mechanics-review-reason",
  );
  mechanicsReviewPrev = mustElement<HTMLButtonElement>(
    root,
    "#mechanics-review-prev",
  );
  mechanicsReviewReplay = mustElement<HTMLButtonElement>(
    root,
    "#mechanics-review-replay",
  );
  mechanicsReviewNext = mustElement<HTMLButtonElement>(
    root,
    "#mechanics-review-next",
  );
  mechanicsReviewCount = mustElement<HTMLElement>(
    root,
    "#mechanics-review-count",
  );
  mechanicsReviewList = mustElement<HTMLDivElement>(
    root,
    "#mechanics-review-list",
  );
  boostPickupFiltersWindowBody = mustElement<HTMLDivElement>(
    root,
    "#boost-pickup-filters-window-body",
  );
  touchControlsWindowBody = mustElement<HTMLDivElement>(
    root,
    "#touch-controls-window-body",
  );
  statsWindowLayer = mustElement<HTMLDivElement>(root, "#stats-window-layer");
  togglePlayback = mustElement<HTMLButtonElement>(root, "#toggle-playback");
  playbackRate = mustElement<HTMLSelectElement>(root, "#playback-rate");
  attachedPlayer = mustElement<HTMLSelectElement>(root, "#attached-player");
  cameraViewFreeButton = mustElement<HTMLButtonElement>(
    root,
    "#camera-view-free",
  );
  cameraViewFollowButton = mustElement<HTMLButtonElement>(
    root,
    "#camera-view-follow",
  );
  cameraViewOverheadButton = mustElement<HTMLButtonElement>(
    root,
    "#camera-view-overhead",
  );
  cameraViewSideButton = mustElement<HTMLButtonElement>(root, "#camera-view-side");
  cameraDistance = mustElement<HTMLInputElement>(root, "#camera-distance");
  cameraDistanceReadout = mustElement<HTMLElement>(
    root,
    "#camera-distance-readout",
  );
  customCameraSettings = mustElement<HTMLInputElement>(
    root,
    "#custom-camera-settings",
  );
  cameraSettingsControls = mustElement<HTMLDivElement>(
    root,
    "#camera-settings-controls",
  );
  customCameraFov = mustElement<HTMLInputElement>(root, "#custom-camera-fov");
  customCameraHeight = mustElement<HTMLInputElement>(
    root,
    "#custom-camera-height",
  );
  customCameraPitch = mustElement<HTMLInputElement>(root, "#custom-camera-pitch");
  customCameraDistance = mustElement<HTMLInputElement>(
    root,
    "#custom-camera-distance",
  );
  customCameraStiffness = mustElement<HTMLInputElement>(
    root,
    "#custom-camera-stiffness",
  );
  customCameraSwivelSpeed = mustElement<HTMLInputElement>(
    root,
    "#custom-camera-swivel-speed",
  );
  customCameraTransitionSpeed = mustElement<HTMLInputElement>(
    root,
    "#custom-camera-transition-speed",
  );
  customCameraFovReadout = mustElement<HTMLElement>(
    root,
    "#custom-camera-fov-readout",
  );
  customCameraHeightReadout = mustElement<HTMLElement>(
    root,
    "#custom-camera-height-readout",
  );
  customCameraPitchReadout = mustElement<HTMLElement>(
    root,
    "#custom-camera-pitch-readout",
  );
  customCameraDistanceReadout = mustElement<HTMLElement>(
    root,
    "#custom-camera-distance-readout",
  );
  customCameraStiffnessReadout = mustElement<HTMLElement>(
    root,
    "#custom-camera-stiffness-readout",
  );
  customCameraSwivelSpeedReadout = mustElement<HTMLElement>(
    root,
    "#custom-camera-swivel-speed-readout",
  );
  customCameraTransitionSpeedReadout = mustElement<HTMLElement>(
    root,
    "#custom-camera-transition-speed-readout",
  );
  ballCam = mustElement<HTMLInputElement>(root, "#ball-cam");
  moduleSummaryEl = mustElement<HTMLDivElement>(root, "#module-summary");
  moduleSettingsEl = mustElement<HTMLDivElement>(root, "#module-settings");
  timeReadout = mustElement<HTMLElement>(root, "#time-readout");
  frameReadout = mustElement<HTMLElement>(root, "#frame-readout");
  durationReadout = mustElement<HTMLElement>(root, "#duration-readout");
  playbackStatusReadout = mustElement<HTMLElement>(
    root,
    "#playback-status-readout",
  );
  statusReadout = mustElement<HTMLElement>(root, "#status-readout");
  playersReadout = mustElement<HTMLElement>(root, "#players-readout");
  framesReadout = mustElement<HTMLElement>(root, "#frames-readout");
  eventsReadout = mustElement<HTMLElement>(root, "#events-readout");
  cameraProfileReadout = mustElement<HTMLElement>(
    root,
    "#camera-profile-readout",
  );
  cameraFovReadout = mustElement<HTMLElement>(root, "#camera-fov-readout");
  cameraHeightReadout = mustElement<HTMLElement>(root, "#camera-height-readout");
  cameraPitchReadout = mustElement<HTMLElement>(root, "#camera-pitch-readout");
  cameraBaseDistanceReadout = mustElement<HTMLElement>(
    root,
    "#camera-base-distance-readout",
  );
  cameraStiffnessReadout = mustElement<HTMLElement>(
    root,
    "#camera-stiffness-readout",
  );
  skipPostGoalTransitions = mustElement<HTMLInputElement>(
    root,
    "#skip-post-goal-transitions",
  );
  skipKickoffs = mustElement<HTMLInputElement>(root, "#skip-kickoffs");
  recordingFps = mustElement<HTMLInputElement>(root, "#recording-fps");
  recordingPlaybackRate = mustElement<HTMLSelectElement>(
    root,
    "#recording-playback-rate",
  );
  recordingStart = mustElement<HTMLButtonElement>(root, "#recording-start");
  recordingFullReplay = mustElement<HTMLButtonElement>(
    root,
    "#recording-full-replay",
  );
  recordingStop = mustElement<HTMLButtonElement>(root, "#recording-stop");
  recordingDownload = mustElement<HTMLButtonElement>(
    root,
    "#recording-download",
  );
  recordingClear = mustElement<HTMLButtonElement>(root, "#recording-clear");
  recordingStatus = mustElement<HTMLElement>(root, "#recording-status");
  recordingElapsed = mustElement<HTMLElement>(root, "#recording-elapsed");
  recordingSize = mustElement<HTMLElement>(root, "#recording-size");
  recordingType = mustElement<HTMLElement>(root, "#recording-type");

  const configParamSnapshot = getStatsPlayerConfigParamSnapshot(window.location);
  const configDebugEnabled = isStatsPlayerConfigDebugEnabled(window.location);
  let configLoadError: unknown = null;
  try {
    initialUrlConfig = getStatsPlayerConfigFromLocation(window.location);
  } catch (error) {
    configLoadError = error;
    console.error("Invalid stats player config:", error);
    statusReadout.textContent = error instanceof Error
      ? error.message
      : "Invalid stats player config";
    initialUrlConfig = null;
  }
  if (configDebugEnabled) {
    logStatsPlayerConfigLoadDebug(
      configParamSnapshot,
      initialUrlConfig,
      configLoadError,
    );
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
    statsWindows.clear();
    clearTimelineEventSources();
    clearTimelineRangeSources();
    clearStandalonePlugins();
    clearRenderCaches();
    activeModules = [];
    replayLoadModal?.destroy();
    replayLoadModal = null;
    activeTimelineEventModuleIds = new Set<string>();
    activeTimelineRangeModuleIds = new Set<string>();
    activeMechanicTimelineKinds = new Set<string>();
    activeRenderEffectModuleIds = new Set<string>();
    activeMechanicsReview = null;
    mechanicsReviewBoundaryGuard = false;
    boostPadOverlayEnabled = true;
    loadedReplayName = null;
    lastFreeCameraPreset = null;
    initialUrlConfig = null;
    if (configUrlUpdateTimer !== null) {
      window.clearTimeout(configUrlUpdateTimer);
      configUrlUpdateTimer = null;
    }
    isApplyingConfig = false;
    nextStatsWindowId = 1;
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

  launcherToggle.addEventListener("click", () => {
    setLauncherOpen(launcherMenu.hidden);
  }, { signal: listeners.signal });

  root.addEventListener("click", (event) => {
    if (!(event.target instanceof Element)) {
      return;
    }
    if (!event.target.closest(".top-chrome")) {
      setLauncherOpen(false);
    }
  }, { signal: listeners.signal });

  loadReplayAction.addEventListener("click", openReplayFilePicker, {
    signal: listeners.signal,
  });
  emptyLoadReplay.addEventListener("click", openReplayFilePicker, {
    signal: listeners.signal,
  });

  root.querySelectorAll<HTMLElement>("[data-window-toggle]").forEach((button) => {
    button.addEventListener("click", () => {
      const id = button.dataset.windowToggle as SingletonWindowId | undefined;
      if (id) {
        toggleWindow(id);
        setLauncherOpen(false);
      }
    }, { signal: listeners.signal });
  });

  root.querySelectorAll<HTMLElement>("[data-window-hide]").forEach((button) => {
    button.addEventListener("click", () => {
      const id = button.dataset.windowHide ?? getElementWindowId(button);
      if (id) {
        hideWindow(id);
      }
    }, { signal: listeners.signal });
  });

  root.querySelectorAll<HTMLElement>("[data-create-stats-window]").forEach((button) => {
    button.addEventListener("click", () => {
      createStatsWindow(button.dataset.createStatsWindow as StatsWindowKind);
    }, { signal: listeners.signal });
  });

  fileInput.addEventListener("change", async () => {
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
  }, { signal: listeners.signal });

  mechanicsReviewFile.addEventListener("change", async () => {
    const file = mechanicsReviewFile.files?.[0];
    if (!file) return;

    try {
      const manifest = parseMechanicsReviewPlaylistJson(await file.text());
      await loadMechanicsReviewPlaylist(manifest, null);
    } catch (error) {
      console.error("Failed to load mechanics review playlist:", error);
      setMechanicsReviewStatus(
        error instanceof Error
          ? error.message
          : "Failed to load mechanics review playlist",
      );
    } finally {
      mechanicsReviewFile.value = "";
    }
  }, { signal: listeners.signal });

  mechanicsReviewLoadUrl.addEventListener("click", () => {
    void loadMechanicsReviewPlaylistFromUrl(mechanicsReviewUrl.value.trim())
      .catch((error) => {
        console.error("Failed to load mechanics review playlist URL:", error);
        setMechanicsReviewStatus(
          error instanceof Error
            ? error.message
            : "Failed to load mechanics review playlist URL",
        );
      });
  }, { signal: listeners.signal });

  mechanicsReviewPrev.addEventListener("click", () => {
    const review = activeMechanicsReview;
    if (review) {
      void activateMechanicsReviewItem(Math.max(0, review.currentIndex - 1));
    }
  }, { signal: listeners.signal });

  mechanicsReviewReplay.addEventListener("click", replayMechanicsReviewClip, {
    signal: listeners.signal,
  });

  mechanicsReviewNext.addEventListener("click", () => {
    const review = activeMechanicsReview;
    if (review) {
      void activateMechanicsReviewItem(
        Math.min(review.manifest.items.length - 1, review.currentIndex + 1),
      );
    }
  }, { signal: listeners.signal });

  togglePlayback.addEventListener("click", () => {
    replayPlayer?.togglePlayback();
    scheduleConfigUrlUpdate();
  }, { signal: listeners.signal });

  playbackRate.addEventListener("change", () => {
    replayPlayer?.setPlaybackRate(Number(playbackRate.value));
    scheduleConfigUrlUpdate();
  }, { signal: listeners.signal });

  recordingStart.addEventListener("click", () => {
    if (!canvasRecorder) {
      return;
    }
    try {
      const { fps } = getRecordingOptions();
      canvasRecorder.start({ fps });
      syncRecordingWindow();
    } catch (error) {
      console.error("Failed to start recording:", error);
      statusReadout.textContent = error instanceof Error
        ? error.message
        : "Failed to start recording";
      syncRecordingWindow(canvasRecorder.getStatus());
    }
  }, { signal: listeners.signal });

  recordingFullReplay.addEventListener("click", () => {
    if (!canvasRecorder) {
      return;
    }
    const { fps, playbackRate } = getRecordingOptions();
    void canvasRecorder.recordFullReplay({
      fps,
      playbackRate,
      restorePlaybackState: true,
    }).catch((error) => {
      console.error("Failed to record replay:", error);
      statusReadout.textContent = error instanceof Error
        ? error.message
        : "Failed to record replay";
      syncRecordingWindow(canvasRecorder?.getStatus() ?? null);
    });
    syncRecordingWindow();
  }, { signal: listeners.signal });

  recordingStop.addEventListener("click", () => {
    void canvasRecorder?.stop().catch((error) => {
      console.error("Failed to stop recording:", error);
      statusReadout.textContent = error instanceof Error
        ? error.message
        : "Failed to stop recording";
    });
    syncRecordingWindow();
  }, { signal: listeners.signal });

  recordingDownload.addEventListener("click", () => {
    const blob = canvasRecorder?.getRecording();
    if (blob) {
      downloadRecording(blob);
    }
  }, { signal: listeners.signal });

  recordingClear.addEventListener("click", () => {
    try {
      canvasRecorder?.clear();
      syncRecordingWindow();
    } catch (error) {
      console.error("Failed to clear recording:", error);
    }
  }, { signal: listeners.signal });

  recordingFps.addEventListener("change", scheduleConfigUrlUpdate, {
    signal: listeners.signal,
  });
  recordingPlaybackRate.addEventListener("change", scheduleConfigUrlUpdate, {
    signal: listeners.signal,
  });

  cameraDistance.addEventListener("input", () => {
    replayPlayer?.setCameraDistanceScale(Number(cameraDistance.value));
    scheduleConfigUrlUpdate();
  }, { signal: listeners.signal });

  customCameraSettings.addEventListener("change", () => {
    cameraSettingsControls.hidden = !customCameraSettings.checked;
    replayPlayer?.setCustomCameraSettings(
      customCameraSettings.checked ? readCustomCameraSettings() : null,
    );
    scheduleConfigUrlUpdate();
  }, { signal: listeners.signal });

  for (const input of [
    customCameraFov,
    customCameraHeight,
    customCameraPitch,
    customCameraDistance,
    customCameraStiffness,
    customCameraSwivelSpeed,
    customCameraTransitionSpeed,
  ]) {
    input.addEventListener("input", () => {
      const settings = readCustomCameraSettings();
      syncCustomCameraSettingControls(settings);
      replayPlayer?.setCustomCameraSettings(settings);
      scheduleConfigUrlUpdate();
    }, { signal: listeners.signal });
  }

  attachedPlayer.addEventListener("change", () => {
    replayPlayer?.setAttachedPlayer(attachedPlayer.value || null);
    lastFreeCameraPreset = null;
    scheduleConfigUrlUpdate();
  }, { signal: listeners.signal });

  cameraViewFreeButton.addEventListener("click", () => {
    replayPlayer?.setCameraViewMode("free");
    lastFreeCameraPreset = null;
    scheduleConfigUrlUpdate();
  }, { signal: listeners.signal });

  cameraViewFollowButton.addEventListener("click", () => {
    replayPlayer?.setCameraViewMode("follow");
    lastFreeCameraPreset = null;
    scheduleConfigUrlUpdate();
  }, { signal: listeners.signal });

  cameraViewOverheadButton.addEventListener("click", () => {
    replayPlayer?.setFreeCameraPreset("overhead");
    lastFreeCameraPreset = "overhead";
    scheduleConfigUrlUpdate();
  }, { signal: listeners.signal });

  cameraViewSideButton.addEventListener("click", () => {
    replayPlayer?.setFreeCameraPreset("side");
    lastFreeCameraPreset = "side";
    scheduleConfigUrlUpdate();
  }, { signal: listeners.signal });

  ballCam.addEventListener("change", () => {
    replayPlayer?.setBallCamEnabled(ballCam.checked);
    scheduleConfigUrlUpdate();
  }, { signal: listeners.signal });

  skipPostGoalTransitions.addEventListener("change", () => {
    replayPlayer?.setSkipPostGoalTransitionsEnabled(
      skipPostGoalTransitions.checked,
    );
    scheduleConfigUrlUpdate();
  }, { signal: listeners.signal });

  skipKickoffs.addEventListener("change", () => {
    replayPlayer?.setSkipKickoffsEnabled(skipKickoffs.checked);
    scheduleConfigUrlUpdate();
  }, { signal: listeners.signal });

  renderModuleSummary();
  renderModuleSettings();
  renderCameraProfile();
  syncCameraModeButtons();
  syncRecordingWindow();
  renderTimelineEventCount();
  renderMechanicsReviewWindow();
  loadReplayFromLocation(listeners.signal);

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
