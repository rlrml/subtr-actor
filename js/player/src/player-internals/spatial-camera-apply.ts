import * as THREE from "three";
import type { ReplayScene } from "../scene";
import { CAMERA_SMOOTHING, DEFAULT_UP } from "./spatial-constants";

export function applyAttachedCameraView(options: {
  sceneState: ReplayScene;
  desiredCameraPosition: THREE.Vector3;
  desiredLookTarget: THREE.Vector3;
  targetFov: number;
}): void {
  const { desiredCameraPosition, desiredLookTarget, sceneState, targetFov } = options;
  sceneState.camera.position.lerp(desiredCameraPosition, CAMERA_SMOOTHING);
  sceneState.camera.up.lerp(DEFAULT_UP, CAMERA_SMOOTHING).normalize();
  sceneState.controls.target.lerp(desiredLookTarget, CAMERA_SMOOTHING);
  sceneState.camera.fov = THREE.MathUtils.lerp(sceneState.camera.fov, targetFov, CAMERA_SMOOTHING);
  sceneState.camera.updateProjectionMatrix();
  sceneState.camera.lookAt(sceneState.controls.target);
}
