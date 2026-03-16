# JS Player Library Plan

## Goal

Turn the current `js/example` replay viewer into a real library package that lives in
its own directory under `js/`, with the example reduced to a thin demo app.

The new library should:

- use `subtr-actor` replay data as directly as practical
- reuse the useful three.js rendering pieces that previously lived in
  `js/example/src/player.js`
- avoid Ballchasing-specific globals and data contracts as the primary internal model
- make any coordinate-system or metadata adaptation explicit and testable

## Current Example Pipeline

Today the example has four distinct layers collapsed into one app:

1. WASM bootstrap and file upload in `js/example/src/index.js`
2. conversion from `serde_wasm_bindgen` objects into plain JS objects via `mapToObject()`
3. ad hoc translation from `subtr-actor` `ReplayData` into the Ballchasing-style
   `window.replayData` shape via `adaptFrameData()`
4. rendering and controls through the legacy `js/example/src/player.js` script
   that existed before the extraction to `js/player`

### Exact Translation Performed by `js/example`

The current translation happens in `ReplayAnalyzer.adaptFrameData()` in
`js/example/src/index.js`.

#### Input shape

The input is the serialized result of `get_replay_frames_data()`:

- `frame_data.ball_data.frames`
- `frame_data.players`
- `frame_data.metadata_frames`
- `meta.team_zero`
- `meta.team_one`
- top-level event collections such as `boost_pad_events` and `goal_events`

`ReplayDataCollector` already provides real sample times per frame in
`metadata_frames[*].time` and serializes the replay in player order as
`team_zero` followed by `team_one`.

#### Ball translation

For each ball frame:

- `rigid_body.location.{x,y,z}` is copied directly into `balls[0].pos`
- `rigid_body.rotation.{x,y,z,w}` is copied directly into `balls[0].quat`
- empty frames are replaced with `(0, 0, 0)` position and identity quaternion

The example does not rescale or re-axis the ball data in JS before handing it to
the player script.

#### Player translation

For each player frame:

- `rigid_body.location.{x,y,z}` is copied directly into `players[*].cars[0].pos`
- `rigid_body.rotation.{x,y,z,w}` is copied directly into `players[*].cars[0].quat`
- `boost_amount` is converted from raw replay units (`0..255`) into a rounded
  percentage (`0..100`)
- `boost_active` edges are converted into `boost_state.start[]` and
  `boost_state.end[]` intervals for boost trail rendering
- empty frames are replaced with zero position, identity quaternion, and zero boost

Again, no explicit scene-space transform happens in the adapter.

#### Timing translation

The example ignores `metadata_frames[*].time`.

Instead it synthesizes time as:

- `frameRate = 30`
- `time = frameIndex / 30`
- `max_time = metadata_frames.length / 30`

This means the example does not use the actual sample times produced by
`ReplayDataCollector`, even though that data already exists.

#### Team and player metadata translation

The example builds:

- `allPlayerMeta = [...meta.team_zero, ...meta.team_one]`
- `playerName = playerMeta?.name || "Player N"`

It then tries to infer team as:

- `playerMeta?.team === 0 ? "blue" : "orange"`

That is not a reliable translation because `ReplayMeta::PlayerInfo` does not
currently expose a `team` field. The ordering of `team_zero` and `team_one`
matches the player frame order, but the example does not use that ordering
explicitly to assign team membership.

#### Data dropped or stubbed out

The example returns Ballchasing-style fields for:

- `countdowns`
- `rem_seconds`
- `blue_score`
- `orange_score`
- `boost_pads`
- `ticks`
- `tracks`
- `events`

but currently populates almost all of them with empty placeholders.

That means the example is throwing away or failing to derive data that already
exists in `ReplayData`, including:

- `metadata_frames[*].seconds_remaining`
- `goal_events`
- `boost_pad_events`
- `touch_events`
- `player_stat_events`
- `demolish_infos`

#### Map metadata translation

The example reads `meta.map_name`, but the serialized `ReplayMeta` does not
currently provide `map_name`. It falls back to `"unknown"` and hardcodes:

- `map_type: "soccar"`
- `ball_type: "sphere"`

### Implicit Coordinate Transform Inside `player.js`

Before the extraction to `js/player`, the example did not translate coordinates
explicitly in `adaptFrameData()`, but `js/example/src/player.js` applied a
hidden scene transform:

- a root `axisFix` group is added with `x *= -1`
- DOM overlays also negate `x` before projection

The effective render-space mapping is therefore:

- `scene_x = -replay_x`
- `scene_y = replay_y`
- `scene_z = replay_z`

Because the raw quaternions are applied under that mirrored parent group, the
orientation transform is also implicit rather than documented in the data model.

### Current Shape Summary

The current example is therefore not "using `subtr-actor` directly". It is doing:

`ReplayData` -> ad hoc Ballchasing adapter -> legacy player globals -> three.js scene

with important behavior split between the adapter and hidden renderer internals.

## Problems With The Current Approach

- The main model is Ballchasing-shaped rather than `subtr-actor`-shaped.
- Time is synthetic instead of using `metadata_frames[*].time`.
- Team membership is not modeled explicitly enough.
- Map metadata is guessed.
- Camera settings require scraping replay header maps in JS.
- Boost pads, score, countdowns, and replay events are mostly discarded.
- `player.js` depends on globals, DOM patches, and CDN-loaded scripts.
- Rendering, playback state, normalization, and demo UI live in one file path.

## Proposed Library Shape

The new library should live in:

- `js/player/`

with the existing example becoming a consumer of that library.

### Library responsibilities

The library should own:

