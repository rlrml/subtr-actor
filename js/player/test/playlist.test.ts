import test from "node:test";
import assert from "node:assert/strict";

import {
  PlaylistLoadCache,
  PlaylistSession,
  createFullReplayPlaylistItem,
  createReplaySource,
  parsePlaylistManifest,
  resolvePlaylistItem,
} from "../src/lib";
import type { LoadedReplay, ReplayModel } from "../src/lib";

test("playlist manifests preserve generic playback policy", () => {
  const manifest = parsePlaylistManifest({
    playback: {
      advanceMode: "manual",
      endMode: "loop",
    },
    replays: [
      {
        id: "replay-a",
        path: "a.replay",
      },
    ],
    items: [
      {
        replay: "replay-a",
        start: { kind: "time", value: 10 },
        end: { kind: "time", value: 20 },
      },
    ],
  });

  assert.deepEqual(manifest.playback, {
    advanceMode: "manual",
    endMode: "loop",
  });
});

test("playlist manifests preserve optional page metadata", () => {
  const manifest = parsePlaylistManifest({
    page: {
      next: "/api/playlists/example?page=2",
      previous: null,
      total: 250,
      count: 100,
      limit: 100,
      offset: 0,
    },
    items: [],
  });

  assert.deepEqual(manifest.page, {
    next: "/api/playlists/example?page=2",
    previous: undefined,
    total: 250,
    count: 100,
    limit: 100,
    offset: 0,
  });
});

test("playlist manifests reject invalid page metadata", () => {
  assert.throws(
    () =>
      parsePlaylistManifest({
        page: {
          total: -1,
        },
        items: [],
      }),
    /manifest\.page\.total/,
  );

  assert.throws(
    () =>
      parsePlaylistManifest({
        page: {
          next: 1,
        },
        items: [],
      }),
    /manifest\.page\.next/,
  );
});

test("playlist manifests reject invalid playback policies", () => {
  assert.throws(
    () =>
      parsePlaylistManifest({
        playback: {
          advanceMode: "sometimes",
        },
        items: [],
      }),
    /manifest\.playback\.advanceMode/,
  );

  assert.throws(
    () =>
      parsePlaylistManifest({
        playback: {
          endMode: "shuffle",
        },
        items: [],
      }),
    /manifest\.playback\.endMode/,
  );
});

test("playlist load cache supports custom loaded result types", async () => {
  const cache = new PlaylistLoadCache<{ replay: string; stats: string }>();
  let loadCount = 0;
  const source = {
    id: "bundle",
    async load() {
      loadCount += 1;
      return {
        replay: "normalized replay",
        stats: "stats timeline",
      };
    },
  };

  assert.deepEqual(await cache.load(source), {
    replay: "normalized replay",
    stats: "stats timeline",
  });
  assert.deepEqual(await cache.load(source), {
    replay: "normalized replay",
    stats: "stats timeline",
  });
  assert.equal(loadCount, 1);
});

test("playlist load cache exposes loading progress and loaded state", async () => {
  const cache = new PlaylistLoadCache<{ replay: string }>();
  const observed: string[] = [];
  cache.subscribe(() => {
    observed.push(cache.getState("tracked").status);
  });

  const loaded = await cache.load({
    id: "tracked",
    async load(context) {
      context?.updateProgress({
        stage: "fetching",
        processedBytes: 4,
        totalBytes: 8,
        progress: 0.5,
      });
      return { replay: "normalized replay" };
    },
  });

  assert.deepEqual(loaded, { replay: "normalized replay" });
  assert.equal(cache.getState("tracked").status, "loaded");
  assert.deepEqual(observed, ["loading", "loading", "loaded"]);
});

test("playlist load cache records preload failures without unhandled rejections", async () => {
  const cache = new PlaylistLoadCache<{ replay: string }>();
  cache.preload([
    {
      id: "missing",
      async load() {
        throw new Error("not found");
      },
    },
  ]);

  await new Promise((resolve) => setTimeout(resolve, 0));
  assert.equal(cache.getState("missing").status, "error");
  assert.equal(cache.getState("missing").error, "not found");
});

test("headless playlist session advances and loops custom loaded bundles", async () => {
  const items = ["first", "second"].map((id) => ({
    replay: {
      id,
      async load() {
        return { bundleId: id };
      },
    },
    start: { kind: "time" as const, value: 0 },
    end: { kind: "time" as const, value: 1 },
  }));
  const session = new PlaylistSession(items, {
    advanceMode: "auto",
    endMode: "loop",
  });

  await session.waitForCurrentItem();
  assert.equal(session.getState().itemIndex, 0);
  assert.deepEqual(session.getState().loaded, { bundleId: "first" });

  assert.equal(await session.completeCurrentItem(), true);
  assert.equal(session.getState().itemIndex, 1);
  assert.deepEqual(session.getState().loaded, { bundleId: "second" });

  assert.equal(await session.completeCurrentItem(), true);
  assert.equal(session.getState().itemIndex, 0);
  assert.deepEqual(session.getState().loaded, { bundleId: "first" });
});

function emptyReplay(duration: number): ReplayModel {
  return {
    frameCount: 2,
    duration,
    frames: [
      {
        time: 0,
        secondsRemaining: 300,
        gameState: 0,
        kickoffCountdown: 0,
      },
      {
        time: duration,
        secondsRemaining: 300 - duration,
        gameState: 0,
        kickoffCountdown: 0,
      },
    ],
    ballFrames: [],
    boostPads: [],
    players: [],
    tickMarks: [],
    timelineEvents: [],
    teamZeroNames: [],
    teamOneNames: [],
  };
}

