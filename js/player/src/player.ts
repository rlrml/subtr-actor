import * as THREE from "three";
import { createReplayScene, updateBoostMeter, type ReplayScene } from "./scene";
import { findFrameIndexAtTime } from "./replay-data";
import type {
  BeforeRenderCallback,
  FrameRenderInfo,
  ReplayPlayerActiveMetadata,
  ReplayModel,
  ReplayPlayerKickoffCountdownMetadata,
  ReplayPlayerPlugin,
  ReplayPlayerPluginContext,
  ReplayPlayerPluginDefinition,
  ReplayPlayerPluginStateContext,
  ReplayPlayerRenderContext,
  ReplayPlayerRenderTrackContext,
  ReplayPlayerOptions,
  ReplayPlayerSnapshot,
  ReplayPlayerState,
  ReplayPlayerStatePatch,
  Vec3,
} from "./types";

const DEFAULT_FIELD_SCALE = 1;
const DEFAULT_CAMERA_DISTANCE_SCALE = 2.25;
const CHASE_CAMERA_HEIGHT_MULTIPLIER = 1.4;
const CAMERA_SMOOTHING = 0.18;
const GROUND_HEIGHT_THRESHOLD_UU = 120;
const MIN_CAMERA_HEIGHT_UU = 90;
const PLAYER_FOCUS_HEIGHT_UU = 40;
const BALL_CAM_HEIGHT_BIAS_UU = 45;
const BALL_CAM_LOOK_BLEND = 0.58;
const BALL_CAM_DIRECTION_BLEND = 0.82;
const BALL_CAM_MAX_FOV = 132;
const DEFAULT_FORWARD = new THREE.Vector3(-1, 0, 0);
const DEFAULT_UP = new THREE.Vector3(0, 0, 1);

type ReplayPlayerListener = (state: ReplayPlayerState) => void;
type FrameWindow = {
  frameIndex: number;
  nextFrameIndex: number;
  alpha: number;
};
type InstalledReplayPlayerPlugin = {
  definition: ReplayPlayerPluginDefinition;
  plugin: ReplayPlayerPlugin;
};

function inferLiveGameState(replay: ReplayModel): number | null {
  if (replay.frames.length === 0) {
    return null;
  }

  const counts = new Map<number, number>();
  for (const frame of replay.frames) {
    counts.set(frame.gameState, (counts.get(frame.gameState) ?? 0) + 1);
  }

  let liveGameState: number | null = null;
  let liveGameStateCount = -1;
  for (const [gameState, count] of counts.entries()) {
    if (count <= liveGameStateCount) {
      continue;
    }

    liveGameState = gameState;
    liveGameStateCount = count;
  }

  return liveGameState;
}

