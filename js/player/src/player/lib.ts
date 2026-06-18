/**
 * @rlrml/player public surface.
 *
 * The one-call embed path:
 *
 *   import { createPlayer, createNameTagPlugin } from "@rlrml/player";
 *   const player = await createPlayer(container, replayBytes, {
 *     autoplay: true,
 *     plugins: [createNameTagPlugin()],
 *   });
 *
 * Fully client-side: bytes are parsed in the browser via subtr-actor's WASM and
 * rendered with three.js. No backend.
 */
import { loadReplay } from "./adapter/wasm.js";
import { SubtrActorPlayer } from "./adapter/SubtrActorPlayer.js";
import { ReplayPlayer } from "./ReplayPlayer.js";
import { createCameraPlugin } from "./plugins/camera.js";
import type { PlayerOptions } from "./types.js";
import type { ReplayLoadResult } from "../types";

/**
 * Parse raw `.replay` bytes and mount a player into `container`.
 *
 * One WASM parse feeds both data layers: the adapter (renderer timelines) and
 * `player.replay` (@rlrml/player's `ReplayModel` — docs/player/PLAYER_PARITY.md
 * Phase 2).
 */
export async function createPlayer(
  container: HTMLElement,
  replayBytes: Uint8Array,
  options: PlayerOptions = {},
): Promise<ReplayPlayer> {
  return createPlayerFromParsed(container, await loadReplay(replayBytes), options);
}

/**
 * Mount a player from an already-parsed replay (`{ raw, replay }`, the shape
 * `loadReplay` / @rlrml/player's `loadReplayFromBytes` return). Synchronous —
 * no WASM call. Use this when the host app already parsed the replay (e.g. in
 * a worker with progress reporting, like js/stat-evaluation-player) so the
 * bytes aren't parsed twice.
 */
export function createPlayerFromParsed(
  container: HTMLElement,
  parsed: ReplayLoadResult,
  options: PlayerOptions = {},
): ReplayPlayer {
  const adapter = new SubtrActorPlayer(parsed.raw as never, {
    motionSmoothing: options.motionSmoothing,
    smoothingBlendFactor: options.smoothingBlendFactor,
    smoothingAnchorInterval: options.smoothingAnchorInterval,
    timelineCompaction: options.timelineCompaction,
    disableFrameFiltering: options.disableFrameFiltering,
  });
  const player = new ReplayPlayer(container, adapter, options, parsed.replay);
  // @rlrml/player parity: ReplayPlayer's camera surface (follow / ballcam /
  // distance scale / custom settings) works out of the box, so the factory
  // installs the camera plugin by default — the bare ReplayPlayer core only
  // delegates to a plugin with id "camera". Consumers that pass their own
  // camera plugin (same id) win; installing late is fine because the core
  // pushes any parity camera state already set onto it (installPlugin).
  if (!player.getPlugins().some((plugin) => plugin.id === "camera")) {
    player.addPlugin(createCameraPlugin());
  }
  return player;
}

export { ReplayPlayer } from "./ReplayPlayer.js";
export { SubtrActorPlayer } from "./adapter/SubtrActorPlayer.js";
export { getPlayerAssetBase, resolvePlayerAssetUrl, setPlayerAssetBase } from "./asset-url.js";
export type {
  RecordedCameraSettings,
  SubtrActorPlayerOptions,
  ReplayPlayerInfo,
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
export type { PlayerEnvironment, PlayerEnvironmentSpec } from "./environments.js";
export type { ReplayLoadResult, ReplayModel } from "../types";
export type { ReplayScene } from "../scene";
export { createNameTagPlugin } from "./plugins/name-tags.js";
export { createBoostPadsPlugin } from "./plugins/boost-pads.js";
export { createFpsOverlayPlugin } from "./plugins/fps-overlay.js";
export type { FpsOverlayOptions, FpsSample } from "./plugins/fps-overlay.js";
export { createScoredTextPlugin } from "./plugins/scored-text.js";
export type { ScoredTextOverlayOptions } from "./plugins/scored-text.js";
// Phase 3 parity: run @rlrml/player plugins on the player, e.g.
//   player.addPlugin(fromReplayPlayerPlugin(createTimelineOverlayPlugin()))
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
