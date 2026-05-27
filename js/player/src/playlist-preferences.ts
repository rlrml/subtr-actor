import type {
  CameraSettings,
  ReplayCameraViewMode,
  ReplayPlaylistPlayerOptions,
} from "./types";

const DEFAULT_CAMERA_DISTANCE_SCALE = 2.25;
const DEFAULT_PLAYBACK_RATE = 1;

export type PlayerPreferences = {
  speed: number;
  cameraDistanceScale: number;
  customCameraSettings: CameraSettings | null;
  cameraViewMode: ReplayCameraViewMode;
  attachedPlayerId: string | null;
  ballCamEnabled: boolean;
  boostPickupAnimationEnabled: boolean;
  skipPostGoalTransitionsEnabled: boolean;
  skipKickoffsEnabled: boolean;
};

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

export function createInitialPreferences(
  options: ReplayPlaylistPlayerOptions,
): PlayerPreferences {
  return {
    speed: Math.max(0.1, options.initialPlaybackRate ?? DEFAULT_PLAYBACK_RATE),
    cameraDistanceScale: Math.max(
      0.25,
      options.initialCameraDistanceScale ?? DEFAULT_CAMERA_DISTANCE_SCALE,
    ),
    customCameraSettings: normalizeCustomCameraSettings(options.initialCustomCameraSettings),
    cameraViewMode:
      options.initialCameraViewMode ?? (options.initialAttachedPlayerId ? "follow" : "free"),
    attachedPlayerId: options.initialAttachedPlayerId ?? null,
    ballCamEnabled: options.initialBallCamEnabled ?? false,
    boostPickupAnimationEnabled: options.initialBoostPickupAnimationEnabled ?? true,
    skipPostGoalTransitionsEnabled: options.initialSkipPostGoalTransitionsEnabled ?? true,
    skipKickoffsEnabled: options.initialSkipKickoffsEnabled ?? false,
  };
}
