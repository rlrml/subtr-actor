import type { ReplayPlayer, ReplayPlayerState } from "@rlrml/player";
import {
  formatMechanicsReviewClipDetails,
  formatMechanicsReviewEventDetails,
  getMechanicsReviewCategoryLabel,
  getMechanicsReviewItemLabel,
  getMechanicsReviewMechanicLabel,
  getMechanicsReviewPlayerId,
  parseMechanicsReviewPlaylistJson,
  resolveMechanicsReviewBoundTime,
  resolveMechanicsReviewTargetTime,
  resolveMechanicsReviewUrl,
  type ActiveMechanicsReview,
  type MechanicsReviewItem,
  type MechanicsReviewPlaybackBound,
  type MechanicsReviewPlaylist,
  type MechanicsReviewReplay,
  type MechanicsReviewTimeBase,
} from "./mechanicsReview.ts";
import type { MechanicsReviewReplayLoadsController } from "./mechanicsReviewReplayLoads.ts";
import type { ReplayLoadBundle } from "./replayLoader.ts";

type MechanicsReviewDecision = "confirmed" | "rejected" | "uncertain";

export interface MechanicsReviewReplaySource {
  name: string;
  preparingStatus: string;
  readBytes(): Promise<Uint8Array>;
}

export interface MechanicsReviewWindowElements {
  readonly file: HTMLInputElement;
  readonly url: HTMLInputElement;
  readonly loadUrl: HTMLButtonElement;
  readonly status: HTMLElement;
  readonly index: HTMLElement;
  readonly title: HTMLElement;
  readonly mechanic: HTMLElement;
  readonly player: HTMLElement;
  readonly clip: HTMLElement;
  readonly event: HTMLElement;
  readonly reason: HTMLElement;
  readonly previous: HTMLButtonElement;
  readonly replay: HTMLButtonElement;
  readonly next: HTMLButtonElement;
  readonly confirm: HTMLButtonElement;
  readonly reject: HTMLButtonElement;
  readonly uncertain: HTMLButtonElement;
  readonly count: HTMLElement;
  readonly list: HTMLDivElement;
}

export interface MechanicsReviewWindowOptions {
  readonly elements: MechanicsReviewWindowElements;
  readonly replayLoads: MechanicsReviewReplayLoadsController;
  getReplayPlayer(): ReplayPlayer | null;
  clearFreeCameraPreset(): void;
  resetReplayTransitionControls(): void;
  activateTimelineSource(item: MechanicsReviewItem): void;
  loadReplayBundleForDisplay(
    source: MechanicsReviewReplaySource,
    bundlePromise: Promise<ReplayLoadBundle>,
  ): Promise<void>;
  showReplayLoadingWindow(): void;
}

export class MechanicsReviewWindowController {
  private activeReview: ActiveMechanicsReview | null = null;
  private boundaryGuard = false;

  constructor(private readonly options: MechanicsReviewWindowOptions) {}

  get review(): ActiveMechanicsReview | null {
    return this.activeReview;
  }

  reset(): void {
    this.activeReview = null;
    this.boundaryGuard = false;
  }

  setUrl(value: string): void {
    this.options.elements.url.value = value;
  }

  clearCurrentClip({ resetReplayId = false, render = false } = {}): void {
    if (!this.activeReview) {
      return;
    }
    this.activeReview.currentClip = null;
    if (resetReplayId) {
      this.activeReview.currentReplayId = null;
    }
    if (render) {
      this.render();
    }
  }

  setStatus(message: string): void {
    this.options.elements.status.textContent = message;
  }

  installEventListeners(signal: AbortSignal): void {
    const { elements } = this.options;
    elements.file.addEventListener(
      "change",
      async () => {
        const file = elements.file.files?.[0];
        if (!file) return;

        try {
          const manifest = parseMechanicsReviewPlaylistJson(await file.text());
          await this.loadPlaylist(manifest, null);
        } catch (error) {
          console.error("Failed to load review playlist:", error);
          this.setStatus(error instanceof Error ? error.message : "Failed to load review playlist");
        } finally {
          elements.file.value = "";
        }
      },
      { signal },
    );

    elements.loadUrl.addEventListener(
      "click",
      () => {
        void this.loadPlaylistFromUrl(elements.url.value.trim()).catch((error) => {
          console.error("Failed to load review playlist URL:", error);
          this.setStatus(
            error instanceof Error ? error.message : "Failed to load review playlist URL",
          );
        });
      },
      { signal },
    );

    elements.previous.addEventListener(
      "click",
      () => {
        const review = this.activeReview;
        if (review) {
          void this.activateItem(Math.max(0, review.currentIndex - 1));
        }
      },
      { signal },
    );

    elements.replay.addEventListener("click", () => this.replayClip(), { signal });

    elements.next.addEventListener(
      "click",
      () => {
        const review = this.activeReview;
        if (review) {
          void this.activateItem(
            Math.min(review.manifest.items.length - 1, review.currentIndex + 1),
          );
        }
      },
      { signal },
    );

    elements.confirm.addEventListener(
      "click",
      () => {
        void this.submitDecision("confirmed");
      },
      { signal },
    );
    elements.reject.addEventListener(
      "click",
      () => {
        void this.submitDecision("rejected");
      },
      { signal },
    );
    elements.uncertain.addEventListener(
      "click",
      () => {
        void this.submitDecision("uncertain");
      },
      { signal },
    );
  }

