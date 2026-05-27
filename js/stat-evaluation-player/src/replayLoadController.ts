import {
  createBallchasingOverlayPlugin,
  createBoostPickupAnimationPlugin,
  createCanvasRecorderPlugin,
  createTimelineOverlayPlugin,
  ReplayPlayer,
  timelineEventSeekTime,
} from "@rlrml/player";
import type {
  BoostPickupAnimationPickup,
  CanvasRecorderPlugin,
  CanvasRecorderStatus,
  ReplayTimelineEvent,
  TimelineOverlayPlugin,
} from "@rlrml/player";
import type { ReplayLoadModalController } from "./replayLoadModal.ts";
import {
  formatReplayLoadProgress,
  type ReplayLoadBundle,
} from "./replayLoader.ts";
import { getReplayFetchRequestFromSearch, type ReplayFetchRequest } from "./replayUrl.ts";
import {
  createRemoteReplaySource,
  loadReplayBundleFromSource,
  type ReplayInputSource,
} from "./replayInputSources.ts";
import { createStatRegistry, type StatDefinition } from "./statRegistry.ts";
import {
  filterReplayTimelineEvents,
} from "./timelineMarkers.ts";
import type {
  StatsFrameLookup,
  StatsTimeline,
} from "./statsTimeline.ts";
import type { StatsPlayerConfig } from "./playerConfig.ts";

export interface ReplayLoadControllerOptions {
  defaultCameraDistanceScale: number;
  emptyState: HTMLElement;
  fileInput: HTMLInputElement;
  replayLoadModal: ReplayLoadModalController;
  statusReadout: HTMLElement;
  viewport: HTMLElement;
  getActiveTimelineEventSourceIds: () => Set<string>;
  getInitialConfig: () => StatsPlayerConfig | null;
  getInitialSkipKickoffsEnabled: () => boolean;
  getInitialSkipPostGoalTransitionsEnabled: () => boolean;
  getReplayPlayer: () => ReplayPlayer | null;
  includeBoostPickupAnimationPickup: (pickup: BoostPickupAnimationPickup) => boolean;
  applyConfigToReplayPlayer: (config: StatsPlayerConfig) => void;
  clearRenderCaches: () => void;
  clearStandalonePlugins: () => void;
  clearTimelineEventSources: () => void;
  clearTimelineRangeSources: () => void;
  eventWindowsRenderPlaylistWindow: () => void;
  eventWindowsRenderTimelineControls: () => void;
  eventWindowsResetPlaylistState: () => void;
  eventWindowsSyncPlaylistTimeline: (
    state: ReturnType<ReplayPlayer["getState"]>,
    options?: { forceScroll?: boolean },
  ) => void;
  migrateMechanicBackedTimelineEventSelections: () => void;
  recordingSync: (status?: CanvasRecorderStatus | null) => void;
  renderModuleSettings: () => void;
  renderScoreboard: (frameIndex?: number) => void;
  renderSnapshot: (state: ReturnType<ReplayPlayer["getState"]>) => void;
  renderTimelineEventCount: () => void;
  setCanvasRecorder: (recorder: CanvasRecorderPlugin | null) => void;
  setIsApplyingConfig: (isApplying: boolean) => void;
  setLoadedReplayName: (name: string | null) => void;
  setReplayDetails: (playersText: string, frameCount: number) => void;
  setReplayPlayer: (player: ReplayPlayer | null) => void;
  setStatRegistry: (registry: StatDefinition[]) => void;
  setStatsFrameLookup: (lookup: StatsFrameLookup | null) => void;
  setStatsTimeline: (timeline: StatsTimeline | null) => void;
  setTimelineOverlay: (overlay: TimelineOverlayPlugin | null) => void;
  setTransportEnabled: (enabled: boolean) => void;
  setUnsubscribe: (unsubscribe: (() => void) | null) => void;
  setupActiveModules: () => void;
  statsWindowsRender: (frameIndex: number) => void;
  syncBoostPadOverlayPlugin: () => void;
  syncCameraAvailability: (state?: ReturnType<ReplayPlayer["getState"]>) => void;
  teardownActiveModules: () => void;
  unsubscribeCurrent: () => void;
}

export interface ReplayLoadController {
  loadReplay(source: ReplayInputSource): Promise<void>;
  loadReplayBundleForDisplay(
    source: ReplayInputSource,
    bundlePromise: Promise<ReplayLoadBundle>,
  ): Promise<void>;
  loadReplayFromLocation(signal: AbortSignal): void;
}

function withTimelineEventSeekTimes(events: ReplayTimelineEvent[]): ReplayTimelineEvent[] {
  return events.map((event) => ({
    ...event,
    seekTime: timelineEventSeekTime(event),
  }));
}

