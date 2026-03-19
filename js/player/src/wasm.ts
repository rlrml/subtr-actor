import * as subtrActor from "@colonelpanic8/subtr-actor";
import type { RawReplayFramesData, ReplayLoadResult } from "./types";
import { normalizeReplayData } from "./replay-data";

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

export async function loadReplayFromBytes(
  data: Uint8Array
): Promise<ReplayLoadResult> {
  await ensureBindingsReady();

  const validation = validateReplayBytes(data);
  if (!validation.valid) {
    throw new Error(validation.error ?? "Replay validation failed");
  }

  const raw = toPlainData(
    subtrActor.get_replay_frames_data(data),
  ) as RawReplayFramesData;
  return {
    raw,
    replay: normalizeReplayData(raw),
  };
}

export function validateReplayBytes(data: Uint8Array): ReplayValidation {
  const result = toPlainData(
    subtrActor.validate_replay(data) as ReplayValidation | Map<string, unknown>
  );
  return result as ReplayValidation;
}
