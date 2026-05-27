import type { ReplayPlayer, ReplayPlayerState } from "@rlrml/player";
import { mustElement } from "./floatingWindows.ts";
import {
  createMechanicsReviewReplaySource,
  formatMechanicsReviewClipDetails,
  formatMechanicsReviewEventDetails,
  formatMechanicsReviewStatus,
  getMechanicsReviewBoundTime,
  getMechanicsReviewDecisionEndpoint,
  getMechanicsReviewItemLabel,
  getMechanicsReviewMechanicLabel,
  getMechanicsReviewPlayerId,
  getMechanicsReviewPlayerName,
  getMechanicsReviewReplayItems,
  getMechanicsReviewReplayLabel,
  getMechanicsReviewReplayPath,
  getMechanicsReviewUrlFromLocation,
  initializeMechanicsReviewReplayLoadStates,
  mechanicsReviewAuthHeaders,
  parseMechanicsReviewPlaylistJson,
  resolveMechanicsReviewUrl,
  type ActiveMechanicsReview,
  type MechanicsReviewItem,
  type MechanicsReviewPlaylist,
  type MechanicsReviewReplay,
  type MechanicsReviewReplayLoadState,
} from "./mechanicsReview.ts";
import {
  formatReplayLoadProgress,
  loadReplayBundleInWorker,
  type ReplayLoadBundle,
  type ReplayLoadProgress,
} from "./replayLoader.ts";
import type { ReplayInputSource } from "./replayInputSources.ts";
import { renderMechanicsReviewReplayLoads } from "./mechanicsReviewReplayLoads.ts";

export interface MechanicsReviewElements {
  file: HTMLInputElement;
  url: HTMLInputElement;
  loadUrl: HTMLButtonElement;
  status: HTMLElement;
  index: HTMLElement;
  title: HTMLElement;
  mechanic: HTMLElement;
  player: HTMLElement;
  clip: HTMLElement;
  event: HTMLElement;
  reason: HTMLElement;
  previous: HTMLButtonElement;
  replay: HTMLButtonElement;
  next: HTMLButtonElement;
  confirm: HTMLButtonElement;
  reject: HTMLButtonElement;
  uncertain: HTMLButtonElement;
  replayLoadSummary: HTMLElement;
  replayLoadingSummary: HTMLElement;
  replayLoadingActive: HTMLElement;
  replayLoadingList: HTMLDivElement;
  count: HTMLElement;
  list: HTMLDivElement;
}

export interface MechanicsReviewControllerOptions {
  elements: MechanicsReviewElements;
  getReplayPlayer: () => ReplayPlayer | null;
  loadReplayBundleForDisplay: (
    source: ReplayInputSource,
    bundlePromise: Promise<ReplayLoadBundle>,
  ) => Promise<void>;
  resetTransitionSkipControls: () => void;
  clearFreeCameraPreset: () => void;
  showWindow: (id: "mechanics-review" | "replay-loading") => void;
  setStatusReadout: (message: string) => void;
  updateReplayLoadModal: (progress: ReplayLoadProgress) => void;
}

export interface MechanicsReviewController {
  clearCurrentClip(): void;
  clearCurrentReplay(): void;
  enforceClipBoundary(state: ReplayPlayerState): boolean;
  installListeners(signal: AbortSignal): void;
  loadFromLocation(signal: AbortSignal): void;
  render(): void;
  reset(): void;
}