function inferKickoffGameState(
  replay: ReplayModel,
  liveGameState: number | null
): number | null {
  if (liveGameState === null) {
    return null;
  }

  for (const frame of replay.frames) {
    if (frame.gameState === liveGameState) {
      break;
    }

    return frame.gameState;
  }

  return null;
}

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
    const frameWindow = this.getFrameWindow(this.currentTime);
    const frameIndex = frameWindow.frameIndex;
    const ballFrame = this.replay.ballFrames[frameIndex] ?? null;
    const nextBallFrame =
      this.replay.ballFrames[frameWindow.nextFrameIndex] ?? ballFrame;
    const interpolatedBallPosition = this.interpolatePosition(
      ballFrame?.position ?? null,
      nextBallFrame?.position ?? null,
      frameWindow.alpha
    );
    const ballPosition = interpolatedBallPosition
      ? this.worldPosition(interpolatedBallPosition)
      : null;
    const renderPlayers: ReplayPlayerRenderTrackContext[] = [];

    if (interpolatedBallPosition) {
      this.sceneState.ballMesh.visible = true;
      this.sceneState.ballMesh.position.copy(
        this.rootPosition(interpolatedBallPosition)
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

      interpolatedPosition = this.interpolatePosition(
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
      mesh.position.copy(this.rootPosition(interpolatedPosition));
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

    this.updateCamera(frameIndex, ballPosition);
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
    if (!frame || !this.isKickoffFrame(frame)) {
      return false;
    }

    const nextLiveFrame = this.replay.frames.find(
      (candidate, index) => index > frameIndex && this.isLiveGameplayFrame(candidate)
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
    if (!frame || !this.isPostGoalTransitionFrame(frame, frameIndex)) {
      return false;
    }

    const nextFrame = this.replay.frames.find(
      (candidate, index) =>
        index > frameIndex && !this.isPostGoalTransitionFrame(candidate, index)
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

  private isLiveGameplayFrame(frame: ReplayModel["frames"][number]): boolean {
    if (this.liveGameState === null) {
      return frame.kickoffCountdown <= 0;
    }

    return frame.gameState === this.liveGameState;
  }

  private isKickoffFrame(frame: ReplayModel["frames"][number]): boolean {
    if (frame.kickoffCountdown > 0) {
      return true;
    }

    return this.kickoffGameState !== null && frame.gameState === this.kickoffGameState;
  }

  private getActiveMetadata(
    frameIndex: number,
    currentTime: number
  ): ReplayPlayerActiveMetadata | null {
    return this.getKickoffCountdownMetadata(frameIndex, currentTime);
  }

  private getKickoffCountdownMetadata(
    frameIndex: number,
    currentTime: number
  ): ReplayPlayerKickoffCountdownMetadata | null {
    const currentFrame = this.replay.frames[frameIndex];
    if (!currentFrame || currentFrame.kickoffCountdown <= 0) {
      return null;
    }

    let startIndex = frameIndex;
    while (
      startIndex > 0 &&
      (this.replay.frames[startIndex - 1]?.kickoffCountdown ?? 0) > 0
    ) {
      startIndex -= 1;
    }

    let endIndex = frameIndex + 1;
    while (
      endIndex < this.replay.frames.length &&
      this.replay.frames[endIndex].kickoffCountdown > 0
    ) {
      endIndex += 1;
    }

    let maxCountdown = 0;
    for (let index = startIndex; index < endIndex; index += 1) {
      maxCountdown = Math.max(
        maxCountdown,
        this.replay.frames[index].kickoffCountdown
      );
    }

    const endsAt = this.replay.frames[endIndex]?.time ?? this.replay.duration;
    const secondsRemaining = Math.max(0, endsAt - currentTime);

    return {
      kind: "kickoff-countdown",
      countdown: Math.max(1, Math.min(maxCountdown, Math.ceil(secondsRemaining))),
      secondsRemaining,
      endsAt,
    };
  }

  private hasRenderableSamples(frameIndex: number): boolean {
    if (this.replay.ballFrames[frameIndex]?.position) {
      return true;
    }

    return this.replay.players.some((player) => player.frames[frameIndex]?.position);
  }

  private isRenderableKickoffFrame(
    frame: ReplayModel["frames"][number],
    frameIndex: number
  ): boolean {
    return this.isKickoffFrame(frame) && this.hasRenderableSamples(frameIndex);
  }

  private isPostGoalTransitionFrame(
    frame: ReplayModel["frames"][number],
    frameIndex: number
  ): boolean {
    return !this.isLiveGameplayFrame(frame) && !this.isRenderableKickoffFrame(frame, frameIndex);
  }

  private updateCamera(
    frameIndex: number,
    ballPosition: THREE.Vector3 | null
  ): void {
    const controls = this.sceneState.controls;

    if (!this.attachedPlayerId) {
      controls.enabled = true;
      this.sceneState.camera.fov = 48;
      this.sceneState.camera.updateProjectionMatrix();
      return;
    }

    const attachedPlayer = this.replay.players.find(
      (player) => player.id === this.attachedPlayerId
    );
    const frame = attachedPlayer?.frames[frameIndex];

    if (!attachedPlayer || !frame?.position) {
      controls.enabled = true;
      return;
    }

    controls.enabled = false;

    const basePosition = this.worldPosition(frame.position);
    const orientation = this.getOrientationVectors(frame);
    const forward = orientation?.forward ?? DEFAULT_FORWARD.clone();
    const right = orientation?.right ?? new THREE.Vector3(0, 1, 0);

    const cameraSettings = attachedPlayer.cameraSettings;
    const distance =
      (cameraSettings.distance ?? 270) *
      this.fieldScale *
      this.cameraDistanceScale;
    const height =
      (cameraSettings.height ?? 100) *
      this.fieldScale *
      CHASE_CAMERA_HEIGHT_MULTIPLIER;
    const pitch = THREE.MathUtils.degToRad(cameraSettings.pitch ?? -4);
    const lookDirection = forward
      .clone()
      .applyAxisAngle(right, pitch)
      .normalize();
    const chaseAnchor = basePosition
      .clone()
      .addScaledVector(DEFAULT_UP, height);
    const playerFocusPoint = basePosition
      .clone()
      .addScaledVector(DEFAULT_UP, PLAYER_FOCUS_HEIGHT_UU * this.fieldScale);
    let targetFov = cameraSettings.fov ?? 110;

    if (this.ballCamEnabled && ballPosition) {
      const ballFocusPoint = ballPosition
        .clone()
        .addScaledVector(DEFAULT_UP, BALL_CAM_HEIGHT_BIAS_UU * this.fieldScale);
      const playerToBall = ballFocusPoint.clone().sub(playerFocusPoint);
      const ballCamDirection = (
        playerToBall.lengthSq() > 0.0001
          ? playerToBall.normalize()
          : lookDirection.clone()
      )
        .multiplyScalar(BALL_CAM_DIRECTION_BLEND)
        .addScaledVector(lookDirection, 1 - BALL_CAM_DIRECTION_BLEND)
        .normalize();

      this.desiredCameraPosition
        .copy(chaseAnchor)
        .addScaledVector(ballCamDirection, -distance);
      this.desiredCameraPosition.z = Math.max(
        MIN_CAMERA_HEIGHT_UU * this.fieldScale,
        this.desiredCameraPosition.z
      );
      this.desiredLookTarget
        .copy(playerFocusPoint)
        .lerp(ballFocusPoint, BALL_CAM_LOOK_BLEND);
      const cameraToPlayer = playerFocusPoint.clone().sub(this.desiredCameraPosition);
      const cameraToBall = ballFocusPoint.clone().sub(this.desiredCameraPosition);
      if (cameraToPlayer.lengthSq() > 0.0001 && cameraToBall.lengthSq() > 0.0001) {
        const separationAngle = cameraToPlayer.angleTo(cameraToBall);
        targetFov = Math.min(
          BALL_CAM_MAX_FOV,
          Math.max(targetFov, THREE.MathUtils.radToDeg(separationAngle) * 1.7)
        );
      }
    } else {
      this.desiredCameraPosition
        .copy(chaseAnchor)
        .addScaledVector(forward, -distance);
      this.desiredCameraPosition.z = Math.max(
        MIN_CAMERA_HEIGHT_UU * this.fieldScale,
        this.desiredCameraPosition.z
      );
      this.desiredLookTarget
        .copy(basePosition)
        .addScaledVector(lookDirection, distance + 8 * this.fieldScale)
        .addScaledVector(DEFAULT_UP, PLAYER_FOCUS_HEIGHT_UU * this.fieldScale);
    }

    this.sceneState.camera.position.lerp(
      this.desiredCameraPosition,
      CAMERA_SMOOTHING
    );
    this.sceneState.camera.up.lerp(DEFAULT_UP, CAMERA_SMOOTHING).normalize();
    controls.target.lerp(this.desiredLookTarget, CAMERA_SMOOTHING);
    this.sceneState.camera.fov = THREE.MathUtils.lerp(
      this.sceneState.camera.fov,
      targetFov,
      CAMERA_SMOOTHING
    );
    this.sceneState.camera.updateProjectionMatrix();
    this.sceneState.camera.lookAt(controls.target);
  }

  private getOrientationVectors(
    frame: ReplayModel["players"][number]["frames"][number]
  ): {
    forward: THREE.Vector3;
    up: THREE.Vector3;
    right: THREE.Vector3;
  } | null {
    const velocity = frame.linearVelocity ? this.worldDirection(frame.linearVelocity) : null;
    const rawForward = frame.forward ? this.worldDirection(frame.forward) : null;
    const rawUp = frame.up ? this.worldDirection(frame.up) : null;
    const grounded = (frame.position?.z ?? Infinity) < GROUND_HEIGHT_THRESHOLD_UU;

    if (grounded) {
      const forward = (rawForward ?? velocity ?? DEFAULT_FORWARD.clone())
        .clone()
        .setZ(0);

      if (forward.lengthSq() < 0.0001) {
        return null;
      }

      forward.normalize();
      if (velocity && velocity.lengthSq() > 0.0001 && forward.dot(velocity) < 0) {
        forward.negate();
      }
      const right = new THREE.Vector3()
        .crossVectors(DEFAULT_UP, forward)
        .normalize();
      const up = new THREE.Vector3()
        .crossVectors(forward, right)
        .normalize();
      return { forward, up, right };
    }

    if (!rawForward || !rawUp) {
      return null;
    }

    const forward = rawForward.clone().normalize();
    const right = new THREE.Vector3().crossVectors(rawUp, forward).normalize();
    const up = new THREE.Vector3()
      .crossVectors(forward, right)
      .normalize();

    return { forward, up, right };
  }

  private getFrameWindow(time: number): FrameWindow {
    const frameIndex = findFrameIndexAtTime(this.replay, time);
    const nextFrameIndex = Math.min(frameIndex + 1, this.replay.frames.length - 1);

    if (nextFrameIndex === frameIndex) {
      return { frameIndex, nextFrameIndex, alpha: 0 };
    }

    const startTime = this.replay.frames[frameIndex]?.time ?? 0;
    const endTime = this.replay.frames[nextFrameIndex]?.time ?? startTime;
    if (endTime <= startTime) {
      return { frameIndex, nextFrameIndex, alpha: 0 };
    }

    return {
      frameIndex,
      nextFrameIndex,
      alpha: THREE.MathUtils.clamp((time - startTime) / (endTime - startTime), 0, 1),
    };
  }

  private interpolatePosition(
    current: Vec3 | null,
    next: Vec3 | null,
    alpha: number
  ): Vec3 | null {
    if (!current) {
      return next;
    }

    if (!next || alpha <= 0) {
      return current;
    }

    return {
      x: THREE.MathUtils.lerp(current.x, next.x, alpha),
      y: THREE.MathUtils.lerp(current.y, next.y, alpha),
      z: THREE.MathUtils.lerp(current.z, next.z, alpha),
    };
  }

  private rootPosition(position: Vec3): THREE.Vector3 {
    return new THREE.Vector3(position.x, position.y, position.z);
  }

  private worldPosition(position: Vec3): THREE.Vector3 {
    return new THREE.Vector3(
      -position.x * this.fieldScale,
      position.y * this.fieldScale,
      position.z * this.fieldScale
    );
  }

  private worldDirection(direction: Vec3): THREE.Vector3 {
    return new THREE.Vector3(-direction.x, direction.y, direction.z).normalize();
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

      plume.traverse((node) => {
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
