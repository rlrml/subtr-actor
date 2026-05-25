import { ReplayPlayer } from "./player";
import { findFrameIndexAtTime } from "./replay-data";
import { loadReplayFromBytes } from "./wasm";
import type {
  CameraSettings,
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
  ReplayCameraViewMode,
  ReplayFreeCameraPreset,
  ReplayPreloadPolicy,
  RawReplayFramesData,
  ReplayModel,
  ReplayPlaylistPlayerOptions,
  ReplayPlaylistPlayerSnapshot,
  ReplayPlaylistPlayerState,
  ReplaySource,
  ReplayPlayerState,
  ResolvedPlaybackBound,
  ResolvedPlaylistItem,
} from "./types";

const DEFAULT_CAMERA_DISTANCE_SCALE = 2.25;
const DEFAULT_PLAYBACK_RATE = 1;
const END_TIME_EPSILON = 0.0001;

type ReplayPlaylistPlayerListener = (state: ReplayPlaylistPlayerState) => void;

type ReplayPathLoader = (
  path: string,
  context?: PlaylistSourceLoadContext,
) => Promise<LoadedReplay>;
type ReplaySourceLoader = (context?: PlaylistSourceLoadContext) => Promise<LoadedReplay>;

export interface FullReplayPlaylistItemOptions {
  label?: string;
  meta?: Record<string, unknown>;
}

export interface ReplayPlaylistPlayerSingleReplayOptions extends ReplayPlaylistPlayerOptions {
  replayId?: string;
  itemLabel?: string;
  itemMeta?: Record<string, unknown>;
}

type PlayerPreferences = {
  speed: number;
  cameraDistanceScale: number;
  customCameraSettings: CameraSettings | null;
  cameraViewMode: ReplayCameraViewMode;
  attachedPlayerId: string | null;
  ballCamEnabled: boolean;
  boostPickupAnimationEnabled: boolean;
  skipPostGoalTransitionsEnabled: boolean;
  skipKickoffsEnabled: boolean;
};

