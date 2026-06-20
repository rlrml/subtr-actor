import type { FrameRenderInfo, ReplayModel } from "@rlrml/player";
import type { CameraControlsController } from "./cameraControls.ts";
import type { StatsEventPayload, StatsTimeline } from "./statsTimeline.ts";
import { statsEventPayloads } from "./statsTimeline.ts";
import type { StatsReplayPlayer } from "./statsReplayPlayer.ts";
import { playerIdToString } from "./touchOverlay.ts";

export interface AutoPossessionCameraControllerOptions {
  getReplayPlayer(): StatsReplayPlayer | null;
  getStatsTimeline(): StatsTimeline | null;
  getCameraControlsController(): CameraControlsController | null;
}

export interface AutoPossessionSpan {
  playerId: string;
  startFrame: number;
  endFrame: number;
  startTime: number;
  endTime: number;
}

type PlayerPossessionEvent = StatsEventPayload<"player_possession">;
type Position3 = { x: number; y: number; z: number };

export function buildAutoPossessionCameraSpans(
  statsTimeline: StatsTimeline | null,
): AutoPossessionSpan[] {
  if (!statsTimeline) {
    return [];
  }

  return statsEventPayloads(statsTimeline, "player_possession")
    .filter((event) => event.duration > 0)
    .map((event: PlayerPossessionEvent) => ({
      playerId: playerIdToString(event.player_id),
      startFrame: event.start_frame,
      endFrame: event.end_frame,
      startTime: event.start_time,
      endTime: event.end_time,
    }))
    .sort((left, right) => {
      if (left.startFrame !== right.startFrame) {
        return left.startFrame - right.startFrame;
      }
      return right.endFrame - left.endFrame;
    });
}

function distanceSquared(left: Position3, right: Position3): number {
  const dx = left.x - right.x;
  const dy = left.y - right.y;
  const dz = left.z - right.z;
  return dx * dx + dy * dy + dz * dz;
}

function activeSpan(
  spans: readonly AutoPossessionSpan[],
  frameIndex: number,
  currentTime: number,
): AutoPossessionSpan | null {
  for (const span of spans) {
    if (
      frameIndex >= span.startFrame &&
      frameIndex <= span.endFrame &&
      currentTime >= span.startTime &&
      currentTime <= span.endTime
    ) {
      return span;
    }
  }
  return null;
}

function closestPlayerToBall(replay: ReplayModel, frameIndex: number): string | null {
  const ballPosition = replay.ballFrames[frameIndex]?.position ?? null;
  let fallbackPlayerId: string | null = null;
  let closestPlayerId: string | null = null;
  let closestDistance = Number.POSITIVE_INFINITY;

  for (const player of replay.players) {
    const frame = player.frames[frameIndex];
    if (!frame || frame.isPresent === false || frame.position === null) {
      continue;
    }

    fallbackPlayerId ??= player.id;
    if (ballPosition === null) {
      continue;
    }

    const distance = distanceSquared(frame.position, ballPosition);
    if (distance < closestDistance) {
      closestDistance = distance;
      closestPlayerId = player.id;
    }
  }

  return closestPlayerId ?? fallbackPlayerId ?? replay.players[0]?.id ?? null;
}

export function selectAutoPossessionCameraPlayer(
  replay: ReplayModel,
  spans: readonly AutoPossessionSpan[],
  frameIndex: number,
  currentTime: number,
): string | null {
  const spanPlayerId = activeSpan(spans, frameIndex, currentTime)?.playerId ?? null;
  if (spanPlayerId && replay.players.some((player) => player.id === spanPlayerId)) {
    return spanPlayerId;
  }
  return closestPlayerToBall(replay, frameIndex);
}

export class AutoPossessionCameraController {
  private spans: AutoPossessionSpan[] = [];
  private unsubscribeBeforeRender: (() => void) | null = null;
  private attachedByAuto: string | null = null;
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
    this.attachedByAuto = null;
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
    this.attachedByAuto = null;
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

    const playerId = selectAutoPossessionCameraPlayer(
      replayPlayer.replay,
      this.spans,
      info.frameIndex,
      info.currentTime,
    );
    if (playerId === null) {
      return;
    }

    const state = replayPlayer.getState();
    if (
      playerId === this.attachedByAuto &&
      state.cameraViewMode === "follow" &&
      state.attachedPlayerId === playerId
    ) {
      return;
    }

    this.attachedByAuto = playerId;
    cameraControls.followPlayerWithReplayCamera(playerId, {
      ballCam: "player",
      preserveAutoPossession: true,
      requestConfigSync: false,
      usePlayerCameraSettings: true,
    });
  }
}

export function createAutoPossessionCameraController(
  options: AutoPossessionCameraControllerOptions,
): AutoPossessionCameraController {
  return new AutoPossessionCameraController(options);
}
