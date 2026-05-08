import { normalizeReplayDataAsync } from "subtr-actor-player";
import type { ReplayModel, RawReplayFramesData } from "subtr-actor-player";
import type { StatsTimeline } from "./statsTimeline";
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
  statsTimelineParts: TransferableStatsTimelineParts;
}

interface ReplayRawDataMessage {
  type: "raw-replay-data";
  rawReplayDataBuffer: ArrayBuffer;
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

type ReplayWorkerMessage =
  | ReplayProgressMessage
  | ReplayRawDataMessage
  | ReplayDoneMessage
  | ReplayErrorMessage;

function parseJsonBuffer<T>(decoder: TextDecoder, buffer: ArrayBuffer): T {
  return JSON.parse(decoder.decode(new Uint8Array(buffer))) as T;
}

async function parseStatsTimelineParts(
  decoder: TextDecoder,
  parts: TransferableStatsTimelineParts,
  onProgress?: (progress: ReplayLoadProgress) => void,
): Promise<StatsTimeline> {
  const config = parseJsonBuffer<StatsTimeline["config"]>(
    decoder,
    parts.configBuffer,
  );
  const replayMeta = parseJsonBuffer<StatsTimeline["replay_meta"]>(
    decoder,
    parts.replayMetaBuffer,
  );
  const events = parseJsonBuffer<StatsTimeline["events"]>(
    decoder,
    parts.eventsBuffer,
  );
  onProgress?.({ stage: "stats-timeline", progress: 0.96 });

  const frames: StatsTimeline["frames"] = [];
  const totalChunks = parts.frameChunkBuffers.length;
  for (let index = 0; index < totalChunks; index += 1) {
    const buffer = parts.frameChunkBuffers[index]!;
    frames.push(...parseJsonBuffer<StatsTimeline["frames"]>(decoder, buffer));
    onProgress?.({
      stage: "stats-timeline",
      progress: 0.96 + (((index + 1) / Math.max(1, totalChunks)) * 0.04),
    });
    await waitForNextPaint();
  }

  return {
    config,
    replay_meta: replayMeta,
    events,
    frames,
  };
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
    let rawReplayDataBuffer: ArrayBuffer | null = null;
    const cleanup = () => {
      worker.terminate();
    };

    worker.onmessage = async (event: MessageEvent<ReplayWorkerMessage>) => {
      const message = event.data;

      if (message.type === "progress") {
        options.onProgress?.(message.progress);
        return;
      }

      if (message.type === "raw-replay-data") {
        rawReplayDataBuffer = message.rawReplayDataBuffer;
        return;
      }

      if (message.type === "error") {
        cleanup();
        reject(new Error(message.error));
        return;
      }

      cleanup();
      if (!rawReplayDataBuffer) {
        reject(new Error("Replay loading worker finished without replay data"));
        return;
      }
      const decoder = new TextDecoder();
      options.onProgress?.({ stage: "stats-timeline", progress: 0.92 });
      await waitForNextPaint();
      const rawReplayData = JSON.parse(
        decoder.decode(new Uint8Array(rawReplayDataBuffer)),
      ) as RawReplayFramesData;
      options.onProgress?.({ stage: "stats-timeline", progress: 0.95 });
      await waitForNextPaint();
      const statsTimeline = await parseStatsTimelineParts(
        decoder,
        message.statsTimelineParts,
        options.onProgress,
      );
      options.onProgress?.({ stage: "normalizing", progress: 0 });
      await waitForNextPaint();
      const replay = await normalizeReplayDataAsync(rawReplayData, {
        progressReportFrameInterval: reportEveryNFrames,
        onProgress(progress) {
          options.onProgress?.({
            stage: "normalizing",
            progress,
          });
        },
      });
      options.onProgress?.({ stage: "normalizing", progress: 1 });
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
