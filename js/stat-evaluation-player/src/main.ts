import "./styles.css";
import { timelineEventSeekTime } from "@rlrml/player";
import type { StatsReplayPlayer } from "./statsReplayPlayer.ts";
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
import type { StatModuleContext } from "./statModules.ts";
import { createBoostPickupFilterController } from "./boostPickupFilters.ts";
import type { StatsFrameLookup, StatsTimeline } from "./statsTimeline.ts";
import { createStatRegistry, type StatDefinition } from "./statRegistry.ts";
import {
  createStatsWindowsController,
  formatTime,
  type RenderStatsWindowsOptions,
  type StatsWindowsController,
} from "./statsWindows.ts";
import { filterReplayTimelineEvents } from "./timelineMarkers.ts";
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
  loadReplayBundleFromSource,
  type ReplayInputSource,
} from "./replaySources.ts";
import { loadReplayBundleForDisplay as loadReplayBundleForDisplayRuntime } from "./replayDisplayRuntime.ts";
import {
  createRecordingWindowController,
  type RecordingWindowController,
} from "./recordingWindow.ts";
import {
  createScoreboardWindowController,
  type ScoreboardWindowController,
} from "./scoreboardWindow.ts";
import {
  createPlaybackReadoutsController,
  type PlaybackReadoutsController,
} from "./playbackReadouts.ts";
import { installMountEventListeners } from "./mountEventListeners.ts";
import { createActiveModulesRuntime } from "./activeModulesRuntime.ts";
import { getMechanicsReviewMechanicKind, type MechanicsReviewItem } from "./mechanicsReview.ts";
import { createMechanicsReviewReplayLoadsController } from "./mechanicsReviewReplayLoads.ts";
import {
  createMechanicsReviewWindowController,
  type MechanicsReviewWindowController,
} from "./mechanicsReviewWindow.ts";
import { installInitialReplayLoads } from "./initialReplayLoads.ts";
import {
  getStatsPlayerConfigParamSnapshot,
  getStatsPlayerConfigFromLocation,
  isStatsPlayerConfigDebugEnabled,
  type StatsPlayerConfig,
  type StatsWindowKind,
  type SingletonWindowId,
  type WindowPlacementConfig,
} from "./playerConfig.ts";
import { logStatsPlayerConfigLoadDebug } from "./playerConfigRuntime.ts";
import { createPlaybackActionController } from "./playbackActions.ts";
import { createPlayerConfigBindings, type PlayerConfigBindings } from "./playerConfigBindings.ts";
import { createWindowCommandController } from "./windowCommands.ts";
import { ShotVisualizationController } from "./shotVisualization.ts";

const GOAL_WATCH_LEAD_SECONDS = 4;
const PLAYING_SNAPSHOT_UI_INTERVAL_MS = 100;

let replayPlayer: StatsReplayPlayer | null = null;
let timelineOverlay: TimelineOverlayPlugin | null = null;
let canvasRecorder: CanvasRecorderPlugin | null = null;
let statsTimeline: StatsTimeline | null = null;
let statsFrameLookup: StatsFrameLookup | null = null;
let unsubscribe: (() => void) | null = null;
let lastPlayingSnapshotUiUpdateAt = 0;

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

const activeModulesRuntime = createActiveModulesRuntime({
  modules: MODULES,
  boostPickupFilters,
  getContext: getModuleContext,
  getReplayPlayer: () => replayPlayer,
  getTimelineOverlay: () => timelineOverlay,
  getEventTimelineSources,
  withTimelineEventSeekTimes,
  renderModuleSummary,
  renderModuleSettings,
  renderStatsWindows() {
    if (replayPlayer) {
      renderStatsWindows(replayPlayer.getState().frameIndex);
    }
  },
  renderTimelineEventCount,
  requestConfigSync: scheduleConfigUrlUpdate,
});

