import "./review.css";
import { mountStatEvaluationPlayer, type StatEvaluationPlayerHandle } from "./main.ts";
import {
  formatReplayLoadProgress,
  loadReplayBundleInWorker,
  type ReplayLoadBundle,
  type ReplayLoadProgress,
} from "./replayLoader.ts";
import { getReplayFetchRequestFromSearch } from "./replayUrl.ts";
import { mountStatsReport, type StatsReportData, type StatsReportHandle } from "./report.ts";
import type { StatsPlayerConfig } from "./playerConfig.ts";
import type { StatsTimeline } from "./statsTimeline.ts";

const REVIEW_DOCUMENT_CLASS = "replay-review-document";
const REVIEW_ROOT_CLASS = "replay-review-root";

export type ReplayReviewMode = "report" | "viewer";

export interface ReplayReviewDataProvider {
  readonly replayName?: string;
  readonly replayUrl?: URL | null;
  getStatsTimeline?(onProgress?: (progress: ReplayLoadProgress) => void): Promise<StatsTimeline>;
  getReplayBundle?(onProgress?: (progress: ReplayLoadProgress) => void): Promise<ReplayLoadBundle>;
}

export interface ReplayReviewMountOptions {
  provider?: ReplayReviewDataProvider | null;
  initialMode?: ReplayReviewMode;
}

export interface ReplayReviewHandle {
  readonly root: HTMLElement;
  setMode(mode: ReplayReviewMode): void;
  setProvider(
    provider: ReplayReviewDataProvider | null,
    options?: { mode?: ReplayReviewMode },
  ): void;
  destroy(): void;
}

function el<K extends keyof HTMLElementTagNameMap>(
  tagName: K,
  options: { className?: string; text?: string; id?: string } = {},
): HTMLElementTagNameMap[K] {
  const element = document.createElement(tagName);
  if (options.className) element.className = options.className;
  if (options.id) element.id = options.id;
  if (options.text !== undefined) element.textContent = options.text;
  return element;
}

function replayNameFromUrl(url: URL): string {
  return decodeURIComponent(url.pathname.split("/").pop() || "remote replay");
}

export function createReplayBytesReviewDataProvider(
  bytesSource: Uint8Array | (() => Promise<Uint8Array>),
  options: { replayName?: string; replayUrl?: URL | null } = {},
): ReplayReviewDataProvider {
  let bundlePromise: Promise<ReplayLoadBundle> | null = null;
  const getBytes = async () =>
    bytesSource instanceof Uint8Array ? bytesSource : await bytesSource();
  const loadBundle = (onProgress?: (progress: ReplayLoadProgress) => void) => {
    if (!bundlePromise) {
      bundlePromise = getBytes().then((bytes) =>
        loadReplayBundleInWorker(bytes, {
          reportEveryNFrames: 100,
          onProgress,
        }),
      );
    }
    return bundlePromise;
  };

  return {
    replayName: options.replayName,
    replayUrl: options.replayUrl ?? null,
    async getStatsTimeline(onProgress) {
      return (await loadBundle(onProgress)).statsTimeline;
    },
    getReplayBundle: loadBundle,
  };
}

export function createReplayUrlReviewDataProvider(
  replayUrl: string | URL,
): ReplayReviewDataProvider {
  const url = replayUrl instanceof URL ? replayUrl : new URL(replayUrl, window.location.href);
  return createReplayBytesReviewDataProvider(
    async () => {
      const response = await fetch(url);
      if (!response.ok) {
        throw new Error(`Failed to fetch replay: ${response.status} ${response.statusText}`);
      }
      return new Uint8Array(await response.arrayBuffer());
    },
    {
      replayName: replayNameFromUrl(url),
      replayUrl: url,
    },
  );
}

export function createReplayReviewDataProviderFromLocation(
  location: Location = window.location,
): ReplayReviewDataProvider | null {
  const request = getReplayFetchRequestFromSearch(location.search, location.href);
  if (!request) {
    return null;
  }
  return createReplayBytesReviewDataProvider(
    async () => {
      const response = await fetch(request.url, request.fetchInit);
      if (!response.ok) {
        const statusText = response.statusText ? ` ${response.statusText}` : "";
        throw new Error(`Failed to fetch replay: ${response.status}${statusText}`);
      }
      return new Uint8Array(await response.arrayBuffer());
    },
    {
      replayName: request.name,
      replayUrl: request.url,
    },
  );
}

function getInitialMode(explicitMode?: ReplayReviewMode): ReplayReviewMode {
  if (explicitMode) {
    return explicitMode;
  }
  const mode = new URL(window.location.href).searchParams.get("mode");
  return mode === "viewer" ? "viewer" : "report";
}

function syncModeToUrl(mode: ReplayReviewMode): void {
  const url = new URL(window.location.href);
  if (mode === "report") {
    url.searchParams.delete("mode");
  } else {
    url.searchParams.set("mode", mode);
  }
  window.history.replaceState(null, "", url);
}

