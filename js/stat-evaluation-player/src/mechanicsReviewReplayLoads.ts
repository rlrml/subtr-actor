import {
  mechanicsReviewReplayLoadProgressValue,
  mechanicsReviewReplayLoadStatusText,
  type ActiveMechanicsReview,
} from "./mechanicsReview.ts";
import type { MechanicsReviewElements } from "./mechanicsReviewController.ts";

export function renderMechanicsReviewReplayLoads(
  elements: MechanicsReviewElements,
  review: ActiveMechanicsReview | null,
): void {
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
  elements.replayLoadSummary.textContent = summaryText;
  elements.replayLoadingSummary.textContent = summaryText;
  elements.replayLoadingActive.textContent =
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

  elements.replayLoadingList.replaceChildren();
  if (!review || states.length === 0) {
    const empty = document.createElement("p");
    empty.className = "stat-window-empty";
    empty.textContent = "No replay sources.";
    elements.replayLoadingList.append(empty);
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
    status.textContent = mechanicsReviewReplayLoadStatusText(state);

    const progress = document.createElement("div");
    progress.className = "mechanics-review-replay-load-progress";
    const bar = document.createElement("span");
    bar.style.width = `${Math.round(mechanicsReviewReplayLoadProgressValue(state) * 100)}%`;
    progress.append(bar);

    row.append(main, status, progress);
    elements.replayLoadingList.append(row);
  }
}
