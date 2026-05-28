import * as THREE from "three";
import { createReplayScene, type ReplayScene } from "./scene";
import { findFrameIndexAtTime } from "./replay-data";
import {
  clampFrameIndex,
  getFrameWindow,
  getKickoffCountdownMetadata,
  inferKickoffGameState,
  inferLiveGameState,
} from "./player-internals/timeline";
import {
  getActiveDemoEvent,
} from "./player-render-effects";
import { renderReplayFrameScene } from "./player-render-frame";
import { normalizeCustomCameraSettings } from "./player-camera-settings";
import { ReplayPlayerTimelineCache } from "./player-timeline-cache";
import { findKickoffSkipTime, findPostGoalTransitionSkipTime } from "./player-skip";
import {
  getFreeCameraPreset,
  updateFreeCameraTransition,
  updateAttachedCamera,
} from "./player-internals/spatial";
import type {
  BeforeRenderCallback,
  CameraSettings,
  FrameRenderInfo,
  ReplayCameraViewMode,
  ReplayFreeCameraPreset,
  ReplayPlayerActiveMetadata,
  ReplayModel,
  ReplayPlayerPlugin,
  ReplayPlayerPluginContext,
  ReplayPlayerPluginDefinition,
  ReplayPlayerPluginStateContext,
  ReplayPlayerRenderContext,
  ReplayPlayerRenderTrackContext,
  ReplayPlayerTimelineProjection,
  ReplayPlayerTimelineSegment,
  ReplayPlayerOptions,
  ReplayPlayerSnapshot,
  ReplayPlayerState,
  ReplayPlayerStatePatch,
  ReplayTimelineEvent,
  Vec3,
} from "./types";

const DEFAULT_FIELD_SCALE = 1;
const DEFAULT_CAMERA_DISTANCE_SCALE = 2.25;
const DEFAULT_CAMERA_VIEW_MODE: ReplayCameraViewMode = "free";

type ReplayPlayerListener = (state: ReplayPlayerState) => void;
type InstalledReplayPlayerPlugin = {
  definition: ReplayPlayerPluginDefinition;
  plugin: ReplayPlayerPlugin;
};
type FreeCameraTransition = {
  position: THREE.Vector3;
  target: THREE.Vector3;
  up: THREE.Vector3;
  fov: number;
};

export class ReplayPlayer extends EventTarget {
  readonly container: HTMLElement;
  readonly replay: ReplayModel;
  readonly options: ReplayPlayerOptions;

  readonly sceneState: ReplayScene;
  private readonly beforeRenderCallbacks: BeforeRenderCallback[] = [];
  private readonly plugins: InstalledReplayPlayerPlugin[] = [];
  private readonly fieldScale: number;
  private readonly desiredCameraPosition = new THREE.Vector3();
  private readonly desiredLookTarget = new THREE.Vector3();
  private readonly boundWindowResize = () => this.sceneState.resize();
  private readonly liveGameState: number | null;
  private readonly kickoffGameState: number | null;
  private readonly timelineCache = new ReplayPlayerTimelineCache();
  private resizeObserver: ResizeObserver | null = null;
  private animationFrameId: number | null = null;
  private disposed = false;
  private playing = false;
  private speed = 1;
  private currentTime = 0;
  private playbackStartedAt = 0;
  private playbackStartedTime = 0;
  private cameraDistanceScale: number;
  private customCameraSettings: CameraSettings | null;
  private cameraViewMode: ReplayCameraViewMode;
  private freeCameraTransition: FreeCameraTransition | null = null;
  private attachedPlayerId: string | null;
  private ballCamEnabled: boolean;
  private boostMeterEnabled: boolean;
  private boostPickupAnimationEnabled: boolean;
  private skipPostGoalTransitionsEnabled: boolean;
  private skipKickoffsEnabled: boolean;

