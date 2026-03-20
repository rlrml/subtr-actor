import * as THREE from "three";
import { createReplayScene, updateBoostMeter, type ReplayScene } from "./scene";
import { findFrameIndexAtTime } from "./replay-data";
import {
  clampFrameIndex,
  computeTimelineSegments,
  getFrameWindow,
  getKickoffCountdownMetadata,
  inferKickoffGameState,
  inferLiveGameState,
  isKickoffFrame,
  isLiveGameplayFrame,
  isPostGoalTransitionFrame,
  projectReplayTimeToTimeline,
  projectTimelineTimeToReplay,
} from "./player-internals/timeline";
import {
  interpolatePosition,
  rootPosition,
  updateAttachedCamera,
  worldPosition,
} from "./player-internals/spatial";
import type {
  BeforeRenderCallback,
  FrameRenderInfo,
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
  Vec3,
} from "./types";

const DEFAULT_FIELD_SCALE = 1;
const DEFAULT_CAMERA_DISTANCE_SCALE = 2.25;

type ReplayPlayerListener = (state: ReplayPlayerState) => void;
type InstalledReplayPlayerPlugin = {
  definition: ReplayPlayerPluginDefinition;
  plugin: ReplayPlayerPlugin;
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
  private timelineSegmentsCacheKey: string | null = null;
  private timelineSegmentsCache: ReplayPlayerTimelineSegment[] = [];
  private timelineDurationCache = 0;
  private resizeObserver: ResizeObserver | null = null;
  private animationFrameId: number | null = null;
  private disposed = false;
  private playing = false;
  private speed = 1;
  private currentTime = 0;
  private playbackStartedAt = 0;
  private playbackStartedTime = 0;
  private cameraDistanceScale: number;
  private attachedPlayerId: string | null;
  private ballCamEnabled: boolean;
  private boostMeterEnabled: boolean;
  private skipPostGoalTransitionsEnabled: boolean;
  private skipKickoffsEnabled: boolean;

  constructor(
    container: HTMLElement,
    replay: ReplayModel,
    options: ReplayPlayerOptions = {}
  ) {
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
      options.initialCameraDistanceScale ?? DEFAULT_CAMERA_DISTANCE_SCALE
    );
    this.attachedPlayerId = options.initialAttachedPlayerId ?? null;
    this.ballCamEnabled = options.initialBallCamEnabled ?? false;
    this.boostMeterEnabled = options.initialBoostMeterEnabled ?? false;
    this.skipPostGoalTransitionsEnabled =
      options.initialSkipPostGoalTransitionsEnabled ?? true;
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

  setAttachedPlayer(playerId: string | null): void {
    this.attachedPlayerId = playerId;
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
    this.currentTime = THREE.MathUtils.clamp(time, 0, this.replay.duration);
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
    if (nextState.attachedPlayerId !== undefined) {
      this.attachedPlayerId = nextState.attachedPlayerId;
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
    if (nextState.skipPostGoalTransitionsEnabled !== undefined) {
      this.skipPostGoalTransitionsEnabled =
        nextState.skipPostGoalTransitionsEnabled;
    }
    if (nextState.skipKickoffsEnabled !== undefined) {
      this.skipKickoffsEnabled = nextState.skipKickoffsEnabled;
    }
    if (nextState.currentTime !== undefined) {
      this.currentTime = THREE.MathUtils.clamp(
        nextState.currentTime,
        0,
        this.replay.duration
      );
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
      attachedPlayerId: this.attachedPlayerId,
      ballCamEnabled: this.ballCamEnabled,
      boostMeterEnabled: this.boostMeterEnabled,
      skipPostGoalTransitionsEnabled: this.skipPostGoalTransitionsEnabled,
      skipKickoffsEnabled: this.skipKickoffsEnabled,
    };
  }

  getSnapshot(): ReplayPlayerSnapshot {
    return this.getState();
  }

  getTimelineDuration(): number {
    return this.getTimelineSegments().length === 0
      ? this.replay.duration
      : this.timelineDurationCache;
  }

  getTimelineCurrentTime(): number {
    return this.projectReplayTimeToTimeline(this.currentTime).timelineTime;
  }

  getTimelineSegments(): ReplayPlayerTimelineSegment[] {
    const cacheKey =
      `${this.skipPostGoalTransitionsEnabled}:${this.skipKickoffsEnabled}`;
    if (this.timelineSegmentsCacheKey === cacheKey) {
      return this.timelineSegmentsCache;
    }

    this.timelineSegmentsCacheKey = cacheKey;
    this.timelineSegmentsCache = this.computeTimelineSegments();
    this.timelineDurationCache = Math.max(
      0,
      this.replay.duration -
        this.timelineSegmentsCache.reduce(
          (total, segment) => total + (segment.endTime - segment.startTime),
          0
        )
    );
    return this.timelineSegmentsCache;
  }

  projectReplayTimeToTimeline(
    replayTime: number
  ): ReplayPlayerTimelineProjection {
    return projectReplayTimeToTimeline(
      this.replay.duration,
      this.getTimelineSegments(),
      replayTime,
    );
  }

  projectTimelineTimeToReplay(timelineTime: number): number {
    return projectTimelineTimeToReplay(
      this.replay.duration,
      this.getTimelineDuration(),
      this.getTimelineSegments(),
      timelineTime,
    );
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
      this.replay.duration
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
      if (this.currentTime >= this.replay.duration) {
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
    const ballFrame = this.replay.ballFrames[frameIndex] ?? null;
    const nextBallFrame =
      this.replay.ballFrames[frameWindow.nextFrameIndex] ?? ballFrame;
    const interpolatedBallPosition = interpolatePosition(
      ballFrame?.position ?? null,
      nextBallFrame?.position ?? null,
      frameWindow.alpha
    );
    const ballPosition = interpolatedBallPosition
      ? worldPosition(interpolatedBallPosition, this.fieldScale)
      : null;
    const renderPlayers: ReplayPlayerRenderTrackContext[] = [];

    if (interpolatedBallPosition) {
      this.sceneState.ballMesh.visible = true;
      this.sceneState.ballMesh.position.copy(
        rootPosition(interpolatedBallPosition)
      );
      if (ballFrame?.rotation) {
        this.sceneState.ballMesh.quaternion.set(
          ballFrame.rotation.x,
          ballFrame.rotation.y,
          ballFrame.rotation.z,
          ballFrame.rotation.w
        );
      } else {
        this.sceneState.ballMesh.quaternion.identity();
      }
    } else {
      this.sceneState.ballMesh.visible = false;
    }

    for (const [playerIndex, player] of this.replay.players.entries()) {
      const mesh = this.sceneState.playerMeshes.get(player.id);
      const boostTrail = this.sceneState.playerBoostTrails.get(player.id);
      const frame = player.frames[frameIndex] ?? null;
      const nextFrame = player.frames[frameWindow.nextFrameIndex] ?? frame;
      let interpolatedPosition: Vec3 | null = null;
      let boostFraction = 0;
      if (!mesh) {
        renderPlayers.push({
          track: player,
          mesh: null,
          boostTrail: boostTrail ?? null,
          frame,
          nextFrame,
          interpolatedPosition,
          boostFraction,
        });
        continue;
      }

      interpolatedPosition = interpolatePosition(
        frame?.position ?? null,
        nextFrame?.position ?? null,
        frameWindow.alpha
      );
      if (!interpolatedPosition) {
        mesh.visible = false;
        if (boostTrail) {
          boostTrail.visible = false;
        }
        renderPlayers.push({
          track: player,
          mesh,
          boostTrail: boostTrail ?? null,
          frame,
          nextFrame,
          interpolatedPosition,
          boostFraction,
        });
        continue;
      }

      mesh.visible = true;
      mesh.position.copy(rootPosition(interpolatedPosition));
      if (frame?.rotation) {
        mesh.quaternion.set(
          frame.rotation.x,
          frame.rotation.y,
          frame.rotation.z,
          frame.rotation.w
        );
      } else {
        mesh.quaternion.identity();
      }

      const currentBoostFraction = frame?.boostFraction ?? 0;
      const nextBoostFraction = nextFrame?.boostFraction ?? currentBoostFraction;
      boostFraction = THREE.MathUtils.lerp(
        currentBoostFraction,
        nextBoostFraction,
        frameWindow.alpha
      );

      if (boostTrail) {
        const boostActive =
          (frameWindow.alpha >= 0.5
            ? nextFrame?.boostActive
            : frame?.boostActive) ??
          frame?.boostActive ??
          nextFrame?.boostActive ??
          false;
        this.updateBoostTrail(
          boostTrail,
          boostActive,
          boostFraction,
          this.currentTime,
          playerIndex
        );
      }

      const boostMeter = this.sceneState.playerBoostMeters.get(player.id);
      if (boostMeter) {
        if (this.boostMeterEnabled) {
          boostMeter.group.visible = true;
          updateBoostMeter(
            boostMeter,
            boostFraction,
            THREE.MathUtils.lerp(
              frame?.boostAmount ?? 0,
              nextFrame?.boostAmount ?? frame?.boostAmount ?? 0,
              frameWindow.alpha
            ),
            this.sceneState.camera
          );
        } else {
          boostMeter.group.visible = false;
        }
      }

      renderPlayers.push({
        track: player,
        mesh,
        boostTrail: boostTrail ?? null,
        frame,
        nextFrame,
        interpolatedPosition,
        boostFraction,
      });
    }

    updateAttachedCamera({
      sceneState: this.sceneState,
      replay: this.replay,
      fieldScale: this.fieldScale,
      attachedPlayerId: this.attachedPlayerId,
      ballCamEnabled: this.ballCamEnabled,
      cameraDistanceScale: this.cameraDistanceScale,
      frameIndex,
      ballPosition,
      desiredCameraPosition: this.desiredCameraPosition,
      desiredLookTarget: this.desiredLookTarget,
    });
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
      renderPlayers
    );
    for (const entry of this.plugins) {
      entry.plugin.beforeRender?.(renderContext);
    }
    this.sceneState.renderer.render(
      this.sceneState.scene,
      this.sceneState.camera
    );
  }

  private skipPastKickoffIfNeeded(now?: number): boolean {
    if (!this.skipKickoffsEnabled) {
      return false;
    }

    const frameIndex = findFrameIndexAtTime(this.replay, this.currentTime);
    const frame = this.replay.frames[frameIndex];
    if (!frame || !isKickoffFrame(frame, this.kickoffGameState)) {
      return false;
    }

    const nextLiveFrame = this.replay.frames.find(
      (candidate, index) => index > frameIndex
        && isLiveGameplayFrame(candidate, this.liveGameState)
    );
    if (!nextLiveFrame || nextLiveFrame.time === this.currentTime) {
      return false;
    }

    this.currentTime = nextLiveFrame.time;
    if (this.playing) {
      this.reanchorPlaybackClock(now);
    }
    return true;
  }

  private skipPostGoalTransitionIfNeeded(now?: number): boolean {
    if (!this.skipPostGoalTransitionsEnabled) {
      return false;
    }

    const frameIndex = findFrameIndexAtTime(this.replay, this.currentTime);
    const frame = this.replay.frames[frameIndex];
    if (!frame || !isPostGoalTransitionFrame(
      this.replay,
      frame,
      frameIndex,
      this.liveGameState,
      this.kickoffGameState,
    )) {
      return false;
    }

    const nextFrame = this.replay.frames.find(
      (candidate, index) =>
        index > frameIndex && !isPostGoalTransitionFrame(
          this.replay,
          candidate,
          index,
          this.liveGameState,
          this.kickoffGameState,
        )
    );
    if (!nextFrame || nextFrame.time === this.currentTime) {
      return false;
    }

    this.currentTime = nextFrame.time;
    if (this.playing) {
      this.reanchorPlaybackClock(now);
    }
    return true;
  }

  private getActiveMetadata(
    frameIndex: number,
    currentTime: number
  ): ReplayPlayerActiveMetadata | null {
    return getKickoffCountdownMetadata(
      this.replay,
      frameIndex,
      currentTime,
    );
  }

  private computeTimelineSegments(): ReplayPlayerTimelineSegment[] {
    return computeTimelineSegments(
      this.replay,
      this.skipPostGoalTransitionsEnabled,
      this.skipKickoffsEnabled,
      this.liveGameState,
      this.kickoffGameState,
    );
  }

  private installPlugin(
    definition: ReplayPlayerPluginDefinition,
    renderAfterSetup: boolean
  ): () => void {
    const plugin =
      typeof definition === "function" ? definition() : definition;

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

  private createPluginStateContext(
    state: ReplayPlayerState
  ): ReplayPlayerPluginStateContext {
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
    players: ReplayPlayerRenderTrackContext[]
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

  private updateBoostTrail(
    boostTrail: THREE.Group,
    boostActive: boolean,
    boostFraction: number,
    time: number,
    playerIndex: number
  ): void {
    if (!boostActive) {
      boostTrail.visible = false;
      return;
    }

    boostTrail.visible = true;

    const phase = time * 36 + playerIndex * 1.7;
    const pulse = 0.86 + 0.14 * Math.sin(phase);
    const intensity = THREE.MathUtils.clamp(0.62 + boostFraction * 0.88, 0.62, 1.5);
    const lengthScale = intensity * (1.02 + pulse * 0.52);
    const widthScale = 1.02 + intensity * 0.28;
    boostTrail.scale.set(lengthScale, widthScale, widthScale);

    for (const [index, child] of boostTrail.children.entries()) {
      const plume = child as THREE.Group;
      const plumePulse = 0.92 + 0.14 * Math.sin(phase + index * 0.85);
      plume.scale.setScalar(plumePulse);

      plume.traverse((node: THREE.Object3D) => {
        if (!(node instanceof THREE.Mesh)) {
          return;
        }

        const material = node.material;
        if (!(material instanceof THREE.MeshBasicMaterial)) {
          return;
        }

        switch (node.name) {
          case "outer-flame":
            material.opacity = 0.24 + intensity * 0.24;
            break;
          case "inner-flame":
            material.opacity = 0.58 + intensity * 0.3;
            break;
          case "glow":
            material.opacity = 0.4 + intensity * 0.26;
            break;
          default:
            break;
        }
      });
    }
  }
}
