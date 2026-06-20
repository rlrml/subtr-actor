import type { SingletonWindowId, StatsWindowKind } from "./playerConfig.ts";
import { formatPlaybackRate, snapPlaybackRate } from "./playbackRateControl.ts";

export interface MountEventListenerElements {
  readonly root: HTMLElement;
  readonly launcherToggle: HTMLButtonElement;
  readonly launcherMenu: HTMLDivElement;
  readonly loadReplayAction: HTMLButtonElement;
  readonly emptyLoadReplay: HTMLButtonElement;
  readonly fileInput: HTMLInputElement;
  readonly togglePlayback: HTMLButtonElement;
  readonly previousFrame: HTMLButtonElement;
  readonly nextFrame: HTMLButtonElement;
  readonly playbackRate: HTMLInputElement;
  readonly playbackRateReadout: HTMLElement;
  readonly skipPostGoalTransitions: HTMLInputElement;
  readonly skipKickoffs: HTMLInputElement;
  readonly hitboxWireframes: HTMLInputElement;
  readonly hitboxOnlyMode: HTMLInputElement;
}

export interface MountEventListenerOptions {
  readonly elements: MountEventListenerElements;
  readonly signal: AbortSignal;
  setLauncherOpen(open: boolean): void;
  openReplayFilePicker(): void;
  getElementWindowId(element: HTMLElement): string | null;
  toggleWindow(id: SingletonWindowId): void;
  hideWindow(id: string): void;
  createStatsWindow(kind: StatsWindowKind): void;
  loadReplayFile(file: File): Promise<void>;
  togglePlayback(): void;
  stepFrames(delta: number): void;
  setPlaybackRate(value: number): void;
  setSkipPostGoalTransitionsEnabled(enabled: boolean): void;
  setSkipKickoffsEnabled(enabled: boolean): void;
  setHitboxWireframesEnabled(enabled: boolean): void;
  setHitboxOnlyModeEnabled(enabled: boolean): void;
}

export function installMountEventListeners({
  elements,
  signal,
  setLauncherOpen,
  openReplayFilePicker,
  getElementWindowId,
  toggleWindow,
  hideWindow,
  createStatsWindow,
  loadReplayFile,
  togglePlayback,
  stepFrames,
  setPlaybackRate,
  setSkipPostGoalTransitionsEnabled,
  setSkipKickoffsEnabled,
  setHitboxWireframesEnabled,
  setHitboxOnlyModeEnabled,
}: MountEventListenerOptions): void {
  elements.launcherToggle.addEventListener(
    "click",
    () => {
      setLauncherOpen(elements.launcherMenu.hidden);
    },
    { signal },
  );

  elements.root.addEventListener(
    "click",
    (event) => {
      if (!(event.target instanceof Element)) {
        return;
      }
      if (!event.target.closest(".top-chrome")) {
        setLauncherOpen(false);
      }
    },
    { signal },
  );

  elements.loadReplayAction.addEventListener("click", openReplayFilePicker, { signal });
  elements.emptyLoadReplay.addEventListener("click", openReplayFilePicker, { signal });

  elements.root.querySelectorAll<HTMLElement>("[data-window-toggle]").forEach((button) => {
    button.addEventListener(
      "click",
      () => {
        const id = button.dataset.windowToggle as SingletonWindowId | undefined;
        if (id) {
          toggleWindow(id);
          setLauncherOpen(false);
        }
      },
      { signal },
    );
  });

  elements.root.querySelectorAll<HTMLElement>("[data-window-hide]").forEach((button) => {
    button.addEventListener(
      "click",
      () => {
        const id = button.dataset.windowHide ?? getElementWindowId(button);
        if (id) {
          hideWindow(id);
        }
      },
      { signal },
    );
  });

  elements.root.querySelectorAll<HTMLElement>("[data-create-stats-window]").forEach((button) => {
    button.addEventListener(
      "click",
      () => {
        createStatsWindow(button.dataset.createStatsWindow as StatsWindowKind);
      },
      { signal },
    );
  });

  elements.fileInput.addEventListener(
    "change",
    () => {
      const file = elements.fileInput.files?.[0];
      if (file) {
        void loadReplayFile(file);
      }
    },
    { signal },
  );

  elements.togglePlayback.addEventListener("click", togglePlayback, { signal });
  elements.previousFrame.addEventListener("click", () => stepFrames(-1), { signal });
  elements.nextFrame.addEventListener("click", () => stepFrames(1), { signal });

  const syncPlaybackRate = () => {
    const rate = snapPlaybackRate(Number(elements.playbackRate.value));
    elements.playbackRate.value = `${rate}`;
    elements.playbackRateReadout.textContent = formatPlaybackRate(rate);
    setPlaybackRate(rate);
  };
  elements.playbackRate.addEventListener("input", syncPlaybackRate, { signal });
  elements.playbackRate.addEventListener("change", syncPlaybackRate, { signal });

  elements.skipPostGoalTransitions.addEventListener(
    "change",
    () => {
      setSkipPostGoalTransitionsEnabled(elements.skipPostGoalTransitions.checked);
    },
    { signal },
  );

  elements.skipKickoffs.addEventListener(
    "change",
    () => {
      setSkipKickoffsEnabled(elements.skipKickoffs.checked);
    },
    { signal },
  );

  elements.hitboxWireframes.addEventListener(
    "change",
    () => {
      setHitboxWireframesEnabled(elements.hitboxWireframes.checked);
    },
    { signal },
  );

  elements.hitboxOnlyMode.addEventListener(
    "change",
    () => {
      setHitboxOnlyModeEnabled(elements.hitboxOnlyMode.checked);
    },
    { signal },
  );
}