  constructor(container: HTMLElement, replay: ReplayModel, options: ReplayPlayerOptions = {}) {
    super();
    this.container = container;
    this.replay = replay;
    this.options = options;
    this.fieldScale = options.fieldScale ?? DEFAULT_FIELD_SCALE;
    this.sceneState = createReplayScene(container, replay, this.fieldScale);
    this.liveGameState = inferLiveGameState(replay);
    this.kickoffGameState = inferKickoffGameState(replay, this.liveGameState);
    this.speed = Math.max(0.1, options.initialPlaybackRate ?? 1);
    this.cameraDistanceScale = Math.max(
      0.25,
      options.initialCameraDistanceScale ?? DEFAULT_CAMERA_DISTANCE_SCALE,
    );
    this.customCameraSettings = normalizeCustomCameraSettings(options.initialCustomCameraSettings);
    this.attachedPlayerId = options.initialAttachedPlayerId ?? null;
    this.cameraViewMode =
      options.initialCameraViewMode ??
      (this.attachedPlayerId ? "follow" : DEFAULT_CAMERA_VIEW_MODE);
    this.ballCamEnabled = options.initialBallCamEnabled ?? false;
    this.boostMeterEnabled = options.initialBoostMeterEnabled ?? false;
    this.boostPickupAnimationEnabled = options.initialBoostPickupAnimationEnabled ?? true;
    this.skipPostGoalTransitionsEnabled = options.initialSkipPostGoalTransitionsEnabled ?? true;
    this.skipKickoffsEnabled = options.initialSkipKickoffsEnabled ?? false;
    this.skipPostGoalTransitionIfNeeded();
    this.skipPastKickoffIfNeeded();

    this.installResizeHandling();
    for (const plugin of options.plugins ?? []) {
      this.installPlugin(plugin, false);
    }
    this.render();
    this.scheduleAnimationFrame();
    this.emitChange();

    if (options.autoplay) {
      this.play();
    }
  }

  play(): void {
    if (this.playing) {
      return;
    }

    this.playing = true;
    this.reanchorPlaybackClock();
    this.emitChange();
  }

  pause(): void {
    if (!this.playing) {
      return;
    }

    this.syncPlaybackClock();
    this.playing = false;
    this.emitChange();
  }

  togglePlayback(): void {
    if (this.playing) {
      this.pause();
    } else {
      this.play();
    }
  }

  setPlaybackRate(speed: number): void {
    if (this.playing) {
      this.syncPlaybackClock();
    }
    this.speed = Math.max(0.1, speed);
    if (this.playing) {
      this.reanchorPlaybackClock();
    }
    this.emitChange();
  }

  setCameraDistanceScale(scale: number): void {
    this.cameraDistanceScale = Math.max(0.25, scale);
    this.render();
    this.emitChange();
  }

  setCustomCameraSettings(settings: CameraSettings | null): void {
    this.customCameraSettings = normalizeCustomCameraSettings(settings);
    this.render();
    this.emitChange();
  }

  setAttachedPlayer(playerId: string | null): void {
    this.attachedPlayerId = playerId;
    this.cameraViewMode = playerId ? "follow" : DEFAULT_CAMERA_VIEW_MODE;
    this.freeCameraTransition = null;
    this.render();
    this.emitChange();
  }

  setCameraViewMode(mode: ReplayCameraViewMode): void {
    this.cameraViewMode = mode;
    this.freeCameraTransition = null;
    this.render();
    this.emitChange();
  }

  setFreeCameraPreset(preset: ReplayFreeCameraPreset): void {
    const { fov, position, target, up } = getFreeCameraPreset(preset, this.fieldScale);
    this.cameraViewMode = DEFAULT_CAMERA_VIEW_MODE;
    this.freeCameraTransition = {
      position,
      target,
      up,
      fov,
    };
    this.render();
    this.emitChange();
  }

  setBallCamEnabled(enabled: boolean): void {
    this.ballCamEnabled = enabled;
    this.render();
    this.emitChange();
  }

  setBoostMeterEnabled(enabled: boolean): void {
    this.boostMeterEnabled = enabled;
    if (!enabled) {
      for (const meter of this.sceneState.playerBoostMeters.values()) {
        meter.group.visible = false;
      }
    }
    this.render();
    this.emitChange();
  }

  setBoostPickupAnimationEnabled(enabled: boolean): void {
    this.boostPickupAnimationEnabled = enabled;
    this.render();
    this.emitChange();
  }

