import type { ReplayPlayerState } from "@rlrml/player";
import type { CameraControlsController } from "./cameraControls.ts";

export interface PlaybackReadoutElements {
  readonly togglePlayback: HTMLButtonElement;
  readonly previousFrame: HTMLButtonElement;
  readonly nextFrame: HTMLButtonElement;
  readonly playbackRate: HTMLSelectElement;
  readonly skipPostGoalTransitions: HTMLInputElement;
  readonly skipKickoffs: HTMLInputElement;
  readonly hitboxWireframes: HTMLInputElement;
  readonly hitboxOnlyMode: HTMLInputElement;
  readonly emptyState: HTMLDivElement;
  readonly timeReadout: HTMLElement;
  readonly frameReadout: HTMLElement;
  readonly durationReadout: HTMLElement;
  readonly playbackStatusReadout: HTMLElement;
}

export interface PlaybackReadoutsOptions {
  readonly elements: PlaybackReadoutElements;
  getFrameCount(): number;
  getCameraControlsController(): CameraControlsController | null;
}

export class PlaybackReadoutsController {
  constructor(private readonly options: PlaybackReadoutsOptions) {}

  setTransportEnabled(enabled: boolean, state?: ReplayPlayerState): void {
    const { elements } = this.options;
    elements.togglePlayback.disabled = !enabled;
    elements.previousFrame.disabled = !enabled;
    elements.nextFrame.disabled = !enabled;
    elements.playbackRate.disabled = !enabled;
    elements.skipPostGoalTransitions.disabled = !enabled;
    elements.skipKickoffs.disabled = !enabled;
    elements.hitboxWireframes.disabled = !enabled;
    elements.hitboxOnlyMode.disabled = !enabled;
    this.options.getCameraControlsController()?.setTransportEnabled(enabled, state);
  }

  renderSnapshot(state: ReplayPlayerState): void {
    const { elements } = this.options;
    elements.timeReadout.textContent = `${state.currentTime.toFixed(2)}s`;
    elements.frameReadout.textContent = `${state.frameIndex}`;
    elements.durationReadout.textContent = `${state.duration.toFixed(2)}s`;
    elements.playbackStatusReadout.textContent = state.playing ? "Playing" : "Paused";
    elements.togglePlayback.textContent = state.playing ? "Pause" : "Play";
    const lastFrameIndex = Math.max(0, this.options.getFrameCount() - 1);
    elements.previousFrame.disabled = state.frameIndex <= 0;
    elements.nextFrame.disabled = state.frameIndex >= lastFrameIndex;
    elements.playbackRate.value = `${state.speed}`;
    this.options.getCameraControlsController()?.syncState(state);
    elements.skipPostGoalTransitions.checked = state.skipPostGoalTransitionsEnabled;
    elements.skipKickoffs.checked = state.skipKickoffsEnabled;
    elements.hitboxWireframes.checked = state.hitboxWireframesEnabled;
    elements.hitboxOnlyMode.checked = state.hitboxOnlyModeEnabled;
    elements.emptyState.hidden = true;
  }
}

export function createPlaybackReadoutsController(
  options: PlaybackReadoutsOptions,
): PlaybackReadoutsController {
  return new PlaybackReadoutsController(options);
}
