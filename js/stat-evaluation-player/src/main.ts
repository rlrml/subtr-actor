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
  CanvasRecorderStatus,
  CameraSettings,
  ReplayCameraViewMode,
  ReplayFreeCameraPreset,
  ReplayTimelineEvent,
  ReplayPlayerState,
  ReplayPlayerTrack,
  TimelineOverlayPlugin,
} from "@rlrml/player";
import { getAppTemplate } from "./appTemplate.ts";
import { createReplayLoadModal } from "./replayLoadModal.ts";
import type { ReplayLoadModalController } from "./replayLoadModal.ts";
import { createStatModules, getTeamClass, RELATIVE_POSITIONING_MODULE_ID } from "./statModules.ts";
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
  formatSetting,
  getEffectiveCameraSettings as mergeEffectiveCameraSettings,
  populateAttachedPlayerOptions as populateAttachedPlayerSelectOptions,
  readCustomCameraSettings as readCustomCameraControlSettings,
  syncCustomCameraSettingControls as syncCustomCameraControlSettings,
  type CameraSettingElements,
} from "./cameraControlHelpers.ts";
import {
  downloadRecording as downloadRecordingBlob,
  formatBytes,
  getRecordingOptions as getRecordingControlOptions,
  recordingFileName,
  recordingLabel,
} from "./recordingControlHelpers.ts";
import {
  createMechanicsReviewReplaySource,
  formatMechanicsReviewClipDetails,
  formatMechanicsReviewEventDetails,
  formatMechanicsReviewStatus,
  getMechanicsReviewBoundTime,
  getMechanicsReviewDecisionEndpoint,
  getMechanicsReviewItemLabel,
  getMechanicsReviewMechanicLabel,
  getMechanicsReviewPlayerId,
  getMechanicsReviewPlayerName,
  getMechanicsReviewReplayItems,
  getMechanicsReviewReplayLabel,
  getMechanicsReviewReplayPath,
  getMechanicsReviewUrlFromLocation,
  initializeMechanicsReviewReplayLoadStates,
  mechanicsReviewAuthHeaders,
  mechanicsReviewReplayLoadProgressValue,
  mechanicsReviewReplayLoadStatusText,
  parseMechanicsReviewPlaylistJson,
  resolveMechanicsReviewUrl,
  type ActiveMechanicsReview,
  type MechanicsReviewItem,
  type MechanicsReviewPlaylist,
  type MechanicsReviewReplay,
  type MechanicsReviewReplayLoadState,
} from "./mechanicsReview.ts";
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
  type RecordingConfig,
  type SingletonWindowId,
  type StatsPlayerConfig,
  type StatsPlayerConfigParamSnapshot,
  type StatsWindowKind,
} from "./playerConfig.ts";

const DEFAULT_CAMERA_DISTANCE_SCALE = 2.25;
const GOAL_WATCH_LEAD_SECONDS = 4;
const PLAYING_SNAPSHOT_UI_INTERVAL_MS = 100;
const CAMERA_VIEW_MODES: ReplayCameraViewMode[] = ["free", "follow"];

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
let boostPadOverlayEnabled = true;
let loadedReplayName: string | null = null;
let lastFreeCameraPreset: ReplayFreeCameraPreset | null = null;
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

