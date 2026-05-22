/// <reference lib="webworker" />

import * as subtrActor from "@colonelpanic8/subtr-actor";
import { normalizeReplayData } from "subtr-actor-player";
import type { RawReplayFramesData } from "subtr-actor-player";
import type { ReplayLoadProgress } from "./replayLoader";

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

interface TransferableStatsTimelineParts {
  configBuffer: ArrayBuffer;
  replayMetaBuffer: ArrayBuffer;
  eventsBuffer: ArrayBuffer;
  frameChunkBuffers: ArrayBuffer[];
}

interface ReplayErrorMessage {
  type: "error";
  error: string;
}

type ReplayWorkerResponse =
  | ReplayProgressMessage
  | ReplayDoneMessage
  | ReplayErrorMessage;

function toPlainData<T>(value: T): T {
  if (value instanceof Map) {
    return Object.fromEntries(
      Array.from(value.entries()).map(([key, entry]) => [key, toPlainData(entry)]),
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

async function ensureBindingsReady(): Promise<void> {
  const maybeInit = (subtrActor as typeof subtrActor & {
    default?: () => Promise<unknown>;
  }).default;
  if (typeof maybeInit === "function") {
    await maybeInit();
  }
}

function postMessageToMain(message: ReplayWorkerResponse): void {
  self.postMessage(message);
}

self.onmessage = async (event: MessageEvent<ReplayLoadRequest>) => {
  if (event.data.type !== "load-replay") {
    return;
  }

  try {
    await ensureBindingsReady();

    const bytes = new Uint8Array(event.data.bytes);
    postMessageToMain({
      type: "progress",
      progress: { stage: "validating", progress: 0 },
    });

    const replayBundle = subtrActor.get_replay_bundle_json_parts_with_progress(
      bytes,
      (progress: unknown) => {
        postMessageToMain({
          type: "progress",
          progress: toPlainData(progress) as ReplayLoadProgress,
        });
      },
      event.data.reportEveryNFrames,
      32 * 1024 * 1024,
    );

    postMessageToMain({
      type: "progress",
      progress: { stage: "decoding-replay", progress: 0 },
    });
    const rawReplayData = JSON.parse(
      new TextDecoder().decode(replayBundle.rawReplayData),
    ) as RawReplayFramesData;
    postMessageToMain({
      type: "progress",
      progress: { stage: "normalizing", progress: 0 },
    });
    const replay = normalizeReplayData(rawReplayData, {
      progressReportFrameInterval: event.data.reportEveryNFrames,
      onProgress(progress) {
        postMessageToMain({
          type: "progress",
          progress: { stage: "normalizing", progress },
        });
      },
    });
    const replayBuffer = new TextEncoder().encode(JSON.stringify(replay));

    const frameChunkBuffers = Array.from(
      replayBundle.statsTimelineParts.frameChunks,
      (chunk) => chunk.buffer,
    );

    self.postMessage({
      type: "done",
      replayBuffer: replayBuffer.buffer,
      statsTimelineParts: {
        configBuffer: replayBundle.statsTimelineParts.config.buffer,
        replayMetaBuffer: replayBundle.statsTimelineParts.replayMeta.buffer,
        eventsBuffer: replayBundle.statsTimelineParts.events.buffer,
        frameChunkBuffers,
      },
    }, [
      replayBuffer.buffer,
      replayBundle.statsTimelineParts.config.buffer,
      replayBundle.statsTimelineParts.replayMeta.buffer,
      replayBundle.statsTimelineParts.events.buffer,
      ...frameChunkBuffers,
    ]);
  } catch (error: unknown) {
    postMessageToMain({
      type: "error",
      error: error instanceof Error ? error.message : String(error),
    });
  }
};