function replayWithFrames(times: number[]): ReplayModel {
  return {
    frameCount: times.length,
    duration: times.at(-1) ?? 0,
    frames: times.map((time) => ({
      time,
      secondsRemaining: 300 - time,
      gameState: 0,
      kickoffCountdown: 0,
    })),
    ballFrames: [],
    boostPads: [],
    players: [],
    tickMarks: [],
    timelineEvents: [],
    teamZeroNames: [],
    teamOneNames: [],
  };
}

test("full replay playlist item represents the natural one-replay playlist", () => {
  const loaded: LoadedReplay = { replay: emptyReplay(123) };
  const source = createReplaySource("single", async () => loaded);
  const item = createFullReplayPlaylistItem(source, {
    label: "Full replay",
  });

  assert.equal(item.replay.id, "single");
  assert.deepEqual(item.start, { kind: "time", value: 0 });
  assert.equal(item.end.kind, "time");
  assert.equal(item.end.value, Number.POSITIVE_INFINITY);
  assert.equal(item.label, "Full replay");
});

test("playlist clip time bounds resolve to replay time and duration", () => {
  const loaded: LoadedReplay = { replay: replayWithFrames([0, 1, 2, 3, 4, 5]) };
  const source = createReplaySource("clip-replay", async () => loaded);
  const item = {
    replay: source,
    start: { kind: "time" as const, value: 1.25 },
    end: { kind: "time" as const, value: 3.75 },
    label: "Time clip",
  };

  const resolved = resolvePlaylistItem(item, loaded);

  assert.equal(resolved.start.time, 1.25);
  assert.equal(resolved.start.frameIndex, 1);
  assert.equal(resolved.end.time, 3.75);
  assert.equal(resolved.end.frameIndex, 3);
  assert.equal(resolved.duration, 2.5);
});

test("playlist clip frame bounds resolve through replay frame times", () => {
  const loaded: LoadedReplay = { replay: replayWithFrames([0, 0.25, 1.5, 2.25, 4]) };
  const source = createReplaySource("frame-clip-replay", async () => loaded);
  const item = {
    replay: source,
    start: { kind: "frame" as const, value: 2 },
    end: { kind: "frame" as const, value: 4 },
  };

  const resolved = resolvePlaylistItem(item, loaded);

  assert.deepEqual(resolved.start, { frameIndex: 2, time: 1.5 });
  assert.deepEqual(resolved.end, { frameIndex: 4, time: 4 });
  assert.equal(resolved.duration, 2.5);
});

test("playlist clip bounds reject clips that end before they start", () => {
  const loaded: LoadedReplay = { replay: replayWithFrames([0, 1, 2, 3]) };
  const source = createReplaySource("bad-clip-replay", async () => loaded);

  assert.throws(
    () =>
      resolvePlaylistItem(
        {
          replay: source,
          start: { kind: "time", value: 3 },
          end: { kind: "time", value: 1 },
          label: "Backwards clip",
        },
        loaded,
      ),
    /Backwards clip.*ends before it starts/,
  );
});

test("playlist cache evicts least recently used replays while retaining active and prefetched sources", async () => {
  const cache = new PlaylistLoadCache<number>();
  let loads = 0;
  const source = (id: string) => ({ id, load: async () => ++loads });
  await cache.load(source("active"));
  await cache.load(source("next"));
  cache.protect(["active", "next"]);
  for (const id of ["a", "b", "c"]) await cache.load(source(id));
  await cache.load(source("a"));
  await cache.load(source("d"));
  assert.equal(cache.has("b"), false);
  assert.equal(cache.getState("b").status, "idle");
  assert.equal(cache.getStates().length, 5);
  assert.equal(cache.has("active"), true);
  assert.equal(cache.has("next"), true);
  assert.equal(await cache.load(source("a")), 3);
  assert.equal(await cache.load(source("b")), 7);
});

test("evicted in-flight loads cannot overwrite or delete a replacement", async () => {
  const cache = new PlaylistLoadCache<string>(1);
  let rejectOld: (error: Error) => void = () => {};
  let reportOld: () => void = () => {};
  const old = cache.load({
    id: "a",
    load: (context) =>
      new Promise<string>((_resolve, reject) => {
        rejectOld = reject;
        reportOld = () => context?.updateProgress({ progress: 0.5 });
      }),
  });
  await Promise.resolve();
  await cache.load({ id: "b", load: async () => "b" });
  const replacement = cache.load({ id: "a", load: async () => "new" });
  await replacement;
  reportOld();
  rejectOld(new Error("obsolete"));
  await assert.rejects(old, /obsolete/);
  assert.equal(await cache.load({ id: "a", load: async () => "wrong" }), "new");
  assert.equal(cache.getState("a").status, "loaded");
  assert.equal(cache.getState("a").progress, null);
});

test("cleared in-flight loads do not restore cached replay state", async () => {
  const cache = new PlaylistLoadCache<string>();
  let finish: (value: string) => void = () => {};
  const pending = cache.load({
    id: "a",
    load: () =>
      new Promise<string>((resolve) => {
        finish = resolve;
      }),
  });
  await Promise.resolve();
  cache.clear();
  finish("loaded");
  assert.equal(await pending, "loaded");
  assert.equal(cache.has("a"), false);
  assert.deepEqual(cache.getStates(), []);
});
