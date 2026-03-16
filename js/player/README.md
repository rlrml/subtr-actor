# subtr-actor-player

`subtr-actor-player` is the reusable replay player library package for this repository.

## Installation

```bash
npm install subtr-actor-player three
```

`three` is a peer dependency. The player also depends on the low-level
[`subtr-actor`](https://www.npmjs.com/package/subtr-actor) bindings package.

It provides:

- a typed replay normalization layer on top of `get_replay_frames_data()`
- a `ReplayPlayer` class with imperative playback and camera APIs
- a `ReplayPlaylistPlayer` wrapper for back-to-back clip playback across replays
- a plugin host for optional scene and UI extensions
- state snapshots and change subscriptions so callers can build their own controls

The player exposes camera state as data, not fixed UI assumptions. The built-in
camera API is a free camera when no player is attached and an attached chase
camera with tunable distance scaling when a player is selected.

Transient replay semantics are exposed the same way. `ReplayPlayerState` and
`ReplayPlaylistPlayerState` include `activeMetadata`, which is currently used
for semantic kickoff countdown data (`{ kind: "kickoff-countdown", ... }`)
without imposing any built-in overlay or styling.

The package does not assume any specific UI. The demo app under `js/example/`
is the reference consumer in this repository.

Optional replay extensions can be installed through `ReplayPlayerOptions.plugins`
or at runtime with `ReplayPlayer.addPlugin(...)`. Plugins receive:

- a setup/teardown lifecycle with access to the player, replay, scene, and container
- state-change hooks for DOM/HUD style integrations
- per-frame render hooks with interpolated frame timing and player sample context

This keeps optional features such as scoreboards, scrubbers, and scene overlays
out of the core playback engine while still giving them a first-class API.

The package ships with reusable UI plugins:

- `createBallchasingOverlayPlugin()` for Ballchasing-style floating player labels,
  floating boost bars, and side team boost HUDs
- `createBoostPadOverlayPlugin()` for in-stadium standard Soccar boost pad
  availability markers driven by replay pad events
- `createTimelineOverlayPlugin()` for a bottom-docked replay scrubber with
  integrated play/pause, time readouts, clickable event markers, default replay markers
  (goals/saves), and caller-supplied custom events

For multi-replay workflows, the playlist layer is intentionally generic. Each
`PlaylistItem` specifies a replay source, a start bound, an end bound, and
optional `label`/`meta`, while `ReplayPlaylistPlayer` handles replay loading,
bound resolution, configurable replay-source prefetching, and clip-to-clip
transitions.

Preloading is controlled with `preloadPolicy`. Built-in modes are:

- `{ kind: "none" }`
- `{ kind: "all" }`
- `{ kind: "adjacent", ahead, behind? }`
- `{ kind: "custom", pick(context) }`

Policies operate on unique replay sources rather than raw playlist items, so a
run of multiple clips from the same replay only triggers one replay preload.

The package also includes lightweight manifest helpers for disk-backed playlist
workflows:

- `loadPlaylistManifestFromFile(file)`
- `parsePlaylistManifest(value)`
- `resolvePlaylistItemsFromManifest(manifest, resolveReplaySource)`

## Development

```bash
npm --prefix js/player install
npm --prefix js/player run check
npm --prefix js/player run build
npm --prefix js/player run smoke:install
```

The build regenerates the local WASM bindings in `js/pkg/` before bundling the
library, emits declaration files, and produces the npm artifact in `dist/`.
