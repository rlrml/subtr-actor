import test from "node:test";
import assert from "node:assert/strict";

import {
  formatReplayLoadProgress,
  getReplayLoadCompletion,
  getReplayLoadPhase,
  getReplayLoadPhaseStates,
  listReplayLoadPhases,
} from "./replayLoadProgress.ts";
import { loadReplayBundleInWorker, waitForNextPaint } from "./replayLoader.ts";

function assertApproximatelyEqual(actual: number, expected: number): void {
  assert.ok(Math.abs(actual - expected) < 1e-9, `${actual} !== ${expected}`);
}

test("replay loading exposes each user-visible work phase", () => {
  assert.deepEqual(
    listReplayLoadPhases().map((phase) => ({
      stage: phase.stage,
      index: phase.index,
      total: phase.total,
      label: phase.label,
    })),
    [
      {
        stage: "validating",
        index: 1,
        total: 9,
        label: "Parse replay",
      },
      {
        stage: "processing",
        index: 2,
        total: 9,
        label: "Process replay frames",
      },
      {
        stage: "building-stats",
        index: 3,
        total: 9,
        label: "Build stats events",
      },
      {
        stage: "serializing-replay",
        index: 4,
        total: 9,
        label: "Serialize replay data",
      },
      {
        stage: "serializing-stats",
        index: 5,
        total: 9,
        label: "Serialize stats timeline",
      },
      {
        stage: "normalizing",
        index: 6,
        total: 9,
        label: "Normalize replay model",
      },
      {
        stage: "decoding-replay",
        index: 7,
        total: 9,
        label: "Decode replay data",
      },
      {
        stage: "decoding-stats",
        index: 8,
        total: 9,
        label: "Decode stats chunks",
      },
      {
        stage: "deriving-stats",
        index: 9,
        total: 9,
        label: "Derive stats snapshots",
      },
    ],
  );
});

test("getReplayLoadCompletion maps replay loading stages to monotonic overall progress", () => {
  assertApproximatelyEqual(getReplayLoadCompletion({ stage: "validating", progress: 0 }), 0.04);
  assertApproximatelyEqual(getReplayLoadCompletion({ stage: "processing", progress: 0 }), 0.08);
  assertApproximatelyEqual(getReplayLoadCompletion({ stage: "processing", progress: 0.5 }), 0.35);
  assertApproximatelyEqual(getReplayLoadCompletion({ stage: "processing", progress: 1 }), 0.62);
  assertApproximatelyEqual(
    getReplayLoadCompletion({ stage: "building-stats", progress: 0.5 }),
    0.66,
  );
  assertApproximatelyEqual(
    getReplayLoadCompletion({ stage: "serializing-replay", progress: 0.5 }),
    0.73,
  );
  assertApproximatelyEqual(
    getReplayLoadCompletion({ stage: "serializing-stats", progress: 0.5 }),
    0.81,
  );
  assertApproximatelyEqual(getReplayLoadCompletion({ stage: "normalizing", progress: 0.6 }), 0.89);
  assertApproximatelyEqual(
    getReplayLoadCompletion({ stage: "decoding-replay", progress: 1 }),
    0.94,
  );
  assertApproximatelyEqual(getReplayLoadCompletion({ stage: "decoding-stats", progress: 1 }), 0.96);
  assertApproximatelyEqual(
    getReplayLoadCompletion({ stage: "deriving-stats", progress: 0.5 }),
    0.98,
  );
  assertApproximatelyEqual(getReplayLoadCompletion({ stage: "deriving-stats", progress: 1 }), 1);
});

test("getReplayLoadCompletion clamps processing progress into the expected range", () => {
  assertApproximatelyEqual(getReplayLoadCompletion({ stage: "processing", progress: -1 }), 0.08);
  assertApproximatelyEqual(getReplayLoadCompletion({ stage: "processing", progress: 2 }), 0.62);
});

test("legacy stats timeline progress maps to the newer concrete phases", () => {
  assert.deepEqual(getReplayLoadPhase({ stage: "stats-timeline", progress: 0.25 }), {
    stage: "building-stats",
    index: 3,
    total: 9,
    label: "Build stats events",
  });
  assert.deepEqual(getReplayLoadPhase({ stage: "stats-timeline", progress: 0.42 }), {
    stage: "serializing-replay",
    index: 4,
    total: 9,
    label: "Serialize replay data",
  });
  assert.deepEqual(getReplayLoadPhase({ stage: "stats-timeline", progress: 0.75 }), {
    stage: "serializing-stats",
    index: 5,
    total: 9,
    label: "Serialize stats timeline",
  });
});

