import type { ControlledPlayEvent } from "./generated/ControlledPlayEvent.ts";
import type { ControlledPlayStats } from "./generated/ControlledPlayStats.ts";
import type { StatsFrame, MaterializedStatsTimeline } from "./statsTimeline.ts";
import { statsEventPayloads } from "./statsTimeline.ts";

function f32(value: number): number {
  return Math.fround(value);
}

function addF32(left: number, right: number): number {
  return f32(f32(left) + f32(right));
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

function defaultControlledPlayStats(): ControlledPlayStats {
  return {
    count: 0,
    total_time: 0,
    longest_time: 0,
    touch_count: 0,
    total_advance_distance: 0,
  };
}

function sortControlledPlayEvents(events: readonly ControlledPlayEvent[]): ControlledPlayEvent[] {
  return events
    .map((event, index) => ({ event, index }))
    .sort((left, right) => {
      if (left.event.end_frame !== right.event.end_frame) {
        return left.event.end_frame - right.event.end_frame;
      }
      if (left.event.end_time !== right.event.end_time) {
        return left.event.end_time - right.event.end_time;
      }
      return left.index - right.index;
    })
    .map(({ event }) => event);
}

function applyControlledPlayEvent(stats: ControlledPlayStats, event: ControlledPlayEvent): void {
  stats.count += 1;
  stats.total_time = addF32(stats.total_time, event.duration);
  stats.longest_time = Math.max(stats.longest_time, event.duration);
  stats.touch_count += event.touch_count;
  stats.total_advance_distance = addF32(stats.total_advance_distance, event.total_advance_distance);
}

function assignControlledPlayStats(
  target: ControlledPlayStats,
  source: ControlledPlayStats | undefined,
): void {
  Object.assign(target, source ?? defaultControlledPlayStats());
}

export function applyControlledPlayEventDerivedStats(
  timeline: MaterializedStatsTimeline,
): MaterializedStatsTimeline {
  const accumulator = createControlledPlayEventDerivedStatsAccumulator(timeline);

  for (const frame of timeline.frames) {
    accumulator.applyFrame(frame);
  }

  return timeline;
}

export function createControlledPlayEventDerivedStatsAccumulator(
  timeline: MaterializedStatsTimeline,
): {
  applyFrame(frame: StatsFrame): void;
} {
  const events = sortControlledPlayEvents(statsEventPayloads(timeline, "controlled_play"));

  let eventIndex = 0;
  const players = new Map<string, ControlledPlayStats>();
  const teamZero = defaultControlledPlayStats();
  const teamOne = defaultControlledPlayStats();

  return {
    applyFrame(frame: StatsFrame): void {
      while (eventIndex < events.length && events[eventIndex]!.end_frame < frame.frame_number) {
        const event = events[eventIndex] as ControlledPlayEvent;
        const playerKey = remoteIdKey(event.player_id);
        const playerStats = players.get(playerKey) ?? defaultControlledPlayStats();
        players.set(playerKey, playerStats);
        applyControlledPlayEvent(playerStats, event);
        applyControlledPlayEvent(event.is_team_0 ? teamZero : teamOne, event);
        eventIndex += 1;
      }

      assignControlledPlayStats(frame.team_zero.controlled_play, teamZero);
      assignControlledPlayStats(frame.team_one.controlled_play, teamOne);
      for (const player of frame.players) {
        assignControlledPlayStats(
          player.controlled_play,
          players.get(remoteIdKey(player.player_id)),
        );
      }
    },
  };
}
