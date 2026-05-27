import * as THREE from "three";
import { updateBoostMeter, type BoostMeter, type ReplayScene } from "./scene";
import {
  getActiveDemoEvent,
  updateBoostTrail,
  updateDemoIndicator,
} from "./player-render-effects";
import {
  interpolatePosition,
  interpolateQuaternion,
  rootPosition,
  worldPosition,
} from "./player-internals/spatial";
import type {
  ReplayModel,
  ReplayPlayerRenderTrackContext,
  Vec3,
} from "./types";

interface ReplayFrameWindow {
  frameIndex: number;
  nextFrameIndex: number;
  alpha: number;
}

export function renderReplayFrameScene(options: {
  replay: ReplayModel;
  sceneState: ReplayScene;
  frameWindow: ReplayFrameWindow;
  fieldScale: number;
  currentTime: number;
  boostMeterEnabled: boolean;
}): {
  ballFrame: ReplayModel["ballFrames"][number] | null;
  nextBallFrame: ReplayModel["ballFrames"][number] | null;
  ballPosition: Vec3 | null;
  ballWorldPosition: THREE.Vector3 | null;
  players: ReplayPlayerRenderTrackContext[];
} {
  const { replay, sceneState, frameWindow, fieldScale, currentTime, boostMeterEnabled } = options;
  const ballFrame = replay.ballFrames[frameWindow.frameIndex] ?? null;
  const nextBallFrame = replay.ballFrames[frameWindow.nextFrameIndex] ?? ballFrame;
  const interpolatedBallPosition = interpolatePosition(
    ballFrame?.position ?? null,
    nextBallFrame?.position ?? null,
    frameWindow.alpha,
  );

  renderBall(sceneState, interpolatedBallPosition, ballFrame, nextBallFrame, frameWindow.alpha);

  return {
    ballFrame,
    nextBallFrame,
    ballPosition: interpolatedBallPosition,
    ballWorldPosition: interpolatedBallPosition
      ? worldPosition(interpolatedBallPosition, fieldScale)
      : null,
    players: renderPlayers({
      replay,
      sceneState,
      frameWindow,
      currentTime,
      boostMeterEnabled,
    }),
  };
}

function renderBall(
  sceneState: ReplayScene,
  interpolatedBallPosition: Vec3 | null,
  ballFrame: ReplayModel["ballFrames"][number] | null,
  nextBallFrame: ReplayModel["ballFrames"][number] | null,
  alpha: number,
): void {
  if (!interpolatedBallPosition) {
    sceneState.ballMesh.visible = false;
    return;
  }

  sceneState.ballMesh.visible = true;
  sceneState.ballMesh.position.copy(rootPosition(interpolatedBallPosition));
  const ballRotation = interpolateQuaternion(
    ballFrame?.rotation ?? null,
    nextBallFrame?.rotation ?? null,
    alpha,
  );
  if (ballRotation) {
    sceneState.ballMesh.quaternion.copy(ballRotation);
  } else {
    sceneState.ballMesh.quaternion.identity();
  }
}

