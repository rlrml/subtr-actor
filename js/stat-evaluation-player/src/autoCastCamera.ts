import { statsEventPayloads, type StatsTimeline } from "./statsTimeline.ts";

const DEFAULT_PRE_ROLL_SECONDS = 0.8;
const DEFAULT_MIN_POSSESSION_SECONDS = 0.45;
const DEFAULT_SAME_PLAYER_BRIDGE_SECONDS = 0.5;
const DEFAULT_FRAME_RATE = 30;

export interface AutoCastCameraOptions {
  readonly preRollSeconds?: number;
  readonly minPossessionSeconds?: number;
  readonly samePlayerBridgeSeconds?: number;
}

export interface AutoCastCameraSpan {
  readonly playerId: string;
  readonly startFrame: number;
  readonly endFrame: number;
  readonly possessionStartFrame: number;
  readonly possessionEndFrame: number;
}

interface PossessionCandidate {
  readonly playerId: string;
  readonly startFrame: number;
  readonly endFrame: number;
  readonly startTime: number;
  readonly endTime: number;
  readonly duration: number;
  readonly sustainedControl: boolean;
}

function remoteIdKey(playerId: unknown): string {
  if (!playerId || typeof playerId !== "object") {
    return String(playerId);
  }
  const [kind, value] = Object.entries(playerId as Record<string, unknown>)[0] ?? [
    "Unknown",
    "unknown",
  ];
  return `${kind}:${typeof value === "string" ? value : JSON.stringify(value)}`;
}

function finiteOrZero(value: number): number {
  return Number.isFinite(value) ? value : 0;
}

function inferFrameRate(
  statsTimeline: StatsTimeline,
  possessions: readonly PossessionCandidate[],
): number {
  for (const possession of possessions) {
    const frameDelta = possession.endFrame - possession.startFrame;
    const timeDelta = possession.endTime - possession.startTime;
    if (frameDelta > 0 && timeDelta > 0) {
      return frameDelta / timeDelta;
    }
  }

  const firstFrame = statsTimeline.frames[0];
  const lastFrame = statsTimeline.frames.at(-1);
  if (firstFrame && lastFrame) {
    const frameDelta = lastFrame.frame_number - firstFrame.frame_number;
    const timeDelta = lastFrame.time - firstFrame.time;
    if (frameDelta > 0 && timeDelta > 0) {
      return frameDelta / timeDelta;
    }
  }

  return DEFAULT_FRAME_RATE;
}

export function buildAutoCastCameraSpans(
  statsTimeline: StatsTimeline,
  options: AutoCastCameraOptions = {},
): AutoCastCameraSpan[] {
  const preRollSeconds = options.preRollSeconds ?? DEFAULT_PRE_ROLL_SECONDS;
  const minPossessionSeconds = options.minPossessionSeconds ?? DEFAULT_MIN_POSSESSION_SECONDS;
  const samePlayerBridgeSeconds =
    options.samePlayerBridgeSeconds ?? DEFAULT_SAME_PLAYER_BRIDGE_SECONDS;

  const possessions: PossessionCandidate[] = statsEventPayloads(statsTimeline, "player_possession")
    .map((event) => ({
      playerId: remoteIdKey(event.player_id),
      startFrame: Math.max(0, Math.trunc(finiteOrZero(event.start_frame))),
      endFrame: Math.max(0, Math.trunc(finiteOrZero(event.end_frame))),
      startTime: finiteOrZero(event.start_time),
      endTime: finiteOrZero(event.end_time),
      duration: finiteOrZero(event.duration),
      sustainedControl: event.sustained_control,
    }))
    .filter((event) => {
      return (
        event.endFrame > event.startFrame &&
        (event.sustainedControl || event.duration >= minPossessionSeconds)
      );
    })
    .sort((left, right) => left.startFrame - right.startFrame || left.endFrame - right.endFrame);

  const frameRate = inferFrameRate(statsTimeline, possessions);
  const preRollFrames = Math.max(0, Math.round(preRollSeconds * frameRate));
  const samePlayerBridgeFrames = Math.max(0, Math.round(samePlayerBridgeSeconds * frameRate));
  const spans: AutoCastCameraSpan[] = [];
  let previousPossession: PossessionCandidate | null = null;

  for (const possession of possessions) {
    let startFrame = Math.max(0, possession.startFrame - preRollFrames);
    if (previousPossession && previousPossession.playerId !== possession.playerId) {
      startFrame = Math.max(startFrame, previousPossession.endFrame);
    }

    const previousSpan = spans.at(-1);
    if (
      previousSpan &&
      previousSpan.playerId === possession.playerId &&
      startFrame <= previousSpan.endFrame + samePlayerBridgeFrames
    ) {
      spans[spans.length - 1] = {
        ...previousSpan,
        endFrame: Math.max(previousSpan.endFrame, possession.endFrame),
        possessionEndFrame: Math.max(previousSpan.possessionEndFrame, possession.endFrame),
      };
      previousPossession = possession;
      continue;
    }

    spans.push({
      playerId: possession.playerId,
      startFrame,
      endFrame: possession.endFrame,
      possessionStartFrame: possession.startFrame,
      possessionEndFrame: possession.endFrame,
    });
    previousPossession = possession;
  }

  return spans.map((span, index) => ({
    ...span,
    endFrame: spans[index + 1]?.startFrame ?? Number.POSITIVE_INFINITY,
  }));
}

export function autoCastPlayerForState(
  spans: readonly AutoCastCameraSpan[],
  state: { frameIndex: number },
): string | null {
  const frameIndex = Math.max(0, Math.trunc(state.frameIndex));
  const span = spans.find((candidate) => {
    return frameIndex >= candidate.startFrame && frameIndex < candidate.endFrame;
  });
  return span?.playerId ?? null;
}
