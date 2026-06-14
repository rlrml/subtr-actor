/**
 * @rlrml/player public surface.
 *
 * The one-call embed path:
 *
 *   import { createViewer, createNameTagPlugin } from "@rlrml/player";
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
import { createScoredTextPlugin } from "./plugins/scored-text.js";
import type { ViewerOptions } from "./types.js";
import type { ReplayLoadResult } from "../types";

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
  const adapter = new SubtrActorPlayer(parsed.raw as never, {
    motionSmoothing: options.motionSmoothing,
    smoothingBlendFactor: options.smoothingBlendFactor,
    smoothingAnchorInterval: options.smoothingAnchorInterval,
    timelineCompaction: options.timelineCompaction,
    disableFrameFiltering: options.disableFrameFiltering,
  });
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
  // The original ballcam "<PLAYER> SCORED !!" goal banner ships on by default
  // (opt out with `scoredText: false`). Skipped when the consumer already
  // installed their own scored-text plugin (same id), so a custom-configured
  // banner wins.
  if (
    options.scoredText !== false &&
    !viewer.getPlugins().some((plugin) => plugin.id === "scored-text")
  ) {
    viewer.addPlugin(createScoredTextPlugin());
  }
  return viewer;
}

export { ViewerPlayer } from "./ViewerPlayer.js";
export { SubtrActorPlayer } from "./adapter/SubtrActorPlayer.js";
export { getViewerAssetBase, resolveViewerAssetUrl, setViewerAssetBase } from "./asset-url.js";
export type {
  RecordedCameraSettings,
  SubtrActorPlayerOptions,
  ViewerPlayerInfo,
} from "./adapter/SubtrActorPlayer.js";
export { loadReplay, parseReplay } from "./adapter/wasm.js";
// Skybox environments (background + image-based lighting). The built-in "space"
// is the default; register more or pass a descriptor inline. See environments.ts.
export {
  DEFAULT_ENVIRONMENT_ID,
  listEnvironments,
  registerEnvironment,
  resolveEnvironment,
} from "./environments.js";
export type { ViewerEnvironment, ViewerEnvironmentSpec } from "./environments.js";
export type { ReplayLoadResult, ReplayModel } from "../types";
export type { ReplayScene } from "../scene";
export { createNameTagPlugin } from "./plugins/name-tags.js";
export { createBoostPadsPlugin } from "./plugins/boost-pads.js";
export { createFpsOverlayPlugin } from "./plugins/fps-overlay.js";
export type { FpsOverlayOptions, FpsSample } from "./plugins/fps-overlay.js";
export { createScoredTextPlugin } from "./plugins/scored-text.js";
export type { ScoredTextOverlayOptions } from "./plugins/scored-text.js";
// Phase 3 parity: run @rlrml/player plugins on the viewer, e.g.
//   viewer.addPlugin(fromReplayPlayerPlugin(createTimelineOverlayPlugin()))
export { fromReplayPlayerPlugin } from "./plugins/replay-player-bridge.js";
export { BOOST_RAW_MAX, boostAmountToPercent, boostPercentToAmount } from "../boost-units";
export { createBoostPadOverlayPlugin } from "../boost-pad-overlay";
export { createBoostPickupAnimationPlugin } from "../boost-pickup-animation";
export { createCanvasRecorderPlugin } from "../canvas-recorder";
export { createTimelineOverlayPlugin, timelineEventSeekTime } from "../timeline-overlay";
export type { BoostPickupAnimationPluginOptions } from "../boost-pickup-animation";
export type { CanvasRecorderPlugin, CanvasRecorderPluginOptions } from "../canvas-recorder";
export type { TimelineOverlayPlugin, TimelineOverlayPluginOptions } from "../timeline-overlay";
export { createCameraPlugin } from "./plugins/camera.js";
export type {
  CameraPlugin,
  CameraPluginMode,
  CameraPluginOptions,
  CameraSettings,
} from "./plugins/camera.js";
export type * from "./types.js";
