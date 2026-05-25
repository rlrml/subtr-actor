import {
  createReplayReviewDataProviderFromLocation,
  mountReplayReview,
  type ReplayReviewDataProvider,
} from "../../stat-evaluation-player/src/lib.ts";

const root = document.querySelector("#app");
if (!(root instanceof HTMLElement)) {
  throw new Error("Missing #app mount element");
}

let provider: ReplayReviewDataProvider | null = null;
try {
  provider = createReplayReviewDataProviderFromLocation(window.location);
} catch (error) {
  console.error("Invalid replay URL:", error);
}

mountReplayReview(root, {
  provider,
});
