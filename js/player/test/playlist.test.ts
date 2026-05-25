import test from "node:test";
import assert from "node:assert/strict";

import {
  PlaylistLoadCache,
  PlaylistSession,
  createFullReplayPlaylistItem,
  createReplaySource,
  parsePlaylistManifest,
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
