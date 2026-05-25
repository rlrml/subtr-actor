import type { HalfFlipEvent } from "./generated/HalfFlipEvent.ts";
import type { HalfFlipStats } from "./generated/HalfFlipStats.ts";
import type { SpeedFlipEvent } from "./generated/SpeedFlipEvent.ts";
import type { SpeedFlipStats } from "./generated/SpeedFlipStats.ts";
import type { WavedashEvent } from "./generated/WavedashEvent.ts";
import type { WavedashStats } from "./generated/WavedashStats.ts";
import type { StatsFrame, StatsTimeline } from "./statsTimeline.ts";

const SPEED_FLIP_HIGH_CONFIDENCE = 0.75;
const HALF_FLIP_HIGH_CONFIDENCE = 0.78;
const WAVEDASH_HIGH_CONFIDENCE = 0.75;

type PlayerEvent = { frame: number; time: number; player: unknown; confidence: number };
type ResolvedPlayerEvent = PlayerEvent & { resolved_frame: number; resolved_time: number };

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

function sortEvents<T extends PlayerEvent>(events: readonly T[]): T[] {
  return [...events].sort((left, right) => {
    if (left.frame !== right.frame) {
      return left.frame - right.frame;
    }
    if (left.time !== right.time) {
      return left.time - right.time;
    }
    return remoteIdKey(left.player).localeCompare(remoteIdKey(right.player));
  });
}

function sortResolvedEvents<T extends ResolvedPlayerEvent>(events: readonly T[]): T[] {
  return [...events].sort((left, right) => {
    if (left.resolved_frame !== right.resolved_frame) {
      return left.resolved_frame - right.resolved_frame;
    }
    if (left.resolved_time !== right.resolved_time) {
      return left.resolved_time - right.resolved_time;
    }
    if (left.frame !== right.frame) {
      return left.frame - right.frame;
    }
    if (left.time !== right.time) {
      return left.time - right.time;
    }
    return remoteIdKey(left.player).localeCompare(remoteIdKey(right.player));
  });
}

interface QualityMechanicAccumulator {
  count: number;
  highConfidenceCount: number;
  lastTime: number | null;
  lastFrame: number | null;
  lastResolvedTime: number | null;
  lastResolvedFrame: number | null;
  lastQuality: number | null;
  bestQuality: number;
  cumulativeQuality: number;
}

function createQualityMechanicAccumulator(): QualityMechanicAccumulator {
  return {
    count: 0,
    highConfidenceCount: 0,
    lastTime: null,
    lastFrame: null,
    lastResolvedTime: null,
    lastResolvedFrame: null,
    lastQuality: null,
    bestQuality: 0,
    cumulativeQuality: 0,
  };
}

function applyQualityMechanicEvent(
  accumulator: QualityMechanicAccumulator,
  event: PlayerEvent,
  resolvedFrame: number,
  resolvedTime: number,
  highConfidenceThreshold: number,
): void {
  accumulator.count += 1;
  if (event.confidence >= highConfidenceThreshold) {
    accumulator.highConfidenceCount += 1;
  }
  accumulator.lastTime = event.time;
  accumulator.lastFrame = event.frame;
  accumulator.lastResolvedTime = resolvedTime;
  accumulator.lastResolvedFrame = resolvedFrame;
  accumulator.lastQuality = event.confidence;
  accumulator.bestQuality = Math.max(accumulator.bestQuality, event.confidence);
  accumulator.cumulativeQuality += event.confidence;
}

function timeSinceLast(
  accumulator: QualityMechanicAccumulator | undefined,
  frame: StatsFrame,
): number | null {
  if (accumulator?.lastTime == null) {
    return null;
  }
  if (accumulator.lastResolvedFrame === frame.frame_number) {
    return 0;
  }
  return Math.max(0, frame.time - accumulator.lastTime);
}

function framesSinceLast(
  accumulator: QualityMechanicAccumulator | undefined,
  frame: StatsFrame,
): number | null {
  if (accumulator?.lastFrame == null) {
    return null;
  }
  if (accumulator.lastResolvedFrame === frame.frame_number) {
    return 0;
  }
  return Math.max(0, frame.frame_number - accumulator.lastFrame);
}

