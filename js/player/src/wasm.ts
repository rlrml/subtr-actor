import * as subtrActor from "@colonelpanic8/subtr-actor";
import type {
  RawReplayFramesData,
  ReplayLoadOptions,
  ReplayLoadProgress,
  ReplayLoadResult,
} from "./types";
import { normalizeReplayDataAsync } from "./replay-data";

let bindingsReady: Promise<unknown> | null = null;

type ReplayValidation = {
  valid: boolean;
  message?: string;
  error?: string;
};

function toPlainData<T>(value: T): T {
  if (value instanceof Map) {
    return Object.fromEntries(
      Array.from(value.entries()).map(([key, entry]) => [key, toPlainData(entry)])
    ) as T;
  }

  if (Array.isArray(value)) {
    return value.map((entry) => toPlainData(entry)) as T;
  }

  if (value && typeof value === "object") {
    const result: Record<string, unknown> = {};
    for (const [key, entry] of Object.entries(value as Record<string, unknown>)) {
      result[key] = toPlainData(entry);
    }
    return result as T;
  }

  return value;
}

export async function ensureBindingsReady(): Promise<void> {
  if (!bindingsReady) {
    const maybeInit = (subtrActor as typeof subtrActor & {
      default?: () => Promise<unknown>;
    }).default;
    bindingsReady = typeof maybeInit === "function"
      ? maybeInit()
      : Promise.resolve();
  }
  await bindingsReady;
}

function shouldUseWorker(options: ReplayLoadOptions): boolean {
  if (options.useWorker !== undefined) {
    return options.useWorker && typeof Worker !== "undefined";
  }

  return typeof Worker !== "undefined" && options.onProgress !== undefined;
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
  rawBuffer: ArrayBuffer;
}

interface ReplayErrorMessage {
  type: "error";
  error: string;
}

type ReplayWorkerMessage =
  | ReplayProgressMessage
  | ReplayDoneMessage
  | ReplayErrorMessage;

async function loadReplayFromBytesWithWorker(
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
      options.onProgress?.({ stage: "normalizing", progress: 0.1 });
      if (typeof requestAnimationFrame === "function") {
        await new Promise<void>((done) => requestAnimationFrame(() => done()));
      }
      const rawJson = new TextDecoder().decode(new Uint8Array(message.rawBuffer));
      options.onProgress?.({ stage: "normalizing", progress: 0.45 });
      const raw = JSON.parse(rawJson) as RawReplayFramesData;
      options.onProgress?.({ stage: "normalizing", progress: 0.65 });
      const replay = await normalizeReplayDataAsync(raw, {
        onProgress(progress) {
          options.onProgress?.({
            stage: "normalizing",
            progress: 0.65 + (progress * 0.35),
          });
        },
      });
      options.onProgress?.({ stage: "normalizing", progress: 1 });
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

export async function loadReplayFromBytes(
  data: Uint8Array,
  options: ReplayLoadOptions = {},
): Promise<ReplayLoadResult> {
  if (shouldUseWorker(options)) {
    return loadReplayFromBytesWithWorker(data, options);
  }

  await ensureBindingsReady();

  options.onProgress?.({ stage: "validating", progress: 0 });
  const validation = validateReplayBytes(data);
  if (!validation.valid) {
    throw new Error(validation.error ?? "Replay validation failed");
  }

  options.onProgress?.({ stage: "processing", progress: 0 });
  const raw = toPlainData(
    options.onProgress
      ? subtrActor.get_replay_frames_data_with_progress(
        data,
        (progress: unknown) => {
          options.onProgress?.(progress as ReplayLoadProgress);
        },
        options.reportEveryNFrames ?? 1000,
      )
      : subtrActor.get_replay_frames_data(data),
  ) as RawReplayFramesData;
  options.onProgress?.({ stage: "normalizing", progress: 0 });
  const replay = await normalizeReplayDataAsync(raw, {
    onProgress(progress) {
      options.onProgress?.({ stage: "normalizing", progress });
    },
  });
  return {
    raw,
    replay,
  };
}

export function validateReplayBytes(data: Uint8Array): ReplayValidation {
  const result = toPlainData(
    subtrActor.validate_replay(data) as ReplayValidation | Map<string, unknown>
  );
  return result as ReplayValidation;
}
