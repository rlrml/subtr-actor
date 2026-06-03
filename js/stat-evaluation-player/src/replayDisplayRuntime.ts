import {
  createBallchasingOverlayPlugin,
  createBoostPickupAnimationPlugin,
  createCanvasRecorderPlugin,
  createTimelineOverlayPlugin,
  ReplayPlayer,
  type BoostPickupAnimationPickup,
  type CanvasRecorderPlugin,
  type ReplayTimelineEvent,
  type ReplayPlayerState,
  type TimelineOverlayPlugin,
} from "@rlrml/player";
import type { CameraControlsController } from "./cameraControls.ts";
import type { StatsPlayerConfig } from "./playerConfig.ts";
import type { ReplayLoadModalController } from "./replayLoadModal.ts";
import type { ReplayLoadBundle } from "./replayLoader.ts";
import type { ReplayInputSource } from "./replaySources.ts";
import { createStatRegistry, type StatDefinition } from "./statRegistry.ts";
import type { StatsFrameLookup, StatsTimeline } from "./statsTimeline.ts";

export interface ReplayDisplayElements {
  readonly fileInput: HTMLInputElement;
  readonly viewport: HTMLDivElement;
  readonly emptyState: HTMLDivElement;
  readonly statusReadout: HTMLElement;
  readonly playersReadout: HTMLElement;
  readonly framesReadout: HTMLElement;
  readonly skipPostGoalTransitions: HTMLInputElement;
  readonly skipKickoffs: HTMLInputElement;
  readonly hitboxWireframes: HTMLInputElement;
}

export interface ReplayDisplayRuntimeOptions {
  readonly elements: ReplayDisplayElements;
  readonly defaultCameraDistanceScale: number;
  getReplayLoadModal(): ReplayLoadModalController | null;
  getReplayPlayer(): ReplayPlayer | null;
  setReplayPlayer(value: ReplayPlayer | null): void;
  getUnsubscribe(): (() => void) | null;
  setUnsubscribe(value: (() => void) | null): void;
  setCanvasRecorder(value: CanvasRecorderPlugin | null): void;
  setLoadedReplayName(value: string | null): void;
  setTimelineOverlay(value: TimelineOverlayPlugin | null): void;
  setStatsTimeline(value: StatsTimeline | null): void;
  setStatsFrameLookup(value: StatsFrameLookup | null): void;
  setStatRegistry(value: StatDefinition[]): void;
  getInitialConfig(): StatsPlayerConfig | null;
  setApplyingConfig(value: boolean): void;
  getReplayTimelineEvents(replay: ReplayPlayer["replay"]): ReplayTimelineEvent[];
  withTimelineEventSeekTimes(events: ReplayTimelineEvent[]): ReplayTimelineEvent[];
  includeBoostPickupAnimationPickup(pickup: BoostPickupAnimationPickup): boolean;
  syncRecordingWindow(): void;
  setTransportEnabled(enabled: boolean): void;
  teardownActiveModules(): void;
  clearTimelineEventSources(): void;
  clearTimelineRangeSources(): void;
  clearStandalonePlugins(): void;
  clearRenderCaches(): void;
  resetEventPlaylistWindow(): void;
  renderScoreboard(frameIndex?: number): void;
  renderTimelineEventCount(): void;
  renderMechanicsTimelineControls(): void;
  renderEventPlaylistWindow(): void;
  renderModuleSettings(): void;
  migrateMechanicBackedTimelineEventSelections(): void;
  syncBoostPadOverlayPlugin(): void;
  setupActiveModules(): void;
  renderSnapshot(state: ReplayPlayerState): void;
  applyConfigToReplayPlayer(config: StatsPlayerConfig): void;
  renderStatsWindows(frameIndex: number): void;
  syncEventPlaylistTimeline(state: ReplayPlayerState, options?: { forceScroll?: boolean }): void;
  getCameraControlsController(): CameraControlsController | null;
}

