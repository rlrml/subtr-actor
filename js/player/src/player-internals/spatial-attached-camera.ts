import * as THREE from "three";
import { CHASE_CAMERA_HEIGHT_MULTIPLIER, DEFAULT_FORWARD, DEFAULT_UP, MIN_CAMERA_HEIGHT_UU, PLAYER_FOCUS_HEIGHT_UU } from "./spatial-constants";
import type { AttachedCameraOptions } from "./spatial-attached-types";
import { updateBallCameraTargets } from "./spatial-ball-cam";
import { applyAttachedCameraView } from "./spatial-camera-apply";
import { enableFreeCamera } from "./spatial-free-camera";
import { getOrientationVectors } from "./spatial-orientation";
import { worldPosition } from "./spatial-vectors";

export function updateAttachedCamera(options: AttachedCameraOptions): void {
  const {
    cameraViewMode,
    attachedPlayerId,
    ballCamEnabled,
    ballPosition,
    cameraDistanceScale,
    customCameraSettings,
    desiredCameraPosition,
    desiredLookTarget,
    attachedPlayerUnavailable = false,
    fieldScale,
    frameIndex,
    replay,
    sceneState,
  } = options;
  const controls = sceneState.controls;

  if (cameraViewMode === "free" || !attachedPlayerId) {
    enableFreeCamera(sceneState);
    return;
  }

  const attachedPlayer = replay.players.find((player) => player.id === attachedPlayerId);
  const frame = attachedPlayer?.frames[frameIndex];

  if (
    !attachedPlayer ||
    attachedPlayerUnavailable ||
    !frame?.position ||
    frame.isPresent === false
  ) {
    controls.enabled = true;
    return;
  }

  controls.enabled = false;

  const basePosition = worldPosition(frame.position, fieldScale);
  const orientation = getOrientationVectors(frame);
  const forward = orientation?.forward ?? DEFAULT_FORWARD.clone();
  const right = orientation?.right ?? new THREE.Vector3(0, 1, 0);

  const cameraSettings = {
    ...attachedPlayer.cameraSettings,
    ...(customCameraSettings ?? {}),
  };
  const distance = (cameraSettings.distance ?? 270) * fieldScale * cameraDistanceScale;
  const height = (cameraSettings.height ?? 100) * fieldScale * CHASE_CAMERA_HEIGHT_MULTIPLIER;
  const pitch = THREE.MathUtils.degToRad(cameraSettings.pitch ?? -4);
  const lookDirection = forward.clone().applyAxisAngle(right, pitch).normalize();
  const chaseAnchor = basePosition.clone().addScaledVector(DEFAULT_UP, height);
  const followOffset = forward
    .clone()
    .multiplyScalar(-distance)
    .addScaledVector(DEFAULT_UP, height)
    .applyAxisAngle(right, pitch);
  const playerFocusPoint = basePosition
    .clone()
    .addScaledVector(DEFAULT_UP, PLAYER_FOCUS_HEIGHT_UU * fieldScale);
  let targetFov = cameraSettings.fov ?? 110;

  if (ballCamEnabled && ballPosition) {
    targetFov = updateBallCameraTargets({
      ballPosition,
      desiredCameraPosition,
      desiredLookTarget,
      distance,
      fieldScale,
      lookDirection,
      playerFocusPoint,
      chaseAnchor,
      targetFov,
    });
  } else {
    desiredCameraPosition.copy(playerFocusPoint).add(followOffset);
    desiredCameraPosition.z = Math.max(MIN_CAMERA_HEIGHT_UU * fieldScale, desiredCameraPosition.z);
    desiredLookTarget.copy(playerFocusPoint);
  }

  applyAttachedCameraView({
    sceneState,
    desiredCameraPosition,
    desiredLookTarget,
    targetFov,
  });
}
