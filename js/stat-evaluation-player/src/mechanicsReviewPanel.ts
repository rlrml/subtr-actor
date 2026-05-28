import type { ReplayPlayer } from "@rlrml/player";
import {
  formatMechanicsReviewClipDetails,
  formatMechanicsReviewEventDetails,
  formatMechanicsReviewStatus,
  getMechanicsReviewDecisionEndpoint,
  getMechanicsReviewItemLabel,
  getMechanicsReviewMechanicLabel,
  getMechanicsReviewPlayerName,
  type ActiveMechanicsReview,
} from "./mechanicsReview.ts";
import type { MechanicsReviewElements } from "./mechanicsReviewController.ts";

interface MechanicsReviewPanelRenderOptions {
  readonly elements: MechanicsReviewElements;
  readonly review: ActiveMechanicsReview | null;
  activateItem(index: number): void;
  getReplayPlayer(): ReplayPlayer | null;
  renderReplayLoads(review: ActiveMechanicsReview | null): void;
}

export function renderMechanicsReviewPanel({
  elements,
  review,
  activateItem,
  getReplayPlayer,
  renderReplayLoads,
}: MechanicsReviewPanelRenderOptions): void {
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
    ? getMechanicsReviewPlayerName(item, getReplayPlayer()?.replay.players)
    : "--";
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
      activateItem(index);
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
