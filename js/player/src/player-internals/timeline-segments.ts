import type { ReplayModel, ReplayPlayerTimelineSegment } from "../types";
import { isKickoffFrame, isPostGoalTransitionFrame } from "./timeline-game-state";

function isSkippedTimelineFrame(
  replay: ReplayModel,
  frame: ReplayModel["frames"][number],
  frameIndex: number,
  skipPostGoalTransitionsEnabled: boolean,
  skipKickoffsEnabled: boolean,
  liveGameState: number | null,
  kickoffGameState: number | null,
): boolean {
  return (
    (skipPostGoalTransitionsEnabled &&
      isPostGoalTransitionFrame(replay, frame, frameIndex, liveGameState, kickoffGameState)) ||
    (skipKickoffsEnabled && isKickoffFrame(frame, kickoffGameState))
  );
}

export function computeTimelineSegments(
  replay: ReplayModel,
  skipPostGoalTransitionsEnabled: boolean,
  skipKickoffsEnabled: boolean,
  liveGameState: number | null,
  kickoffGameState: number | null,
): ReplayPlayerTimelineSegment[] {
  const segments: ReplayPlayerTimelineSegment[] = [];
  const { frames } = replay;

  if (frames.length === 0 || (!skipPostGoalTransitionsEnabled && !skipKickoffsEnabled)) {
    return segments;
  }

  let index = 0;
  while (index < frames.length) {
    const frame = frames[index];
    if (
      !frame ||
      !isSkippedTimelineFrame(
        replay,
        frame,
        index,
        skipPostGoalTransitionsEnabled,
        skipKickoffsEnabled,
        liveGameState,
        kickoffGameState,
      )
    ) {
      index += 1;
      continue;
    }

    const startTime = frame.time;
    let endIndex = index + 1;
    while (
      endIndex < frames.length &&
      isSkippedTimelineFrame(
        replay,
        frames[endIndex],
        endIndex,
        skipPostGoalTransitionsEnabled,
        skipKickoffsEnabled,
        liveGameState,
        kickoffGameState,
      )
    ) {
      endIndex += 1;
    }

    const endTime = frames[endIndex]?.time ?? replay.duration;
    if (endTime > startTime) {
      const previous = segments.at(-1);
      if (previous && previous.endTime >= startTime) {
        previous.endTime = Math.max(previous.endTime, endTime);
      } else {
        segments.push({ startTime, endTime });
      }
    }

    index = endIndex;
  }

  return segments;
}
