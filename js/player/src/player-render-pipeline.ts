import * as THREE from "three";
import type { ReplayScene } from "./scene";
import type { ReplayPlayer } from "./player";
import { getFrameWindow } from "./player-internals/timeline";
import {
  updateAttachedCamera,
  updateFreeCameraTransition,
} from "./player-internals/spatial";
import { getActiveDemoEvent } from "./player-render-effects";
import { renderReplayFrameScene } from "./player-render-frame";
import type {
  BeforeRenderCallback,
  FrameRenderInfo,
  ReplayCameraViewMode,
  ReplayModel,
  ReplayPlayerOptions,
  ReplayPlayerPlugin,
  ReplayPlayerRenderContext,
  ReplayPlayerRenderTrackContext,
  ReplayPlayerState,
  Vec3,
} from "./types";

export interface FreeCameraTransition {
  position: THREE.Vector3;
  target: THREE.Vector3;
  up: THREE.Vector3;
  fov: number;
}

export interface ReplayPlayerRenderPipelineOptions {
  readonly replay: ReplayModel;
  readonly player: ReplayPlayer;
  readonly container: HTMLElement;
  readonly playerOptions: ReplayPlayerOptions;
  readonly sceneState: ReplayScene;
  readonly fieldScale: number;
  readonly currentTime: number;
  readonly boostMeterEnabled: boolean;
  readonly cameraViewMode: ReplayCameraViewMode;
  readonly attachedPlayerId: string | null;
  readonly ballCamEnabled: boolean;
  readonly cameraDistanceScale: number;
  readonly customCameraSettings: ReplayPlayerState["customCameraSettings"];
  readonly desiredCameraPosition: THREE.Vector3;
  readonly desiredLookTarget: THREE.Vector3;
  readonly freeCameraTransition: FreeCameraTransition | null;
  readonly beforeRenderCallbacks: readonly BeforeRenderCallback[];
  readonly plugins: readonly ReplayPlayerPlugin[];
  getState(): ReplayPlayerState;
}

export function renderReplayPlayerFrame(
  options: ReplayPlayerRenderPipelineOptions,
): FreeCameraTransition | null {
  const frameWindow = getFrameWindow(options.replay, options.currentTime);
  const frameIndex = frameWindow.frameIndex;
  const {
    ballFrame,
    nextBallFrame,
    ballPosition,
    ballWorldPosition,
    players: renderPlayers,
  } =
    renderReplayFrameScene({
      replay: options.replay,
      sceneState: options.sceneState,
      frameWindow,
      fieldScale: options.fieldScale,
      currentTime: options.currentTime,
      boostMeterEnabled: options.boostMeterEnabled,
    });

  updateAttachedCamera({
    sceneState: options.sceneState,
    replay: options.replay,
    fieldScale: options.fieldScale,
    cameraViewMode: options.cameraViewMode,
    attachedPlayerId: options.attachedPlayerId,
    ballCamEnabled: options.ballCamEnabled,
    cameraDistanceScale: options.cameraDistanceScale,
    customCameraSettings: options.customCameraSettings,
    frameIndex,
    attachedPlayerUnavailable:
      options.attachedPlayerId !== null &&
      getActiveDemoEvent(options.replay, options.attachedPlayerId, options.currentTime) !== null,
    ballPosition: ballWorldPosition,
    desiredCameraPosition: options.desiredCameraPosition,
    desiredLookTarget: options.desiredLookTarget,
  });

  let nextFreeCameraTransition = options.freeCameraTransition;
  if (options.cameraViewMode === "free" && nextFreeCameraTransition) {
    const completed = updateFreeCameraTransition({
      sceneState: options.sceneState,
      ...nextFreeCameraTransition,
    });
    if (completed) {
      nextFreeCameraTransition = null;
    }
  }
  options.sceneState.controls.update();
  options.sceneState.updateWallVisibility();

  const renderInfo: FrameRenderInfo = {
    frameIndex: frameWindow.frameIndex,
    nextFrameIndex: frameWindow.nextFrameIndex,
    alpha: frameWindow.alpha,
    currentTime: options.currentTime,
  };
  for (const callback of options.beforeRenderCallbacks) {
    callback(renderInfo);
  }

  const renderContext = createRenderContext({
    replay: options.replay,
    player: options.player,
    scene: options.sceneState,
    container: options.container,
    playerOptions: options.playerOptions,
    state: options.getState(),
    renderInfo,
    ballFrame,
    nextBallFrame,
    ballPosition,
    players: renderPlayers,
  });
  for (const plugin of options.plugins) {
    plugin.beforeRender?.(renderContext);
  }
  options.sceneState.renderer.render(options.sceneState.scene, options.sceneState.camera);
  return nextFreeCameraTransition;
}

function createRenderContext(options: {
  replay: ReplayModel;
  player: ReplayPlayer;
  scene: ReplayScene;
  container: HTMLElement;
  playerOptions: ReplayPlayerOptions;
  state: ReplayPlayerState;
  renderInfo: FrameRenderInfo;
  ballFrame: ReplayModel["ballFrames"][number] | null;
  nextBallFrame: ReplayModel["ballFrames"][number] | null;
  ballPosition: Vec3 | null;
  players: ReplayPlayerRenderTrackContext[];
}): ReplayPlayerRenderContext {
  const { replay, state, renderInfo } = options;
  return {
    player: options.player,
    replay,
    scene: options.scene,
    container: options.container,
    options: options.playerOptions,
    state,
    ...renderInfo,
    frame: replay.frames[renderInfo.frameIndex] ?? null,
    nextFrame: replay.frames[renderInfo.nextFrameIndex] ?? null,
    ballFrame: options.ballFrame,
    nextBallFrame: options.nextBallFrame,
    ballPosition: options.ballPosition,
    players: options.players,
  };
}