  setSkipPostGoalTransitionsEnabled(enabled: boolean): void {
    this.skipPostGoalTransitionsEnabled = enabled;
    if (enabled) {
      this.skipPostGoalTransitionIfNeeded();
    }
    this.render();
    this.emitChange();
  }

  setSkipKickoffsEnabled(enabled: boolean): void {
    this.skipKickoffsEnabled = enabled;
    if (enabled) {
      this.skipPostGoalTransitionIfNeeded();
      this.skipPastKickoffIfNeeded();
    }
    this.render();
    this.emitChange();
  }

  seek(time: number): void {
    this.currentTime = this.clampReplayTime(time);
    this.skipPostGoalTransitionIfNeeded();
    this.skipPastKickoffIfNeeded();
    if (this.playing) {
      this.reanchorPlaybackClock();
    }
    this.render();
    this.emitChange();
  }

  setFrameIndex(frameIndex: number): void {
    const nextFrameIndex = clampFrameIndex(this.replay, frameIndex);
    const nextTime = this.replay.frames[nextFrameIndex]?.time ?? 0;
    const wasPlaying = this.playing;
    const changed = this.currentTime !== nextTime || wasPlaying;

    this.playing = false;
    this.currentTime = nextTime;
    this.render();
    if (changed) {
      this.emitChange();
    }
  }

  stepFrames(delta: number): void {
    if (!Number.isFinite(delta) || this.replay.frames.length === 0) {
      return;
    }

    const currentFrameIndex = findFrameIndexAtTime(this.replay, this.currentTime);
    this.setFrameIndex(currentFrameIndex + Math.trunc(delta));
  }

  stepForwardFrame(): void {
    this.stepFrames(1);
  }

  stepBackwardFrame(): void {
    this.stepFrames(-1);
  }

  setState(nextState: ReplayPlayerStatePatch): void {
    const now = performance.now();
    if (nextState.speed !== undefined) {
      if (this.playing) {
        this.syncPlaybackClock(now);
      }
      this.speed = Math.max(0.1, nextState.speed);
    }
    if (nextState.cameraDistanceScale !== undefined) {
      this.cameraDistanceScale = Math.max(0.25, nextState.cameraDistanceScale);
    }
    if (nextState.customCameraSettings !== undefined) {
      this.customCameraSettings = normalizeCustomCameraSettings(nextState.customCameraSettings);
    }
    if (nextState.cameraViewMode !== undefined) {
      this.cameraViewMode = nextState.cameraViewMode;
    }
    if (nextState.attachedPlayerId !== undefined) {
      this.attachedPlayerId = nextState.attachedPlayerId;
      if (nextState.cameraViewMode === undefined) {
        this.cameraViewMode = this.attachedPlayerId ? "follow" : DEFAULT_CAMERA_VIEW_MODE;
      }
    }
    if (nextState.ballCamEnabled !== undefined) {
      this.ballCamEnabled = nextState.ballCamEnabled;
    }
    if (nextState.boostMeterEnabled !== undefined) {
      this.boostMeterEnabled = nextState.boostMeterEnabled;
      if (!this.boostMeterEnabled) {
        for (const meter of this.sceneState.playerBoostMeters.values()) {
          meter.group.visible = false;
        }
      }
    }
    if (nextState.boostPickupAnimationEnabled !== undefined) {
      this.boostPickupAnimationEnabled = nextState.boostPickupAnimationEnabled;
    }
    if (nextState.skipPostGoalTransitionsEnabled !== undefined) {
      this.skipPostGoalTransitionsEnabled = nextState.skipPostGoalTransitionsEnabled;
    }
    if (nextState.skipKickoffsEnabled !== undefined) {
      this.skipKickoffsEnabled = nextState.skipKickoffsEnabled;
    }
    if (nextState.currentTime !== undefined) {
      this.currentTime = this.clampReplayTime(nextState.currentTime);
    }
    if (nextState.playing !== undefined && nextState.playing !== this.playing) {
      if (nextState.playing) {
        this.playing = true;
      } else {
        if (nextState.currentTime === undefined) {
          this.syncPlaybackClock(now);
        }
        this.playing = false;
      }
    }
    if (this.playing) {
      this.reanchorPlaybackClock(now);
    }
    this.skipPostGoalTransitionIfNeeded(now);
    this.skipPastKickoffIfNeeded(now);

    this.render();
    this.emitChange();
  }

