import type { ReplayPlayerState } from "@rlrml/player";
import type { StatEvaluationPlayerElements } from "./appElements.ts";
import type { CameraControls } from "./cameraControls.ts";
import type { EventWindowsManager } from "./eventWindows.ts";
import type { MechanicsReviewController } from "./mechanicsReviewController.ts";
import type { StatsWindowsManager } from "./statsWindows.ts";

export interface ReplaySnapshotRenderer {
  render(state: ReplayPlayerState): void;
}

interface ReplaySnapshotRendererDeps {
  readonly elements: StatEvaluationPlayerElements;
  readonly playingUiUpdateIntervalMs: number;
  readonly statsWindowManager: StatsWindowsManager;
  getCameraControls(): CameraControls | null;
  getEventWindowsManager(): EventWindowsManager;
  getMechanicsReviewController(): MechanicsReviewController | null;
  renderScoreboard(frameIndex: number): void;
}

export function createReplaySnapshotRenderer(
  deps: ReplaySnapshotRendererDeps,
): ReplaySnapshotRenderer {
  let lastPlayingSnapshotUiUpdateAt = 0;

  return {
    render(state): void {
      if (deps.getMechanicsReviewController()?.enforceClipBoundary(state)) {
        return;
      }

      const now = performance.now();
      if (state.playing && now - lastPlayingSnapshotUiUpdateAt < deps.playingUiUpdateIntervalMs) {
        return;
      }
      lastPlayingSnapshotUiUpdateAt = now;

      deps.elements.timeReadout.textContent = `${state.currentTime.toFixed(2)}s`;
      deps.elements.frameReadout.textContent = `${state.frameIndex}`;
      deps.elements.durationReadout.textContent = `${state.duration.toFixed(2)}s`;
      deps.elements.playbackStatusReadout.textContent = state.playing ? "Playing" : "Paused";
      deps.elements.togglePlayback.textContent = state.playing ? "Pause" : "Play";
      deps.elements.playbackRate.value = `${state.speed}`;
      deps.elements.skipPostGoalTransitions.checked = state.skipPostGoalTransitionsEnabled;
      deps.elements.skipKickoffs.checked = state.skipKickoffsEnabled;
      deps.elements.emptyState.hidden = true;

      deps.getCameraControls()?.syncSnapshot(state);
      deps.statsWindowManager.render(state.frameIndex, { preserveOpenPickers: true });
      deps.renderScoreboard(state.frameIndex);
      deps.getEventWindowsManager().syncPlaylistTimeline(state);
    },
  };
}
