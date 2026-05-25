import type { ReplayModel } from "subtr-actor-player";
import { applyBackboardEventDerivedStats } from "./backboardEventDerivation.ts";
import { applyBallCarryEventDerivedStats } from "./ballCarryEventDerivation.ts";
import { applyBumpEventDerivedStats } from "./bumpEventDerivation.ts";
import { applyBoostLedgerDerivedStats } from "./boostLedgerDerivation.ts";
import { applyCeilingShotEventDerivedStats } from "./ceilingShotEventDerivation.ts";
import { applyCoreEventDerivedStats } from "./coreEventDerivation.ts";
import { applyDodgeResetEventDerivedStats } from "./dodgeResetEventDerivation.ts";
import { applyDoubleTapEventDerivedStats } from "./doubleTapEventDerivation.ts";
import { applyDemoEventDerivedStats } from "./demoEventDerivation.ts";
import { applyFiftyFiftyEventDerivedStats } from "./fiftyFiftyEventDerivation.ts";
import { applyFlickEventDerivedStats } from "./flickEventDerivation.ts";
import { applyHalfVolleyEventDerivedStats } from "./halfVolleyEventDerivation.ts";
import { applyMechanicEventDerivedStats } from "./mechanicEventDerivation.ts";
import { applyMovementEventDerivedStats } from "./movementEventDerivation.ts";
import { applyMustyFlickEventDerivedStats } from "./mustyFlickEventDerivation.ts";
import { applyOneTimerEventDerivedStats } from "./oneTimerEventDerivation.ts";
import { applyPassEventDerivedStats } from "./passEventDerivation.ts";
import { applyPossessionEventDerivedStats } from "./possessionEventDerivation.ts";
import { applyPositioningEventDerivedStats } from "./positioningEventDerivation.ts";
import { applyPowerslideEventDerivedStats } from "./powerslideEventDerivation.ts";
import { applyPressureEventDerivedStats } from "./pressureEventDerivation.ts";
import { applyRotationEventDerivedStats } from "./rotationEventDerivation.ts";
import { applyRushEventDerivedStats } from "./rushEventDerivation.ts";
import { applyTouchEventDerivedStats } from "./touchEventDerivation.ts";
import { applyWhiffEventDerivedStats } from "./whiffEventDerivation.ts";
import { applyWallAerialEventDerivedStats } from "./wallAerialEventDerivation.ts";
import { applyWallAerialShotEventDerivedStats } from "./wallAerialShotEventDerivation.ts";
import type { StatsTimeline } from "./statsTimeline";
import {
  createPlayerStatsSnapshot,
  createTeamStatsSnapshot,
  type DeepPartial,
} from "./statsSnapshotFactories.ts";
export type { ReplayLoadProgress, ReplayLoadStage } from "./replayLoadProgress.ts";
export {
  formatReplayLoadProgress,
  getReplayLoadCompletion,
  getReplayLoadPhase,
  getReplayLoadPhaseStates,
  listReplayLoadPhases,
} from "./replayLoadProgress.ts";
import type { ReplayLoadProgress } from "./replayLoadProgress.ts";

export interface ReplayLoadBundle {
  replay: ReplayModel;
  statsTimeline: StatsTimeline;
}

interface ReplayLoadRequest {
  type: "load-replay";
  bytes: ArrayBuffer;
  reportEveryNFrames: number;
}

interface ReplayProgressMessage {
  type: "progress";
  progress: ReplayLoadProgress;
}

interface ReplayDoneMessage {
  type: "done";
  replayBuffer: ArrayBuffer;
  statsTimelineParts: TransferableStatsTimelineParts;
}

interface ReplayErrorMessage {
  type: "error";
  error: string;
}

interface TransferableStatsTimelineParts {
  configBuffer: ArrayBuffer;
  replayMetaBuffer: ArrayBuffer;
  eventsBuffer: ArrayBuffer;
  frameChunkBuffers: ArrayBuffer[];
}

type ReplayWorkerMessage = ReplayProgressMessage | ReplayDoneMessage | ReplayErrorMessage;

interface StatsTimelineEventDerivedApplier {
  id: string;
  playerModules: readonly string[];
  teamModules: readonly string[];
  apply: (timeline: StatsTimeline) => StatsTimeline;
}

