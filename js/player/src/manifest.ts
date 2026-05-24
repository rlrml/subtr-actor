import type {
  PlaybackBound,
  PlaylistItem,
  PlaylistManifest,
  PlaylistManifestItem,
  PlaylistManifestPage,
  PlaylistManifestPlaybackOptions,
  PlaylistManifestReplay,
  PlaylistManifestReplayLocator,
  ReplaySource,
} from "./types";

type JsonObject = Record<string, unknown>;

function isObject(value: unknown): value is JsonObject {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function isRecordOfUnknown(value: unknown): value is Record<string, unknown> {
  return isObject(value);
}

function parsePlaybackBound(value: unknown, path: string): PlaybackBound {
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

function parseManifestReplay(value: unknown, index: number): PlaylistManifestReplay {
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

function parseManifestItem(value: unknown, index: number): PlaylistManifestItem {
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

function parsePlaybackOptions(value: unknown): PlaylistManifestPlaybackOptions {
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

function parseOptionalFiniteNonnegativeInteger(value: unknown, path: string): number | undefined {
  if (value === undefined || value === null) {
    return undefined;
  }
  if (
    typeof value !== "number" ||
    !Number.isInteger(value) ||
    !Number.isFinite(value) ||
    value < 0
  ) {
    throw new Error(`${path} must be a non-negative integer when provided`);
  }
  return value;
}

function parseOptionalString(value: unknown, path: string): string | undefined {
  if (value === undefined || value === null) {
    return undefined;
  }
  if (typeof value !== "string") {
    throw new Error(`${path} must be a string when provided`);
  }
  return value;
}

function parseManifestPage(value: unknown): PlaylistManifestPage {
  if (!isObject(value)) {
    throw new Error("manifest.page must be an object when provided");
  }

  return {
    next: parseOptionalString(value.next, "manifest.page.next"),
    previous: parseOptionalString(value.previous, "manifest.page.previous"),
    total: parseOptionalFiniteNonnegativeInteger(value.total, "manifest.page.total"),
    count: parseOptionalFiniteNonnegativeInteger(value.count, "manifest.page.count"),
    limit: parseOptionalFiniteNonnegativeInteger(value.limit, "manifest.page.limit"),
    offset: parseOptionalFiniteNonnegativeInteger(value.offset, "manifest.page.offset"),
  };
}

export function parsePlaylistManifest(manifest: unknown): PlaylistManifest {
  if (!isObject(manifest)) {
    throw new Error("manifest must be an object");
  }

  if (!Array.isArray(manifest.items)) {
    throw new Error("manifest.items must be an array");
  }

  if (manifest.replays !== undefined && !Array.isArray(manifest.replays)) {
    throw new Error("manifest.replays must be an array when provided");
  }

  if (manifest.label !== undefined && typeof manifest.label !== "string") {
    throw new Error("manifest.label must be a string when provided");
  }

  if (manifest.meta !== undefined && !isRecordOfUnknown(manifest.meta)) {
    throw new Error("manifest.meta must be an object when provided");
  }

  const playback =
    manifest.playback === undefined
      ? { advanceMode: "manual" as const, endMode: "stop" as const }
      : parsePlaybackOptions(manifest.playback);

  return {
    version: typeof manifest.version === "number" ? manifest.version : 1,
    kind: typeof manifest.kind === "string" ? manifest.kind : "playlist",
    replays: (manifest.replays ?? []).map(parseManifestReplay),
    items: manifest.items.map(parseManifestItem),
    label: typeof manifest.label === "string" ? manifest.label : "Playlist",
    page: manifest.page === undefined ? undefined : parseManifestPage(manifest.page),
    meta: manifest.meta ?? {},
    playback,
  };
}

export async function loadPlaylistManifestFromFile(file: Blob): Promise<PlaylistManifest> {
  const text = await file.text();
  let parsed: unknown;
  try {
    parsed = JSON.parse(text);
  } catch (error) {
    throw new Error(
      `Failed to parse playlist manifest JSON: ${
        error instanceof Error ? error.message : String(error)
      }`,
    );
  }

  return parsePlaylistManifest(parsed);
}

export function resolvePlaylistItemsFromManifest(
  manifest: PlaylistManifest,
  resolveReplaySource: (context: {
    replayId: string;
    replay?: PlaylistManifestReplay;
  }) => ReplaySource,
): PlaylistItem[] {
  const replaysById = new Map<string, PlaylistManifestReplay>(
    (manifest.replays ?? []).map((replay) => [replay.id, replay]),
  );

  return manifest.items.map((item) => {
    const replay = replaysById.get(item.replay);
    return {
      replay: resolveReplaySource({
        replayId: item.replay,
        replay,
      }),
      start: item.start,
      end: item.end,
      label: item.label || replay?.label,
      meta: item.meta,
    };
  });
}
