import * as THREE from "three";
import { findFrameIndexAtTime } from "../replay-data";
import type {
  ReplayModel,
  ReplayPlayerKickoffCountdownMetadata,
  ReplayPlayerTimelineProjection,
  ReplayPlayerTimelineSegment,
} from "../types";

export function clampFrameIndex(replay: ReplayModel, frameIndex: number): number {
  if (replay.frames.length === 0) {
    return 0;
  }

  return THREE.MathUtils.clamp(
    Math.round(frameIndex),
    0,
    replay.frames.length - 1,
  );
}

export function inferLiveGameState(replay: ReplayModel): number | null {
  if (replay.frames.length === 0) {
    return null;
  }

  const counts = new Map<number, number>();
  for (const frame of replay.frames) {
    counts.set(frame.gameState, (counts.get(frame.gameState) ?? 0) + 1);
  }

  let liveGameState: number | null = null;
  let liveGameStateCount = -1;
  for (const [gameState, count] of counts.entries()) {
    if (count <= liveGameStateCount) {
      continue;
    }

    liveGameState = gameState;
    liveGameStateCount = count;
  }

  return liveGameState;
}

export function inferKickoffGameState(
  replay: ReplayModel,
  liveGameState: number | null,
): number | null {
  if (liveGameState === null) {
    return null;
  }

  for (const frame of replay.frames) {
    if (frame.gameState === liveGameState) {
      break;
    }

    return frame.gameState;
  }

  return null;
}

export function isLiveGameplayFrame(
  frame: ReplayModel["frames"][number],
  liveGameState: number | null,
): boolean {
  if (liveGameState === null) {
    return frame.kickoffCountdown <= 0;
  }

  return frame.gameState === liveGameState;
}

export function isKickoffFrame(
  frame: ReplayModel["frames"][number],
  kickoffGameState: number | null,
): boolean {
  if (frame.kickoffCountdown > 0) {
    return true;
  }

  return kickoffGameState !== null && frame.gameState === kickoffGameState;
}

function hasRenderableSamples(replay: ReplayModel, frameIndex: number): boolean {
  if (replay.ballFrames[frameIndex]?.position) {
    return true;
  }

  return replay.players.some((player) => player.frames[frameIndex]?.position);
}

function isRenderableKickoffFrame(
  replay: ReplayModel,
  frame: ReplayModel["frames"][number],
  frameIndex: number,
  kickoffGameState: number | null,
): boolean {
  return isKickoffFrame(frame, kickoffGameState) && hasRenderableSamples(replay, frameIndex);
}

export function isPostGoalTransitionFrame(
  replay: ReplayModel,
  frame: ReplayModel["frames"][number],
  frameIndex: number,
  liveGameState: number | null,
  kickoffGameState: number | null,
): boolean {
  return !isLiveGameplayFrame(frame, liveGameState)
    && !isRenderableKickoffFrame(replay, frame, frameIndex, kickoffGameState);
}

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
      isPostGoalTransitionFrame(
        replay,
        frame,
        frameIndex,
        liveGameState,
        kickoffGameState,
      )) ||
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

  if (
    frames.length === 0 ||
    (!skipPostGoalTransitionsEnabled && !skipKickoffsEnabled)
  ) {
    return segments;
  }

  let index = 0;
  while (index < frames.length) {
    const frame = frames[index];
    if (!frame || !isSkippedTimelineFrame(
      replay,
      frame,
      index,
      skipPostGoalTransitionsEnabled,
      skipKickoffsEnabled,
      liveGameState,
      kickoffGameState,
    )) {
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

export function projectReplayTimeToTimeline(
  replayDuration: number,
  segments: ReplayPlayerTimelineSegment[],
  replayTime: number,
): ReplayPlayerTimelineProjection {
  const clampedReplayTime = THREE.MathUtils.clamp(
    replayTime,
    0,
    replayDuration,
  );
  let skippedDuration = 0;

  for (const segment of segments) {
    if (clampedReplayTime < segment.startTime) {
      break;
    }

    if (clampedReplayTime < segment.endTime) {
      return {
        replayTime: clampedReplayTime,
        timelineTime: segment.startTime - skippedDuration,
        seekTime: segment.startTime,
        hiddenBySkip: true,
      };
    }

    skippedDuration += segment.endTime - segment.startTime;
  }

  return {
    replayTime: clampedReplayTime,
    timelineTime: clampedReplayTime - skippedDuration,
    seekTime: clampedReplayTime,
    hiddenBySkip: false,
  };
}

export function projectTimelineTimeToReplay(
  replayDuration: number,
  timelineDuration: number,
  segments: ReplayPlayerTimelineSegment[],
  timelineTime: number,
): number {
  const clampedTimelineTime = THREE.MathUtils.clamp(
    timelineTime,
    0,
    timelineDuration,
  );
  let skippedDuration = 0;

  for (const segment of segments) {
    const visibleEnd = segment.startTime - skippedDuration;
    if (clampedTimelineTime <= visibleEnd) {
      return clampedTimelineTime + skippedDuration;
    }

    skippedDuration += segment.endTime - segment.startTime;
  }

  return THREE.MathUtils.clamp(
    clampedTimelineTime + skippedDuration,
    0,
    replayDuration,
  );
}

export function getKickoffCountdownMetadata(
  replay: ReplayModel,
  frameIndex: number,
  currentTime: number,
): ReplayPlayerKickoffCountdownMetadata | null {
  const currentFrame = replay.frames[frameIndex];
  if (!currentFrame || currentFrame.kickoffCountdown <= 0) {
    return null;
  }

  let startIndex = frameIndex;
  while (
    startIndex > 0 &&
    (replay.frames[startIndex - 1]?.kickoffCountdown ?? 0) > 0
  ) {
    startIndex -= 1;
  }

  let endIndex = frameIndex + 1;
  while (
    endIndex < replay.frames.length &&
    replay.frames[endIndex].kickoffCountdown > 0
  ) {
    endIndex += 1;
  }

  let maxCountdown = 0;
  for (let index = startIndex; index < endIndex; index += 1) {
    maxCountdown = Math.max(
      maxCountdown,
      replay.frames[index].kickoffCountdown,
    );
  }

  const endsAt = replay.frames[endIndex]?.time ?? replay.duration;
  const secondsRemaining = Math.max(0, endsAt - currentTime);

  return {
    kind: "kickoff-countdown",
    countdown: Math.max(1, Math.min(maxCountdown, Math.ceil(secondsRemaining))),
    secondsRemaining,
    endsAt,
  };
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