export const STATS_TIMELINE_EVENT_DERIVED_APPLIERS: readonly StatsTimelineEventDerivedApplier[] = [
  {
    id: "boost-ledger",
    playerModules: ["boost"],
    teamModules: ["boost"],
    apply: applyBoostLedgerDerivedStats,
  },
  {
    id: "core",
    playerModules: ["core"],
    teamModules: ["core"],
    apply: applyCoreEventDerivedStats,
  },
  {
    id: "possession",
    playerModules: [],
    teamModules: ["possession"],
    apply: applyPossessionEventDerivedStats,
  },
  {
    id: "pressure",
    playerModules: [],
    teamModules: ["pressure"],
    apply: applyPressureEventDerivedStats,
  },
  {
    id: "movement",
    playerModules: ["movement"],
    teamModules: ["movement"],
    apply: applyMovementEventDerivedStats,
  },
  {
    id: "positioning",
    playerModules: ["positioning"],
    teamModules: [],
    apply: applyPositioningEventDerivedStats,
  },
  {
    id: "rotation",
    playerModules: ["rotation"],
    teamModules: ["rotation"],
    apply: applyRotationEventDerivedStats,
  },
  {
    id: "mechanics",
    playerModules: ["speed_flip", "half_flip", "wavedash"],
    teamModules: [],
    apply: applyMechanicEventDerivedStats,
  },
  {
    id: "whiff",
    playerModules: ["whiff"],
    teamModules: [],
    apply: applyWhiffEventDerivedStats,
  },
  {
    id: "backboard",
    playerModules: ["backboard"],
    teamModules: ["backboard"],
    apply: applyBackboardEventDerivedStats,
  },
  {
    id: "double-tap",
    playerModules: ["double_tap"],
    teamModules: ["double_tap"],
    apply: applyDoubleTapEventDerivedStats,
  },
  {
    id: "demo",
    playerModules: ["demo"],
    teamModules: ["demo"],
    apply: applyDemoEventDerivedStats,
  },
  {
    id: "fifty-fifty",
    playerModules: ["fifty_fifty"],
    teamModules: ["fifty_fifty"],
    apply: applyFiftyFiftyEventDerivedStats,
  },
  {
    id: "bump",
    playerModules: ["bump"],
    teamModules: ["bump"],
    apply: applyBumpEventDerivedStats,
  },
  {
    id: "rush",
    playerModules: [],
    teamModules: ["rush"],
    apply: applyRushEventDerivedStats,
  },
  {
    id: "pass",
    playerModules: ["pass"],
    teamModules: ["pass"],
    apply: applyPassEventDerivedStats,
  },
  {
    id: "one-timer",
    playerModules: ["one_timer"],
    teamModules: ["one_timer"],
    apply: applyOneTimerEventDerivedStats,
  },
  {
    id: "ball-carry",
    playerModules: ["ball_carry", "air_dribble"],
    teamModules: ["ball_carry", "air_dribble"],
    apply: applyBallCarryEventDerivedStats,
  },
  {
    id: "wall-aerial",
    playerModules: ["wall_aerial"],
    teamModules: [],
    apply: applyWallAerialEventDerivedStats,
  },
  {
    id: "wall-aerial-shot",
    playerModules: ["wall_aerial_shot"],
    teamModules: [],
    apply: applyWallAerialShotEventDerivedStats,
  },
  {
    id: "flick",
    playerModules: ["flick"],
    teamModules: [],
    apply: applyFlickEventDerivedStats,
  },
  {
    id: "ceiling-shot",
    playerModules: ["ceiling_shot"],
    teamModules: [],
    apply: applyCeilingShotEventDerivedStats,
  },
  {
    id: "musty-flick",
    playerModules: ["musty_flick"],
    teamModules: [],
    apply: applyMustyFlickEventDerivedStats,
  },
  {
    id: "dodge-reset",
    playerModules: ["dodge_reset"],
    teamModules: [],
    apply: applyDodgeResetEventDerivedStats,
  },
  {
    id: "powerslide",
    playerModules: ["powerslide"],
    teamModules: ["powerslide"],
    apply: applyPowerslideEventDerivedStats,
  },
  {
    id: "touch",
    playerModules: ["touch"],
    teamModules: [],
    apply: applyTouchEventDerivedStats,
  },
  {
    id: "half-volley",
    playerModules: ["half_volley"],
    teamModules: ["half_volley"],
    apply: applyHalfVolleyEventDerivedStats,
  },
];

export function applyStatsTimelineEventDerivedStats(timeline: StatsTimeline): StatsTimeline {
  timeline = hydrateStatsTimelineFrameScaffolding(timeline);
  for (const applier of STATS_TIMELINE_EVENT_DERIVED_APPLIERS) {
    timeline = applier.apply(timeline);
  }
  return timeline;
}

