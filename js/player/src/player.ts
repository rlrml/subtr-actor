import * as THREE from "three";
import {
  createReplayScene,
  EXAMPLE_CAR_WHEEL_RADIUS_UU,
  getCarWheels,
  setHitboxOverlayOnlyMode,
  updateBoostMeter,
  type ReplayScene,
} from "./scene";
import { createBoostPadOverlayPlugin } from "./boost-pad-overlay";
import { findFrameIndexAtTime } from "./replay-data";
import {
  DEFAULT_CAMERA_VIEW_MODE,
  getActiveDemoEvent,
  getKickoffSkipTargetTime,
  getPostGoalTransitionSkipTargetTime,
  isPlayerSamplePresent,
  normalizeCustomCameraSettings,
  resolveInitialPlayerSettings,
  updateBoostTrail,
  updateDemoIndicator,
  updateReplayBallRender,
} from "./player-helpers";
import {
  clampFrameIndex,
  computeTimelineSegments,
  getFrameWindow,
  getKickoffCountdownMetadata,
  getReplayPlaybackEndTime,
  inferKickoffGameState,
  inferLiveGameState,
  projectReplayTimeToTimeline,
  projectTimelineTimeToReplay,
} from "./player-internals/timeline";
import {
  type AttachedCameraBlendState,
  getFreeCameraPreset,
  interpolateQuaternion,
  interpolatePositionHermite,
  isPositionDiscontinuity,
  rootPosition,
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
  Vec3,
} from "./types";

const DEFAULT_FIELD_SCALE = 1;

const WHEEL_MAX_STEER_RAD = Math.PI / 6;
const WHEEL_SPIN_AXIS = new THREE.Vector3(0, 1, 0);
const WHEEL_LAY_DOWN_QUAT = new THREE.Quaternion().setFromAxisAngle(
  new THREE.Vector3(0, 0, 1),
  Math.PI / 2,
);
const wheelSpinQuat = new THREE.Quaternion();

/**
 * Animates the example car's wheels from replay data: front wheels steer from
 * the normalized steer input, and all wheels spin proportionally to how far the
 * car travelled (accumulated on the mesh's userData). Wheels always spin in the
 * roll direction to avoid the wagon-wheel strobe. No-op for car meshes without
 * the example-car wheel rig.
 */
