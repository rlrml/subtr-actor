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
  rawReplayDataBuffer: ArrayBuffer;
  statsTimelineBuffer: ArrayBuffer;
}

interface ReplayErrorMessage {
  type: "error";
  error: string;
}

type ReplayWorkerMessage =
  | ReplayProgressMessage
  | ReplayDoneMessage
  | ReplayErrorMessage;

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
  const reportEveryNFrames = options.reportEveryNFrames ?? 1000;

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
      options.onProgress?.({ stage: "normalizing", progress: 0.1 });
      if (typeof requestAnimationFrame === "function") {
        await new Promise<void>((done) => requestAnimationFrame(() => done()));
      }
      const rawReplayData = JSON.parse(
        decoder.decode(new Uint8Array(message.rawReplayDataBuffer)),
      ) as RawReplayFramesData;
      options.onProgress?.({ stage: "normalizing", progress: 0.4 });
      const statsTimeline = JSON.parse(
        decoder.decode(new Uint8Array(message.statsTimelineBuffer)),
      ) as StatsTimeline;
      options.onProgress?.({ stage: "normalizing", progress: 0.65 });
      const replay = await normalizeReplayDataAsync(rawReplayData, {
        onProgress(progress) {
          options.onProgress?.({
            stage: "normalizing",
            progress: 0.65 + (progress * 0.35),
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
