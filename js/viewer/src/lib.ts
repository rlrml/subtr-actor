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
import type { ViewerOptions } from "./types.js";

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
  const { replay, raw } = await loadReplay(replayBytes);
  const adapter = new SubtrActorPlayer(raw as never);
  return new ViewerPlayer(container, adapter, options, replay);
}

export { ViewerPlayer } from "./ViewerPlayer.js";
export { SubtrActorPlayer } from "./adapter/SubtrActorPlayer.js";
export type { RecordedCameraSettings, ViewerPlayerInfo } from "./adapter/SubtrActorPlayer.js";
export { loadReplay, parseReplay } from "./adapter/wasm.js";
export type { ReplayLoadResult, ReplayModel, ReplayScene } from "@rlrml/player";
export { createNameTagPlugin } from "./plugins/name-tags.js";
export { createBoostPadsPlugin } from "./plugins/boost-pads.js";
// Phase 3 parity: run @rlrml/player DOM plugins on the viewer, e.g.
//   viewer.addPlugin(fromReplayPlayerPlugin(createTimelineOverlayPlugin()))
export { fromReplayPlayerPlugin } from "./plugins/replay-player-bridge.js";
export { createTimelineOverlayPlugin, timelineEventSeekTime } from "@rlrml/player";
export type { TimelineOverlayPlugin, TimelineOverlayPluginOptions } from "@rlrml/player";
export { createCameraPlugin } from "./plugins/camera.js";
export type {
  CameraPlugin,
  CameraPluginMode,
  CameraPluginOptions,
  CameraSettings,
} from "./plugins/camera.js";
export type * from "./types.js";
