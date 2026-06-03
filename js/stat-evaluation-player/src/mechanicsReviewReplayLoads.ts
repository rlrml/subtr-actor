import {
  getMechanicsReviewReplayLabel,
  getMechanicsReviewReplayPath,
  resolveMechanicsReviewUrl,
  type ActiveMechanicsReview,
  type MechanicsReviewItem,
  type MechanicsReviewReplayLoadState,
} from "./mechanicsReview.ts";
import {
  formatReplayLoadProgress,
  loadReplayBundleInWorker,
  type ReplayLoadBundle,
  type ReplayLoadProgress,
} from "./replayLoader.ts";

export interface MechanicsReviewReplaySource {
  name: string;
  preparingStatus: string;
  readBytes(): Promise<Uint8Array>;
}

export interface MechanicsReviewReplayLoadsElements {
  readonly reviewSummary: HTMLElement;
  readonly loadingSummary: HTMLElement;
  readonly loadingActive: HTMLElement;
  readonly loadingList: HTMLDivElement;
}

export interface MechanicsReviewReplayLoadsOptions {
  readonly elements: MechanicsReviewReplayLoadsElements;
  isActiveReview(review: ActiveMechanicsReview): boolean;
  onActiveLoadProgress(progress: ReplayLoadProgress): void;
}

export class MechanicsReviewReplayLoadsController {
  constructor(private readonly options: MechanicsReviewReplayLoadsOptions) {}

  createReplaySource(
    item: MechanicsReviewItem,
    review: ActiveMechanicsReview,
    signal?: AbortSignal,
  ): MechanicsReviewReplaySource {
    const replayPath = getMechanicsReviewReplayPath(item, review);
    const url = resolveMechanicsReviewUrl(replayPath, review.sourceUrl);
    return {
      name: getMechanicsReviewReplayLabel(item, review),
      preparingStatus: "Loading review replay...",
      async readBytes() {
        const response = await fetch(url, { signal });
        if (!response.ok) {
          const statusText = response.statusText ? ` ${response.statusText}` : "";
          throw new Error(
            `Failed to fetch review replay from ${url} (${response.status}${statusText})`,
          );
        }
        return new Uint8Array(await response.arrayBuffer());
      },
    };
  }

  initialize(review: ActiveMechanicsReview): void {
    const clipCounts = this.getReplayClipCounts(review);
    for (const [replayId, item] of this.getReplayItems(review)) {
      let path = "";
      let label = replayId;
      try {
        path = getMechanicsReviewReplayPath(item, review);
        label = getMechanicsReviewReplayLabel(item, review);
      } catch {
        const replay = review.replaysById.get(replayId);
        label = replay?.label ?? replayId;
      }
      review.replayLoadStates.set(replayId, {
        replayId,
        label,
        path,
        clipCount: clipCounts.get(replayId) ?? 0,
        status: "idle",
        progress: null,
        error: null,
      });
    }
  }

  preload(review: ActiveMechanicsReview, currentReplayId: string): void {
    if (review.preloading) {
      return;
    }
    review.preloading = true;
    void (async () => {
      try {
        for (const [replayId, item] of this.getReplayItems(review)) {
          if (replayId === currentReplayId) {
            continue;
          }
          const state = review.replayLoadStates.get(replayId);
          if (state?.status === "loaded" || state?.status === "loading") {
            continue;
          }
          try {
            await this.loadBundle(item, review);
          } catch {
            // Background preload failures are rendered in the replay load window.
          }
        }
      } finally {
        review.preloading = false;
      }
    })();
  }

  loadBundle(
    item: MechanicsReviewItem,
    review: ActiveMechanicsReview,
  ): Promise<ReplayLoadBundle> {
    const cached = review.replayLoadCache.get(item.replay);
    if (cached) {
      return cached;
    }

    const source = this.createReplaySource(item, review);
    this.updateLoadState(review, item.replay, {
      label: source.name,
      path: getMechanicsReviewReplayPath(item, review),
      status: "loading",
      progress: null,
      error: null,
    });
    const loadPromise = Promise.resolve()
      .then(async () => {
        const bytes = await source.readBytes();
        return loadReplayBundleInWorker(bytes, {
          reportEveryNFrames: 100,
          onProgress: (progress) => {
            this.updateLoadState(review, item.replay, {
              status: "loading",
              progress,
              error: null,
            });
          },
        });
      })
      .then((bundle) => {
        this.updateLoadState(review, item.replay, {
          status: "loaded",
          progress: null,
          error: null,
        });
        return bundle;
      })
      .catch((error) => {
        review.replayLoadCache.delete(item.replay);
        this.updateLoadState(review, item.replay, {
          status: "error",
          error: error instanceof Error ? error.message : String(error),
        });
        throw error;
      });
    review.replayLoadCache.set(item.replay, loadPromise);
    return loadPromise;
  }

