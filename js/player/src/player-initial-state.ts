import { normalizeCustomCameraSettings } from "./player-camera-settings";
import type {
  CameraSettings,
  ReplayCameraViewMode,
  ReplayPlayerOptions,
} from "./types";

const DEFAULT_FIELD_SCALE = 1;
const DEFAULT_CAMERA_DISTANCE_SCALE = 2.25;
export const DEFAULT_REPLAY_CAMERA_VIEW_MODE: ReplayCameraViewMode = "free";

export interface ReplayPlayerInitialState {
  fieldScale: number;
  speed: number;
  cameraDistanceScale: number;
  customCameraSettings: CameraSettings | null;
  attachedPlayerId: string | null;
  cameraViewMode: ReplayCameraViewMode;
  ballCamEnabled: boolean;
  boostMeterEnabled: boolean;
  boostPickupAnimationEnabled: boolean;
  skipPostGoalTransitionsEnabled: boolean;
  skipKickoffsEnabled: boolean;
}

export function getReplayPlayerInitialState(
  options: ReplayPlayerOptions,
): ReplayPlayerInitialState {
  const attachedPlayerId = options.initialAttachedPlayerId ?? null;
  return {
    fieldScale: options.fieldScale ?? DEFAULT_FIELD_SCALE,
    speed: Math.max(0.1, options.initialPlaybackRate ?? 1),
    cameraDistanceScale: Math.max(
      0.25,
      options.initialCameraDistanceScale ?? DEFAULT_CAMERA_DISTANCE_SCALE,
    ),
    customCameraSettings: normalizeCustomCameraSettings(options.initialCustomCameraSettings),
    attachedPlayerId,
    cameraViewMode:
      options.initialCameraViewMode ??
      (attachedPlayerId ? "follow" : DEFAULT_REPLAY_CAMERA_VIEW_MODE),
    ballCamEnabled: options.initialBallCamEnabled ?? false,
    boostMeterEnabled: options.initialBoostMeterEnabled ?? false,
    boostPickupAnimationEnabled: options.initialBoostPickupAnimationEnabled ?? true,
    skipPostGoalTransitionsEnabled: options.initialSkipPostGoalTransitionsEnabled ?? true,
    skipKickoffsEnabled: options.initialSkipKickoffsEnabled ?? false,
  };
}
