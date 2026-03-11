import * as THREE from "three";
import { createReplayScene, type ReplayScene } from "./scene";
import { findFrameIndexAtTime } from "./replay-data";
import type {
  CameraMode,
  ReplayModel,
  ReplayPlayerOptions,
  ReplayPlayerSnapshot,
} from "./types";

const UP_OFFSET = 0.25;
const BALL_RADIUS = 1.82;
const CAMERA_DISTANCE_SCALE = 0.01;
const CAMERA_HEIGHT_SCALE = 0.01;
const ATTACHED_DISTANCE_MULTIPLIER = 1.8;
const THIRD_PERSON_DISTANCE_MULTIPLIER = 2.7;
const ATTACHED_HEIGHT_MULTIPLIER = 1.2;
const THIRD_PERSON_HEIGHT_MULTIPLIER = 1.6;
const CAMERA_SMOOTHING = 0.18;
const MIN_CAMERA_HEIGHT = 0.9;
const GROUND_HEIGHT_THRESHOLD = 1.2;
const BALL_CAM_HEIGHT_BIAS = 0.45;
const BALL_CAM_LOOK_BLEND = 0.58;
const BALL_CAM_DIRECTION_BLEND = 0.82;
const BALL_CAM_MAX_FOV = 132;
const DEFAULT_FORWARD = new THREE.Vector3(0, 0, 1);
const DEFAULT_UP = new THREE.Vector3(0, 1, 0);

export class ReplayPlayer extends EventTarget {
  readonly container: HTMLElement;
  readonly replay: ReplayModel;
  readonly options: ReplayPlayerOptions;

  private readonly sceneState: ReplayScene;
  private animationFrameId: number | null = null;
  private lastTickTime = 0;
  private playing = false;
  private speed = 1;
  private currentTime = 0;
  private cameraMode: CameraMode;
  private trackedPlayerId: string | null;
  private ballCamEnabled: boolean;
  private readonly desiredCameraPosition = new THREE.Vector3();
  private readonly desiredLookTarget = new THREE.Vector3();
  private boundResize = () => this.sceneState.resize();

  constructor(
    container: HTMLElement,
    replay: ReplayModel,
    options: ReplayPlayerOptions = {}
  ) {
    super();
    this.container = container;
    this.replay = replay;
    this.options = options;
    this.sceneState = createReplayScene(container, replay);
    this.cameraMode = options.initialCameraMode ?? "overview";
    this.trackedPlayerId =
      options.initialTrackedPlayerId ?? replay.players[0]?.id ?? null;
    this.ballCamEnabled = options.initialBallCamEnabled ?? false;

    window.addEventListener("resize", this.boundResize);
    this.render();

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
    this.tick();
    this.emitChange();
  }

