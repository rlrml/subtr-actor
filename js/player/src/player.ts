import * as THREE from "three";
import { createReplayScene, type ReplayScene } from "./scene";
import { findFrameIndexAtTime } from "./replay-data";
import type {
  ReplayModel,
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

export class ReplayPlayer extends EventTarget {
  readonly container: HTMLElement;
  readonly replay: ReplayModel;
  readonly options: ReplayPlayerOptions;

  private readonly sceneState: ReplayScene;
  private readonly fieldScale: number;
  private readonly desiredCameraPosition = new THREE.Vector3();
  private readonly desiredLookTarget = new THREE.Vector3();
  private readonly boundWindowResize = () => this.sceneState.resize();
  private resizeObserver: ResizeObserver | null = null;
  private animationFrameId: number | null = null;
  private lastTickTime = 0;
  private playing = false;
  private speed = 1;
  private currentTime = 0;
  private cameraDistanceScale: number;
  private attachedPlayerId: string | null;
  private ballCamEnabled: boolean;

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
    this.speed = Math.max(0.1, options.initialPlaybackRate ?? 1);
    this.cameraDistanceScale = Math.max(
      0.25,
      options.initialCameraDistanceScale ?? DEFAULT_CAMERA_DISTANCE_SCALE
    );
    this.attachedPlayerId = options.initialAttachedPlayerId ?? null;
    this.ballCamEnabled = options.initialBallCamEnabled ?? false;

    this.installResizeHandling();
    this.render();
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
    this.lastTickTime = performance.now();
    this.animationFrameId = requestAnimationFrame(this.tick);
    this.emitChange();
  }

  pause(): void {
    if (!this.playing && this.animationFrameId === null) {
      return;
    }

    this.playing = false;
    if (this.animationFrameId !== null) {
      cancelAnimationFrame(this.animationFrameId);
      this.animationFrameId = null;
    }
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
    this.speed = Math.max(0.1, speed);
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

  seek(time: number): void {
    this.currentTime = THREE.MathUtils.clamp(time, 0, this.replay.duration);
    this.render();
    this.emitChange();
  }

  setState(nextState: ReplayPlayerStatePatch): void {
    if (nextState.speed !== undefined) {
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
        this.lastTickTime = performance.now();
        this.animationFrameId = requestAnimationFrame(this.tick);
      } else {
        this.playing = false;
        if (this.animationFrameId !== null) {
          cancelAnimationFrame(this.animationFrameId);
          this.animationFrameId = null;
        }
      }
    }

    this.render();
    this.emitChange();
  }

  getState(): ReplayPlayerState {
    return {
      currentTime: this.currentTime,
      duration: this.replay.duration,
      frameIndex: findFrameIndexAtTime(this.replay, this.currentTime),
      playing: this.playing,
      speed: this.speed,
      cameraDistanceScale: this.cameraDistanceScale,
      attachedPlayerId: this.attachedPlayerId,
      ballCamEnabled: this.ballCamEnabled,
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

  destroy(): void {
    this.pause();
    if (this.resizeObserver) {
      this.resizeObserver.disconnect();
      this.resizeObserver = null;
    } else {
      window.removeEventListener("resize", this.boundWindowResize);
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

  private tick = (): void => {
    if (!this.playing) {
      return;
    }

    const now = performance.now();
    const elapsedSeconds = (now - this.lastTickTime) / 1000;
    this.lastTickTime = now;
    this.currentTime += elapsedSeconds * this.speed;

    if (this.currentTime >= this.replay.duration) {
      this.currentTime = this.replay.duration;
      this.playing = false;
      this.animationFrameId = null;
      this.render();
      this.emitChange();
      return;
    }

    this.render();
    this.emitChange();
    this.animationFrameId = requestAnimationFrame(this.tick);
  };

  private render(): void {
    const frameWindow = this.getFrameWindow(this.currentTime);
    const frameIndex = frameWindow.frameIndex;
    const ballFrame = this.replay.ballFrames[frameIndex];
    const nextBallFrame = this.replay.ballFrames[frameWindow.nextFrameIndex] ?? ballFrame;
    const interpolatedBallPosition = this.interpolatePosition(
      ballFrame?.position ?? null,
      nextBallFrame?.position ?? null,
      frameWindow.alpha
    );
    const ballPosition = interpolatedBallPosition
      ? this.worldPosition(interpolatedBallPosition)
      : null;

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

    for (const player of this.replay.players) {
      const mesh = this.sceneState.playerMeshes.get(player.id);
      if (!mesh) {
        continue;
      }

      const frame = player.frames[frameIndex];
      const nextFrame = player.frames[frameWindow.nextFrameIndex] ?? frame;
      const interpolatedPosition = this.interpolatePosition(
        frame?.position ?? null,
        nextFrame?.position ?? null,
        frameWindow.alpha
      );
      if (!interpolatedPosition) {
        mesh.visible = false;
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
    }

    this.updateCamera(frameIndex, ballPosition);
    this.sceneState.controls.update();
    this.sceneState.updateWallVisibility();
    this.sceneState.renderer.render(
      this.sceneState.scene,
      this.sceneState.camera
    );
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

  private emitChange(): void {
    this.dispatchEvent(
      new CustomEvent<ReplayPlayerState>("change", {
        detail: this.getState(),
      })
    );
  }
}
