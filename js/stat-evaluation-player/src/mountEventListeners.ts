import type { SingletonWindowId, StatsWindowKind } from "./playerConfig.ts";

export interface MountEventListenerElements {
  readonly root: HTMLElement;
  readonly launcherToggle: HTMLButtonElement;
  readonly launcherMenu: HTMLDivElement;
  readonly loadReplayAction: HTMLButtonElement;
  readonly emptyLoadReplay: HTMLButtonElement;
  readonly fileInput: HTMLInputElement;
  readonly togglePlayback: HTMLButtonElement;
  readonly playbackRate: HTMLSelectElement;
  readonly skipPostGoalTransitions: HTMLInputElement;
  readonly skipKickoffs: HTMLInputElement;
  readonly hitboxWireframes: HTMLInputElement;
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
  setPlaybackRate(value: number): void;
  setSkipPostGoalTransitionsEnabled(enabled: boolean): void;
  setSkipKickoffsEnabled(enabled: boolean): void;
  setHitboxWireframesEnabled(enabled: boolean): void;
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
  setPlaybackRate,
  setSkipPostGoalTransitionsEnabled,
  setSkipKickoffsEnabled,
  setHitboxWireframesEnabled,
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

  elements.playbackRate.addEventListener(
    "change",
    () => {
      setPlaybackRate(Number(elements.playbackRate.value));
    },
    { signal },
  );

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
}
