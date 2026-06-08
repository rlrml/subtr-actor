import type { ReplayModel } from "@rlrml/player";
import {
  createStatsFrameLookup,
  type CompactStatsTimeline,
  type StatsFrameLookup,
  type StatsTimeline,
} from "./statsTimeline";
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
  statsFrameLookup: StatsFrameLookup;
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
  positioningSummaryBuffer: ArrayBuffer;
  frameChunkBuffers: ArrayBuffer[];
}

type ReplayWorkerMessage = ReplayProgressMessage | ReplayDoneMessage | ReplayErrorMessage;

function errorFromUnknown(error: unknown): Error {
  return error instanceof Error ? error : new Error(String(error));
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
  const positioningSummary = parseJsonBuffer<CompactStatsTimeline["positioning_summary"]>(
    decoder,
    parts.positioningSummaryBuffer,
  );
  onProgress?.({ stage: "decoding-stats", progress: 0.15 });
  await waitForNextPaint();

  const frames: Array<StatsTimeline["frames"][number]> = [];
  const totalChunks = parts.frameChunkBuffers.length;
  for (let index = 0; index < totalChunks; index += 1) {
    const buffer = parts.frameChunkBuffers[index]!;
    frames.push(...parseJsonBuffer<Array<StatsTimeline["frames"][number]>>(decoder, buffer));
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

  return {
    config,
    replay_meta: replayMeta,
    events,
    frames,
    positioning_summary: positioningSummary,
  };
}

export function waitForNextPaint(timeoutMs = 100): Promise<void> {
  if (typeof requestAnimationFrame !== "function") {
    return Promise.resolve();
  }
  return new Promise((done) => {
    let finished = false;
    let timeoutId: ReturnType<typeof setTimeout> | null = null;
    const finish = () => {
      if (finished) {
        return;
      }
      finished = true;
      if (timeoutId !== null) {
        clearTimeout(timeoutId);
      }
      done();
    };

    timeoutId = setTimeout(finish, timeoutMs);
    requestAnimationFrame(() => finish());
  });
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
      try {
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
        const statsFrameLookup = createStatsFrameLookup(statsTimeline);
        resolve({
          replay,
          statsTimeline,
          statsFrameLookup,
        });
      } catch (error) {
        reject(errorFromUnknown(error));
      }
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
