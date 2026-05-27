import type { ReplayModel, ReplayPlayerKickoffCountdownMetadata } from "../types";

export function getKickoffCountdownMetadata(
  replay: ReplayModel,
  frameIndex: number,
  currentTime: number,
): ReplayPlayerKickoffCountdownMetadata | null {
  const currentFrame = replay.frames[frameIndex];
  if (!currentFrame || currentFrame.kickoffCountdown <= 0) {
    return null;
  }

  let startIndex = frameIndex;
  while (startIndex > 0 && (replay.frames[startIndex - 1]?.kickoffCountdown ?? 0) > 0) {
    startIndex -= 1;
  }

  let endIndex = frameIndex + 1;
  while (endIndex < replay.frames.length && replay.frames[endIndex].kickoffCountdown > 0) {
    endIndex += 1;
  }

  let maxCountdown = 0;
  for (let index = startIndex; index < endIndex; index += 1) {
    maxCountdown = Math.max(maxCountdown, replay.frames[index].kickoffCountdown);
  }

  const endsAt = replay.frames[endIndex]?.time ?? replay.duration;
  const secondsRemaining = Math.max(0, endsAt - currentTime);

  return {
    kind: "kickoff-countdown",
    countdown: Math.max(1, Math.min(maxCountdown, Math.ceil(secondsRemaining))),
    secondsRemaining,
    endsAt,
  };
}
