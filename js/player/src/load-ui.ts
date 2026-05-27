import type {
  ReplayLoadOverlayController,
  ReplayLoadOverlayOptions,
} from "./types";
import { formatReplayLoadProgress, formatReplayLoadProgressMeta } from "./load-ui-format";
import { ensureReplayLoadOverlayStyles } from "./load-ui-styles";

export { formatReplayLoadProgress, formatReplayLoadProgressMeta } from "./load-ui-format";

export function createReplayLoadOverlay(
  container: HTMLElement,
  options: ReplayLoadOverlayOptions = {},
): ReplayLoadOverlayController {
  ensureReplayLoadOverlayStyles();

  let originalContainerPosition: string | null = null;
  if (getComputedStyle(container).position === "static") {
    originalContainerPosition = container.style.position;
    container.style.position = "relative";
  }

  const root = document.createElement("div");
  root.className = "sap-load-overlay";

  const panel = document.createElement("div");
  panel.className = "sap-load-overlay__panel";
  panel.dataset.state = "loading";

  const title = document.createElement("p");
  title.className = "sap-load-overlay__title";
  title.textContent = options.title ?? "Replay Loading";

  const status = document.createElement("p");
  status.className = "sap-load-overlay__status";
  status.textContent = "Loading replay...";

  const bar = document.createElement("div");
  bar.className = "sap-load-overlay__bar";

  const fill = document.createElement("div");
  fill.className = "sap-load-overlay__fill";
  bar.append(fill);

  const meta = document.createElement("div");
  meta.className = "sap-load-overlay__meta";
  meta.textContent = "";

  panel.append(title, status, bar, meta);
  root.append(panel);
  container.append(root);

  const setProgressWidth = (progress: number | undefined) => {
    const bounded = Math.max(0, Math.min(1, progress ?? 0));
    fill.style.width = `${Math.round(bounded * 100)}%`;
  };

  return {
    update(progress) {
      panel.dataset.state = "loading";
      status.textContent = options.formatProgress?.(progress) ?? formatReplayLoadProgress(progress);
      setProgressWidth(progress.progress);
      meta.textContent = formatReplayLoadProgressMeta(progress);
    },
    complete(message = "Replay loaded") {
      panel.dataset.state = "complete";
      status.textContent = message;
      fill.style.width = "100%";
      meta.textContent = "";
    },
    fail(message) {
      panel.dataset.state = "error";
      status.textContent = message;
      meta.textContent = "Loading failed";
    },
    destroy() {
      root.remove();
      if (originalContainerPosition !== null) {
        container.style.position = originalContainerPosition;
      }
    },
  };
}
