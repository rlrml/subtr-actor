/// <reference lib="webworker" />

import * as subtrActor from "@colonelpanic8/subtr-actor";
import { normalizeReplayData } from "./replay-data";
import type {
  RawReplayFramesData,
  ReplayLoadProgress,
  ReplayLoadResult,
} from "./types";

type ReplayValidation = {
  valid: boolean;
  message?: string;
  error?: string;
};

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
  result: ReplayLoadResult;
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

    const validation = toPlainData(
      subtrActor.validate_replay(bytes) as ReplayValidation | Map<string, unknown>,
    ) as ReplayValidation;
    if (!validation.valid) {
      throw new Error(validation.error ?? "Replay validation failed");
    }

    const raw = toPlainData(
      subtrActor.get_replay_frames_data_with_progress(
        bytes,
        (progress: unknown) => {
          postMessageToMain({
            type: "progress",
            progress: progress as ReplayLoadProgress,
          });
        },
        event.data.reportEveryNFrames,
      ),
    ) as RawReplayFramesData;

    postMessageToMain({
      type: "progress",
      progress: { stage: "normalizing", progress: 0 },
    });

    postMessageToMain({
      type: "done",
      result: {
        raw,
        replay: normalizeReplayData(raw),
      },
    });
  } catch (error: unknown) {
    postMessageToMain({
      type: "error",
      error: error instanceof Error ? error.message : String(error),
    });
  }
};
