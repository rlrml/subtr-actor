import type { WallAerialEvent } from "./generated/WallAerialEvent.ts";
import type { WallAerialStats } from "./generated/WallAerialStats.ts";
import type { StatsTimeline } from "./statsTimeline.ts";

const WALL_AERIAL_HIGH_CONFIDENCE = 0.78;

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

function defaultWallAerialStats(): WallAerialStats {
  return {
    count: 0,
    high_confidence_count: 0,
    is_last_wall_aerial: false,
    last_wall_aerial_time: null,
    last_wall_aerial_frame: null,
    time_since_last_wall_aerial: null,
    frames_since_last_wall_aerial: null,
    last_confidence: null,
    best_confidence: 0,
    cumulative_confidence: 0,
    cumulative_setup_duration: 0,
    cumulative_takeoff_to_touch_time: 0,
    cumulative_touch_height: 0,
  };
}

function sortWallAerialEvents(events: readonly WallAerialEvent[]): WallAerialEvent[] {
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

function advanceWallAerialStats(
  stats: WallAerialStats,
  frameNumber: number,
  frameTime: number,
  isLastWallAerialPlayer: boolean,
): void {
  stats.is_last_wall_aerial = isLastWallAerialPlayer;
  stats.time_since_last_wall_aerial =
    stats.last_wall_aerial_time == null
      ? null
      : Math.max(0, frameTime - stats.last_wall_aerial_time);
  stats.frames_since_last_wall_aerial =
    stats.last_wall_aerial_frame == null
      ? null
      : Math.max(0, frameNumber - stats.last_wall_aerial_frame);
}

function applyWallAerialEvent(
  stats: WallAerialStats,
  event: WallAerialEvent,
  frameNumber: number,
  frameTime: number,
): void {
  stats.count += 1;
  if (event.confidence >= WALL_AERIAL_HIGH_CONFIDENCE) {
    stats.high_confidence_count += 1;
  }
  stats.is_last_wall_aerial = true;
  stats.last_wall_aerial_time = event.time;
  stats.last_wall_aerial_frame = event.frame;
  stats.time_since_last_wall_aerial = Math.max(0, frameTime - event.time);
  stats.frames_since_last_wall_aerial = Math.max(0, frameNumber - event.frame);
  stats.last_confidence = event.confidence;
  stats.best_confidence = Math.max(stats.best_confidence, event.confidence);
  stats.cumulative_confidence += event.confidence;
  stats.cumulative_setup_duration += event.setup_duration;
  stats.cumulative_takeoff_to_touch_time += event.time_since_takeoff;
  stats.cumulative_touch_height += event.player_position[2] ?? 0;
}

function assignWallAerialStats(
  target: WallAerialStats,
  source: WallAerialStats | undefined,
): void {
  Object.assign(target, source ?? defaultWallAerialStats());
}

export function applyWallAerialEventDerivedStats(timeline: StatsTimeline): StatsTimeline {
  const events = sortWallAerialEvents(timeline.events.wall_aerial ?? []);

  let eventIndex = 0;
  let lastWallAerialPlayer: string | null = null;
  const players = new Map<string, WallAerialStats>();

  for (const frame of timeline.frames) {
    for (const [playerKey, stats] of players) {
      advanceWallAerialStats(
        stats,
        frame.frame_number,
        frame.time,
        frame.is_live_play && playerKey === lastWallAerialPlayer,
      );
    }

    if (!frame.is_live_play) {
      lastWallAerialPlayer = null;
    } else {
      while (eventIndex < events.length && events[eventIndex]!.frame <= frame.frame_number) {
        const event = events[eventIndex] as WallAerialEvent;
        const playerKey = remoteIdKey(event.player);
        const stats = players.get(playerKey) ?? defaultWallAerialStats();
        players.set(playerKey, stats);
        applyWallAerialEvent(stats, event, frame.frame_number, frame.time);
        lastWallAerialPlayer = playerKey;
        eventIndex += 1;
      }

      if (lastWallAerialPlayer != null) {
        const stats = players.get(lastWallAerialPlayer);
        if (stats) {
          stats.is_last_wall_aerial = true;
        }
      }
    }

    for (const player of frame.players) {
      assignWallAerialStats(player.wall_aerial, players.get(remoteIdKey(player.player_id)));
    }
  }

  return timeline;
}