function isPlaylistSource<TSource extends PlaylistLoadSource<unknown>>(
  value: string | TSource,
): value is TSource {
  return typeof value !== "string";
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

function clampFrameIndex(replay: ReplayModel, value: number): number {
  if (replay.frames.length === 0) {
    return 0;
  }

  const maxFrameIndex = replay.frames.length - 1;
  return clamp(Math.round(value), 0, maxFrameIndex);
}

function describeError(error: unknown): string {
  return error instanceof Error ? error.message : "Failed to load replay";
}

function finiteSetting(value: number | undefined): number | undefined {
  return typeof value === "number" && Number.isFinite(value) ? value : undefined;
}

function normalizeCustomCameraSettings(
  settings: CameraSettings | null | undefined,
): CameraSettings | null {
  if (!settings) {
    return null;
  }

  const normalized: CameraSettings = {};
  const fov = finiteSetting(settings.fov);
  const height = finiteSetting(settings.height);
  const pitch = finiteSetting(settings.pitch);
  const distance = finiteSetting(settings.distance);
  const stiffness = finiteSetting(settings.stiffness);
  const swivelSpeed = finiteSetting(settings.swivelSpeed);
  const transitionSpeed = finiteSetting(settings.transitionSpeed);
  if (fov !== undefined) normalized.fov = fov;
  if (height !== undefined) normalized.height = height;
  if (pitch !== undefined) normalized.pitch = pitch;
  if (distance !== undefined) normalized.distance = distance;
  if (stiffness !== undefined) normalized.stiffness = stiffness;
  if (swivelSpeed !== undefined) normalized.swivelSpeed = swivelSpeed;
  if (transitionSpeed !== undefined) {
    normalized.transitionSpeed = transitionSpeed;
  }
  return normalized;
}

function resolvePlaybackBound(replay: ReplayModel, bound: PlaybackBound): ResolvedPlaybackBound {
  if (bound.kind === "frame") {
    const frameIndex = clampFrameIndex(replay, bound.value);
    return {
      frameIndex,
      time: replay.frames[frameIndex]?.time ?? 0,
    };
  }

  const time = clamp(bound.value, 0, replay.duration);
  return {
    frameIndex: findFrameIndexAtTime(replay, time),
    time,
  };
}

function validateResolvedBounds(
  item: PlaylistItem,
  start: ResolvedPlaybackBound,
  end: ResolvedPlaybackBound,
): void {
  if (end.time < start.time) {
    const label = item.label ? ` "${item.label}"` : "";
    throw new Error(`Playlist item${label} ends before it starts`);
  }
}

function createInitialPreferences(options: ReplayPlaylistPlayerOptions): PlayerPreferences {
  return {
    speed: Math.max(0.1, options.initialPlaybackRate ?? DEFAULT_PLAYBACK_RATE),
    cameraDistanceScale: Math.max(
      0.25,
      options.initialCameraDistanceScale ?? DEFAULT_CAMERA_DISTANCE_SCALE,
    ),
    customCameraSettings: normalizeCustomCameraSettings(options.initialCustomCameraSettings),
    cameraViewMode:
      options.initialCameraViewMode ?? (options.initialAttachedPlayerId ? "follow" : "free"),
    attachedPlayerId: options.initialAttachedPlayerId ?? null,
    ballCamEnabled: options.initialBallCamEnabled ?? false,
    boostPickupAnimationEnabled: options.initialBoostPickupAnimationEnabled ?? true,
    skipPostGoalTransitionsEnabled: options.initialSkipPostGoalTransitionsEnabled ?? true,
    skipKickoffsEnabled: options.initialSkipKickoffsEnabled ?? false,
  };
}

function normalizePreloadPolicy<
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

function normalizeAdvanceMode(options: {
  advanceMode?: PlaylistAdvanceMode;
  advanceOnEnd?: boolean;
}): PlaylistAdvanceMode {
  if (options.advanceMode) {
    return options.advanceMode;
  }

  return options.advanceOnEnd === false ? "manual" : "auto";
}

function normalizeEndMode(options: { endMode?: PlaylistEndMode }): PlaylistEndMode {
  return options.endMode ?? "stop";
}

function uniqueSourcesFromItems<TSource extends PlaylistLoadSource<unknown>>(
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

function resolvePolicySources<
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

export function resolvePlaylistItem(
  item: PlaylistItem,
  replay: LoadedReplay,
): ResolvedPlaylistItem {
  const start = resolvePlaybackBound(replay.replay, item.start);
  const end = resolvePlaybackBound(replay.replay, item.end);
  validateResolvedBounds(item, start, end);

  return {
    source: item,
    replay,
    start,
    end,
    duration: Math.max(0, end.time - start.time),
  };
}

export class ReplayPlaylistPlayer extends EventTarget {
  readonly container: HTMLElement;
  readonly items: PlaylistItem[];
  readonly options: ReplayPlaylistPlayerOptions;

  private player: ReplayPlayer | null = null;
  private playerUnsubscribe: (() => void) | null = null;
  private currentResolvedItem: ResolvedPlaylistItem | null = null;
  private currentItemIndex = 0;
  private pendingItemIndex: number | null = null;
  private loading = false;
  private error: string | null = null;
  private disposed = false;
  private playbackIntent: boolean;
  private loadGeneration = 0;
  private boundaryGuard = false;
  private pendingLoad: Promise<void> = Promise.resolve();
  private readonly replayCache = new PlaylistLoadCache<LoadedReplay, ReplaySource>();
  private replayCacheUnsubscribe: (() => void) | null = null;
  private readonly preferences: PlayerPreferences;
  private readonly preloadPolicy: ReplayPreloadPolicy;
  private advanceMode: PlaylistAdvanceMode;
  private endMode: PlaylistEndMode;

  static fromReplay(
    container: HTMLElement,
    replay: LoadedReplay,
    options: ReplayPlaylistPlayerSingleReplayOptions = {},
  ): ReplayPlaylistPlayer {
    return ReplayPlaylistPlayer.fromReplaySource(
      container,
      createStaticReplaySource(options.replayId ?? "replay", replay),
      options,
    );
  }

  static fromReplaySource(
    container: HTMLElement,
    source: ReplaySource,
    options: ReplayPlaylistPlayerSingleReplayOptions = {},
  ): ReplayPlaylistPlayer {
    return new ReplayPlaylistPlayer(
      container,
      [
        createFullReplayPlaylistItem(source, {
          label: options.itemLabel,
          meta: options.itemMeta,
        }),
      ],
      options,
    );
  }

  constructor(
    container: HTMLElement,
    items: PlaylistItem[],
    options: ReplayPlaylistPlayerOptions = {},
  ) {
    super();
    this.container = container;
    this.items = items;
    this.options = options;
    this.preferences = createInitialPreferences(options);
    this.preloadPolicy = normalizePreloadPolicy(options);
    this.advanceMode = normalizeAdvanceMode(options);
    this.endMode = normalizeEndMode(options);
    this.playbackIntent = options.autoplay ?? false;
    this.replayCacheUnsubscribe = this.replayCache.subscribe(() => {
      this.emitChange();
    });

    if (items.length > 0) {
      const initialIndex = clamp(options.initialItemIndex ?? 0, 0, items.length - 1);
      this.pendingLoad = this.loadItem(initialIndex);
      return;
    }

    this.emitChange();
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

  play(): void {
    this.playbackIntent = true;
    this.player?.play();
    this.emitChange();
  }

  pause(): void {
    this.playbackIntent = false;
    this.player?.pause();
    this.emitChange();
  }

  togglePlayback(): void {
    if (this.player?.getState().playing) {
      this.pause();
    } else {
      this.play();
    }
  }

  seek(time: number): void {
    if (!this.player || !this.currentResolvedItem) {
      return;
    }

    const targetTime = clamp(
      this.currentResolvedItem.start.time + time,
      this.currentResolvedItem.start.time,
      this.currentResolvedItem.end.time,
    );
    this.player.seek(targetTime);
  }

  setReplayFrameIndex(frameIndex: number): boolean {
    if (!this.player) {
      return false;
    }

    this.playbackIntent = false;
    this.player.setFrameIndex(frameIndex);
    this.emitChange();
    return true;
  }

  stepFrames(delta: number): boolean {
    if (!this.player || !Number.isFinite(delta)) {
      return false;
    }

    this.playbackIntent = false;
    this.player.stepFrames(delta);
    this.emitChange();
    return true;
  }

  stepForwardFrame(): boolean {
    return this.stepFrames(1);
  }

  stepBackwardFrame(): boolean {
    return this.stepFrames(-1);
  }

  setPlaybackRate(speed: number): void {
    this.preferences.speed = Math.max(0.1, speed);
    this.player?.setPlaybackRate(this.preferences.speed);
    this.emitChange();
  }

  setCameraDistanceScale(scale: number): void {
    this.preferences.cameraDistanceScale = Math.max(0.25, scale);
    this.player?.setCameraDistanceScale(this.preferences.cameraDistanceScale);
    this.emitChange();
  }

  setCustomCameraSettings(settings: CameraSettings | null): void {
    this.preferences.customCameraSettings = normalizeCustomCameraSettings(settings);
    this.player?.setCustomCameraSettings(this.preferences.customCameraSettings);
    this.emitChange();
  }

  setCameraViewMode(mode: ReplayCameraViewMode): void {
    this.preferences.cameraViewMode = mode;
    this.player?.setCameraViewMode(mode);
    this.emitChange();
  }

  setFreeCameraPreset(preset: ReplayFreeCameraPreset): void {
    this.preferences.cameraViewMode = "free";
    this.player?.setFreeCameraPreset(preset);
    this.emitChange();
  }

  setAttachedPlayer(playerId: string | null): void {
    this.preferences.attachedPlayerId = playerId;
    this.preferences.cameraViewMode = playerId ? "follow" : "free";
    this.player?.setAttachedPlayer(playerId);
    this.emitChange();
  }

  setBallCamEnabled(enabled: boolean): void {
    this.preferences.ballCamEnabled = enabled;
    this.player?.setBallCamEnabled(enabled);
    this.emitChange();
  }

  setBoostPickupAnimationEnabled(enabled: boolean): void {
    this.preferences.boostPickupAnimationEnabled = enabled;
    this.player?.setBoostPickupAnimationEnabled(enabled);
    this.emitChange();
  }

  setSkipPostGoalTransitionsEnabled(enabled: boolean): void {
    this.preferences.skipPostGoalTransitionsEnabled = enabled;
    this.player?.setSkipPostGoalTransitionsEnabled(enabled);
    this.emitChange();
  }

  setSkipKickoffsEnabled(enabled: boolean): void {
    this.preferences.skipKickoffsEnabled = enabled;
    this.player?.setSkipKickoffsEnabled(enabled);
    this.emitChange();
  }

  setAdvanceMode(mode: PlaylistAdvanceMode): void {
    this.advanceMode = mode;
    this.emitChange();
  }

  setEndMode(mode: PlaylistEndMode): void {
    this.endMode = mode;
    this.emitChange();
  }

  getState(): ReplayPlaylistPlayerState {
    const playerState = this.player?.getState() ?? null;
    const itemIndex = this.pendingItemIndex ?? this.currentItemIndex;
    const item = this.items[itemIndex] ?? null;
    const replayCurrentTime = playerState?.currentTime ?? 0;
    const replayDuration =
      playerState?.duration ?? this.currentResolvedItem?.replay.replay.duration ?? 0;
    const itemStartTime = this.currentResolvedItem?.start.time ?? 0;
    const duration = this.currentResolvedItem?.duration ?? 0;
    const currentTime = clamp(replayCurrentTime - itemStartTime, 0, duration);
    const itemEnded =
      this.currentResolvedItem !== null && currentTime >= duration - END_TIME_EPSILON;

    return {
      ready: this.currentResolvedItem !== null && !this.loading && this.error === null,
      loading: this.loading,
      error: this.error,
      replayLoadStates: this.getReplayLoadStates(),
      itemIndex,
      itemCount: this.items.length,
      item,
      advanceMode: this.advanceMode,
      endMode: this.endMode,
      itemEnded,
      playlistEnded: itemEnded && itemIndex >= this.items.length - 1,
      currentTime,
      duration,
      replayCurrentTime,
      replayDuration,
      frameIndex: playerState?.frameIndex ?? this.currentResolvedItem?.start.frameIndex ?? 0,
      activeMetadata: playerState?.activeMetadata ?? null,
      playing: playerState?.playing ?? false,
      speed: playerState?.speed ?? this.preferences.speed,
      cameraDistanceScale: playerState?.cameraDistanceScale ?? this.preferences.cameraDistanceScale,
      customCameraSettings:
        playerState?.customCameraSettings ?? this.preferences.customCameraSettings,
      cameraViewMode: playerState?.cameraViewMode ?? this.preferences.cameraViewMode,
      attachedPlayerId: playerState?.attachedPlayerId ?? this.preferences.attachedPlayerId,
      ballCamEnabled: playerState?.ballCamEnabled ?? this.preferences.ballCamEnabled,
      boostPickupAnimationEnabled:
        playerState?.boostPickupAnimationEnabled ?? this.preferences.boostPickupAnimationEnabled,
      skipPostGoalTransitionsEnabled:
        playerState?.skipPostGoalTransitionsEnabled ??
        this.preferences.skipPostGoalTransitionsEnabled,
      skipKickoffsEnabled: playerState?.skipKickoffsEnabled ?? this.preferences.skipKickoffsEnabled,
    };
  }

  getSnapshot(): ReplayPlaylistPlayerSnapshot {
    return this.getState();
  }

  getCurrentReplay(): LoadedReplay | null {
    return this.currentResolvedItem?.replay ?? null;
  }

  getCurrentPlayer(): ReplayPlayer | null {
    return this.player;
  }

  getCurrentResolvedItem(): ResolvedPlaylistItem | null {
    return this.currentResolvedItem;
  }

  subscribe(listener: ReplayPlaylistPlayerListener): () => void {
    const handleChange = (event: Event): void => {
      listener((event as CustomEvent<ReplayPlaylistPlayerState>).detail);
    };
    this.addEventListener("change", handleChange);
    listener(this.getState());
    return () => {
      this.removeEventListener("change", handleChange);
    };
  }

  destroy(): void {
    this.disposed = true;
    this.replayCacheUnsubscribe?.();
    this.replayCacheUnsubscribe = null;
    this.detachPlayer();
    this.replayCache.clear();
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
    const item = this.items[clampedIndex];

    this.pendingItemIndex = clampedIndex;
    this.loading = true;
    this.error = null;
    this.emitChange();

    try {
      const replayPromise = this.loadReplaySource(item.replay);
      this.prefetchNearbyReplays(clampedIndex);
      const replay = await replayPromise;
      if (this.disposed || generation !== this.loadGeneration) {
        return;
      }

      const resolvedItem = resolvePlaylistItem(item, replay);
      this.currentItemIndex = clampedIndex;
      this.pendingItemIndex = null;
      this.currentResolvedItem = resolvedItem;
      this.attachPlayer(resolvedItem);
      this.loading = false;
      this.error = null;
      this.prefetchNearbyReplays(clampedIndex);
      this.emitChange();
    } catch (error) {
      if (this.disposed || generation !== this.loadGeneration) {
        return;
      }

      this.playbackIntent = false;
      this.pendingItemIndex = null;
      this.loading = false;
      this.error = describeError(error);
      this.detachPlayer();
      this.currentResolvedItem = null;
      this.emitChange();
      throw error;
    }
  }

  private loadReplaySource(source: ReplaySource): Promise<LoadedReplay> {
    return this.replayCache.load(source);
  }

  private prefetchNearbyReplays(index: number): void {
    this.replayCache.preload(resolvePolicySources(this.items, index, this.preloadPolicy));
  }

  private getReplayLoadStates(): PlaylistSourceLoadState[] {
    return uniqueSourcesFromItems(this.items).map((source) => this.replayCache.getState(source));
  }

  private attachPlayer(resolvedItem: ResolvedPlaylistItem): void {
    this.detachPlayer();

    const replay = resolvedItem.replay.replay;
    const attachedPlayerId = replay.players.some(
      (player) => player.id === this.preferences.attachedPlayerId,
    )
      ? this.preferences.attachedPlayerId
      : null;
    this.preferences.attachedPlayerId = attachedPlayerId;
    if (attachedPlayerId === null && this.preferences.cameraViewMode === "follow") {
      this.preferences.cameraViewMode = "free";
    }

    this.player = new ReplayPlayer(this.container, replay, {
      fieldScale: this.options.fieldScale,
      initialPlaybackRate: this.preferences.speed,
      initialCameraDistanceScale: this.preferences.cameraDistanceScale,
      initialCustomCameraSettings: this.preferences.customCameraSettings,
      initialCameraViewMode: this.preferences.cameraViewMode,
      initialAttachedPlayerId: attachedPlayerId,
      initialBallCamEnabled: this.preferences.ballCamEnabled,
      initialBoostPickupAnimationEnabled: this.preferences.boostPickupAnimationEnabled,
      initialSkipPostGoalTransitionsEnabled: this.preferences.skipPostGoalTransitionsEnabled,
      initialSkipKickoffsEnabled: this.preferences.skipKickoffsEnabled,
      plugins: this.options.plugins,
    });
    this.player.seek(resolvedItem.start.time);
    this.playerUnsubscribe = this.player.subscribe((state) => {
      this.handlePlayerState(state);
    });

    if (this.playbackIntent) {
      this.player.play();
    }
  }

  private detachPlayer(): void {
    this.playerUnsubscribe?.();
    this.playerUnsubscribe = null;
    this.player?.destroy();
    this.player = null;
  }

  private handlePlayerState(state: ReplayPlayerState): void {
    if (!this.currentResolvedItem || this.boundaryGuard) {
      this.emitChange();
      return;
    }

    const endTime = this.currentResolvedItem.end.time;
    if (state.playing && state.currentTime >= endTime - END_TIME_EPSILON) {
      this.boundaryGuard = true;
      if (this.advanceMode === "auto" && this.playbackIntent) {
        if (this.currentItemIndex < this.items.length - 1) {
          void this.setCurrentItemIndex(this.currentItemIndex + 1);
        } else if (this.endMode === "loop" && this.items.length > 0) {
          void this.setCurrentItemIndex(0);
        } else {
          this.playbackIntent = false;
          this.player?.setState({ currentTime: endTime, playing: false });
        }
      } else {
        this.playbackIntent = false;
        this.player?.setState({ currentTime: endTime, playing: false });
      }
      this.boundaryGuard = false;
      this.emitChange();
      return;
    }

    if (state.currentTime > endTime + END_TIME_EPSILON) {
      this.boundaryGuard = true;
      this.player?.setState({ currentTime: endTime, playing: false });
      this.playbackIntent = false;
      this.boundaryGuard = false;
      this.emitChange();
      return;
    }

    this.emitChange();
  }

  private emitChange(): void {
    this.dispatchEvent(new CustomEvent("change", { detail: this.getState() }));
  }
}

export type { RawReplayFramesData };
