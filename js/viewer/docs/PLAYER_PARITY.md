# @rlrml/player parity

The goal: make `ViewerPlayer` API-compatible with [`@rlrml/player`](../../player)'s
`ReplayPlayer`, so consumers written against it — first and foremost
[`js/stat-evaluation-player`](../../stat-evaluation-player) — can run on this
high-fidelity viewer unchanged.

Three phases:

1. **Control surface** *(done)*: state shape, setters, frame stepping,
   `setState`/`getSnapshot`/`subscribe`/`onBeforeRender`, camera delegation,
   `initial*` constructor options, stable player ids.
2. **Shared data layer** *(done — see below)*: `viewer.replay: ReplayModel` from
   `@rlrml/player`'s `normalizeReplayData` over the same raw WASM output the
   adapter consumes, with the adapter's ids and time axis aligned to it.
3. **Plugin-context parity** *(done — see below)*: `replay`/`options`/`state`
   in plugin contexts, `FrameRenderInfo` in the render context, the timeline
   projection / skip-window APIs (live, off the shared `ReplayModel`), and a
   bridge (`fromReplayPlayerPlugin`) that runs @rlrml/player's DOM plugins
   unmodified — the timeline overlay is mounted through it in the dev harness.
   Still open within Phase 3: ports of the renderer-coupled plugins
   (boost-pickup animation, canvas recorder; they use `beforeRender`, which the
   bridge deliberately rejects).

## State (`getState()` / `subscribe` payload)

`ViewerState` matches `ReplayPlayerState` field-for-field:

| Field | Status | Notes |
| --- | --- | --- |
| `currentTime`, `duration`, `playing`, `speed` | ✅ live | |
| `frameIndex` | ✅ live | binary search over the adapter's `frameTimes` (the replay's metadata-frame timeline) |
| `activeMetadata` | ✅ live | kickoff countdowns via @rlrml/player's own `getKickoffCountdownMetadata` over the shared `ReplayModel` (null when constructed without one) |
| `cameraDistanceScale` | ✅ live | scales the **follow-camera** distance via the camera plugin; no effect on the orbit camera (user scroll-zoom owns that) |
| `customCameraSettings` | ✅ live | delegated to the camera plugin as explicit overrides (they win over the recorded preset). `pitch` is accepted as an alias of the viewer-native `angle` |
| `cameraViewMode` | ✅ live | `"follow"` ⇄ camera-plugin follow mode; `"free"` covers all unattached modes (orbit / fly / ballOrbit). Derived from the plugin when installed, so dev-UI-driven changes stay truthful |
| `attachedPlayerId` | ✅ live | stable per-player id (from the replay's remote id) — `adapter.playerList[].id` |
| `ballCamEnabled` | ✅ live | reports the **effective** ball-cam state. Until explicitly set, the viewer follows the replay's recorded per-player ball-cam state (richer than @rlrml/player's static default) |
| `boostMeterEnabled` | 🟡 tracked-but-inert | no boost-meter rendering yet |
| `boostPickupAnimationEnabled` | 🟡 tracked-but-inert | pads animate unconditionally via `createBoostPadsPlugin()` |
| `hitboxWireframesEnabled` | 🟡 tracked-but-inert | `HitboxManager` exists but isn't wired |
| `hitboxOnlyModeEnabled` | 🟡 tracked-but-inert | |
| `skipPostGoalTransitionsEnabled` | ✅ live | @rlrml/player's exact skip-window semantics over the shared `ReplayModel` (`computeTimelineSegments` + skip-target helpers); applied on play/seek/tick, never while paused |
| `skipKickoffsEnabled` | ✅ live | same machinery; t=0 starts past the opening kickoff when enabled |

“Tracked-but-inert”: the setter updates state and notifies subscribers (so UI
toggles round-trip correctly), but no rendering behavior is attached yet.

## Methods

| `ReplayPlayer` | `ViewerPlayer` | Notes |
| --- | --- | --- |
| `play` / `pause` / `togglePlayback` / `seek` / `setPlaybackRate` | ✅ same | |
| `setFrameIndex` / `stepFrames` / `stepForwardFrame` / `stepBackwardFrame` | ✅ same | stepping pauses playback, like @rlrml/player |
| `setState(patch)` / `getState()` / `getSnapshot()` / `subscribe()` | ✅ same | `ViewerStatePatch` has the same keys as `ReplayPlayerStatePatch` |
| `onBeforeRender(cb) → unsub` | ✅ same | called each render with `FrameRenderInfo {frameIndex, nextFrameIndex, alpha, currentTime}` |
| `setCameraDistanceScale` | ✅ | follow-camera distance multiplier (≥ 0.25) |
| `setCustomCameraSettings(settings \| null)` | ✅ | replace-not-merge, `null` clears; `pitch`→`angle` mapped |
| `setAttachedPlayer(id \| null)` | ✅ | resolves the id to the adapter roster and drives camera-plugin `follow()` / `release()` |
| `setCameraViewMode("free" \| "follow")` | ✅ | `"free"` only leaves follow mode — it won't stomp viewer-native fly/ballOrbit modes set on the plugin handle |
| `setFreeCameraPreset("overhead" \| "side")` | ✅ | @rlrml/player's exact poses, converted to this package's Y-up space; positions the core orbit camera |
| `setBallCamEnabled(bool)` | ✅ | forces ball/car cam (overrides the recorded state) |
| toggle setters (`setBoostMeterEnabled`, …) | 🟡 | accepted; tracked-but-inert (see table above) |
| `addPlugin` / `removePlugin` / `getPlugins` / `destroy` / `dispose` | ✅ same | contexts now carry `replay`/`options`/`state` + `FrameRenderInfo`; @rlrml/player DOM plugins mount via `fromReplayPlayerPlugin` (see “Plugin bridge”) |
| `.replay: ReplayModel` | ✅ same | see “Shared data layer” below |
| `getTimelineDuration` / `getTimelineCurrentTime` / `getTimelineSegments` / `projectReplayTimeToTimeline` / `projectTimelineTimeToReplay` | ✅ same | @rlrml/player's own pure timeline utilities (exported from its lib) over the shared `ReplayModel`; identity projection when constructed without one |
| playlist APIs | ❌ | `ReplayPlaylistPlayer` wraps a player; out of scope for the core |

