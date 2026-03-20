import {
  formatReplayLoadProgress,
  getReplayLoadPhase,
  getReplayLoadPhaseStates,
  listReplayLoadPhases,
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

  const title = document.createElement("h2");
  title.id = "replay-load-modal-title";
  title.className = "replay-load-modal__title";
  title.textContent = "Preparing replay pipeline";

  const status = document.createElement("p");
  status.className = "replay-load-modal__status";
  status.textContent = "Preparing replay...";

  const phaseList = document.createElement("div");
  phaseList.className = "replay-load-modal__phase-list";

  const phaseRows = new Map<
    ReplayLoadProgress["stage"],
    { row: HTMLDivElement; fill: HTMLDivElement }
  >();

  for (const phase of listReplayLoadPhases()) {
    const row = document.createElement("div");
    row.className = "replay-load-modal__phase-row";
    row.dataset.state = "pending";

    const label = document.createElement("p");
    label.className = "replay-load-modal__phase-label";
    label.textContent = `${phase.index}. ${phase.label}`;

    const bar = document.createElement("div");
    bar.className = "replay-load-modal__phase-bar";

    const fill = document.createElement("div");
    fill.className = "replay-load-modal__phase-fill";
    fill.dataset.indeterminate = "false";
    bar.append(fill);

    row.append(label, bar);
    phaseList.append(row);
    phaseRows.set(phase.stage, { row, fill });
  }

  const meta = document.createElement("p");
  meta.className = "replay-load-modal__meta";

  dialog.append(eyebrow, title, status, phaseList, meta);
  modal.append(dialog);
  root.append(modal);

  let activeFileName = "";

  const resetPhaseRows = () => {
    for (const { row, fill } of phaseRows.values()) {
      row.dataset.state = "pending";
      fill.style.width = "0%";
      fill.dataset.indeterminate = "false";
    }
  };

  const updatePhaseRows = (progress: ReplayLoadProgress) => {
    for (const phaseState of getReplayLoadPhaseStates(progress)) {
      const row = phaseRows.get(phaseState.stage);
      if (!row) {
        continue;
      }
      row.row.dataset.state = phaseState.state;
      row.fill.dataset.indeterminate = phaseState.indeterminate ? "true" : "false";
      row.fill.style.width = `${Math.round(phaseState.completion * 100)}%`;
    }
  };

  const setVisible = (visible: boolean) => {
    modal.hidden = !visible;
  };

  return {
    show(fileName, nextStatus = "Preparing replay...") {
      activeFileName = fileName;
      setVisible(true);
      resetPhaseRows();
      title.textContent = "Preparing replay pipeline";
      status.textContent = nextStatus;
      meta.textContent = `Loading ${fileName}`;
    },
    update(progress) {
      setVisible(true);
      const currentPhase = getReplayLoadPhase(progress);
      updatePhaseRows(progress);
      title.textContent =
        `Phase ${currentPhase.index} of ${currentPhase.total}: ${currentPhase.label}`;
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
