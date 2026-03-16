# subtr-actor player

`js/player` is the reusable replay player library package for this repository.

It provides:

- a typed replay normalization layer on top of `get_replay_frames_data()`
- a `ReplayPlayer` class with imperative playback and camera APIs
- state snapshots and change subscriptions so callers can build their own controls

The player exposes camera state as data, not fixed UI assumptions. The built-in
camera API is a free camera when no player is attached and an attached chase
camera with tunable distance scaling when a player is selected.

The package does not assume any specific UI. The demo app under
[`js/example`](../example/README.md) is the reference consumer.

## Development

```bash
npm --prefix js/player install
npm --prefix js/player run check
npm --prefix js/player run build
```

The build regenerates the local WASM bindings in `js/pkg/` before bundling the
library.