Camera delegation requires an installed camera plugin
(`createCameraPlugin()`, plugin id `"camera"`). Parity camera state is tracked
either way and pushed onto a camera plugin whenever one is installed — but
without one the calls only update reported state.

## Shared data layer (Phase 2)

`viewer.replay` is `@rlrml/player`'s `ReplayModel` — the exact structure its
consumers (most importantly `js/stat-evaluation-player`) read. One WASM parse
feeds both layers: `createViewer` calls `loadReplay` (`adapter/wasm.ts`), which
returns `loadReplayFromBytes`'s `{ replay, raw }`; `raw` builds the adapter,
`replay` is stored on the viewer. The model costs nothing extra —
`loadReplayFromBytes` computes it anyway.

Two cross-layer invariants make the model and the adapter interchangeable
(`src/dev/validate.mts` asserts both against a real replay):

- **Player ids.** The adapter's `_idKey` mirrors `@rlrml/player`'s
  `playerIdToString` (`Kind:value` from the remote-id tagged union), so
  `adapter.playerList[].id` ≡ `replay.players[].id` byte-for-byte.
- **Time axis.** Raw replay clocks don't start at 0; `normalizeReplayData`
  shifts every time by the first frame's raw time (`rawStartTime`). The adapter
  applies the identical shift to its frame timeline, keyframes, and boost-pad
  events, and exposes `adapter.rawStartTime`. `viewer.currentTime`,
  `FrameRenderInfo`, and `ReplayModel` times are all directly comparable.

**Build prerequisite.** The published `@rlrml/player` predates `rawStartTime`
(and the WASM-side fields the viewer needs), so the package depends on the
workspace build: `"@rlrml/player": "file:../player"` (build it with
`npx vite build && npx tsc --project tsconfig.build.json` in `js/player`) and
`"@rlrml/subtr-actor": "file:../pkg"` (built by `js/scripts/build-wasm.sh`,
which also writes the `package.json` that makes `js/pkg` installable).

## Plugin bridge (Phase 3)

`ViewerPluginContext` now carries everything `ReplayPlayerPluginContext` does —
`player` (the parity control + timeline surface), `replay` (the shared
`ReplayModel`), `options`, `container`, and `state` / `FrameRenderInfo` on the
state/render contexts. That makes @rlrml/player's **DOM-only** plugins run
unmodified through `fromReplayPlayerPlugin`
(`src/plugins/replay-player-bridge.ts`):

```ts
import { createTimelineOverlayPlugin, fromReplayPlayerPlugin } from "@rlrml/viewer";
viewer.addPlugin(fromReplayPlayerPlugin(createTimelineOverlayPlugin()));
```

The dev harness mounts the timeline overlay this way — markers, skip toggles,
and the scrubber all drive the viewer through the parity methods
(`projectTimelineTimeToReplay`, `getTimelineCurrentTime`, `seek`, …).

Two surfaces deliberately do **not** bridge, and fail loudly:

- `context.scene` is @rlrml/player's `ReplayScene` (its schematic renderer's
  internals); accessing it through the bridge throws.
- `beforeRender` plugins (boost-pickup animation, canvas recorder) receive
  renderer-internal frame state; the bridge rejects them at install time. They
  need native `ViewerPlugin` ports.

## Constructor options

All of `ReplayPlayerOptions`' `initial*` fields are accepted (`ViewerOptions`):
`initialPlaybackRate` (wins over viewer-native `speed`),
`initialCameraDistanceScale`, `initialCustomCameraSettings`,
`initialCameraViewMode`, `initialAttachedPlayerId`, `initialBallCamEnabled`,
plus the inert toggles. Viewer-native options (`loop`, `effects`, `autoplay`,
`plugins`) are unchanged. `fieldScale` is **not** supported: this renderer works
1:1 in Unreal Units.

## Known semantic differences

- **Player identity.** @rlrml/player keys everything on `track.id`; the viewer
  renderer keys on player *name* internally. The parity layer maps ids → names
  at the boundary (`adapter.playerList[].id`, `getPlayerById`).
- **Ball cam default.** @rlrml/player defaults `ballCamEnabled` to a static
  `false`; the viewer defaults to the *recorded* per-player ball-cam state and
  only forces a value once `setBallCamEnabled` / `initialBallCamEnabled` is
  used.
- **`cameraViewMode: "free"`** is a family of modes here (orbit, FPS fly,
  ballOrbit — see `createCameraPlugin`), all reported as `"free"`.
- **Camera settings naming.** @rlrml/player's `pitch` = viewer's `angle`; both
  accepted everywhere, `angle` wins if both are set.
