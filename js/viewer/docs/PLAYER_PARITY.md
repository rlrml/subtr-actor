# @rlrml/player parity

The goal: make `ViewerPlayer` API-compatible with [`@rlrml/player`](../../player)'s
`ReplayPlayer`, so consumers written against it — first and foremost
[`js/stat-evaluation-player`](../../stat-evaluation-player) — can run on this
high-fidelity viewer unchanged.

Three phases:

1. **Control surface** *(done — this document)*: state shape, setters, frame
   stepping, `setState`/`getSnapshot`/`subscribe`/`onBeforeRender`, camera
   delegation, `initial*` constructor options, stable player ids.
2. **Shared data layer** *(next)*: expose `viewer.replay: ReplayModel` by running
   `@rlrml/player`'s `normalizeReplayData` on the same raw WASM output the
   adapter already consumes.
3. **Plugin-context parity**: add `replay`/`options`/`state` to plugin contexts,
   `FrameRenderInfo` in the render context, and port/reuse the timeline overlay,
   boost-pickup animation, ballchasing overlay, and canvas recorder plugins.

## State (`getState()` / `subscribe` payload)

`ViewerState` matches `ReplayPlayerState` field-for-field:

| Field | Status | Notes |
| --- | --- | --- |
| `currentTime`, `duration`, `playing`, `speed` | ✅ live | |
| `frameIndex` | ✅ live | binary search over the adapter's `frameTimes` (the replay's metadata-frame timeline) |
| `activeMetadata` | ⚪ always `null` | @rlrml/player surfaces kickoff countdowns here; no equivalent yet |
| `cameraDistanceScale` | ✅ live | scales the **follow-camera** distance via the camera plugin; no effect on the orbit camera (user scroll-zoom owns that) |
| `customCameraSettings` | ✅ live | delegated to the camera plugin as explicit overrides (they win over the recorded preset). `pitch` is accepted as an alias of the viewer-native `angle` |
| `cameraViewMode` | ✅ live | `"follow"` ⇄ camera-plugin follow mode; `"free"` covers all unattached modes (orbit / fly / ballOrbit). Derived from the plugin when installed, so dev-UI-driven changes stay truthful |
| `attachedPlayerId` | ✅ live | stable per-player id (from the replay's remote id) — `adapter.playerList[].id` |
| `ballCamEnabled` | ✅ live | reports the **effective** ball-cam state. Until explicitly set, the viewer follows the replay's recorded per-player ball-cam state (richer than @rlrml/player's static default) |
| `boostMeterEnabled` | 🟡 tracked-but-inert | no boost-meter rendering yet |
| `boostPickupAnimationEnabled` | 🟡 tracked-but-inert | pads animate unconditionally via `createBoostPadsPlugin()` |
| `hitboxWireframesEnabled` | 🟡 tracked-but-inert | `HitboxManager` exists but isn't wired |
| `hitboxOnlyModeEnabled` | 🟡 tracked-but-inert | |
| `skipPostGoalTransitionsEnabled` | 🟡 tracked-but-inert | no goal events from the adapter yet |
| `skipKickoffsEnabled` | 🟡 tracked-but-inert | |

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
| `addPlugin` / `removePlugin` / `getPlugins` / `destroy` / `dispose` | ✅ same | plugin *contract* differs until Phase 3 (`ViewerPlugin` vs `ReplayPlayerPlugin`) |
| `.replay` / playlist & timeline-projection APIs | ❌ Phase 2+ | `.adapter` is the data surface for now |

Camera delegation requires an installed camera plugin
(`createCameraPlugin()`, plugin id `"camera"`). Parity camera state is tracked
either way and pushed onto a camera plugin whenever one is installed — but
without one the calls only update reported state.

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
