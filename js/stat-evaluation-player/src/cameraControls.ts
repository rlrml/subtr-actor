import {
  DEFAULT_FLOATING_NAMEPLATE_LIFT_UU,
  type CameraSettings,
  type ReplayCameraViewMode,
  type ReplayFreeCameraPreset,
  type ReplayPlayerState,
  type ReplayPlayerTrack,
} from "@rlrml/player";
import type { StatsReplayPlayer } from "./statsReplayPlayer.ts";

const CAMERA_VIEW_MODES: ReplayCameraViewMode[] = ["free", "follow"];

/**
 * Ball-cam control modes:
 * - `"off"` / `"on"` force car cam / ball cam.
 * - `"player"` follows the attached player's recorded ball-cam toggle — their
 *   true in-game view — and is the default when following a player.
 */
export type BallCamMode = "off" | "on" | "player";

/** Map a player state onto a tri-state ball-cam mode for the UI. */
export function ballCamModeFromState(
  state: Pick<ReplayPlayerState, "ballCamEnabled" | "useReplayBallCam">,
): BallCamMode {
  if (state.useReplayBallCam ?? false) {
    return "player";
  }
  return state.ballCamEnabled ? "on" : "off";
}

/**
 * Resolve a persisted camera config to a ball-cam mode. Absent settings (and
 * legacy configs without `useReplayBallCam`) default to "player" so following a
 * player uses their recorded view unless a forced ball/car cam was saved.
 */
export function ballCamModeFromConfig(camera: {
  ballCam?: boolean;
  useReplayBallCam?: boolean;
}): BallCamMode {
  if (camera.useReplayBallCam) {
    return "player";
  }
  if (camera.ballCam === true) {
    return "on";
  }
  if (camera.ballCam === false) {
    return "off";
  }
  return "player";
}

export const DEFAULT_CUSTOM_CAMERA_SETTINGS: Required<CameraSettings> = {
  fov: 110,
  height: 100,
  pitch: -4,
  distance: 270,
  stiffness: 0,
  swivelSpeed: 1,
  transitionSpeed: 1,
};

export interface CameraControlsElements {
  readonly attachedPlayer: HTMLSelectElement;
  readonly cameraViewFreeButton: HTMLButtonElement;
  readonly cameraViewFollowButton: HTMLButtonElement;
  readonly cameraViewAutoPossession: HTMLInputElement;
  readonly cameraViewOverheadButton: HTMLButtonElement;
  readonly cameraViewSideButton: HTMLButtonElement;
  readonly usePlayerCameraSettings: HTMLInputElement;
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
  readonly ballCamOffButton: HTMLButtonElement;
  readonly ballCamOnButton: HTMLButtonElement;
  readonly ballCamPlayerButton: HTMLButtonElement;
  readonly nameplateLift: HTMLInputElement;
  readonly nameplateLiftReadout: HTMLElement;
  readonly cameraProfileReadout: HTMLElement;
  readonly cameraFovReadout: HTMLElement;
  readonly cameraHeightReadout: HTMLElement;
  readonly cameraPitchReadout: HTMLElement;
  readonly cameraBaseDistanceReadout: HTMLElement;
  readonly cameraStiffnessReadout: HTMLElement;
}

export interface CameraControlsOptions {
  readonly elements: CameraControlsElements;
  getReplayPlayer(): StatsReplayPlayer | null;
  requestConfigSync(): void;
  onAutoPossessionChange?(enabled: boolean): void;
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

  private ballCamModeValue: BallCamMode = "player";

  get ballCamMode(): BallCamMode {
    return this.ballCamModeValue;
  }

  private autoPossessionEnabledValue = false;

  get autoPossessionEnabled(): boolean {
    return this.autoPossessionEnabledValue;
  }

  /** Convert a tri-state mode to the player's `setBallCamEnabled` argument. */
  private static ballCamEnabledForMode(mode: BallCamMode): boolean | null {
    return mode === "player" ? null : mode === "on";
  }

