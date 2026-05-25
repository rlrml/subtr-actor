import type { LabeledCounts } from "./generated/LabeledCounts.ts";
import type { MustyFlickEvent } from "./generated/MustyFlickEvent.ts";
import type { MustyFlickStats } from "./generated/MustyFlickStats.ts";
import type { StatLabel } from "./generated/StatLabel.ts";
import type { StatsTimeline } from "./statsTimeline.ts";

const MUSTY_HIGH_CONFIDENCE = 0.8;

type MustyFlickStatsWithLabels = MustyFlickStats & {
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

function defaultMustyFlickStats(): MustyFlickStatsWithLabels {
  return {
    count: 0,
    aerial_count: 0,
    high_confidence_count: 0,
    is_last_musty: false,
    last_musty_time: null,
    last_musty_frame: null,
    time_since_last_musty: null,
    frames_since_last_musty: null,
    last_confidence: null,
    best_confidence: 0,
    cumulative_confidence: 0,
  };
}

function sortMustyFlickEvents(events: readonly MustyFlickEvent[]): MustyFlickEvent[] {
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

function incrementLabels(stats: MustyFlickStatsWithLabels, labels: StatLabel[]): void {
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

function countWithLabel(stats: MustyFlickStatsWithLabels, key: string, value: string): number {
  return (
    stats.labeled_event_counts?.entries
      .filter((entry) => entry.labels.some((label) => label.key === key && label.value === value))
      .reduce((total, entry) => total + entry.count, 0) ?? 0
  );
}

function totalLabeledCount(stats: MustyFlickStatsWithLabels): number {
  return stats.labeled_event_counts?.entries.reduce((total, entry) => total + entry.count, 0) ?? 0;
}

function advanceMustyFlickStats(
  stats: MustyFlickStatsWithLabels,
  frameNumber: number,
  frameTime: number,
  isLastMustyPlayer: boolean,
): void {
  stats.is_last_musty = isLastMustyPlayer;
  stats.time_since_last_musty =
    stats.last_musty_time == null ? null : Math.max(0, frameTime - stats.last_musty_time);
  stats.frames_since_last_musty =
    stats.last_musty_frame == null ? null : Math.max(0, frameNumber - stats.last_musty_frame);
}

function applyMustyFlickEvent(
  stats: MustyFlickStatsWithLabels,
  event: MustyFlickEvent,
  frameNumber: number,
  frameTime: number,
): void {
  incrementLabels(stats, [
    {
      key: "vertical_state",
      value: event.aerial ? "aerial" : "grounded",
    },
    {
      key: "confidence_band",
      value: event.confidence >= MUSTY_HIGH_CONFIDENCE ? "high" : "standard",
    },
  ]);
  stats.count = totalLabeledCount(stats);
  stats.aerial_count = countWithLabel(stats, "vertical_state", "aerial");
  stats.high_confidence_count = countWithLabel(stats, "confidence_band", "high");
  stats.is_last_musty = true;
  stats.last_musty_time = event.time;
  stats.last_musty_frame = event.frame;
  stats.time_since_last_musty = Math.max(0, frameTime - event.time);
  stats.frames_since_last_musty = Math.max(0, frameNumber - event.frame);
  stats.last_confidence = event.confidence;
  stats.best_confidence = Math.max(stats.best_confidence, event.confidence);
  stats.cumulative_confidence += event.confidence;
}

function assignMustyFlickStats(
  target: MustyFlickStats,
  source: MustyFlickStatsWithLabels | undefined,
): void {
  Object.assign(target, source ?? defaultMustyFlickStats());
  if (!source?.labeled_event_counts) {
    delete (target as MustyFlickStatsWithLabels).labeled_event_counts;
  }
}

export function applyMustyFlickEventDerivedStats(timeline: StatsTimeline): StatsTimeline {
  const events = sortMustyFlickEvents(timeline.events.musty_flick ?? []);

  let eventIndex = 0;
  let lastMustyPlayer: string | null = null;
  const players = new Map<string, MustyFlickStatsWithLabels>();

  for (const frame of timeline.frames) {
    if (frame.is_live_play) {
      for (const [playerKey, stats] of players) {
        advanceMustyFlickStats(
          stats,
          frame.frame_number,
          frame.time,
          lastMustyPlayer === playerKey,
        );
      }

      while (eventIndex < events.length && events[eventIndex]!.frame <= frame.frame_number) {
        const event = events[eventIndex] as MustyFlickEvent;
        const playerKey = remoteIdKey(event.player);
        const stats = players.get(playerKey) ?? defaultMustyFlickStats();
        players.set(playerKey, stats);
        applyMustyFlickEvent(stats, event, frame.frame_number, frame.time);
        lastMustyPlayer = playerKey;
        eventIndex += 1;
      }
    } else {
      lastMustyPlayer = null;
    }

    for (const player of frame.players) {
      assignMustyFlickStats(player.musty_flick, players.get(remoteIdKey(player.player_id)));
    }
  }

  return timeline;
}