const playbackActions = createPlaybackActionController({
  goalWatchLeadSeconds: GOAL_WATCH_LEAD_SECONDS,
  getReplayPlayer: () => replayPlayer,
  getCameraControlsController: () => cameraControlsController,
  getMechanicsReviewController: () => mechanicsReviewController,
  getSkipPostGoalTransitions: () => skipPostGoalTransitions,
  getSkipKickoffs: () => skipKickoffs,
  syncBoostPadOverlayPlugin,
  setupActiveModules,
  renderModuleSummary,
  renderModuleSettings,
  renderStatsWindows(frameIndex) {
    renderStatsWindows(frameIndex);
  },
  scheduleConfigUrlUpdate,
});

const windowCommands = createWindowCommandController({
  getFloatingWindowController: () => floatingWindowController,
  getLauncherMenu: () => launcherMenu,
  getLauncherToggle: () => launcherToggle,
  getFileInput: () => fileInput,
});

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
let hitboxOnlyMode!: HTMLInputElement;
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
let playbackReadoutsController: PlaybackReadoutsController | null = null;
let shotVisualizationController: ShotVisualizationController | null = null;
let configBindings: PlayerConfigBindings | null = null;
let loadedReplayName: string | null = null;
let initialUrlConfig: StatsPlayerConfig | null = null;

function getActiveCapabilityIds(kind: ModuleCapabilityKind): ReadonlySet<string> {
  return activeModulesRuntime.getActiveCapabilityIds(kind);
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
    // The viewer renders 1:1 in Unreal Units (no @rlrml/player fieldScale).
    fieldScale: 1,
  };
}

function setupActiveModules(): void {
  activeModulesRuntime.setupActiveModules();
}

function migrateMechanicBackedTimelineEventSelections(): void {
  activeModulesRuntime.migrateMechanicBackedTimelineEventSelections();
}

function teardownActiveModules(): void {
  activeModulesRuntime.teardownActiveModules();
}

function toggleCapability(id: string, kind: ModuleCapabilityKind, enabled: boolean): void {
  activeModulesRuntime.toggleCapability(id, kind, enabled);
}

function clearTimelineEventSources(): void {
  activeModulesRuntime.clearTimelineEventSources();
}

function clearTimelineRangeSources(): void {
  activeModulesRuntime.clearTimelineRangeSources();
}

function clearStandalonePlugins(): void {
  activeModulesRuntime.clearStandalonePlugins();
}

function syncBoostPadOverlayPlugin(): void {
  activeModulesRuntime.syncBoostPadOverlayPlugin();
}

function toggleBoostPadOverlay(): void {
  activeModulesRuntime.toggleBoostPadOverlay();
}

function syncTimelineEvents(): void {
  activeModulesRuntime.syncTimelineEvents();
}

