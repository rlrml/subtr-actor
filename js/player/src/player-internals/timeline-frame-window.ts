import * as THREE from "three";
import { findFrameIndexAtTime } from "../replay-data";
import type { ReplayModel } from "../types";

export function clampFrameIndex(replay: ReplayModel, frameIndex: number): number {
  if (replay.frames.length === 0) {
    return 0;
  }

  return THREE.MathUtils.clamp(Math.round(frameIndex), 0, replay.frames.length - 1);
}

export function getFrameWindow(
  replay: ReplayModel,
  time: number,
): { frameIndex: number; nextFrameIndex: number; alpha: number } {
  const frameIndex = findFrameIndexAtTime(replay, time);
  const nextFrameIndex = Math.min(frameIndex + 1, replay.frames.length - 1);

  if (nextFrameIndex === frameIndex) {
    return { frameIndex, nextFrameIndex, alpha: 0 };
  }

  const startTime = replay.frames[frameIndex]?.time ?? 0;
  const endTime = replay.frames[nextFrameIndex]?.time ?? startTime;
  if (endTime <= startTime) {
    return { frameIndex, nextFrameIndex, alpha: 0 };
  }

  return {
    frameIndex,
    nextFrameIndex,
    alpha: THREE.MathUtils.clamp((time - startTime) / (endTime - startTime), 0, 1),
  };
}
