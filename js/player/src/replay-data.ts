import type { RawReplayFramesData, ReplayModel } from "./types";
import { buildBoostPads, buildBoostPadsAsync } from "./replay-boost-pads";
import {
  buildBallFrames,
  buildBallFramesAsync,
  buildPlaybackFrames,
  buildPlaybackFramesAsync,
  buildPlayerTracks,
  buildPlayerTracksAsync,
} from "./replay-data-frames";
import {
  createAsyncNormalizationProgressTracker,
  createNormalizationProgressTracker,
  type NormalizeReplayProgress,
} from "./replay-normalization-progress";
import { buildTimelineEvents, buildTimelineEventsAsync } from "./replay-timeline-events";

export type { NormalizeReplayProgress } from "./replay-normalization-progress";

export interface NormalizeReplayDataOptions {
  onProgress?: (progress: number, details: NormalizeReplayProgress) => void;
  progressReportMinDelta?: number;
  progressReportFrameInterval?: number;
}

export interface NormalizeReplayDataAsyncOptions extends NormalizeReplayDataOptions {
  yieldEveryMs?: number;
  yieldToMainThread?: () => Promise<void>;
}

export function normalizeReplayData(
  raw: RawReplayFramesData,
  options: NormalizeReplayDataOptions = {},
): ReplayModel {
  const progressTracker = createNormalizationProgressTracker(raw, options.onProgress, {
    progressReportMinDelta: options.progressReportMinDelta,
    progressReportFrameInterval: options.progressReportFrameInterval,
  });
  const startTime = raw.frame_data.metadata_frames[0]?.time ?? 0;
  const frames = buildPlaybackFrames(raw, progressTracker);
  const players = buildPlayerTracks(raw, progressTracker);
  const ballFrames = buildBallFrames(raw, progressTracker);
  const boostPads = buildBoostPads(raw, players, startTime, progressTracker);
  const timelineEvents = buildTimelineEvents(raw, players, startTime, progressTracker);
  progressTracker.finish();

  return {
    frameCount: frames.length,
    duration: frames.at(-1)?.time ?? 0,
    frames,
    ballFrames,
    boostPads,
    players,
    timelineEvents,
    teamZeroNames: raw.meta.team_zero.map((player) => player.name),
    teamOneNames: raw.meta.team_one.map((player) => player.name),
  };
}

export async function normalizeReplayDataAsync(
  raw: RawReplayFramesData,
  options: NormalizeReplayDataAsyncOptions = {},
): Promise<ReplayModel> {
  const progressTracker = createAsyncNormalizationProgressTracker(raw, options);
  const startTime = raw.frame_data.metadata_frames[0]?.time ?? 0;
  const frames = await buildPlaybackFramesAsync(raw, progressTracker);
  const players = await buildPlayerTracksAsync(raw, progressTracker);
  const ballFrames = await buildBallFramesAsync(raw, progressTracker);
  const boostPads = await buildBoostPadsAsync(raw, players, startTime, progressTracker);
  const timelineEvents = await buildTimelineEventsAsync(raw, players, startTime, progressTracker);
  progressTracker.finish();

  return {
    frameCount: frames.length,
    duration: frames.at(-1)?.time ?? 0,
    frames,
    ballFrames,
    boostPads,
    players,
    timelineEvents,
    teamZeroNames: raw.meta.team_zero.map((player) => player.name),
    teamOneNames: raw.meta.team_one.map((player) => player.name),
  };
}

export function findFrameIndexAtTime(replay: ReplayModel, time: number): number {
  if (replay.frames.length === 0) {
    return 0;
  }

  let low = 0;
  let high = replay.frames.length - 1;

  while (low <= high) {
    const middle = Math.floor((low + high) / 2);
    const middleTime = replay.frames[middle]?.time ?? 0;

    if (middleTime < time) {
      low = middle + 1;
    } else if (middleTime > time) {
      high = middle - 1;
    } else {
      return middle;
    }
  }

  return Math.max(0, low - 1);
}
