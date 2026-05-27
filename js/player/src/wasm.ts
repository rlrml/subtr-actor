import * as subtrActor from "@rlrml/subtr-actor";
import type {
  RawReplayFramesData,
  ReplayLoadOptions,
  ReplayLoadProgress,
  ReplayLoadResult,
} from "./types";
import { normalizeReplayDataAsync } from "./replay-data";
import type { ReplayValidation } from "./wasm-messages";
import { toPlainData } from "./wasm-plain-data";
import { loadReplayFromBytesWithWorker } from "./wasm-worker-client";

let bindingsReady: Promise<unknown> | null = null;

export async function ensureBindingsReady(): Promise<void> {
  if (!bindingsReady) {
    const maybeInit = (
      subtrActor as typeof subtrActor & {
        default?: () => Promise<unknown>;
      }
    ).default;
    bindingsReady = typeof maybeInit === "function" ? maybeInit() : Promise.resolve();
  }
  await bindingsReady;
}

function shouldUseWorker(options: ReplayLoadOptions): boolean {
  if (options.useWorker !== undefined) {
    return options.useWorker && typeof Worker !== "undefined";
  }

  return typeof Worker !== "undefined";
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
    subtrActor.validate_replay(data) as ReplayValidation | Map<string, unknown>,
  );
  return result as ReplayValidation;
}