  getState(): ReplayPlayerState {
    const frameIndex = findFrameIndexAtTime(this.replay, this.currentTime);
    return {
      currentTime: this.currentTime,
      duration: this.replay.duration,
      frameIndex,
      activeMetadata: this.getActiveMetadata(frameIndex, this.currentTime),
      playing: this.playing,
      speed: this.speed,
      cameraDistanceScale: this.cameraDistanceScale,
      customCameraSettings: this.customCameraSettings,
      cameraViewMode: this.cameraViewMode,
      attachedPlayerId: this.attachedPlayerId,
      ballCamEnabled: this.ballCamEnabled,
      boostMeterEnabled: this.boostMeterEnabled,
      boostPickupAnimationEnabled: this.boostPickupAnimationEnabled,
      skipPostGoalTransitionsEnabled: this.skipPostGoalTransitionsEnabled,
      skipKickoffsEnabled: this.skipKickoffsEnabled,
    };
  }

  getSnapshot(): ReplayPlayerSnapshot {
    return this.getState();
  }

  getTimelineDuration(): number {
    return this.timelineCache.getDuration(this.getTimelineOptions());
  }

  getTimelineCurrentTime(): number {
    return this.timelineCache.projectReplayTime(
      this.getTimelineOptions(),
      this.currentTime,
    ).timelineTime;
  }

  getTimelineSegments(): ReplayPlayerTimelineSegment[] {
    return this.timelineCache.getSegments(this.getTimelineOptions());
  }

  projectReplayTimeToTimeline(replayTime: number): ReplayPlayerTimelineProjection {
    return this.timelineCache.projectReplayTime(this.getTimelineOptions(), replayTime);
  }

  projectTimelineTimeToReplay(timelineTime: number): number {
    return this.timelineCache.projectTimelineTime(this.getTimelineOptions(), timelineTime);
  }

  private clampReplayTime(time: number): number {
    return THREE.MathUtils.clamp(time, 0, this.replay.duration);
  }

  private getPlaybackEndTime(): number {
    return this.timelineCache.getPlaybackEndTime(this.getTimelineOptions());
  }

  subscribe(listener: ReplayPlayerListener): () => void {
    const handleChange = (event: Event): void => {
      listener((event as CustomEvent<ReplayPlayerState>).detail);
    };
    this.addEventListener("change", handleChange);
    listener(this.getState());
    return () => {
      this.removeEventListener("change", handleChange);
    };
  }

  onBeforeRender(callback: BeforeRenderCallback): () => void {
    this.beforeRenderCallbacks.push(callback);
    return () => {
      const index = this.beforeRenderCallbacks.indexOf(callback);
      if (index >= 0) {
        this.beforeRenderCallbacks.splice(index, 1);
      }
    };
  }

  addPlugin(definition: ReplayPlayerPluginDefinition): () => void {
    return this.installPlugin(definition, true);
  }

  removePlugin(id: string): boolean {
    const index = this.plugins.findIndex((entry) => entry.plugin.id === id);
    if (index < 0) {
      return false;
    }

    const [entry] = this.plugins.splice(index, 1);
    entry.plugin.teardown?.(this.createPluginContext());
    this.render();
    return true;
  }

  getPlugins(): ReplayPlayerPlugin[] {
    return this.plugins.map((entry) => entry.plugin);
  }

  destroy(): void {
    if (this.playing) {
      this.pause();
    }
    this.disposed = true;
    if (this.animationFrameId !== null) {
      cancelAnimationFrame(this.animationFrameId);
      this.animationFrameId = null;
    }
    if (this.resizeObserver) {
      this.resizeObserver.disconnect();
      this.resizeObserver = null;
    } else {
      window.removeEventListener("resize", this.boundWindowResize);
    }
    while (this.plugins.length > 0) {
      const entry = this.plugins.pop();
      entry?.plugin.teardown?.(this.createPluginContext());
    }
    this.sceneState.dispose();
  }