function applySpeedFlipStats(
  stats: SpeedFlipStats,
  accumulator: QualityMechanicAccumulator | undefined,
  frame: StatsFrame,
  isLastPlayer: boolean,
): void {
  stats.count = accumulator?.count ?? 0;
  stats.high_confidence_count = accumulator?.highConfidenceCount ?? 0;
  stats.is_last_speed_flip = isLastPlayer;
  stats.last_speed_flip_time = accumulator?.lastTime ?? null;
  stats.last_speed_flip_frame = accumulator?.lastFrame ?? null;
  stats.time_since_last_speed_flip = timeSinceLast(accumulator, frame);
  stats.frames_since_last_speed_flip = framesSinceLast(accumulator, frame);
  stats.last_quality = accumulator?.lastQuality ?? null;
  stats.best_quality = accumulator?.bestQuality ?? 0;
  stats.cumulative_quality = accumulator?.cumulativeQuality ?? 0;
}

function applyHalfFlipStats(
  stats: HalfFlipStats,
  accumulator: QualityMechanicAccumulator | undefined,
  frame: StatsFrame,
  isLastPlayer: boolean,
): void {
  stats.count = accumulator?.count ?? 0;
  stats.high_confidence_count = accumulator?.highConfidenceCount ?? 0;
  stats.is_last_half_flip = isLastPlayer;
  stats.last_half_flip_time = accumulator?.lastTime ?? null;
  stats.last_half_flip_frame = accumulator?.lastFrame ?? null;
  stats.time_since_last_half_flip = timeSinceLast(accumulator, frame);
  stats.frames_since_last_half_flip = framesSinceLast(accumulator, frame);
  stats.last_quality = accumulator?.lastQuality ?? null;
  stats.best_quality = accumulator?.bestQuality ?? 0;
  stats.cumulative_quality = accumulator?.cumulativeQuality ?? 0;
}

function applyWavedashStats(
  stats: WavedashStats,
  accumulator: QualityMechanicAccumulator | undefined,
  frame: StatsFrame,
  isLastPlayer: boolean,
): void {
  stats.count = accumulator?.count ?? 0;
  stats.high_confidence_count = accumulator?.highConfidenceCount ?? 0;
  stats.is_last_wavedash = isLastPlayer;
  stats.last_wavedash_time = accumulator?.lastTime ?? null;
  stats.last_wavedash_frame = accumulator?.lastFrame ?? null;
  stats.time_since_last_wavedash = timeSinceLast(accumulator, frame);
  stats.frames_since_last_wavedash = framesSinceLast(accumulator, frame);
  stats.last_quality = accumulator?.lastQuality ?? null;
  stats.best_quality = accumulator?.bestQuality ?? 0;
  stats.cumulative_quality = accumulator?.cumulativeQuality ?? 0;
}

function copySpeedFlipStats(stats: SpeedFlipStats): SpeedFlipStats {
  return { ...stats };
}

function copyHalfFlipStats(stats: HalfFlipStats): HalfFlipStats {
  return { ...stats };
}

function copyWavedashStats(stats: WavedashStats): WavedashStats {
  return { ...stats };
}

function restoreFrozenSpeedFlipStats(
  stats: SpeedFlipStats,
  frozen: SpeedFlipStats | undefined,
): void {
  if (frozen) {
    Object.assign(stats, frozen);
    return;
  }
  applySpeedFlipStats(stats, undefined, { frame_number: 0, time: 0 } as StatsFrame, false);
}

function restoreFrozenHalfFlipStats(stats: HalfFlipStats, frozen: HalfFlipStats | undefined): void {
  if (frozen) {
    Object.assign(stats, frozen);
    return;
  }
  applyHalfFlipStats(stats, undefined, { frame_number: 0, time: 0 } as StatsFrame, false);
}

function restoreFrozenWavedashStats(stats: WavedashStats, frozen: WavedashStats | undefined): void {
  if (frozen) {
    Object.assign(stats, frozen);
    return;
  }
  applyWavedashStats(stats, undefined, { frame_number: 0, time: 0 } as StatsFrame, false);
}

function frameAdvancesSpeedFlipStats(frame: StatsFrame): boolean {
  return frame.gameplay_phase === "active_play" || frame.gameplay_phase === "kickoff_waiting_for_touch";
}

