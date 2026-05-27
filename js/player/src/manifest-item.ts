import type { PlaylistManifestItem } from "./types";
import { parsePlaybackBound } from "./manifest-bound";
import { isObject, isRecordOfUnknown } from "./manifest-json";

export function parseManifestItem(value: unknown, index: number): PlaylistManifestItem {
  const path = `manifest.items[${index}]`;
  if (!isObject(value)) {
    throw new Error(`${path} must be an object`);
  }

  if (typeof value.replay !== "string" || value.replay.trim() === "") {
    throw new Error(`${path}.replay must be a non-empty string`);
  }

  if (value.label !== undefined && typeof value.label !== "string") {
    throw new Error(`${path}.label must be a string when provided`);
  }

  if (value.meta !== undefined && !isRecordOfUnknown(value.meta)) {
    throw new Error(`${path}.meta must be an object when provided`);
  }

  return {
    id:
      typeof value.id === "string" && value.id.trim() !== ""
        ? value.id
        : `${value.replay}:${index}`,
    replay: value.replay,
    start: parsePlaybackBound(value.start, `${path}.start`),
    end: parsePlaybackBound(value.end, `${path}.end`),
    label: typeof value.label === "string" ? value.label : "",
    meta: value.meta ?? {},
  };
}
