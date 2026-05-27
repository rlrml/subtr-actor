import type { CameraSettings } from "./types";

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
