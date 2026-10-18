import assert from "node:assert/strict";
import test from "node:test";
import type { ReplayLoadBundle } from "./replayLoader.ts";
import {
  loadReplayBundleForDisplay,
  type ReplayDisplayRuntimeOptions,
} from "./replayDisplayRuntime.ts";
import type { ReplayInputSource } from "./replaySources.ts";

function deferred<T>(): {
  promise: Promise<T>;
  resolve(value: T): void;
  reject(error: unknown): void;
} {
  let resolve!: (value: T) => void;
  let reject!: (error: unknown) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

const flush = () => new Promise((resolve) => setImmediate(resolve));

const source: ReplayInputSource = {
  name: "replay",
  preparingStatus: "Preparing replay...",
  async readBytes() {
    throw new Error("unused");
  },
};

function bundle(statsTimeline: string): ReplayLoadBundle {
  return {
    replay: { players: [], frameCount: 1 },
    raw: {
      meta: { team_zero: [], team_one: [] },
      frame_data: { metadata_frames: [], ball_data: { frames: [] }, players: [] },
    },
    statsTimeline,
    statsFrameLookup: null,
  } as unknown as ReplayLoadBundle;
}

function createHarness(replaceReplay: () => Promise<void>) {
  const calls: string[] = [];
  const fileInput = { disabled: false };
  const display = { statsTimeline: null as unknown, transportEnabled: true };
  const player = {
    pause() {},
    replaceReplay,
    subscribe: () => () => {},
    getState: () => ({ frameIndex: 0 }),
  };
  const target: Record<string | symbol, unknown> = {
    elements: {
      fileInput,
      viewport: {},
      emptyState: { hidden: false },
      statusReadout: { textContent: "" },
      playersReadout: { textContent: "" },
      framesReadout: { textContent: "" },
    },
    getReplayLoadModal: () => null,
    getReplayPlayer: () => player,
    getCameraControlsController: () => null,
    getUnsubscribe: () => null,
    getInitialConfig: () => null,
    setStatsTimeline(value: unknown) {
      display.statsTimeline = value;
    },
    setTransportEnabled(enabled: boolean) {
      calls.push("setTransportEnabled");
      display.transportEnabled = enabled;
    },
  };
  const options = new Proxy(target, {
    get(target, key) {
      return key in target ? target[key] : () => calls.push(String(key));
    },
  }) as unknown as ReplayDisplayRuntimeOptions;
  return { calls, display, fileInput, options };
}

test("a superseded display load discards its result", async () => {
  const { calls, fileInput, options } = createHarness(async () => {
    calls.push("replaceReplay");
  });

  const older = deferred<ReplayLoadBundle>();
  const newer = deferred<ReplayLoadBundle>();
  const olderLoad = loadReplayBundleForDisplay(source, older.promise, options);
  const newerLoad = loadReplayBundleForDisplay(source, newer.promise, options);

  older.resolve(bundle("older"));
  assert.equal(await olderLoad, false);
  assert.deepEqual(calls, ["setTransportEnabled", "setTransportEnabled"]);
  assert.equal(fileInput.disabled, true);

  newer.reject(new Error("newer failed"));
  await assert.rejects(newerLoad, /newer failed/);
  assert.equal(fileInput.disabled, false);
});

test("an in-place replacement is adopted when the load superseding it fails", async () => {
  const replacing = deferred<void>();
  const { calls, display, fileInput, options } = createHarness(() => replacing.promise);

  const olderLoad = loadReplayBundleForDisplay(source, Promise.resolve(bundle("older")), options);
  await flush();
  assert.ok(calls.includes("teardownActiveModules"));
  assert.equal(display.statsTimeline, null);

  const newer = deferred<ReplayLoadBundle>();
  const newerLoad = loadReplayBundleForDisplay(source, newer.promise, options);
  newer.reject(new Error("newer failed"));
  await assert.rejects(newerLoad, /newer failed/);

  replacing.resolve();
  assert.equal(await olderLoad, true);
  assert.equal(display.statsTimeline, "older");
  assert.equal(display.transportEnabled, true);
  assert.ok(calls.lastIndexOf("setupActiveModules") > calls.lastIndexOf("teardownActiveModules"));
  assert.equal(fileInput.disabled, false);
});