function syncTimelineRanges(): void {
  activeModulesRuntime.syncTimelineRanges();
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

function readWindowPlacement(windowEl: HTMLElement): WindowPlacementConfig {
  if (!floatingWindowController) {
    throw new Error("Floating windows are not initialized.");
  }
  return floatingWindowController.readPlacement(windowEl);
}

function applyWindowPlacement(windowEl: HTMLElement, placement: WindowPlacementConfig): void {
  floatingWindowController?.applyPlacement(windowEl, placement);
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

function clearStatsWindows(): void {
  statsWindowsController?.clear();
}

function scheduleConfigUrlUpdate(): void {
  configBindings?.scheduleConfigUrlUpdate();
}

function applyConfigToStaticControls(config: StatsPlayerConfig): void {
  configBindings?.applyConfigToStaticControls(config);
}

function withTimelineEventSeekTimes(events: ReplayTimelineEvent[]): ReplayTimelineEvent[] {
  return events.map((event) => ({
    ...event,
    seekTime: timelineEventSeekTime(event),
  }));
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

function isSingletonWindowVisible(id: string): boolean {
  const windowEl = appRoot?.querySelector<HTMLElement>(`[data-window-id="${id}"]`);
  return Boolean(windowEl && !windowEl.hidden);
}

function syncEventPlaylistTimeline(
  state: ReplayPlayerState,
  options: SyncEventPlaylistTimelineOptions = {},
): void {
  if (!options.forceScroll && !isSingletonWindowVisible("event-playlist")) {
    return;
  }
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

  activeModulesRuntime.activateMechanicTimelineKind(mechanic);
  renderMechanicsTimelineControls();
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

function renderShotVisualization(state = replayPlayer?.getState() ?? null): void {
  if (!isSingletonWindowVisible("shot-visualization")) {
    return;
  }
  shotVisualizationController?.render(state);
}

function toggleSingletonWindow(id: SingletonWindowId): void {
  windowCommands.toggleWindow(id);
  if (!isSingletonWindowVisible(id)) {
    return;
  }

  if (id === "event-playlist") {
    renderEventPlaylistWindow();
    const state = replayPlayer?.getState();
    if (state) {
      syncEventPlaylistTimeline(state, { forceScroll: true });
    }
  }
  if (id === "shot-visualization") {
    renderShotVisualization(replayPlayer?.getState() ?? null);
  }
}

function setTransportEnabled(enabled: boolean): void {
  playbackReadoutsController?.setTransportEnabled(enabled, replayPlayer?.getState());
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

  playbackReadoutsController?.renderSnapshot(state);

  renderStatsWindows(state.frameIndex, { preserveOpenPickers: true });
  renderScoreboard(state.frameIndex);
  renderShotVisualization(state);
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
  await loadReplayBundleForDisplayRuntime(source, bundlePromise, {
    elements: {
      fileInput,
      viewport,
      emptyState,
      statusReadout,
      playersReadout,
      framesReadout,
      skipPostGoalTransitions,
      skipKickoffs,
      hitboxWireframes,
      hitboxOnlyMode,
    },
    getReplayLoadModal: () => replayLoadModal,
    getReplayPlayer: () => replayPlayer,
    setReplayPlayer(value) {
      replayPlayer = value;
    },
    getUnsubscribe: () => unsubscribe,
    setUnsubscribe(value) {
      unsubscribe = value;
    },
    setCanvasRecorder(value) {
      canvasRecorder = value;
    },
    setLoadedReplayName(value) {
      loadedReplayName = value;
    },
    setTimelineOverlay(value) {
      timelineOverlay = value;
    },
    setStatsTimeline(value) {
      statsTimeline = value;
    },
    setStatsFrameLookup(value) {
      statsFrameLookup = value;
    },
    setStatRegistry(value) {
      statRegistry = value;
    },
    getInitialConfig: () => initialUrlConfig,
    setApplyingConfig(value) {
      configBindings?.setApplyingConfig(value);
    },
    getReplayTimelineEvents(replay) {
      return filterReplayTimelineEvents(
        replay,
        activeModulesRuntime.getActiveTimelineEventSourceIds(),
      );
    },
    withTimelineEventSeekTimes,
    includeBoostPickupAnimationPickup,
    syncRecordingWindow,
    setTransportEnabled,
    teardownActiveModules,
    clearTimelineEventSources,
    clearTimelineRangeSources,
    clearStandalonePlugins,
    clearRenderCaches,
    resetEventPlaylistWindow,
    renderScoreboard,
    renderTimelineEventCount,
    renderMechanicsTimelineControls,
    renderEventPlaylistWindow,
    renderModuleSettings,
    migrateMechanicBackedTimelineEventSelections,
    syncBoostPadOverlayPlugin,
    setupActiveModules,
    renderSnapshot,
    applyConfigToReplayPlayer: playbackActions.applyConfigToReplayPlayer,
    renderStatsWindows,
    syncEventPlaylistTimeline,
    getCameraControlsController: () => cameraControlsController,
  });
}

export function mountStatEvaluationPlayer(
  root: HTMLElement,
  options: StatEvaluationPlayerMountOptions = {},
): StatEvaluationPlayerHandle {
  currentMountCleanup?.();

  root.innerHTML = getAppTemplate();
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
    getActiveTimelineEventSourceIds: () => activeModulesRuntime.getActiveTimelineEventSourceIds(),
    getActiveMechanicTimelineKinds: () => activeModulesRuntime.getActiveMechanicTimelineKinds(),
    toggleEventSource(id, enabled) {
      toggleCapability(id, "events", enabled);
    },
    setMechanicTimelineKind(kind, enabled) {
      activeModulesRuntime.setMechanicTimelineKind(kind, enabled);
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
    cueTimelineEvent: playbackActions.cueTimelineEvent,
    formatTime,
  });
  shotVisualizationController = new ShotVisualizationController({
    body: mustElement<HTMLDivElement>(root, "#shot-visualization-window-body"),
    getReplayPlayer: () => replayPlayer,
    cueTimelineEvent: playbackActions.cueTimelineEvent,
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
      windowCommands.showWindow("replay-loading");
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
    bringWindowToFront: windowCommands.bringWindowToFront,
    setLauncherOpen: windowCommands.setLauncherOpen,
    requestConfigSync: scheduleConfigUrlUpdate,
    watchGoalReplay: playbackActions.watchGoalReplay,
    cueGoalReplay: playbackActions.cueGoalReplay,
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
      usePlayerCameraSettings: mustElement<HTMLInputElement>(root, "#use-player-camera-settings"),
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
      nameplateLift: mustElement<HTMLInputElement>(root, "#custom-nameplate-lift"),
      nameplateLiftReadout: mustElement<HTMLElement>(root, "#custom-nameplate-lift-readout"),
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
    getTimelineSources: () => getEventTimelineSources(getModuleContext()),
    getActiveModules: () => activeModulesRuntime.getActiveModules(),
    getActiveCapabilityIds,
    getBoostPickupAnimationEnabled: () =>
      replayPlayer?.getState().boostPickupAnimationEnabled ?? false,
    getBoostPadOverlayEnabled: () => activeModulesRuntime.getBoostPadOverlayEnabled(),
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
    syncTimelineEvents,
    syncTimelineRanges,
    renderTimelineEventCount,
    requestConfigSync: scheduleConfigUrlUpdate,
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
  hitboxOnlyMode = mustElement<HTMLInputElement>(root, "#hitbox-only-mode");
  playbackReadoutsController = createPlaybackReadoutsController({
    elements: {
      togglePlayback,
      playbackRate,
      skipPostGoalTransitions,
      skipKickoffs,
      hitboxWireframes,
      hitboxOnlyMode,
      emptyState,
      timeReadout,
      frameReadout,
      durationReadout,
      playbackStatusReadout,
    },
    getCameraControlsController: () => cameraControlsController,
  });
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
  configBindings = createPlayerConfigBindings({
    modules: MODULES,
    playbackRate,
    skipPostGoalTransitions,
    skipKickoffs,
    hitboxWireframes,
    hitboxOnlyMode,
    getReplayPlayer: () => replayPlayer,
    getCameraControlsController: () => cameraControlsController,
    getRecordingWindowController: () => recordingWindowController,
    getFloatingWindowController: () => floatingWindowController,
    getStatsWindowsController: () => statsWindowsController,
    getActiveModulesRuntime: () => activeModulesRuntime,
    getInitialConfig: () => initialUrlConfig,
    renderModuleSummary,
    renderModuleSettings,
    renderTimelineEventCount,
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
  windowCommands.installWindowDragging(floatingWindowLayer, listeners.signal);
  windowCommands.installWindowDragging(statsWindowLayer, listeners.signal);
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
    activeModulesRuntime.reset();
    replayLoadModal?.destroy();
    replayLoadModal = null;
    resetEventPlaylistWindow();
    eventTimelineControlsController = null;
    eventPlaylistController = null;
    mechanicsReviewController?.reset();
    mechanicsReviewController = null;
    loadedReplayName = null;
    cameraControlsController = null;
    recordingWindowController = null;
    moduleControlsController = null;
    scoreboardWindowController = null;
    playbackReadoutsController = null;
    shotVisualizationController?.destroy();
    shotVisualizationController = null;
    initialUrlConfig = null;
    configBindings?.reset();
    configBindings = null;
    floatingWindowController?.reset();
    floatingWindowController = null;
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
    configBindings?.setApplyingConfig(true);
    try {
      applyConfigToStaticControls(initialUrlConfig);
    } finally {
      configBindings?.setApplyingConfig(false);
    }
  }

  installMountEventListeners({
    elements: {
      root,
      launcherToggle,
      launcherMenu,
      loadReplayAction,
      emptyLoadReplay,
      fileInput,
      togglePlayback,
      playbackRate,
      skipPostGoalTransitions,
      skipKickoffs,
      hitboxWireframes,
      hitboxOnlyMode,
    },
    signal: listeners.signal,
    setLauncherOpen: windowCommands.setLauncherOpen,
    openReplayFilePicker: windowCommands.openReplayFilePicker,
    getElementWindowId: windowCommands.getElementWindowId,
    toggleWindow: toggleSingletonWindow,
    hideWindow: windowCommands.hideWindow,
    createStatsWindow,
    async loadReplayFile(file) {
      try {
        mechanicsReviewController?.clearCurrentClip({ resetReplayId: true, render: true });
        await loadReplay(createFileReplaySource(file));
      } catch (error) {
        console.error("Failed to load replay:", error);
        statusReadout.textContent =
          error instanceof Error ? error.message : "Failed to load replay";
      }
    },
    togglePlayback() {
      replayPlayer?.togglePlayback();
      scheduleConfigUrlUpdate();
    },
    setPlaybackRate(value) {
      replayPlayer?.setPlaybackRate(value);
      scheduleConfigUrlUpdate();
    },
    setSkipPostGoalTransitionsEnabled(enabled) {
      replayPlayer?.setSkipPostGoalTransitionsEnabled(enabled);
      scheduleConfigUrlUpdate();
    },
    setSkipKickoffsEnabled(enabled) {
      replayPlayer?.setSkipKickoffsEnabled(enabled);
      scheduleConfigUrlUpdate();
    },
    setHitboxWireframesEnabled(enabled) {
      replayPlayer?.setHitboxWireframesEnabled(enabled);
      scheduleConfigUrlUpdate();
    },
    setHitboxOnlyModeEnabled(enabled) {
      replayPlayer?.setHitboxOnlyModeEnabled(enabled);
      scheduleConfigUrlUpdate();
    },
  });

  mechanicsReviewController?.installEventListeners(listeners.signal);
  recordingWindowController?.installEventListeners(listeners.signal);
  cameraControlsController?.installEventListeners(listeners.signal);

  // Allow an embedding parent window (e.g. the Rocket Sense stats UI) to drive
  // the active review clip without reloading the replay, so hovering a goal can
  // scrub the player to that goal's clip. Messages are only honored from the
  // same origin to avoid cross-site control of the player.
  window.addEventListener(
    "message",
    (event: MessageEvent) => {
      if (event.origin !== window.location.origin) {
        return;
      }
      const data = event.data as { source?: unknown; type?: unknown; index?: unknown } | null;
      if (!data || typeof data !== "object" || data.source !== "rocket-sense") {
        return;
      }
      if (
        data.type === "activateReviewItem" &&
        typeof data.index === "number" &&
        Number.isInteger(data.index)
      ) {
        void mechanicsReviewController?.activateItem(data.index);
      }
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
  installInitialReplayLoads({
    signal: listeners.signal,
    location: window.location,
    statusReadout,
    initialBundle: options.initialBundle,
    initialReplayName: options.initialReplayName,
    loadFromLocation: options.loadFromLocation,
    loadReplay,
    loadReplayBundleForDisplay,
    getMechanicsReviewController: () => mechanicsReviewController,
    showMechanicsReviewWindow() {
      windowCommands.showWindow("mechanics-review");
    },
  });

  return {
    root,
    destroy: cleanup,
  };
}
