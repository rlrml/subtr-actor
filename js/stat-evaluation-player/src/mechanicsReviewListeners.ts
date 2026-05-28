import {
  parseMechanicsReviewPlaylistJson,
  type MechanicsReviewPlaylist,
} from "./mechanicsReview.ts";
import type { MechanicsReviewElements } from "./mechanicsReviewController.ts";

type MechanicsReviewDecisionStatus = "confirmed" | "rejected" | "uncertain";

interface MechanicsReviewNavigationState {
  currentIndex: number;
  itemCount: number;
}

interface MechanicsReviewListenersOptions {
  readonly elements: MechanicsReviewElements;
  readonly signal: AbortSignal;
  activateItem(index: number): Promise<void>;
  getNavigationState(): MechanicsReviewNavigationState | null;
  loadPlaylist(manifest: MechanicsReviewPlaylist, sourceUrl: string | null): Promise<void>;
  loadPlaylistFromUrl(urlText: string): Promise<void>;
  replayClip(): void;
  setStatus(message: string): void;
  submitDecision(status: MechanicsReviewDecisionStatus): Promise<void>;
}

export function installMechanicsReviewListeners(options: MechanicsReviewListenersOptions): void {
  const { elements, signal } = options;
  elements.file.addEventListener(
    "change",
    async () => {
      const file = elements.file.files?.[0];
      if (!file) return;

      try {
        const manifest = parseMechanicsReviewPlaylistJson(await file.text());
        await options.loadPlaylist(manifest, null);
      } catch (error) {
        console.error("Failed to load mechanics review playlist:", error);
        options.setStatus(
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
      void options.loadPlaylistFromUrl(elements.url.value.trim()).catch((error) => {
        console.error("Failed to load mechanics review playlist URL:", error);
        options.setStatus(
          error instanceof Error ? error.message : "Failed to load mechanics review playlist URL",
        );
      });
    },
    { signal },
  );

  elements.previous.addEventListener(
    "click",
    () => {
      const state = options.getNavigationState();
      if (state) {
        void options.activateItem(Math.max(0, state.currentIndex - 1));
      }
    },
    { signal },
  );

  elements.replay.addEventListener("click", options.replayClip, { signal });

  elements.next.addEventListener(
    "click",
    () => {
      const state = options.getNavigationState();
      if (state) {
        void options.activateItem(Math.min(state.itemCount - 1, state.currentIndex + 1));
      }
    },
    { signal },
  );

  elements.confirm.addEventListener(
    "click",
    () => {
      void options.submitDecision("confirmed");
    },
    { signal },
  );

  elements.reject.addEventListener(
    "click",
    () => {
      void options.submitDecision("rejected");
    },
    { signal },
  );

  elements.uncertain.addEventListener(
    "click",
    () => {
      void options.submitDecision("uncertain");
    },
    { signal },
  );
}
