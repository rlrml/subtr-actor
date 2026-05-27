import * as THREE from "three";
import type { ReplayPlayerTimelineProjection, ReplayPlayerTimelineSegment } from "../types";

export function projectReplayTimeToTimeline(
  replayDuration: number,
  segments: ReplayPlayerTimelineSegment[],
  replayTime: number,
): ReplayPlayerTimelineProjection {
  const clampedReplayTime = THREE.MathUtils.clamp(replayTime, 0, replayDuration);
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
  const clampedTimelineTime = THREE.MathUtils.clamp(timelineTime, 0, timelineDuration);
  let skippedDuration = 0;

  for (const segment of segments) {
    const visibleEnd = segment.startTime - skippedDuration;
    if (clampedTimelineTime <= visibleEnd) {
      return clampedTimelineTime + skippedDuration;
    }

    skippedDuration += segment.endTime - segment.startTime;
  }

  return THREE.MathUtils.clamp(clampedTimelineTime + skippedDuration, 0, replayDuration);
}

export function getReplayPlaybackEndTime(
  replayDuration: number,
  segments: ReplayPlayerTimelineSegment[],
): number {
  const finalSegment = segments.at(-1);
  if (!finalSegment || finalSegment.endTime < replayDuration) {
    return replayDuration;
  }

  return THREE.MathUtils.clamp(finalSegment.startTime, 0, replayDuration);
}