function renderPlayers(options: {
  replay: ReplayModel;
  sceneState: ReplayScene;
  frameWindow: ReplayFrameWindow;
  currentTime: number;
  boostMeterEnabled: boolean;
}): ReplayPlayerRenderTrackContext[] {
  const { replay, sceneState, frameWindow, currentTime, boostMeterEnabled } = options;
  const renderPlayers: ReplayPlayerRenderTrackContext[] = [];

  for (const [playerIndex, player] of replay.players.entries()) {
    const mesh = sceneState.playerMeshes.get(player.id);
    const boostTrail = sceneState.playerBoostTrails.get(player.id);
    const boostMeter = sceneState.playerBoostMeters.get(player.id);
    const demoIndicator = sceneState.playerDemoIndicators.get(player.id);
    const frame = player.frames[frameWindow.frameIndex] ?? null;
    const nextFrame = player.frames[frameWindow.nextFrameIndex] ?? frame;
    let boostFraction = 0;

    if (!mesh) {
      if (demoIndicator) {
        demoIndicator.group.visible = false;
      }
      renderPlayers.push({
        track: player,
        mesh: null,
        boostTrail: boostTrail ?? null,
        frame,
        nextFrame,
        interpolatedPosition: null,
        boostFraction,
      });
      continue;
    }

    const interpolatedPosition = interpolatePosition(
      frame?.position ?? null,
      nextFrame?.position ?? null,
      frameWindow.alpha,
    );
    const activeDemoEvent = getActiveDemoEvent(replay, player.id, currentTime);
    if (!interpolatedPosition || activeDemoEvent || !isPlayerSamplePresent(frame)) {
      hidePlayerEffects({
        mesh,
        boostTrail: boostTrail ?? null,
        boostMeter: boostMeter ?? null,
      });
      updateDemoIndicator({
        indicator: demoIndicator ?? null,
        fallbackPosition: interpolatedPosition,
        demoEvent: activeDemoEvent,
        currentTime,
        camera: sceneState.camera,
      });
      renderPlayers.push({
        track: player,
        mesh,
        boostTrail: boostTrail ?? null,
        frame,
        nextFrame,
        interpolatedPosition: null,
        boostFraction,
      });
      continue;
    }

    mesh.visible = true;
    if (demoIndicator) {
      demoIndicator.group.visible = false;
    }
    mesh.position.copy(rootPosition(interpolatedPosition));
    const rotation = interpolateQuaternion(
      frame?.rotation ?? null,
      nextFrame?.rotation ?? null,
      frameWindow.alpha,
    );
    if (rotation) {
      mesh.quaternion.copy(rotation);
    } else {
      mesh.quaternion.identity();
    }

    const currentBoostFraction = frame?.boostFraction ?? 0;
    const nextBoostFraction = nextFrame?.boostFraction ?? currentBoostFraction;
    boostFraction = THREE.MathUtils.lerp(
      currentBoostFraction,
      nextBoostFraction,
      frameWindow.alpha,
    );
    renderBoostEffects({
      boostTrail: boostTrail ?? null,
      boostMeter: boostMeter ?? null,
      boostMeterEnabled,
      boostFraction,
      boostAmount: THREE.MathUtils.lerp(
        frame?.boostAmount ?? 0,
        nextFrame?.boostAmount ?? frame?.boostAmount ?? 0,
        frameWindow.alpha,
      ),
      boostActive:
        (frameWindow.alpha >= 0.5 ? nextFrame?.boostActive : frame?.boostActive) ??
        frame?.boostActive ??
        nextFrame?.boostActive ??
        false,
      currentTime,
      playerIndex,
      camera: sceneState.camera,
    });

    renderPlayers.push({
      track: player,
      mesh,
      boostTrail: boostTrail ?? null,
      frame,
      nextFrame,
      interpolatedPosition,
      boostFraction,
    });
  }

  return renderPlayers;
}

function renderBoostEffects(options: {
  boostTrail: THREE.Group | null;
  boostMeter: BoostMeter | null;
  boostMeterEnabled: boolean;
  boostFraction: number;
  boostAmount: number;
  boostActive: boolean;
  currentTime: number;
  playerIndex: number;
  camera: THREE.Camera;
}): void {
  const {
    boostTrail,
    boostMeter,
    boostMeterEnabled,
    boostFraction,
    boostAmount,
    boostActive,
    currentTime,
    playerIndex,
    camera,
  } = options;

  if (boostTrail) {
    updateBoostTrail(boostTrail, boostActive, boostFraction, currentTime, playerIndex);
  }

  if (!boostMeter) {
    return;
  }

  if (boostMeterEnabled) {
    boostMeter.group.visible = true;
    updateBoostMeter(boostMeter, boostFraction, boostAmount, camera);
  } else {
    boostMeter.group.visible = false;
  }
}

function hidePlayerEffects(options: {
  mesh: THREE.Object3D;
  boostTrail: THREE.Group | null;
  boostMeter: BoostMeter | null;
}): void {
  const { mesh, boostTrail, boostMeter } = options;
  mesh.visible = false;
  if (boostTrail) {
    boostTrail.visible = false;
  }
  if (boostMeter) {
    boostMeter.group.visible = false;
  }
}

function isPlayerSamplePresent(
  sample: ReplayModel["players"][number]["frames"][number] | null | undefined,
): boolean {
  return Boolean(sample?.position) && sample?.isPresent !== false;
}
