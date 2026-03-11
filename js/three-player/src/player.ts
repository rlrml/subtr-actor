import * as THREE from "three";
import { createReplayScene, type ReplayScene } from "./scene";
import { findFrameIndexAtTime } from "./replay-data";
import type {
  CameraMode,
  ReplayModel,
  ReplayPlayerOptions,
  ReplayPlayerSnapshot,
} from "./types";

const UP_OFFSET = 0.6;
const BALL_RADIUS = 1.82;
const ATTACHED_CAMERA_HEIGHT = 1.8;
const ATTACHED_CAMERA_DISTANCE = 4.6;
const THIRD_PERSON_CAMERA_HEIGHT = 4.2;
const THIRD_PERSON_CAMERA_DISTANCE = 10.5;
const CAMERA_SMOOTHING = 0.18;

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

      if (frame.velocity) {
        const heading = Math.atan2(frame.velocity.x, frame.velocity.y);
        mesh.rotation.set(0, -heading, 0);
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
    const velocity = frame.velocity
      ? new THREE.Vector3(frame.velocity.x, 0, frame.velocity.y)
      : new THREE.Vector3();

    const forward =
      velocity.lengthSq() > 0.0001
        ? velocity.normalize()
        : new THREE.Vector3(0, 0, 1).applyQuaternion(trackedMesh.quaternion);

    const height =
      this.cameraMode === "attached"
        ? ATTACHED_CAMERA_HEIGHT
        : THIRD_PERSON_CAMERA_HEIGHT;
    const distance =
      this.cameraMode === "attached"
        ? ATTACHED_CAMERA_DISTANCE
        : THIRD_PERSON_CAMERA_DISTANCE;

    this.desiredCameraPosition
      .copy(basePosition)
      .addScaledVector(forward, -distance)
      .add(new THREE.Vector3(0, height, 0));

    this.desiredLookTarget
      .copy(basePosition)
      .addScaledVector(forward, 8)
      .add(new THREE.Vector3(0, 1.1, 0));

    if (ballPosition) {
      this.desiredLookTarget.lerp(ballPosition, 0.35);
    }

    this.sceneState.camera.position.lerp(
      this.desiredCameraPosition,
      CAMERA_SMOOTHING
    );
    controls.target.lerp(this.desiredLookTarget, CAMERA_SMOOTHING);
    this.sceneState.camera.lookAt(controls.target);
  }

  private emitChange(): void {
    this.dispatchEvent(
      new CustomEvent<ReplayPlayerSnapshot>("change", {
        detail: this.getSnapshot(),
      })
    );
  }
}