export async function loadReplayBundleForDisplay(
  source: ReplayInputSource,
  bundlePromise: Promise<ReplayLoadBundle>,
  options: ReplayDisplayRuntimeOptions,
): Promise<void> {
  const { elements } = options;
  elements.statusReadout.textContent = source.preparingStatus;
  elements.fileInput.disabled = true;
  options.getReplayLoadModal()?.show(source.name, source.preparingStatus);
  options.setTransportEnabled(false);
  options.getCameraControlsController()?.syncAvailability();
  elements.emptyState.hidden = false;

  const unsubscribe = options.getUnsubscribe();
  if (unsubscribe) {
    unsubscribe();
    options.setUnsubscribe(null);
  }

  options.teardownActiveModules();
  options.getReplayPlayer()?.destroy();
  options.setReplayPlayer(null);
  options.setCanvasRecorder(null);
  options.setLoadedReplayName(null);
  options.setTimelineOverlay(null);
  options.setStatsTimeline(null);
  options.setStatsFrameLookup(null);
  options.setStatRegistry(createStatRegistry(null));
  options.clearTimelineEventSources();
  options.clearTimelineRangeSources();
  options.clearStandalonePlugins();
  options.clearRenderCaches();
  options.resetEventPlaylistWindow();
  options.renderScoreboard();
  options.renderTimelineEventCount();
  options.renderMechanicsTimelineControls();
  options.renderEventPlaylistWindow();
  options.renderModuleSettings();
  options.syncRecordingWindow();

  try {
    elements.statusReadout.textContent = "Parsing replay...";
    options.getReplayLoadModal()?.show(source.name, "Parsing replay...");
    const loadedReplay = await bundlePromise;
    const { replay } = loadedReplay;
    options.setStatsTimeline(loadedReplay.statsTimeline);
    options.setStatsFrameLookup(loadedReplay.statsFrameLookup);
    options.setStatRegistry(createStatRegistry(null));
    options.migrateMechanicBackedTimelineEventSelections();

    const timelineOverlay = createTimelineOverlayPlugin({
      replayEventsLabel: "Replay",
      replayEvents: (context) =>
        options.withTimelineEventSeekTimes(options.getReplayTimelineEvents(context.replay)),
    });
    const recorder = createCanvasRecorderPlugin({
      onStatusChange: options.syncRecordingWindow,
    });
    options.setCanvasRecorder(recorder);
    const config = options.getInitialConfig();

    const replayPlayer = new ReplayPlayer(elements.viewport, replay, {
      initialPlaybackRate: config?.playback.rate,
      initialCameraDistanceScale:
        config?.camera.distanceScale ?? options.defaultCameraDistanceScale,
      initialCustomCameraSettings: config?.camera.customSettings ?? null,
      initialAttachedPlayerId: config?.camera.attachedPlayerId ?? null,
      initialCameraViewMode: config?.camera.mode,
      initialBallCamEnabled: config?.camera.ballCam ?? false,
      initialBoostPickupAnimationEnabled: config?.overlays.boostPickupAnimation ?? false,
      initialHitboxWireframesEnabled:
        config?.overlays.hitboxWireframes ?? elements.hitboxWireframes.checked,
      initialSkipPostGoalTransitionsEnabled: elements.skipPostGoalTransitions.checked,
      initialSkipKickoffsEnabled: elements.skipKickoffs.checked,
      plugins: [
        createBallchasingOverlayPlugin(),
        createBoostPickupAnimationPlugin({
          includePickup: options.includeBoostPickupAnimationPickup,
        }),
        recorder,
        timelineOverlay,
      ],
    });
    options.setTimelineOverlay(timelineOverlay);
    options.setReplayPlayer(replayPlayer);
    options.syncBoostPadOverlayPlugin();

    options.setupActiveModules();
    options.setUnsubscribe(replayPlayer.subscribe(options.renderSnapshot));
    if (config) {
      options.setApplyingConfig(true);
      try {
        options.applyConfigToReplayPlayer(config);
      } finally {
        options.setApplyingConfig(false);
      }
    }

    options.getCameraControlsController()?.populateAttachedPlayerOptions(replay.players);
    elements.emptyState.hidden = true;
    elements.statusReadout.textContent = `Loaded ${source.name}`;
    options.setLoadedReplayName(source.name);
    elements.playersReadout.textContent = replay.players.map((player) => player.name).join(", ");
    elements.framesReadout.textContent = `${replay.frameCount}`;
    options.renderTimelineEventCount();
    options.renderMechanicsTimelineControls();
    options.resetEventPlaylistWindow();
    options.renderEventPlaylistWindow();
    options.setTransportEnabled(true);
    options.getCameraControlsController()?.syncAvailability(replayPlayer.getState());
    options.renderSnapshot(replayPlayer.getState());
    options.renderStatsWindows(replayPlayer.getState().frameIndex);
    options.renderScoreboard(replayPlayer.getState().frameIndex);
    options.syncEventPlaylistTimeline(replayPlayer.getState(), { forceScroll: true });
    options.renderModuleSettings();
    options.syncRecordingWindow();
    options.getReplayLoadModal()?.hide();
  } catch (error) {
    options.getReplayLoadModal()?.hide();
    options.getReplayPlayer()?.destroy();
    options.setReplayPlayer(null);
    options.setCanvasRecorder(null);
    options.syncRecordingWindow();
    throw error;
  } finally {
    elements.fileInput.disabled = false;
  }
}
