import type { PlaylistManifestPlaybackOptions } from "./types";
import { isObject } from "./manifest-json";

export function parsePlaybackOptions(value: unknown): PlaylistManifestPlaybackOptions {
  if (!isObject(value)) {
    throw new Error("manifest.playback must be an object");
  }

  if (
    value.advanceMode !== undefined &&
    value.advanceMode !== "auto" &&
    value.advanceMode !== "manual"
  ) {
    throw new Error('manifest.playback.advanceMode must be "auto" or "manual"');
  }

  if (value.endMode !== undefined && value.endMode !== "stop" && value.endMode !== "loop") {
    throw new Error('manifest.playback.endMode must be "stop" or "loop"');
  }

  if (value.advanceOnEnd !== undefined && typeof value.advanceOnEnd !== "boolean") {
    throw new Error("manifest.playback.advanceOnEnd must be a boolean");
  }

  return {
    advanceMode: value.advanceMode ?? (value.advanceOnEnd === true ? "auto" : "manual"),
    endMode: value.endMode ?? "stop",
  };
}
