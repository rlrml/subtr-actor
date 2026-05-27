import type { PlaylistManifest } from "./types";
import { isObject, isRecordOfUnknown } from "./manifest-json";
import { parseManifestItem } from "./manifest-item";
import { parseManifestPage } from "./manifest-page";
import { parsePlaybackOptions } from "./manifest-playback";
import { parseManifestReplay } from "./manifest-replay";

export { resolvePlaylistItemsFromManifest } from "./manifest-resolve";

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
