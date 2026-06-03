import type {
  CameraSettings,
  ReplayCameraViewMode,
  ReplayFreeCameraPreset,
  ReplayPlayer,
  ReplayPlayerState,
  ReplayPlayerTrack,
} from "@rlrml/player";

const CAMERA_VIEW_MODES: ReplayCameraViewMode[] = ["free", "follow"];

export interface CameraControlsElements {
  readonly attachedPlayer: HTMLSelectElement;
  readonly cameraViewFreeButton: HTMLButtonElement;
  readonly cameraViewFollowButton: HTMLButtonElement;
  readonly cameraViewOverheadButton: HTMLButtonElement;
  readonly cameraViewSideButton: HTMLButtonElement;
  readonly cameraDistance: HTMLInputElement;
  readonly cameraDistanceReadout: HTMLElement;
  readonly customCameraSettings: HTMLInputElement;
  readonly cameraSettingsControls: HTMLDivElement;
  readonly customCameraFov: HTMLInputElement;
  readonly customCameraHeight: HTMLInputElement;
  readonly customCameraPitch: HTMLInputElement;
  readonly customCameraDistance: HTMLInputElement;
  readonly customCameraStiffness: HTMLInputElement;
  readonly customCameraSwivelSpeed: HTMLInputElement;
  readonly customCameraTransitionSpeed: HTMLInputElement;
  readonly customCameraFovReadout: HTMLElement;
  readonly customCameraHeightReadout: HTMLElement;
  readonly customCameraPitchReadout: HTMLElement;
  readonly customCameraDistanceReadout: HTMLElement;
  readonly customCameraStiffnessReadout: HTMLElement;
  readonly customCameraSwivelSpeedReadout: HTMLElement;
  readonly customCameraTransitionSpeedReadout: HTMLElement;
  readonly ballCam: HTMLInputElement;
  readonly cameraProfileReadout: HTMLElement;
  readonly cameraFovReadout: HTMLElement;
  readonly cameraHeightReadout: HTMLElement;
  readonly cameraPitchReadout: HTMLElement;
  readonly cameraBaseDistanceReadout: HTMLElement;
  readonly cameraStiffnessReadout: HTMLElement;
}

export interface CameraControlsOptions {
  readonly elements: CameraControlsElements;
  getReplayPlayer(): ReplayPlayer | null;
  requestConfigSync(): void;
}

export class CameraControlsController {
  private lastFreeCameraPreset: ReplayFreeCameraPreset | null = null;

  constructor(private readonly options: CameraControlsOptions) {}

  get freeCameraPreset(): ReplayFreeCameraPreset | null {
    return this.lastFreeCameraPreset;
  }

  set freeCameraPreset(value: ReplayFreeCameraPreset | null) {
    this.lastFreeCameraPreset = value;
  }

  get ballCamChecked(): boolean {
    return this.options.elements.ballCam.checked;
  }

  installEventListeners(signal: AbortSignal): void {
    const { elements } = this.options;
    elements.cameraDistance.addEventListener(
      "input",
      () => {
        this.options
          .getReplayPlayer()
          ?.setCameraDistanceScale(Number(elements.cameraDistance.value));
        this.options.requestConfigSync();
      },
      { signal },
    );

    elements.customCameraSettings.addEventListener(
      "change",
      () => {
        elements.cameraSettingsControls.hidden = !elements.customCameraSettings.checked;
        this.options
          .getReplayPlayer()
          ?.setCustomCameraSettings(
            elements.customCameraSettings.checked ? this.readCustomCameraSettings() : null,
          );
        this.options.requestConfigSync();
      },
      { signal },
    );

    for (const input of [
      elements.customCameraFov,
      elements.customCameraHeight,
      elements.customCameraPitch,
      elements.customCameraDistance,
      elements.customCameraStiffness,
      elements.customCameraSwivelSpeed,
      elements.customCameraTransitionSpeed,
    ]) {
      input.addEventListener(
        "input",
        () => {
          const settings = this.readCustomCameraSettings();
          this.syncCustomCameraSettingControls(settings);
          this.options.getReplayPlayer()?.setCustomCameraSettings(settings);
          this.options.requestConfigSync();
        },
        { signal },
      );
    }

    elements.attachedPlayer.addEventListener(
      "change",
      () => {
        this.options.getReplayPlayer()?.setAttachedPlayer(elements.attachedPlayer.value || null);
        this.lastFreeCameraPreset = null;
        this.options.requestConfigSync();
      },
      { signal },
    );

    elements.cameraViewFreeButton.addEventListener(
      "click",
      () => {
        this.options.getReplayPlayer()?.setCameraViewMode("free");
        this.lastFreeCameraPreset = null;
        this.options.requestConfigSync();
      },
      { signal },
    );

    elements.cameraViewFollowButton.addEventListener(
      "click",
      () => {
        this.options.getReplayPlayer()?.setCameraViewMode("follow");
        this.lastFreeCameraPreset = null;
        this.options.requestConfigSync();
      },
      { signal },
    );

    elements.cameraViewOverheadButton.addEventListener(
      "click",
      () => {
        this.options.getReplayPlayer()?.setFreeCameraPreset("overhead");
        this.lastFreeCameraPreset = "overhead";
        this.options.requestConfigSync();
      },
      { signal },
    );

    elements.cameraViewSideButton.addEventListener(
      "click",
      () => {
        this.options.getReplayPlayer()?.setFreeCameraPreset("side");
        this.lastFreeCameraPreset = "side";
        this.options.requestConfigSync();
      },
      { signal },
    );

    elements.ballCam.addEventListener(
      "change",
      () => {
        this.options.getReplayPlayer()?.setBallCamEnabled(elements.ballCam.checked);
        this.options.requestConfigSync();
      },
      { signal },
    );
  }

