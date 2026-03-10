# Replay Viewer Example

This is a Vite-based example app for the JavaScript binding. It loads a `.replay` file in the browser, validates it, converts the structured replay data into the shape expected by the ballchasing.com viewer, and renders a 3D replay player.

## What It Demonstrates

- browser-side replay parsing with the WASM binding
- file upload via drag and drop or file picker
- replay validation and metadata extraction
- conversion from `get_replay_frames_data()` output into a viewer-friendly format
- integration with the ballchasing.com player scripts

## Prerequisites

- [Node.js](https://nodejs.org/) 18 or newer
- [wasm-pack](https://rustwasm.github.io/wasm-pack/)

## Quick Start

From the repository root:

```bash
cd js/example
npm install
npm run build-wasm
npm run dev
```

Open `http://localhost:5173`, then drop in a `.replay` file.

`npm run build-wasm` is required before the app can start cleanly. It builds the web-target WASM package into `js/pkg/`, which this example imports directly.

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
5. It adapts the replay into the viewer format expected by `src/player.js`.

## Relevant Files

```text
js/example/
├── index.html
├── package.json
├── src/index.js
├── src/player.js
└── vite.config.js
```

## Troubleshooting

### Import or WASM initialization errors

- Run `npm run build-wasm` again.
- Confirm that `js/pkg/` contains the generated JS and `.wasm` files.
- Make sure `wasm-pack` is on your `PATH`.

### Invalid replay errors

- Use a Rocket League `.replay` file.
- Some corrupted or very old replays may fail parsing.

### Viewer assets fail to load

- Check the browser console for network errors from CDN-hosted dependencies.
- Reload after confirming your network allows requests to `cdnjs.cloudflare.com` and `unpkg.com`.

## Notes

- The app currently uses the web-target JS package build, not the repo-local bundler target from `just build-js`.
- The example is useful as an integration reference, but the main package README under [`js/README.md`](../README.md) is the better starting point for library consumers.
