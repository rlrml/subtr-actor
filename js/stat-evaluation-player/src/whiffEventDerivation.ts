import type { WhiffEvent } from "./generated/WhiffEvent.ts";
import type { WhiffStats } from "./generated/WhiffStats.ts";
import type { StatsFrame, MaterializedStatsTimeline } from "./statsTimeline.ts";
import { statsEventPayloads } from "./statsTimeline.ts";

type WhiffAccumulator = Omit<WhiffStats, "labeled_whiff_counts">;

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

function defaultWhiffStats(): WhiffAccumulator {
  return {
    whiff_count: 0,
    beaten_to_ball_count: 0,
    grounded_whiff_count: 0,
    aerial_whiff_count: 0,
    dodge_whiff_count: 0,
    is_last_whiff: false,
    last_whiff_time: null,
    last_whiff_frame: null,
    time_since_last_whiff: null,
    frames_since_last_whiff: null,
    last_closest_approach_distance: null,
    best_closest_approach_distance: null,
    cumulative_closest_approach_distance: 0,
  };
}

function copyWhiffStats(stats: WhiffAccumulator): WhiffAccumulator {
  return { ...stats };
}

function sortWhiffEvents(events: readonly WhiffEvent[]): WhiffEvent[] {
  return events
    .map((event, index) => ({ event, index }))
    .sort((left, right) => {
      if (left.event.resolved_frame !== right.event.resolved_frame) {
        return left.event.resolved_frame - right.event.resolved_frame;
      }
      if (left.event.resolved_time !== right.event.resolved_time) {
        return left.event.resolved_time - right.event.resolved_time;
      }
      return left.index - right.index;
    })
    .map(({ event }) => event);
}

function advanceWhiffFrame(stats: WhiffAccumulator, frameNumber: number, frameTime: number): void {
  stats.is_last_whiff = false;
  stats.time_since_last_whiff =
    stats.last_whiff_time == null ? null : Math.max(0, frameTime - stats.last_whiff_time);
  stats.frames_since_last_whiff =
    stats.last_whiff_frame == null ? null : Math.max(0, frameNumber - stats.last_whiff_frame);
}

function applyWhiffEvent(
  stats: WhiffAccumulator,
  event: WhiffEvent,
  frameNumber: number,
  frameTime: number,
): void {
  if ((event.kind ?? "whiff") === "beaten_to_ball") {
    stats.beaten_to_ball_count += 1;
    return;
  }

  stats.whiff_count += 1;
  if (event.aerial) {
    stats.aerial_whiff_count += 1;
  } else {
    stats.grounded_whiff_count += 1;
  }
  if (event.dodge_active) {
    stats.dodge_whiff_count += 1;
  }
  stats.is_last_whiff = true;
  stats.last_whiff_time = event.time;
  stats.last_whiff_frame = event.frame;
  stats.time_since_last_whiff = Math.max(0, frameTime - event.time);
  stats.frames_since_last_whiff = Math.max(0, frameNumber - event.frame);
  stats.last_closest_approach_distance = event.closest_approach_distance;
  stats.best_closest_approach_distance =
    stats.best_closest_approach_distance == null
      ? event.closest_approach_distance
      : Math.min(stats.best_closest_approach_distance, event.closest_approach_distance);
  stats.cumulative_closest_approach_distance += event.closest_approach_distance;
}

function assignWhiffStats(target: WhiffStats, source: WhiffAccumulator | undefined): void {
  Object.assign(target, source ?? defaultWhiffStats());
}

export function applyWhiffEventDerivedStats(
  timeline: MaterializedStatsTimeline,
): MaterializedStatsTimeline {
  const accumulator = createWhiffEventDerivedStatsAccumulator(timeline);

  for (const frame of timeline.frames) {
    accumulator.applyFrame(frame);
  }

  return timeline;
}

export function createWhiffEventDerivedStatsAccumulator(timeline: MaterializedStatsTimeline): {
  applyFrame(frame: StatsFrame): void;
} {
  const events = sortWhiffEvents(statsEventPayloads(timeline, "whiff"));

  let eventIndex = 0;
  let lastWhiffPlayer: string | null = null;
  const players = new Map<string, WhiffAccumulator>();
  const frozenPlayers = new Map<string, WhiffAccumulator>();

  return {
    applyFrame(frame: StatsFrame): void {
      if (frame.is_live_play) {
        for (const stats of players.values()) {
          advanceWhiffFrame(stats, frame.frame_number, frame.time);
        }

        while (
          eventIndex < events.length &&
          events[eventIndex]!.resolved_frame <= frame.frame_number
        ) {
          const event = events[eventIndex] as WhiffEvent;
          const playerKey = remoteIdKey(event.player);
          const stats = players.get(playerKey) ?? defaultWhiffStats();
          players.set(playerKey, stats);
          applyWhiffEvent(stats, event, frame.frame_number, frame.time);
          if ((event.kind ?? "whiff") === "whiff") {
            lastWhiffPlayer = playerKey;
          }
          eventIndex += 1;
        }

        if (lastWhiffPlayer != null) {
          const stats = players.get(lastWhiffPlayer);
          if (stats) {
            stats.is_last_whiff = true;
          }
        }

        for (const player of frame.players) {
          const playerKey = remoteIdKey(player.player_id);
          const stats = players.get(playerKey);
          assignWhiffStats(player.whiff, stats);
          frozenPlayers.set(playerKey, copyWhiffStats(stats ?? defaultWhiffStats()));
        }
      } else {
        for (const player of frame.players) {
          const playerKey = remoteIdKey(player.player_id);
          assignWhiffStats(player.whiff, frozenPlayers.get(playerKey));
        }
        lastWhiffPlayer = null;
      }
    },
  };
}
