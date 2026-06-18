# @rlrml/player

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
  - actors) from the adapter to render a match.

- **Car models are wired.** `ActorManager` selects each car's GLB from
  subtr-actor's per-player `car_body_name` / `car_hitbox_family`
  (`SubtrActorPlayer` → `CarModelLoader`), replacing the legacy body-id lookup.
  This requires the **local** WASM build (`js/pkg`, built from this repo's Rust via
  `js/scripts/build-wasm.sh`): the _published_ `@rlrml/player` WASM predates the
  `car_hitbox_family` field, so the dev server's `vite` alias points
  `@rlrml/subtr-actor` at `../pkg`.

- **`ReplayPlayer` core + plugin host.** `src/ReplayPlayer.ts` is the bare
  playback core: it owns scene/arena/actors, the playback clock
  (play/pause/seek/speed/loop, `subscribe`), and the `PlayerPlugin` host from
  [`docs/player/EXTENSIBILITY.md`](./docs/player/EXTENSIBILITY.md). The public entry is
  `createPlayer(container, bytes, options)` (`src/lib.ts`). The first plugin,
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
- **Skybox environments** (`src/environments.ts`). An HDR skybox drives both the
  visible background and the image-based lighting (reflections/ambient on cars,
  arena, ball) — the polish layer the original ballcam player got from its HDRs.
  The built-in `"space"` (ballcam's PlanetaryEarth4k, shipped in
  `public/skyboxes/`) is the default; select it via the `environment` option (a
  built-in id, a full `PlayerEnvironment` descriptor, or `false` for neutral
  default lighting), switch at runtime with `player.setEnvironment(...)`, and add
  your own with `registerEnvironment(...)`. Loading is lazy: playback starts on
  the asset-free `RoomEnvironment` fallback and the HDR swaps in once decoded.
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
- **Goal explosion + scored text.** The team-colored goal explosion (the
  original GameEngine's pooled particle/shockwave burst, in `EffectsManager`)
  now fires in the core: `ReplayPlayer` feeds the replay's goal events to the
  effects system and `ActorManager` triggers the blast at the ball as playback
  crosses each goal (gated by `effects`, default on). The centered
  "&lt;PLAYER&gt; SCORED !!" banner is a separate, toggleable plugin
  (`createScoredTextPlugin()`) — a faithful reproduction of the original gold
  6rem Bourgeois HUD text, add/remove it at runtime like any other plugin.
- **@rlrml/player control-surface parity (Phase 1).** `ReplayPlayer` now
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
  [`docs/player/PLAYER_PARITY.md`](./docs/player/PLAYER_PARITY.md).
- **Shared data layer (Parity Phase 2).** `player.replay` is `@rlrml/player`'s
  `ReplayModel`, built from the same single WASM parse that feeds the adapter
  (`loadReplay` in `src/adapter/wasm.ts`). The adapter's player ids and time
  axis are aligned with it exactly: ids mirror `playerIdToString`, and all
  adapter times are shifted by `rawStartTime` so t=0 is the first frame — the
  same normalization `normalizeReplayData` applies. Headless cross-checks in
  `src/dev/validate.mts` assert id-set and time-axis equality. This requires
  the **workspace** `@rlrml/player` (`file:../player`, see
  [`docs/player/PLAYER_PARITY.md`](./docs/player/PLAYER_PARITY.md) for the build steps).
- **Plugin-context parity + plugin bridge (Parity Phase 3).** Plugin contexts
  carry `replay`/`options`/`state` (+ `FrameRenderInfo` in the render
  context), the timeline-projection / skip-window APIs are live —
  @rlrml/player's own pure `ReplayModel` utilities, so `skipKickoffs` /
  `skipPostGoalTransitions`, `activeMetadata` kickoff countdowns, and the
  skip-aware playback end all behave identically — and
  `fromReplayPlayerPlugin()` mounts @rlrml/player plugins unmodified. DOM
  hooks pass straight through; `beforeRender` plugins get a synthesized
  `ReplayPlayerRenderContext` built from the shared `ReplayModel` with
  @rlrml/player's own exported math (frame windows, interpolated positions,
  boost fractions — identical semantics). The dev harness runs its real
  timeline overlay (markers, skip toggles, scrubber), boost-pickup
  animation, and canvas recorder through the bridge — see
  [`docs/player/PLAYER_PARITY.md`](./docs/player/PLAYER_PARITY.md).
- **`player.sceneState` + `replayRoot`.** A `ReplayScene`-shaped surface for
  @rlrml/player consumers that mount THREE overlays (the stat-evaluation
  player's stat modules). `player.replayRoot` is the portable seam: a group
  whose local space is raw Unreal coordinates in both players, so overlays
  positioned in UE coords render correctly here with `fieldScale = 1`.
  `ballMesh`/`playerMeshes` are live views onto this renderer's actors.
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
3. **Port `js/stat-evaluation-player` onto the player.** Every parity surface
   it consumes is now in place ([`docs/player/PLAYER_PARITY.md`](./docs/player/PLAYER_PARITY.md));
   what remains is the port itself (constructor shape via `createPlayer`, and
   rerouting its few scene-level overlays through `replayRoot`).

## Focused layout (cleanup complete)

The web-app code (a full routing/auth/uploads/comments/admin/collab application)
and the second replay-parsing stack (`framework/`) have been removed. `src/` now
mirrors `@rlrml/player`'s lean shape:

- `src/ReplayPlayer.ts`, `src/lib.ts`, `src/types.ts` — the playback core,
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
[`docs/player/EXTENSIBILITY.md`](./docs/player/EXTENSIBILITY.md). The host for those plugins is
`ReplayPlayer` (`src/ReplayPlayer.ts`) behind `createPlayer()` (`src/lib.ts`).

## Development

```
npm install
npm run dev          # dev server with a sample replay
npx tsx src/dev/validate.mts   # headless data-pipeline + parity cross-checks
```

Local-workspace dependencies (both `file:` installs, wired by `npm install`):

- `@rlrml/subtr-actor` → `../pkg`, built by `js/scripts/build-wasm.sh` (the
  published WASM predates fields the player needs).
- `@rlrml/player` → `../player`, built with
  `cd ../player && npm ci && npx vite build && npx tsc --project tsconfig.build.json`
  (the published package predates the `ReplayModel` time-axis fields).
  The player resolves `@rlrml/subtr-actor` at runtime through
  `js/player/node_modules/@rlrml/subtr-actor → ../pkg` (a symlink; recreate it
  after an `npm ci` in `js/player`).