export function getMechanicsReviewElements(root: ParentNode): MechanicsReviewElements {
  return {
    file: mustElement<HTMLInputElement>(root, "#mechanics-review-file"),
    url: mustElement<HTMLInputElement>(root, "#mechanics-review-url"),
    loadUrl: mustElement<HTMLButtonElement>(root, "#mechanics-review-load-url"),
    status: mustElement<HTMLElement>(root, "#mechanics-review-status"),
    index: mustElement<HTMLElement>(root, "#mechanics-review-index"),
    title: mustElement<HTMLElement>(root, "#mechanics-review-title"),
    mechanic: mustElement<HTMLElement>(root, "#mechanics-review-mechanic"),
    player: mustElement<HTMLElement>(root, "#mechanics-review-player"),
    clip: mustElement<HTMLElement>(root, "#mechanics-review-clip"),
    event: mustElement<HTMLElement>(root, "#mechanics-review-event"),
    reason: mustElement<HTMLElement>(root, "#mechanics-review-reason"),
    previous: mustElement<HTMLButtonElement>(root, "#mechanics-review-prev"),
    replay: mustElement<HTMLButtonElement>(root, "#mechanics-review-replay"),
    next: mustElement<HTMLButtonElement>(root, "#mechanics-review-next"),
    confirm: mustElement<HTMLButtonElement>(root, "#mechanics-review-confirm"),
    reject: mustElement<HTMLButtonElement>(root, "#mechanics-review-reject"),
    uncertain: mustElement<HTMLButtonElement>(root, "#mechanics-review-uncertain"),
    replayLoadSummary: mustElement<HTMLElement>(root, "#mechanics-review-replay-load-summary"),
    replayLoadingSummary: mustElement<HTMLElement>(root, "#replay-loading-summary"),
    replayLoadingActive: mustElement<HTMLElement>(root, "#replay-loading-active"),
    replayLoadingList: mustElement<HTMLDivElement>(root, "#replay-loading-list"),
    count: mustElement<HTMLElement>(root, "#mechanics-review-count"),
    list: mustElement<HTMLDivElement>(root, "#mechanics-review-list"),
  };
}

