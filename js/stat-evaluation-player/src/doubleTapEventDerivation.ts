import type { DoubleTapEvent } from "./generated/DoubleTapEvent.ts";
import type { DoubleTapPlayerStats } from "./generated/DoubleTapPlayerStats.ts";
import type { DoubleTapTeamStats } from "./generated/DoubleTapTeamStats.ts";
import type { StatsFrame, MaterializedStatsTimeline } from "./statsTimeline.ts";

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

function defaultDoubleTapPlayerStats(): DoubleTapPlayerStats {
  return {
    count: 0,
    is_last_double_tap: false,
    last_double_tap_time: null,
    last_double_tap_frame: null,
    time_since_last_double_tap: null,
    frames_since_last_double_tap: null,
  };
}

function sortDoubleTapEvents(events: readonly DoubleTapEvent[]): DoubleTapEvent[] {
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

function advanceDoubleTapFrame(
  stats: DoubleTapPlayerStats,
  frameNumber: number,
  frameTime: number,
  isLastDoubleTapPlayer: boolean,
): void {
  stats.is_last_double_tap = isLastDoubleTapPlayer;
  stats.time_since_last_double_tap =
    stats.last_double_tap_time == null ? null : Math.max(0, frameTime - stats.last_double_tap_time);
  stats.frames_since_last_double_tap =
    stats.last_double_tap_frame == null
      ? null
      : Math.max(0, frameNumber - stats.last_double_tap_frame);
}

function applyDoubleTapEvent(
  stats: DoubleTapPlayerStats,
  event: DoubleTapEvent,
  frameNumber: number,
  frameTime: number,
): void {
  stats.count += 1;
  stats.last_double_tap_time = event.time;
  stats.last_double_tap_frame = event.frame;
  stats.time_since_last_double_tap = Math.max(0, frameTime - event.time);
  stats.frames_since_last_double_tap = Math.max(0, frameNumber - event.frame);
}

function assignDoubleTapPlayerStats(
  target: DoubleTapPlayerStats,
  source: DoubleTapPlayerStats | undefined,
): void {
  Object.assign(target, source ?? defaultDoubleTapPlayerStats());
}

function assignDoubleTapTeamStats(target: DoubleTapTeamStats, count: number): void {
  target.count = count;
}

export function applyDoubleTapEventDerivedStats(
  timeline: MaterializedStatsTimeline,
): MaterializedStatsTimeline {
  const accumulator = createDoubleTapEventDerivedStatsAccumulator(timeline);

  for (const frame of timeline.frames) {
    accumulator.applyFrame(frame);
  }

  return timeline;
}

export function createDoubleTapEventDerivedStatsAccumulator(timeline: MaterializedStatsTimeline): {
  applyFrame(frame: StatsFrame): void;
} {
  const events = sortDoubleTapEvents(timeline.events.double_tap ?? []);

  let eventIndex = 0;
  let teamZeroCount = 0;
  let teamOneCount = 0;
  let lastDoubleTapPlayer: string | null = null;
  const players = new Map<string, DoubleTapPlayerStats>();

  return {
    applyFrame(frame: StatsFrame): void {
      for (const [playerKey, stats] of players) {
        advanceDoubleTapFrame(
          stats,
          frame.frame_number,
          frame.time,
          playerKey === lastDoubleTapPlayer,
        );
      }

      let processedEvent = false;
      while (eventIndex < events.length && events[eventIndex]!.frame <= frame.frame_number) {
        const event = events[eventIndex] as DoubleTapEvent;
        const playerKey = remoteIdKey(event.player);
        const stats = players.get(playerKey) ?? defaultDoubleTapPlayerStats();
        players.set(playerKey, stats);
        applyDoubleTapEvent(stats, event, frame.frame_number, frame.time);
        if (event.is_team_0) {
          teamZeroCount += 1;
        } else {
          teamOneCount += 1;
        }
        lastDoubleTapPlayer = playerKey;
        processedEvent = true;
        eventIndex += 1;
      }

      if (processedEvent) {
        for (const stats of players.values()) {
          stats.is_last_double_tap = false;
        }
      }
      if (lastDoubleTapPlayer != null) {
        const stats = players.get(lastDoubleTapPlayer);
        if (stats) {
          stats.is_last_double_tap = true;
        }
      }

      assignDoubleTapTeamStats(frame.team_zero.double_tap, teamZeroCount);
      assignDoubleTapTeamStats(frame.team_one.double_tap, teamOneCount);
      for (const player of frame.players) {
        assignDoubleTapPlayerStats(player.double_tap, players.get(remoteIdKey(player.player_id)));
      }
    },
  };
}