- replay normalization from WASM output into a typed playback model
- playback clock and seeking
- three.js scene construction and entity updates
- camera attachment state and tracked-player logic
- a small public imperative API for apps to control playback

The example app should own:

- file upload
- replay loading from bytes
- demo controls and app-specific UI

### Proposed internal layers

#### 1. Raw binding layer

Thin functions around the existing WASM package:

- initialize bindings
- validate replay bytes
- fetch `get_replay_frames_data()`

This layer should stay very small.

#### 2. Normalized replay model

Introduce a typed JS/TS model that stays close to `subtr-actor` instead of
converting into Ballchasing arrays.

The useful shape from the now-removed `js/three-player/src/replay-data.ts` was:

- separate normalization from rendering
- build a `ReplayModel`
- index players by stable ID
- use real frame times

We should keep that separation, but make the model even closer to the raw replay
data than the old clean-room attempt did.

Recommended direction:

- preserve raw replay units initially
- preserve the replay's native `z`-up convention internally
- preserve raw quaternions on the model
- attach explicitly derived convenience fields only when they are renderer-facing
  and deterministic

Examples of good derived fields:

- normalized playback time starting at zero
- stable player ID string
- explicit `isTeamZero`
- typed camera settings
- precomputed boost-active intervals

Examples of data that should remain raw:

- rigid body location
- rigid body rotation quaternion
- rigid body velocities

#### 3. Explicit scene transform boundary

If the renderer wants mirrored X, a different up axis, or scaling, that should
be expressed in one place only:

- a scene root transform or matrix utility
- a documented quaternion conversion function if needed

The replay model should not hide render-space transforms inside unrelated
normalization code.

#### 4. Renderer modules

Extract reusable pieces from the former `js/example/src/player.js` implementation
into ESM/TypeScript modules.

Likely reusable pieces:

- stadium mesh construction
- car mesh construction
- ball mesh construction
- trail/explosion materials and helpers
- playback interpolation helpers

Likely not worth carrying over directly:

- Ballchasing global event bus
- `window.ReplayPlayer`
- cookie-based settings helpers
- dynamic script loading
- DOM patching for missing shader scripts
- Ballchasing data contract assumptions

#### 5. Public player API

The library should expose something like:

- `loadReplayFromBytes()`
- `normalizeReplayData()`
- `ReplayPlayer`

with `ReplayPlayer` providing:

- `play()`
- `pause()`
- `seek(time)`
- `setAttachedPlayer(id | null)`
- `destroy()`
- snapshot/state subscriptions for UI integration

The example app can then become a thin wrapper around this API.

## Recommended Data Strategy

### What to keep from the example

- the example's stadium and mesh-building knowledge is still valuable
- the example proves that raw replay-space coordinates can drive a viewer
- the resize/fullscreen fixes from the old `js/example/src/player.js` are still
  useful

### What to stop doing

- stop adapting into `window.replayData`
- stop depending on Ballchasing globals as the main architecture
- stop synthesizing `frameIndex / 30`
- stop inferring metadata from missing fields

### What to keep from the old `js/three-player` work

The previous `js/three-player` package should not be revived wholesale, but it
contains good structural ideas:

- separate normalization from rendering
- use typed models
- derive stable player IDs
- use real replay times

Those ideas should be reused in the new `js/player/` package.

## `subtr-actor` Changes That Would Make This Cleaner

We can build a first pass with the current `get_replay_frames_data()` output, but
several small crate/binding changes would make the player library substantially
cleaner.

### 1. Add explicit team metadata to exported player info

Today JS has to infer team membership from which list a player came from.

Recommended change:

- extend serialized player metadata with `team` or `is_team_0`

That removes a fragile JS-side assumption.

### 2. Export typed camera/player settings

`src/stats/reducers.rs` already contains a typed `PlayerSettings` extractor for:

- `CameraFOV`
- `CameraHeight`
- `CameraPitch`
- `CameraDistance`
- `CameraStiffness`
- `CameraSwivelSpeed`
- `CameraTransitionSpeed`

Recommended change:

- surface those settings in a serialized replay/player-facing type instead of
  forcing JS to scrape `PlayerInfo.stats`

### 3. Export map metadata needed by the renderer

Recommended change:

- include explicit `map_name`
- include explicit `map_type`
- include explicit `ball_type` when relevant

This removes current placeholders and hardcoded defaults.

### 4. Export boost pad layout in a player-friendly form

`subtr-actor` already knows the standard soccar boost pad layout, but the JS
binding does not currently expose it in a convenient viewer-facing API.

Recommended change:

- expose boost pad layout through the crate and WASM binding
- ideally keyed by map/mode, not just standard soccar

This would let the player render pads and availability directly from
`boost_pad_events`.

### 5. Consider a dedicated player-facing export later

Not required for the first pass, but likely worth considering after the JS model
stabilizes:

- a dedicated `PlayerReplayData` or similar collector/binding output

That export would sit between raw `ReplayData` and the renderer and could carry
the exact metadata the player needs without Ballchasing baggage.

## Suggested Implementation Order

1. Create `js/player/` with types plus a normalization module only.
2. Convert the example app to consume that normalized model without Ballchasing.
3. Port reusable rendering primitives from `js/example/src/player.js` into modules.
4. Add crate/binding improvements for team metadata, camera settings, map metadata,
   and boost pad layout.
5. Fill in score, countdown, goal markers, and boost pad rendering from real replay data.

## Immediate Working Agreement

Before starting implementation, treat this document as the contract:

- the player library lives under `js/player/`
- the internal source of truth is a typed model derived directly from
  `subtr-actor` replay output
- all scene-space transforms must be explicit
- the example becomes a demo, not the implementation architecture
