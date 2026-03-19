# Replay Viewer Example

This is a Vite-based demo app for the local player library under `js/player`.
It loads a `.replay` file in the browser, validates it through the WASM
bindings, builds a typed replay model from `get_replay_frames_data()`, and
drives the reusable `ReplayPlayer` API with example controls.

## What It Demonstrates

- browser-side replay parsing with the WASM binding
- worker-backed replay loading with progress callbacks from the player library
- the built-in replay loading overlay from the player library, alongside a custom status readout
- example controls layered on top of the player library state API
- rendering kickoff countdown UI from the player library's semantic metadata
- free/attached camera selection, follow-distance tuning, seeking, and playback-rate control
- integration testing for the local `js/player` package

## Prerequisites

- [Node.js](https://nodejs.org/) 18 or newer
- [wasm-pack](https://rustwasm.github.io/wasm-pack/)

## Quick Start

From the repository root:

```bash
cd js/example
npm install
npm run dev
```

Open `http://localhost:5173`, then drop in a `.replay` file.

`npm run dev` and `npm run build` now rebuild `js/pkg/` automatically when the
Rust or JS binding sources are newer than the generated package. `npm run
build-wasm` remains available when you want to force a rebuild.

## Scripts

```bash
npm run dev
npm run build
npm run preview
npm run build-wasm
npm run dev-with-wasm
```

## How It Works

1. `src/index.js` initializes the WASM package from `../pkg/`.
2. The app validates the uploaded replay with `validate_replay()`.
3. It collects lightweight metadata with `get_replay_info()` and `get_replay_meta()`.
4. It loads full structured frame data with `get_replay_frames_data()`.
5. It passes the bytes into `js/player` via `loadReplayFromBytes(..., { useWorker, onProgress })`.
6. It renders the built-in `createReplayLoadOverlay()` UI while also updating its own custom status text from the same progress events.
6. It wires the demo controls to `ReplayPlayer` methods and change events.

## Relevant Files

```text
js/example/
├── index.html
├── package.json
├── src/index.js
├── src/styles.css
└── vite.config.js
```

## Troubleshooting

### Import or WASM initialization errors

- Run `npm run build-wasm` to force a clean rebuild.
- Confirm that `js/pkg/` contains the generated JS and `.wasm` files.

### Invalid replay errors

- Use a Rocket League `.replay` file.
- Some corrupted or very old replays may fail parsing.

## Notes

- The app currently uses the web-target JS package build, not the repo-local bundler target from `just build-js`.
- The example is the integration harness for the local player library in [`js/player`](../player/README.md).
- The player-library design notes live in [`docs/js-player-library-plan.md`](../../docs/js-player-library-plan.md).
