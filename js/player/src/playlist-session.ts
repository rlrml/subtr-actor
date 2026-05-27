import type {
  PlaylistAdvanceMode,
  PlaylistEndMode,
  PlaylistItem,
  PlaylistLoadSource,
  PlaylistPreloadPolicy,
} from "./types";
import { describeError } from "./playlist-errors";
import { PlaylistLoadCache } from "./playlist-load-cache";
import {
  clamp,
  normalizeAdvanceMode,
  normalizeEndMode,
  normalizePreloadPolicy,
  resolvePolicySources,
} from "./playlist-policy";

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
