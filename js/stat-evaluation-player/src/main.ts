import "./styles.css";
import {
  createBoostPadOverlayPlugin,
  timelineEventSeekTime,
  ReplayPlayer,
} from "@rlrml/player";
import type {
  CanvasRecorderPlugin,
  ReplayTimelineEvent,
  ReplayPlayerState,
  TimelineOverlayPlugin,
} from "@rlrml/player";
import { getAppTemplate } from "./appTemplate.ts";
import { createReplayLoadModal } from "./replayLoadModal.ts";
import type { ReplayLoadModalController } from "./replayLoadModal.ts";
import { FloatingWindowController, mustElement } from "./floatingWindows.ts";
import { getStatsFrameForReplayFrame } from "./statsTimeline.ts";
import type {
  StatsFrame,
  StatsFrameLookup,
  StatsTimeline,
} from "./statsTimeline.ts";
import { createStatRegistry, type StatDefinition } from "./statRegistry.ts";
import type { ReplayLoadBundle } from "./replayLoader.ts";
import { createFileReplaySource } from "./replayInputSources.ts";
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
import {
  createReplayLoadController,
  type ReplayLoadController,
} from "./replayLoadController.ts";
import { renderScoreboardWindow } from "./scoreboardWindow.ts";
import {
  createModuleRuntimeController,
  type ModuleRuntimeController,
} from "./moduleRuntimeController.ts";
import {
  createMechanicsReviewController,
  getMechanicsReviewElements,
  type MechanicsReviewController,
} from "./mechanicsReviewController.ts";
import { createStatsWindowsManager } from "./statsWindows.ts";
import { createEventWindowsManager, type EventWindowsManager } from "./eventWindows.ts";
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
  type StatsWindowKind,
} from "./playerConfig.ts";
import { logStatsPlayerConfigLoadDebug } from "./playerConfigDebug.ts";

const DEFAULT_CAMERA_DISTANCE_SCALE = 2.25;
const GOAL_WATCH_LEAD_SECONDS = 4;
const PLAYING_SNAPSHOT_UI_INTERVAL_MS = 100;

let replayPlayer: ReplayPlayer | null = null;
let timelineOverlay: TimelineOverlayPlugin | null = null;
let canvasRecorder: CanvasRecorderPlugin | null = null;
let statsTimeline: StatsTimeline | null = null;
let statsFrameLookup: StatsFrameLookup | null = null;
let unsubscribe: (() => void) | null = null;
let lastPlayingSnapshotUiUpdateAt = 0;

const standalonePluginRemovers = new Map<string, () => void>();

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
let replayLoadController: ReplayLoadController | null = null;
let moduleRuntimeController: ModuleRuntimeController;
let eventWindowsManager: EventWindowsManager;

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

moduleRuntimeController = createModuleRuntimeController({
  getEventWindowsManager() {
    return eventWindowsManager;
  },
  getReplayPlayer() {
    return replayPlayer;
  },
  getStatsFrameLookup() {
    return statsFrameLookup;
  },
  getStatsTimeline() {
    return statsTimeline;
  },
  getTimelineOverlay() {
    return timelineOverlay;
  },
  renderTimelineEvent(event) {
    return {
      ...event,
      seekTime: timelineEventSeekTime(event),
    };
  },
  rerenderStatsWindow() {
    if (!replayPlayer) {
      return;
    }

    const state = replayPlayer.getState();
    statsWindowManager.render(state.frameIndex);
  },
  renderModuleRuntimeViews: afterModuleRuntimeChange,
  renderTimelineEventCountValue(value) {
    eventsReadout.textContent = value;
  },
  requestConfigSync: scheduleConfigUrlUpdate,
});

