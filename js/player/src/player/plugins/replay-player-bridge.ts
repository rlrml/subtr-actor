/**
 * Bridge: mount an `@rlrml/player` `ReplayPlayerPlugin` on a `ReplayPlayer`.
 *
 * Phase 3 of docs/player/PLAYER_PARITY.md. The two plugin contracts are structurally
 * aligned on everything a plugin reads — `player` (control surface + timeline
 * projection), `replay` (the shared `ReplayModel`), `state`, `options`,
 * `container` — so @rlrml/player plugins run unmodified:
 *
 *   player.addPlugin(fromReplayPlayerPlugin(createTimelineOverlayPlugin()));
 *
 * `context.scene` is the player's `ReplayScene`-shaped `sceneState`
 * (ReplayPlayer.sceneState): `scene`/`camera`/`renderer`/`controls`/`resize`
 * are real, `replayRoot` shares @rlrml/player's UE-coordinate convention, and
 * `ballMesh`/`playerMeshes` view this renderer's live actors. The
 * schematic-player internals (body meshes, hitboxes, boost trails/meters,
 * demo indicators) are empty maps.
 *
 * `beforeRender` receives a synthesized `ReplayPlayerRenderContext`, computed
 * from the shared `ReplayModel` with @rlrml/player's own exported math
 * (`getFrameWindow`, `interpolatePosition`, …), so frame windows, ball/player
 * samples, interpolated positions, and boost fractions match what
 * `ReplayPlayer.render()` would hand the plugin. Track `mesh`es are this
 * renderer's live car objects (UE-coordinate `replayRoot` children, like the
 * schematic player's); `boostTrail` is always null here, and `ballPosition`
 * is in THIS renderer's world space (Y-up — scene-level math doesn't port,
 * see docs/player/PLAYER_PARITY.md).
 */
import * as THREE from "three";
import { getActiveDemoEvent, isPlayerSamplePresent } from "../../player-helpers";
import { interpolatePosition } from "../../player-internals/spatial";
import { getFrameWindow } from "../../player-internals/timeline";
import type {
  ReplayModel,
  ReplayPlayerPlugin,
  ReplayPlayerPluginContext,
  ReplayPlayerPluginStateContext,
  ReplayPlayerRenderContext,
  ReplayPlayerRenderTrackContext,
} from "../../types";
import type { ReplayScene } from "../../scene";
import type {
  PlayerPlugin,
  PlayerPluginContext,
  PlayerPluginStateContext,
  PlayerRenderContext,
} from "../types.js";

function toPlayerContext(
  context: PlayerPluginContext,
  pluginId: string,
): ReplayPlayerPluginContext {
  if (!context.replay) {
    throw new Error(
      `[player] cannot run @rlrml/player plugin "${pluginId}" without a ReplayModel — ` +
        "construct the player via createPlayer(), which always provides one.",
    );
  }
  return {
    // ReplayPlayer implements ReplayPlayer's control + timeline surface
    // (docs/player/PLAYER_PARITY.md), which is all a DOM plugin calls.
    player: context.player as unknown as ReplayPlayerPluginContext["player"],
    replay: context.replay,
    scene: context.player.sceneState,
    container: context.container,
    options: context.options as ReplayPlayerPluginContext["options"],
  };
}

function toPlayerStateContext(
  context: PlayerPluginStateContext,
  pluginId: string,
): ReplayPlayerPluginStateContext {
  return { ...toPlayerContext(context, pluginId), state: context.state };
}

/**
 * Mirror of `ReplayPlayer.render()`'s per-player track construction
 * (js/player/src/player.ts): `interpolatedPosition`/`boostFraction` stay
 * null/0 on every early-out branch (no mesh, no interpolable position, active
 * demo, sample absent) and only populate for a fully rendered player.
 */
