# @rlrml/viewer

A focused three.js Rocket League replay player, backed entirely by subtr-actor.

This is the high-fidelity counterpart to [`@rlrml/player`](../player): same idea —
subtr-actor parses the replay, this package renders it — but with full 3D car and
stadium models instead of a schematic scene. The goal is a lean, embeddable
library with a tight public surface, exactly like `@rlrml/player`.

**Embeddable and fully client-side.** A consumer drops this player into a page,
hands it raw `.replay` bytes, and gets playback — no backend, no server-side
preprocessing, no API. Everything (parsing via WASM, rendering, playback) runs in
the browser, so the player can be a self-contained, client-only playback machine.
The assets it needs (3D models, WASM) ship with the package.

## Architecture

```
.replay bytes
  → @rlrml/subtr-actor (WASM)        the only backend / data source
  → SubtrActorPlayer  (src/adapter)  per-frame timelines + live entities
  → three.js renderer (src/managers) GLB cars, stadium, ball, cameras
```

subtr-actor is the single source of truth. There is no second replay parser and
no server: replays are parsed in the browser via WASM and rendered directly.

Key modules:

- `src/adapter/coords.ts` — the one coordinate/unit transform (subtr-actor's
  native Unreal space → three.js world space). Single source of truth.
- `src/adapter/SubtrActorPlayer.ts` — turns subtr-actor's `ReplayData` into the
  data the renderer reads each frame: motion timelines + per-frame ball/car
  state (position, rotation, velocity, boost, visibility).
- `src/adapter/wasm.ts` — reuses `@rlrml/player`'s WASM loader to get raw
  `ReplayData`.
- `src/managers/`, `src/lib/` — the three.js renderer (scene, arena, actors,
  car-model loading, cameras, effects, trails).

## Status

**Working and verified:**

- Package builds (`vite`, `tsc` clean) and a dev server runs (`npm run dev`).
- The full data path — `.replay → WASM → adapter → three.js timelines` — is
  implemented and validated headlessly (`src/dev/validate.mts`): correct roster,
  teams, match duration, and **correct coordinates** (ball and cars land inside
  field dimensions; the up-axis is height). The coordinate transform — the main
  integration risk — is confirmed.
- A bring-up harness (`src/dev/main.ts`) drives the real renderer (scene + arena
  + actors) from the adapter to render a match.

**Not yet done:**

1. **Car models.** Every car currently renders as the default body. subtr-actor
   already provides each player's car/hitbox family; that needs to be fed into
   the car-model loader, replacing the renderer's legacy body-id lookup.
2. **Visual confirmation.** The data is verified; the rendered output still needs
   to be eyeballed in a browser.
3. **Full playback engine.** The harness wires managers directly. The complete
   camera/effects/UI path needs bringing up, and the adapter's interpolation can
   then be simplified — the renderer carries position-smoothing and frame-
   filtering passes that exist to clean raw replay jitter that subtr-actor
   already handles upstream; those should be removed, not preserved.

## Cleanup required to make this focused

The package currently contains a large amount of code that does **not** belong in
a focused player and must be removed:

- `src/pages/`, `src/components/`, `src/hooks/`, `src/api/`, `src/services/`,
  `src/collab/` — a full web application (routing, auth, uploads, comments,
  admin, collaboration, API clients). None of this is part of rendering a replay.
- `framework/` — a second, self-contained replay-parsing/compiler stack. The
  adapter replaces it. Only a couple of static data constants it holds (hitbox
  dimensions, car-family mapping) are still referenced by the renderer; those
  should be lifted into this package and the rest deleted.
- `sourcemaps/` — not part of the package.

The end state mirrors `@rlrml/player`: `src/` contains only the renderer
(`managers`, `lib`), the adapter, and a small `lib.ts` entry point exposing a
mount/embed API; `public/` holds the 3D assets (GLB models, draco decoder).

## Development

```
npm install
npm run dev          # dev server with a sample replay
npx tsx src/dev/validate.mts   # headless data-pipeline check
```
