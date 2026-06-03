import { loadReplayFromBytes } from "./wasm";
import type {
  LoadedReplay,
  PlaybackBound,
  PlaylistAdvanceMode,
  PlaylistEndMode,
  PlaylistItem,
  PlaylistLoadSource,
  PlaylistPreloadContext,
  PlaylistPreloadPolicy,
  PlaylistSourceLoadContext,
  PlaylistSourceLoadProgress,
  PlaylistSourceLoadState,
  ReplaySource,
} from "./types";

type ReplayPathLoader = (
  path: string,
  context?: PlaylistSourceLoadContext,
) => Promise<LoadedReplay>;
type ReplaySourceLoader = (context?: PlaylistSourceLoadContext) => Promise<LoadedReplay>;

export interface FullReplayPlaylistItemOptions {
  label?: string;
  meta?: Record<string, unknown>;
}

function isPlaylistSource<TSource extends PlaylistLoadSource<unknown>>(
  value: string | TSource,
): value is TSource {
  return typeof value !== "string";
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

function describeError(error: unknown): string {
  return error instanceof Error ? error.message : "Failed to load replay";
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

export function frameBound(value: number): PlaybackBound {
  return { kind: "frame", value };
}

export function timeBound(value: number): PlaybackBound {
  return { kind: "time", value };
}

export function createReplaySource(id: string, load: ReplaySourceLoader): ReplaySource {
  return { id, load };
}

export function createStaticReplaySource(id: string, replay: LoadedReplay): ReplaySource {
  return createReplaySource(id, async () => replay);
}

export function createReplayBytesSource(id: string, data: Uint8Array): ReplaySource {
  return createReplaySource(id, async () => loadReplayFromBytes(data, { useWorker: true }));
}

export function createReplayFileSource(
  file: File,
  id = file.webkitRelativePath || file.name,
): ReplaySource {
  return createReplaySource(id, async () => {
    const bytes = new Uint8Array(await file.arrayBuffer());
    return loadReplayFromBytes(bytes, { useWorker: true });
  });
}

export function createReplayPathSource(
  path: string,
  loadReplay: ReplayPathLoader,
  id = path,
): ReplaySource {
  return createReplaySource(id, async (context) => loadReplay(path, context));
}

export function createFullReplayPlaylistItem(
  replay: ReplaySource,
  options: FullReplayPlaylistItemOptions = {},
): PlaylistItem {
  return {
    replay,
    start: timeBound(0),
    end: timeBound(Number.POSITIVE_INFINITY),
    label: options.label,
    meta: options.meta,
  };
}

export class PlaylistLoadCache<
  TLoaded,
  TSource extends PlaylistLoadSource<TLoaded> = PlaylistLoadSource<TLoaded>,
> {
  private readonly cache = new Map<string, Promise<TLoaded>>();
  private readonly states = new Map<string, PlaylistSourceLoadState>();
  private readonly listeners = new Set<() => void>();

  load(source: TSource): Promise<TLoaded> {
    const cached = this.cache.get(source.id);
    if (cached) {
      return cached;
    }

    this.setSourceState(source.id, {
      status: "loading",
      progress: null,
      error: null,
      startedAt: Date.now(),
      completedAt: null,
    });
    const context: PlaylistSourceLoadContext = {
      sourceId: source.id,
      updateProgress: (progress) => this.updateProgress(source.id, progress),
    };
    const loadPromise = Promise.resolve()
      .then(() => source.load(context))
      .then((loaded) => {
        this.setSourceState(source.id, {
          status: "loaded",
          progress: null,
          error: null,
          completedAt: Date.now(),
        });
        return loaded;
      })
      .catch((error) => {
        this.cache.delete(source.id);
        this.setSourceState(source.id, {
          status: "error",
          error: describeError(error),
          completedAt: Date.now(),
        });
        throw error;
      });
    this.cache.set(source.id, loadPromise);
    return loadPromise;
  }

  preload(sources: Iterable<TSource>): void {
    for (const source of sources) {
      void this.load(source).catch(() => {
        // Preload errors are exposed through cache state and should not surface
        // as unhandled promise rejections.
      });
    }
  }

  has(source: TSource | string): boolean {
    return this.cache.has(typeof source === "string" ? source : source.id);
  }

  delete(source: TSource | string): boolean {
    const sourceId = typeof source === "string" ? source : source.id;
    const deleted = this.cache.delete(sourceId);
    if (deleted) {
      this.states.delete(sourceId);
      this.emitChange();
    }
    return deleted;
  }

  clear(): void {
    this.cache.clear();
    if (this.states.size > 0) {
      this.states.clear();
      this.emitChange();
    }
  }

  getState(source: TSource | string): PlaylistSourceLoadState {
    const sourceId = typeof source === "string" ? source : source.id;
    return (
      this.states.get(sourceId) ?? {
        sourceId,
        status: "idle",
        progress: null,
        error: null,
        startedAt: null,
        updatedAt: null,
        completedAt: null,
      }
    );
  }

  getStates(): PlaylistSourceLoadState[] {
    return Array.from(this.states.values());
  }

  subscribe(listener: () => void): () => void {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  }

  private updateProgress(sourceId: string, progress: PlaylistSourceLoadProgress): void {
    const current = this.getState(sourceId);
    this.setSourceState(sourceId, {
      status: current.status === "idle" ? "loading" : current.status,
      progress,
      updatedAt: Date.now(),
    });
  }

  private setSourceState(
    sourceId: string,
    patch: Partial<Omit<PlaylistSourceLoadState, "sourceId">>,
  ): void {
    const current = this.getState(sourceId);
    this.states.set(sourceId, {
      ...current,
      ...patch,
      sourceId,
      updatedAt: patch.updatedAt ?? Date.now(),
    });
    this.emitChange();
  }

  private emitChange(): void {
    for (const listener of this.listeners) {
      listener();
    }
  }
}

export interface PlaylistSessionState<
  TLoaded,
  TSource extends PlaylistLoadSource<TLoaded> = PlaylistLoadSource<TLoaded>,
  TItem extends PlaylistItem<TSource> = PlaylistItem<TSource>,
> {
  ready: boolean;
  loading: boolean;
  error: string | null;
  itemIndex: number;
  itemCount: number;
  item: TItem | null;
  loaded: TLoaded | null;
  advanceMode: PlaylistAdvanceMode;
  endMode: PlaylistEndMode;
  playlistEnded: boolean;
}

export interface PlaylistSessionOptions<
  TLoaded,
  TSource extends PlaylistLoadSource<TLoaded> = PlaylistLoadSource<TLoaded>,
  TItem extends PlaylistItem<TSource> = PlaylistItem<TSource>,
> {
  initialItemIndex?: number;
  advanceMode?: PlaylistAdvanceMode;
  endMode?: PlaylistEndMode;
  advanceOnEnd?: boolean;
  preloadPolicy?: PlaylistPreloadPolicy<TSource, TItem>;
  preloadRadius?: number;
  loadCache?: PlaylistLoadCache<TLoaded, TSource>;
}

type PlaylistSessionListener<
  TLoaded,
  TSource extends PlaylistLoadSource<TLoaded>,
  TItem extends PlaylistItem<TSource>,
> = (state: PlaylistSessionState<TLoaded, TSource, TItem>) => void;

export class PlaylistSession<
  TLoaded,
  TSource extends PlaylistLoadSource<TLoaded> = PlaylistLoadSource<TLoaded>,
  TItem extends PlaylistItem<TSource> = PlaylistItem<TSource>,
> {
  readonly items: TItem[];
  readonly loadCache: PlaylistLoadCache<TLoaded, TSource>;

  private currentItemIndex = 0;
  private pendingItemIndex: number | null = null;
  private loading = false;
  private error: string | null = null;
  private currentLoaded: TLoaded | null = null;
  private disposed = false;
  private loadGeneration = 0;
  private pendingLoad: Promise<void> = Promise.resolve();
  private playlistEnded = false;
  private readonly listeners = new Set<PlaylistSessionListener<TLoaded, TSource, TItem>>();
  private readonly preloadPolicy: PlaylistPreloadPolicy<TSource, TItem>;
  private advanceMode: PlaylistAdvanceMode;
  private endMode: PlaylistEndMode;

  constructor(items: TItem[], options: PlaylistSessionOptions<TLoaded, TSource, TItem> = {}) {
    this.items = items;
    this.loadCache = options.loadCache ?? new PlaylistLoadCache<TLoaded, TSource>();
    this.preloadPolicy = normalizePreloadPolicy<TSource, TItem>(options);
    this.advanceMode = normalizeAdvanceMode(options);
    this.endMode = normalizeEndMode(options);

    if (items.length > 0) {
      this.currentItemIndex = clamp(options.initialItemIndex ?? 0, 0, items.length - 1);
      this.pendingLoad = this.loadItem(this.currentItemIndex);
    }
  }

  async waitForCurrentItem(): Promise<void> {
    await this.pendingLoad;
  }

  async setCurrentItemIndex(index: number): Promise<void> {
    this.pendingLoad = this.loadItem(index);
    await this.pendingLoad;
  }

  async next(): Promise<boolean> {
    const currentIndex = this.pendingItemIndex ?? this.currentItemIndex;
    if (currentIndex >= this.items.length - 1) {
      if (this.endMode === "loop" && this.items.length > 0) {
        await this.setCurrentItemIndex(0);
        return true;
      }
      return false;
    }

    await this.setCurrentItemIndex(currentIndex + 1);
    return true;
  }

  async previous(): Promise<boolean> {
    const currentIndex = this.pendingItemIndex ?? this.currentItemIndex;
    if (currentIndex <= 0) {
      if (this.endMode === "loop" && this.items.length > 0) {
        await this.setCurrentItemIndex(this.items.length - 1);
        return true;
      }
      return false;
    }

    await this.setCurrentItemIndex(currentIndex - 1);
    return true;
  }

  async completeCurrentItem(): Promise<boolean> {
    if (this.advanceMode !== "auto") {
      this.playlistEnded = this.currentItemIndex >= this.items.length - 1;
      this.emitChange();
      return false;
    }

    const advanced = await this.next();
    this.playlistEnded = !advanced && this.currentItemIndex >= this.items.length - 1;
    this.emitChange();
    return advanced;
  }

  setAdvanceMode(mode: PlaylistAdvanceMode): void {
    this.advanceMode = mode;
    this.emitChange();
  }

  setEndMode(mode: PlaylistEndMode): void {
    this.endMode = mode;
    this.playlistEnded = false;
    this.emitChange();
  }

  getCurrentLoaded(): TLoaded | null {
    return this.currentLoaded;
  }

  getState(): PlaylistSessionState<TLoaded, TSource, TItem> {
    const itemIndex = this.pendingItemIndex ?? this.currentItemIndex;
    return {
      ready: this.currentLoaded !== null && !this.loading && this.error === null,
      loading: this.loading,
      error: this.error,
      itemIndex,
      itemCount: this.items.length,
      item: this.items[itemIndex] ?? null,
      loaded: this.currentLoaded,
      advanceMode: this.advanceMode,
      endMode: this.endMode,
      playlistEnded: this.playlistEnded,
    };
  }

  subscribe(listener: PlaylistSessionListener<TLoaded, TSource, TItem>): () => void {
    this.listeners.add(listener);
    listener(this.getState());
    return () => {
      this.listeners.delete(listener);
    };
  }

  destroy(): void {
    this.disposed = true;
    this.listeners.clear();
  }

  dispose(): void {
    this.destroy();
  }

  private async loadItem(index: number): Promise<void> {
    if (this.items.length === 0) {
      return;
    }

    const clampedIndex = clamp(index, 0, this.items.length - 1);
    const generation = ++this.loadGeneration;
    const item = this.items[clampedIndex]!;

    this.pendingItemIndex = clampedIndex;
    this.loading = true;
    this.error = null;
    this.playlistEnded = false;
    this.emitChange();

    try {
      const loaded = await this.loadCache.load(item.replay);
      if (this.disposed || generation !== this.loadGeneration) {
        return;
      }

      this.currentItemIndex = clampedIndex;
      this.pendingItemIndex = null;
      this.currentLoaded = loaded;
      this.loading = false;
      this.error = null;
      this.loadCache.preload(resolvePolicySources(this.items, clampedIndex, this.preloadPolicy));
      this.emitChange();
    } catch (error) {
      if (this.disposed || generation !== this.loadGeneration) {
        return;
      }

      this.pendingItemIndex = null;
      this.loading = false;
      this.error = describeError(error);
      this.currentLoaded = null;
      this.emitChange();
      throw error;
    }
  }

  private emitChange(): void {
    const state = this.getState();
    for (const listener of this.listeners) {
      listener(state);
    }
  }
}
