import { findFrameIndexAtTime } from "./replay-data";
import {
  isKickoffFrame,
  isLiveGameplayFrame,
  isPostGoalTransitionFrame,
} from "./player-internals/timeline";
import type { ReplayModel } from "./types";

export function findKickoffSkipTime(
  replay: ReplayModel,
  currentTime: number,
  kickoffGameState: number | null,
  liveGameState: number | null,
): number | null {
  const frameIndex = findFrameIndexAtTime(replay, currentTime);
  const frame = replay.frames[frameIndex];
  if (!frame || !isKickoffFrame(frame, kickoffGameState)) {
    return null;
  }

  const nextLiveFrame = replay.frames.find(
    (candidate, index) => index > frameIndex && isLiveGameplayFrame(candidate, liveGameState),
  );
  if (!nextLiveFrame || nextLiveFrame.time === currentTime) {
    return null;
  }

  return nextLiveFrame.time;
}

export function findPostGoalTransitionSkipTime(
  replay: ReplayModel,
  currentTime: number,
  liveGameState: number | null,
  kickoffGameState: number | null,
): number | null {
  const frameIndex = findFrameIndexAtTime(replay, currentTime);
  const frame = replay.frames[frameIndex];
  if (
    !frame ||
    !isPostGoalTransitionFrame(replay, frame, frameIndex, liveGameState, kickoffGameState)
  ) {
    return null;
  }

  const nextFrame = replay.frames.find(
    (candidate, index) =>
      index > frameIndex &&
      !isPostGoalTransitionFrame(replay, candidate, index, liveGameState, kickoffGameState),
  );
  if (nextFrame) {
    return nextFrame.time === currentTime ? null : nextFrame.time;
  }

  let startIndex = frameIndex;
  while (
    startIndex > 0 &&
    isPostGoalTransitionFrame(
      replay,
      replay.frames[startIndex - 1],
      startIndex - 1,
      liveGameState,
      kickoffGameState,
    )
  ) {
    startIndex -= 1;
  }

  const transitionStartTime = replay.frames[startIndex]?.time;
  if (transitionStartTime === undefined || transitionStartTime === currentTime) {
    return null;
  }
  return transitionStartTime;
}
