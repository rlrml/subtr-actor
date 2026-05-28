import type {
  PlaylistAdvanceMode,
  PlaylistEndMode,
  PlaylistItem,
  PlaylistSourceLoadState,
  ReplayPlayerState,
  ReplayPlaylistPlayerState,
  ResolvedPlaylistItem,
} from "./types";
import { END_TIME_EPSILON } from "./playlist-item-resolution";
import { clamp } from "./playlist-policy";
import type { PlayerPreferences } from "./playlist-preferences";

interface ReplayPlaylistPlayerStateOptions {
  readonly advanceMode: PlaylistAdvanceMode;
  readonly currentItemIndex: number;
  readonly currentResolvedItem: ResolvedPlaylistItem | null;
  readonly endMode: PlaylistEndMode;
  readonly error: string | null;
  readonly items: readonly PlaylistItem[];
  readonly loading: boolean;
  readonly pendingItemIndex: number | null;
  readonly playerState: ReplayPlayerState | null;
  readonly preferences: PlayerPreferences;
  readonly replayLoadStates: PlaylistSourceLoadState[];
}

export function createReplayPlaylistPlayerState({
  advanceMode,
  currentItemIndex,
  currentResolvedItem,
  endMode,
  error,
  items,
  loading,
  pendingItemIndex,
  playerState,
  preferences,
  replayLoadStates,
}: ReplayPlaylistPlayerStateOptions): ReplayPlaylistPlayerState {
  const itemIndex = pendingItemIndex ?? currentItemIndex;
  const item = items[itemIndex] ?? null;
  const replayCurrentTime = playerState?.currentTime ?? 0;
  const replayDuration = playerState?.duration ?? currentResolvedItem?.replay.replay.duration ?? 0;
  const itemStartTime = currentResolvedItem?.start.time ?? 0;
  const duration = currentResolvedItem?.duration ?? 0;
  const currentTime = clamp(replayCurrentTime - itemStartTime, 0, duration);
  const itemEnded = currentResolvedItem !== null && currentTime >= duration - END_TIME_EPSILON;

  return {
    ready: currentResolvedItem !== null && !loading && error === null,
    loading,
    error,
    replayLoadStates,
    itemIndex,
    itemCount: items.length,
    item,
    advanceMode,
    endMode,
    itemEnded,
    playlistEnded: itemEnded && itemIndex >= items.length - 1,
    currentTime,
    duration,
    replayCurrentTime,
    replayDuration,
    frameIndex: playerState?.frameIndex ?? currentResolvedItem?.start.frameIndex ?? 0,
    activeMetadata: playerState?.activeMetadata ?? null,
    playing: playerState?.playing ?? false,
    speed: playerState?.speed ?? preferences.speed,
    cameraDistanceScale: playerState?.cameraDistanceScale ?? preferences.cameraDistanceScale,
    customCameraSettings: playerState?.customCameraSettings ?? preferences.customCameraSettings,
    cameraViewMode: playerState?.cameraViewMode ?? preferences.cameraViewMode,
    attachedPlayerId: playerState?.attachedPlayerId ?? preferences.attachedPlayerId,
    ballCamEnabled: playerState?.ballCamEnabled ?? preferences.ballCamEnabled,
    boostPickupAnimationEnabled:
      playerState?.boostPickupAnimationEnabled ?? preferences.boostPickupAnimationEnabled,
    skipPostGoalTransitionsEnabled:
      playerState?.skipPostGoalTransitionsEnabled ?? preferences.skipPostGoalTransitionsEnabled,
    skipKickoffsEnabled: playerState?.skipKickoffsEnabled ?? preferences.skipKickoffsEnabled,
  };
}
