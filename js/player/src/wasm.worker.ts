/// <reference lib="webworker" />

import * as subtrActor from "@rlrml/subtr-actor";
import { normalizeReplayData } from "./replay-data";
import type { RawReplayFramesData, ReplayLoadProgress } from "./types";
import type { ReplayLoadRequest, ReplayValidation, ReplayWorkerMessage } from "./wasm-messages";
import { toPlainData } from "./wasm-plain-data";

async function ensureBindingsReady(): Promise<void> {
  const maybeInit = (
    subtrActor as typeof subtrActor & {
      default?: () => Promise<unknown>;
    }
  ).default;
  if (typeof maybeInit === "function") {
    await maybeInit();
  }
}

function postMessageToMain(message: ReplayWorkerMessage): void {
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

    const rawBuffer = subtrActor.get_replay_frames_data_json_with_progress(
      bytes,
      (progress: unknown) => {
        postMessageToMain({
          type: "progress",
          progress: progress as ReplayLoadProgress,
        });
      },
      event.data.reportEveryNFrames,
    );

    postMessageToMain({
      type: "progress",
      progress: { stage: "normalizing", progress: 0 },
    });
    const rawReplayData = JSON.parse(new TextDecoder().decode(rawBuffer)) as RawReplayFramesData;
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

    self.postMessage(
      {
        type: "done",
        rawBuffer: rawBuffer.buffer,
        replayBuffer: replayBuffer.buffer,
      },
      [rawBuffer.buffer, replayBuffer.buffer],
    );
  } catch (error: unknown) {
    postMessageToMain({
      type: "error",
      error: error instanceof Error ? error.message : String(error),
    });
  }
};
