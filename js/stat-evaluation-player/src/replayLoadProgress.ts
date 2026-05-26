export type ReplayLoadStage =
  | "validating"
  | "processing"
  | "building-stats"
  | "serializing-replay"
  | "serializing-stats"
  | "decoding-replay"
  | "decoding-stats"
  | "deriving-stats"
  | "normalizing"
  | "stats-timeline";

export interface ReplayLoadProgress {
  stage: ReplayLoadStage;
  processedFrames?: number;
  totalFrames?: number;
  processedChunks?: number;
  totalChunks?: number;
  progress?: number;
}

export interface ReplayLoadPhase {
  stage: ReplayLoadStage;
  index: number;
  total: number;
  label: string;
}

export interface ReplayLoadPhaseState extends ReplayLoadPhase {
  state: "pending" | "active" | "complete";
  completion: number;
  indeterminate: boolean;
}

const REPLAY_LOAD_PHASES: Array<ReplayLoadPhase & { start: number; end: number }> = [
  {
    stage: "validating",
    index: 1,
    total: 9,
    label: "Parse replay",
    start: 0,
    end: 0.08,
  },
  {
    stage: "processing",
    index: 2,
    total: 9,
    label: "Process replay frames",
    start: 0.08,
    end: 0.62,
  },
  {
    stage: "building-stats",
    index: 3,
    total: 9,
    label: "Build stats events",
    start: 0.62,
    end: 0.7,
  },
  {
    stage: "serializing-replay",
    index: 4,
    total: 9,
    label: "Serialize replay data",
    start: 0.7,
    end: 0.76,
  },
  {
    stage: "serializing-stats",
    index: 5,
    total: 9,
    label: "Serialize stats timeline",
    start: 0.76,
    end: 0.86,
  },
  {
    stage: "normalizing",
    index: 6,
    total: 9,
    label: "Normalize replay model",
    start: 0.86,
    end: 0.91,
  },
  {
    stage: "decoding-replay",
    index: 7,
    total: 9,
    label: "Decode replay data",
    start: 0.91,
    end: 0.94,
  },
  {
    stage: "decoding-stats",
    index: 8,
    total: 9,
    label: "Decode stats chunks",
    start: 0.94,
    end: 0.96,
  },
  {
    stage: "deriving-stats",
    index: 9,
    total: 9,
    label: "Derive stats snapshots",
    start: 0.96,
    end: 1,
  },
];

function clampUnitInterval(value: number): number {
  return Math.max(0, Math.min(1, value));
}

function scaleProgress(value: number | undefined, start: number, end: number): number | undefined {
  if (value === undefined) {
    return undefined;
  }
  return clampUnitInterval((value - start) / (end - start));
}

function normalizeReplayLoadProgress(progress: ReplayLoadProgress): ReplayLoadProgress {
  if (progress.stage !== "stats-timeline") {
    return progress;
  }

  const value = progress.progress;
  if (value === undefined) {
    return {
      ...progress,
      stage: "building-stats",
    };
  }

  if (value < 0.35) {
    return {
      ...progress,
      stage: "building-stats",
      progress: scaleProgress(value, 0, 0.35),
    };
  }

  if (value < 0.55) {
    return {
      ...progress,
      stage: "serializing-replay",
      progress: scaleProgress(value, 0.35, 0.55),
    };
  }

  return {
    ...progress,
    stage: "serializing-stats",
    progress: scaleProgress(value, 0.55, 0.92),
  };
}

function getReplayLoadPhaseConfig(progress: ReplayLoadProgress) {
  const normalizedProgress = normalizeReplayLoadProgress(progress);
  return REPLAY_LOAD_PHASES.find((phase) => phase.stage === normalizedProgress.stage)!;
}

export function listReplayLoadPhases(): ReplayLoadPhase[] {
  return REPLAY_LOAD_PHASES.map(({ stage, index, total, label }) => ({
    stage,
    index,
    total,
    label,
  }));
}

