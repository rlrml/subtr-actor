import type { PlaylistManifestReplay, PlaylistManifestReplayLocator } from "./types";
import { isObject, isRecordOfUnknown } from "./manifest-json";

export function parseManifestReplay(value: unknown, index: number): PlaylistManifestReplay {
  const path = `manifest.replays[${index}]`;
  if (!isObject(value)) {
    throw new Error(`${path} must be an object`);
  }

  if (typeof value.id !== "string" || value.id.trim() === "") {
    throw new Error(`${path}.id must be a non-empty string`);
  }

  if (value.path !== undefined && typeof value.path !== "string") {
    throw new Error(`${path}.path must be a string when provided`);
  }

  if (value.label !== undefined && typeof value.label !== "string") {
    throw new Error(`${path}.label must be a string when provided`);
  }

  if (value.meta !== undefined && !isRecordOfUnknown(value.meta)) {
    throw new Error(`${path}.meta must be an object when provided`);
  }

  const replayPath = typeof value.path === "string" ? value.path : "";
  return {
    id: value.id,
    path: replayPath,
    label: typeof value.label === "string" ? value.label : value.id,
    locator: parseManifestReplayLocator(value.locator, `${path}.locator`, replayPath),
    meta: value.meta ?? {},
  };
}

function parseManifestReplayLocator(
  value: unknown,
  path: string,
  fallbackPath: string,
): PlaylistManifestReplayLocator {
  if (value === undefined) {
    return fallbackPath ? { kind: "path", path: fallbackPath } : { kind: "inline" };
  }

  if (!isObject(value)) {
    throw new Error(`${path} must be an object when provided`);
  }

  if (typeof value.kind !== "string" || value.kind.trim() === "") {
    throw new Error(`${path}.kind must be a non-empty string`);
  }

  if (value.id !== undefined && typeof value.id !== "string") {
    throw new Error(`${path}.id must be a string when provided`);
  }

  if (value.path !== undefined && typeof value.path !== "string") {
    throw new Error(`${path}.path must be a string when provided`);
  }

  if (value.cachePath !== undefined && typeof value.cachePath !== "string") {
    throw new Error(`${path}.cachePath must be a string when provided`);
  }

  return {
    kind: value.kind,
    id: value.id,
    path: value.path,
    cachePath: value.cachePath,
  };
}
