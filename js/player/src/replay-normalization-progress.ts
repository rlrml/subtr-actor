import type { RawReplayFramesData } from "./types";
import { STANDARD_SOCCAR_BOOST_PAD_COUNT } from "./replay-boost-pads";

const NORMALIZATION_PROGRESS_REPORT_MIN_DELTA = 0.005;
const NORMALIZATION_PROGRESS_REPORT_FRAME_INTERVAL = Number.POSITIVE_INFINITY;
const NORMALIZATION_ASYNC_YIELD_INTERVAL_MS = 16;

export interface NormalizeReplayProgress {
  progress: number;
  processedFrames: number;
  totalFrames: number;
  processedUnits: number;
  totalUnits: number;
}

export interface NormalizeReplayProgressTracker {
  advance(units?: number): boolean;
  advanceFrame(units?: number): boolean;
  finish(): void;
}

export interface AsyncNormalizeReplayProgressTracker extends NormalizeReplayProgressTracker {
  yieldToMainThread(): Promise<void>;
}

interface NormalizationProgressTrackerOptions {
  progressReportMinDelta?: number;
  progressReportFrameInterval?: number;
  yieldEveryMs?: number;
}

interface AsyncNormalizationProgressTrackerOptions extends NormalizationProgressTrackerOptions {
  onProgress?: (progress: number, details: NormalizeReplayProgress) => void;
  yieldToMainThread?: () => Promise<void>;
}

function currentTimeMs(): number {
  return typeof performance === "undefined" ? Date.now() : performance.now();
}

function defaultYieldToMainThread(): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, 0));
}

function getNormalizationTotalUnits(raw: RawReplayFramesData): number {
  const playerInfoCount = raw.meta.team_zero.length + raw.meta.team_one.length;
  const playerFrameCount = raw.frame_data.players.reduce(
    (count, [, playerData]) => count + playerData.frames.length,
    0,
  );
  const boostPadCount = raw.boost_pads?.length ?? STANDARD_SOCCAR_BOOST_PAD_COUNT;
  const boostPadEventCount = raw.boost_pad_events?.length ?? 0;
  const timelineEventCount =
    (raw.goal_events?.length ?? 0) +
    (raw.player_stat_events?.length ?? 0) +
    (raw.demolish_infos?.length ?? 0);

  return [
    Math.max(1, raw.frame_data.metadata_frames.length),
    Math.max(1, playerInfoCount),
    Math.max(1, playerFrameCount),
    Math.max(1, raw.frame_data.ball_data.frames.length),
    Math.max(1, boostPadCount + boostPadEventCount),
    Math.max(1, timelineEventCount),
  ].reduce((sum, count) => sum + count, 0);
}

function getNormalizationTotalFrameUnits(raw: RawReplayFramesData): number {
  const playerFrameCount = raw.frame_data.players.reduce(
    (count, [, playerData]) => count + playerData.frames.length,
    0,
  );

  return [
    Math.max(1, raw.frame_data.metadata_frames.length),
    Math.max(1, playerFrameCount),
    Math.max(1, raw.frame_data.ball_data.frames.length),
  ].reduce((sum, count) => sum + count, 0);
}

export function createNormalizationProgressTracker(
  raw: RawReplayFramesData,
  onProgress?: (progress: number, details: NormalizeReplayProgress) => void,
  options: NormalizationProgressTrackerOptions = {},
): NormalizeReplayProgressTracker {
  const totalUnits = getNormalizationTotalUnits(raw);
  const totalFrameUnits = getNormalizationTotalFrameUnits(raw);
  let completedUnits = 0;
  let completedFrameUnits = 0;
  let lastReportedProgress = -1;
  let lastReportedFrameUnits = -1;
  let lastYieldedAt = currentTimeMs();
  const yieldEveryMs = options.yieldEveryMs ?? Number.POSITIVE_INFINITY;
  const progressReportMinDelta =
    options.progressReportMinDelta ?? NORMALIZATION_PROGRESS_REPORT_MIN_DELTA;
  const progressReportFrameInterval = Math.max(
    1,
    options.progressReportFrameInterval ?? NORMALIZATION_PROGRESS_REPORT_FRAME_INTERVAL,
  );

  const maybeReport = () => {
    if (!onProgress) {
      return false;
    }

    const progress = Math.max(0, Math.min(1, completedUnits / totalUnits));
    if (progress <= lastReportedProgress) {
      return false;
    }

    const frameDelta = completedFrameUnits - lastReportedFrameUnits;
    const reachedFrameInterval = frameDelta >= progressReportFrameInterval;
    if (
      progress >= 1 ||
      progress - lastReportedProgress >= progressReportMinDelta ||
      reachedFrameInterval
    ) {
      lastReportedProgress = progress;
      lastReportedFrameUnits = completedFrameUnits;
      onProgress(progress, {
        progress,
        processedFrames: Math.min(completedFrameUnits, totalFrameUnits),
        totalFrames: totalFrameUnits,
        processedUnits: completedUnits,
        totalUnits,
      });
      return true;
    }

    return false;
  };

  const shouldYield = (force = false) => {
    const now = currentTimeMs();
    if (!force && now - lastYieldedAt < yieldEveryMs) {
      return false;
    }
    lastYieldedAt = now;
    return true;
  };

  maybeReport();

  return {
    advance(units = 1) {
      if (units <= 0) {
        return false;
      }
      completedUnits = Math.min(totalUnits, completedUnits + units);
      const reported = maybeReport();
      return shouldYield(reported);
    },
    advanceFrame(units = 1) {
      if (units <= 0) {
        return false;
      }
      completedFrameUnits = Math.min(totalFrameUnits, completedFrameUnits + units);
      completedUnits = Math.min(totalUnits, completedUnits + units);
      const reported = maybeReport();
      return shouldYield(reported);
    },
    finish() {
      completedUnits = totalUnits;
      completedFrameUnits = totalFrameUnits;
      maybeReport();
    },
  };
}

export function createAsyncNormalizationProgressTracker(
  raw: RawReplayFramesData,
  options: AsyncNormalizationProgressTrackerOptions,
): AsyncNormalizeReplayProgressTracker {
  const progressTracker = createNormalizationProgressTracker(raw, options.onProgress, {
    progressReportMinDelta: options.progressReportMinDelta,
    progressReportFrameInterval: options.progressReportFrameInterval,
    yieldEveryMs: options.yieldEveryMs ?? NORMALIZATION_ASYNC_YIELD_INTERVAL_MS,
  });

  return {
    ...progressTracker,
    yieldToMainThread: options.yieldToMainThread ?? defaultYieldToMainThread,
  };
}
