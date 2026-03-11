# subtr-actor three-player

Clean-room three.js replay playback package built against the local `subtr-actor` wasm bindings.

## What it includes

- A small library entrypoint at `src/lib.ts`
- A `ReplayPlayer` class for scene setup and playback control
- Replay loading through fresh local wasm builds from `../pkg`
- A Vite demo page that loads `.replay` files from disk
- Camera modes for detached overview, close attached follow, and wider third-person follow

## Run the demo

```bash
npm --prefix js/three-player install
npm --prefix js/three-player run dev
```

That will rebuild the local wasm web package first, then start the demo server.

## Build

```bash
npm --prefix js/three-player run build
```

Output:

- Demo bundle: `js/three-player/dist/demo/`
- Library bundle: `js/three-player/dist/lib/`
