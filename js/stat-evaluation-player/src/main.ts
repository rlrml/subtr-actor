import "./styles.css";
import { ReplayPlayer } from "@rlrml/player";
import type {
  CanvasRecorderPlugin,
  ReplayTimelineEvent,
  ReplayPlayerState,
  TimelineOverlayPlugin,
} from "@rlrml/player";
import { getAppTemplate } from "./appTemplate.ts";
import {
  getStatEvaluationPlayerElements,
  type StatEvaluationPlayerElements,
} from "./appElements.ts";
import { installStatEvaluationPlayerEventListeners } from "./appEventListeners.ts";
import { createReplayLoadModal } from "./replayLoadModal.ts";
import type { ReplayLoadModalController } from "./replayLoadModal.ts";
import { FloatingWindowController } from "./floatingWindows.ts";
import { getStatsFrameForReplayFrame } from "./statsTimeline.ts";
import type {
  StatsFrame,
  StatsFrameLookup,
  StatsTimeline,
} from "./statsTimeline.ts";
import { createStatRegistry, type StatDefinition } from "./statRegistry.ts";
import type { ReplayLoadBundle } from "./replayLoader.ts";
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
  createReplaySnapshotRenderer,
  type ReplaySnapshotRenderer,
} from "./replaySnapshotRenderer.ts";
import type { ReplayLoadController } from "./replayLoadController.ts";
import { renderScoreboardWindow } from "./scoreboardWindow.ts";
import type { ModuleRuntimeController } from "./moduleRuntimeController.ts";
import {
  createMechanicsReviewController,
  getMechanicsReviewElements,
  type MechanicsReviewController,
} from "./mechanicsReviewController.ts";
import { createReplayCueingController } from "./replayCueing.ts";
import type { EventWindowsManager } from "./eventWindows.ts";
import {
  getReplayPlayerStatePatchFromConfig,
} from "./appConfigSnapshot.ts";
import {
  type SingletonWindowId,
  type StatsPlayerConfig,
} from "./playerConfig.ts";
import { loadInitialStatsPlayerConfig } from "./appInitialConfig.ts";
import {
  createStatsPlayerConfigUrlSyncController,
  type StatsPlayerConfigUrlSyncController,
} from "./appConfigUrlSync.ts";
import { createStandalonePluginController } from "./standalonePlugins.ts";
import { createAppStatsWindowsManager } from "./appStatsWindowsManager.ts";
import { createAppEventWindowsManager } from "./appEventWindowsManager.ts";
import { createAppModuleRuntimeController } from "./appModuleRuntimeController.ts";
import { createAppReplayLoadController } from "./appReplayLoadController.ts";

const DEFAULT_CAMERA_DISTANCE_SCALE = 2.25;
const GOAL_WATCH_LEAD_SECONDS = 4;
const PLAYING_SNAPSHOT_UI_INTERVAL_MS = 100;

let replayPlayer: ReplayPlayer | null = null;
let timelineOverlay: TimelineOverlayPlugin | null = null;
let canvasRecorder: CanvasRecorderPlugin | null = null;
let statsTimeline: StatsTimeline | null = null;
let statsFrameLookup: StatsFrameLookup | null = null;
let unsubscribe: (() => void) | null = null;

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
let appElements!: StatEvaluationPlayerElements;
let replayLoadModal: ReplayLoadModalController | null = null;
let currentMountCleanup: (() => void) | null = null;
let statRegistry: StatDefinition[] = createStatRegistry(null);
let loadedReplayName: string | null = null;
let initialUrlConfig: StatsPlayerConfig | null = null;
let configUrlSyncController: StatsPlayerConfigUrlSyncController | null = null;

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
const standalonePluginController = createStandalonePluginController({
  getReplayPlayer() {
    return replayPlayer;
  },
});

let mechanicsReviewController: MechanicsReviewController | null = null;
let recordingControls: RecordingControls | null = null;
let cameraControls: CameraControls | null = null;
let replayLoadController: ReplayLoadController | null = null;
let replaySnapshotRenderer: ReplaySnapshotRenderer | null = null;
let moduleRuntimeController: ModuleRuntimeController;
let eventWindowsManager: EventWindowsManager;
let cueTimelineEvent: (event: ReplayTimelineEvent) => void = () => {};
let watchGoalReplay: (time: number, scorerId: string | null) => void = () => {};

