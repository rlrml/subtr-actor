import type { DodgeResetEvent } from "./generated/DodgeResetEvent.ts";
import type { DodgeResetStats } from "./generated/DodgeResetStats.ts";
import type { StatsFrame, MaterializedStatsTimeline } from "./statsTimeline.ts";
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

function defaultDodgeResetStats(): DodgeResetStats {
  return {
    count: 0,
    on_ball_count: 0,
    flip_reset_used_count: 0,
    flip_reset_unused_count: 0,
    flip_reset_total_time_to_use: 0,
    flip_reset_min_time_to_use: null,
  };
}

function sortDodgeResetEvents(events: readonly DodgeResetEvent[]): DodgeResetEvent[] {
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

function applyDodgeResetEvent(stats: DodgeResetStats, event: DodgeResetEvent): void {
  stats.count += 1;
  if (event.on_ball) {
    stats.on_ball_count += 1;
  }
}

function assignDodgeResetStats(target: DodgeResetStats, source: DodgeResetStats | undefined): void {
  Object.assign(target, source ?? defaultDodgeResetStats());
}

export function applyDodgeResetEventDerivedStats(
  timeline: MaterializedStatsTimeline,
): MaterializedStatsTimeline {
  const accumulator = createDodgeResetEventDerivedStatsAccumulator(timeline);

  for (const frame of timeline.frames) {
    accumulator.applyFrame(frame);
  }

  return timeline;
}

export function createDodgeResetEventDerivedStatsAccumulator(timeline: MaterializedStatsTimeline): {
  applyFrame(frame: StatsFrame): void;
} {
  const events = sortDodgeResetEvents(statsEventPayloads(timeline, "dodge_reset"));

  let eventIndex = 0;
  const players = new Map<string, DodgeResetStats>();

  return {
    applyFrame(frame: StatsFrame): void {
      while (eventIndex < events.length && events[eventIndex]!.frame <= frame.frame_number) {
        const event = events[eventIndex] as DodgeResetEvent;
        const playerKey = remoteIdKey(event.player);
        const stats = players.get(playerKey) ?? defaultDodgeResetStats();
        players.set(playerKey, stats);
        applyDodgeResetEvent(stats, event);
        eventIndex += 1;
      }

      for (const player of frame.players) {
        assignDodgeResetStats(player.dodge_reset, players.get(remoteIdKey(player.player_id)));
      }
    },
  };
}
