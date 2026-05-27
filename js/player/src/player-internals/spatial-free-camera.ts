import * as THREE from "three";
import type { ReplayScene } from "../scene";
import type { ReplayFreeCameraPreset } from "../types";
import {
  CAMERA_SMOOTHING,
  CAMERA_FOV_EPSILON,
  CAMERA_POSITION_EPSILON_SQ,
  CAMERA_TARGET_EPSILON_SQ,
  CAMERA_UP_EPSILON_RAD,
  DEFAULT_UP,
  FREE_CAMERA_FOV,
  FREE_CAMERA_TRANSITION_SMOOTHING,
  OVERHEAD_CAMERA_POSITION_UU,
  OVERHEAD_LOOK_TARGET_UU,
  OVERHEAD_UP,
  SIDE_CAMERA_POSITION_UU,
  SIDE_LOOK_TARGET_UU,
} from "./spatial-constants";

export function getFreeCameraPreset(
  preset: ReplayFreeCameraPreset,
  fieldScale: number,
): {
  position: THREE.Vector3;
  target: THREE.Vector3;
  up: THREE.Vector3;
  fov: number;
} {
  switch (preset) {
    case "overhead":
      return {
        position: OVERHEAD_CAMERA_POSITION_UU.clone().multiplyScalar(fieldScale),
        target: OVERHEAD_LOOK_TARGET_UU.clone().multiplyScalar(fieldScale),
        up: OVERHEAD_UP.clone(),
        fov: FREE_CAMERA_FOV,
      };
    case "side":
      return {
        position: SIDE_CAMERA_POSITION_UU.clone().multiplyScalar(fieldScale),
        target: SIDE_LOOK_TARGET_UU.clone().multiplyScalar(fieldScale),
        up: DEFAULT_UP.clone(),
        fov: FREE_CAMERA_FOV,
      };
  }
}

export function updateFreeCameraTransition(options: {
  sceneState: ReplayScene;
  position: THREE.Vector3;
  target: THREE.Vector3;
  up: THREE.Vector3;
  fov: number;
}): boolean {
  const { fov, position, sceneState, target, up } = options;
  const { camera, controls } = sceneState;

  controls.enabled = false;
  camera.position.lerp(position, FREE_CAMERA_TRANSITION_SMOOTHING);
  controls.target.lerp(target, FREE_CAMERA_TRANSITION_SMOOTHING);
  camera.up.lerp(up, FREE_CAMERA_TRANSITION_SMOOTHING).normalize();
  camera.fov = THREE.MathUtils.lerp(camera.fov, fov, FREE_CAMERA_TRANSITION_SMOOTHING);
  camera.updateProjectionMatrix();
  camera.lookAt(controls.target);

  const reachedPosition = camera.position.distanceToSquared(position) <= CAMERA_POSITION_EPSILON_SQ;
  const reachedTarget = controls.target.distanceToSquared(target) <= CAMERA_TARGET_EPSILON_SQ;
  const reachedUp = camera.up.angleTo(up) <= CAMERA_UP_EPSILON_RAD;
  const reachedFov = Math.abs(camera.fov - fov) <= CAMERA_FOV_EPSILON;
  if (!reachedPosition || !reachedTarget || !reachedUp || !reachedFov) {
    return false;
  }

  camera.position.copy(position);
  controls.target.copy(target);
  camera.up.copy(up).normalize();
  camera.fov = fov;
  camera.updateProjectionMatrix();
  camera.lookAt(target);
  controls.enabled = true;
  return true;
}

export function enableFreeCamera(sceneState: ReplayScene): void {
  sceneState.controls.enabled = true;
  sceneState.camera.fov = THREE.MathUtils.lerp(
    sceneState.camera.fov,
    FREE_CAMERA_FOV,
    CAMERA_SMOOTHING,
  );
  sceneState.camera.updateProjectionMatrix();
}