  setTransportEnabled(enabled: boolean, state?: ReplayPlayerState): void {
    this.options.elements.attachedPlayer.disabled = !enabled;
    this.syncModeButtons(enabled ? state : undefined);
  }

  syncState(state: ReplayPlayerState): void {
    const { elements } = this.options;
    elements.cameraDistance.value = `${state.cameraDistanceScale}`;
    elements.cameraDistanceReadout.textContent = `${state.cameraDistanceScale.toFixed(2)}x`;
    elements.customCameraSettings.checked = state.customCameraSettings !== null;
    elements.cameraSettingsControls.hidden = !elements.customCameraSettings.checked;
    this.syncCustomCameraSettingControls(this.getEffectiveCameraSettings(state));
    elements.ballCam.checked = state.ballCamEnabled;
    elements.attachedPlayer.value = state.attachedPlayerId ?? "";
    this.syncAvailability(state);
    this.renderProfile(state);
  }

  syncAvailability(state?: ReplayPlayerState): void {
    this.syncModeButtons(state);
    const replayPlayer = this.options.getReplayPlayer();
    const hasAttachedCamera =
      replayPlayer !== null &&
      state?.cameraViewMode === "follow" &&
      (state.attachedPlayerId ?? null) !== null;
    this.options.elements.cameraDistance.disabled = !hasAttachedCamera;
    this.options.elements.customCameraSettings.disabled = !hasAttachedCamera;
    this.setCameraSettingControlsEnabled(hasAttachedCamera && state?.customCameraSettings !== null);
    this.options.elements.ballCam.disabled = !hasAttachedCamera;
  }

  syncModeButtons(state?: ReplayPlayerState): void {
    const activeMode = state?.cameraViewMode ?? "free";
    const hasReplay = this.options.getReplayPlayer() !== null && state !== undefined;
    const canFollow = (state?.attachedPlayerId ?? null) !== null;

    for (const mode of CAMERA_VIEW_MODES) {
      const button = this.getCameraViewButton(mode);
      button.disabled = !hasReplay || (mode === "follow" && !canFollow);
      const isActive = mode === activeMode;
      button.dataset.active = isActive ? "true" : "false";
      button.setAttribute("aria-pressed", isActive ? "true" : "false");
    }

    const { cameraViewOverheadButton, cameraViewSideButton } = this.options.elements;
    cameraViewOverheadButton.disabled = !hasReplay;
    cameraViewSideButton.disabled = !hasReplay;
    cameraViewOverheadButton.dataset.active = "false";
    cameraViewSideButton.dataset.active = "false";
    cameraViewOverheadButton.setAttribute("aria-pressed", "false");
    cameraViewSideButton.setAttribute("aria-pressed", "false");
  }

  populateAttachedPlayerOptions(players: ReplayPlayerTrack[]): void {
    const { attachedPlayer } = this.options.elements;
    attachedPlayer.replaceChildren();
    attachedPlayer.append(new Option("Free camera", ""));

    for (const player of players) {
      attachedPlayer.append(
        new Option(`${player.name} (${player.isTeamZero ? "Blue" : "Orange"})`, player.id),
      );
    }
  }

  renderProfile(state?: ReplayPlayerState): void {
    const elements = this.options.elements;
    const replayPlayer = this.options.getReplayPlayer();
    const attachedPlayerId = state?.attachedPlayerId ?? null;
    if (!replayPlayer || state?.cameraViewMode !== "follow" || attachedPlayerId === null) {
      this.renderEmptyProfile("Free camera");
      return;
    }

    const player = replayPlayer.replay.players.find((candidate) => candidate.id === attachedPlayerId);
    if (!player) {
      this.renderEmptyProfile("Unknown");
      return;
    }

    const cameraSettings = this.getEffectiveCameraSettings(state);
    elements.cameraProfileReadout.textContent =
      state.customCameraSettings === null ? player.name : `${player.name} custom`;
    elements.cameraFovReadout.textContent = formatSetting(cameraSettings.fov, "", 0);
    elements.cameraHeightReadout.textContent = formatSetting(cameraSettings.height, "", 0);
    elements.cameraPitchReadout.textContent = formatSetting(cameraSettings.pitch, "", 0);
    elements.cameraBaseDistanceReadout.textContent = formatSetting(cameraSettings.distance, "", 0);
    elements.cameraStiffnessReadout.textContent = formatSetting(cameraSettings.stiffness, "", 2);
  }

