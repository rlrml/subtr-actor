import init, {
  get_replay_frames_data,
  validate_replay,
} from "@subtr-actor-wasm";
import type { RawReplayFramesData, ReplayLoadResult } from "./types";
import { normalizeReplayData } from "./replay-data";

let bindingsReady: Promise<void> | null = null;

type ReplayValidation = {
  valid: boolean;
  message?: string;
  error?: string;
};

export async function ensureBindingsReady(): Promise<void> {
  if (!bindingsReady) {
    bindingsReady = init();
  }
  await bindingsReady;
}

export async function loadReplayFromBytes(
  data: Uint8Array
): Promise<ReplayLoadResult> {
  await ensureBindingsReady();

  const validation = validateReplayBytes(data);
  if (!validation.valid) {
    throw new Error(validation.error ?? "Replay validation failed");
  }

  const raw = get_replay_frames_data(data) as RawReplayFramesData;
  return {
    raw,
    replay: normalizeReplayData(raw),
  };
}

export function validateReplayBytes(data: Uint8Array): {
  valid: boolean;
  message?: string;
  error?: string;
} {
  const result = validate_replay(data) as ReplayValidation | Map<string, unknown>;
  if (result instanceof Map) {
    return Object.fromEntries(result) as ReplayValidation;
  }

  return result;
}
