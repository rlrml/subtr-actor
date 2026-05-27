import type { StatEvaluationPlayerElements } from "./appElements.ts";
import type { CameraControls } from "./cameraControls.ts";
import type { MechanicsReviewController } from "./mechanicsReviewController.ts";
import type { SingletonWindowId, StatsWindowKind } from "./playerConfig.ts";
import type { RecordingControls } from "./recordingControls.ts";
import type { ReplayLoadController } from "./replayLoadController.ts";
import { createFileReplaySource } from "./replayInputSources.ts";

interface ReplayPlaybackControls {
  setPlaybackRate(rate: number): void;
  setSkipKickoffsEnabled(enabled: boolean): void;
  setSkipPostGoalTransitionsEnabled(enabled: boolean): void;
  togglePlayback(): void;
}

export interface StatEvaluationPlayerEventListenerDeps {
  root: HTMLElement;
  elements: StatEvaluationPlayerElements;
  signal: AbortSignal;
  createStatsWindow(kind: StatsWindowKind): void;
  getCameraControls(): CameraControls | null;
  getElementWindowId(element: HTMLElement): string | null;
  getMechanicsReviewController(): MechanicsReviewController | null;
  getRecordingControls(): RecordingControls | null;
  getReplayLoadController(): ReplayLoadController | null;
  getReplayPlayer(): ReplayPlaybackControls | null;
  hideWindow(id: string): void;
  openReplayFilePicker(): void;
  scheduleConfigUrlUpdate(): void;
  setLauncherOpen(open: boolean): void;
  toggleWindow(id: SingletonWindowId): void;
}

export function installStatEvaluationPlayerEventListeners(
  deps: StatEvaluationPlayerEventListenerDeps,
): void {
  const { elements, root, signal } = deps;

  elements.launcherToggle.addEventListener(
    "click",
    () => {
      deps.setLauncherOpen(elements.launcherMenu.hidden);
    },
    { signal },
  );

  root.addEventListener(
    "click",
    (event) => {
      if (!(event.target instanceof Element)) {
        return;
      }
      if (!event.target.closest(".top-chrome")) {
        deps.setLauncherOpen(false);
      }
    },
    { signal },
  );

  elements.loadReplayAction.addEventListener("click", deps.openReplayFilePicker, { signal });
  elements.emptyLoadReplay.addEventListener("click", deps.openReplayFilePicker, { signal });

  root.querySelectorAll<HTMLElement>("[data-window-toggle]").forEach((button) => {
    button.addEventListener(
      "click",
      () => {
        const id = button.dataset.windowToggle as SingletonWindowId | undefined;
        if (id) {
          deps.toggleWindow(id);
          deps.setLauncherOpen(false);
        }
      },
      { signal },
    );
  });

  root.querySelectorAll<HTMLElement>("[data-window-hide]").forEach((button) => {
    button.addEventListener(
      "click",
      () => {
        const id = button.dataset.windowHide ?? deps.getElementWindowId(button);
        if (id) {
          deps.hideWindow(id);
        }
      },
      { signal },
    );
  });

  root.querySelectorAll<HTMLElement>("[data-create-stats-window]").forEach((button) => {
    button.addEventListener(
      "click",
      () => {
        deps.createStatsWindow(button.dataset.createStatsWindow as StatsWindowKind);
      },
      { signal },
    );
  });

  elements.fileInput.addEventListener(
    "change",
    async () => {
      const file = elements.fileInput.files?.[0];
      if (!file) return;

      try {
        deps.getMechanicsReviewController()?.clearCurrentReplay();
        await deps.getReplayLoadController()?.loadReplay(createFileReplaySource(file));
      } catch (error) {
        console.error("Failed to load replay:", error);
        elements.statusReadout.textContent =
          error instanceof Error ? error.message : "Failed to load replay";
      }
    },
    { signal },
  );

  deps.getMechanicsReviewController()?.installListeners(signal);

  elements.togglePlayback.addEventListener(
    "click",
    () => {
      deps.getReplayPlayer()?.togglePlayback();
      deps.scheduleConfigUrlUpdate();
    },
    { signal },
  );

  elements.playbackRate.addEventListener(
    "change",
    () => {
      deps.getReplayPlayer()?.setPlaybackRate(Number(elements.playbackRate.value));
      deps.scheduleConfigUrlUpdate();
    },
    { signal },
  );

  deps.getRecordingControls()?.installListeners(signal);
  deps.getCameraControls()?.installListeners(signal);

  elements.skipPostGoalTransitions.addEventListener(
    "change",
    () => {
      deps
        .getReplayPlayer()
        ?.setSkipPostGoalTransitionsEnabled(elements.skipPostGoalTransitions.checked);
      deps.scheduleConfigUrlUpdate();
    },
    { signal },
  );

  elements.skipKickoffs.addEventListener(
    "change",
    () => {
      deps.getReplayPlayer()?.setSkipKickoffsEnabled(elements.skipKickoffs.checked);
      deps.scheduleConfigUrlUpdate();
    },
    { signal },
  );
}
