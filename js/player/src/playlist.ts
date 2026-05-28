import { ReplayPlayer } from "./player";
import type {
  CameraSettings,
  LoadedReplay,
  PlaylistAdvanceMode,
  PlaylistEndMode,
  PlaylistItem,
  PlaylistSourceLoadState,
  ReplayCameraViewMode,
  ReplayFreeCameraPreset,
  ReplayPreloadPolicy,
  ReplayPlaylistPlayerOptions,
  ReplayPlaylistPlayerSnapshot,
  ReplayPlaylistPlayerState,
  ReplaySource,
  ReplayPlayerState,
  ResolvedPlaylistItem,
} from "./types";
import { createFullReplayPlaylistItem, createStaticReplaySource } from "./playlist-sources";
import { describeError } from "./playlist-errors";
import { PlaylistLoadCache } from "./playlist-load-cache";
import {
  clamp,
  normalizeAdvanceMode,
  normalizeEndMode,
  normalizePreloadPolicy,
  resolvePolicySources,
  uniqueSourcesFromItems,
} from "./playlist-policy";
import { END_TIME_EPSILON, resolvePlaylistItem } from "./playlist-item-resolution";
import { createReplayPlaylistPlayerState } from "./playlist-state";
import {
  createInitialPreferences,
  normalizeCustomCameraSettings,
  type PlayerPreferences,
} from "./playlist-preferences";

export { PlaylistLoadCache } from "./playlist-load-cache";
export { PlaylistSession } from "./playlist-session";
export type { PlaylistSessionOptions, PlaylistSessionState } from "./playlist-session";
export {
  createFullReplayPlaylistItem,
  createReplayBytesSource,
  createReplayFileSource,
  createReplayPathSource,
  createReplaySource,
  createStaticReplaySource,
  frameBound,
  timeBound,
} from "./playlist-sources";
export type { FullReplayPlaylistItemOptions } from "./playlist-sources";
export { resolvePlaylistItem } from "./playlist-item-resolution";

type ReplayPlaylistPlayerListener = (state: ReplayPlaylistPlayerState) => void;

export interface ReplayPlaylistPlayerSingleReplayOptions extends ReplayPlaylistPlayerOptions {
  replayId?: string;
  itemLabel?: string;
  itemMeta?: Record<string, unknown>;
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
    return createReplayPlaylistPlayerState({
      advanceMode: this.advanceMode,
      currentItemIndex: this.currentItemIndex,
      currentResolvedItem: this.currentResolvedItem,
      endMode: this.endMode,
      error: this.error,
      items: this.items,
      loading: this.loading,
      pendingItemIndex: this.pendingItemIndex,
      playerState: this.player?.getState() ?? null,
      preferences: this.preferences,
      replayLoadStates: this.getReplayLoadStates(),
    });
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

export type { RawReplayFramesData } from "./types";
