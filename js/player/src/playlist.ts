import { ReplayPlayer } from "./player";
import { findFrameIndexAtTime } from "./replay-data";
import { loadReplayFromBytes } from "./wasm";
import type {
  LoadedReplay,
  PlaybackBound,
  PlaylistItem,
  ReplayCameraViewMode,
  ReplayFreeCameraPreset,
  ReplayPreloadContext,
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

type ReplayPlaylistPlayerListener = (
  state: ReplayPlaylistPlayerState
) => void;

type ReplayPathLoader = (path: string) => Promise<LoadedReplay>;

type PlayerPreferences = {
  speed: number;
  cameraDistanceScale: number;
  cameraViewMode: ReplayCameraViewMode;
  attachedPlayerId: string | null;
  ballCamEnabled: boolean;
  skipPostGoalTransitionsEnabled: boolean;
  skipKickoffsEnabled: boolean;
};

function isReplaySource(value: string | ReplaySource): value is ReplaySource {
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

function resolvePlaybackBound(
  replay: ReplayModel,
  bound: PlaybackBound
): ResolvedPlaybackBound {
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
  end: ResolvedPlaybackBound
): void {
  if (end.time < start.time) {
    const label = item.label ? ` "${item.label}"` : "";
    throw new Error(`Playlist item${label} ends before it starts`);
  }
}

function createInitialPreferences(
  options: ReplayPlaylistPlayerOptions
): PlayerPreferences {
  return {
    speed: Math.max(0.1, options.initialPlaybackRate ?? DEFAULT_PLAYBACK_RATE),
    cameraDistanceScale: Math.max(
      0.25,
      options.initialCameraDistanceScale ?? DEFAULT_CAMERA_DISTANCE_SCALE
    ),
    cameraViewMode: options.initialCameraViewMode ??
      (options.initialAttachedPlayerId ? "follow" : "free"),
    attachedPlayerId: options.initialAttachedPlayerId ?? null,
    ballCamEnabled: options.initialBallCamEnabled ?? false,
    skipPostGoalTransitionsEnabled:
      options.initialSkipPostGoalTransitionsEnabled ?? true,
    skipKickoffsEnabled: options.initialSkipKickoffsEnabled ?? false,
  };
}

function normalizePreloadPolicy(
  options: ReplayPlaylistPlayerOptions
): ReplayPreloadPolicy {
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

function uniqueSourcesFromItems(items: PlaylistItem[]): ReplaySource[] {
  const uniqueSources: ReplaySource[] = [];
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

function collectAdjacentSources(
  items: PlaylistItem[],
  currentIndex: number,
  direction: -1 | 1,
  limit: number,
  currentSourceId: string
): ReplaySource[] {
  const sources: ReplaySource[] = [];
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

function resolvePolicySources(
  items: PlaylistItem[],
  currentIndex: number,
  preloadPolicy: ReplayPreloadPolicy
): ReplaySource[] {
  const currentItem = items[currentIndex];
  if (!currentItem) {
    return [];
  }

  if (preloadPolicy.kind === "none") {
    return [];
  }

  if (preloadPolicy.kind === "all") {
    return uniqueSourcesFromItems(items).filter(
      (source) => source.id !== currentItem.replay.id
    );
  }

  if (preloadPolicy.kind === "adjacent") {
    const behind = collectAdjacentSources(
      items,
      currentIndex,
      -1,
      preloadPolicy.behind ?? 0,
      currentItem.replay.id
    );
    const ahead = collectAdjacentSources(
      items,
      currentIndex,
      1,
      preloadPolicy.ahead,
      currentItem.replay.id
    );
    return [...behind, ...ahead];
  }

  const preloadContext: ReplayPreloadContext = {
    items,
    currentIndex,
    currentItem,
  };
  const sourceIds = new Set<string>([currentItem.replay.id]);
  const sources: ReplaySource[] = [];
  const availableSources = new Map<string, ReplaySource>(
    uniqueSourcesFromItems(items).map((source) => [source.id, source])
  );

  for (const selected of preloadPolicy.pick(preloadContext)) {
    const source = isReplaySource(selected)
      ? selected
      : availableSources.get(selected);
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

export function createReplaySource(
  id: string,
  load: () => Promise<LoadedReplay>
): ReplaySource {
  return { id, load };
}

export function createStaticReplaySource(
  id: string,
  replay: LoadedReplay
): ReplaySource {
  return createReplaySource(id, async () => replay);
}

export function createReplayBytesSource(
  id: string,
  data: Uint8Array
): ReplaySource {
  return createReplaySource(id, async () => loadReplayFromBytes(data));
}

export function createReplayFileSource(
  file: File,
  id = file.webkitRelativePath || file.name
): ReplaySource {
  return createReplaySource(id, async () => {
    const bytes = new Uint8Array(await file.arrayBuffer());
    return loadReplayFromBytes(bytes);
  });
}

export function createReplayPathSource(
  path: string,
  loadReplay: ReplayPathLoader,
  id = path
): ReplaySource {
  return createReplaySource(id, async () => loadReplay(path));
}

export function resolvePlaylistItem(
  item: PlaylistItem,
  replay: LoadedReplay
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
  private readonly replayCache = new Map<string, Promise<LoadedReplay>>();
  private readonly preferences: PlayerPreferences;
  private readonly preloadPolicy: ReplayPreloadPolicy;

  constructor(
    container: HTMLElement,
    items: PlaylistItem[],
    options: ReplayPlaylistPlayerOptions = {}
  ) {
    super();
    this.container = container;
    this.items = items;
    this.options = options;
    this.preferences = createInitialPreferences(options);
    this.preloadPolicy = normalizePreloadPolicy(options);
    this.playbackIntent = options.autoplay ?? false;

    if (items.length > 0) {
      const initialIndex = clamp(
        options.initialItemIndex ?? 0,
        0,
        items.length - 1
      );
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
      return false;
    }

    await this.setCurrentItemIndex(currentIndex + 1);
    return true;
  }

  async previous(): Promise<boolean> {
    const currentIndex = this.pendingItemIndex ?? this.currentItemIndex;
    if (currentIndex <= 0) {
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
      this.currentResolvedItem.end.time
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

  getState(): ReplayPlaylistPlayerState {
    const playerState = this.player?.getState() ?? null;
    const itemIndex = this.pendingItemIndex ?? this.currentItemIndex;
    const item = this.items[itemIndex] ?? null;
    const replayCurrentTime = playerState?.currentTime ?? 0;
    const replayDuration =
      playerState?.duration ?? this.currentResolvedItem?.replay.replay.duration ?? 0;
    const itemStartTime = this.currentResolvedItem?.start.time ?? 0;
    const duration = this.currentResolvedItem?.duration ?? 0;

    return {
      ready: this.currentResolvedItem !== null && !this.loading && this.error === null,
      loading: this.loading,
      error: this.error,
      itemIndex,
      itemCount: this.items.length,
      item,
      currentTime: clamp(replayCurrentTime - itemStartTime, 0, duration),
      duration,
      replayCurrentTime,
      replayDuration,
      frameIndex: playerState?.frameIndex ?? this.currentResolvedItem?.start.frameIndex ?? 0,
      activeMetadata: playerState?.activeMetadata ?? null,
      playing: playerState?.playing ?? false,
      speed: playerState?.speed ?? this.preferences.speed,
      cameraDistanceScale:
        playerState?.cameraDistanceScale ?? this.preferences.cameraDistanceScale,
      cameraViewMode:
        playerState?.cameraViewMode ?? this.preferences.cameraViewMode,
      attachedPlayerId:
        playerState?.attachedPlayerId ?? this.preferences.attachedPlayerId,
      ballCamEnabled:
        playerState?.ballCamEnabled ?? this.preferences.ballCamEnabled,
      skipPostGoalTransitionsEnabled:
        playerState?.skipPostGoalTransitionsEnabled ??
        this.preferences.skipPostGoalTransitionsEnabled,
      skipKickoffsEnabled:
        playerState?.skipKickoffsEnabled ??
        this.preferences.skipKickoffsEnabled,
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
      const replay = await this.loadReplaySource(item.replay);
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
    const cachedReplay = this.replayCache.get(source.id);
    if (cachedReplay) {
      return cachedReplay;
    }

    const loadPromise = source.load().catch((error) => {
      this.replayCache.delete(source.id);
      throw error;
    });
    this.replayCache.set(source.id, loadPromise);
    return loadPromise;
  }

  private prefetchNearbyReplays(index: number): void {
    for (const source of resolvePolicySources(
      this.items,
      index,
      this.preloadPolicy
    )) {
      void this.loadReplaySource(source);
    }
  }

  private attachPlayer(resolvedItem: ResolvedPlaylistItem): void {
    this.detachPlayer();

    const replay = resolvedItem.replay.replay;
    const attachedPlayerId = replay.players.some(
      (player) => player.id === this.preferences.attachedPlayerId
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
      initialCameraViewMode: this.preferences.cameraViewMode,
      initialAttachedPlayerId: attachedPlayerId,
      initialBallCamEnabled: this.preferences.ballCamEnabled,
      initialSkipPostGoalTransitionsEnabled:
        this.preferences.skipPostGoalTransitionsEnabled,
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
    if (state.currentTime > endTime + END_TIME_EPSILON) {
      this.boundaryGuard = true;
      this.player?.setState({ currentTime: endTime, playing: false });
      this.playbackIntent = false;
      this.boundaryGuard = false;
      this.emitChange();
      return;
    }

    if (state.playing && state.currentTime >= endTime - END_TIME_EPSILON) {
      this.boundaryGuard = true;
      if (
        (this.options.advanceOnEnd ?? true) &&
        this.playbackIntent &&
        this.currentItemIndex < this.items.length - 1
      ) {
        void this.setCurrentItemIndex(this.currentItemIndex + 1);
      } else {
        this.playbackIntent = false;
        this.player?.setState({ currentTime: endTime, playing: false });
      }
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