eventWindowsManager = createEventWindowsManager({
  cueTimelineEvent,
  formatTime,
  getActiveMechanicTimelineKinds() {
    return moduleRuntimeController.getActiveMechanicTimelineKinds();
  },
  getActiveTimelineEventSourceIds() {
    return moduleRuntimeController.getActiveTimelineEventSourceIds();
  },
  getModuleContext: () => moduleRuntimeController.getContext(),
  getModules() {
    return moduleRuntimeController.modules;
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
  renderTimelineEventCount: () => moduleRuntimeController.renderTimelineEventCount(),
  scheduleConfigUrlUpdate,
  setMechanicTimelineKind(kind, enabled) {
    moduleRuntimeController.setMechanicTimelineKind(kind, enabled);
  },
  setupActiveModules: () => moduleRuntimeController.setupActiveModules(),
  syncTimelineEvents: () => moduleRuntimeController.syncTimelineEvents(),
  syncTimelineRanges: () => moduleRuntimeController.syncTimelineRanges(),
  toggleCapability: (id, kind, enabled) =>
    moduleRuntimeController.toggleCapability(id, kind, enabled),
});

function afterModuleRuntimeChange(): void {
  renderModuleSummary();
  renderModuleSettings();
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

function getStatsPlayerConfigSnapshot(): StatsPlayerConfig {
  return {
    version: STATS_PLAYER_CONFIG_VERSION,
    playback: getPlaybackConfigSnapshot(),
    camera: cameraControls?.getConfigSnapshot() ?? {},
    overlays: {
      ...moduleRuntimeController.getOverlayConfigSnapshot(),
      followedPlayerHud: false,
      boostPads: boostPadOverlayEnabled,
      boostPickupAnimation: replayPlayer?.getState().boostPickupAnimationEnabled ?? false,
    },
    recording: recordingControls?.getConfigSnapshot() ?? {},
    singletonWindows: floatingWindows.getSingletonWindowConfigs(SINGLETON_WINDOW_IDS),
    statsWindows: statsWindowManager.getConfigs(),
    moduleConfigs: moduleRuntimeController.getModuleConfigSnapshot(),
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

function applyConfigToStaticControls(config: StatsPlayerConfig): void {
  moduleRuntimeController.setOverlayConfig(config.overlays);
  boostPadOverlayEnabled = config.overlays.boostPads;
  skipPostGoalTransitions.checked =
    config.playback.skipPostGoalTransitions ?? skipPostGoalTransitions.checked;
  skipKickoffs.checked = config.playback.skipKickoffs ?? skipKickoffs.checked;
  if (config.playback.rate !== undefined) {
    playbackRate.value = `${config.playback.rate}`;
  }
  recordingControls?.applyConfig(config.recording);
  moduleRuntimeController.applyModuleConfigSnapshot(config.moduleConfigs);
  floatingWindows.applyWindowConfigs(config.singletonWindows);
  statsWindowManager.replaceFromConfig(config.statsWindows);
  renderModuleSummary();
  renderModuleSettings();
  moduleRuntimeController.renderTimelineEventCount();
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

function applyConfigToReplayPlayer(config: StatsPlayerConfig): void {
  if (!replayPlayer) {
    return;
  }
  replayPlayer.setState(
    getReplayPlayerStatePatchFromConfig(config.playback, config.camera, config),
  );
  cameraControls?.applyReplayConfig(config.camera);
  syncBoostPadOverlayPlugin();
  moduleRuntimeController.setupActiveModules();
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

function renderModuleSummary(): void {
  moduleRuntimeController.renderModuleSummary(moduleSummaryEl, {
    boostPadOverlayEnabled,
    toggleBoostPadOverlay,
  });
}

function renderModuleSettings(): void {
  moduleRuntimeController.renderModuleSettings(moduleSettingsEl, touchControlsWindowBody);
  moduleRuntimeController.renderBoostPickupFiltersWindow(boostPickupFiltersWindowBody);
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

  replayLoadController = createReplayLoadController({
    defaultCameraDistanceScale: DEFAULT_CAMERA_DISTANCE_SCALE,
    emptyState,
    fileInput,
    replayLoadModal,
    statusReadout,
    viewport,
    getActiveTimelineEventSourceIds() {
      return moduleRuntimeController.getActiveTimelineEventSourceIds();
    },
    getInitialConfig() {
      return initialUrlConfig;
    },
    getInitialSkipKickoffsEnabled() {
      return skipKickoffs.checked;
    },
    getInitialSkipPostGoalTransitionsEnabled() {
      return skipPostGoalTransitions.checked;
    },
    getReplayPlayer() {
      return replayPlayer;
    },
    includeBoostPickupAnimationPickup(pickup) {
      return moduleRuntimeController.includeBoostPickupAnimationPickup(pickup);
    },
    applyConfigToReplayPlayer,
    clearRenderCaches() {
      moduleRuntimeController.clearRenderCaches();
    },
    clearStandalonePlugins,
    clearTimelineEventSources() {
      moduleRuntimeController.clearTimelineEventSources();
    },
    clearTimelineRangeSources() {
      moduleRuntimeController.clearTimelineRangeSources();
    },
    eventWindowsRenderPlaylistWindow() {
      eventWindowsManager.renderPlaylistWindow();
    },
    eventWindowsRenderTimelineControls() {
      eventWindowsManager.renderTimelineControls();
    },
    eventWindowsResetPlaylistState() {
      eventWindowsManager.resetPlaylistState();
    },
    eventWindowsSyncPlaylistTimeline(state, options) {
      eventWindowsManager.syncPlaylistTimeline(state, options);
    },
    migrateMechanicBackedTimelineEventSelections() {
      moduleRuntimeController.migrateMechanicBackedTimelineEventSelections();
    },
    recordingSync(status) {
      recordingControls?.sync(status);
    },
    renderModuleSettings,
    renderScoreboard,
    renderSnapshot,
    renderTimelineEventCount() {
      moduleRuntimeController.renderTimelineEventCount();
    },
    setCanvasRecorder(recorder) {
      canvasRecorder = recorder;
    },
    setIsApplyingConfig(isApplying) {
      isApplyingConfig = isApplying;
    },
    setLoadedReplayName(name) {
      loadedReplayName = name;
    },
    setReplayDetails(playersText, frameCount) {
      playersReadout.textContent = playersText;
      framesReadout.textContent = `${frameCount}`;
    },
    setReplayPlayer(player) {
      replayPlayer = player;
    },
    setStatRegistry(registry) {
      statRegistry = registry;
    },
    setStatsFrameLookup(lookup) {
      statsFrameLookup = lookup;
    },
    setStatsTimeline(timeline) {
      statsTimeline = timeline;
    },
    setTimelineOverlay(overlay) {
      timelineOverlay = overlay;
    },
    setTransportEnabled,
    setUnsubscribe(nextUnsubscribe) {
      unsubscribe = nextUnsubscribe;
    },
    setupActiveModules() {
      moduleRuntimeController.setupActiveModules();
    },
    statsWindowsRender(frameIndex) {
      statsWindowManager.render(frameIndex);
    },
    syncBoostPadOverlayPlugin,
    syncCameraAvailability(state) {
      cameraControls?.syncAvailability(state);
    },
    teardownActiveModules() {
      moduleRuntimeController.teardownActiveModules();
    },
    unsubscribeCurrent() {
      unsubscribe?.();
      unsubscribe = null;
    },
  });

  mechanicsReviewController = createMechanicsReviewController({
    elements: getMechanicsReviewElements(root),
    getReplayPlayer() {
      return replayPlayer;
    },
    loadReplayBundleForDisplay(source, bundlePromise) {
      return replayLoadController
        ? replayLoadController.loadReplayBundleForDisplay(source, bundlePromise)
        : Promise.reject(new Error("Replay loader is not initialized"));
    },
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
  floatingWindows.installDragging(floatingWindowLayer, listeners.signal);
  floatingWindows.installDragging(statsWindowLayer, listeners.signal);
  const cleanup = () => {
    listeners.abort();
    unsubscribe?.();
    unsubscribe = null;
    moduleRuntimeController.teardownActiveModules();
    replayPlayer?.destroy();
    replayPlayer = null;
    canvasRecorder = null;
    timelineOverlay = null;
    statsTimeline = null;
    statsFrameLookup = null;
    statRegistry = createStatRegistry(null);
    statsWindowManager.clear();
    moduleRuntimeController.clearTimelineEventSources();
    moduleRuntimeController.clearTimelineRangeSources();
    clearStandalonePlugins();
    moduleRuntimeController.reset();
    replayLoadModal?.destroy();
    replayLoadModal = null;
    eventWindowsManager.resetPlaylistState();
    mechanicsReviewController?.reset();
    mechanicsReviewController = null;
    recordingControls = null;
    cameraControls = null;
    replayLoadController = null;
    boostPadOverlayEnabled = true;
    loadedReplayName = null;
    initialUrlConfig = null;
    if (configUrlUpdateTimer !== null) {
      window.clearTimeout(configUrlUpdateTimer);
      configUrlUpdateTimer = null;
    }
    isApplyingConfig = false;
    floatingWindows.resetZIndex();
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
        await replayLoadController?.loadReplay(createFileReplaySource(file));
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
  moduleRuntimeController.renderTimelineEventCount();
  mechanicsReviewController?.render();
  eventWindowsManager.renderPlaylistWindow();
  if (options.initialBundle) {
    void replayLoadController?.loadReplayBundleForDisplay(
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
    replayLoadController?.loadReplayFromLocation(listeners.signal);
  }

  mechanicsReviewController?.loadFromLocation(listeners.signal);

  return {
    root,
    destroy: cleanup,
  };
}
