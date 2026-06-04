import * as THREE from "three";
import type { Vector3 } from "three";
import { findFrameIndexAtTime } from "./replay-data";
import {
  isKickoffFrame,
  isLiveGameplayFrame,
  isPostGoalTransitionFrame,
} from "./player-internals/timeline";
import {
  interpolatePosition,
  interpolateQuaternion,
  rootPosition,
  worldPosition,
} from "./player-internals/spatial";
import type { DemoIndicator, ReplayScene } from "./scene";
import type {
  CameraSettings,
  ReplayCameraViewMode,
  ReplayModel,
  ReplayPlayerOptions,
  ReplayTimelineEvent,
  Vec3,
} from "./types";

const DEFAULT_CAMERA_DISTANCE_SCALE = 2.25;
const DEMO_INDICATOR_DURATION_SECONDS = 3.2;

export const DEFAULT_CAMERA_VIEW_MODE: ReplayCameraViewMode = "free";

export interface ReplayPlayerInitialSettings {
  speed: number;
  cameraDistanceScale: number;
  customCameraSettings: CameraSettings | null;
  attachedPlayerId: string | null;
  cameraViewMode: ReplayCameraViewMode;
  ballCamEnabled: boolean;
  boostMeterEnabled: boolean;
  boostPickupAnimationEnabled: boolean;
  hitboxWireframesEnabled: boolean;
  hitboxOnlyModeEnabled: boolean;
  skipPostGoalTransitionsEnabled: boolean;
  skipKickoffsEnabled: boolean;
}

export interface ReplayBallRenderResult {
  ballFrame: ReplayModel["ballFrames"][number] | null;
  nextBallFrame: ReplayModel["ballFrames"][number] | null;
  ballPosition: Vector3 | null;
}

function finiteSetting(value: number | undefined): number | undefined {
  return typeof value === "number" && Number.isFinite(value) ? value : undefined;
}

export function normalizeCustomCameraSettings(
  settings: CameraSettings | null | undefined,
): CameraSettings | null {
  if (!settings) {
    return null;
  }

  const normalized: CameraSettings = {};
  const fov = finiteSetting(settings.fov);
  const height = finiteSetting(settings.height);
  const pitch = finiteSetting(settings.pitch);
  const distance = finiteSetting(settings.distance);
  const stiffness = finiteSetting(settings.stiffness);
  const swivelSpeed = finiteSetting(settings.swivelSpeed);
  const transitionSpeed = finiteSetting(settings.transitionSpeed);
  if (fov !== undefined) normalized.fov = fov;
  if (height !== undefined) normalized.height = height;
  if (pitch !== undefined) normalized.pitch = pitch;
  if (distance !== undefined) normalized.distance = distance;
  if (stiffness !== undefined) normalized.stiffness = stiffness;
  if (swivelSpeed !== undefined) normalized.swivelSpeed = swivelSpeed;
  if (transitionSpeed !== undefined) {
    normalized.transitionSpeed = transitionSpeed;
  }
  return normalized;
}

export function resolveInitialPlayerSettings(
  options: ReplayPlayerOptions,
): ReplayPlayerInitialSettings {
  const attachedPlayerId = options.initialAttachedPlayerId ?? null;
  return {
    speed: Math.max(0.1, options.initialPlaybackRate ?? 1),
    cameraDistanceScale: Math.max(
      0.25,
      options.initialCameraDistanceScale ?? DEFAULT_CAMERA_DISTANCE_SCALE,
    ),
    customCameraSettings: normalizeCustomCameraSettings(options.initialCustomCameraSettings),
    attachedPlayerId,
    cameraViewMode:
      options.initialCameraViewMode ?? (attachedPlayerId ? "follow" : DEFAULT_CAMERA_VIEW_MODE),
    ballCamEnabled: options.initialBallCamEnabled ?? false,
    boostMeterEnabled: options.initialBoostMeterEnabled ?? false,
    boostPickupAnimationEnabled: options.initialBoostPickupAnimationEnabled ?? true,
    hitboxWireframesEnabled: options.initialHitboxWireframesEnabled ?? false,
    hitboxOnlyModeEnabled: options.initialHitboxOnlyModeEnabled ?? false,
    skipPostGoalTransitionsEnabled: options.initialSkipPostGoalTransitionsEnabled ?? true,
    skipKickoffsEnabled: options.initialSkipKickoffsEnabled ?? false,
  };
}

export function getKickoffSkipTargetTime(
  replay: ReplayModel,
  currentTime: number,
  liveGameState: number | null,
  kickoffGameState: number | null,
): number | null {
  const frameIndex = findFrameIndexAtTime(replay, currentTime);
  const frame = replay.frames[frameIndex];
  if (!frame || !isKickoffFrame(frame, kickoffGameState)) {
    return null;
  }

  const nextLiveFrame = replay.frames.find(
    (candidate, index) => index > frameIndex && isLiveGameplayFrame(candidate, liveGameState),
  );
  if (!nextLiveFrame || nextLiveFrame.time === currentTime) {
    return null;
  }

  return nextLiveFrame.time;
}