function updateExampleCarWheels(
  mesh: THREE.Object3D,
  steer: number | null | undefined,
  position: Vec3,
): void {
  const wheels = getCarWheels(mesh);
  if (!wheels) {
    return;
  }
  const wheelState = mesh.userData as { wheelSpin?: number; lastWheelPos?: Vec3 };
  const last = wheelState.lastWheelPos;
  if (last) {
    const distance = Math.hypot(position.x - last.x, position.y - last.y, position.z - last.z);
    wheelState.wheelSpin = (wheelState.wheelSpin ?? 0) + distance / EXAMPLE_CAR_WHEEL_RADIUS_UU;
  }
  wheelState.lastWheelPos = { x: position.x, y: position.y, z: position.z };

  wheelSpinQuat.setFromAxisAngle(WHEEL_SPIN_AXIS, wheelState.wheelSpin ?? 0);
  const steerAngle = -(steer ?? 0) * WHEEL_MAX_STEER_RAD;
  for (const { pivot, wheel, isFront } of wheels) {
    wheel.quaternion.copy(WHEEL_LAY_DOWN_QUAT).multiply(wheelSpinQuat);
    if (isFront) {
      pivot.rotation.z = steerAngle;
    }
  }
}

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
  private readonly attachedCameraBlendState: AttachedCameraBlendState;
  private readonly liveGameState: number | null;
  private readonly kickoffGameState: number | null;
  private timelineSegmentsCacheKey: string | null = null;
  private timelineSegmentsCache: ReplayPlayerTimelineSegment[] = [];
  private resizeObserver: ResizeObserver | null = null;
  private animationFrameId: number | null = null;
  private disposed = false;
  private playing = false;
  private speed = 1;
  private currentTime = 0;
  private lastCameraRenderAt: number | null = null;
  private playbackStartedAt = 0;
  private playbackStartedTime = 0;
  private cameraDistanceScale: number;
  private customCameraSettings: CameraSettings | null;
  private cameraViewMode: ReplayCameraViewMode;
  private freeCameraTransition: FreeCameraTransition | null = null;
  private attachedPlayerId: string | null;
  private ballCamEnabled: boolean;
  private useReplayBallCam: boolean;
  private useReplayCameraLook: boolean;
  private lastEffectiveBallCamEnabled = false;
  private boostMeterEnabled: boolean;
  private boostPickupAnimationEnabled: boolean;
  private hitboxWireframesEnabled: boolean;
  private hitboxOnlyModeEnabled: boolean;
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
    const initialSettings = resolveInitialPlayerSettings(options);
    this.speed = initialSettings.speed;
    this.cameraDistanceScale = initialSettings.cameraDistanceScale;
    this.customCameraSettings = initialSettings.customCameraSettings;
    this.attachedPlayerId = initialSettings.attachedPlayerId;
    this.cameraViewMode = initialSettings.cameraViewMode;
    this.ballCamEnabled = initialSettings.ballCamEnabled;
    this.useReplayBallCam = initialSettings.useReplayBallCam;
    this.useReplayCameraLook = options.initialUseReplayCameraLook ?? false;
    this.lastEffectiveBallCamEnabled = initialSettings.ballCamEnabled;
    this.attachedCameraBlendState = {
      currentBlend: initialSettings.ballCamEnabled ? 1 : 0,
      targetBlend: initialSettings.ballCamEnabled ? 1 : 0,
      lastIsBallCam: initialSettings.ballCamEnabled,
    };
    this.boostMeterEnabled = initialSettings.boostMeterEnabled;
    this.boostPickupAnimationEnabled = initialSettings.boostPickupAnimationEnabled;
    this.hitboxWireframesEnabled = initialSettings.hitboxWireframesEnabled;
    this.hitboxOnlyModeEnabled = initialSettings.hitboxOnlyModeEnabled;
    this.skipPostGoalTransitionsEnabled = initialSettings.skipPostGoalTransitionsEnabled;
    this.skipKickoffsEnabled = initialSettings.skipKickoffsEnabled;
    this.setHitboxVisualizationVisibility();

    this.installResizeHandling();
    for (const plugin of options.plugins ?? []) {
      this.installPlugin(plugin, false);
    }
    if (!this.plugins.some((entry) => entry.plugin.id === "boost-pad-overlay")) {
      this.installPlugin(createBoostPadOverlayPlugin(), false);
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
    this.skipPostGoalTransitionIfNeeded();
    this.skipPastKickoffIfNeeded();
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
    const { fov, position, target, up } = getFreeCameraPreset(
      preset,
      this.fieldScale,
      this.sceneState.camera.aspect,
    );
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
    // A manual toggle is an explicit override: stop following the replay's
    // ball-cam state until the caller re-enables replay-driven mode.
    this.useReplayBallCam = false;
    this.render();
    this.emitChange();
  }

  setUseReplayBallCam(enabled: boolean): void {
    this.useReplayBallCam = enabled;
    this.render();
    this.emitChange();
  }

  setUseReplayCameraLook(enabled: boolean): void {
    this.useReplayCameraLook = enabled;
    this.render();
    this.emitChange();
  }

  /**
   * The ball-cam state to actually apply this frame. When replay-driven ball
   * cam is enabled and we are following a player, we resolve their ball-cam
   * toggle from the coalesced camera-event stream (the last change at or before
   * this frame). Otherwise we fall back to the manual `ballCamEnabled` flag.
   */
  private resolveEffectiveBallCam(frameIndex: number): boolean {
    if (this.useReplayBallCam && this.attachedPlayerId) {
      const attachedPlayer = this.replay.players.find(
        (player) => player.id === this.attachedPlayerId,
      );
      const events = attachedPlayer?.cameraEvents;
      if (events) {
        for (let index = events.length - 1; index >= 0; index -= 1) {
          const event = events[index]!;
          if (event.frame > frameIndex) {
            continue;
          }
          if (event.ballCamActive != null) {
            return event.ballCamActive;
          }
        }
      }
    }
    return this.ballCamEnabled;
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

  setHitboxWireframesEnabled(enabled: boolean): void {
    this.hitboxWireframesEnabled = enabled;
    this.setHitboxVisualizationVisibility();
    this.render();
    this.emitChange();
  }

  setHitboxOnlyModeEnabled(enabled: boolean): void {
    this.hitboxOnlyModeEnabled = enabled;
    this.setHitboxVisualizationVisibility();
    this.render();
    this.emitChange();
  }

  setSkipPostGoalTransitionsEnabled(enabled: boolean): void {
    this.skipPostGoalTransitionsEnabled = enabled;
    if (enabled && this.playing) {
      this.skipPostGoalTransitionIfNeeded();
    }
    this.render();
    this.emitChange();
  }

  setSkipKickoffsEnabled(enabled: boolean): void {
    this.skipKickoffsEnabled = enabled;
    if (enabled && this.playing) {
      this.skipPostGoalTransitionIfNeeded();
      this.skipPastKickoffIfNeeded();
    }
    this.render();
    this.emitChange();
  }

  seek(time: number): void {
    this.currentTime = this.clampReplayTime(time);
    if (this.playing) {
      this.skipPostGoalTransitionIfNeeded();
      this.skipPastKickoffIfNeeded();
    }
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
      // Patching the manual flag is a manual override unless the patch also
      // explicitly re-enables replay-driven ball cam.
      if (nextState.useReplayBallCam === undefined) {
        this.useReplayBallCam = false;
      }
    }
    if (nextState.useReplayBallCam !== undefined) {
      this.useReplayBallCam = nextState.useReplayBallCam;
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
    if (nextState.hitboxWireframesEnabled !== undefined) {
      this.hitboxWireframesEnabled = nextState.hitboxWireframesEnabled;
      this.setHitboxVisualizationVisibility();
    }
    if (nextState.hitboxOnlyModeEnabled !== undefined) {
      this.hitboxOnlyModeEnabled = nextState.hitboxOnlyModeEnabled;
      this.setHitboxVisualizationVisibility();
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
      this.skipPostGoalTransitionIfNeeded(now);
      this.skipPastKickoffIfNeeded(now);
      this.reanchorPlaybackClock(now);
    }

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
      useReplayBallCam: this.useReplayBallCam,
      effectiveBallCamEnabled: this.lastEffectiveBallCamEnabled,
      boostMeterEnabled: this.boostMeterEnabled,
      boostPickupAnimationEnabled: this.boostPickupAnimationEnabled,
      hitboxWireframesEnabled: this.hitboxWireframesEnabled,
      hitboxOnlyModeEnabled: this.hitboxOnlyModeEnabled,
      skipPostGoalTransitionsEnabled: this.skipPostGoalTransitionsEnabled,
      skipKickoffsEnabled: this.skipKickoffsEnabled,
    };
  }

  getSnapshot(): ReplayPlayerSnapshot {
    return this.getState();
  }

  getTimelineDuration(): number {
    return this.replay.duration;
  }

  getTimelineCurrentTime(): number {
    return this.projectReplayTimeToTimeline(this.currentTime).timelineTime;
  }

  getTimelineSegments(): ReplayPlayerTimelineSegment[] {
    const cacheKey = `${this.skipPostGoalTransitionsEnabled}:${this.skipKickoffsEnabled}`;
    if (this.timelineSegmentsCacheKey === cacheKey) {
      return this.timelineSegmentsCache;
    }

    this.timelineSegmentsCacheKey = cacheKey;
    this.timelineSegmentsCache = this.computeTimelineSegments();
    return this.timelineSegmentsCache;
  }

  projectReplayTimeToTimeline(replayTime: number): ReplayPlayerTimelineProjection {
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

  private clampReplayTime(time: number): number {
    return THREE.MathUtils.clamp(time, 0, this.replay.duration);
  }

  private getPlaybackEndTime(): number {
    return getReplayPlaybackEndTime(this.replay.duration, this.getTimelineSegments());
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

  private setHitboxVisualizationVisibility(): void {
    for (const hitbox of this.sceneState.playerHitboxes.values()) {
      hitbox.visible = this.hitboxWireframesEnabled || this.hitboxOnlyModeEnabled;
      setHitboxOverlayOnlyMode(hitbox, this.hitboxOnlyModeEnabled);
    }
    for (const body of this.sceneState.playerBodyMeshes.values()) {
      body.visible = !this.hitboxOnlyModeEnabled;
    }
    if (this.hitboxOnlyModeEnabled) {
      for (const boostTrail of this.sceneState.playerBoostTrails.values()) {
        boostTrail.visible = false;
      }
      for (const meter of this.sceneState.playerBoostMeters.values()) {
        meter.group.visible = false;
      }
    }
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

  private getCameraRenderDelta(): number {
    const now = typeof performance === "undefined" ? Date.now() : performance.now();
    const previous = this.lastCameraRenderAt;
    this.lastCameraRenderAt = now;
    if (previous === null) {
      return 1 / 60;
    }
    return Math.max(0, Math.min(0.1, (now - previous) / 1000));
  }

  private render(): void {
    const frameWindow = getFrameWindow(this.replay, this.currentTime);
    const frameIndex = frameWindow.frameIndex;
    const { ballFrame, nextBallFrame, ballPosition } = updateReplayBallRender({
      replay: this.replay,
      sceneState: this.sceneState,
      fieldScale: this.fieldScale,
      frameWindow,
    });
    const renderPlayers: ReplayPlayerRenderTrackContext[] = [];

    for (const [playerIndex, player] of this.replay.players.entries()) {
      const mesh = this.sceneState.playerMeshes.get(player.id);
      const boostTrail = this.sceneState.playerBoostTrails.get(player.id);
      const boostMeter = this.sceneState.playerBoostMeters.get(player.id);
      const demoIndicator = this.sceneState.playerDemoIndicators.get(player.id);
      const frame = player.frames[frameIndex] ?? null;
      const nextFrame = player.frames[frameWindow.nextFrameIndex] ?? frame;
      let interpolatedPosition: Vec3 | null = null;
      let renderPosition: Vec3 | null = null;
      let boostFraction = 0;
      if (!mesh) {
        if (demoIndicator) {
          demoIndicator.group.visible = false;
        }
        renderPlayers.push({
          track: player,
          mesh: null,
          boostTrail: boostTrail ?? null,
          frame,
          nextFrame,
          interpolatedPosition: renderPosition,
          boostFraction,
        });
        continue;
      }

      interpolatedPosition = interpolatePositionHermite(
        frame?.position ?? null,
        nextFrame?.position ?? null,
        frame?.linearVelocity ?? null,
        nextFrame?.linearVelocity ?? null,
        frameWindow.dt,
        frameWindow.alpha,
      );
      const activeDemoEvent = getActiveDemoEvent(
        this.replay.timelineEvents,
        player.id,
        this.currentTime,
      );
      if (!interpolatedPosition) {
        mesh.visible = false;
        if (boostTrail) {
          boostTrail.visible = false;
        }
        if (boostMeter) {
          boostMeter.group.visible = false;
        }
        updateDemoIndicator({
          indicator: demoIndicator ?? null,
          fallbackPosition: null,
          demoEvent: activeDemoEvent,
          currentTime: this.currentTime,
          camera: this.sceneState.camera,
        });
        renderPlayers.push({
          track: player,
          mesh,
          boostTrail: boostTrail ?? null,
          frame,
          nextFrame,
          interpolatedPosition: renderPosition,
          boostFraction,
        });
        continue;
      }

      if (activeDemoEvent) {
        mesh.visible = false;
        if (boostTrail) {
          boostTrail.visible = false;
        }
        if (boostMeter) {
          boostMeter.group.visible = false;
        }
        updateDemoIndicator({
          indicator: demoIndicator ?? null,
          fallbackPosition: interpolatedPosition,
          demoEvent: activeDemoEvent,
          currentTime: this.currentTime,
          camera: this.sceneState.camera,
        });
        renderPlayers.push({
          track: player,
          mesh,
          boostTrail: boostTrail ?? null,
          frame,
          nextFrame,
          interpolatedPosition: renderPosition,
          boostFraction,
        });
        continue;
      }

      const playerVisible = isPlayerSamplePresent(frame);
      if (!playerVisible) {
        mesh.visible = false;
        if (boostTrail) {
          boostTrail.visible = false;
        }
        if (boostMeter) {
          boostMeter.group.visible = false;
        }
        updateDemoIndicator({
          indicator: demoIndicator ?? null,
          fallbackPosition: interpolatedPosition,
          demoEvent: null,
          currentTime: this.currentTime,
          camera: this.sceneState.camera,
        });
        renderPlayers.push({
          track: player,
          mesh,
          boostTrail: boostTrail ?? null,
          frame,
          nextFrame,
          interpolatedPosition: renderPosition,
          boostFraction,
        });
        continue;
      }

      mesh.visible = true;
      if (demoIndicator) {
        demoIndicator.group.visible = false;
      }
      renderPosition = interpolatedPosition;
      mesh.position.copy(rootPosition(interpolatedPosition));
      // When the position snaps across a teleport (demo respawn / kickoff
      // reposition), snap the orientation too instead of slerping through the
      // relocation, which otherwise spins the car between frames.
      const positionTeleport = isPositionDiscontinuity(
        frame?.position ?? null,
        nextFrame?.position ?? null,
        frameWindow.dt,
      );
      const rotation = interpolateQuaternion(
        frame?.rotation ?? null,
        positionTeleport ? null : (nextFrame?.rotation ?? null),
        frameWindow.alpha,
      );
      if (rotation) {
        mesh.quaternion.copy(rotation);
      } else {
        mesh.quaternion.identity();
      }

      updateExampleCarWheels(mesh, frame?.steer, interpolatedPosition);

      const currentBoostFraction = frame?.boostFraction ?? 0;
      const nextBoostFraction = nextFrame?.boostFraction ?? currentBoostFraction;
      boostFraction = THREE.MathUtils.lerp(
        currentBoostFraction,
        nextBoostFraction,
        frameWindow.alpha,
      );

      if (boostTrail) {
        const boostActive =
          (frameWindow.alpha >= 0.5 ? nextFrame?.boostActive : frame?.boostActive) ??
          frame?.boostActive ??
          nextFrame?.boostActive ??
          false;
        if (this.hitboxOnlyModeEnabled) {
          boostTrail.visible = false;
        } else {
          updateBoostTrail(boostTrail, boostActive, boostFraction, this.currentTime, playerIndex);
        }
      }

      if (boostMeter) {
        if (this.boostMeterEnabled && !this.hitboxOnlyModeEnabled) {
          boostMeter.group.visible = true;
          updateBoostMeter(
            boostMeter,
            boostFraction,
            THREE.MathUtils.lerp(
              frame?.boostAmount ?? 0,
              nextFrame?.boostAmount ?? frame?.boostAmount ?? 0,
              frameWindow.alpha,
            ),
            this.sceneState.camera,
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
        interpolatedPosition: renderPosition,
        boostFraction,
      });
    }

    const effectiveBallCamEnabled = this.resolveEffectiveBallCam(frameIndex);
    this.lastEffectiveBallCamEnabled = effectiveBallCamEnabled;
    updateAttachedCamera({
      sceneState: this.sceneState,
      replay: this.replay,
      fieldScale: this.fieldScale,
      cameraViewMode: this.cameraViewMode,
      attachedPlayerId: this.attachedPlayerId,
      ballCamEnabled: effectiveBallCamEnabled,
      cameraDistanceScale: this.cameraDistanceScale,
      customCameraSettings: this.customCameraSettings,
      frameIndex,
      nextFrameIndex: frameWindow.nextFrameIndex,
      alpha: frameWindow.alpha,
      dt: frameWindow.dt,
      renderDelta: this.getCameraRenderDelta(),
      attachedPlayerUnavailable:
        this.attachedPlayerId !== null &&
        getActiveDemoEvent(this.replay.timelineEvents, this.attachedPlayerId, this.currentTime) !==
          null,
      ballPosition,
      desiredCameraPosition: this.desiredCameraPosition,
      desiredLookTarget: this.desiredLookTarget,
      blendState: this.attachedCameraBlendState,
      replayCameraLook: this.useReplayCameraLook,
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

    const targetTime = getKickoffSkipTargetTime(
      this.replay,
      this.currentTime,
      this.liveGameState,
      this.kickoffGameState,
    );
    if (targetTime === null) {
      return false;
    }

    this.currentTime = targetTime;
    if (this.playing) {
      this.reanchorPlaybackClock(now);
    }
    return true;
  }

  private skipPostGoalTransitionIfNeeded(now?: number): boolean {
    if (!this.skipPostGoalTransitionsEnabled) {
      return false;
    }

    const targetTime = getPostGoalTransitionSkipTargetTime(
      this.replay,
      this.currentTime,
      this.liveGameState,
      this.kickoffGameState,
    );
    if (targetTime === null) {
      return false;
    }

    this.currentTime = targetTime;
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
