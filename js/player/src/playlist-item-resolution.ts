import { findFrameIndexAtTime } from "./replay-data";
import type {
  LoadedReplay,
  PlaybackBound,
  PlaylistItem,
  ReplayModel,
  ResolvedPlaybackBound,
  ResolvedPlaylistItem,
} from "./types";
import { clamp } from "./playlist-policy";

export const END_TIME_EPSILON = 0.0001;

function clampFrameIndex(replay: ReplayModel, value: number): number {
  if (replay.frames.length === 0) {
    return 0;
  }

  const maxFrameIndex = replay.frames.length - 1;
  return clamp(Math.round(value), 0, maxFrameIndex);
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
