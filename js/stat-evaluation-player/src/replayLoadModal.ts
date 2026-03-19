import {
  formatReplayLoadProgress,
  getReplayLoadCompletion,
  type ReplayLoadProgress,
} from "./replayLoader.ts";

export interface ReplayLoadModalController {
  show(fileName: string, status?: string): void;
  update(progress: ReplayLoadProgress): void;
  hide(): void;
  destroy(): void;
}

export function createReplayLoadModal(
  root: HTMLElement,
): ReplayLoadModalController {
  const modal = document.createElement("div");
  modal.className = "replay-load-modal";
  modal.hidden = true;

  const dialog = document.createElement("div");
  dialog.className = "replay-load-modal__dialog";
  dialog.setAttribute("role", "dialog");
  dialog.setAttribute("aria-modal", "true");
  dialog.setAttribute("aria-labelledby", "replay-load-modal-title");

  const eyebrow = document.createElement("p");
  eyebrow.className = "replay-load-modal__eyebrow";
  eyebrow.textContent = "Replay loading";

  const percent = document.createElement("h2");
  percent.id = "replay-load-modal-title";
  percent.className = "replay-load-modal__percent";
  percent.textContent = "0%";

  const status = document.createElement("p");
  status.className = "replay-load-modal__status";
  status.textContent = "Preparing replay...";

  const bar = document.createElement("div");
  bar.className = "replay-load-modal__bar";

  const fill = document.createElement("div");
  fill.className = "replay-load-modal__fill";
  bar.append(fill);

  const meta = document.createElement("p");
  meta.className = "replay-load-modal__meta";

  dialog.append(eyebrow, percent, status, bar, meta);
  modal.append(dialog);
  root.append(modal);

  let activeFileName = "";

  const setProgress = (fraction: number) => {
    const bounded = Math.max(0, Math.min(1, fraction));
    fill.style.width = `${bounded * 100}%`;
    percent.textContent = `${Math.round(bounded * 100)}%`;
  };

  const setVisible = (visible: boolean) => {
    modal.hidden = !visible;
  };

  return {
    show(fileName, nextStatus = "Preparing replay...") {
      activeFileName = fileName;
      setVisible(true);
      setProgress(0);
      status.textContent = nextStatus;
      meta.textContent = `Loading ${fileName}`;
    },
    update(progress) {
      setVisible(true);
      setProgress(getReplayLoadCompletion(progress));
      status.textContent = formatReplayLoadProgress(progress);
      if (
        progress.stage === "processing" &&
        progress.totalFrames !== undefined
      ) {
        meta.textContent =
          `${progress.processedFrames ?? 0}/${progress.totalFrames} frames`;
        return;
      }
      meta.textContent = activeFileName ? `Loading ${activeFileName}` : "";
    },
    hide() {
      setVisible(false);
    },
    destroy() {
      modal.remove();
    },
  };
}
