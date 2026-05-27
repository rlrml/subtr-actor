import type { ReplayModel } from "@rlrml/subtr-actor-player";

export const BLUE_TIMELINE_COLOR = "#3b82f6";
export const ORANGE_TIMELINE_COLOR = "#f59e0b";
export const NEUTRAL_TIMELINE_COLOR = "#d1d9e0";

export function getReplayPlayerName(replay: ReplayModel, playerId: string): string {
  return replay.players.find((player) => player.id === playerId)?.name ?? playerId;
}

export function getReplayFrameTime(
  replay: ReplayModel,
  frame: number | undefined,
  fallbackTime: number,
): number {
  return replay.frames[frame ?? -1]?.time ?? fallbackTime;
}
