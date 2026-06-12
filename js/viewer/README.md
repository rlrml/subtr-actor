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

- **Car models are wired.** `ActorManager` selects each car's GLB from
  subtr-actor's per-player `car_body_name` / `car_hitbox_family`
  (`SubtrActorPlayer` → `CarModelLoader`), replacing the legacy body-id lookup.
  This requires the **local** WASM build (`js/pkg`, built from this repo's Rust via
  `js/scripts/build-wasm.sh`): the *published* `@rlrml/player` WASM predates the
  `car_hitbox_family` field, so the dev server's `vite` alias points
  `@rlrml/subtr-actor` at `../pkg`.

- **`ViewerPlayer` core + plugin host.** `src/ViewerPlayer.ts` is the bare
  playback core: it owns scene/arena/actors, the playback clock
  (play/pause/seek/speed/loop, `subscribe`), and the `ViewerPlugin` host from
  [`docs/EXTENSIBILITY.md`](./docs/EXTENSIBILITY.md). The public entry is
  `createViewer(container, bytes, options)` (`src/lib.ts`). The first plugin,
  `createNameTagPlugin()`, drives name tags entirely through hooks. The dev
  harness (`src/dev/main.ts`) now uses this public API.
- **Trail effects + the original per-frame path.** The real `EffectsManager` is
  wired into the core (opt out with `effects: false`): boost trails, supersonic
  trails, and the ball trail run off the adapter's per-frame state. The core's
  render loop preserves the original GameEngine frame order and behaviors:
  animation-mixer advance before actor updates, per-player boost particle state
  (with kickoff-reset suppression), supersonic state, wheel spin/steering from
  position deltas, and seek-time resets (animations, ball trail, wheel
  tracking — also applied on loop wrap). Explosions stay dormant until the
  adapter exposes goal/demo events (and `setRenderContext` warmup is deferred
  with them — it blocks the main thread for seconds).
- **Boost pads** (`createBoostPadsPlugin()`). The original ballcam GameEngine
  pad rendering (glowing spheres + point lights for big pads, flat cylinders
  for small), fed by subtr-actor's resolved pad layout and exact
  pickup/availability events — pads fade out when collected and relight on
  respawn, in sync with playback time.