let activeMechanicsReview: ActiveMechanicsReview | null = null;
let mechanicsReviewBoundaryGuard = false;

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
      timelineEvents: [...activeTimelineEventSourceIds],
      timelineRanges: [...activeTimelineRangeModuleIds],
      mechanics: [...activeMechanicTimelineKinds],
      renderEffects: [...activeRenderEffectModuleIds],
      followedPlayerHud: false,
      boostPads: boostPadOverlayEnabled,
      boostPickupAnimation: replayPlayer?.getState().boostPickupAnimationEnabled ?? false,
    },
    recording: getRecordingConfigSnapshot(),
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
  if (config.recording.fps !== undefined) {
    recordingFps.value = `${config.recording.fps}`;
  }
  if (config.recording.playbackRate !== undefined) {
    recordingPlaybackRate.value = `${config.recording.playbackRate}`;
  }
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

  if (activeMechanicsReview) {
    activeMechanicsReview.currentClip = null;
  }

  const canFollowScorer =
    scorerId !== null && replayPlayer.replay.players.some((player) => player.id === scorerId);
  if (canFollowScorer) {
    replayPlayer.setAttachedPlayer(scorerId);
    replayPlayer.setCameraViewMode("follow");
    lastFreeCameraPreset = null;
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
  lastFreeCameraPreset = config.camera.freePreset ?? null;
  if (config.camera.mode === "free" && config.camera.freePreset) {
    replayPlayer.setFreeCameraPreset(config.camera.freePreset);
  }
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

function setMechanicsReviewStatus(message: string): void {
  if (mechanicsReviewStatus) {
    mechanicsReviewStatus.textContent = message;
  }
}

function updateMechanicsReviewReplayLoadState(
  review: ActiveMechanicsReview,
  replayId: string,
  patch: Partial<Omit<MechanicsReviewReplayLoadState, "replayId">>,
): void {
  const current =
    review.replayLoadStates.get(replayId) ??
    ({
      replayId,
      label: replayId,
      path: "",
      clipCount: 0,
      status: "idle",
      progress: null,
      error: null,
    } satisfies MechanicsReviewReplayLoadState);
  review.replayLoadStates.set(replayId, {
    ...current,
    ...patch,
  });
  const activeItem = review.manifest.items[review.currentIndex];
  if (review.loading && activeItem?.replay === replayId && patch.progress) {
    statusReadout.textContent = formatReplayLoadProgress(patch.progress);
    replayLoadModal?.update(patch.progress);
  }
  if (activeMechanicsReview === review) {
    renderMechanicsReviewReplayLoads(review);
  }
}

function renderMechanicsReviewReplayLoads(review: ActiveMechanicsReview | null): void {
  if (!mechanicsReviewReplayLoadSummary || !replayLoadingSummary || !replayLoadingList) {
    return;
  }

  const states = review ? Array.from(review.replayLoadStates.values()) : [];
  const loaded = states.filter((state) => state.status === "loaded").length;
  const loading = states.filter((state) => state.status === "loading").length;
  const failed = states.filter((state) => state.status === "error").length;
  const pending = states.filter((state) => state.status === "idle").length;
  const summaryText =
    states.length === 0
      ? "0 replays"
      : `${loaded}/${states.length} loaded${loading > 0 ? `, ${loading} loading` : ""}${
          failed > 0 ? `, ${failed} failed` : ""
        }`;
  mechanicsReviewReplayLoadSummary.textContent = summaryText;
  replayLoadingSummary.textContent = summaryText;
  replayLoadingActive.textContent =
    states.length === 0
      ? "No playlist"
      : loading > 0
        ? `${loading} active, ${pending} pending`
        : failed > 0
          ? `${failed} failed`
          : review?.preloading
            ? `Background queue, ${pending} pending`
            : loaded === states.length
              ? "Complete"
              : `${pending} pending`;

  replayLoadingList.replaceChildren();
  if (!review || states.length === 0) {
    const empty = document.createElement("p");
    empty.className = "stat-window-empty";
    empty.textContent = "No replay sources.";
    replayLoadingList.append(empty);
    return;
  }

  for (const state of states) {
    const row = document.createElement("div");
    row.className = `mechanics-review-replay-load ${state.status}`;

    const main = document.createElement("div");
    main.className = "mechanics-review-replay-load-main";
    const title = document.createElement("span");
    title.className = "mechanics-review-replay-load-title";
    title.textContent = state.label;
    const meta = document.createElement("span");
    meta.className = "mechanics-review-replay-load-meta";
    meta.textContent = [
      state.replayId,
      `${state.clipCount} ${state.clipCount === 1 ? "clip" : "clips"}`,
      state.path,
    ]
      .filter(Boolean)
      .join(" · ");
    main.append(title, meta);

    const status = document.createElement("strong");
    status.className = "mechanics-review-replay-load-status";
    status.textContent = mechanicsReviewReplayLoadStatusText(state);

    const progress = document.createElement("div");
    progress.className = "mechanics-review-replay-load-progress";
    const bar = document.createElement("span");
    bar.style.width = `${Math.round(mechanicsReviewReplayLoadProgressValue(state) * 100)}%`;
    progress.append(bar);

    row.append(main, status, progress);
    replayLoadingList.append(row);
  }
}

function preloadMechanicsReviewReplays(
  review: ActiveMechanicsReview,
  currentReplayId: string,
): void {
  if (review.preloading) {
    return;
  }
  review.preloading = true;
  void (async () => {
    try {
      for (const [replayId, item] of getMechanicsReviewReplayItems(review)) {
        if (replayId === currentReplayId) {
          continue;
        }
        const state = review.replayLoadStates.get(replayId);
        if (state?.status === "loaded" || state?.status === "loading") {
          continue;
        }
        try {
          await loadMechanicsReviewReplayBundle(item, review);
        } catch {
          // Background preload failures are rendered in the replay load window.
        }
      }
    } finally {
      review.preloading = false;
    }
  })();
}

function loadMechanicsReviewReplayBundle(
  item: MechanicsReviewItem,
  review: ActiveMechanicsReview,
): Promise<ReplayLoadBundle> {
  const cached = review.replayLoadCache.get(item.replay);
  if (cached) {
    return cached;
  }

  const source = createMechanicsReviewReplaySource(item, review);
  updateMechanicsReviewReplayLoadState(review, item.replay, {
    label: source.name,
    path: getMechanicsReviewReplayPath(item, review),
    status: "loading",
    progress: null,
    error: null,
  });
  const loadPromise = Promise.resolve()
    .then(async () => {
      const bytes = await source.readBytes();
      return loadReplayBundleInWorker(bytes, {
        reportEveryNFrames: 100,
        onProgress(progress) {
          updateMechanicsReviewReplayLoadState(review, item.replay, {
            status: "loading",
            progress,
            error: null,
          });
        },
      });
    })
    .then((bundle) => {
      updateMechanicsReviewReplayLoadState(review, item.replay, {
        status: "loaded",
        progress: null,
        error: null,
      });
      return bundle;
    })
    .catch((error) => {
      review.replayLoadCache.delete(item.replay);
      updateMechanicsReviewReplayLoadState(review, item.replay, {
        status: "error",
        error: error instanceof Error ? error.message : String(error),
      });
      throw error;
    });
  review.replayLoadCache.set(item.replay, loadPromise);
  return loadPromise;
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
  mechanicsReviewPlayer.textContent = item
    ? getMechanicsReviewPlayerName(item, replayPlayer?.replay.players)
    : "--";
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
    meta.textContent = [
      getMechanicsReviewMechanicLabel(candidate),
      formatMechanicsReviewStatus(candidate.meta?.reviewStatus),
    ].join(" · ");

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

    const startTime = Math.max(
      0,
      getMechanicsReviewBoundTime(item.start, replayPlayer?.replay.frames),
    );
    const endTime = Math.min(
      replayPlayer?.getState().duration ?? Number.POSITIVE_INFINITY,
      Math.max(startTime, getMechanicsReviewBoundTime(item.end, replayPlayer?.replay.frames)),
    );
    if (!Number.isFinite(startTime) || !Number.isFinite(endTime) || endTime <= startTime) {
      throw new Error("Review item has an empty playback range.");
    }

    const playerId = getMechanicsReviewPlayerId(item);
    if (playerId && replayPlayer?.replay.players.some((player) => player.id === playerId)) {
      replayPlayer.setAttachedPlayer(playerId);
      replayPlayer.setCameraViewMode("follow");
      lastFreeCameraPreset = null;
    }

    skipPostGoalTransitions.checked = false;
    skipKickoffs.checked = false;
    review.currentClip = { startTime, endTime };
    replayPlayer?.setState({
      currentTime: startTime,
      playing: true,
      skipPostGoalTransitionsEnabled: false,
      skipKickoffsEnabled: false,
    });
    setMechanicsReviewStatus(`Playing ${startTime.toFixed(2)}s to ${endTime.toFixed(2)}s`);
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
  const frame = getCurrentStatsFrame(frameIndex);
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

function getAttachedPlayerCameraSettings(attachedPlayerId: string | null): CameraSettings | null {
  if (!replayPlayer || attachedPlayerId === null) {
    return null;
  }

  return (
    replayPlayer.replay.players.find((candidate) => candidate.id === attachedPlayerId)
      ?.cameraSettings ?? null
  );
}

function getEffectiveCameraSettings(state: ReplayPlayerState): CameraSettings {
  return mergeEffectiveCameraSettings(
    state,
    getAttachedPlayerCameraSettings(state.attachedPlayerId),
  );
}

function getCameraSettingElements(): CameraSettingElements {
  return {
    fov: customCameraFov,
    height: customCameraHeight,
    pitch: customCameraPitch,
    distance: customCameraDistance,
    stiffness: customCameraStiffness,
    swivelSpeed: customCameraSwivelSpeed,
    transitionSpeed: customCameraTransitionSpeed,
    fovReadout: customCameraFovReadout,
    heightReadout: customCameraHeightReadout,
    pitchReadout: customCameraPitchReadout,
    distanceReadout: customCameraDistanceReadout,
    stiffnessReadout: customCameraStiffnessReadout,
    swivelSpeedReadout: customCameraSwivelSpeedReadout,
    transitionSpeedReadout: customCameraTransitionSpeedReadout,
  };
}

function readCustomCameraSettings(): CameraSettings {
  return readCustomCameraControlSettings(getCameraSettingElements());
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
  syncCustomCameraControlSettings(getCameraSettingElements(), settings);
}

function setTransportEnabled(enabled: boolean): void {
  togglePlayback.disabled = !enabled;
  playbackRate.disabled = !enabled;
  attachedPlayer.disabled = !enabled;
  skipPostGoalTransitions.disabled = !enabled;
  skipKickoffs.disabled = !enabled;
  syncCameraModeButtons(enabled ? replayPlayer?.getState() : undefined);
}

function getCameraViewButton(mode: ReplayCameraViewMode): HTMLButtonElement {
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
  const hasAttachedCamera =
    replayPlayer !== null &&
    state?.cameraViewMode === "follow" &&
    (state.attachedPlayerId ?? null) !== null;
  cameraDistance.disabled = !hasAttachedCamera;
  customCameraSettings.disabled = !hasAttachedCamera;
  setCameraSettingControlsEnabled(hasAttachedCamera && state?.customCameraSettings !== null);
  ballCam.disabled = !hasAttachedCamera;
}

function populateAttachedPlayerOptions(players: ReplayPlayerTrack[]): void {
  populateAttachedPlayerSelectOptions(attachedPlayer, players);
}

function getRecordingOptions(): { fps: number; playbackRate: number } {
  return getRecordingControlOptions({
    fps: recordingFps,
    playbackRate: recordingPlaybackRate,
  });
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

function downloadRecording(blob: Blob): void {
  downloadRecordingBlob(blob, recordingFileName(loadedReplayName));
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

  const player = replayPlayer.replay.players.find((candidate) => candidate.id === attachedPlayerId);
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
  cameraProfileReadout.textContent =
    state.customCameraSettings === null ? player.name : `${player.name} custom`;
  cameraFovReadout.textContent = formatSetting(cameraSettings.fov, "", 0);
  cameraHeightReadout.textContent = formatSetting(cameraSettings.height, "", 0);
  cameraPitchReadout.textContent = formatSetting(cameraSettings.pitch, "", 0);
  cameraBaseDistanceReadout.textContent = formatSetting(cameraSettings.distance, "", 0);
  cameraStiffnessReadout.textContent = formatSetting(cameraSettings.stiffness, "", 2);
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
  eventWindowsManager.resetPlaylistState();
  renderScoreboard();
  renderTimelineEventCount();
  eventWindowsManager.renderTimelineControls();
  eventWindowsManager.renderPlaylistWindow();
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
    playersReadout.textContent = replay.players.map((player) => player.name).join(", ");
    framesReadout.textContent = `${replay.frameCount}`;
    renderTimelineEventCount();
    eventWindowsManager.renderTimelineControls();
    eventWindowsManager.resetPlaylistState();
    eventWindowsManager.renderPlaylistWindow();
    setTransportEnabled(true);
    syncCameraControlAvailability(replayPlayer.getState());
    renderSnapshot(replayPlayer.getState());
    statsWindowManager.render(replayPlayer.getState().frameIndex);
    renderScoreboard(replayPlayer.getState().frameIndex);
    eventWindowsManager.syncPlaylistTimeline(replayPlayer.getState(), { forceScroll: true });
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
  mechanicsTimelineWindowBody = mustElement<HTMLDivElement>(
    root,
    "#mechanics-timeline-window-body",
  );
  eventPlaylistWindowBody = mustElement<HTMLDivElement>(root, "#event-playlist-window-body");
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
  mechanicsReviewCount = mustElement<HTMLElement>(root, "#mechanics-review-count");
  mechanicsReviewList = mustElement<HTMLDivElement>(root, "#mechanics-review-list");
  boostPickupFiltersWindowBody = mustElement<HTMLDivElement>(
    root,
    "#boost-pickup-filters-window-body",
  );
  touchControlsWindowBody = mustElement<HTMLDivElement>(root, "#touch-controls-window-body");
  statsWindowLayer = mustElement<HTMLDivElement>(root, "#stats-window-layer");
  togglePlayback = mustElement<HTMLButtonElement>(root, "#toggle-playback");
  playbackRate = mustElement<HTMLSelectElement>(root, "#playback-rate");
  attachedPlayer = mustElement<HTMLSelectElement>(root, "#attached-player");
  cameraViewFreeButton = mustElement<HTMLButtonElement>(root, "#camera-view-free");
  cameraViewFollowButton = mustElement<HTMLButtonElement>(root, "#camera-view-follow");
  cameraViewOverheadButton = mustElement<HTMLButtonElement>(root, "#camera-view-overhead");
  cameraViewSideButton = mustElement<HTMLButtonElement>(root, "#camera-view-side");
  cameraDistance = mustElement<HTMLInputElement>(root, "#camera-distance");
  cameraDistanceReadout = mustElement<HTMLElement>(root, "#camera-distance-readout");
  customCameraSettings = mustElement<HTMLInputElement>(root, "#custom-camera-settings");
  cameraSettingsControls = mustElement<HTMLDivElement>(root, "#camera-settings-controls");
  customCameraFov = mustElement<HTMLInputElement>(root, "#custom-camera-fov");
  customCameraHeight = mustElement<HTMLInputElement>(root, "#custom-camera-height");
  customCameraPitch = mustElement<HTMLInputElement>(root, "#custom-camera-pitch");
  customCameraDistance = mustElement<HTMLInputElement>(root, "#custom-camera-distance");
  customCameraStiffness = mustElement<HTMLInputElement>(root, "#custom-camera-stiffness");
  customCameraSwivelSpeed = mustElement<HTMLInputElement>(root, "#custom-camera-swivel-speed");
  customCameraTransitionSpeed = mustElement<HTMLInputElement>(
    root,
    "#custom-camera-transition-speed",
  );
  customCameraFovReadout = mustElement<HTMLElement>(root, "#custom-camera-fov-readout");
  customCameraHeightReadout = mustElement<HTMLElement>(root, "#custom-camera-height-readout");
  customCameraPitchReadout = mustElement<HTMLElement>(root, "#custom-camera-pitch-readout");
  customCameraDistanceReadout = mustElement<HTMLElement>(root, "#custom-camera-distance-readout");
  customCameraStiffnessReadout = mustElement<HTMLElement>(root, "#custom-camera-stiffness-readout");
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
  playbackStatusReadout = mustElement<HTMLElement>(root, "#playback-status-readout");
  statusReadout = mustElement<HTMLElement>(root, "#status-readout");
  playersReadout = mustElement<HTMLElement>(root, "#players-readout");
  framesReadout = mustElement<HTMLElement>(root, "#frames-readout");
  eventsReadout = mustElement<HTMLElement>(root, "#events-readout");
  cameraProfileReadout = mustElement<HTMLElement>(root, "#camera-profile-readout");
  cameraFovReadout = mustElement<HTMLElement>(root, "#camera-fov-readout");
  cameraHeightReadout = mustElement<HTMLElement>(root, "#camera-height-readout");
  cameraPitchReadout = mustElement<HTMLElement>(root, "#camera-pitch-readout");
  cameraBaseDistanceReadout = mustElement<HTMLElement>(root, "#camera-base-distance-readout");
  cameraStiffnessReadout = mustElement<HTMLElement>(root, "#camera-stiffness-readout");
  skipPostGoalTransitions = mustElement<HTMLInputElement>(root, "#skip-post-goal-transitions");
  skipKickoffs = mustElement<HTMLInputElement>(root, "#skip-kickoffs");
  recordingFps = mustElement<HTMLInputElement>(root, "#recording-fps");
  recordingPlaybackRate = mustElement<HTMLSelectElement>(root, "#recording-playback-rate");
  recordingStart = mustElement<HTMLButtonElement>(root, "#recording-start");
  recordingFullReplay = mustElement<HTMLButtonElement>(root, "#recording-full-replay");
  recordingStop = mustElement<HTMLButtonElement>(root, "#recording-stop");
  recordingDownload = mustElement<HTMLButtonElement>(root, "#recording-download");
  recordingClear = mustElement<HTMLButtonElement>(root, "#recording-clear");
  recordingStatus = mustElement<HTMLElement>(root, "#recording-status");
  recordingElapsed = mustElement<HTMLElement>(root, "#recording-elapsed");
  recordingSize = mustElement<HTMLElement>(root, "#recording-size");
  recordingType = mustElement<HTMLElement>(root, "#recording-type");

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

  recordingStart.addEventListener(
    "click",
    () => {
      if (!canvasRecorder) {
        return;
      }
      try {
        const { fps } = getRecordingOptions();
        canvasRecorder.start({ fps });
        syncRecordingWindow();
      } catch (error) {
        console.error("Failed to start recording:", error);
        statusReadout.textContent =
          error instanceof Error ? error.message : "Failed to start recording";
        syncRecordingWindow(canvasRecorder.getStatus());
      }
    },
    { signal: listeners.signal },
  );

  recordingFullReplay.addEventListener(
    "click",
    () => {
      if (!canvasRecorder) {
        return;
      }
      const { fps, playbackRate } = getRecordingOptions();
      void canvasRecorder
        .recordFullReplay({
          fps,
          playbackRate,
          restorePlaybackState: true,
        })
        .catch((error) => {
          console.error("Failed to record replay:", error);
          statusReadout.textContent =
            error instanceof Error ? error.message : "Failed to record replay";
          syncRecordingWindow(canvasRecorder?.getStatus() ?? null);
        });
      syncRecordingWindow();
    },
    { signal: listeners.signal },
  );

  recordingStop.addEventListener(
    "click",
    () => {
      void canvasRecorder?.stop().catch((error) => {
        console.error("Failed to stop recording:", error);
        statusReadout.textContent =
          error instanceof Error ? error.message : "Failed to stop recording";
      });
      syncRecordingWindow();
    },
    { signal: listeners.signal },
  );

  recordingDownload.addEventListener(
    "click",
    () => {
      const blob = canvasRecorder?.getRecording();
      if (blob) {
        downloadRecording(blob);
      }
    },
    { signal: listeners.signal },
  );

  recordingClear.addEventListener(
    "click",
    () => {
      try {
        canvasRecorder?.clear();
        syncRecordingWindow();
      } catch (error) {
        console.error("Failed to clear recording:", error);
      }
    },
    { signal: listeners.signal },
  );

  recordingFps.addEventListener("change", scheduleConfigUrlUpdate, {
    signal: listeners.signal,
  });
  recordingPlaybackRate.addEventListener("change", scheduleConfigUrlUpdate, {
    signal: listeners.signal,
  });

  cameraDistance.addEventListener(
    "input",
    () => {
      replayPlayer?.setCameraDistanceScale(Number(cameraDistance.value));
      scheduleConfigUrlUpdate();
    },
    { signal: listeners.signal },
  );

  customCameraSettings.addEventListener(
    "change",
    () => {
      cameraSettingsControls.hidden = !customCameraSettings.checked;
      replayPlayer?.setCustomCameraSettings(
        customCameraSettings.checked ? readCustomCameraSettings() : null,
      );
      scheduleConfigUrlUpdate();
    },
    { signal: listeners.signal },
  );

  for (const input of [
    customCameraFov,
    customCameraHeight,
    customCameraPitch,
    customCameraDistance,
    customCameraStiffness,
    customCameraSwivelSpeed,
    customCameraTransitionSpeed,
  ]) {
    input.addEventListener(
      "input",
      () => {
        const settings = readCustomCameraSettings();
        syncCustomCameraSettingControls(settings);
        replayPlayer?.setCustomCameraSettings(settings);
        scheduleConfigUrlUpdate();
      },
      { signal: listeners.signal },
    );
  }

  attachedPlayer.addEventListener(
    "change",
    () => {
      replayPlayer?.setAttachedPlayer(attachedPlayer.value || null);
      lastFreeCameraPreset = null;
      scheduleConfigUrlUpdate();
    },
    { signal: listeners.signal },
  );

  cameraViewFreeButton.addEventListener(
    "click",
    () => {
      replayPlayer?.setCameraViewMode("free");
      lastFreeCameraPreset = null;
      scheduleConfigUrlUpdate();
    },
    { signal: listeners.signal },
  );

  cameraViewFollowButton.addEventListener(
    "click",
    () => {
      replayPlayer?.setCameraViewMode("follow");
      lastFreeCameraPreset = null;
      scheduleConfigUrlUpdate();
    },
    { signal: listeners.signal },
  );

  cameraViewOverheadButton.addEventListener(
    "click",
    () => {
      replayPlayer?.setFreeCameraPreset("overhead");
      lastFreeCameraPreset = "overhead";
      scheduleConfigUrlUpdate();
    },
    { signal: listeners.signal },
  );

  cameraViewSideButton.addEventListener(
    "click",
    () => {
      replayPlayer?.setFreeCameraPreset("side");
      lastFreeCameraPreset = "side";
      scheduleConfigUrlUpdate();
    },
    { signal: listeners.signal },
  );

  ballCam.addEventListener(
    "change",
    () => {
      replayPlayer?.setBallCamEnabled(ballCam.checked);
      scheduleConfigUrlUpdate();
    },
    { signal: listeners.signal },
  );

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
  renderCameraProfile();
  syncCameraModeButtons();
  syncRecordingWindow();
  renderTimelineEventCount();
  renderMechanicsReviewWindow();
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