function hydrateStatsTimelineFrameScaffolding(timeline: StatsTimeline): StatsTimeline {
  for (const frame of timeline.frames) {
    frame.team_zero = createTeamStatsSnapshot(
      (frame.team_zero ?? {}) as DeepPartial<typeof frame.team_zero>,
    );
    frame.team_one = createTeamStatsSnapshot(
      (frame.team_one ?? {}) as DeepPartial<typeof frame.team_one>,
    );
    frame.players = frame.players.map((player) =>
      createPlayerStatsSnapshot(player as DeepPartial<typeof player>),
    );
  }
  return timeline;
}

function parseJsonBuffer<T>(decoder: TextDecoder, buffer: ArrayBuffer): T {
  return JSON.parse(decoder.decode(new Uint8Array(buffer))) as T;
}

async function parseStatsTimelineParts(
  decoder: TextDecoder,
  parts: TransferableStatsTimelineParts,
  onProgress?: (progress: ReplayLoadProgress) => void,
): Promise<StatsTimeline> {
  onProgress?.({ stage: "decoding-stats", progress: 0 });
  const config = parseJsonBuffer<StatsTimeline["config"]>(decoder, parts.configBuffer);
  onProgress?.({ stage: "decoding-stats", progress: 0.05 });
  await waitForNextPaint();
  const replayMeta = parseJsonBuffer<StatsTimeline["replay_meta"]>(decoder, parts.replayMetaBuffer);
  onProgress?.({ stage: "decoding-stats", progress: 0.1 });
  await waitForNextPaint();
  const events = parseJsonBuffer<StatsTimeline["events"]>(decoder, parts.eventsBuffer);
  onProgress?.({ stage: "decoding-stats", progress: 0.15 });
  await waitForNextPaint();

  const frames: StatsTimeline["frames"] = [];
  const totalChunks = parts.frameChunkBuffers.length;
  for (let index = 0; index < totalChunks; index += 1) {
    const buffer = parts.frameChunkBuffers[index]!;
    frames.push(...parseJsonBuffer<StatsTimeline["frames"]>(decoder, buffer));
    onProgress?.({
      stage: "decoding-stats",
      processedChunks: index + 1,
      totalChunks,
      progress: 0.15 + ((index + 1) / Math.max(1, totalChunks)) * 0.85,
    });
    await waitForNextPaint();
  }

  if (totalChunks === 0) {
    onProgress?.({ stage: "decoding-stats", progress: 1 });
  }

  return applyStatsTimelineEventDerivedStats({
    config,
    replay_meta: replayMeta,
    events,
    frames,
  });
}

function waitForNextPaint(): Promise<void> {
  if (typeof requestAnimationFrame !== "function") {
    return Promise.resolve();
  }
  return new Promise((done) => requestAnimationFrame(() => done()));
}

export async function loadReplayBundleInWorker(
  bytes: Uint8Array,
  options: {
    onProgress?: (progress: ReplayLoadProgress) => void;
    reportEveryNFrames?: number;
  } = {},
): Promise<ReplayLoadBundle> {
  if (typeof Worker === "undefined") {
    throw new Error("Replay loading worker is not available in this environment");
  }

  const worker = new Worker(new URL("./replayLoader.worker.ts", import.meta.url), {
    type: "module",
  });
  const workerBytes = bytes.slice();
  const reportEveryNFrames = options.reportEveryNFrames ?? 100;

  return new Promise<ReplayLoadBundle>((resolve, reject) => {
    const cleanup = () => {
      worker.terminate();
    };

    worker.onmessage = async (event: MessageEvent<ReplayWorkerMessage>) => {
      const message = event.data;

      if (message.type === "progress") {
        options.onProgress?.(message.progress);
        return;
      }

      if (message.type === "error") {
        cleanup();
        reject(new Error(message.error));
        return;
      }

      cleanup();
      const decoder = new TextDecoder();
      options.onProgress?.({ stage: "decoding-replay", progress: 0 });
      await waitForNextPaint();
      const replay = parseJsonBuffer<ReplayModel>(decoder, message.replayBuffer);
      options.onProgress?.({ stage: "decoding-replay", progress: 1 });
      await waitForNextPaint();
      const statsTimeline = await parseStatsTimelineParts(
        decoder,
        message.statsTimelineParts,
        options.onProgress,
      );
      resolve({
        replay,
        statsTimeline,
      });
    };

    worker.onerror = (event) => {
      cleanup();
      reject(new Error(event.message || "Replay loading worker failed"));
    };

    const request: ReplayLoadRequest = {
      type: "load-replay",
      bytes: workerBytes.buffer,
      reportEveryNFrames,
    };
    worker.postMessage(request, [workerBytes.buffer]);
  });
}
