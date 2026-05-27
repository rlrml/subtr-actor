import type * as THREE from "three";
import type { ReplayScene } from "../scene";
import type { CameraSettings, ReplayCameraViewMode, ReplayModel } from "../types";

export interface AttachedCameraOptions {
  sceneState: ReplayScene;
  replay: ReplayModel;
  fieldScale: number;
  cameraViewMode: ReplayCameraViewMode;
  attachedPlayerId: string | null;
  ballCamEnabled: boolean;
  cameraDistanceScale: number;
  customCameraSettings: CameraSettings | null;
  frameIndex: number;
  attachedPlayerUnavailable?: boolean;
  ballPosition: THREE.Vector3 | null;
  desiredCameraPosition: THREE.Vector3;
  desiredLookTarget: THREE.Vector3;
}