const statsWindowManager = createAppStatsWindowsManager({
  floatingWindows,
  goalWatchLeadSeconds: GOAL_WATCH_LEAD_SECONDS,
  getElements() {
    return appElements;
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
  scheduleConfigUrlUpdate,
  setLauncherOpen,
  watchGoalReplay,
});

moduleRuntimeController = createAppModuleRuntimeController({
  statsWindowManager,
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
  renderModuleRuntimeViews: afterModuleRuntimeChange,
  renderTimelineEventCountValue(value) {
    appElements.eventsReadout.textContent = value;
  },
  requestConfigSync: scheduleConfigUrlUpdate,
});

eventWindowsManager = createAppEventWindowsManager({
  cueTimelineEvent,
  formatTime,
  getElements() {
    return appElements;
  },
  getModuleRuntimeController() {
    return moduleRuntimeController;
  },
  getReplayPlayer() {
    return replayPlayer;
  },
  renderModuleSettings,
  renderModuleSummary,
  scheduleConfigUrlUpdate,
});

function afterModuleRuntimeChange(): void {
  renderModuleSummary();
  renderModuleSettings();
}

function clearStandalonePlugins(): void {
  standalonePluginController.clear();
}

function syncBoostPadOverlayPlugin(): void {
  standalonePluginController.syncBoostPadOverlayPlugin();
}

function toggleBoostPadOverlay(): void {
  standalonePluginController.toggleBoostPadOverlay();
  renderModuleSummary();
  scheduleConfigUrlUpdate();
}

function scheduleConfigUrlUpdate(): void {
  configUrlSyncController?.schedule();
}

function applyConfigToStaticControls(config: StatsPlayerConfig): void {
  moduleRuntimeController.setOverlayConfig(config.overlays);
  standalonePluginController.setBoostPadOverlayEnabled(config.overlays.boostPads);
  appElements.skipPostGoalTransitions.checked =
    config.playback.skipPostGoalTransitions ?? appElements.skipPostGoalTransitions.checked;
  appElements.skipKickoffs.checked =
    config.playback.skipKickoffs ?? appElements.skipKickoffs.checked;
  if (config.playback.rate !== undefined) {
    appElements.playbackRate.value = `${config.playback.rate}`;
  }
  recordingControls?.applyConfig(config.recording);
  moduleRuntimeController.applyModuleConfigSnapshot(config.moduleConfigs);
  floatingWindows.applyWindowConfigs(config.singletonWindows);
  statsWindowManager.replaceFromConfig(config.statsWindows);
  renderModuleSummary();
  renderModuleSettings();
  moduleRuntimeController.renderTimelineEventCount();
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
  appElements.launcherMenu.hidden = !open;
  appElements.launcherToggle.setAttribute("aria-label", open ? "Close menu" : "Open menu");
  appElements.launcherToggle.setAttribute("aria-expanded", open ? "true" : "false");
}

function openReplayFilePicker(): void {
  appElements.fileInput.click();
  setLauncherOpen(false);
}

function renderModuleSummary(): void {
  moduleRuntimeController.renderModuleSummary(appElements.moduleSummaryEl, {
    boostPadOverlayEnabled: standalonePluginController.isBoostPadOverlayEnabled(),
    toggleBoostPadOverlay,
  });
}

function renderModuleSettings(): void {
  moduleRuntimeController.renderModuleSettings(
    appElements.moduleSettingsEl,
    appElements.touchControlsWindowBody,
  );
  moduleRuntimeController.renderBoostPickupFiltersWindow(appElements.boostPickupFiltersWindowBody);
}

function renderScoreboard(frameIndex = replayPlayer?.getState().frameIndex ?? 0): void {
  if (!appElements.scoreboardWindowBody) {
    return;
  }

  renderScoreboardWindow(
    appElements.scoreboardWindowBody,
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
  appElements.togglePlayback.disabled = !enabled;
  appElements.playbackRate.disabled = !enabled;
  appElements.skipPostGoalTransitions.disabled = !enabled;
  appElements.skipKickoffs.disabled = !enabled;
  cameraControls?.setEnabled(enabled);
}

function renderSnapshot(state: ReplayPlayerState): void {
  replaySnapshotRenderer?.render(state);
}

export function mountStatEvaluationPlayer(
  root: HTMLElement,
  options: StatEvaluationPlayerMountOptions = {},
): StatEvaluationPlayerHandle {
  currentMountCleanup?.();

  root.innerHTML = getAppTemplate(DEFAULT_CAMERA_DISTANCE_SCALE);
  appRoot = root;
  replayLoadModal = createReplayLoadModal(root);

  appElements = getStatEvaluationPlayerElements(root);
  replaySnapshotRenderer = createReplaySnapshotRenderer({
    elements: appElements,
    playingUiUpdateIntervalMs: PLAYING_SNAPSHOT_UI_INTERVAL_MS,
    statsWindowManager,
    getCameraControls() {
      return cameraControls;
    },
    getEventWindowsManager() {
      return eventWindowsManager;
    },
    getMechanicsReviewController() {
      return mechanicsReviewController;
    },
    renderScoreboard,
  });
  configUrlSyncController = createStatsPlayerConfigUrlSyncController({
    getLocation() {
      return window.location;
    },
    getSnapshotOptions() {
      return {
        boostPadOverlayEnabled: standalonePluginController.isBoostPadOverlayEnabled(),
        cameraControls,
        elements: appElements,
        floatingWindows,
        moduleRuntimeController,
        recordingControls,
        replayPlayer,
        singletonWindowIds: SINGLETON_WINDOW_IDS,
        statsWindowManager,
      };
    },
    replaceUrl(url) {
      window.history.replaceState(window.history.state, "", url);
    },
  });
  ({ cueTimelineEvent, watchGoalReplay } = createReplayCueingController({
    elements: appElements,
    getCameraControls() {
      return cameraControls;
    },
    getMechanicsReviewController() {
      return mechanicsReviewController;
    },
    getReplayPlayer() {
      return replayPlayer;
    },
    scheduleConfigUrlUpdate,
  }));

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
      appElements.statusReadout.textContent = message;
    },
  });

  cameraControls = createCameraControls({
    elements: getCameraControlElements(root),
    getReplayPlayer() {
      return replayPlayer;
    },
    scheduleConfigUrlUpdate,
  });

  replayLoadController = createAppReplayLoadController({
    defaultCameraDistanceScale: DEFAULT_CAMERA_DISTANCE_SCALE,
    elements: appElements,
    replayLoadModal,
    getInitialConfig() {
      return initialUrlConfig;
    },
    getReplayPlayer() {
      return replayPlayer;
    },
    getModuleRuntimeController() {
      return moduleRuntimeController;
    },
    getEventWindowsManager() {
      return eventWindowsManager;
    },
    getRecordingControls() {
      return recordingControls;
    },
    getCameraControls() {
      return cameraControls;
    },
    applyConfigToReplayPlayer,
    clearStandalonePlugins,
    renderModuleSettings,
    renderScoreboard,
    renderSnapshot,
    setTransportEnabled,
    setCanvasRecorder(recorder) {
      canvasRecorder = recorder;
    },
    setIsApplyingConfig(isApplying) {
      configUrlSyncController?.setApplyingConfig(isApplying);
    },
    setLoadedReplayName(name) {
      loadedReplayName = name;
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
    setUnsubscribe(nextUnsubscribe) {
      unsubscribe = nextUnsubscribe;
    },
    statsWindowsRender(frameIndex) {
      statsWindowManager.render(frameIndex);
    },
    syncBoostPadOverlayPlugin,
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
      appElements.skipPostGoalTransitions.checked = false;
      appElements.skipKickoffs.checked = false;
    },
    clearFreeCameraPreset() {
      cameraControls?.clearFreePreset();
    },
    showWindow,
    setStatusReadout(message) {
      appElements.statusReadout.textContent = message;
    },
    updateReplayLoadModal(progress) {
      replayLoadModal?.update(progress);
    },
  });

  initialUrlConfig = loadInitialStatsPlayerConfig({
    initialConfig: options.initialConfig,
    location: window.location,
    setStatus(message) {
      appElements.statusReadout.textContent = message;
    },
  });

  const listeners = new AbortController();
  floatingWindows.installDragging(appElements.floatingWindowLayer, listeners.signal);
  floatingWindows.installDragging(appElements.statsWindowLayer, listeners.signal);
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
    moduleRuntimeController.reset();
    replayLoadModal?.destroy();
    replayLoadModal = null;
    eventWindowsManager.resetPlaylistState();
    mechanicsReviewController?.reset();
    mechanicsReviewController = null;
    recordingControls = null;
    cameraControls = null;
    replayLoadController = null;
    replaySnapshotRenderer = null;
    standalonePluginController.reset();
    loadedReplayName = null;
    initialUrlConfig = null;
    configUrlSyncController?.reset();
    configUrlSyncController = null;
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
    configUrlSyncController?.setApplyingConfig(true);
    try {
      applyConfigToStaticControls(initialUrlConfig);
    } finally {
      configUrlSyncController?.setApplyingConfig(false);
    }
  }

  installStatEvaluationPlayerEventListeners({
    root,
    elements: appElements,
    signal: listeners.signal,
    createStatsWindow(kind) {
      statsWindowManager.create(kind);
    },
    getCameraControls() {
      return cameraControls;
    },
    getElementWindowId(element) {
      return floatingWindows.getElementWindowId(element);
    },
    getMechanicsReviewController() {
      return mechanicsReviewController;
    },
    getRecordingControls() {
      return recordingControls;
    },
    getReplayLoadController() {
      return replayLoadController;
    },
    getReplayPlayer() {
      return replayPlayer;
    },
    hideWindow,
    openReplayFilePicker,
    scheduleConfigUrlUpdate,
    setLauncherOpen,
    toggleWindow,
  });

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
      appElements.statusReadout.textContent =
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