export function applyMechanicEventDerivedStats(timeline: StatsTimeline): StatsTimeline {
  const speedFlipEvents = sortResolvedEvents(timeline.events.speed_flip ?? []);
  const halfFlipEvents = sortEvents(timeline.events.half_flip ?? []);
  const wavedashEvents = sortEvents(timeline.events.wavedash ?? []);

  let speedFlipIndex = 0;
  let halfFlipIndex = 0;
  let wavedashIndex = 0;
  let lastSpeedFlipPlayer: string | null = null;
  let lastHalfFlipPlayer: string | null = null;
  let lastWavedashPlayer: string | null = null;
  const speedFlipPlayers = new Map<string, QualityMechanicAccumulator>();
  const halfFlipPlayers = new Map<string, QualityMechanicAccumulator>();
  const wavedashPlayers = new Map<string, QualityMechanicAccumulator>();
  const speedFlipFrameStats = new Map<string, SpeedFlipStats>();
  const halfFlipFrameStats = new Map<string, HalfFlipStats>();
  const wavedashFrameStats = new Map<string, WavedashStats>();

  for (const frame of timeline.frames) {
    if (frameAdvancesSpeedFlipStats(frame)) {
      while (
        speedFlipIndex < speedFlipEvents.length &&
        speedFlipEvents[speedFlipIndex]!.resolved_frame <= frame.frame_number
      ) {
        const event = speedFlipEvents[speedFlipIndex] as SpeedFlipEvent;
        const playerKey = remoteIdKey(event.player);
        const accumulator = speedFlipPlayers.get(playerKey) ?? createQualityMechanicAccumulator();
        speedFlipPlayers.set(playerKey, accumulator);
        applyQualityMechanicEvent(
          accumulator,
          event,
          event.resolved_frame,
          event.resolved_time,
          SPEED_FLIP_HIGH_CONFIDENCE,
        );
        lastSpeedFlipPlayer = playerKey;
        speedFlipIndex += 1;
      }

      for (const player of frame.players) {
        const playerKey = remoteIdKey(player.player_id);
        applySpeedFlipStats(
          player.speed_flip,
          speedFlipPlayers.get(playerKey),
          frame,
          playerKey === lastSpeedFlipPlayer,
        );
        speedFlipFrameStats.set(playerKey, copySpeedFlipStats(player.speed_flip));
      }
    } else {
      for (const player of frame.players) {
        const playerKey = remoteIdKey(player.player_id);
        restoreFrozenSpeedFlipStats(player.speed_flip, speedFlipFrameStats.get(playerKey));
      }
    }

    if (frame.is_live_play) {
      while (
        halfFlipIndex < halfFlipEvents.length &&
        halfFlipEvents[halfFlipIndex]!.frame <= frame.frame_number
      ) {
        const event = halfFlipEvents[halfFlipIndex] as HalfFlipEvent;
        const playerKey = remoteIdKey(event.player);
        const accumulator = halfFlipPlayers.get(playerKey) ?? createQualityMechanicAccumulator();
        halfFlipPlayers.set(playerKey, accumulator);
        applyQualityMechanicEvent(
          accumulator,
          event,
          event.frame,
          event.time,
          HALF_FLIP_HIGH_CONFIDENCE,
        );
        lastHalfFlipPlayer = playerKey;
        halfFlipIndex += 1;
      }

      while (
        wavedashIndex < wavedashEvents.length &&
        wavedashEvents[wavedashIndex]!.frame <= frame.frame_number
      ) {
        const event = wavedashEvents[wavedashIndex] as WavedashEvent;
        const playerKey = remoteIdKey(event.player);
        const accumulator = wavedashPlayers.get(playerKey) ?? createQualityMechanicAccumulator();
        wavedashPlayers.set(playerKey, accumulator);
        applyQualityMechanicEvent(
          accumulator,
          event,
          event.frame,
          event.time,
          WAVEDASH_HIGH_CONFIDENCE,
        );
        lastWavedashPlayer = playerKey;
        wavedashIndex += 1;
      }

      for (const player of frame.players) {
        const playerKey = remoteIdKey(player.player_id);
        applyHalfFlipStats(
          player.half_flip,
          halfFlipPlayers.get(playerKey),
          frame,
          playerKey === lastHalfFlipPlayer,
        );
        halfFlipFrameStats.set(playerKey, copyHalfFlipStats(player.half_flip));

        applyWavedashStats(
          player.wavedash,
          wavedashPlayers.get(playerKey),
          frame,
          playerKey === lastWavedashPlayer,
        );
        wavedashFrameStats.set(playerKey, copyWavedashStats(player.wavedash));
      }
    } else {
      for (const player of frame.players) {
        const playerKey = remoteIdKey(player.player_id);
        restoreFrozenHalfFlipStats(player.half_flip, halfFlipFrameStats.get(playerKey));
        restoreFrozenWavedashStats(player.wavedash, wavedashFrameStats.get(playerKey));
      }
      lastHalfFlipPlayer = null;
      lastWavedashPlayer = null;
    }
  }

  return timeline;
}
