import assert from "node:assert/strict";
import test from "node:test";
import { PlaylistLoadCache } from "@rlrml/player";
import type { ActiveMechanicsReview } from "./mechanicsReview.ts";
import { MechanicsReviewReplayLoadsController } from "./mechanicsReviewReplayLoads.ts";

test("review replay cache retains active and next replays and resets evicted load state", async () => {
  const review: ActiveMechanicsReview = {
    manifest: {
      items: Array.from({ length: 8 }, (_, index) => ({
        replay: String(index),
        start: { kind: "time", value: 0 },
        end: { kind: "time", value: 10 },
      })),
    },
    sourceUrl: null,
    replaysById: new Map(),
    replayLoadStates: new Map(),
    replayLoadCache: new PlaylistLoadCache(),
    currentIndex: 0,
    loading: false,
    preloading: false,
    currentReplayId: null,
    currentClip: null,
  };
  const controller = new MechanicsReviewReplayLoadsController({
    elements: {} as never,
    isActiveReview: () => false,
    onActiveLoadProgress: () => {},
  });
  controller.initialize(review);
  // Pending loads avoid invoking the browser worker while retaining real cache behavior.
  for (const id of ["0", "1"]) {
    void review.replayLoadCache.load({ id, load: () => new Promise(() => {}) });
  }
  void controller.loadBundle(review.manifest.items[0]!, review);
  for (const id of ["2", "3", "4", "5", "6", "7"]) {
    void review.replayLoadCache.load({ id, load: () => new Promise(() => {}) });
  }
  assert.equal(review.replayLoadCache.has("0"), true);
  assert.equal(review.replayLoadCache.has("1"), true);
  assert.equal(review.replayLoadCache.has("2"), false);
  assert.equal(review.replayLoadStates.get("2")?.status, "idle");
  assert.equal(review.replayLoadCache.getStates().length, 5);
  review.replayLoadCache.clear();
  assert.ok([...review.replayLoadStates.values()].every((state) => state.status === "idle"));
});
