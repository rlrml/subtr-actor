# Replay Review Playlists

## Goal

Build a playlist workflow for human review of exact replay targets. The primary
use case is assessing heuristic output across many replays, but the playlist
layer should stay abstract: a target is "something to review" rather than a
hard-coded mechanic, stat, or detector result.

The workflow should support playlists hosted separately from the app, replay
files hosted on other origins, and background loading of nearby replays so that
moving through review targets feels smooth.

Out of scope:

- discovering missed cases automatically
- range-request based replay streaming
- coupling the playlist abstraction to a specific stats module or heuristic

## Existing State

`js/player` already has a useful generic playlist foundation:

- `PlaylistItem` points at a `ReplaySource`, has `start`/`end` bounds, and
  carries optional `label` and `meta`.
- `ReplayPlaylistPlayer` wraps `ReplayPlayer`, resolves bounds after replay
  load, manages next/previous playback, and preloads unique adjacent replay
  sources according to `ReplayPreloadPolicy`.
- `js/player/src/manifest.ts` parses simple JSON manifests and resolves them
  through caller-provided replay source creation.

The main limitations for the review workflow are:

- `PlaylistManifestReplay` only models `path`, not URL-backed or inline replay
  locators.
- there is no helper to load a playlist manifest from a URL.
- `ReplaySource.load()` has no `AbortSignal`, so jumps cannot cancel obsolete
  fetch/parse work.
- the generic player playlist loads `LoadedReplay`, while
  `js/stat-evaluation-player` needs a `ReplayLoadBundle` containing both the
  normalized `ReplayModel` and the stats timeline.
- the stats evaluation player currently owns a single-replay loading path in
  `main.ts` and constructs `ReplayPlayer` directly.

## Playlist Model

Keep playlists as exact target lists. Query/search based workflows can generate
manifests, but the review app should consume a concrete snapshot.

Proposed manifest shape:

```ts
interface ReviewPlaylistManifest {
  version: 1;
  id?: string;
  label?: string;
  meta?: Record<string, unknown>;
  replays?: ReviewPlaylistReplay[];
  items: ReviewPlaylistItem[];
}

interface ReviewPlaylistReplay {
  id: string;
  locator: ReplayLocator;
  label?: string;
  meta?: Record<string, unknown>;
}

type ReplayLocator =
  | { kind: "url"; url: string; label?: string; mimeType?: string; sha256?: string }
  | { kind: "path"; path: string; label?: string }
  | { kind: "inline"; id: string; label?: string };

interface ReviewPlaylistItem {
  id?: string;
  replay: string | ReplayLocator;
  start: PlaybackBound;
  end: PlaybackBound;
  label?: string;
  meta?: Record<string, unknown>;
}
```

`meta` is the abstraction point for heuristic-specific details. For example, a
mechanic evaluator can include detector name, confidence, event id, player id,
or any other payload without changing the player package.

## Cross-Origin Replay URLs

The browser can fetch cross-site replay URLs only when the replay host allows
CORS for the review app origin. The playlist system does not need range
requests; a normal full-object `fetch()` is enough.

Supported URL workflow:

1. Load a playlist manifest from a URL, file, or inline object.
2. Resolve each URL replay locator into a replay source.
3. Fetch the full replay bytes with `fetch(url, { signal })`.
4. Parse the replay bytes through the normal subtr-actor loading path.

If a replay host cannot set CORS headers, support should come from a separate
proxy loader later. The manifest format should not assume that proxying is
always available.

## Architecture

Separate the generic playlist/session mechanics from the replay parsing output
type:

```ts
interface ReplayLoader<TLoaded> {
  load(locator: ReplayLocator, signal?: AbortSignal): Promise<TLoaded>;
}

interface ReviewTarget<TReplayRef = ReplayLocator> {
  id?: string;
  replay: TReplayRef;
  start: PlaybackBound;
  end: PlaybackBound;
  label?: string;
  meta?: Record<string, unknown>;
}

interface ReviewSession<TLoaded> {
  readonly items: ReviewTarget[];
  setCurrentItemIndex(index: number): Promise<void>;
  next(): Promise<boolean>;
  previous(): Promise<boolean>;
  preloadAround(index: number): void;
}
```

