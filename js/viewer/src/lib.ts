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
import { parseReplay } from "./adapter/wasm.js";
import { SubtrActorPlayer } from "./adapter/SubtrActorPlayer.js";
import { ViewerPlayer } from "./ViewerPlayer.js";
import type { ViewerOptions } from "./types.js";

/** Parse raw `.replay` bytes and mount a player into `container`. */
export async function createViewer(
  container: HTMLElement,
  replayBytes: Uint8Array,
  options: ViewerOptions = {},
): Promise<ViewerPlayer> {
  const raw = await parseReplay(replayBytes);
  const adapter = new SubtrActorPlayer(raw as never);
  return new ViewerPlayer(container, adapter, options);
}

export { ViewerPlayer } from "./ViewerPlayer.js";
export { SubtrActorPlayer } from "./adapter/SubtrActorPlayer.js";
export type { RecordedCameraSettings, ViewerPlayerInfo } from "./adapter/SubtrActorPlayer.js";
export { parseReplay } from "./adapter/wasm.js";
export { createNameTagPlugin } from "./plugins/name-tags.js";
export { createBoostPadsPlugin } from "./plugins/boost-pads.js";
export { createCameraPlugin } from "./plugins/camera.js";
export type {
  CameraPlugin,
  CameraPluginMode,
  CameraPluginOptions,
  CameraSettings,
} from "./plugins/camera.js";
export type * from "./types.js";