test("getReplayLoadPhaseStates marks prior detailed phases complete", () => {
  const states = getReplayLoadPhaseStates({
    stage: "decoding-stats",
    processedChunks: 2,
    totalChunks: 4,
    progress: 0.5,
  });

  assert.deepEqual(
    states.map((state) => ({
      stage: state.stage,
      state: state.state,
      completion: state.completion,
      indeterminate: state.indeterminate,
    })),
    [
      {
        stage: "validating",
        state: "complete",
        completion: 1,
        indeterminate: false,
      },
      {
        stage: "processing",
        state: "complete",
        completion: 1,
        indeterminate: false,
      },
      {
        stage: "building-stats",
        state: "complete",
        completion: 1,
        indeterminate: false,
      },
      {
        stage: "serializing-replay",
        state: "complete",
        completion: 1,
        indeterminate: false,
      },
      {
        stage: "serializing-stats",
        state: "complete",
        completion: 1,
        indeterminate: false,
      },
      {
        stage: "normalizing",
        state: "complete",
        completion: 1,
        indeterminate: false,
      },
      {
        stage: "decoding-replay",
        state: "complete",
        completion: 1,
        indeterminate: false,
      },
      {
        stage: "decoding-stats",
        state: "active",
        completion: 0.5,
        indeterminate: false,
      },
      {
        stage: "deriving-stats",
        state: "pending",
        completion: 0,
        indeterminate: false,
      },
    ],
  );
});

test("phase states stay monotonic through the worker and main thread load sequence", () => {
  const progressSequence = [
    { stage: "validating", progress: 0 },
    { stage: "processing", progress: 0 },
    { stage: "processing", progress: 1 },
    { stage: "stats-timeline", progress: 0.25 },
    { stage: "stats-timeline", progress: 0.42 },
    { stage: "stats-timeline", progress: 0.75 },
    { stage: "normalizing", progress: 0 },
    { stage: "normalizing", progress: 1 },
    { stage: "decoding-replay", progress: 0 },
    { stage: "decoding-replay", progress: 1 },
    { stage: "decoding-stats", progress: 0 },
    { stage: "decoding-stats", progress: 1 },
    { stage: "deriving-stats", progress: 0 },
    { stage: "deriving-stats", progress: 1 },
  ] as const;
  const previousCompletionByStage = new Map<string, number>();

  for (const progress of progressSequence) {
    for (const state of getReplayLoadPhaseStates(progress)) {
      const previousCompletion = previousCompletionByStage.get(state.stage) ?? 0;
      assert.ok(
        state.completion >= previousCompletion,
        `${state.stage} regressed from ${previousCompletion} to ${state.completion}`,
      );
      previousCompletionByStage.set(state.stage, state.completion);
    }
  }
});

test("getReplayLoadPhaseStates keeps legacy stats timeline indeterminate without progress", () => {
  const states = getReplayLoadPhaseStates({ stage: "stats-timeline" });

  assert.equal(states[2]?.stage, "building-stats");
  assert.equal(states[2]?.state, "active");
  assert.equal(states[2]?.completion, 1);
  assert.equal(states[2]?.indeterminate, true);
  assert.equal(states[3]?.state, "pending");
});

test("formatReplayLoadProgress reports detailed stats and decode work", () => {
  assert.equal(
    formatReplayLoadProgress({
      stage: "building-stats",
      processedFrames: 3,
      totalFrames: 10,
      progress: 0.3,
    }),
    "Building stats events... 30% (3/10)",
  );
  assert.equal(
    formatReplayLoadProgress({ stage: "stats-timeline", progress: 0.42 }),
    "Serializing replay data... 35%",
  );
  assert.equal(
    formatReplayLoadProgress({
      stage: "decoding-stats",
      processedChunks: 2,
      totalChunks: 4,
      progress: 0.5,
    }),
    "Decoding stats chunks... 50% (2/4)",
  );
  assert.equal(
    formatReplayLoadProgress({ stage: "deriving-stats", progress: 0.25 }),
    "Deriving stats snapshots... 25%",
  );
  assert.equal(
    formatReplayLoadProgress({ stage: "normalizing", progress: 1 }),
    "Normalizing replay model... 100%",
  );
});

test("waitForNextPaint falls back when animation frames do not fire", async () => {
  const originalRequestAnimationFrame = globalThis.requestAnimationFrame;
  Object.defineProperty(globalThis, "requestAnimationFrame", {
    configurable: true,
    value: () => 1,
  });

  try {
    await waitForNextPaint(1);
  } finally {
    Object.defineProperty(globalThis, "requestAnimationFrame", {
      configurable: true,
      value: originalRequestAnimationFrame,
    });
  }
});

test("loadReplayBundleInWorker rejects malformed worker decode payloads", async () => {
  const originalWorker = globalThis.Worker;
  const invalidJsonBuffer = new TextEncoder().encode("{").buffer;

  class MalformedDoneWorker {
    onmessage: ((event: MessageEvent<unknown>) => void) | null = null;
    onerror: ((event: ErrorEvent) => void) | null = null;

    constructor(_url: URL, _options: WorkerOptions) {}

    postMessage(): void {
      queueMicrotask(() => {
        this.onmessage?.({
          data: {
            type: "done",
            replayBuffer: invalidJsonBuffer,
            statsTimelineParts: {
              configBuffer: new ArrayBuffer(0),
              replayMetaBuffer: new ArrayBuffer(0),
              eventsBuffer: new ArrayBuffer(0),
              frameChunkBuffers: [],
            },
          },
        } as MessageEvent<unknown>);
      });
    }

    terminate(): void {}
  }

  Object.defineProperty(globalThis, "Worker", {
    configurable: true,
    value: MalformedDoneWorker,
  });

  try {
    await assert.rejects(
      () => loadReplayBundleInWorker(new Uint8Array([1, 2, 3])),
      SyntaxError,
    );
  } finally {
    Object.defineProperty(globalThis, "Worker", {
      configurable: true,
      value: originalWorker,
    });
  }
});
