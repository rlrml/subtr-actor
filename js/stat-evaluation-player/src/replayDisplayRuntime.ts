import {
  createBallchasingOverlayPlugin,
  createBoostPickupAnimationPlugin,
  createCanvasRecorderPlugin,
  createTimelineOverlayPlugin,
  type BoostPickupAnimationPickup,
  type CanvasRecorderPlugin,
  type ReplayTimelineEvent,
  type ReplayPlayerState,
  type TimelineOverlayPlugin,
} from "@rlrml/player";
import {
  createFpsOverlayPlugin,
  createPlayerFromParsed,
  fromReplayPlayerPlugin,
  SubtrActorPlayer,
} from "@rlrml/player";
import type { StatsReplayPlayer } from "./statsReplayPlayer.ts";
import type { CameraControlsController } from "./cameraControls.ts";
import type { StatsPlayerConfig } from "./playerConfig.ts";
import { getCustomCameraSettingsFromConfig } from "./playerConfigRuntime.ts";
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
  readonly hitboxOnlyMode: HTMLInputElement;
}

export interface ReplayDisplayRuntimeOptions {
  readonly elements: ReplayDisplayElements;
  getReplayLoadModal(): ReplayLoadModalController | null;
  getReplayPlayer(): StatsReplayPlayer | null;
  setReplayPlayer(value: StatsReplayPlayer | null): void;
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
  getReplayTimelineEvents(replay: StatsReplayPlayer["replay"]): ReplayTimelineEvent[];
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
  renderModuleSummary(): void;
  renderScoreboard(frameIndex?: number): void;
  renderTimelineEventCount(): void;
  renderMechanicsTimelineControls(): void;
  renderEventPlaylistWindow(): void;
  renderModuleSettings(): void;
  syncBoostPadOverlayPlugin(): void;
  setupActiveModules(): void;
  renderSnapshot(state: ReplayPlayerState): void;
  applyConfigToReplayPlayer(config: StatsPlayerConfig): void;
  renderStatsWindows(frameIndex: number): void;
  syncEventPlaylistTimeline(state: ReplayPlayerState, options?: { forceScroll?: boolean }): void;
  getCameraControlsController(): CameraControlsController | null;
}

function setReplayPlayerCanvasPending(player: StatsReplayPlayer, pending: boolean): void {
  const { style } = player.renderer.domElement;
  style.visibility = pending ? "hidden" : "";
  style.pointerEvents = pending ? "none" : "";
}

function clearDisplayedReplay(
  options: ReplayDisplayRuntimeOptions,
  settings: { destroyPlayer?: boolean; clearPlayerPluginHandles?: boolean } = {},
): void {
  const destroyPlayer = settings.destroyPlayer ?? true;
  const clearPlayerPluginHandles = settings.clearPlayerPluginHandles ?? true;
  const unsubscribe = options.getUnsubscribe();
  if (unsubscribe) {
    unsubscribe();
    options.setUnsubscribe(null);
  }

  options.teardownActiveModules();
  if (destroyPlayer) {
    options.getReplayPlayer()?.destroy();
    options.setReplayPlayer(null);
  }
  if (clearPlayerPluginHandles) {
    options.setCanvasRecorder(null);
    options.setTimelineOverlay(null);
  }
  options.setLoadedReplayName(null);
  options.setStatsTimeline(null);
  options.setStatsFrameLookup(null);
  options.setStatRegistry(createStatRegistry(null));
  options.clearTimelineEventSources();
  options.clearTimelineRangeSources();
  options.clearStandalonePlugins();
  options.clearRenderCaches();
  options.resetEventPlaylistWindow();
  options.renderModuleSummary();
  options.renderScoreboard();
  options.renderTimelineEventCount();
  options.renderMechanicsTimelineControls();
  options.renderEventPlaylistWindow();
  options.renderModuleSettings();
  options.syncRecordingWindow();
}

