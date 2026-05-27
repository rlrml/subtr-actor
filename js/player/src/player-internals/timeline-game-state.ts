import type { ReplayModel } from "../types";

export function inferLiveGameState(replay: ReplayModel): number | null {
  if (replay.frames.length === 0) {
    return null;
  }

  const counts = new Map<number, number>();
  for (const frame of replay.frames) {
    counts.set(frame.gameState, (counts.get(frame.gameState) ?? 0) + 1);
  }

  let liveGameState: number | null = null;
  let liveGameStateCount = -1;
  for (const [gameState, count] of counts.entries()) {
    if (count <= liveGameStateCount) {
      continue;
    }

    liveGameState = gameState;
    liveGameStateCount = count;
  }

  return liveGameState;
}

export function inferKickoffGameState(
  replay: ReplayModel,
  liveGameState: number | null,
): number | null {
  if (liveGameState === null) {
    return null;
  }

  for (const frame of replay.frames) {
    if (frame.gameState === liveGameState) {
      break;
    }

    return frame.gameState;
  }

  return null;
}

export function isLiveGameplayFrame(
  frame: ReplayModel["frames"][number],
  liveGameState: number | null,
): boolean {
  if (liveGameState === null) {
    return frame.kickoffCountdown <= 0;
  }

  return frame.gameState === liveGameState;
}

export function isKickoffFrame(
  frame: ReplayModel["frames"][number],
  kickoffGameState: number | null,
): boolean {
  if (frame.kickoffCountdown > 0) {
    return true;
  }

  return kickoffGameState !== null && frame.gameState === kickoffGameState;
}

function hasRenderableSamples(replay: ReplayModel, frameIndex: number): boolean {
  if (replay.ballFrames[frameIndex]?.position) {
    return true;
  }

  return replay.players.some((player) => player.frames[frameIndex]?.position);
}

function isRenderableKickoffFrame(
  replay: ReplayModel,
  frame: ReplayModel["frames"][number],
  frameIndex: number,
  kickoffGameState: number | null,
): boolean {
  return isKickoffFrame(frame, kickoffGameState) && hasRenderableSamples(replay, frameIndex);
}

export function isPostGoalTransitionFrame(
  replay: ReplayModel,
  frame: ReplayModel["frames"][number],
  frameIndex: number,
  liveGameState: number | null,
  kickoffGameState: number | null,
): boolean {
  return (
    !isLiveGameplayFrame(frame, liveGameState) &&
    !isRenderableKickoffFrame(replay, frame, frameIndex, kickoffGameState)
  );
}