  private getFallbackCameraSettings(): Required<CameraSettings> {
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

  private getAttachedPlayerCameraSettings(attachedPlayerId: string | null): CameraSettings | null {
    const replayPlayer = this.options.getReplayPlayer();
    if (!replayPlayer || attachedPlayerId === null) {
      return null;
    }

    return (
      replayPlayer.replay.players.find((candidate) => candidate.id === attachedPlayerId)
        ?.cameraSettings ?? null
    );
  }

  private getEffectiveCameraSettings(state: ReplayPlayerState): CameraSettings {
    return {
      ...this.getFallbackCameraSettings(),
      ...(this.getAttachedPlayerCameraSettings(state.attachedPlayerId) ?? {}),
      ...(state.customCameraSettings ?? {}),
    };
  }

  private readCustomCameraSettings(): CameraSettings {
    const elements = this.options.elements;
    return {
      fov: Number(elements.customCameraFov.value),
      height: Number(elements.customCameraHeight.value),
      pitch: Number(elements.customCameraPitch.value),
      distance: Number(elements.customCameraDistance.value),
      stiffness: Number(elements.customCameraStiffness.value),
      swivelSpeed: Number(elements.customCameraSwivelSpeed.value),
      transitionSpeed: Number(elements.customCameraTransitionSpeed.value),
    };
  }

  private setCameraSettingControlsEnabled(enabled: boolean): void {
    const elements = this.options.elements;
    elements.cameraSettingsControls.hidden = !elements.customCameraSettings.checked;
    elements.customCameraFov.disabled = !enabled;
    elements.customCameraHeight.disabled = !enabled;
    elements.customCameraPitch.disabled = !enabled;
    elements.customCameraDistance.disabled = !enabled;
    elements.customCameraStiffness.disabled = !enabled;
    elements.customCameraSwivelSpeed.disabled = !enabled;
    elements.customCameraTransitionSpeed.disabled = !enabled;
  }

  private syncCustomCameraSettingControls(settings: CameraSettings): void {
    const elements = this.options.elements;
    const fallback = this.getFallbackCameraSettings();
    const fov = settings.fov ?? fallback.fov;
    const height = settings.height ?? fallback.height;
    const pitch = settings.pitch ?? fallback.pitch;
    const distance = settings.distance ?? fallback.distance;
    const stiffness = settings.stiffness ?? fallback.stiffness;
    const swivelSpeed = settings.swivelSpeed ?? fallback.swivelSpeed;
    const transitionSpeed = settings.transitionSpeed ?? fallback.transitionSpeed;

    elements.customCameraFov.value = `${fov}`;
    elements.customCameraHeight.value = `${height}`;
    elements.customCameraPitch.value = `${pitch}`;
    elements.customCameraDistance.value = `${distance}`;
    elements.customCameraStiffness.value = `${stiffness}`;
    elements.customCameraSwivelSpeed.value = `${swivelSpeed}`;
    elements.customCameraTransitionSpeed.value = `${transitionSpeed}`;

    elements.customCameraFovReadout.textContent = formatSetting(fov, "", 0);
    elements.customCameraHeightReadout.textContent = formatSetting(height, "", 0);
    elements.customCameraPitchReadout.textContent = formatSetting(pitch, "", 0);
    elements.customCameraDistanceReadout.textContent = formatSetting(distance, "", 0);
    elements.customCameraStiffnessReadout.textContent = formatSetting(stiffness, "", 2);
    elements.customCameraSwivelSpeedReadout.textContent = formatSetting(swivelSpeed, "", 1);
    elements.customCameraTransitionSpeedReadout.textContent = formatSetting(transitionSpeed, "", 2);
  }

  private getCameraViewButton(mode: ReplayCameraViewMode): HTMLButtonElement {
    switch (mode) {
      case "free":
        return this.options.elements.cameraViewFreeButton;
      case "follow":
        return this.options.elements.cameraViewFollowButton;
    }
  }

  private renderEmptyProfile(label: string): void {
    const elements = this.options.elements;
    elements.cameraProfileReadout.textContent = label;
    elements.cameraFovReadout.textContent = "--";
    elements.cameraHeightReadout.textContent = "--";
    elements.cameraPitchReadout.textContent = "--";
    elements.cameraBaseDistanceReadout.textContent = "--";
    elements.cameraStiffnessReadout.textContent = "--";
  }
}

function formatSetting(value: number | undefined, suffix = "", digits = 0): string {
  if (value === undefined || Number.isNaN(value)) {
    return "--";
  }

  return `${value.toFixed(digits)}${suffix}`;
}

export function createCameraControlsController(
  options: CameraControlsOptions,
): CameraControlsController {
  return new CameraControlsController(options);
}