  render(review: ActiveMechanicsReview | null): void {
    const { reviewSummary, loadingSummary, loadingActive, loadingList } = this.options.elements;
    const states = review ? Array.from(review.replayLoadStates.values()) : [];
    const loaded = states.filter((state) => state.status === "loaded").length;
    const loading = states.filter((state) => state.status === "loading").length;
    const failed = states.filter((state) => state.status === "error").length;
    const pending = states.filter((state) => state.status === "idle").length;
    const summaryText =
      states.length === 0
        ? "0 replays"
        : `${loaded}/${states.length} loaded${loading > 0 ? `, ${loading} loading` : ""}${
            failed > 0 ? `, ${failed} failed` : ""
          }`;
    reviewSummary.textContent = summaryText;
    loadingSummary.textContent = summaryText;
    loadingActive.textContent =
      states.length === 0
        ? "No playlist"
        : loading > 0
          ? `${loading} active, ${pending} pending`
          : failed > 0
            ? `${failed} failed`
            : review?.preloading
              ? `Background queue, ${pending} pending`
              : loaded === states.length
                ? "Complete"
                : `${pending} pending`;

    loadingList.replaceChildren();
    if (!review || states.length === 0) {
      const empty = document.createElement("p");
      empty.className = "stat-window-empty";
      empty.textContent = "No replay sources.";
      loadingList.append(empty);
      return;
    }

    for (const state of states) {
      const row = document.createElement("div");
      row.className = `mechanics-review-replay-load ${state.status}`;

      const main = document.createElement("div");
      main.className = "mechanics-review-replay-load-main";
      const title = document.createElement("span");
      title.className = "mechanics-review-replay-load-title";
      title.textContent = state.label;
      const meta = document.createElement("span");
      meta.className = "mechanics-review-replay-load-meta";
      meta.textContent = [
        state.replayId,
        `${state.clipCount} ${state.clipCount === 1 ? "clip" : "clips"}`,
        state.path,
      ]
        .filter(Boolean)
        .join(" · ");
      main.append(title, meta);

      const status = document.createElement("strong");
      status.className = "mechanics-review-replay-load-status";
      status.textContent = this.replayLoadStatusText(state);

      const progress = document.createElement("div");
      progress.className = "mechanics-review-replay-load-progress";
      const bar = document.createElement("span");
      bar.style.width = `${Math.round(this.replayLoadProgressValue(state) * 100)}%`;
      progress.append(bar);

      row.append(main, status, progress);
      loadingList.append(row);
    }
  }

  private updateLoadState(
    review: ActiveMechanicsReview,
    replayId: string,
    patch: Partial<Omit<MechanicsReviewReplayLoadState, "replayId">>,
  ): void {
    const current =
      review.replayLoadStates.get(replayId) ??
      ({
        replayId,
        label: replayId,
        path: "",
        clipCount: 0,
        status: "idle",
        progress: null,
        error: null,
      } satisfies MechanicsReviewReplayLoadState);
    review.replayLoadStates.set(replayId, {
      ...current,
      ...patch,
    });
    const activeItem = review.manifest.items[review.currentIndex];
    if (review.loading && activeItem?.replay === replayId && patch.progress) {
      this.options.onActiveLoadProgress(patch.progress);
    }
    if (this.options.isActiveReview(review)) {
      this.render(review);
    }
  }

  private getReplayItems(review: ActiveMechanicsReview): Map<string, MechanicsReviewItem> {
    const itemsByReplayId = new Map<string, MechanicsReviewItem>();
    for (const item of review.manifest.items) {
      if (!itemsByReplayId.has(item.replay)) {
        itemsByReplayId.set(item.replay, item);
      }
    }
    return itemsByReplayId;
  }

  private getReplayClipCounts(review: ActiveMechanicsReview): Map<string, number> {
    const counts = new Map<string, number>();
    for (const item of review.manifest.items) {
      counts.set(item.replay, (counts.get(item.replay) ?? 0) + 1);
    }
    return counts;
  }

  private replayLoadStatusText(state: MechanicsReviewReplayLoadState): string {
    if (state.status === "idle") {
      return "Pending";
    }
    if (state.status === "loading") {
      return this.replayLoadStateProgress(state.progress) || "Loading";
    }
    if (state.status === "loaded") {
      return "Loaded";
    }
    return state.error ? `Failed: ${state.error}` : "Failed";
  }

  private replayLoadStateProgress(progress: ReplayLoadProgress | null): string {
    if (!progress) {
      return "";
    }
    const label = formatReplayLoadProgress(progress);
    if (progress.processedFrames !== undefined) {
      const total = progress.totalFrames !== undefined ? ` / ${progress.totalFrames}` : "";
      return `${label} (${progress.processedFrames}${total} frames)`;
    }
    if (progress.processedChunks !== undefined) {
      const total = progress.totalChunks !== undefined ? ` / ${progress.totalChunks}` : "";
      return `${label} (${progress.processedChunks}${total} chunks)`;
    }
    return label;
  }

  private replayLoadProgressValue(state: MechanicsReviewReplayLoadState): number {
    if (state.status === "loaded") {
      return 1;
    }
    const value = state.progress?.progress;
    return typeof value === "number" && Number.isFinite(value) ? Math.max(0, Math.min(1, value)) : 0;
  }
}

export function createMechanicsReviewReplayLoadsController(
  options: MechanicsReviewReplayLoadsOptions,
): MechanicsReviewReplayLoadsController {
  return new MechanicsReviewReplayLoadsController(options);
}
