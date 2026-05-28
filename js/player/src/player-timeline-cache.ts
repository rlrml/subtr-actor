import {
  computeTimelineSegments,
  getReplayPlaybackEndTime,
  projectReplayTimeToTimeline,
  projectTimelineTimeToReplay,
} from "./player-internals/timeline";
import type {
  ReplayModel,
  ReplayPlayerTimelineProjection,
  ReplayPlayerTimelineSegment,
} from "./types";

interface ReplayPlayerTimelineOptions {
  replay: ReplayModel;
  skipPostGoalTransitionsEnabled: boolean;
  skipKickoffsEnabled: boolean;
  liveGameState: number | null;
  kickoffGameState: number | null;
}

export class ReplayPlayerTimelineCache {
  private segmentsCacheKey: string | null = null;
  private segmentsCache: ReplayPlayerTimelineSegment[] = [];
  private durationCache = 0;

  getSegments(options: ReplayPlayerTimelineOptions): ReplayPlayerTimelineSegment[] {
    const cacheKey = `${options.skipPostGoalTransitionsEnabled}:${options.skipKickoffsEnabled}`;
    if (this.segmentsCacheKey === cacheKey) {
      return this.segmentsCache;
    }

    this.segmentsCacheKey = cacheKey;
    this.segmentsCache = computeTimelineSegments(
      options.replay,
      options.skipPostGoalTransitionsEnabled,
      options.skipKickoffsEnabled,
      options.liveGameState,
      options.kickoffGameState,
    );
    this.durationCache = Math.max(
      0,
      options.replay.duration -
        this.segmentsCache.reduce(
          (total, segment) => total + (segment.endTime - segment.startTime),
          0,
        ),
    );
    return this.segmentsCache;
  }

  getDuration(options: ReplayPlayerTimelineOptions): number {
    return this.getSegments(options).length === 0 ? options.replay.duration : this.durationCache;
  }

  getPlaybackEndTime(options: ReplayPlayerTimelineOptions): number {
    return getReplayPlaybackEndTime(options.replay.duration, this.getSegments(options));
  }

  projectReplayTime(
    options: ReplayPlayerTimelineOptions,
    replayTime: number,
  ): ReplayPlayerTimelineProjection {
    return projectReplayTimeToTimeline(
      options.replay.duration,
      this.getSegments(options),
      replayTime,
    );
  }

  projectTimelineTime(options: ReplayPlayerTimelineOptions, timelineTime: number): number {
    return projectTimelineTimeToReplay(
      options.replay.duration,
      this.getDuration(options),
      this.getSegments(options),
      timelineTime,
    );
  }
}