export function createMechanicsReviewController(
  options: MechanicsReviewControllerOptions,
): MechanicsReviewController {
  const { elements } = options;
  let activeReview: ActiveMechanicsReview | null = null;
  let boundaryGuard = false;

  function setStatus(message: string): void {
    elements.status.textContent = message;
  }

  function updateReplayLoadState(
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
      options.setStatusReadout(formatReplayLoadProgress(patch.progress));
      options.updateReplayLoadModal(patch.progress);
    }
    if (activeReview === review) {
      renderReplayLoads(review);
    }
  }

  function renderReplayLoads(review: ActiveMechanicsReview | null): void {
    renderMechanicsReviewReplayLoads(elements, review);
  }

  function loadReplayBundle(
    item: MechanicsReviewItem,
    review: ActiveMechanicsReview,
  ): Promise<ReplayLoadBundle> {
    const cached = review.replayLoadCache.get(item.replay);
    if (cached) {
      return cached;
    }

    const source = createMechanicsReviewReplaySource(item, review);
    updateReplayLoadState(review, item.replay, {
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
          onProgress(progress) {
            updateReplayLoadState(review, item.replay, {
              status: "loading",
              progress,
              error: null,
            });
          },
        });
      })
      .then((bundle) => {
        updateReplayLoadState(review, item.replay, {
          status: "loaded",
          progress: null,
          error: null,
        });
        return bundle;
      })
      .catch((error) => {
        review.replayLoadCache.delete(item.replay);
        updateReplayLoadState(review, item.replay, {
          status: "error",
          error: error instanceof Error ? error.message : String(error),
        });
        throw error;
      });
    review.replayLoadCache.set(item.replay, loadPromise);
    return loadPromise;
  }

  function preloadReplays(review: ActiveMechanicsReview, currentReplayId: string): void {
    if (review.preloading) {
      return;
    }
    review.preloading = true;
    void (async () => {
      try {
        for (const [replayId, item] of getMechanicsReviewReplayItems(review)) {
          if (replayId === currentReplayId) {
            continue;
          }
          const state = review.replayLoadStates.get(replayId);
          if (state?.status === "loaded" || state?.status === "loading") {
            continue;
          }
          try {
            await loadReplayBundle(item, review);
          } catch {
            // Background preload failures are rendered in the replay load window.
          }
        }
      } finally {
        review.preloading = false;
      }
    })();
  }

  async function activateItem(index: number): Promise<void> {
    const review = activeReview;
    const item = review?.manifest.items[index];
    if (!review || !item || review.loading) {
      return;
    }

    review.loading = true;
    review.currentIndex = index;
    render();
    setStatus(`Loading ${getMechanicsReviewItemLabel(item, index)}...`);

    try {
      const replayPlayer = options.getReplayPlayer();
      if (!replayPlayer || review.currentReplayId !== item.replay) {
        const source = createMechanicsReviewReplaySource(item, review);
        const replayBundlePromise = loadReplayBundle(item, review);
        await options.loadReplayBundleForDisplay(source, replayBundlePromise);
        review.currentReplayId = item.replay;
      }
      const activePlayer = options.getReplayPlayer();
      preloadReplays(review, item.replay);

      const startTime = Math.max(
        0,
        getMechanicsReviewBoundTime(item.start, activePlayer?.replay.frames),
      );
      const endTime = Math.min(
        activePlayer?.getState().duration ?? Number.POSITIVE_INFINITY,
        Math.max(startTime, getMechanicsReviewBoundTime(item.end, activePlayer?.replay.frames)),
      );
      if (!Number.isFinite(startTime) || !Number.isFinite(endTime) || endTime <= startTime) {
        throw new Error("Review item has an empty playback range.");
      }

      const playerId = getMechanicsReviewPlayerId(item);
      if (playerId && activePlayer?.replay.players.some((player) => player.id === playerId)) {
        activePlayer.setAttachedPlayer(playerId);
        activePlayer.setCameraViewMode("follow");
        options.clearFreeCameraPreset();
      }

      options.resetTransitionSkipControls();
      review.currentClip = { startTime, endTime };
      activePlayer?.setState({
        currentTime: startTime,
        playing: true,
        skipPostGoalTransitionsEnabled: false,
        skipKickoffsEnabled: false,
      });
      setStatus(`Playing ${startTime.toFixed(2)}s to ${endTime.toFixed(2)}s`);
    } catch (error) {
      console.error("Failed to activate mechanics review item:", error);
      review.currentClip = null;
      setStatus(error instanceof Error ? error.message : "Failed to load review item");
    } finally {
      review.loading = false;
      render();
    }
  }

  async function loadPlaylist(
    manifest: MechanicsReviewPlaylist,
    sourceUrl: string | null,
  ): Promise<void> {
    const replaysById = new Map<string, MechanicsReviewReplay>();
    for (const replay of manifest.replays ?? []) {
      replaysById.set(replay.id, replay);
    }

    activeReview = {
      manifest,
      sourceUrl,
      replaysById,
      replayLoadStates: new Map(),
      replayLoadCache: new Map(),
      currentIndex: 0,
      loading: false,
      preloading: false,
      currentReplayId: null,
      currentClip: null,
    };
    initializeMechanicsReviewReplayLoadStates(activeReview);
    options.showWindow("replay-loading");
    setStatus(manifest.label ? `Loaded ${manifest.label}.` : `Loaded review playlist.`);
    render();

    if (manifest.items.length > 0) {
      await activateItem(0);
    }
  }

  async function loadPlaylistFromUrl(urlText: string): Promise<void> {
    if (!urlText) {
      setStatus("Enter a review playlist URL.");
      return;
    }
    const url = resolveMechanicsReviewUrl(urlText, window.location.href);
    setStatus("Loading review playlist...");
    const response = await fetch(url);
    if (!response.ok) {
      const statusText = response.statusText ? ` ${response.statusText}` : "";
      throw new Error(
        `Failed to fetch review playlist from ${url} (${response.status}${statusText})`,
      );
    }
    const manifest = parseMechanicsReviewPlaylistJson(await response.text());
    await loadPlaylist(manifest, response.url || url);
  }

  function replayClip(): void {
    const clip = activeReview?.currentClip;
    const replayPlayer = options.getReplayPlayer();
    if (!clip || !replayPlayer) {
      return;
    }
    replayPlayer.setState({
      currentTime: clip.startTime,
      playing: true,
      skipPostGoalTransitionsEnabled: false,
      skipKickoffsEnabled: false,
    });
  }

  async function submitDecision(status: "confirmed" | "rejected" | "uncertain"): Promise<void> {
    const review = activeReview;
    const item = review?.manifest.items[review.currentIndex] ?? null;
    const endpoint = getMechanicsReviewDecisionEndpoint(item);
    if (!review || !item || !endpoint) {
      setStatus("Current review item has no review endpoint.");
      return;
    }

    setStatus(`Submitting ${formatMechanicsReviewStatus(status)}...`);
    const response = await fetch(endpoint, {
      method: "POST",
      headers: {
        "content-type": "application/json",
        ...mechanicsReviewAuthHeaders(),
      },
      credentials: "same-origin",
      body: JSON.stringify({ status }),
    });
    if (!response.ok) {
      let message = `${response.status}${response.statusText ? ` ${response.statusText}` : ""}`;
      try {
        const body = (await response.json()) as { error?: unknown };
        if (typeof body.error === "string") {
          message = body.error;
        }
      } catch {
        // Keep the HTTP status fallback.
      }
      setStatus(`Review failed: ${message}`);
      return;
    }

    item.meta = item.meta ?? {};
    item.meta.reviewStatus = status;
    setStatus(`Marked ${formatMechanicsReviewStatus(status)}.`);
    render();
  }

  function render(): void {
    const review = activeReview;
    const items = review?.manifest.items ?? [];
    const item = review ? (items[review.currentIndex] ?? null) : null;
    const hasItems = items.length > 0;

    elements.count.textContent = `${items.length} item${items.length === 1 ? "" : "s"}`;
    elements.index.textContent =
      hasItems && review ? `${review.currentIndex + 1} / ${items.length}` : "0 / 0";
    elements.title.textContent = item
      ? getMechanicsReviewItemLabel(item, review?.currentIndex ?? 0)
      : "No candidate selected";
    elements.mechanic.textContent = item ? getMechanicsReviewMechanicLabel(item) : "--";
    elements.player.textContent = item
      ? getMechanicsReviewPlayerName(item, options.getReplayPlayer()?.replay.players)
      : "--";
    elements.clip.textContent = item ? formatMechanicsReviewClipDetails(item) : "--";
    elements.event.textContent = item ? formatMechanicsReviewEventDetails(item) : "--";
    elements.reason.textContent = item?.meta?.reason ?? "--";
    elements.previous.disabled = !review || review.loading || review.currentIndex <= 0;
    elements.replay.disabled = !review || review.loading || !review.currentClip;
    elements.next.disabled =
      !review || review.loading || review.currentIndex >= items.length - 1;
    const decisionDisabled =
      !review || review.loading || getMechanicsReviewDecisionEndpoint(item) === null;
    elements.confirm.disabled = decisionDisabled;
    elements.reject.disabled = decisionDisabled;
    elements.uncertain.disabled = decisionDisabled;
    renderReplayLoads(review);

    elements.list.replaceChildren();
    if (!review || items.length === 0) {
      const empty = document.createElement("p");
      empty.className = "stat-window-empty";
      empty.textContent = "No review playlist loaded.";
      elements.list.append(empty);
      return;
    }

    items.forEach((candidate, index) => {
      const button = document.createElement("button");
      button.type = "button";
      button.className = "mechanics-review-item";
      button.dataset.active = index === review.currentIndex ? "true" : "false";
      button.disabled = review.loading;
      button.addEventListener("click", () => {
        void activateItem(index);
      });

      const title = document.createElement("span");
      title.textContent = getMechanicsReviewItemLabel(candidate, index);

      const meta = document.createElement("strong");
      meta.textContent = [
        getMechanicsReviewMechanicLabel(candidate),
        formatMechanicsReviewStatus(candidate.meta?.reviewStatus),
      ].join(" · ");

      button.append(title, meta);
      elements.list.append(button);
    });
  }

  return {
    clearCurrentClip() {
      if (activeReview) {
        activeReview.currentClip = null;
      }
    },
    clearCurrentReplay() {
      if (activeReview) {
        activeReview.currentClip = null;
        activeReview.currentReplayId = null;
        render();
      }
    },
    enforceClipBoundary(state) {
      const clip = activeReview?.currentClip;
      const replayPlayer = options.getReplayPlayer();
      if (!clip || !replayPlayer || boundaryGuard) {
        return false;
      }

      const beforeStart = state.currentTime < clip.startTime - 0.1;
      const atOrPastEnd = state.playing && state.currentTime >= clip.endTime - 0.025;
      if (!beforeStart && !atOrPastEnd) {
        return false;
      }

      boundaryGuard = true;
      try {
        replayPlayer.setState({
          currentTime: beforeStart ? clip.startTime : clip.endTime,
          playing: false,
          skipPostGoalTransitionsEnabled: false,
          skipKickoffsEnabled: false,
        });
        if (atOrPastEnd) {
          setStatus(`Finished clip at ${clip.endTime.toFixed(2)}s`);
        }
      } finally {
        boundaryGuard = false;
      }
      return true;
    },
    installListeners(signal) {
      elements.file.addEventListener(
        "change",
        async () => {
          const file = elements.file.files?.[0];
          if (!file) return;

          try {
            const manifest = parseMechanicsReviewPlaylistJson(await file.text());
            await loadPlaylist(manifest, null);
          } catch (error) {
            console.error("Failed to load mechanics review playlist:", error);
            setStatus(
              error instanceof Error ? error.message : "Failed to load mechanics review playlist",
            );
          } finally {
            elements.file.value = "";
          }
        },
        { signal },
      );

      elements.loadUrl.addEventListener(
        "click",
        () => {
          void loadPlaylistFromUrl(elements.url.value.trim()).catch((error) => {
            console.error("Failed to load mechanics review playlist URL:", error);
            setStatus(
              error instanceof Error
                ? error.message
                : "Failed to load mechanics review playlist URL",
            );
          });
        },
        { signal },
      );

      elements.previous.addEventListener(
        "click",
        () => {
          const review = activeReview;
          if (review) {
            void activateItem(Math.max(0, review.currentIndex - 1));
          }
        },
        { signal },
      );

      elements.replay.addEventListener("click", replayClip, { signal });

      elements.next.addEventListener(
        "click",
        () => {
          const review = activeReview;
          if (review) {
            void activateItem(Math.min(review.manifest.items.length - 1, review.currentIndex + 1));
          }
        },
        { signal },
      );

      elements.confirm.addEventListener(
        "click",
        () => {
          void submitDecision("confirmed");
        },
        { signal },
      );

      elements.reject.addEventListener(
        "click",
        () => {
          void submitDecision("rejected");
        },
        { signal },
      );

      elements.uncertain.addEventListener(
        "click",
        () => {
          void submitDecision("uncertain");
        },
        { signal },
      );
    },
    loadFromLocation(signal) {
      const reviewUrl = getMechanicsReviewUrlFromLocation();
      if (!reviewUrl) {
        return;
      }
      elements.url.value = reviewUrl;
      options.showWindow("mechanics-review");
      void loadPlaylistFromUrl(reviewUrl).catch((error) => {
        if (signal.aborted) {
          return;
        }
        console.error("Failed to load mechanics review playlist from URL:", error);
        setStatus(
          error instanceof Error
            ? error.message
            : "Failed to load mechanics review playlist from URL",
        );
      });
    },
    render,
    reset() {
      activeReview = null;
      boundaryGuard = false;
    },
  };
}
