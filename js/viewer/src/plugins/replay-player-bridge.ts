/**
 * Bridge: mount an `@rlrml/player` `ReplayPlayerPlugin` on a `ViewerPlayer`.
 *
 * Phase 3 of docs/PLAYER_PARITY.md. The two plugin contracts are structurally
 * aligned on everything a DOM plugin reads — `player` (control surface +
 * timeline projection), `replay` (the shared `ReplayModel`), `state`,
 * `options`, `container` — so DOM-only plugins (the timeline overlay,
 * ballchasing overlay, …) run unmodified:
 *
 *   viewer.addPlugin(fromReplayPlayerPlugin(createTimelineOverlayPlugin()));
 *
 * `context.scene` is the viewer's `ReplayScene`-shaped `sceneState`
 * (ViewerPlayer.sceneState): `scene`/`camera`/`renderer`/`controls`/`resize`
 * are real, `replayRoot` shares @rlrml/player's UE-coordinate convention, and
 * `ballMesh`/`playerMeshes` view this renderer's live actors. The
 * schematic-player internals (body meshes, hitboxes, boost trails/meters,
 * demo indicators) are empty maps.
 *
 * One part deliberately does NOT bridge, and fails loudly instead of silently:
 * `beforeRender` receives renderer-internal frame state (`ballPosition`,
 * per-track meshes) that can't be faked faithfully. Bridging a plugin that
 * defines it throws at install time.
 */
import type {
  ReplayPlayerPlugin,
  ReplayPlayerPluginContext,
  ReplayPlayerPluginStateContext,
} from "@rlrml/player";
import type { ViewerPlugin, ViewerPluginContext, ViewerPluginStateContext } from "../types.js";

function toPlayerContext(
  context: ViewerPluginContext,
  pluginId: string,
): ReplayPlayerPluginContext {
  if (!context.replay) {
    throw new Error(
      `[viewer] cannot run @rlrml/player plugin "${pluginId}" without a ReplayModel — ` +
        "construct the viewer via createViewer(), which always provides one.",
    );
  }
  return {
    // ViewerPlayer implements ReplayPlayer's control + timeline surface
    // (docs/PLAYER_PARITY.md), which is all a DOM plugin calls.
    player: context.player as unknown as ReplayPlayerPluginContext["player"],
    replay: context.replay,
    scene: context.player.sceneState,
    container: context.container,
    options: context.options as ReplayPlayerPluginContext["options"],
  };
}

function toPlayerStateContext(
  context: ViewerPluginStateContext,
  pluginId: string,
): ReplayPlayerPluginStateContext {
  return { ...toPlayerContext(context, pluginId), state: context.state };
}

/**
 * Wrap a `ReplayPlayerPlugin` (or one with extra members, e.g.
 * `TimelineOverlayPlugin`) as a `ViewerPlugin`. Extra members survive on the
 * returned object so handles like `overlay.setVisible()` keep working.
 */
export function fromReplayPlayerPlugin<P extends ReplayPlayerPlugin>(
  plugin: P,
): ViewerPlugin & Omit<P, keyof ReplayPlayerPlugin> {
  if (plugin.beforeRender) {
    throw new Error(
      `[viewer] @rlrml/player plugin "${plugin.id}" defines beforeRender — its render ` +
        "context is renderer-internal and can't be bridged. Port it as a native ViewerPlugin.",
    );
  }
  return {
    ...plugin,
    setup: plugin.setup
      ? (context: ViewerPluginContext): void => {
          plugin.setup?.(toPlayerContext(context, plugin.id));
        }
      : undefined,
    onStateChange: plugin.onStateChange
      ? (context: ViewerPluginStateContext): void => {
          plugin.onStateChange?.(toPlayerStateContext(context, plugin.id));
        }
      : undefined,
    beforeRender: undefined,
    teardown: plugin.teardown
      ? (context: ViewerPluginContext): void => {
          plugin.teardown?.(toPlayerContext(context, plugin.id));
        }
      : undefined,
  };
}
