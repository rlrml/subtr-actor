import type {
  CameraSettings,
  ReplayCameraViewMode,
  ReplayFreeCameraPreset,
  ReplayPlayer,
  ReplayPlayerState,
  ReplayPlayerTrack,
} from "@rlrml/subtr-actor-player";
import {
  formatSetting,
  getEffectiveCameraSettings as mergeEffectiveCameraSettings,
  populateAttachedPlayerOptions,
  readCustomCameraSettings,
  syncCustomCameraSettingControls,
  type CameraSettingElements,
} from "./cameraControlHelpers.ts";
import { mustElement } from "./floatingWindows.ts";
import type { PlayerCameraConfig } from "./playerConfig.ts";

const CAMERA_VIEW_MODES: ReplayCameraViewMode[] = ["free", "follow"];

export interface CameraControlElements {
  attachedPlayer: HTMLSelectElement;
  viewFree: HTMLButtonElement;
  viewFollow: HTMLButtonElement;
  viewOverhead: HTMLButtonElement;
  viewSide: HTMLButtonElement;
  distance: HTMLInputElement;
  distanceReadout: HTMLElement;
  customSettings: HTMLInputElement;
  settingsControls: HTMLDivElement;
  customFov: HTMLInputElement;
  customHeight: HTMLInputElement;
  customPitch: HTMLInputElement;
  customDistance: HTMLInputElement;
  customStiffness: HTMLInputElement;
  customSwivelSpeed: HTMLInputElement;
  customTransitionSpeed: HTMLInputElement;
  customFovReadout: HTMLElement;
  customHeightReadout: HTMLElement;
  customPitchReadout: HTMLElement;
  customDistanceReadout: HTMLElement;
  customStiffnessReadout: HTMLElement;
  customSwivelSpeedReadout: HTMLElement;
  customTransitionSpeedReadout: HTMLElement;
  ballCam: HTMLInputElement;
  profileReadout: HTMLElement;
  fovReadout: HTMLElement;
  heightReadout: HTMLElement;
  pitchReadout: HTMLElement;
  baseDistanceReadout: HTMLElement;
  stiffnessReadout: HTMLElement;
}

export interface CameraControlsOptions {
  elements: CameraControlElements;
  getReplayPlayer: () => ReplayPlayer | null;
  scheduleConfigUrlUpdate: () => void;
}

export interface CameraControls {
  applyReplayConfig(config: PlayerCameraConfig): void;
  clearFreePreset(): void;
  getConfigSnapshot(state?: ReplayPlayerState): PlayerCameraConfig;
  getCustomSettings(): CameraSettings;
  installListeners(signal: AbortSignal): void;
  populateAttachedPlayers(players: ReplayPlayerTrack[]): void;
  setEnabled(enabled: boolean): void;
  syncAvailability(state?: ReplayPlayerState): void;
  syncSnapshot(state: ReplayPlayerState): void;
}

