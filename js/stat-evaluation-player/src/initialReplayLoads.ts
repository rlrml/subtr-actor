import type { MechanicsReviewWindowController } from "./mechanicsReviewWindow.ts";
import { getMechanicsReviewUrlFromLocation } from "./mechanicsReview.ts";
import type { ReplayLoadBundle } from "./replayLoader.ts";
import { getReplayFetchRequestFromSearch } from "./replayUrl.ts";
import { createRemoteReplaySource, type ReplayInputSource } from "./replaySources.ts";

export interface InitialReplayLoadsOptions {
  readonly signal: AbortSignal;
  readonly location: Location;
  readonly statusReadout: HTMLElement;
  readonly initialBundle?: ReplayLoadBundle | Promise<ReplayLoadBundle>;
  readonly initialReplayName?: string;
  readonly loadFromLocation?: boolean;
  loadReplay(source: ReplayInputSource): Promise<void>;
  loadReplayBundleForDisplay(
    source: ReplayInputSource,
    bundlePromise: Promise<ReplayLoadBundle>,
  ): Promise<void>;
  getMechanicsReviewController(): MechanicsReviewWindowController | null;
  showMechanicsReviewWindow(): void;
}

export function loadReplayFromLocation(options: InitialReplayLoadsOptions): void {
  let replayRequest;
  try {
    replayRequest = getReplayFetchRequestFromSearch(options.location.search, options.location.href);
  } catch (error) {
    console.error("Invalid replay URL:", error);
    options.statusReadout.textContent =
      error instanceof Error ? error.message : "Invalid replay URL";
    return;
  }

  if (!replayRequest) {
    return;
  }

  void options
    .loadReplay(createRemoteReplaySource(replayRequest, options.signal))
    .catch((error) => {
      if (options.signal.aborted) {
        return;
      }
      console.error("Failed to load replay URL:", error);
      options.statusReadout.textContent =
        error instanceof Error ? error.message : "Failed to load replay URL";
    });
}

export function installInitialReplayLoads(options: InitialReplayLoadsOptions): void {
  if (options.initialBundle) {
    void options
      .loadReplayBundleForDisplay(
        {
          name: options.initialReplayName ?? "replay",
          preparingStatus: "Preparing replay...",
          async readBytes() {
            throw new Error("Replay bytes are not available for this preloaded replay");
          },
        },
        Promise.resolve(options.initialBundle),
      )
      .catch((error) => {
        if (options.signal.aborted) {
          return;
        }
        console.error("Failed to load preprocessed replay bundle:", error);
        options.statusReadout.textContent =
          error instanceof Error ? error.message : "Failed to load preprocessed replay bundle";
      });
  } else if (options.loadFromLocation !== false) {
    loadReplayFromLocation(options);
  }

  const reviewUrl = getMechanicsReviewUrlFromLocation();
  if (!reviewUrl) {
    return;
  }

  const mechanicsReviewController = options.getMechanicsReviewController();
  mechanicsReviewController?.setUrl(reviewUrl);
  options.showMechanicsReviewWindow();
  void mechanicsReviewController?.loadPlaylistFromUrl(reviewUrl).catch((error) => {
    if (options.signal.aborted) {
      return;
    }
    console.error("Failed to load mechanics review playlist from URL:", error);
    mechanicsReviewController?.setStatus(
      error instanceof Error ? error.message : "Failed to load mechanics review playlist from URL",
    );
  });
}
