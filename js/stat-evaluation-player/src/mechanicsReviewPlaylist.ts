import type { PlaylistManifestPage } from "@rlrml/player";
import type {
  MechanicsReviewItem,
  MechanicsReviewPlaybackBound,
  MechanicsReviewPlaylist,
  MechanicsReviewReplay,
} from "./mechanicsReviewTypes.ts";

export function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function parseMechanicsReviewBound(value: unknown): MechanicsReviewPlaybackBound | null {
  if (!isRecord(value)) {
    return null;
  }
  if (
    (value.kind === "time" || value.kind === "frame") &&
    typeof value.value === "number" &&
    Number.isFinite(value.value)
  ) {
    return {
      kind: value.kind,
      value: value.value,
    };
  }
  return null;
}

function parseOptionalMechanicsReviewPageInteger(
  value: unknown,
  field: string,
): number | undefined {
  if (value === undefined || value === null) {
    return undefined;
  }
  if (
    typeof value !== "number" ||
    !Number.isInteger(value) ||
    !Number.isFinite(value) ||
    value < 0
  ) {
    throw new Error(`Review playlist page ${field} must be a non-negative integer.`);
  }
  return value;
}

function parseOptionalMechanicsReviewPageString(value: unknown, field: string): string | undefined {
  if (value === undefined || value === null) {
    return undefined;
  }
  if (typeof value !== "string") {
    throw new Error(`Review playlist page ${field} must be a string.`);
  }
  return value;
}

function parseMechanicsReviewPage(value: unknown): PlaylistManifestPage | undefined {
  if (value === undefined || value === null) {
    return undefined;
  }
  if (!isRecord(value)) {
    throw new Error("Review playlist page must be an object.");
  }

  return {
    next: parseOptionalMechanicsReviewPageString(value.next, "next"),
    previous: parseOptionalMechanicsReviewPageString(value.previous, "previous"),
    total: parseOptionalMechanicsReviewPageInteger(value.total, "total"),
    count: parseOptionalMechanicsReviewPageInteger(value.count, "count"),
    limit: parseOptionalMechanicsReviewPageInteger(value.limit, "limit"),
    offset: parseOptionalMechanicsReviewPageInteger(value.offset, "offset"),
  };
}

export function parseMechanicsReviewPlaylist(value: unknown): MechanicsReviewPlaylist {
  if (!isRecord(value) || !Array.isArray(value.items)) {
    throw new Error("Review playlist must contain an items array.");
  }

  const items = value.items.map((rawItem, index): MechanicsReviewItem => {
    if (!isRecord(rawItem) || typeof rawItem.replay !== "string") {
      throw new Error(`Invalid review item at index ${index}.`);
    }
    const start = parseMechanicsReviewBound(rawItem.start);
    const end = parseMechanicsReviewBound(rawItem.end);
    if (!start || !end) {
      throw new Error(`Review item ${index + 1} has invalid start or end.`);
    }
    return {
      id: typeof rawItem.id === "string" ? rawItem.id : undefined,
      replay: rawItem.replay,
      start,
      end,
      label: typeof rawItem.label === "string" ? rawItem.label : undefined,
      meta: isRecord(rawItem.meta) ? rawItem.meta : undefined,
    };
  });

  const replays = Array.isArray(value.replays)
    ? value.replays
        .map((rawReplay): MechanicsReviewReplay | null => {
          if (!isRecord(rawReplay) || typeof rawReplay.id !== "string") {
            return null;
          }
          return {
            id: rawReplay.id,
            path: typeof rawReplay.path === "string" ? rawReplay.path : undefined,
            label: typeof rawReplay.label === "string" ? rawReplay.label : undefined,
            locator: isRecord(rawReplay.locator) ? rawReplay.locator : undefined,
            meta: isRecord(rawReplay.meta) ? rawReplay.meta : undefined,
          };
        })
        .filter((replay): replay is MechanicsReviewReplay => replay !== null)
    : undefined;

  return {
    label: typeof value.label === "string" ? value.label : undefined,
    replays,
    items,
    page: parseMechanicsReviewPage(value.page),
    playback: value.playback,
    meta: value.meta,
  };
}

export function parseMechanicsReviewPlaylistJson(text: string): MechanicsReviewPlaylist {
  let parsed: unknown;
  try {
    parsed = JSON.parse(text);
  } catch (error) {
    throw new Error(
      `Invalid review playlist JSON: ${error instanceof Error ? error.message : String(error)}`,
    );
  }
  return parseMechanicsReviewPlaylist(parsed);
}