export function getCameraControlElements(root: ParentNode): CameraControlElements {
  return {
    attachedPlayer: mustElement<HTMLSelectElement>(root, "#attached-player"),
    viewFree: mustElement<HTMLButtonElement>(root, "#camera-view-free"),
    viewFollow: mustElement<HTMLButtonElement>(root, "#camera-view-follow"),
    viewOverhead: mustElement<HTMLButtonElement>(root, "#camera-view-overhead"),
    viewSide: mustElement<HTMLButtonElement>(root, "#camera-view-side"),
    distance: mustElement<HTMLInputElement>(root, "#camera-distance"),
    distanceReadout: mustElement<HTMLElement>(root, "#camera-distance-readout"),
    customSettings: mustElement<HTMLInputElement>(root, "#custom-camera-settings"),
    settingsControls: mustElement<HTMLDivElement>(root, "#camera-settings-controls"),
    customFov: mustElement<HTMLInputElement>(root, "#custom-camera-fov"),
    customHeight: mustElement<HTMLInputElement>(root, "#custom-camera-height"),
    customPitch: mustElement<HTMLInputElement>(root, "#custom-camera-pitch"),
    customDistance: mustElement<HTMLInputElement>(root, "#custom-camera-distance"),
    customStiffness: mustElement<HTMLInputElement>(root, "#custom-camera-stiffness"),
    customSwivelSpeed: mustElement<HTMLInputElement>(root, "#custom-camera-swivel-speed"),
    customTransitionSpeed: mustElement<HTMLInputElement>(
      root,
      "#custom-camera-transition-speed",
    ),
    customFovReadout: mustElement<HTMLElement>(root, "#custom-camera-fov-readout"),
    customHeightReadout: mustElement<HTMLElement>(root, "#custom-camera-height-readout"),
    customPitchReadout: mustElement<HTMLElement>(root, "#custom-camera-pitch-readout"),
    customDistanceReadout: mustElement<HTMLElement>(root, "#custom-camera-distance-readout"),
    customStiffnessReadout: mustElement<HTMLElement>(root, "#custom-camera-stiffness-readout"),
    customSwivelSpeedReadout: mustElement<HTMLElement>(
      root,
      "#custom-camera-swivel-speed-readout",
    ),
    customTransitionSpeedReadout: mustElement<HTMLElement>(
      root,
      "#custom-camera-transition-speed-readout",
    ),
    ballCam: mustElement<HTMLInputElement>(root, "#ball-cam"),
    profileReadout: mustElement<HTMLElement>(root, "#camera-profile-readout"),
    fovReadout: mustElement<HTMLElement>(root, "#camera-fov-readout"),
    heightReadout: mustElement<HTMLElement>(root, "#camera-height-readout"),
    pitchReadout: mustElement<HTMLElement>(root, "#camera-pitch-readout"),
    baseDistanceReadout: mustElement<HTMLElement>(root, "#camera-base-distance-readout"),
    stiffnessReadout: mustElement<HTMLElement>(root, "#camera-stiffness-readout"),
  };
}

