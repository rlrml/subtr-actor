import type {
  PlaylistLoadSource,
  PlaylistSourceLoadContext,
  PlaylistSourceLoadProgress,
  PlaylistSourceLoadState,
} from "./types";
import { describeError } from "./playlist-errors";

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
