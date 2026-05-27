import type {
  CameraSettings,
  ReplayPlayerState,
  ReplayPlayerTrack,
} from "@rlrml/subtr-actor-player";

export interface CameraSettingElements {
  fov: HTMLInputElement;
  height: HTMLInputElement;
  pitch: HTMLInputElement;
  distance: HTMLInputElement;
  stiffness: HTMLInputElement;
  swivelSpeed: HTMLInputElement;
  transitionSpeed: HTMLInputElement;
  fovReadout: HTMLElement;
  heightReadout: HTMLElement;
  pitchReadout: HTMLElement;
  distanceReadout: HTMLElement;
  stiffnessReadout: HTMLElement;
  swivelSpeedReadout: HTMLElement;
  transitionSpeedReadout: HTMLElement;
}

export function formatSetting(value: number | undefined, suffix = "", digits = 0): string {
  if (value === undefined || Number.isNaN(value)) {
    return "--";
  }

  return `${value.toFixed(digits)}${suffix}`;
}

export function getFallbackCameraSettings(): Required<CameraSettings> {
  return {
    fov: 110,
    height: 100,
    pitch: -4,
    distance: 270,
    stiffness: 0,
    swivelSpeed: 1,
    transitionSpeed: 1,
  };
}

export function getEffectiveCameraSettings(
  state: ReplayPlayerState,
  attachedPlayerSettings: CameraSettings | null,
): CameraSettings {
  return {
    ...getFallbackCameraSettings(),
    ...(attachedPlayerSettings ?? {}),
    ...(state.customCameraSettings ?? {}),
  };
}

export function readCustomCameraSettings(elements: CameraSettingElements): CameraSettings {
  return {
    fov: Number(elements.fov.value),
    height: Number(elements.height.value),
    pitch: Number(elements.pitch.value),
    distance: Number(elements.distance.value),
    stiffness: Number(elements.stiffness.value),
    swivelSpeed: Number(elements.swivelSpeed.value),
    transitionSpeed: Number(elements.transitionSpeed.value),
  };
}

export function syncCustomCameraSettingControls(
  elements: CameraSettingElements,
  settings: CameraSettings,
): void {
  const fallback = getFallbackCameraSettings();
  const fov = settings.fov ?? fallback.fov;
  const height = settings.height ?? fallback.height;
  const pitch = settings.pitch ?? fallback.pitch;
  const distance = settings.distance ?? fallback.distance;
  const stiffness = settings.stiffness ?? fallback.stiffness;
  const swivelSpeed = settings.swivelSpeed ?? fallback.swivelSpeed;
  const transitionSpeed = settings.transitionSpeed ?? fallback.transitionSpeed;

  elements.fov.value = `${fov}`;
  elements.height.value = `${height}`;
  elements.pitch.value = `${pitch}`;
  elements.distance.value = `${distance}`;
  elements.stiffness.value = `${stiffness}`;
  elements.swivelSpeed.value = `${swivelSpeed}`;
  elements.transitionSpeed.value = `${transitionSpeed}`;

  elements.fovReadout.textContent = formatSetting(fov, "", 0);
  elements.heightReadout.textContent = formatSetting(height, "", 0);
  elements.pitchReadout.textContent = formatSetting(pitch, "", 0);
  elements.distanceReadout.textContent = formatSetting(distance, "", 0);
  elements.stiffnessReadout.textContent = formatSetting(stiffness, "", 2);
  elements.swivelSpeedReadout.textContent = formatSetting(swivelSpeed, "", 1);
  elements.transitionSpeedReadout.textContent = formatSetting(transitionSpeed, "", 2);
}

export function populateAttachedPlayerOptions(
  select: HTMLSelectElement,
  players: ReplayPlayerTrack[],
): void {
  select.replaceChildren();
  select.append(new Option("Free camera", ""));

  for (const player of players) {
    select.append(
      new Option(`${player.name} (${player.isTeamZero ? "Blue" : "Orange"})`, player.id),
    );
  }
}
