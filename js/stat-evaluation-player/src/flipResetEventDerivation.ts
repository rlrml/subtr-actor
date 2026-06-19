import type { FlipResetEvent } from "./generated/FlipResetEvent.ts";
import type { FlipResetStats } from "./generated/FlipResetStats.ts";
import type { MaterializedStatsTimeline, StatsFrame } from "./statsTimeline.ts";
import { statsEventPayloads } from "./statsTimeline.ts";

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

function defaultFlipResetStats(): FlipResetStats {
  return {
    count: 0,
    total_time_to_use: 0,
    min_time_to_use: null,
  };
}

function sortFlipResetEvents(events: readonly FlipResetEvent[]): FlipResetEvent[] {
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

function applyFlipResetEvent(stats: FlipResetStats, event: FlipResetEvent): void {
  stats.count += 1;
  stats.total_time_to_use += event.time_since_reset;
  stats.min_time_to_use =
    stats.min_time_to_use === null
      ? event.time_since_reset
      : Math.min(stats.min_time_to_use, event.time_since_reset);
}

function assignFlipResetStats(target: FlipResetStats, source: FlipResetStats | undefined): void {
  Object.assign(target, source ?? defaultFlipResetStats());
}

export function applyFlipResetEventDerivedStats(
  timeline: MaterializedStatsTimeline,
): MaterializedStatsTimeline {
  const accumulator = createFlipResetEventDerivedStatsAccumulator(timeline);

  for (const frame of timeline.frames) {
    accumulator.applyFrame(frame);
  }

  return timeline;
}

export function createFlipResetEventDerivedStatsAccumulator(timeline: MaterializedStatsTimeline): {
  applyFrame(frame: StatsFrame): void;
} {
  const events = sortFlipResetEvents(statsEventPayloads(timeline, "flip_reset"));

  let eventIndex = 0;
  const players = new Map<string, FlipResetStats>();

  return {
    applyFrame(frame: StatsFrame): void {
      while (eventIndex < events.length && events[eventIndex]!.frame <= frame.frame_number) {
        const event = events[eventIndex] as FlipResetEvent;
        const playerKey = remoteIdKey(event.player);
        const stats = players.get(playerKey) ?? defaultFlipResetStats();
        players.set(playerKey, stats);
        applyFlipResetEvent(stats, event);
        eventIndex += 1;
      }

      for (const player of frame.players) {
        assignFlipResetStats(player.flip_reset, players.get(remoteIdKey(player.player_id)));
      }
    },
  };
}