export function getPostGoalTransitionSkipTargetTime(
  replay: ReplayModel,
  currentTime: number,
  liveGameState: number | null,
  kickoffGameState: number | null,
): number | null {
  const frameIndex = findFrameIndexAtTime(replay, currentTime);
  const frame = replay.frames[frameIndex];
  if (
    !frame ||
    !isPostGoalTransitionFrame(replay, frame, frameIndex, liveGameState, kickoffGameState)
  ) {
    return null;
  }

  const nextFrame = replay.frames.find(
    (candidate, index) =>
      index > frameIndex &&
      !isPostGoalTransitionFrame(replay, candidate, index, liveGameState, kickoffGameState),
  );
  if (nextFrame) {
    return nextFrame.time === currentTime ? null : nextFrame.time;
  }

  let startIndex = frameIndex;
  while (
    startIndex > 0 &&
    isPostGoalTransitionFrame(
      replay,
      replay.frames[startIndex - 1],
      startIndex - 1,
      liveGameState,
      kickoffGameState,
    )
  ) {
    startIndex -= 1;
  }

  const transitionStartTime = replay.frames[startIndex]?.time;
  if (transitionStartTime === undefined || transitionStartTime === currentTime) {
    return null;
  }
  return transitionStartTime;
}

export function updateReplayBallRender({
  replay,
  sceneState,
  fieldScale,
  frameWindow,
}: {
  replay: ReplayModel;
  sceneState: ReplayScene;
  fieldScale: number;
  frameWindow: { frameIndex: number; nextFrameIndex: number; alpha: number };
}): ReplayBallRenderResult {
  const ballFrame = replay.ballFrames[frameWindow.frameIndex] ?? null;
  const nextBallFrame = replay.ballFrames[frameWindow.nextFrameIndex] ?? ballFrame;
  const interpolatedBallPosition = interpolatePosition(
    ballFrame?.position ?? null,
    nextBallFrame?.position ?? null,
    frameWindow.alpha,
  );
  const ballPosition = interpolatedBallPosition
    ? worldPosition(interpolatedBallPosition, fieldScale)
    : null;

  if (interpolatedBallPosition) {
    sceneState.ballMesh.visible = true;
    sceneState.ballMesh.position.copy(rootPosition(interpolatedBallPosition));
    const ballRotation = interpolateQuaternion(
      ballFrame?.rotation ?? null,
      nextBallFrame?.rotation ?? null,
      frameWindow.alpha,
    );
    if (ballRotation) {
      sceneState.ballMesh.quaternion.copy(ballRotation);
    } else {
      sceneState.ballMesh.quaternion.identity();
    }
  } else {
    sceneState.ballMesh.visible = false;
  }

  return { ballFrame, nextBallFrame, ballPosition };
}

export function isPlayerSamplePresent(
  sample: ReplayModel["players"][number]["frames"][number] | null | undefined,
): boolean {
  return Boolean(sample?.position) && sample?.isPresent !== false;
}

export function getActiveDemoEvent(
  timelineEvents: ReplayTimelineEvent[],
  victimPlayerId: string,
  currentTime: number,
): ReplayTimelineEvent | null {
  for (let index = timelineEvents.length - 1; index >= 0; index -= 1) {
    const event = timelineEvents[index]!;
    const age = currentTime - event.time;
    if (age < 0) {
      continue;
    }
    if (age > DEMO_INDICATOR_DURATION_SECONDS) {
      break;
    }
    if (event.kind === "demo" && event.secondaryPlayerId === victimPlayerId) {
      return event;
    }
  }
  return null;
}

export function updateDemoIndicator({
  indicator,
  fallbackPosition,
  demoEvent,
  currentTime,
  camera,
}: {
  indicator: DemoIndicator | null;
  fallbackPosition: Vec3 | null;
  demoEvent: ReplayTimelineEvent | null;
  currentTime: number;
  camera: THREE.Camera;
}): void {
  if (!indicator) {
    return;
  }

  const position = demoEvent?.location ?? fallbackPosition;
  if (!demoEvent || !position) {
    indicator.group.visible = false;
    return;
  }

  const age = Math.max(0, currentTime - demoEvent.time);
  const phase = currentTime * 8;
  const pulse = 1 + 0.08 * Math.sin(phase);
  indicator.group.visible = true;
  indicator.group.position.copy(rootPosition(position));
  indicator.ring.rotation.z = phase * 0.15;
  indicator.ring.scale.setScalar(pulse);
  indicator.label.quaternion.copy(camera.quaternion);
  indicator.label.scale.setScalar(1 + 0.04 * Math.sin(phase + 1.3));

  const opacity = THREE.MathUtils.clamp(1 - age / DEMO_INDICATOR_DURATION_SECONDS, 0.28, 1);
  for (const node of [indicator.ring, indicator.label]) {
    const material = node.material;
    if (material instanceof THREE.Material) {
      material.opacity = opacity;
    }
  }
}

export function updateBoostTrail(
  boostTrail: THREE.Group,
  boostActive: boolean,
  boostFraction: number,
  time: number,
  playerIndex: number,
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
