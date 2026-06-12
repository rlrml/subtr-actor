/**
 * @rlrml/viewer public surface.
 *
 * The one-call embed path:
 *
 *   import { createViewer, createNameTagPlugin } from "@rlrml/viewer";
 *   const viewer = await createViewer(container, replayBytes, {
 *     autoplay: true,
 *     plugins: [createNameTagPlugin()],
 *   });
 *
 * Fully client-side: bytes are parsed in the browser via subtr-actor's WASM and
 * rendered with three.js. No backend.
 */
import { loadReplay } from "./adapter/wasm.js";
import { SubtrActorPlayer } from "./adapter/SubtrActorPlayer.js";
import { ViewerPlayer } from "./ViewerPlayer.js";
import { createCameraPlugin } from "./plugins/camera.js";
import type { ViewerOptions } from "./types.js";
import type { ReplayLoadResult } from "@rlrml/player";

/**
 * Parse raw `.replay` bytes and mount a player into `container`.
 *
 * One WASM parse feeds both data layers: the adapter (renderer timelines) and
 * `viewer.replay` (@rlrml/player's `ReplayModel` — docs/PLAYER_PARITY.md
 * Phase 2).
 */
export async function createViewer(
  container: HTMLElement,
  replayBytes: Uint8Array,
  options: ViewerOptions = {},
): Promise<ViewerPlayer> {
  return createViewerFromParsed(container, await loadReplay(replayBytes), options);
}

/**
 * Mount a player from an already-parsed replay (`{ raw, replay }`, the shape
 * `loadReplay` / @rlrml/player's `loadReplayFromBytes` return). Synchronous —
 * no WASM call. Use this when the host app already parsed the replay (e.g. in
 * a worker with progress reporting, like js/stat-evaluation-player) so the
 * bytes aren't parsed twice.
 */
export function createViewerFromParsed(
  container: HTMLElement,
  parsed: ReplayLoadResult,
  options: ViewerOptions = {},
): ViewerPlayer {
  const adapter = new SubtrActorPlayer(parsed.raw as never);
  const viewer = new ViewerPlayer(container, adapter, options, parsed.replay);
  // @rlrml/player parity: ReplayPlayer's camera surface (follow / ballcam /
  // distance scale / custom settings) works out of the box, so the factory
  // installs the camera plugin by default — the bare ViewerPlayer core only
  // delegates to a plugin with id "camera". Consumers that pass their own
  // camera plugin (same id) win; installing late is fine because the core
  // pushes any parity camera state already set onto it (installPlugin).
  if (!viewer.getPlugins().some((plugin) => plugin.id === "camera")) {
    viewer.addPlugin(createCameraPlugin());
  }
  return viewer;
}

export { ViewerPlayer } from "./ViewerPlayer.js";
export { SubtrActorPlayer } from "./adapter/SubtrActorPlayer.js";
export type { RecordedCameraSettings, ViewerPlayerInfo } from "./adapter/SubtrActorPlayer.js";
export { loadReplay, parseReplay } from "./adapter/wasm.js";
export type { ReplayLoadResult, ReplayModel, ReplayScene } from "@rlrml/player";
export { createNameTagPlugin } from "./plugins/name-tags.js";
export { createBoostPadsPlugin } from "./plugins/boost-pads.js";
// Phase 3 parity: run @rlrml/player plugins on the viewer, e.g.
//   viewer.addPlugin(fromReplayPlayerPlugin(createTimelineOverlayPlugin()))
export { fromReplayPlayerPlugin } from "./plugins/replay-player-bridge.js";
export {
  createBoostPadOverlayPlugin,
  createBoostPickupAnimationPlugin,
  createCanvasRecorderPlugin,
  createTimelineOverlayPlugin,
  timelineEventSeekTime,
} from "@rlrml/player";
export type {
  BoostPickupAnimationPluginOptions,
  CanvasRecorderPlugin,
  CanvasRecorderPluginOptions,
  TimelineOverlayPlugin,
  TimelineOverlayPluginOptions,
} from "@rlrml/player";
export { createCameraPlugin } from "./plugins/camera.js";
export type {
  CameraPlugin,
  CameraPluginMode,
  CameraPluginOptions,
  CameraSettings,
} from "./plugins/camera.js";
export type * from "./types.js";
