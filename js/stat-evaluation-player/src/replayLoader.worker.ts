/// <reference lib="webworker" />

import * as subtrActor from "@colonelpanic8/subtr-actor";
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
  rawReplayDataBuffer: ArrayBuffer;
  statsTimelineBuffer: ArrayBuffer;
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

    const replayBundle = subtrActor.get_replay_bundle_json_with_progress(
      bytes,
      (progress: unknown) => {
        postMessageToMain({
          type: "progress",
          progress: toPlainData(progress) as ReplayLoadProgress,
        });
      },
      event.data.reportEveryNFrames,
    );

    postMessageToMain({
      type: "progress",
      progress: { stage: "normalizing", progress: 0 },
    });

    self.postMessage({
      type: "done",
      rawReplayDataBuffer: replayBundle.rawReplayData.buffer,
      statsTimelineBuffer: replayBundle.statsTimeline.buffer,
    }, [
      replayBundle.rawReplayData.buffer,
      replayBundle.statsTimeline.buffer,
    ]);
  } catch (error: unknown) {
    postMessageToMain({
      type: "error",
      error: error instanceof Error ? error.message : String(error),
    });
  }
};
