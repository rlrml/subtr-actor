import type { ReplayLoadProgress } from "./types";

export function formatReplayLoadProgress(progress: ReplayLoadProgress): string {
  const percent = progress.progress === undefined ? null : Math.round(progress.progress * 100);

  if (progress.stage === "processing") {
    if (percent === null || progress.totalFrames === undefined) {
      return "Processing replay frames...";
    }
    return `Processing replay frames... ${percent}% (${progress.processedFrames ?? 0}/${progress.totalFrames})`;
  }

  if (progress.stage === "validating") {
    return "Validating replay...";
  }

  if (progress.stage === "normalizing") {
    if (percent !== null) {
      return `Normalizing replay data... ${percent}%`;
    }
    return "Normalizing replay data...";
  }

  return "Loading replay...";
}

export function formatReplayLoadProgressMeta(progress: ReplayLoadProgress): string {
  const percent = progress.progress ?? 0;

  if (progress.stage === "processing") {
    if (progress.totalFrames !== undefined) {
      return progress.processedFrames === undefined
        ? `${progress.totalFrames} frames`
        : `${progress.processedFrames}/${progress.totalFrames} frames`;
    }
    return "Extracting frame data";
  }

  if (progress.stage === "validating") {
    return "Checking replay file";
  }

  if (progress.stage === "normalizing") {
    if (percent < 0.45) {
      return "Decoding structured replay data";
    }
    if (percent < 0.65) {
      return "Parsing frame data";
    }
    if (percent < 1) {
      return "Building playback model";
    }
    return "Playback model ready";
  }

  return progress.stage;
}
