import type { WallAerialShotEvent } from "./generated/WallAerialShotEvent.ts";
import type { WallAerialShotStats } from "./generated/WallAerialShotStats.ts";
import type { StatsFrame, MaterializedStatsTimeline } from "./statsTimeline.ts";

const WALL_AERIAL_HIGH_CONFIDENCE = 0.78;

function f32(value: number): number {
  return Math.fround(value);
}

function addF32(left: number, right: number): number {
  return f32(f32(left) + f32(right));
}

function subF32(left: number, right: number): number {
  return f32(f32(left) - f32(right));
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

function defaultWallAerialShotStats(): WallAerialShotStats {
  return {
    count: 0,
    high_confidence_count: 0,
    is_last_wall_aerial_shot: false,
    last_wall_aerial_shot_time: null,
    last_wall_aerial_shot_frame: null,
    time_since_last_wall_aerial_shot: null,
    frames_since_last_wall_aerial_shot: null,
    last_confidence: null,
    best_confidence: 0,
    cumulative_confidence: 0,
    cumulative_takeoff_to_shot_time: 0,
    cumulative_shot_height: 0,
  };
}

function sortWallAerialShotEvents(
  events: readonly WallAerialShotEvent[],
): WallAerialShotEvent[] {
  return events
    .map((event, index) => ({ event, index }))
    .sort((left, right) => {
      if (left.event.frame !== right.event.frame) {
        return left.event.frame - right.event.frame;
      }
      if (left.event.time !== right.event.time) {
        return left.event.time - right.event.time;
      }
      return left.index - right.index;
    })
    .map(({ event }) => event);
}

function advanceWallAerialShotStats(
  stats: WallAerialShotStats,
  frameNumber: number,
  frameTime: number,
  isLastWallAerialShotPlayer: boolean,
): void {
  stats.is_last_wall_aerial_shot = isLastWallAerialShotPlayer;
  stats.time_since_last_wall_aerial_shot =
    stats.last_wall_aerial_shot_time == null
      ? null
      : Math.max(0, subF32(frameTime, stats.last_wall_aerial_shot_time));
  stats.frames_since_last_wall_aerial_shot =
    stats.last_wall_aerial_shot_frame == null
      ? null
      : Math.max(0, frameNumber - stats.last_wall_aerial_shot_frame);
}

function applyWallAerialShotEvent(
  stats: WallAerialShotStats,
  event: WallAerialShotEvent,
  frameNumber: number,
  frameTime: number,
): void {
  stats.count += 1;
  if (event.confidence >= WALL_AERIAL_HIGH_CONFIDENCE) {
    stats.high_confidence_count += 1;
  }
  stats.is_last_wall_aerial_shot = true;
  stats.last_wall_aerial_shot_time = event.time;
  stats.last_wall_aerial_shot_frame = event.frame;
  stats.time_since_last_wall_aerial_shot = Math.max(0, subF32(frameTime, event.time));
  stats.frames_since_last_wall_aerial_shot = Math.max(0, frameNumber - event.frame);
  stats.last_confidence = event.confidence;
  stats.best_confidence = Math.max(stats.best_confidence, event.confidence);
  stats.cumulative_confidence = addF32(stats.cumulative_confidence, event.confidence);
  stats.cumulative_takeoff_to_shot_time = addF32(
    stats.cumulative_takeoff_to_shot_time,
    event.time_since_takeoff,
  );
  stats.cumulative_shot_height = addF32(
    stats.cumulative_shot_height,
    event.player_position[2] ?? 0,
  );
}

function assignWallAerialShotStats(
  target: WallAerialShotStats,
  source: WallAerialShotStats | undefined,
): void {
  Object.assign(target, source ?? defaultWallAerialShotStats());
}

export function applyWallAerialShotEventDerivedStats(timeline: MaterializedStatsTimeline): MaterializedStatsTimeline {
  const accumulator = createWallAerialShotEventDerivedStatsAccumulator(timeline);

  for (const frame of timeline.frames) {
    accumulator.applyFrame(frame);
  }

  return timeline;
}

export function createWallAerialShotEventDerivedStatsAccumulator(timeline: MaterializedStatsTimeline): {
  applyFrame(frame: StatsFrame): void;
} {
  const events = sortWallAerialShotEvents(timeline.events.wall_aerial_shot ?? []);

  let eventIndex = 0;
  let lastWallAerialShotPlayer: string | null = null;
  const players = new Map<string, WallAerialShotStats>();

  return {
    applyFrame(frame: StatsFrame): void {
      for (const [playerKey, stats] of players) {
        advanceWallAerialShotStats(
          stats,
          frame.frame_number,
          frame.time,
          frame.is_live_play && playerKey === lastWallAerialShotPlayer,
        );
      }

      if (!frame.is_live_play) {
        lastWallAerialShotPlayer = null;
      } else {
        let processedEvent = false;
        while (eventIndex < events.length && events[eventIndex]!.frame <= frame.frame_number) {
          const event = events[eventIndex] as WallAerialShotEvent;
          const playerKey = remoteIdKey(event.player);
          const stats = players.get(playerKey) ?? defaultWallAerialShotStats();
          players.set(playerKey, stats);
          applyWallAerialShotEvent(stats, event, frame.frame_number, frame.time);
          lastWallAerialShotPlayer = playerKey;
          processedEvent = true;
          eventIndex += 1;
        }

        if (processedEvent) {
          for (const stats of players.values()) {
            stats.is_last_wall_aerial_shot = false;
          }
        }
        if (lastWallAerialShotPlayer != null) {
          const stats = players.get(lastWallAerialShotPlayer);
          if (stats) {
            stats.is_last_wall_aerial_shot = true;
          }
        }
      }

      for (const player of frame.players) {
        assignWallAerialShotStats(
          player.wall_aerial_shot,
          players.get(remoteIdKey(player.player_id)),
        );
      }
    },
  };
}
