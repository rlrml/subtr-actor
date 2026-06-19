import { ReplayPlayer, createPlayerFromParsed } from "./player/lib";
import { findFrameIndexAtTime } from "./replay-data";
import type {
  CameraSettings,
  LoadedReplay,
  PlaybackBound,
  PlaylistAdvanceMode,
  PlaylistEndMode,
  PlaylistItem,
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
import {
  PlaylistLoadCache,
  createFullReplayPlaylistItem,
  createStaticReplaySource,
  normalizeAdvanceMode,
  normalizeEndMode,
  normalizePreloadPolicy,
  resolvePolicySources,
  uniqueSourcesFromItems,
} from "./playlist-core";
export {
  PlaylistLoadCache,
  PlaylistSession,
  createFullReplayPlaylistItem,
  createReplayBytesSource,
  createReplayFileSource,
  createReplayPathSource,
  createReplaySource,
  createStaticReplaySource,
  frameBound,
  timeBound,
} from "./playlist-core";
export type {
  FullReplayPlaylistItemOptions,
  PlaylistSessionOptions,
  PlaylistSessionState,
} from "./playlist-core";

const DEFAULT_CAMERA_DISTANCE_SCALE = 2.25;
const DEFAULT_PLAYBACK_RATE = 1;
const END_TIME_EPSILON = 0.0001;

type ReplayPlaylistPlayerListener = (state: ReplayPlaylistPlayerState) => void;

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
  hitboxWireframesEnabled: boolean;
  hitboxOnlyModeEnabled: boolean;
  skipPostGoalTransitionsEnabled: boolean;
  skipKickoffsEnabled: boolean;
};

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
    hitboxWireframesEnabled: options.initialHitboxWireframesEnabled ?? false,
    hitboxOnlyModeEnabled: options.initialHitboxOnlyModeEnabled ?? false,
    skipPostGoalTransitionsEnabled: options.initialSkipPostGoalTransitionsEnabled ?? true,
    skipKickoffsEnabled: options.initialSkipKickoffsEnabled ?? false,
  };
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

  setHitboxWireframesEnabled(enabled: boolean): void {
    this.preferences.hitboxWireframesEnabled = enabled;
    this.player?.setHitboxWireframesEnabled(enabled);
    this.emitChange();
  }

  setHitboxOnlyModeEnabled(enabled: boolean): void {
    this.preferences.hitboxOnlyModeEnabled = enabled;
    this.player?.setHitboxOnlyModeEnabled(enabled);
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
      hitboxWireframesEnabled:
        playerState?.hitboxWireframesEnabled ?? this.preferences.hitboxWireframesEnabled,
      hitboxOnlyModeEnabled:
        playerState?.hitboxOnlyModeEnabled ?? this.preferences.hitboxOnlyModeEnabled,
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
      const attached = await this.attachPlayer(resolvedItem, generation);
      if (!attached || this.disposed || generation !== this.loadGeneration) {
        return;
      }
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

  private async attachPlayer(
    resolvedItem: ResolvedPlaylistItem,
    generation: number,
  ): Promise<boolean> {
    const loadedReplay = resolvedItem.replay;
    const { replay, raw } = loadedReplay;
    if (!raw) {
      throw new Error(
        "ReplayPlaylistPlayer requires LoadedReplay.raw; load replays with loadReplayFromBytes, createReplayBytesSource, or createReplayFileSource.",
      );
    }
    const attachedPlayerId = replay.players.some(
      (player) => player.id === this.preferences.attachedPlayerId,
    )
      ? this.preferences.attachedPlayerId
      : null;
    this.preferences.attachedPlayerId = attachedPlayerId;
    if (attachedPlayerId === null && this.preferences.cameraViewMode === "follow") {
      this.preferences.cameraViewMode = "free";
    }

    const player = createPlayerFromParsed(
      this.container,
      { replay, raw },
      {
        initialPlaybackRate: this.preferences.speed,
        initialCameraDistanceScale: this.preferences.cameraDistanceScale,
        initialCustomCameraSettings: this.preferences.customCameraSettings,
        initialCameraViewMode: this.preferences.cameraViewMode,
        initialAttachedPlayerId: attachedPlayerId,
        initialBallCamEnabled: this.preferences.ballCamEnabled,
        initialBoostPickupAnimationEnabled: this.preferences.boostPickupAnimationEnabled,
        initialHitboxWireframesEnabled: this.preferences.hitboxWireframesEnabled,
        initialHitboxOnlyModeEnabled: this.preferences.hitboxOnlyModeEnabled,
        initialSkipPostGoalTransitionsEnabled: this.preferences.skipPostGoalTransitionsEnabled,
        initialSkipKickoffsEnabled: this.preferences.skipKickoffsEnabled,
        plugins: this.options.plugins,
      },
    );
    const { style } = player.renderer.domElement;
    style.visibility = "hidden";
    style.pointerEvents = "none";
    player.seek(resolvedItem.start.time);

    try {
      await player.ready;
    } catch (error) {
      player.destroy();
      throw error;
    }
    if (this.disposed || generation !== this.loadGeneration) {
      player.destroy();
      return false;
    }

    this.detachPlayer();
    style.visibility = "";
    style.pointerEvents = "";
    this.player = player;
    this.playerUnsubscribe = player.subscribe((state) => {
      this.handlePlayerState(state);
    });

    if (this.playbackIntent) {
      player.play();
    }
    return true;
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
