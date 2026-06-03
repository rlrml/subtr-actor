import type { Vector3 } from "three";
import type { ReplayScene } from "./scene";
import {
  interpolatePosition,
  interpolateQuaternion,
  rootPosition,
  worldPosition,
} from "./player-internals/spatial";
import type { ReplayModel } from "./types";

export interface ReplayBallRenderResult {
  ballFrame: ReplayModel["ballFrames"][number] | null;
  nextBallFrame: ReplayModel["ballFrames"][number] | null;
  ballPosition: Vector3 | null;
}

export function updateReplayBallRender({
  replay,
  sceneState,
  fieldScale,
  frameWindow,
}: {
  replay: ReplayModel;
  sceneState: ReplayScene;
  fieldScale: number;
  frameWindow: { frameIndex: number; nextFrameIndex: number; alpha: number };
}): ReplayBallRenderResult {
  const ballFrame = replay.ballFrames[frameWindow.frameIndex] ?? null;
  const nextBallFrame = replay.ballFrames[frameWindow.nextFrameIndex] ?? ballFrame;
  const interpolatedBallPosition = interpolatePosition(
    ballFrame?.position ?? null,
    nextBallFrame?.position ?? null,
    frameWindow.alpha,
  );
  const ballPosition = interpolatedBallPosition
    ? worldPosition(interpolatedBallPosition, fieldScale)
    : null;

  if (interpolatedBallPosition) {
    sceneState.ballMesh.visible = true;
    sceneState.ballMesh.position.copy(rootPosition(interpolatedBallPosition));
    const ballRotation = interpolateQuaternion(
      ballFrame?.rotation ?? null,
      nextBallFrame?.rotation ?? null,
      frameWindow.alpha,
    );
    if (ballRotation) {
      sceneState.ballMesh.quaternion.copy(ballRotation);
    } else {
      sceneState.ballMesh.quaternion.identity();
    }
  } else {
    sceneState.ballMesh.visible = false;
  }

  return { ballFrame, nextBallFrame, ballPosition };
}