  pause(): void {
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

  setCameraMode(mode: CameraMode): void {
    this.cameraMode = mode;
    this.render();
    this.emitChange();
  }

  setTrackedPlayer(playerId: string): void {
    this.trackedPlayerId = playerId;
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

  getSnapshot(): ReplayPlayerSnapshot {
    return {
      currentTime: this.currentTime,
      duration: this.replay.duration,
      frameIndex: findFrameIndexAtTime(this.replay, this.currentTime),
      playing: this.playing,
      speed: this.speed,
      cameraMode: this.cameraMode,
      trackedPlayerId: this.trackedPlayerId,
      ballCamEnabled: this.ballCamEnabled,
    };
  }

  dispose(): void {
    this.pause();
    window.removeEventListener("resize", this.boundResize);
    this.sceneState.dispose();
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
      this.pause();
    }

    this.render();
    this.emitChange();
    this.animationFrameId = requestAnimationFrame(this.tick);
  };

  private render(): void {
    const frameIndex = findFrameIndexAtTime(this.replay, this.currentTime);
    const ballFrame = this.replay.ballFrames[frameIndex];
    const ballPosition = ballFrame?.position
      ? new THREE.Vector3(
          ballFrame.position.x,
          ballFrame.position.z + BALL_RADIUS,
          ballFrame.position.y
        )
      : null;

    if (ballPosition) {
      this.sceneState.ballMesh.visible = true;
      this.sceneState.ballMesh.position.copy(ballPosition);
    } else {
      this.sceneState.ballMesh.visible = false;
    }

    for (const player of this.replay.players) {
      const mesh = this.sceneState.playerMeshes.get(player.id);
      if (!mesh) {
        continue;
      }

      const frame = player.frames[frameIndex];
      if (!frame?.position) {
        mesh.visible = false;
        continue;
      }

      mesh.visible = true;
      mesh.position.set(
        frame.position.x,
        frame.position.z + UP_OFFSET,
        frame.position.y
      );

      const orientation = this.getOrientationVectors(frame);
      if (orientation) {
        const basis = new THREE.Matrix4().makeBasis(
          orientation.right,
          orientation.up,
          orientation.forward
        );
        mesh.quaternion.setFromRotationMatrix(basis);
      }
    }

    this.updateCamera(frameIndex, ballPosition);
    this.sceneState.controls.update();
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

    if (this.cameraMode === "overview" || !this.trackedPlayerId) {
      controls.enabled = true;
      this.sceneState.camera.fov = 48;
      this.sceneState.camera.updateProjectionMatrix();
      return;
    }

    const trackedPlayer = this.replay.players.find(
      (player) => player.id === this.trackedPlayerId
    );
    const trackedMesh = this.sceneState.playerMeshes.get(this.trackedPlayerId);
    const frame = trackedPlayer?.frames[frameIndex];

    if (!trackedPlayer || !trackedMesh || !frame?.position) {
      controls.enabled = true;
      return;
    }

    controls.enabled = false;

    const basePosition = new THREE.Vector3(
      frame.position.x,
      frame.position.z + UP_OFFSET,
      frame.position.y
    );
    const orientation = this.getOrientationVectors(frame);
    const forward = orientation?.forward ?? DEFAULT_FORWARD.clone();
    const right = orientation?.right ?? new THREE.Vector3(1, 0, 0);

    const cameraSettings = trackedPlayer.cameraSettings;
    const distance =
      (cameraSettings.distance ?? 270) *
      CAMERA_DISTANCE_SCALE *
      (this.cameraMode === "attached"
        ? ATTACHED_DISTANCE_MULTIPLIER
        : THIRD_PERSON_DISTANCE_MULTIPLIER);
    const height =
      (cameraSettings.height ?? 100) *
      CAMERA_HEIGHT_SCALE *
      (this.cameraMode === "attached"
        ? ATTACHED_HEIGHT_MULTIPLIER
        : THIRD_PERSON_HEIGHT_MULTIPLIER);
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
      .addScaledVector(DEFAULT_UP, Math.max(0.35, height * 0.35));
    let targetFov = cameraSettings.fov ?? 110;

    if (this.ballCamEnabled && ballPosition) {
      const ballFocusPoint = ballPosition
        .clone()
        .addScaledVector(DEFAULT_UP, BALL_CAM_HEIGHT_BIAS);
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
      this.desiredCameraPosition.y = Math.max(
        MIN_CAMERA_HEIGHT,
        this.desiredCameraPosition.y
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
          Math.max(
            targetFov,
            THREE.MathUtils.radToDeg(separationAngle) * 1.7
          )
        );
      }
    } else {
      this.desiredCameraPosition
        .copy(chaseAnchor)
        .addScaledVector(forward, -distance);
      this.desiredCameraPosition.y = Math.max(
        MIN_CAMERA_HEIGHT,
        this.desiredCameraPosition.y
      );
      this.desiredLookTarget
        .copy(basePosition)
        .addScaledVector(lookDirection, distance + 8)
        .addScaledVector(DEFAULT_UP, 0.8);
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

  private getOrientationVectors(frame: ReplayModel["players"][number]["frames"][number]): {
    forward: THREE.Vector3;
    up: THREE.Vector3;
    right: THREE.Vector3;
  } | null {
    const velocity = frame.velocity
      ? new THREE.Vector3(frame.velocity.x, frame.velocity.y, frame.velocity.z)
      : null;
    const rawForward = frame.forward
      ? new THREE.Vector3(frame.forward.x, frame.forward.y, frame.forward.z)
      : null;
    const rawUp = frame.up
      ? new THREE.Vector3(frame.up.x, frame.up.y, frame.up.z)
      : null;

    const grounded = (frame.position?.z ?? Infinity) < GROUND_HEIGHT_THRESHOLD;

    if (grounded) {
      const forward = (rawForward ?? velocity ?? DEFAULT_FORWARD.clone())
        .clone()
        .setY(0);

      if (forward.lengthSq() < 0.0001) {
        return null;
      }

      forward.normalize();
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

  private emitChange(): void {
    this.dispatchEvent(
      new CustomEvent<ReplayPlayerSnapshot>("change", {
        detail: this.getSnapshot(),
      })
    );
  }
}