export function getReplayLoadPhase(progress: ReplayLoadProgress): ReplayLoadPhase {
  const phase = getReplayLoadPhaseConfig(progress);
  return {
    stage: phase.stage,
    index: phase.index,
    total: phase.total,
    label: phase.label,
  };
}

export function getReplayLoadPhaseStates(progress: ReplayLoadProgress): ReplayLoadPhaseState[] {
  const normalizedProgress = normalizeReplayLoadProgress(progress);
  const currentPhase = getReplayLoadPhaseConfig(normalizedProgress);

  return REPLAY_LOAD_PHASES.map(({ stage, index, total, label }) => {
    if (index < currentPhase.index) {
      return {
        stage,
        index,
        total,
        label,
        state: "complete",
        completion: 1,
        indeterminate: false,
      };
    }

    if (index > currentPhase.index) {
      return {
        stage,
        index,
        total,
        label,
        state: "pending",
        completion: 0,
        indeterminate: false,
      };
    }

    const isDeterminate = normalizedProgress.progress !== undefined;
    return {
      stage,
      index,
      total,
      label,
      state: "active",
      completion: isDeterminate ? clampUnitInterval(normalizedProgress.progress ?? 0) : 1,
      indeterminate: !isDeterminate,
    };
  });
}

export function formatReplayLoadProgress(progress: ReplayLoadProgress): string {
  const normalizedProgress = normalizeReplayLoadProgress(progress);
  const percent =
    normalizedProgress.progress === undefined
      ? null
      : Math.round(normalizedProgress.progress * 100);

  switch (normalizedProgress.stage) {
    case "validating":
      return "Parsing replay...";
    case "processing":
      if (percent !== null && normalizedProgress.totalFrames !== undefined) {
        return `Processing replay frames... ${percent}% (${normalizedProgress.processedFrames ?? 0}/${normalizedProgress.totalFrames})`;
      }
      return "Processing replay frames...";
    case "building-stats":
      if (percent !== null) {
        if (normalizedProgress.totalFrames !== undefined) {
          return `Building stats events... ${percent}% (${normalizedProgress.processedFrames ?? 0}/${normalizedProgress.totalFrames})`;
        }
        return `Building stats events... ${percent}%`;
      }
      return "Building stats events...";
    case "serializing-replay":
      if (percent !== null) {
        return `Serializing replay data... ${percent}%`;
      }
      return "Serializing replay data...";
    case "serializing-stats":
      if (percent !== null) {
        return `Serializing stats timeline... ${percent}%`;
      }
      return "Serializing stats timeline...";
    case "decoding-replay":
      if (percent !== null) {
        return `Decoding replay data... ${percent}%`;
      }
      return "Decoding replay data...";
    case "decoding-stats":
      if (percent !== null) {
        if (normalizedProgress.totalChunks !== undefined) {
          return `Decoding stats chunks... ${percent}% (${normalizedProgress.processedChunks ?? 0}/${normalizedProgress.totalChunks})`;
        }
        return `Decoding stats chunks... ${percent}%`;
      }
      return "Decoding stats chunks...";
    case "deriving-stats":
      if (percent !== null) {
        return `Deriving stats snapshots... ${percent}%`;
      }
      return "Deriving stats snapshots...";
    case "normalizing":
      if (percent !== null) {
        return `Normalizing replay model... ${percent}%`;
      }
      return "Normalizing replay model...";
    default:
      return "Loading replay...";
  }
}

export function getReplayLoadCompletion(progress: ReplayLoadProgress): number {
  const normalizedProgress = normalizeReplayLoadProgress(progress);
  if (normalizedProgress.stage !== "validating" && normalizedProgress.progress !== undefined) {
    const phase = getReplayLoadPhaseConfig(normalizedProgress);
    return phase.start + clampUnitInterval(normalizedProgress.progress) * (phase.end - phase.start);
  }

  const phase = getReplayLoadPhaseConfig(normalizedProgress);
  return phase.start + (phase.end - phase.start) * 0.5;
}
