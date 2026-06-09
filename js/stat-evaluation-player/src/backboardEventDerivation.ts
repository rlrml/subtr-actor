import type { BackboardBounceEvent } from "./generated/BackboardBounceEvent.ts";
import type { BackboardPlayerStats } from "./generated/BackboardPlayerStats.ts";
import type { BackboardTeamStats } from "./generated/BackboardTeamStats.ts";
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

function defaultBackboardPlayerStats(): BackboardPlayerStats {
  return {
    count: 0,
    is_last_backboard: false,
    last_backboard_time: null,
    last_backboard_frame: null,
    time_since_last_backboard: null,
    frames_since_last_backboard: null,
  };
}

function sortBackboardEvents(events: readonly BackboardBounceEvent[]): BackboardBounceEvent[] {
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

function advanceBackboardFrame(
  stats: BackboardPlayerStats,
  frameNumber: number,
  frameTime: number,
  isLastBackboardPlayer: boolean,
): void {
  stats.is_last_backboard = isLastBackboardPlayer;
  stats.time_since_last_backboard =
    stats.last_backboard_time == null ? null : Math.max(0, frameTime - stats.last_backboard_time);
  stats.frames_since_last_backboard =
    stats.last_backboard_frame == null
      ? null
      : Math.max(0, frameNumber - stats.last_backboard_frame);
}

function applyBackboardEvent(
  stats: BackboardPlayerStats,
  event: BackboardBounceEvent,
  frameNumber: number,
  frameTime: number,
): void {
  stats.count += 1;
  stats.last_backboard_time = event.time;
  stats.last_backboard_frame = event.frame;
  stats.time_since_last_backboard = Math.max(0, frameTime - event.time);
  stats.frames_since_last_backboard = Math.max(0, frameNumber - event.frame);
}

function assignBackboardPlayerStats(
  target: BackboardPlayerStats,
  source: BackboardPlayerStats | undefined,
): void {
  Object.assign(target, source ?? defaultBackboardPlayerStats());
}

function assignBackboardTeamStats(target: BackboardTeamStats, count: number): void {
  target.count = count;
}

export function applyBackboardEventDerivedStats(
  timeline: MaterializedStatsTimeline,
): MaterializedStatsTimeline {
  const accumulator = createBackboardEventDerivedStatsAccumulator(timeline);

  for (const frame of timeline.frames) {
    accumulator.applyFrame(frame);
  }

  return timeline;
}

export function createBackboardEventDerivedStatsAccumulator(timeline: MaterializedStatsTimeline): {
  applyFrame(frame: StatsFrame): void;
} {
  const events = sortBackboardEvents(statsEventPayloads(timeline, "backboard"));

  let eventIndex = 0;
  let teamZeroCount = 0;
  let teamOneCount = 0;
  let lastBackboardPlayer: string | null = null;
  const players = new Map<string, BackboardPlayerStats>();

  return {
    applyFrame(frame: StatsFrame): void {
      for (const [playerKey, stats] of players) {
        advanceBackboardFrame(
          stats,
          frame.frame_number,
          frame.time,
          playerKey === lastBackboardPlayer,
        );
      }

      let processedEvent = false;
      while (eventIndex < events.length && events[eventIndex]!.frame <= frame.frame_number) {
        const event = events[eventIndex] as BackboardBounceEvent;
        const playerKey = remoteIdKey(event.player);
        const stats = players.get(playerKey) ?? defaultBackboardPlayerStats();
        players.set(playerKey, stats);
        applyBackboardEvent(stats, event, frame.frame_number, frame.time);
        if (event.is_team_0) {
          teamZeroCount += 1;
        } else {
          teamOneCount += 1;
        }
        lastBackboardPlayer = playerKey;
        processedEvent = true;
        eventIndex += 1;
      }

      if (processedEvent) {
        for (const stats of players.values()) {
          stats.is_last_backboard = false;
        }
      }
      if (lastBackboardPlayer != null) {
        const stats = players.get(lastBackboardPlayer);
        if (stats) {
          stats.is_last_backboard = true;
        }
      }

      assignBackboardTeamStats(frame.team_zero.backboard, teamZeroCount);
      assignBackboardTeamStats(frame.team_one.backboard, teamOneCount);
      for (const player of frame.players) {
        assignBackboardPlayerStats(player.backboard, players.get(remoteIdKey(player.player_id)));
      }
    },
  };
}
