import type {
  CanvasRecorderPlugin,
  ReplayPlayer,
  ReplayPlayerState,
  TimelineOverlayPlugin,
} from "@rlrml/player";
import type { StatEvaluationPlayerElements } from "./appElements.ts";
import type { CameraControls } from "./cameraControls.ts";
import type { EventWindowsManager } from "./eventWindows.ts";
import type { ModuleRuntimeController } from "./moduleRuntimeController.ts";
import type { StatsPlayerConfig } from "./playerConfig.ts";
import type { RecordingControls } from "./recordingControls.ts";
import {
  createReplayLoadController,
  type ReplayLoadController,
} from "./replayLoadController.ts";
import type { ReplayLoadModalController } from "./replayLoadModal.ts";
import type { StatDefinition } from "./statRegistry.ts";
import type { StatsFrameLookup, StatsTimeline } from "./statsTimeline.ts";

interface AppReplayLoadControllerDeps {
  readonly defaultCameraDistanceScale: number;
  readonly elements: StatEvaluationPlayerElements;
  readonly replayLoadModal: ReplayLoadModalController;
  applyConfigToReplayPlayer(config: StatsPlayerConfig): void;
  clearStandalonePlugins(): void;
  getCameraControls(): CameraControls | null;
  getEventWindowsManager(): EventWindowsManager;
  getInitialConfig(): StatsPlayerConfig | null;
  getModuleRuntimeController(): ModuleRuntimeController;
  getRecordingControls(): RecordingControls | null;
  getReplayPlayer(): ReplayPlayer | null;
  renderModuleSettings(): void;
  renderScoreboard(frameIndex?: number): void;
  renderSnapshot(state: ReplayPlayerState): void;
  setCanvasRecorder(recorder: CanvasRecorderPlugin | null): void;
  setIsApplyingConfig(isApplying: boolean): void;
  setLoadedReplayName(name: string | null): void;
  setReplayPlayer(player: ReplayPlayer | null): void;
  setStatRegistry(registry: StatDefinition[]): void;
  setStatsFrameLookup(lookup: StatsFrameLookup | null): void;
  setStatsTimeline(timeline: StatsTimeline | null): void;
  setTimelineOverlay(overlay: TimelineOverlayPlugin | null): void;
  setTransportEnabled(enabled: boolean): void;
  setUnsubscribe(unsubscribe: (() => void) | null): void;
  statsWindowsRender(frameIndex: number): void;
  syncBoostPadOverlayPlugin(): void;
  unsubscribeCurrent(): void;
}

export function createAppReplayLoadController(
  deps: AppReplayLoadControllerDeps,
): ReplayLoadController {
  return createReplayLoadController({
    defaultCameraDistanceScale: deps.defaultCameraDistanceScale,
    emptyState: deps.elements.emptyState,
    fileInput: deps.elements.fileInput,
    replayLoadModal: deps.replayLoadModal,
    statusReadout: deps.elements.statusReadout,
    viewport: deps.elements.viewport,
    getActiveTimelineEventSourceIds() {
      return deps.getModuleRuntimeController().getActiveTimelineEventSourceIds();
    },
    getInitialConfig: deps.getInitialConfig,
    getInitialSkipKickoffsEnabled() {
      return deps.elements.skipKickoffs.checked;
    },
    getInitialSkipPostGoalTransitionsEnabled() {
      return deps.elements.skipPostGoalTransitions.checked;
    },
    getReplayPlayer: deps.getReplayPlayer,
    includeBoostPickupAnimationPickup(pickup) {
      return deps.getModuleRuntimeController().includeBoostPickupAnimationPickup(pickup);
    },
    applyConfigToReplayPlayer: deps.applyConfigToReplayPlayer,
    clearRenderCaches() {
      deps.getModuleRuntimeController().clearRenderCaches();
    },
    clearStandalonePlugins: deps.clearStandalonePlugins,
    clearTimelineEventSources() {
      deps.getModuleRuntimeController().clearTimelineEventSources();
    },
    clearTimelineRangeSources() {
      deps.getModuleRuntimeController().clearTimelineRangeSources();
    },
    eventWindowsRenderPlaylistWindow() {
      deps.getEventWindowsManager().renderPlaylistWindow();
    },
    eventWindowsRenderTimelineControls() {
      deps.getEventWindowsManager().renderTimelineControls();
    },
    eventWindowsResetPlaylistState() {
      deps.getEventWindowsManager().resetPlaylistState();
    },
    eventWindowsSyncPlaylistTimeline(state, options) {
      deps.getEventWindowsManager().syncPlaylistTimeline(state, options);
    },
    migrateMechanicBackedTimelineEventSelections() {
      deps.getModuleRuntimeController().migrateMechanicBackedTimelineEventSelections();
    },
    recordingSync(status) {
      deps.getRecordingControls()?.sync(status);
    },
    renderModuleSettings: deps.renderModuleSettings,
    renderScoreboard: deps.renderScoreboard,
    renderSnapshot: deps.renderSnapshot,
    renderTimelineEventCount() {
      deps.getModuleRuntimeController().renderTimelineEventCount();
    },
    setCanvasRecorder: deps.setCanvasRecorder,
    setIsApplyingConfig: deps.setIsApplyingConfig,
    setLoadedReplayName: deps.setLoadedReplayName,
    setReplayDetails(playersText, frameCount) {
      deps.elements.playersReadout.textContent = playersText;
      deps.elements.framesReadout.textContent = `${frameCount}`;
    },
    setReplayPlayer: deps.setReplayPlayer,
    setStatRegistry: deps.setStatRegistry,
    setStatsFrameLookup: deps.setStatsFrameLookup,
    setStatsTimeline: deps.setStatsTimeline,
    setTimelineOverlay: deps.setTimelineOverlay,
    setTransportEnabled: deps.setTransportEnabled,
    setUnsubscribe: deps.setUnsubscribe,
    setupActiveModules() {
      deps.getModuleRuntimeController().setupActiveModules();
    },
    statsWindowsRender: deps.statsWindowsRender,
    syncBoostPadOverlayPlugin: deps.syncBoostPadOverlayPlugin,
    syncCameraAvailability(state) {
      deps.getCameraControls()?.syncAvailability(state);
    },
    teardownActiveModules() {
      deps.getModuleRuntimeController().teardownActiveModules();
    },
    unsubscribeCurrent: deps.unsubscribeCurrent,
  });
}
