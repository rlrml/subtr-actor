import type {
  ReplayLoadOverlayController,
  ReplayLoadOverlayOptions,
  ReplayLoadProgress,
} from "./types";

const STYLE_ID = "subtr-actor-replay-load-overlay-styles";

function ensureStyles(): void {
  if (document.getElementById(STYLE_ID)) {
    return;
  }

  const style = document.createElement("style");
  style.id = STYLE_ID;
  style.textContent = `
    .sap-load-overlay {
      position: absolute;
      inset: 0;
      display: flex;
      align-items: center;
      justify-content: center;
      padding: 20px;
      background:
        radial-gradient(circle at top, rgba(255,255,255,0.12), transparent 50%),
        rgba(10, 15, 26, 0.72);
      backdrop-filter: blur(8px);
      z-index: 40;
      pointer-events: none;
    }

    .sap-load-overlay__panel {
      width: min(360px, 100%);
      padding: 18px 20px;
      border: 1px solid rgba(255,255,255,0.14);
      border-radius: 16px;
      background: rgba(8, 12, 20, 0.88);
      box-shadow: 0 20px 60px rgba(0,0,0,0.35);
      color: #f5f7fb;
      font: 500 14px/1.4 "IBM Plex Sans", "Avenir Next", sans-serif;
      letter-spacing: 0.01em;
    }

    .sap-load-overlay__title {
      margin: 0 0 10px;
      font-size: 12px;
      text-transform: uppercase;
      letter-spacing: 0.08em;
      color: rgba(255,255,255,0.64);
    }

    .sap-load-overlay__status {
      margin: 0 0 12px;
      font-size: 15px;
      color: #ffffff;
    }

    .sap-load-overlay__bar {
      overflow: hidden;
      height: 8px;
      border-radius: 999px;
      background: rgba(255,255,255,0.12);
    }

    .sap-load-overlay__fill {
      width: 0%;
      height: 100%;
      border-radius: inherit;
      background: linear-gradient(90deg, #58c4dd 0%, #f4b860 100%);
      transition: width 120ms linear;
    }

    .sap-load-overlay__meta {
      margin-top: 10px;
      font-size: 12px;
      color: rgba(255,255,255,0.6);
    }

    .sap-load-overlay__panel[data-state="error"] .sap-load-overlay__fill {
      background: linear-gradient(90deg, #ff6b6b 0%, #ff9b6b 100%);
      width: 100% !important;
    }
  `;
  document.head.append(style);
}

export function formatReplayLoadProgress(progress: ReplayLoadProgress): string {
  if (progress.stage === "processing") {
    const percent = progress.progress === undefined
      ? null
      : Math.round(progress.progress * 100);
    if (percent === null || progress.totalFrames === undefined) {
      return "Processing replay frames...";
    }
    return `Processing replay frames... ${percent}% (${progress.processedFrames ?? 0}/${progress.totalFrames})`;
  }

  if (progress.stage === "validating") {
    return "Validating replay...";
  }

  if (progress.stage === "normalizing") {
    return "Normalizing replay data...";
  }

  return "Loading replay...";
}

export function createReplayLoadOverlay(
  container: HTMLElement,
  options: ReplayLoadOverlayOptions = {},
): ReplayLoadOverlayController {
  ensureStyles();

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
      status.textContent = options.formatProgress?.(progress)
        ?? formatReplayLoadProgress(progress);
      setProgressWidth(progress.progress);
      if (progress.totalFrames !== undefined) {
        meta.textContent = progress.processedFrames === undefined
          ? `${progress.totalFrames} frames`
          : `${progress.processedFrames}/${progress.totalFrames} frames`;
      } else {
        meta.textContent = progress.stage;
      }
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