- **Camera plugin** (`createCameraPlugin()`). The full original ballcam camera
  system wrapped as a plugin, with four modes: `orbit` (the core's
  OrbitControls, default), `free` (FPS fly cam: WASD/arrows + Space/Shift,
  right-click-drag look with pointer lock), `ballOrbit` (orbit the ball while
  tracking it), and `follow` (RL-style state-blended car cam ⇄ ball cam;
  `follow(name)` / `release()` / `setBallCam(bool|null)` — `null` follows the
  replay's recorded ball-cam state). Also ports the original RL camera
  settings (`setCameraSettings`: distance/height/angle/stiffness/swivel/
  transition speed) and the horizontal→vertical FOV conversion with its 16:9
  ultra-wide floor. The dev harness mounts a mode/player dropdown + ball-cam
  toggle (`B` key) + stiffness slider, and accepts
  `?follow=<player>&t=<seconds>` URL params.
- **@rlrml/player control-surface parity (Phase 1).** `ViewerPlayer` now
  exposes `@rlrml/player`'s `ReplayPlayer` API: the full
  `ReplayPlayerState`-shaped state (frameIndex, camera fields, display
  toggles), `setState(patch)` / `getSnapshot()`, frame stepping
  (`setFrameIndex` / `stepFrames` / `stepForwardFrame` / `stepBackwardFrame`),
  `onBeforeRender(cb)` with `FrameRenderInfo`, camera delegation
  (`setAttachedPlayer` / `setCameraViewMode` / `setBallCamEnabled` /
  `setCustomCameraSettings` / `setCameraDistanceScale` /
  `setFreeCameraPreset`) routed to the installed camera plugin, all `initial*`
  constructor options, and stable per-player ids on the adapter roster. The
  compatibility matrix, the id/name and `pitch`/`angle` mappings, and the
  3-phase roadmap toward porting `js/stat-evaluation-player` live in
  [`docs/PLAYER_PARITY.md`](./docs/PLAYER_PARITY.md).
- **Shared data layer (Parity Phase 2).** `viewer.replay` is `@rlrml/player`'s
  `ReplayModel`, built from the same single WASM parse that feeds the adapter
  (`loadReplay` in `src/adapter/wasm.ts`). The adapter's player ids and time
  axis are aligned with it exactly: ids mirror `playerIdToString`, and all
  adapter times are shifted by `rawStartTime` so t=0 is the first frame — the
  same normalization `normalizeReplayData` applies. Headless cross-checks in
  `src/dev/validate.mts` assert id-set and time-axis equality. This requires
  the **workspace** `@rlrml/player` (`file:../player`, see
  [`docs/PLAYER_PARITY.md`](./docs/PLAYER_PARITY.md) for the build steps).
- **Plugin-context parity + plugin bridge (Parity Phase 3).** Plugin contexts
  carry `replay`/`options`/`state` (+ `FrameRenderInfo` in the render
  context), the timeline-projection / skip-window APIs are live —
  @rlrml/player's own pure `ReplayModel` utilities, so `skipKickoffs` /
  `skipPostGoalTransitions`, `activeMetadata` kickoff countdowns, and the
  skip-aware playback end all behave identically — and
  `fromReplayPlayerPlugin()` mounts @rlrml/player's DOM plugins unmodified.
  The dev harness runs its real timeline overlay (markers, skip toggles,
  scrubber) through the bridge. Renderer-coupled plugins (`beforeRender`)
  are rejected loudly and need native ports — see
  [`docs/PLAYER_PARITY.md`](./docs/PLAYER_PARITY.md).
- **Recorded camera settings.** subtr-actor now extracts each player's
  replicated RL camera preset (`TAGame.CameraSettingsActor_TA:ProfileSettings`
  → `PlayerInfo.camera_settings`: fov/height/angle/distance/stiffness/swivel/
  transition). The adapter surfaces it as `playerList[].cameraSettings`, and
  follow mode defaults to the followed player's recorded preset — so the
  camera feels like the player's own — with `setCameraSettings` overrides
  winning per field (`useRecordedSettings: false` opts out). Requires the
  local WASM build (`js/scripts/build-wasm.sh`).

**Not yet done:**

1. **Goal/demo events → explosions.** Fill the adapter's stubbed event getters
   from subtr-actor, then trigger `EffectsManager` explosions (and call
   `setRenderContext` to warm the pools).
2. **Drop redundant smoothing.** The renderer (`ActorManager`) carries
   position-smoothing and frame-filtering passes that exist to clean raw replay
   jitter that subtr-actor already handles upstream; those should be removed,
   not preserved.
3. **Native ports of @rlrml/player's renderer-coupled plugins**
   (boost-pickup animation, canvas recorder): they use `beforeRender`, whose
   render context is renderer-internal, so the Phase 3 bridge deliberately
   rejects them ([`docs/PLAYER_PARITY.md`](./docs/PLAYER_PARITY.md)).

## Focused layout (cleanup complete)

The web-app code (a full routing/auth/uploads/comments/admin/collab application)
and the second replay-parsing stack (`framework/`) have been removed. `src/` now
mirrors `@rlrml/player`'s lean shape:

- `src/ViewerPlayer.ts`, `src/lib.ts`, `src/types.ts` — the playback core,
  public embed API, and plugin contract.
- `src/plugins/` — built-in plugins (`createNameTagPlugin`).
- `src/adapter/` — subtr-actor → renderer data (the only data source).
- `src/managers/`, `src/lib/` — the three.js renderer (scene, arena, actors, car
  models, cameras, effects, trails).
- `src/data/hitboxes.js` — the static hitbox constants lifted out of `framework/`.
- `src/dev/` — the bring-up harness + headless validator.
- `public/` — the 3D assets (GLB models, draco decoder).

Everything above raw playback (scoreboard/HUD, overlays, clips, collab, dev tools)
was intentionally dropped and is meant to return as **plugins** — the contract and
a ledger of what was removed live in
[`docs/EXTENSIBILITY.md`](./docs/EXTENSIBILITY.md). The host for those plugins is
`ViewerPlayer` (`src/ViewerPlayer.ts`) behind `createViewer()` (`src/lib.ts`).

## Development

```
npm install
npm run dev          # dev server with a sample replay
npx tsx src/dev/validate.mts   # headless data-pipeline + parity cross-checks
```

Local-workspace dependencies (both `file:` installs, wired by `npm install`):

- `@rlrml/subtr-actor` → `../pkg`, built by `js/scripts/build-wasm.sh` (the
  published WASM predates fields the viewer needs).
- `@rlrml/player` → `../player`, built with
  `cd ../player && npm ci && npx vite build && npx tsc --project tsconfig.build.json`
  (the published package predates the `ReplayModel` time-axis fields).
  The player resolves `@rlrml/subtr-actor` at runtime through
  `js/player/node_modules/@rlrml/subtr-actor → ../pkg` (a symlink; recreate it
  after an `npm ci` in `js/player`).