  dispose(): void {
    this.destroy();
  }

  private installResizeHandling(): void {
    if (typeof ResizeObserver !== "undefined") {
      this.resizeObserver = new ResizeObserver(() => {
        this.sceneState.resize();
      });
      this.resizeObserver.observe(this.container);
      return;
    }

    window.addEventListener("resize", this.boundWindowResize);
  }

  private scheduleAnimationFrame(): void {
    if (this.animationFrameId !== null || this.disposed) {
      return;
    }

    this.animationFrameId = requestAnimationFrame(this.tick);
  }

  private reanchorPlaybackClock(now = performance.now()): void {
    this.playbackStartedAt = now;
    this.playbackStartedTime = this.currentTime;
  }

  private syncPlaybackClock(now = performance.now()): boolean {
    if (!this.playing) {
      return false;
    }

    const elapsedSeconds = (now - this.playbackStartedAt) / 1000;
    const nextTime = THREE.MathUtils.clamp(
      this.playbackStartedTime + elapsedSeconds * this.speed,
      0,
      this.getPlaybackEndTime(),
    );
    const timeChanged = nextTime !== this.currentTime;
    this.currentTime = nextTime;
    return timeChanged;
  }

  private tick = (now: number): void => {
    this.animationFrameId = null;
    if (this.disposed) {
      return;
    }

    let shouldEmitChange = false;
    if (this.playing) {
      shouldEmitChange = this.syncPlaybackClock(now);
      shouldEmitChange = this.skipPostGoalTransitionIfNeeded(now) || shouldEmitChange;
      shouldEmitChange = this.skipPastKickoffIfNeeded(now) || shouldEmitChange;
      if (this.currentTime >= this.getPlaybackEndTime()) {
        this.playing = false;
        shouldEmitChange = true;
      }
    }

    this.render();
    if (shouldEmitChange) {
      this.emitChange();
    }
    this.scheduleAnimationFrame();
  };

  private render(): void {
    const frameWindow = getFrameWindow(this.replay, this.currentTime);
    const frameIndex = frameWindow.frameIndex;
    const {
      ballFrame,
      nextBallFrame,
      ballPosition,
      ballWorldPosition,
      players: renderPlayers,
    } =
      renderReplayFrameScene({
        replay: this.replay,
        sceneState: this.sceneState,
        frameWindow,
        fieldScale: this.fieldScale,
        currentTime: this.currentTime,
        boostMeterEnabled: this.boostMeterEnabled,
      });

    updateAttachedCamera({
      sceneState: this.sceneState,
      replay: this.replay,
      fieldScale: this.fieldScale,
      cameraViewMode: this.cameraViewMode,
      attachedPlayerId: this.attachedPlayerId,
      ballCamEnabled: this.ballCamEnabled,
      cameraDistanceScale: this.cameraDistanceScale,
      customCameraSettings: this.customCameraSettings,
      frameIndex,
      attachedPlayerUnavailable:
        this.attachedPlayerId !== null &&
        getActiveDemoEvent(this.replay, this.attachedPlayerId, this.currentTime) !== null,
      ballPosition: ballWorldPosition,
      desiredCameraPosition: this.desiredCameraPosition,
      desiredLookTarget: this.desiredLookTarget,
    });
    if (this.cameraViewMode === "free" && this.freeCameraTransition) {
      const completed = updateFreeCameraTransition({
        sceneState: this.sceneState,
        ...this.freeCameraTransition,
      });
      if (completed) {
        this.freeCameraTransition = null;
      }
    }
    this.sceneState.controls.update();
    this.sceneState.updateWallVisibility();
    const renderInfo: FrameRenderInfo = {
      frameIndex: frameWindow.frameIndex,
      nextFrameIndex: frameWindow.nextFrameIndex,
      alpha: frameWindow.alpha,
      currentTime: this.currentTime,
    };
    for (const callback of this.beforeRenderCallbacks) {
      callback(renderInfo);
    }
    const renderContext = this.createRenderContext(
      renderInfo,
      ballFrame,
      nextBallFrame,
      ballPosition,
      renderPlayers,
    );
    for (const entry of this.plugins) {
      entry.plugin.beforeRender?.(renderContext);
    }
    this.sceneState.renderer.render(this.sceneState.scene, this.sceneState.camera);
  }

