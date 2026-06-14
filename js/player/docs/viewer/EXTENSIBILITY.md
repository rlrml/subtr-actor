# Extensibility — the plugin seam

`@rlrml/player` is deliberately **bare**, like [`@rlrml/player`](../../player). The core
is "parse a replay (subtr-actor) and render it (three.js)" — nothing more. Everything
above raw playback — scoreboard / HUD, name tags, killfeed, overlays, telestrator,
pings, clip recording, collaboration, dev tooling — is a **plugin**, not core.

This package was seeded from a full web app, so all of those features existed here as
managers/components. They were removed (see the ledger below) to get back to a focused
player. They are meant to return as plugins on a hook system modeled directly on
`@rlrml/player`'s.

> **Status:** the contract below is **implemented**. The host is `ViewerPlayer`
> (`src/ViewerPlayer.ts`): a bare core owning scene/arena/actors + the playback clock,
> dispatching these hooks. Types live in `src/types.ts`; the public entry is
> `createViewer()` (`src/lib.ts`). Built-in plugins so far:
> `createNameTagPlugin()` (wraps NameTagManager — the template for new plugins),
> `createBoostPadsPlugin()` (the original GameEngine pad rendering fed by
> subtr-actor's resolved layout + pickup events), and `createCameraPlugin()`
> (the full original CameraManager behind one handle: orbit / free-fly /
> ball-orbit / RL-style follow modes, recorded-or-forced ball cam, RL camera
> settings — follow mode seeds from the player's recorded replay preset —
> and the horizontal→vertical FOV conversion), plus `createScoredTextPlugin()`
> (the original centered "&lt;PLAYER&gt; SCORED !!" goal banner — a DOM overlay
> auto-installed by `createViewer`, default on, opt out with `scoredText: false`).
> Trail effects
> (boost/supersonic/ball) are wired in the core via the real EffectsManager
> (`effects: false` opts out), and the core render loop keeps the original
> GameEngine per-frame path (animation mixer, boost/supersonic particle state,
> wheel rotations, seek-time resets). Goal explosions are now wired too: the
> core feeds the replay's goal events to EffectsManager and fires the
> team-colored explosion as playback crosses each goal.

## The `ViewerPlugin` contract

Mirror `@rlrml/player`'s plugin model (`js/player/src/types.ts`,
`js/player/src/player.ts`): a core class holds an ordered list of installed plugins and
calls their lifecycle hooks. Plugins are added via `options.plugins` at construction or
`player.addPlugin(def)` / `player.removePlugin(id)` at runtime, exactly as
`ReplayPlayer` does.

```ts
interface ViewerPlugin {
  id: string;
  setup?(ctx: ViewerPluginContext): void; // install: attach meshes/DOM, subscribe
  onStateChange?(ctx: ViewerPluginStateContext): void; // play/pause/seek/camera changes
  beforeRender?(ctx: ViewerRenderContext): void; // per-frame, after positions resolve
  teardown?(ctx: ViewerPluginContext): void; // uninstall: dispose everything created
}

type ViewerPluginFactory = () => ViewerPlugin;
type ViewerPluginDefinition = ViewerPlugin | ViewerPluginFactory;

interface ViewerPluginContext {
  player: ViewerPlayer; // the core; exposes seek/play/pause/state + the adapter
  scene: THREE.Scene; // for plugins that add 3D objects
  camera: THREE.Camera; // for screen-space projection (HUD/indicators)
  renderer: THREE.WebGLRenderer;
  container: HTMLElement; // for plugins that add DOM overlays (HUD, scoreboard)
}

interface ViewerRenderContext extends ViewerPluginContext {
  time: number; // current playback time (s)
  ball: BallRenderState; // resolved position/rotation/velocity this frame
  cars: CarRenderState[]; // per-player resolved transform + boost/visible/team
}
```

`BallRenderState` / `CarRenderState` are the per-frame values `SubtrActorPlayer`
already produces (`src/adapter/SubtrActorPlayer.ts`: `ball`, `getAllPlayers()`). The
core just needs to pass them into `beforeRender` so plugins read transforms without
re-querying.

Convention from `@rlrml/player`: ship each feature as a `createXxxPlugin(options)`
factory exported from the package root, so consumers compose only what they want:

```ts
new ViewerPlayer(container, bytes, {
  plugins: [createScoreboardPlugin(), createNameTagPlugin()],
});
```

### Data the core already exposes for plugins

`SubtrActorPlayer` carries stubbed-but-typed getters that overlay plugins will read:
`getEvents()`, `getEventsInRange()`, `getPlayerStatsTimelines()`,
`getGameEventTimeline()`, `getGameTimeMap()`, `getCountdownEvents()`,
`getTextOverlaysAt()`, `getGamePhaseAt()`. They return empty for now; filling them from
subtr-actor (it already produces this data — see `js/stat-evaluation-player` and the
stats timeline) is what unlocks the overlay plugins below.

## Removed-feature ledger

Each row is a capability that was in this package and was removed to keep the core bare.
"Hooks" = which `ViewerPlugin` hook a re-implementation would primarily use. "Needs" =
data/wiring it depends on.

| Removed (was)                                       | What it did                                                         | Hooks                                          | Needs                                                                                |
| --------------------------------------------------- | ------------------------------------------------------------------- | ---------------------------------------------- | ------------------------------------------------------------------------------------ |
| **Scoreboard / HUD** (`components/`)                | Team scores, clock, player panels                                   | `setup` (DOM), `onStateChange`, `beforeRender` | `getGameTimeMap`, score events from `getEvents`; container overlay                   |
| **NameTagManager** _(now `createNameTagPlugin()`)_  | Floating player name labels                                         | `beforeRender`                                 | per-car transform + camera projection                                                |
| **Killfeed** (`components/Killfeed`)                | Demolition feed                                                     | `onStateChange`                                | demolish events from `getEvents`                                                     |
| **OffscreenIndicatorManager**                       | 2D arrows to offscreen ball/cars                                    | `beforeRender`                                 | camera + ball/car positions; container overlay                                       |
| **DrawingManager** (telestrator)                    | Freehand drawing over the scene                                     | `setup` (canvas), `beforeRender`               | container overlay; input; (collab) a shared transport                                |
| **PingManager**                                     | 3D world-space ping markers                                         | `setup`, `onStateChange`                       | scene; (collab) shared transport                                                     |
| **collab/** + **ClipRecording/ClipPlaybackManager** | Shared sessions, clip record/replay (camera keyframes, annotations) | `onStateChange`, `beforeRender`                | a transport (was a backend socket); camera state from the core                       |
| **EnvironmentManager**                              | Swap stadium meshes / lighting presets                              | `setup`                                        | an asset source — bundle GLBs in `public/` or a pluggable loader (was a backend API) |
| **DevToolsManager / KeyframeVisualizer**            | Debug panels, keyframe viz                                          | `setup`, `beforeRender`                        | dev-only; scene + raw timelines                                                      |

Notes on the two non-plugin removals:

- **`ReplayLoader` / `framework/`** were a _second replay parser_ (a JS boxcars
  pipeline). The `src/adapter` (`SubtrActorPlayer` + `wasm.ts`) replaces them entirely —
  subtr-actor is the sole data source. These are gone for good, not future plugins.
  Their only still-useful artifacts were two data constants, now lifted into
  `src/data/hitboxes.js`.
- Anything that loaded from a **backend API** (environments, assets, dev tools, collab
  sockets) must become _client-side or pluggable_ to fit the "embeddable, fully
  client-side" goal: bundle the assets, or let the consumer supply a transport.
