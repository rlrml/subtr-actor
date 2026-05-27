import type {
  RawReplayFramesData,
  ReplayLoadOptions,
  ReplayLoadResult,
} from "./types";
import type { ReplayLoadRequest, ReplayWorkerMessage } from "./wasm-messages";

export async function loadReplayFromBytesWithWorker(
  data: Uint8Array,
  options: ReplayLoadOptions,
): Promise<ReplayLoadResult> {
  const worker = new Worker(new URL("./wasm.worker.ts", import.meta.url), {
    type: "module",
  });
  const workerBytes = data.slice();

  return new Promise<ReplayLoadResult>((resolve, reject) => {
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
      options.onProgress?.({ stage: "decoding-replay", progress: 0 });
      if (typeof requestAnimationFrame === "function") {
        await new Promise<void>((done) => requestAnimationFrame(() => done()));
      }
      const decoder = new TextDecoder();
      const raw = JSON.parse(
        decoder.decode(new Uint8Array(message.rawBuffer)),
      ) as RawReplayFramesData;
      options.onProgress?.({ stage: "decoding-replay", progress: 0.5 });
      const replay = JSON.parse(
        decoder.decode(new Uint8Array(message.replayBuffer)),
      ) as ReplayLoadResult["replay"];
      options.onProgress?.({ stage: "decoding-replay", progress: 1 });
      resolve({
        raw,
        replay,
      });
    };

    worker.onerror = (event) => {
      cleanup();
      reject(new Error(event.message || "Replay loading worker failed"));
    };

    const request: ReplayLoadRequest = {
      type: "load-replay",
      bytes: workerBytes.buffer,
      reportEveryNFrames: options.reportEveryNFrames ?? 1000,
    };
    worker.postMessage(request, [workerBytes.buffer]);
  });
}