  private skipPastKickoffIfNeeded(now?: number): boolean {
    if (!this.skipKickoffsEnabled) {
      return false;
    }

    const skipTime = findKickoffSkipTime(
      this.replay,
      this.currentTime,
      this.kickoffGameState,
      this.liveGameState,
    );
    if (skipTime === null) {
      return false;
    }

    this.currentTime = skipTime;
    if (this.playing) {
      this.reanchorPlaybackClock(now);
    }
    return true;
  }

  private skipPostGoalTransitionIfNeeded(now?: number): boolean {
    if (!this.skipPostGoalTransitionsEnabled) {
      return false;
    }

    const skipTime = findPostGoalTransitionSkipTime(
      this.replay,
      this.currentTime,
      this.liveGameState,
      this.kickoffGameState,
    );
    if (skipTime === null) {
      return false;
    }

    this.currentTime = skipTime;
    if (this.playing) {
      this.reanchorPlaybackClock(now);
    }
    return true;
  }

  private getActiveMetadata(
    frameIndex: number,
    currentTime: number,
  ): ReplayPlayerActiveMetadata | null {
    return getKickoffCountdownMetadata(this.replay, frameIndex, currentTime);
  }

  private getTimelineOptions() {
    return {
      replay: this.replay,
      skipPostGoalTransitionsEnabled: this.skipPostGoalTransitionsEnabled,
      skipKickoffsEnabled: this.skipKickoffsEnabled,
      liveGameState: this.liveGameState,
      kickoffGameState: this.kickoffGameState,
    };
  }

  private installPlugin(
    definition: ReplayPlayerPluginDefinition,
    renderAfterSetup: boolean,
  ): () => void {
    const plugin = typeof definition === "function" ? definition() : definition;

    if (this.plugins.some((entry) => entry.plugin.id === plugin.id)) {
      throw new Error(`Replay player plugin "${plugin.id}" is already installed`);
    }

    const entry = { definition, plugin };
    this.plugins.push(entry);
    plugin.setup?.(this.createPluginContext());
    plugin.onStateChange?.(this.createPluginStateContext(this.getState()));

    if (renderAfterSetup) {
      this.render();
    }

    return () => {
      const index = this.plugins.indexOf(entry);
      if (index < 0) {
        return;
      }
      this.plugins.splice(index, 1);
      plugin.teardown?.(this.createPluginContext());
      this.render();
    };
  }

  private createPluginContext(): ReplayPlayerPluginContext {
    return {
      player: this,
      replay: this.replay,
      scene: this.sceneState,
      container: this.container,
      options: this.options,
    };
  }

  private createPluginStateContext(state: ReplayPlayerState): ReplayPlayerPluginStateContext {
    return {
      ...this.createPluginContext(),
      state,
    };
  }

  private createRenderContext(
    renderInfo: FrameRenderInfo,
    ballFrame: ReplayModel["ballFrames"][number] | null,
    nextBallFrame: ReplayModel["ballFrames"][number] | null,
    ballPosition: Vec3 | null,
    players: ReplayPlayerRenderTrackContext[],
  ): ReplayPlayerRenderContext {
    return {
      ...this.createPluginStateContext(this.getState()),
      ...renderInfo,
      frame: this.replay.frames[renderInfo.frameIndex] ?? null,
      nextFrame: this.replay.frames[renderInfo.nextFrameIndex] ?? null,
      ballFrame,
      nextBallFrame,
      ballPosition,
      players,
    };
  }

  private emitChange(): void {
    const state = this.getState();
    const pluginStateContext = this.createPluginStateContext(state);
    for (const entry of this.plugins) {
      entry.plugin.onStateChange?.(pluginStateContext);
    }
    this.dispatchEvent(new CustomEvent<ReplayPlayerState>("change", { detail: state }));
  }

}