For `js/player`, `TLoaded` is `LoadedReplay`.

For `js/stat-evaluation-player`, `TLoaded` is `ReplayLoadBundle`, and the
player-facing replay is `bundle.replay`. Stats windows, timeline overlays, and
module state come from `bundle.statsTimeline`.

This avoids putting stats-timeline concepts into `subtr-actor-player` while
still letting the stats player reuse the same playlist manifest and preload
policy.

## Implementation Plan

1. Extend manifest types in `js/player`.

   Add `ReplayLocator` and URL-capable manifest replay/item parsing while
   keeping backward compatibility for the existing `path` field. Existing
   manifests should continue to parse unchanged.

2. Add manifest loading helpers.

   Add helpers such as:

   - `loadPlaylistManifestFromUrl(url, options?)`
   - `createReplayUrlSource(locator, options?)`
   - `resolvePlaylistItemsFromManifest(manifest, resolver)` updated to pass the
     full locator context

   URL helpers should use full-object `fetch()` only. No range behavior should
   be introduced.

3. Add cancellation-aware source loading.

   Change or extend `ReplaySource.load` to accept an optional `AbortSignal`.
   `ReplayPlaylistPlayer` should create a new controller per current-item load,
   abort stale current loads when jumping, and optionally abort preloads when
   the preload window changes.

4. Extract the generic cache/preload policy.

   `ReplayPlaylistPlayer` currently has a private promise cache keyed by replay
   id. Pull that into a small reusable cache/preload helper so the stats player
   can cache `ReplayLoadBundle` without depending on `ReplayPlaylistPlayer`.

   Requirements:

   - key by replay locator/source id, not target id
   - dedupe concurrent loads
   - evict failed loads
   - support bounded preload policies
   - expose enough state for basic loading/error UI

5. Integrate with `js/stat-evaluation-player`.

   Add a stats-player playlist mode that can be driven by a playlist manifest
   URL or local manifest file. It should reuse the existing replay loading
   worker by wrapping URL/file bytes in a `ReplayLoader<ReplayLoadBundle>`.

   The current single-replay path should remain available. Playlist mode should
   be an additional source of replay loads, not a replacement for drag-and-drop
   or `?replayUrl=...`.

6. Add review navigation UI.

   Add minimal controls to the stats evaluation player:

   - current target label/index
   - previous/next target
   - jump to target start
   - item loading/error state

   The first UI pass does not need persistent human labels or judgments. Those
   can be added as a separate review-results feature once the playlist mechanics
   are stable.

7. Add query-parameter entry points.

   Support a playlist manifest URL parameter alongside the existing replay URL
   parameter, for example:

   - `?playlistUrl=https://example.com/review-playlist.json`
   - compressed aliases if URL length becomes a problem

   If both a replay URL and playlist URL are provided, the playlist should win
   because it describes an exact review session.

8. Document CORS requirements.

   Update `js/player/README.md` and `js/stat-evaluation-player/README.md` with
   examples for hosted manifests and hosted replay URLs. Be explicit that the
   replay origin must allow browser CORS for direct cross-site loading.

9. Test the core behavior.

   Add unit tests for:

   - parsing legacy path manifests
   - parsing URL locator manifests
   - resolving repeated targets that share one replay source
   - preload deduplication by replay id
   - cancellation/failed-load cache eviction
   - playlist URL query parsing in the stats player

## Open Design Choices

- Whether `ReplaySource.load(signal?)` should be a breaking type change or
  introduced as a parallel `ReplayLoader` abstraction first.
- Whether review target `start`/`end` should remain required or whether a
  focus-only item should be allowed and expanded to a default clip window.
- Whether playlist item review state should live in the playlist module or in a
  separate review-results module.
- Whether URL replay locators should support custom request headers. This is
  useful for some internal workflows but unsafe to expose casually in public
  manifests.
