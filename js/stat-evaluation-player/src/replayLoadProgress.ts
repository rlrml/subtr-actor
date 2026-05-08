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
    total: 4,
    label: "Parse replay",
    start: 0,
    end: 0.1,
  },
  {
    stage: "processing",
    index: 2,
    total: 4,
    label: "Process replay frames",
    start: 0.1,
    end: 0.75,
  },
  {
    stage: "stats-timeline",
    index: 3,
    total: 4,
    label: "Build stats timeline",
    start: 0.75,
    end: 0.9,
  },
  {
    stage: "normalizing",
    index: 4,
    total: 4,
    label: "Normalize replay data",
    start: 0.9,
    end: 0.99,
  },
];

function clampUnitInterval(value: number): number {
  return Math.max(0, Math.min(1, value));
}

function getReplayLoadPhaseConfig(progress: ReplayLoadProgress) {
  return REPLAY_LOAD_PHASES.find((phase) => phase.stage === progress.stage)!;
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

export function getReplayLoadPhaseStates(
  progress: ReplayLoadProgress,
): ReplayLoadPhaseState[] {
  const currentPhase = getReplayLoadPhaseConfig(progress);

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

    const isDeterminate = progress.progress !== undefined;
    return {
      stage,
      index,
      total,
      label,
      state: "active",
      completion: isDeterminate ? clampUnitInterval(progress.progress ?? 0) : 1,
      indeterminate: !isDeterminate,
    };
  });
}

export function formatReplayLoadProgress(progress: ReplayLoadProgress): string {
  const percent = progress.progress === undefined
    ? null
    : Math.round(progress.progress * 100);

  switch (progress.stage) {
    case "validating":
      return "Parsing replay...";
    case "processing":
      if (percent !== null && progress.totalFrames !== undefined) {
        return `Processing replay frames... ${percent}% (${progress.processedFrames ?? 0}/${progress.totalFrames})`;
      }
      return "Processing replay frames...";
    case "stats-timeline":
      if (percent !== null) {
        return `Building stats timeline... ${percent}%`;
      }
      return "Building stats timeline...";
    case "normalizing":
      if (percent !== null) {
        return `Normalizing replay data... ${percent}%`;
      }
      return "Normalizing replay data...";
    default:
      return "Loading replay...";
  }
}

export function getReplayLoadCompletion(progress: ReplayLoadProgress): number {
  if (progress.stage !== "validating" && progress.progress !== undefined) {
    const phase = getReplayLoadPhaseConfig(progress);
    return phase.start + (
      clampUnitInterval(progress.progress) * (phase.end - phase.start)
    );
  }

  const phase = getReplayLoadPhaseConfig(progress);
  return phase.start + ((phase.end - phase.start) * 0.5);
}
