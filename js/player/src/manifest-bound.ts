import type { PlaybackBound } from "./types";
import { isObject } from "./manifest-json";

export function parsePlaybackBound(value: unknown, path: string): PlaybackBound {
  if (!isObject(value)) {
    throw new Error(`${path} must be an object`);
  }

  const kind = value.kind;
  const rawBoundValue = value.value;
  if (kind !== "frame" && kind !== "time") {
    throw new Error(`${path}.kind must be "frame" or "time"`);
  }
  if (typeof rawBoundValue !== "number" || !Number.isFinite(rawBoundValue)) {
    throw new Error(`${path}.value must be a finite number`);
  }

  return {
    kind,
    value: rawBoundValue,
  };
}
