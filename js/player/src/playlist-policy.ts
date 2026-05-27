import type {
  PlaylistAdvanceMode,
  PlaylistEndMode,
  PlaylistItem,
  PlaylistLoadSource,
  PlaylistPreloadContext,
  PlaylistPreloadPolicy,
} from "./types";

export function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

export function normalizePreloadPolicy<
  TSource extends PlaylistLoadSource<unknown>,
  TItem extends PlaylistItem<TSource>,
>(options: {
  preloadPolicy?: PlaylistPreloadPolicy<TSource, TItem>;
  preloadRadius?: number;
}): PlaylistPreloadPolicy<TSource, TItem> {
  if (options.preloadPolicy) {
    return options.preloadPolicy;
  }

  if (options.preloadRadius !== undefined) {
    return {
      kind: "adjacent",
      ahead: options.preloadRadius,
      behind: options.preloadRadius,
    };
  }

  return {
    kind: "adjacent",
    ahead: 1,
    behind: 1,
  };
}

export function normalizeAdvanceMode(options: {
  advanceMode?: PlaylistAdvanceMode;
  advanceOnEnd?: boolean;
}): PlaylistAdvanceMode {
  if (options.advanceMode) {
    return options.advanceMode;
  }

  return options.advanceOnEnd === false ? "manual" : "auto";
}

export function normalizeEndMode(options: { endMode?: PlaylistEndMode }): PlaylistEndMode {
  return options.endMode ?? "stop";
}

export function uniqueSourcesFromItems<TSource extends PlaylistLoadSource<unknown>>(
  items: PlaylistItem<TSource>[],
): TSource[] {
  const uniqueSources: TSource[] = [];
  const seenSourceIds = new Set<string>();

  for (const item of items) {
    if (seenSourceIds.has(item.replay.id)) {
      continue;
    }
    seenSourceIds.add(item.replay.id);
    uniqueSources.push(item.replay);
  }

  return uniqueSources;
}

export function resolvePolicySources<
  TSource extends PlaylistLoadSource<unknown>,
  TItem extends PlaylistItem<TSource>,
>(
  items: TItem[],
  currentIndex: number,
  preloadPolicy: PlaylistPreloadPolicy<TSource, TItem>,
): TSource[] {
  const currentItem = items[currentIndex];
  if (!currentItem) {
    return [];
  }

  if (preloadPolicy.kind === "none") {
    return [];
  }

  if (preloadPolicy.kind === "all") {
    return uniqueSourcesFromItems(items).filter((source) => source.id !== currentItem.replay.id);
  }

  if (preloadPolicy.kind === "adjacent") {
    const behind = collectAdjacentSources(
      items,
      currentIndex,
      -1,
      preloadPolicy.behind ?? 0,
      currentItem.replay.id,
    );
    const ahead = collectAdjacentSources(
      items,
      currentIndex,
      1,
      preloadPolicy.ahead,
      currentItem.replay.id,
    );
    return [...behind, ...ahead];
  }

  const preloadContext: PlaylistPreloadContext<TSource, TItem> = {
    items,
    currentIndex,
    currentItem,
  };
  const sourceIds = new Set<string>([currentItem.replay.id]);
  const sources: TSource[] = [];
  const availableSources = new Map<string, TSource>(
    uniqueSourcesFromItems(items).map((source) => [source.id, source]),
  );

  for (const selected of preloadPolicy.pick(preloadContext)) {
    const source = isPlaylistSource(selected) ? selected : availableSources.get(selected);
    if (!source || sourceIds.has(source.id)) {
      continue;
    }
    sourceIds.add(source.id);
    sources.push(source);
  }

  return sources;
}

function isPlaylistSource<TSource extends PlaylistLoadSource<unknown>>(
  value: string | TSource,
): value is TSource {
  return typeof value !== "string";
}

function collectAdjacentSources<TSource extends PlaylistLoadSource<unknown>>(
  items: PlaylistItem<TSource>[],
  currentIndex: number,
  direction: -1 | 1,
  limit: number,
  currentSourceId: string,
): TSource[] {
  const sources: TSource[] = [];
  const seenSourceIds = new Set<string>([currentSourceId]);

  for (
    let index = currentIndex + direction;
    index >= 0 && index < items.length && sources.length < Math.max(0, limit);
    index += direction
  ) {
    const source = items[index]?.replay;
    if (!source || seenSourceIds.has(source.id)) {
      continue;
    }
    seenSourceIds.add(source.id);
    sources.push(source);
  }

  return sources;
}
