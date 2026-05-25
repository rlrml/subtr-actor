import type { FlickEvent } from "./generated/FlickEvent.ts";
import type { FlickStats } from "./generated/FlickStats.ts";
import type { LabeledCounts } from "./generated/LabeledCounts.ts";
import type { StatLabel } from "./generated/StatLabel.ts";
import type { StatsTimeline } from "./statsTimeline.ts";

const FLICK_HIGH_CONFIDENCE = 0.8;

type FlickStatsWithLabels = FlickStats & {
  labeled_event_counts?: LabeledCounts;
};

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

function defaultFlickStats(): FlickStatsWithLabels {
  return {
    count: 0,
    high_confidence_count: 0,
    is_last_flick: false,
    last_flick_time: null,
    last_flick_frame: null,
    time_since_last_flick: null,
    frames_since_last_flick: null,
    last_confidence: null,
    best_confidence: 0,
    cumulative_confidence: 0,
    cumulative_setup_duration: 0,
    cumulative_ball_speed_change: 0,
  };
}

function sortFlickEvents(events: readonly FlickEvent[]): FlickEvent[] {
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

function labelSortKey(label: StatLabel): string {
  return `${label.key}\u0000${label.value}`;
}

function labelsSortKey(labels: readonly StatLabel[]): string {
  return labels.map(labelSortKey).join("\u0001");
}

function incrementLabels(stats: FlickStatsWithLabels, labels: StatLabel[]): void {
  labels.sort((left, right) => labelSortKey(left).localeCompare(labelSortKey(right)));
  const labeledCounts = (stats.labeled_event_counts ??= { entries: [] });
  const existing = labeledCounts.entries.find(
    (entry) => labelsSortKey(entry.labels) === labelsSortKey(labels),
  );
  if (existing) {
    existing.count += 1;
  } else {
    labeledCounts.entries.push({ labels: [...labels], count: 1 });
    labeledCounts.entries.sort((left, right) =>
      labelsSortKey(left.labels).localeCompare(labelsSortKey(right.labels)),
    );
  }
}

function countWithLabel(stats: FlickStatsWithLabels, value: "standard" | "high"): number {
  return (
    stats.labeled_event_counts?.entries
      .filter((entry) =>
        entry.labels.some((label) => label.key === "confidence_band" && label.value === value),
      )
      .reduce((total, entry) => total + entry.count, 0) ?? 0
  );
}

function totalLabeledCount(stats: FlickStatsWithLabels): number {
  return stats.labeled_event_counts?.entries.reduce((total, entry) => total + entry.count, 0) ?? 0;
}

function advanceFlickStats(
  stats: FlickStatsWithLabels,
  frameNumber: number,
  frameTime: number,
  isLastFlickPlayer: boolean,
): void {
  stats.is_last_flick = isLastFlickPlayer;
  stats.time_since_last_flick =
    stats.last_flick_time == null ? null : Math.max(0, frameTime - stats.last_flick_time);
  stats.frames_since_last_flick =
    stats.last_flick_frame == null ? null : Math.max(0, frameNumber - stats.last_flick_frame);
}

function applyFlickEvent(
  stats: FlickStatsWithLabels,
  event: FlickEvent,
  frameNumber: number,
  frameTime: number,
): void {
  incrementLabels(stats, [
    {
      key: "confidence_band",
      value: event.confidence >= FLICK_HIGH_CONFIDENCE ? "high" : "standard",
    },
  ]);
  stats.count = totalLabeledCount(stats);
  stats.high_confidence_count = countWithLabel(stats, "high");
  stats.is_last_flick = true;
  stats.last_flick_time = event.time;
  stats.last_flick_frame = event.frame;
  stats.time_since_last_flick = Math.max(0, frameTime - event.time);
  stats.frames_since_last_flick = Math.max(0, frameNumber - event.frame);
  stats.last_confidence = event.confidence;
  stats.best_confidence = Math.max(stats.best_confidence, event.confidence);
  stats.cumulative_confidence += event.confidence;
  stats.cumulative_setup_duration += event.setup_duration;
  stats.cumulative_ball_speed_change += event.ball_speed_change;
}

function assignFlickStats(target: FlickStats, source: FlickStatsWithLabels | undefined): void {
  Object.assign(target, source ?? defaultFlickStats());
  if (!source?.labeled_event_counts) {
    delete (target as FlickStatsWithLabels).labeled_event_counts;
  }
}

export function applyFlickEventDerivedStats(timeline: StatsTimeline): StatsTimeline {
  const events = sortFlickEvents(timeline.events.flick ?? []);

  let eventIndex = 0;
  let lastFlickPlayer: string | null = null;
  const players = new Map<string, FlickStatsWithLabels>();

  for (const frame of timeline.frames) {
    if (frame.is_live_play) {
      for (const [playerKey, stats] of players) {
        advanceFlickStats(stats, frame.frame_number, frame.time, playerKey === lastFlickPlayer);
      }

      while (eventIndex < events.length && events[eventIndex]!.frame <= frame.frame_number) {
        const event = events[eventIndex] as FlickEvent;
        const playerKey = remoteIdKey(event.player);
        const stats = players.get(playerKey) ?? defaultFlickStats();
        players.set(playerKey, stats);
        applyFlickEvent(stats, event, frame.frame_number, frame.time);
        lastFlickPlayer = playerKey;
        eventIndex += 1;
      }
    } else {
      lastFlickPlayer = null;
    }

    for (const player of frame.players) {
      assignFlickStats(player.flick, players.get(remoteIdKey(player.player_id)));
    }
  }

  return timeline;
}