  followPlayerWithReplayCamera(
    playerId: string,
    options: {
      ballCam?: BallCamMode;
      preserveAutoPossession?: boolean;
      requestConfigSync?: boolean;
      usePlayerCameraSettings?: boolean;
    } = {},
  ): void {
    const replayPlayer = this.options.getReplayPlayer();
    if (!replayPlayer) {
      return;
    }

    if (!options.preserveAutoPossession) {
      this.setAutoPossessionEnabled(false, { requestConfigSync: false });
    }
    replayPlayer.setAttachedPlayer(playerId);
    replayPlayer.setCameraViewMode("follow");
    if (options.usePlayerCameraSettings !== false) {
      replayPlayer.setCustomCameraSettings(null);
    }
    this.setBallCamMode(options.ballCam ?? "player");
    this.lastFreeCameraPreset = null;
    if (options.requestConfigSync !== false) {
      this.options.requestConfigSync();
    }
  }

  setAutoPossessionEnabled(
    enabled: boolean,
    options: { notify?: boolean; requestConfigSync?: boolean } = {},
  ): void {
    if (this.autoPossessionEnabledValue === enabled) {
      this.renderAutoPossessionButton();
      return;
    }

    this.autoPossessionEnabledValue = enabled;
    this.renderAutoPossessionButton();
    if (options.notify !== false) {
      this.options.onAutoPossessionChange?.(enabled);
    }
    if (options.requestConfigSync !== false) {
      this.options.requestConfigSync();
    }
  }

  private renderBallCamButtons(): void {
    const { ballCamOffButton, ballCamOnButton, ballCamPlayerButton } = this.options.elements;
    const buttons: ReadonlyArray<[BallCamMode, HTMLButtonElement]> = [
      ["off", ballCamOffButton],
      ["on", ballCamOnButton],
      ["player", ballCamPlayerButton],
    ];
    for (const [mode, button] of buttons) {
      const isActive = mode === this.ballCamModeValue;
      button.dataset.active = isActive ? "true" : "false";
      button.setAttribute("aria-pressed", isActive ? "true" : "false");
    }
  }

  private renderAutoPossessionButton(): void {
    this.options.elements.cameraViewAutoPossession.checked = this.autoPossessionEnabledValue;
  }

  private disableAutoPossessionForManualCameraControl(): void {
    this.setAutoPossessionEnabled(false, { requestConfigSync: false });
  }

  /** Apply a ball-cam mode to the player and reflect it in the button group. */
  private setBallCamMode(mode: BallCamMode): void {
    this.ballCamModeValue = mode;
    this.options
      .getReplayPlayer()
      ?.setBallCamEnabled(CameraControlsController.ballCamEnabledForMode(mode));
    this.renderBallCamButtons();
  }

  /** Current floating name-plate lift (Unreal units) from the slider. */
  get nameplateLiftUu(): number {
    const value = Number(this.options.elements.nameplateLift.value);
    return Number.isFinite(value) ? value : DEFAULT_FLOATING_NAMEPLATE_LIFT_UU;
  }

  /** Apply a persisted name-plate lift to the slider + readout (config load). */
  applyNameplateLiftUu(value: number | undefined): void {
    const { nameplateLift, nameplateLiftReadout } = this.options.elements;
    const lift = value ?? DEFAULT_FLOATING_NAMEPLATE_LIFT_UU;
    nameplateLift.value = `${lift}`;
    nameplateLiftReadout.textContent = formatSetting(lift, "", 0);
  }