export function createCameraControls(options: CameraControlsOptions): CameraControls {
  const { elements } = options;
  let lastFreePreset: ReplayFreeCameraPreset | null = null;

  function getSettingElements(): CameraSettingElements {
    return {
      fov: elements.customFov,
      height: elements.customHeight,
      pitch: elements.customPitch,
      distance: elements.customDistance,
      stiffness: elements.customStiffness,
      swivelSpeed: elements.customSwivelSpeed,
      transitionSpeed: elements.customTransitionSpeed,
      fovReadout: elements.customFovReadout,
      heightReadout: elements.customHeightReadout,
      pitchReadout: elements.customPitchReadout,
      distanceReadout: elements.customDistanceReadout,
      stiffnessReadout: elements.customStiffnessReadout,
      swivelSpeedReadout: elements.customSwivelSpeedReadout,
      transitionSpeedReadout: elements.customTransitionSpeedReadout,
    };
  }

  function getAttachedPlayerSettings(attachedPlayerId: string | null): CameraSettings | null {
    const replayPlayer = options.getReplayPlayer();
    if (!replayPlayer || attachedPlayerId === null) {
      return null;
    }

    return (
      replayPlayer.replay.players.find((candidate) => candidate.id === attachedPlayerId)
        ?.cameraSettings ?? null
    );
  }

  function getEffectiveSettings(state: ReplayPlayerState): CameraSettings {
    return mergeEffectiveCameraSettings(state, getAttachedPlayerSettings(state.attachedPlayerId));
  }

  function readCustomSettings(): CameraSettings {
    return readCustomCameraSettings(getSettingElements());
  }

  function syncCustomSettings(settings: CameraSettings): void {
    syncCustomCameraSettingControls(getSettingElements(), settings);
  }

  function setCameraSettingControlsEnabled(enabled: boolean): void {
    elements.settingsControls.hidden = !elements.customSettings.checked;
    elements.customFov.disabled = !enabled;
    elements.customHeight.disabled = !enabled;
    elements.customPitch.disabled = !enabled;
    elements.customDistance.disabled = !enabled;
    elements.customStiffness.disabled = !enabled;
    elements.customSwivelSpeed.disabled = !enabled;
    elements.customTransitionSpeed.disabled = !enabled;
  }

  function getCameraViewButton(mode: ReplayCameraViewMode): HTMLButtonElement {
    switch (mode) {
      case "free":
        return elements.viewFree;
      case "follow":
        return elements.viewFollow;
    }
  }

  function syncCameraModeButtons(state?: ReplayPlayerState): void {
    const activeMode = state?.cameraViewMode ?? "free";
    const hasReplay = options.getReplayPlayer() !== null && state !== undefined;
    const canFollow = (state?.attachedPlayerId ?? null) !== null;

    for (const mode of CAMERA_VIEW_MODES) {
      const button = getCameraViewButton(mode);
      button.disabled = !hasReplay || (mode === "follow" && !canFollow);
      const isActive = mode === activeMode;
      button.dataset.active = isActive ? "true" : "false";
      button.setAttribute("aria-pressed", isActive ? "true" : "false");
    }

    elements.viewOverhead.disabled = !hasReplay;
    elements.viewSide.disabled = !hasReplay;
    elements.viewOverhead.dataset.active = "false";
    elements.viewSide.dataset.active = "false";
    elements.viewOverhead.setAttribute("aria-pressed", "false");
    elements.viewSide.setAttribute("aria-pressed", "false");
  }

  function syncProfile(state?: ReplayPlayerState): void {
    const replayPlayer = options.getReplayPlayer();
    const attachedPlayerId = state?.attachedPlayerId ?? null;
    if (!replayPlayer || state?.cameraViewMode !== "follow" || attachedPlayerId === null) {
      elements.profileReadout.textContent = "Free camera";
      elements.fovReadout.textContent = "--";
      elements.heightReadout.textContent = "--";
      elements.pitchReadout.textContent = "--";
      elements.baseDistanceReadout.textContent = "--";
      elements.stiffnessReadout.textContent = "--";
      return;
    }

    const player = replayPlayer.replay.players.find((candidate) => candidate.id === attachedPlayerId);
    if (!player) {
      elements.profileReadout.textContent = "Unknown";
      elements.fovReadout.textContent = "--";
      elements.heightReadout.textContent = "--";
      elements.pitchReadout.textContent = "--";
      elements.baseDistanceReadout.textContent = "--";
      elements.stiffnessReadout.textContent = "--";
      return;
    }

    const cameraSettings = getEffectiveSettings(state);
    elements.profileReadout.textContent =
      state.customCameraSettings === null ? player.name : `${player.name} custom`;
    elements.fovReadout.textContent = formatSetting(cameraSettings.fov, "", 0);
    elements.heightReadout.textContent = formatSetting(cameraSettings.height, "", 0);
    elements.pitchReadout.textContent = formatSetting(cameraSettings.pitch, "", 0);
    elements.baseDistanceReadout.textContent = formatSetting(cameraSettings.distance, "", 0);
    elements.stiffnessReadout.textContent = formatSetting(cameraSettings.stiffness, "", 2);
  }

  function syncAvailability(state?: ReplayPlayerState): void {
    syncCameraModeButtons(state);
    const hasAttachedCamera =
      options.getReplayPlayer() !== null &&
      state?.cameraViewMode === "follow" &&
      (state.attachedPlayerId ?? null) !== null;
    elements.distance.disabled = !hasAttachedCamera;
    elements.customSettings.disabled = !hasAttachedCamera;
    setCameraSettingControlsEnabled(hasAttachedCamera && state?.customCameraSettings !== null);
    elements.ballCam.disabled = !hasAttachedCamera;
  }

  return {
    applyReplayConfig(config) {
      const replayPlayer = options.getReplayPlayer();
      lastFreePreset = config.freePreset ?? null;
      if (replayPlayer && config.mode === "free" && config.freePreset) {
        replayPlayer.setFreeCameraPreset(config.freePreset);
      }
    },
    clearFreePreset() {
      lastFreePreset = null;
    },
    getConfigSnapshot(state = options.getReplayPlayer()?.getState()) {
      return {
        mode: state?.cameraViewMode,
        freePreset: lastFreePreset,
        attachedPlayerId: state?.attachedPlayerId,
        distanceScale: state?.cameraDistanceScale,
        ballCam: state?.ballCamEnabled,
        customSettings: state?.customCameraSettings,
      };
    },
    getCustomSettings: readCustomSettings,
    installListeners(signal) {
      elements.distance.addEventListener(
        "input",
        () => {
          options.getReplayPlayer()?.setCameraDistanceScale(Number(elements.distance.value));
          options.scheduleConfigUrlUpdate();
        },
        { signal },
      );

      elements.customSettings.addEventListener(
        "change",
        () => {
          elements.settingsControls.hidden = !elements.customSettings.checked;
          options
            .getReplayPlayer()
            ?.setCustomCameraSettings(elements.customSettings.checked ? readCustomSettings() : null);
          options.scheduleConfigUrlUpdate();
        },
        { signal },
      );

      for (const input of [
        elements.customFov,
        elements.customHeight,
        elements.customPitch,
        elements.customDistance,
        elements.customStiffness,
        elements.customSwivelSpeed,
        elements.customTransitionSpeed,
      ]) {
        input.addEventListener(
          "input",
          () => {
            const settings = readCustomSettings();
            syncCustomSettings(settings);
            options.getReplayPlayer()?.setCustomCameraSettings(settings);
            options.scheduleConfigUrlUpdate();
          },
          { signal },
        );
      }

      elements.attachedPlayer.addEventListener(
        "change",
        () => {
          options.getReplayPlayer()?.setAttachedPlayer(elements.attachedPlayer.value || null);
          lastFreePreset = null;
          options.scheduleConfigUrlUpdate();
        },
        { signal },
      );

      elements.viewFree.addEventListener(
        "click",
        () => {
          options.getReplayPlayer()?.setCameraViewMode("free");
          lastFreePreset = null;
          options.scheduleConfigUrlUpdate();
        },
        { signal },
      );

      elements.viewFollow.addEventListener(
        "click",
        () => {
          options.getReplayPlayer()?.setCameraViewMode("follow");
          lastFreePreset = null;
          options.scheduleConfigUrlUpdate();
        },
        { signal },
      );

      elements.viewOverhead.addEventListener(
        "click",
        () => {
          options.getReplayPlayer()?.setFreeCameraPreset("overhead");
          lastFreePreset = "overhead";
          options.scheduleConfigUrlUpdate();
        },
        { signal },
      );

      elements.viewSide.addEventListener(
        "click",
        () => {
          options.getReplayPlayer()?.setFreeCameraPreset("side");
          lastFreePreset = "side";
          options.scheduleConfigUrlUpdate();
        },
        { signal },
      );

      elements.ballCam.addEventListener(
        "change",
        () => {
          options.getReplayPlayer()?.setBallCamEnabled(elements.ballCam.checked);
          options.scheduleConfigUrlUpdate();
        },
        { signal },
      );
    },
    populateAttachedPlayers(players) {
      populateAttachedPlayerOptions(elements.attachedPlayer, players);
    },
    setEnabled(enabled) {
      elements.attachedPlayer.disabled = !enabled;
      syncAvailability(enabled ? options.getReplayPlayer()?.getState() : undefined);
    },
    syncAvailability,
    syncSnapshot(state) {
      elements.distance.value = `${state.cameraDistanceScale}`;
      elements.distanceReadout.textContent = `${state.cameraDistanceScale.toFixed(2)}x`;
      elements.customSettings.checked = state.customCameraSettings !== null;
      elements.settingsControls.hidden = !elements.customSettings.checked;
      syncCustomSettings(getEffectiveSettings(state));
      elements.ballCam.checked = state.ballCamEnabled;
      elements.attachedPlayer.value = state.attachedPlayerId ?? "";
      syncAvailability(state);
      syncProfile(state);
    },
  };
}
