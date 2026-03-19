export type ReplayLoadStage =
  | "validating"
  | "processing"
  | "stats-timeline"
  | "normalizing";

export interface ReplayLoadProgress {
  stage: ReplayLoadStage;
  processedFrames?: number;
  totalFrames?: number;
  progress?: number;
}

function clampUnitInterval(value: number): number {
  return Math.max(0, Math.min(1, value));
}

export function formatReplayLoadProgress(progress: ReplayLoadProgress): string {
  if (progress.stage === "processing") {
    const percent = progress.progress === undefined
      ? null
      : Math.round(progress.progress * 100);
    if (percent === null || progress.totalFrames === undefined) {
      return "Processing replay frames...";
    }
    return `Processing replay frames... ${percent}% (${progress.processedFrames ?? 0}/${progress.totalFrames})`;
  }

  switch (progress.stage) {
    case "validating":
      return "Validating replay...";
    case "stats-timeline":
      return "Building stats timeline...";
    case "normalizing":
      return "Normalizing replay data...";
    default:
      return "Loading replay...";
  }
}

export function getReplayLoadCompletion(progress: ReplayLoadProgress): number {
  switch (progress.stage) {
    case "validating":
      return 0.02;
    case "processing":
      return 0.05 + (clampUnitInterval(progress.progress ?? 0) * 0.9);
    case "stats-timeline":
      return 0.96;
    case "normalizing":
      return 0.99;
    default:
      return 0;
  }
}
