import type { CameraSettings, ReplayCameraViewMode, ReplayPlayerOptions } from "./types";

const DEFAULT_CAMERA_DISTANCE_SCALE = 2.25;
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
  skipPostGoalTransitionsEnabled: boolean;
  skipKickoffsEnabled: boolean;
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
    skipPostGoalTransitionsEnabled: options.initialSkipPostGoalTransitionsEnabled ?? true,
    skipKickoffsEnabled: options.initialSkipKickoffsEnabled ?? false,
  };
}