export function mountReplayReview(
  root: HTMLElement,
  options: ReplayReviewMountOptions = {},
): ReplayReviewHandle {
  document.documentElement.classList.add(REVIEW_DOCUMENT_CLASS);
  document.body.classList.add(REVIEW_DOCUMENT_CLASS);
  root.classList.add(REVIEW_ROOT_CLASS);

  let provider: ReplayReviewDataProvider | null = options.provider ?? null;
  let mode = getInitialMode(options.initialMode);
  let reportHandle: StatsReportHandle | null = null;
  let playerHandle: StatEvaluationPlayerHandle | null = null;
  let statsTimelinePromise: Promise<StatsTimeline> | null = null;
  let bundlePromise: Promise<ReplayLoadBundle> | null = null;
  let pendingViewerConfig: StatsPlayerConfig | null = null;
  let disposed = false;

  const shell = el("main", { className: "replay-review-shell" });
  const toolbar = el("div", { className: "replay-review-toolbar" });
  const status = el("div", { className: "replay-review-status" });
  const reportButton = el("button", { text: "Stats" });
  const viewerButton = el("button", { text: "Viewer" });
  const fileLabel = el("label", { className: "replay-review-file", text: "Load replay" });
  const fileInput = el("input");
  const reportPane = el("section", { className: "replay-review-pane" });
  const viewerPane = el("section", { className: "replay-review-pane" });

  fileInput.type = "file";
  fileInput.accept = ".replay";
  fileLabel.append(fileInput);
  toolbar.append(status, fileLabel, reportButton, viewerButton);
  shell.append(toolbar, reportPane, viewerPane);
  root.replaceChildren(shell);

  const setStatus = (message: string) => {
    status.textContent = message;
  };

  const setProgressStatus = (progress: ReplayLoadProgress) => {
    setStatus(formatReplayLoadProgress(progress));
  };

  const resetMountedViews = () => {
    reportHandle?.destroy();
    reportHandle = null;
    playerHandle?.destroy();
    playerHandle = null;
    statsTimelinePromise = null;
    bundlePromise = null;
    pendingViewerConfig = null;
  };

  const getBundle = () => {
    if (!provider?.getReplayBundle) {
      return null;
    }
    if (!bundlePromise) {
      bundlePromise = provider.getReplayBundle(setProgressStatus);
    }
    return bundlePromise;
  };

  const getStatsTimeline = () => {
    if (!provider) {
      return null;
    }
    if (!statsTimelinePromise) {
      statsTimelinePromise = provider.getStatsTimeline
        ? provider.getStatsTimeline(setProgressStatus)
        : (getBundle()?.then((bundle) => bundle.statsTimeline) ?? null);
    }
    return statsTimelinePromise;
  };

  const renderEmpty = () => {
    reportPane.replaceChildren(
      el("section", {
        className: "replay-review-empty",
        text: "Load a replay to review stats and playback.",
      }),
    );
  };

  const ensureReport = async () => {
    if (reportHandle) {
      return;
    }
    const statsTimeline = getStatsTimeline();
    if (!statsTimeline) {
      renderEmpty();
      setStatus("No replay loaded");
      return;
    }
    reportPane.replaceChildren(
      el("section", {
        className: "replay-review-empty",
        text: "Loading stats...",
      }),
    );
    const data: StatsReportData = {
      fileName: provider?.replayName ?? "replay",
      replayUrl: provider?.replayUrl ?? null,
      statsTimeline: await statsTimeline,
    };
    if (disposed) {
      return;
    }
    reportHandle = mountStatsReport(reportPane, {
      initialData: data,
      showStandaloneActions: false,
      onWatchGoal(request) {
        pendingViewerConfig = request.config;
        playerHandle?.destroy();
        playerHandle = null;
        mode = "viewer";
        renderMode();
      },
    });
    setStatus(`Loaded ${data.fileName}`);
  };

  const ensureViewer = async () => {
    if (playerHandle) {
      return;
    }
    const bundle = getBundle();
    if (!bundle) {
      viewerPane.replaceChildren(
        el("section", {
          className: "replay-review-empty",
          text: "Replay playback is not available for this data source.",
        }),
      );
      setStatus("Viewer unavailable");
      return;
    }
    viewerPane.replaceChildren(
      el("section", {
        className: "replay-review-empty",
        text: "Loading viewer...",
      }),
    );
    const loadedBundle = await bundle;
    if (disposed) {
      return;
    }
    playerHandle = mountStatEvaluationPlayer(viewerPane, {
      initialBundle: loadedBundle,
      initialConfig: pendingViewerConfig,
      initialReplayName: provider?.replayName,
      loadFromLocation: false,
    });
    pendingViewerConfig = null;
    setStatus(`Loaded ${provider?.replayName ?? "replay"}`);
  };

  const renderMode = () => {
    reportButton.dataset.active = String(mode === "report");
    viewerButton.dataset.active = String(mode === "viewer");
    reportPane.hidden = mode !== "report";
    viewerPane.hidden = mode !== "viewer";
    syncModeToUrl(mode);
    void (mode === "report" ? ensureReport() : ensureViewer()).catch((error) => {
      console.error("Failed to render replay review mode:", error);
      setStatus(error instanceof Error ? error.message : "Failed to load replay review");
    });
  };

  reportButton.addEventListener("click", () => {
    mode = "report";
    renderMode();
  });
  viewerButton.addEventListener("click", () => {
    mode = "viewer";
    renderMode();
  });
  fileInput.addEventListener("change", () => {
    const file = fileInput.files?.[0];
    if (!file) {
      return;
    }
    provider = createReplayBytesReviewDataProvider(
      async () => new Uint8Array(await file.arrayBuffer()),
      {
        replayName: file.name,
        replayUrl: null,
      },
    );
    resetMountedViews();
    renderMode();
  });

  renderMode();

  return {
    root,
    setMode(nextMode) {
      mode = nextMode;
      renderMode();
    },
    setProvider(nextProvider, setProviderOptions = {}) {
      provider = nextProvider;
      if (setProviderOptions.mode) {
        mode = setProviderOptions.mode;
      }
      resetMountedViews();
      renderMode();
    },
    destroy() {
      disposed = true;
      resetMountedViews();
      root.classList.remove(REVIEW_ROOT_CLASS);
      document.documentElement.classList.remove(REVIEW_DOCUMENT_CLASS);
      document.body.classList.remove(REVIEW_DOCUMENT_CLASS);
      root.replaceChildren();
    },
  };
}