function toRenderTrack(
  player: ReplayModel["players"][number],
  replay: ReplayModel,
  sceneState: ReplayScene,
  frameWindow: { frameIndex: number; nextFrameIndex: number; alpha: number },
  currentTime: number,
): ReplayPlayerRenderTrackContext {
  const frame = player.frames[frameWindow.frameIndex] ?? null;
  const nextFrame = player.frames[frameWindow.nextFrameIndex] ?? frame;
  const track: ReplayPlayerRenderTrackContext = {
    track: player,
    mesh: sceneState.playerMeshes.get(player.id) ?? null,
    boostTrail: sceneState.playerBoostTrails.get(player.id) ?? null,
    frame,
    nextFrame,
    interpolatedPosition: null,
    boostFraction: 0,
  };
  if (!track.mesh) {
    return track;
  }
  const interpolated = interpolatePosition(
    frame?.position ?? null,
    nextFrame?.position ?? null,
    frameWindow.alpha,
  );
  if (
    !interpolated ||
    getActiveDemoEvent(replay.timelineEvents, player.id, currentTime) ||
    !isPlayerSamplePresent(frame)
  ) {
    return track;
  }
  track.interpolatedPosition = interpolated;
  const currentBoostFraction = frame?.boostFraction ?? 0;
  const nextBoostFraction = nextFrame?.boostFraction ?? currentBoostFraction;
  track.boostFraction = THREE.MathUtils.lerp(
    currentBoostFraction,
    nextBoostFraction,
    frameWindow.alpha,
  );
  return track;
}

function toPlayerRenderContext(
  context: PlayerRenderContext,
  pluginId: string,
): ReplayPlayerRenderContext {
  const stateContext = toPlayerStateContext(context, pluginId);
  const replay = stateContext.replay;
  const sceneState = stateContext.scene;
  // Recompute the frame window with @rlrml/player's own math over the shared
  // ReplayModel so the synthesized context is self-consistent and matches
  // ReplayPlayer.render() exactly (the player's FrameRenderInfo is computed
  // off the adapter's aligned-but-separate time index).
  const frameWindow = getFrameWindow(replay, context.currentTime);
  const ballFrame = replay.ballFrames[frameWindow.frameIndex] ?? null;
  const nextBallFrame = replay.ballFrames[frameWindow.nextFrameIndex] ?? ballFrame;
  const interpolatedBallPosition = interpolatePosition(
    ballFrame?.position ?? null,
    nextBallFrame?.position ?? null,
    frameWindow.alpha,
  );
  // @rlrml/player hands plugins the ball's world-space position; the analog
  // here is replayRoot's UE→world mapping (Y-up in this renderer).
  const ballPosition = interpolatedBallPosition
    ? sceneState.replayRoot.localToWorld(
        new THREE.Vector3(
          interpolatedBallPosition.x,
          interpolatedBallPosition.y,
          interpolatedBallPosition.z,
        ),
      )
    : null;
  return {
    ...stateContext,
    frameIndex: frameWindow.frameIndex,
    nextFrameIndex: frameWindow.nextFrameIndex,
    alpha: frameWindow.alpha,
    currentTime: context.currentTime,
    frame: replay.frames[frameWindow.frameIndex] ?? null,
    nextFrame: replay.frames[frameWindow.nextFrameIndex] ?? null,
    ballFrame,
    nextBallFrame,
    ballPosition,
    players: replay.players.map((player) =>
      toRenderTrack(player, replay, sceneState, frameWindow, context.currentTime),
    ),
  };
}

/**
 * Wrap a `ReplayPlayerPlugin` (or one with extra members, e.g.
 * `TimelineOverlayPlugin`) as a `PlayerPlugin`. Extra members survive on the
 * returned object so handles like `overlay.setVisible()` keep working.
 */
export function fromReplayPlayerPlugin<P extends ReplayPlayerPlugin>(
  plugin: P,
): PlayerPlugin & Omit<P, keyof ReplayPlayerPlugin> {
  return {
    ...plugin,
    setup: plugin.setup
      ? (context: PlayerPluginContext): void => {
          plugin.setup?.(toPlayerContext(context, plugin.id));
        }
      : undefined,
    onStateChange: plugin.onStateChange
      ? (context: PlayerPluginStateContext): void => {
          plugin.onStateChange?.(toPlayerStateContext(context, plugin.id));
        }
      : undefined,
    beforeRender: plugin.beforeRender
      ? (context: PlayerRenderContext): void => {
          plugin.beforeRender?.(toPlayerRenderContext(context, plugin.id));
        }
      : undefined,
    teardown: plugin.teardown
      ? (context: PlayerPluginContext): void => {
          plugin.teardown?.(toPlayerContext(context, plugin.id));
        }
      : undefined,
  };
}