  installEventListeners(signal: AbortSignal): void {
    const { elements } = this.options;
    elements.usePlayerCameraSettings.addEventListener(
      "change",
      () => {
        const usePlayerCameraSettings = elements.usePlayerCameraSettings.checked;
        elements.cameraSettingsControls.hidden = usePlayerCameraSettings;
        this.options
          .getReplayPlayer()
          ?.setCustomCameraSettings(
            usePlayerCameraSettings ? null : this.readCustomCameraSettings(),
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
        const replayPlayer = this.options.getReplayPlayer();
        const attachedPlayerId = elements.attachedPlayer.value || null;
        this.disableAutoPossessionForManualCameraControl();
        replayPlayer?.setAttachedPlayer(attachedPlayerId);
        if (attachedPlayerId) {
          replayPlayer?.setCustomCameraSettings(null);
          // Default to the player's own recorded view when attaching.
          this.setBallCamMode("player");
        }
        this.lastFreeCameraPreset = null;
        this.options.requestConfigSync();
      },
      { signal },
    );

    elements.cameraViewFreeButton.addEventListener(
      "click",
      () => {
        this.options.getReplayPlayer()?.setCameraViewMode("free");
        this.disableAutoPossessionForManualCameraControl();
        this.lastFreeCameraPreset = null;
        this.options.requestConfigSync();
      },
      { signal },
    );

    elements.cameraViewFollowButton.addEventListener(
      "click",
      () => {
        const replayPlayer = this.options.getReplayPlayer();
        this.disableAutoPossessionForManualCameraControl();
        replayPlayer?.setCameraViewMode("follow");
        if (replayPlayer?.getState().attachedPlayerId) {
          replayPlayer.setCustomCameraSettings(null);
          // Default to the player's own recorded view when following.
          this.setBallCamMode("player");
        }
        this.lastFreeCameraPreset = null;
        this.options.requestConfigSync();
      },
      { signal },
    );

    elements.cameraViewOverheadButton.addEventListener(
      "click",
      () => {
        this.options.getReplayPlayer()?.setFreeCameraPreset("overhead");
        this.disableAutoPossessionForManualCameraControl();
        this.lastFreeCameraPreset = "overhead";
        this.options.requestConfigSync();
      },
      { signal },
    );

    elements.cameraViewSideButton.addEventListener(
      "click",
      () => {
        this.options.getReplayPlayer()?.setFreeCameraPreset("side");
        this.disableAutoPossessionForManualCameraControl();
        this.lastFreeCameraPreset = "side";
        this.options.requestConfigSync();
      },
      { signal },
    );

    const ballCamButtons: ReadonlyArray<[BallCamMode, HTMLButtonElement]> = [
      ["off", elements.ballCamOffButton],
      ["on", elements.ballCamOnButton],
      ["player", elements.ballCamPlayerButton],
    ];
    for (const [mode, button] of ballCamButtons) {
      button.addEventListener(
        "click",
        () => {
          this.setBallCamMode(mode);
          this.options.requestConfigSync();
        },
        { signal },
      );
    }

    elements.cameraViewAutoPossession.addEventListener(
      "change",
      () => {
        this.setAutoPossessionEnabled(elements.cameraViewAutoPossession.checked);
      },
      { signal },
    );
    // The ballchasing overlay reads nameplateLiftUu live each frame, so changing
    // the slider takes effect without touching the player — just refresh the
    // readout and persist.
    elements.nameplateLift.addEventListener(
      "input",
      () => {
        elements.nameplateLiftReadout.textContent = formatSetting(this.nameplateLiftUu, "", 0);
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
    elements.usePlayerCameraSettings.checked = state.customCameraSettings === null;
    elements.cameraSettingsControls.hidden = elements.usePlayerCameraSettings.checked;
    this.syncCustomCameraSettingControls(
      state.customCameraSettings ?? this.getFallbackCameraSettings(),
    );
    this.ballCamModeValue = ballCamModeFromState(state);
    this.renderBallCamButtons();
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
    this.options.elements.usePlayerCameraSettings.disabled = !hasAttachedCamera;
    this.setCameraSettingControlsEnabled(hasAttachedCamera && state?.customCameraSettings !== null);
    this.options.elements.ballCamOffButton.disabled = !hasAttachedCamera;
    this.options.elements.ballCamOnButton.disabled = !hasAttachedCamera;
    this.options.elements.ballCamPlayerButton.disabled = !hasAttachedCamera;
    this.renderBallCamButtons();
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
    const { cameraViewAutoPossession } = this.options.elements;
    cameraViewOverheadButton.disabled = !hasReplay;
    cameraViewSideButton.disabled = !hasReplay;
    cameraViewAutoPossession.disabled = !hasReplay;
    cameraViewOverheadButton.dataset.active = "false";
    cameraViewSideButton.dataset.active = "false";
    this.renderAutoPossessionButton();
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

    const player = replayPlayer.replay.players.find(
      (candidate) => candidate.id === attachedPlayerId,
    );
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
    return DEFAULT_CUSTOM_CAMERA_SETTINGS;
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
    elements.cameraSettingsControls.hidden = elements.usePlayerCameraSettings.checked;
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
