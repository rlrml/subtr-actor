import type { FrameRenderInfo } from "@rlrml/player";
import type { CameraControlsController } from "./cameraControls.ts";
import { statsEventPayloads, type StatsTimeline } from "./statsTimeline.ts";
import type { StatsReplayPlayer } from "./statsReplayPlayer.ts";
import { playerIdToString } from "./touchOverlay.ts";

const DEFAULT_PRE_ROLL_SECONDS = 0.8;
const DEFAULT_MIN_POSSESSION_SECONDS = 0.45;
const DEFAULT_SAME_PLAYER_BRIDGE_SECONDS = 0.5;
const DEFAULT_FRAME_RATE = 30;

export interface AutoPossessionCameraControllerOptions {
  getReplayPlayer(): StatsReplayPlayer | null;
  getStatsTimeline(): StatsTimeline | null;
  getCameraControlsController(): CameraControlsController | null;
}

export interface AutoPossessionCameraOptions {
  readonly preRollSeconds?: number;
  readonly minPossessionSeconds?: number;
  readonly samePlayerBridgeSeconds?: number;
}

export interface AutoPossessionSpan {
  playerId: string;
  startFrame: number;
  endFrame: number;
  possessionStartFrame: number;
  possessionEndFrame: number;
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

export function buildAutoPossessionCameraSpans(
  statsTimeline: StatsTimeline | null,
  options: AutoPossessionCameraOptions = {},
): AutoPossessionSpan[] {
  if (!statsTimeline) {
    return [];
  }

  const preRollSeconds = options.preRollSeconds ?? DEFAULT_PRE_ROLL_SECONDS;
  const minPossessionSeconds = options.minPossessionSeconds ?? DEFAULT_MIN_POSSESSION_SECONDS;
  const samePlayerBridgeSeconds =
    options.samePlayerBridgeSeconds ?? DEFAULT_SAME_PLAYER_BRIDGE_SECONDS;

  const possessions: PossessionCandidate[] = statsEventPayloads(statsTimeline, "player_possession")
    .map((event) => ({
      playerId: playerIdToString(event.player_id),
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
  const spans: AutoPossessionSpan[] = [];
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

export function selectAutoPossessionCameraPlayer(
  spans: readonly AutoPossessionSpan[],
  frameIndex: number,
): string | null {
  const normalizedFrameIndex = Math.max(0, Math.trunc(frameIndex));
  const span = spans.find((candidate) => {
    return (
      normalizedFrameIndex >= candidate.startFrame && normalizedFrameIndex < candidate.endFrame
    );
  });
  return span?.playerId ?? null;
}

export class AutoPossessionCameraController {
  private spans: AutoPossessionSpan[] = [];
  private unsubscribeBeforeRender: (() => void) | null = null;
  private sourcePlayer: StatsReplayPlayer | null = null;
  private sourceTimeline: StatsTimeline | null = null;

  constructor(private readonly options: AutoPossessionCameraControllerOptions) {}

  syncSource(): void {
    const player = this.options.getReplayPlayer();
    const statsTimeline = this.options.getStatsTimeline();
    if (player === this.sourcePlayer && statsTimeline === this.sourceTimeline) {
      return;
    }

    this.unsubscribeBeforeRender?.();
    this.unsubscribeBeforeRender = null;
    this.sourcePlayer = player;
    this.sourceTimeline = statsTimeline;
    this.spans = buildAutoPossessionCameraSpans(statsTimeline);

    if (player) {
      this.unsubscribeBeforeRender = player.onBeforeRender((info) => this.update(info));
    }
  }

  reset(): void {
    this.unsubscribeBeforeRender?.();
    this.unsubscribeBeforeRender = null;
    this.sourcePlayer = null;
    this.sourceTimeline = null;
    this.spans = [];
  }

  syncCurrentFrame(): void {
    const replayPlayer = this.options.getReplayPlayer();
    if (!replayPlayer) {
      return;
    }

    const state = replayPlayer.getState();
    this.update({
      frameIndex: state.frameIndex,
      nextFrameIndex: state.frameIndex,
      alpha: 0,
      currentTime: state.currentTime,
    });
  }

  update(info: FrameRenderInfo): void {
    const cameraControls = this.options.getCameraControlsController();
    const replayPlayer = this.options.getReplayPlayer();
    if (!cameraControls?.autoPossessionEnabled || !replayPlayer) {
      return;
    }

    const playerId = selectAutoPossessionCameraPlayer(this.spans, info.frameIndex);
    if (!playerId || !replayPlayer.replay.players.some((player) => player.id === playerId)) {
      return;
    }

    const state = replayPlayer.getState();
    if (state.cameraViewMode === "follow" && state.attachedPlayerId === playerId) {
      return;
    }

    cameraControls.followPlayerWithReplayCamera(playerId, {
      ballCam: "player",
      preserveAutoPossession: true,
      requestConfigSync: false,
      usePlayerCameraSettings: false,
    });
  }
}

export function createAutoPossessionCameraController(
  options: AutoPossessionCameraControllerOptions,
): AutoPossessionCameraController {
  return new AutoPossessionCameraController(options);
}