  render(): void {
    const { elements } = this.options;
    const review = this.activeReview;
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
    elements.player.textContent = item ? this.getPlayerName(item) : "--";
    elements.clip.textContent = item ? formatMechanicsReviewClipDetails(item) : "--";
    elements.event.textContent = item ? formatMechanicsReviewEventDetails(item) : "--";
    elements.reason.textContent = item?.meta?.reason ?? "--";
    elements.previous.disabled = !review || review.loading || review.currentIndex <= 0;
    elements.replay.disabled = !review || review.loading || !review.currentClip;
    elements.next.disabled = !review || review.loading || review.currentIndex >= items.length - 1;
    const decisionDisabled =
      !review || review.loading || getMechanicsReviewDecisionEndpoint(item) === null;
    elements.confirm.disabled = decisionDisabled;
    elements.reject.disabled = decisionDisabled;
    elements.uncertain.disabled = decisionDisabled;
    this.options.replayLoads.render(review);

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
        void this.activateItem(index);
      });

      const title = document.createElement("span");
      title.textContent = getMechanicsReviewItemLabel(candidate, index);

      const meta = document.createElement("strong");
      meta.textContent =
        [
          getMechanicsReviewCategoryLabel(candidate),
          getMechanicsReviewMechanicLabel(candidate),
          this.getPlayerName(candidate),
          formatMechanicsReviewStatus(candidate.meta?.reviewStatus),
        ]
          .filter((part) => part && part !== "--")
          .join(" · ") || "--";

      button.append(title, meta);
      elements.list.append(button);
    });
  }

  async loadPlaylist(manifest: MechanicsReviewPlaylist, sourceUrl: string | null): Promise<void> {
    const replaysById = new Map<string, MechanicsReviewReplay>();
    for (const replay of manifest.replays ?? []) {
      replaysById.set(replay.id, replay);
    }

    this.activeReview = {
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
    this.options.replayLoads.initialize(this.activeReview);
    this.options.showReplayLoadingWindow();
    this.setStatus(manifest.label ? `Loaded ${manifest.label}.` : `Loaded review playlist.`);
    this.render();

    if (manifest.items.length > 0) {
      await this.activateItem(0);
    }
  }

  async loadPlaylistFromUrl(urlText: string): Promise<void> {
    if (!urlText) {
      this.setStatus("Enter a review playlist URL.");
      return;
    }
    const url = resolveMechanicsReviewUrl(urlText, window.location.href);
    this.setStatus("Loading review playlist...");
    const response = await fetch(url);
    if (!response.ok) {
      const statusText = response.statusText ? ` ${response.statusText}` : "";
      throw new Error(
        `Failed to fetch review playlist from ${url} (${response.status}${statusText})`,
      );
    }
    const manifest = parseMechanicsReviewPlaylistJson(await response.text());
    await this.loadPlaylist(manifest, response.url || url);
  }

  async activateItem(index: number): Promise<void> {
    const review = this.activeReview;
    const item = review?.manifest.items[index];
    if (!review || !item || review.loading) {
      return;
    }

    review.loading = true;
    review.currentIndex = index;
    this.render();
    this.setStatus(`Loading ${getMechanicsReviewItemLabel(item, index)}...`);

    try {
      const replayPlayer = this.options.getReplayPlayer();
      if (!replayPlayer || review.currentReplayId !== item.replay) {
        const source = this.options.replayLoads.createReplaySource(item, review);
        const replayBundlePromise = this.options.replayLoads.loadBundle(item, review);
        await this.options.loadReplayBundleForDisplay(source, replayBundlePromise);
        review.currentReplayId = item.replay;
      }
      this.options.replayLoads.preload(review, index);

      const timeBase = review.manifest.playback?.timeBase;
      const startTime = Math.max(0, this.getBoundTime(item, item.start, timeBase));
      const endTime = Math.min(
        this.options.getReplayPlayer()?.getState().duration ?? Number.POSITIVE_INFINITY,
        Math.max(startTime, this.getBoundTime(item, item.end, timeBase)),
      );
      if (!Number.isFinite(startTime) || !Number.isFinite(endTime) || endTime <= startTime) {
        throw new Error("Review item has an empty playback range.");
      }

      const playerId = getMechanicsReviewPlayerId(item);
      const activeReplayPlayer = this.options.getReplayPlayer();
      if (playerId && activeReplayPlayer?.replay.players.some((player) => player.id === playerId)) {
        activeReplayPlayer.setAttachedPlayer(playerId);
        activeReplayPlayer.setCameraViewMode("follow");
        this.options.clearFreeCameraPreset();
      }

      this.options.resetReplayTransitionControls();
      const targetTime =
        activeReplayPlayer === null
          ? null
          : resolveMechanicsReviewTargetTime(item, activeReplayPlayer.replay, timeBase);
      review.currentClip = { startTime, endTime, targetTime };
      this.options.activateTimelineSource(item);
      this.options.getReplayPlayer()?.setState({
        currentTime: startTime,
        playing: true,
        skipPostGoalTransitionsEnabled: false,
        skipKickoffsEnabled: false,
      });
      this.setStatus(
        targetTime === null
          ? `Playing ${startTime.toFixed(2)}s to ${endTime.toFixed(2)}s`
          : `Playing ${startTime.toFixed(2)}s to ${endTime.toFixed(2)}s; target ${targetTime.toFixed(2)}s`,
      );
    } catch (error) {
      console.error("Failed to activate mechanics review item:", error);
      review.currentClip = null;
      this.setStatus(error instanceof Error ? error.message : "Failed to load review item");
    } finally {
      review.loading = false;
      this.render();
    }
  }

  replayClip(): void {
    const clip = this.activeReview?.currentClip;
    const replayPlayer = this.options.getReplayPlayer();
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

  async submitDecision(status: MechanicsReviewDecision): Promise<void> {
    const review = this.activeReview;
    const item = review?.manifest.items[review.currentIndex] ?? null;
    const endpoint = getMechanicsReviewDecisionEndpoint(item);
    if (!review || !item || !endpoint) {
      this.setStatus("Current review item has no review endpoint.");
      return;
    }

    this.setStatus(`Submitting ${formatMechanicsReviewStatus(status)}...`);
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
      this.setStatus(`Review failed: ${message}`);
      return;
    }

    item.meta = item.meta ?? {};
    item.meta.reviewStatus = status;
    this.setStatus(`Marked ${formatMechanicsReviewStatus(status)}.`);
    this.render();
  }

  enforceClipBoundary(state: ReplayPlayerState): boolean {
    const clip = this.activeReview?.currentClip;
    const replayPlayer = this.options.getReplayPlayer();
    if (!clip || !replayPlayer || this.boundaryGuard) {
      return false;
    }

    const beforeStart = state.currentTime < clip.startTime - 0.1;
    const atOrPastEnd = state.playing && state.currentTime >= clip.endTime - 0.025;
    if (!beforeStart && !atOrPastEnd) {
      return false;
    }

    this.boundaryGuard = true;
    try {
      replayPlayer.setState({
        currentTime: beforeStart ? clip.startTime : clip.endTime,
        playing: false,
        skipPostGoalTransitionsEnabled: false,
        skipKickoffsEnabled: false,
      });
      if (atOrPastEnd) {
        this.setStatus(`Finished clip at ${clip.endTime.toFixed(2)}s`);
      }
    } finally {
      this.boundaryGuard = false;
    }
    return true;
  }

  private getBoundTime(
    item: MechanicsReviewItem,
    bound: MechanicsReviewPlaybackBound,
    timeBase: MechanicsReviewTimeBase | undefined,
  ): number {
    const replayPlayer = this.options.getReplayPlayer();
    if (!replayPlayer) {
      return bound.kind === "time" ? bound.value : 0;
    }
    return resolveMechanicsReviewBoundTime(item, bound, replayPlayer.replay, timeBase);
  }

  private getPlayerName(item: MechanicsReviewItem): string {
    if (typeof item.meta?.playerName === "string" && item.meta.playerName.trim()) {
      return item.meta.playerName;
    }
    const playerId = getMechanicsReviewPlayerId(item);
    return playerId
      ? (this.options.getReplayPlayer()?.replay.players.find((player) => player.id === playerId)
          ?.name ?? playerId)
      : "--";
  }
}

function formatMechanicsReviewStatus(value: unknown): string {
  return typeof value === "string" && value.trim() ? value.replaceAll("_", " ") : "unreviewed";
}

function getMechanicsReviewDecisionEndpoint(item: MechanicsReviewItem | null): string | null {
  if (!item) {
    return null;
  }
  if (typeof item.meta?.reviewEndpoint === "string" && item.meta.reviewEndpoint) {
    return item.meta.reviewEndpoint;
  }
  const eventId =
    typeof item.meta?.eventId === "string" && item.meta.eventId ? item.meta.eventId : item.id;
  return eventId ? `/api/v1/events/${encodeURIComponent(eventId)}/reviews` : null;
}

function mechanicsReviewAuthHeaders(): Record<string, string> {
  const params = new URLSearchParams(window.location.search);
  const token =
    params.get("reviewToken") ??
    params.get("token") ??
    window.localStorage.getItem("rocket_sense_access_token");
  return token ? { Authorization: `Bearer ${token}` } : {};
}

export function createMechanicsReviewWindowController(
  options: MechanicsReviewWindowOptions,
): MechanicsReviewWindowController {
  return new MechanicsReviewWindowController(options);
}