export function createReplayLoadController(
  options: ReplayLoadControllerOptions,
): ReplayLoadController {
  async function loadReplayBundleForDisplay(
    source: ReplayInputSource,
    bundlePromise: Promise<ReplayLoadBundle>,
  ): Promise<void> {
    options.statusReadout.textContent = source.preparingStatus;
    options.fileInput.disabled = true;
    options.replayLoadModal.show(source.name, source.preparingStatus);
    options.setTransportEnabled(false);
    options.syncCameraAvailability();
    options.emptyState.hidden = false;

    options.unsubscribeCurrent();
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
    options.eventWindowsResetPlaylistState();
    options.renderScoreboard();
    options.renderTimelineEventCount();
    options.eventWindowsRenderTimelineControls();
    options.eventWindowsRenderPlaylistWindow();
    options.renderModuleSettings();
    options.recordingSync();

    try {
      options.statusReadout.textContent = "Parsing replay...";
      options.replayLoadModal.show(source.name, "Parsing replay...");
      const loadedReplay = await bundlePromise;
      const { replay } = loadedReplay;
      options.setStatsTimeline(loadedReplay.statsTimeline);
      options.setStatsFrameLookup(loadedReplay.statsFrameLookup);
      options.setStatRegistry(createStatRegistry(null));
      options.migrateMechanicBackedTimelineEventSelections();

      const timelineOverlay = createTimelineOverlayPlugin({
        replayEventsLabel: "Replay",
        replayEvents: (context) =>
          withTimelineEventSeekTimes(
            filterReplayTimelineEvents(context.replay, options.getActiveTimelineEventSourceIds()),
          ),
      });
      options.setTimelineOverlay(timelineOverlay);

      const recorder = createCanvasRecorderPlugin({
        onStatusChange: (status) => options.recordingSync(status),
      });
      options.setCanvasRecorder(recorder);

      const config = options.getInitialConfig();
      const replayPlayer = new ReplayPlayer(options.viewport, replay, {
        initialPlaybackRate: config?.playback.rate,
        initialCameraDistanceScale:
          config?.camera.distanceScale ?? options.defaultCameraDistanceScale,
        initialCustomCameraSettings: config?.camera.customSettings ?? null,
        initialAttachedPlayerId: config?.camera.attachedPlayerId ?? null,
        initialCameraViewMode: config?.camera.mode,
        initialBallCamEnabled: config?.camera.ballCam ?? false,
        initialBoostPickupAnimationEnabled: config?.overlays.boostPickupAnimation ?? false,
        initialSkipPostGoalTransitionsEnabled: options.getInitialSkipPostGoalTransitionsEnabled(),
        initialSkipKickoffsEnabled: options.getInitialSkipKickoffsEnabled(),
        plugins: [
          createBallchasingOverlayPlugin(),
          createBoostPickupAnimationPlugin({
            includePickup: options.includeBoostPickupAnimationPickup,
          }),
          recorder,
          timelineOverlay,
        ],
      });
      options.setReplayPlayer(replayPlayer);
      options.syncBoostPadOverlayPlugin();

      options.setupActiveModules();
      options.setUnsubscribe(replayPlayer.subscribe(options.renderSnapshot));
      if (config) {
        options.setIsApplyingConfig(true);
        try {
          options.applyConfigToReplayPlayer(config);
        } finally {
          options.setIsApplyingConfig(false);
        }
      }

      options.emptyState.hidden = true;
      options.statusReadout.textContent = `Loaded ${source.name}`;
      options.setLoadedReplayName(source.name);
      options.setReplayDetails(
        replay.players.map((player) => player.name).join(", "),
        replay.frameCount,
      );
      options.renderTimelineEventCount();
      options.eventWindowsRenderTimelineControls();
      options.eventWindowsResetPlaylistState();
      options.eventWindowsRenderPlaylistWindow();
      options.setTransportEnabled(true);
      options.syncCameraAvailability(replayPlayer.getState());
      options.renderSnapshot(replayPlayer.getState());
      options.statsWindowsRender(replayPlayer.getState().frameIndex);
      options.renderScoreboard(replayPlayer.getState().frameIndex);
      options.eventWindowsSyncPlaylistTimeline(replayPlayer.getState(), { forceScroll: true });
      options.renderModuleSettings();
      options.recordingSync();
      options.replayLoadModal.hide();
    } catch (error) {
      options.replayLoadModal.hide();
      options.getReplayPlayer()?.destroy();
      options.setReplayPlayer(null);
      options.setCanvasRecorder(null);
      options.recordingSync();
      throw error;
    } finally {
      options.fileInput.disabled = false;
    }
  }

  return {
    async loadReplay(source) {
      await loadReplayBundleForDisplay(
        source,
        Promise.resolve().then(() =>
          loadReplayBundleFromSource(source, (progress) => {
            options.statusReadout.textContent = formatReplayLoadProgress(progress);
            options.replayLoadModal.update(progress);
          }),
        ),
      );
    },
    loadReplayBundleForDisplay,
    loadReplayFromLocation(signal) {
      let replayRequest: ReplayFetchRequest | null;
      try {
        replayRequest = getReplayFetchRequestFromSearch(
          window.location.search,
          window.location.href,
        );
      } catch (error) {
        console.error("Invalid replay URL:", error);
        options.statusReadout.textContent =
          error instanceof Error ? error.message : "Invalid replay URL";
        return;
      }

      if (!replayRequest) {
        return;
      }

      void this.loadReplay(createRemoteReplaySource(replayRequest, signal)).catch((error) => {
        if (signal.aborted) {
          return;
        }
        console.error("Failed to load replay URL:", error);
        options.statusReadout.textContent =
          error instanceof Error ? error.message : "Failed to load replay URL";
      });
    },
  };
}