export async function loadReplayBundleForDisplay(
  source: ReplayInputSource,
  bundlePromise: Promise<ReplayLoadBundle>,
  options: ReplayDisplayRuntimeOptions,
): Promise<void> {
  const { elements } = options;
  let pendingReplayPlayer: StatsReplayPlayer | null = null;

  elements.statusReadout.textContent = source.preparingStatus;
  elements.fileInput.disabled = true;
  options.getReplayLoadModal()?.show(source.name, source.preparingStatus);
  options.setTransportEnabled(false);
  options.getCameraControlsController()?.syncAvailability();
  elements.emptyState.hidden = options.getReplayPlayer() !== null;
  options.getReplayPlayer()?.pause();

  try {
    elements.statusReadout.textContent = "Parsing replay...";
    options.getReplayLoadModal()?.show(source.name, "Parsing replay...");
    const loadedReplay = await bundlePromise;
    const { replay } = loadedReplay;
    const existingReplayPlayer = options.getReplayPlayer();

    if (existingReplayPlayer) {
      clearDisplayedReplay(options, {
        destroyPlayer: false,
        clearPlayerPluginHandles: false,
      });
      const adapter = new SubtrActorPlayer(loadedReplay.raw as never);
      await existingReplayPlayer.replaceReplay(adapter, replay, { preservePlayback: false });

      options.setStatsTimeline(loadedReplay.statsTimeline);
      options.setStatsFrameLookup(loadedReplay.statsFrameLookup);
      options.setStatRegistry(createStatRegistry(null));
      options.setReplayPlayer(existingReplayPlayer);
      options.syncBoostPadOverlayPlugin();

      options.setupActiveModules();
      options.setUnsubscribe(existingReplayPlayer.subscribe(options.renderSnapshot));

      const config = options.getInitialConfig();
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
      options.renderModuleSummary();
      options.renderTimelineEventCount();
      options.renderMechanicsTimelineControls();
      options.resetEventPlaylistWindow();
      options.renderEventPlaylistWindow();
      options.setTransportEnabled(true);
      options.getCameraControlsController()?.syncAvailability(existingReplayPlayer.getState());
      options.renderSnapshot(existingReplayPlayer.getState());
      options.renderStatsWindows(existingReplayPlayer.getState().frameIndex);
      options.renderScoreboard(existingReplayPlayer.getState().frameIndex);
      options.syncEventPlaylistTimeline(existingReplayPlayer.getState(), { forceScroll: true });
      options.renderModuleSettings();
      options.syncRecordingWindow();
      options.getReplayLoadModal()?.hide();
      return;
    }

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

    // The player implements @rlrml/player's full ReplayPlayer surface; player
    // plugins are bridged via fromReplayPlayerPlugin (the original `recorder` /
    // `timelineOverlay` handles share closure state with the wrapped copies, so
    // they keep working for setCanvasRecorder/setTimelineOverlay below).
    const replayPlayer = createPlayerFromParsed(elements.viewport, loadedReplay, {
      initialPlaybackRate: config?.playback.rate,
      initialCustomCameraSettings: getCustomCameraSettingsFromConfig(config?.camera),
      initialAttachedPlayerId: config?.camera.attachedPlayerId ?? null,
      initialCameraViewMode: config?.camera.mode,
      // Ball cam defaults to "player" (follow the recorded toggle): leave the
      // initial override unset (null) so the camera plugin tracks the recorded
      // state. A saved config's forced ball/car cam is applied by
      // applyConfigToReplayPlayer immediately after construction.
      initialBoostPickupAnimationEnabled: config?.overlays.boostPickupAnimation ?? false,
      initialHitboxWireframesEnabled:
        config?.overlays.hitboxWireframes ?? elements.hitboxWireframes.checked,
      initialHitboxOnlyModeEnabled:
        config?.overlays.hitboxOnlyMode ?? elements.hitboxOnlyMode.checked,
      initialSkipPostGoalTransitionsEnabled: elements.skipPostGoalTransitions.checked,
      initialSkipKickoffsEnabled: elements.skipKickoffs.checked,
      plugins: [
        // Render + live replay FPS, reported into the Playback window's detail
        // grid (headless: we own the markup so it matches the other fields).
        createFpsOverlayPlugin({
          onSample: ({ renderFps, replayFps }) => {
            const render = document.getElementById("render-fps-readout");
            const replay = document.getElementById("replay-fps-readout");
            if (render) render.textContent = `${renderFps.toFixed(0)} fps`;
            if (replay) replay.textContent = `${replayFps.toFixed(0)} fps`;
          },
        }),
        fromReplayPlayerPlugin(
          createBallchasingOverlayPlugin({
            floatingLiftUu: () => options.getCameraControlsController()?.nameplateLiftUu,
          }),
        ),
        fromReplayPlayerPlugin(
          createBoostPickupAnimationPlugin({
            includePickup: options.includeBoostPickupAnimationPickup,
          }),
        ),
        fromReplayPlayerPlugin(recorder),
        fromReplayPlayerPlugin(timelineOverlay),
      ],
    }) as StatsReplayPlayer;
    pendingReplayPlayer = replayPlayer;
    setReplayPlayerCanvasPending(replayPlayer, true);
    await replayPlayer.ready;

    clearDisplayedReplay(options);
    pendingReplayPlayer = null;
    setReplayPlayerCanvasPending(replayPlayer, false);

    options.setStatsTimeline(loadedReplay.statsTimeline);
    options.setStatsFrameLookup(loadedReplay.statsFrameLookup);
    options.setStatRegistry(createStatRegistry(null));
    if (import.meta.env.DEV) {
      // Console/debug handle (dev server only): inspect playback, A/B camera or
      // motion-interpolation settings live, sample mesh positions, etc.
      (window as { __statsReplayPlayer?: StatsReplayPlayer }).__statsReplayPlayer = replayPlayer;
    }
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
    options.renderModuleSummary();
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
    pendingReplayPlayer?.destroy();
    if (!options.getReplayPlayer()) {
      elements.emptyState.hidden = false;
      options.setCanvasRecorder(null);
    }
    options.syncRecordingWindow();
    throw error;
  } finally {
    elements.fileInput.disabled = false;
  }
}
