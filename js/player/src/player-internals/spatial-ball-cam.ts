import * as THREE from "three";
import {
  BALL_CAM_DIRECTION_BLEND,
  BALL_CAM_HEIGHT_BIAS_UU,
  BALL_CAM_LOOK_BLEND,
  BALL_CAM_MAX_FOV,
  DEFAULT_UP,
  MIN_CAMERA_HEIGHT_UU,
} from "./spatial-constants";

export function updateBallCameraTargets(options: {
  ballPosition: THREE.Vector3;
  desiredCameraPosition: THREE.Vector3;
  desiredLookTarget: THREE.Vector3;
  distance: number;
  fieldScale: number;
  lookDirection: THREE.Vector3;
  playerFocusPoint: THREE.Vector3;
  chaseAnchor: THREE.Vector3;
  targetFov: number;
}): number {
  const {
    ballPosition,
    desiredCameraPosition,
    desiredLookTarget,
    distance,
    fieldScale,
    lookDirection,
    playerFocusPoint,
    chaseAnchor,
  } = options;
  const ballFocusPoint = ballPosition
    .clone()
    .addScaledVector(DEFAULT_UP, BALL_CAM_HEIGHT_BIAS_UU * fieldScale);
  const playerToBall = ballFocusPoint.clone().sub(playerFocusPoint);
  const ballCamDirection = (
    playerToBall.lengthSq() > 0.0001 ? playerToBall.normalize() : lookDirection.clone()
  )
    .multiplyScalar(BALL_CAM_DIRECTION_BLEND)
    .addScaledVector(lookDirection, 1 - BALL_CAM_DIRECTION_BLEND)
    .normalize();

  desiredLookTarget.copy(playerFocusPoint).lerp(ballFocusPoint, BALL_CAM_LOOK_BLEND);
  desiredCameraPosition.copy(chaseAnchor).addScaledVector(ballCamDirection, -distance);
  desiredCameraPosition.z = Math.max(MIN_CAMERA_HEIGHT_UU * fieldScale, desiredCameraPosition.z);
  const cameraToPlayer = playerFocusPoint.clone().sub(desiredCameraPosition);
  const cameraToBall = ballFocusPoint.clone().sub(desiredCameraPosition);
  if (cameraToPlayer.lengthSq() <= 0.0001 || cameraToBall.lengthSq() <= 0.0001) {
    return options.targetFov;
  }

  const separationAngle = cameraToPlayer.angleTo(cameraToBall);
  return Math.min(
    BALL_CAM_MAX_FOV,
    Math.max(options.targetFov, THREE.MathUtils.radToDeg(separationAngle) * 1.7),
  );
}
