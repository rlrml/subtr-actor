import type {
  PlaybackBound,
  PlaylistItem,
  PlaylistManifest,
  PlaylistManifestItem,
  PlaylistManifestReplay,
  ReplaySource,
} from "./types";

type JsonObject = Record<string, unknown>;

function isObject(value: unknown): value is JsonObject {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function isRecordOfUnknown(value: unknown): value is Record<string, unknown> {
  return isObject(value);
}

function parsePlaybackBound(
  value: unknown,
  path: string
): PlaybackBound {
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

function parseManifestReplay(
  value: unknown,
  index: number
): PlaylistManifestReplay {
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

  return {
    id: value.id,
    path: value.path,
    label: value.label,
    meta: value.meta,
  };
}

function parseManifestItem(
  value: unknown,
  index: number
): PlaylistManifestItem {
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
    replay: value.replay,
    start: parsePlaybackBound(value.start, `${path}.start`),
    end: parsePlaybackBound(value.end, `${path}.end`),
    label: value.label,
    meta: value.meta,
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

  return {
    replays: manifest.replays?.map(parseManifestReplay),
    items: manifest.items.map(parseManifestItem),
    label: manifest.label,
    meta: manifest.meta,
  };
}

export async function loadPlaylistManifestFromFile(
  file: Blob
): Promise<PlaylistManifest> {
  const text = await file.text();
  let parsed: unknown;
  try {
    parsed = JSON.parse(text);
  } catch (error) {
    throw new Error(
      `Failed to parse playlist manifest JSON: ${
        error instanceof Error ? error.message : String(error)
      }`
    );
  }

  return parsePlaylistManifest(parsed);
}

export function resolvePlaylistItemsFromManifest(
  manifest: PlaylistManifest,
  resolveReplaySource: (context: {
    replayId: string;
    replay?: PlaylistManifestReplay;
  }) => ReplaySource
): PlaylistItem[] {
  const replaysById = new Map<string, PlaylistManifestReplay>(
    (manifest.replays ?? []).map((replay) => [replay.id, replay])
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
      label: item.label ?? replay?.label,
      meta: item.meta,
    };
  });
}
