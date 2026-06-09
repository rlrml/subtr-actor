import type { LabeledCounts } from "./generated/LabeledCounts.ts";
import type { MustyFlickEvent } from "./generated/MustyFlickEvent.ts";
import type { MustyFlickStats } from "./generated/MustyFlickStats.ts";
import type { StatLabel } from "./generated/StatLabel.ts";
import type { StatsFrame, MaterializedStatsTimeline } from "./statsTimeline.ts";
import { statsEventPayloads } from "./statsTimeline.ts";

const MUSTY_HIGH_CONFIDENCE = 0.8;

type MustyFlickStatsWithLabels = MustyFlickStats & {
  labeled_event_counts?: LabeledCounts;
};

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
      const leftSampleFrame = left.event.sample_frame ?? left.event.frame;
      const rightSampleFrame = right.event.sample_frame ?? right.event.frame;
      if (leftSampleFrame !== rightSampleFrame) {
        return leftSampleFrame - rightSampleFrame;
      }
      const leftSampleTime = left.event.sample_time ?? left.event.time;
      const rightSampleTime = right.event.sample_time ?? right.event.time;
      if (leftSampleTime !== rightSampleTime) {
        return leftSampleTime - rightSampleTime;
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

function cloneLabeledCounts(counts: LabeledCounts): LabeledCounts {
  return {
    entries: counts.entries.map((entry) => ({
      labels: entry.labels.map((label) => ({ ...label })),
      count: entry.count,
    })),
  };
}

function advanceMustyFlickStats(
  stats: MustyFlickStatsWithLabels,
  frameNumber: number,
  frameTime: number,
  isLastMustyPlayer: boolean,
): void {
  stats.is_last_musty = isLastMustyPlayer;
  stats.time_since_last_musty =
    stats.last_musty_time == null ? null : Math.max(0, subF32(frameTime, stats.last_musty_time));
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
  stats.time_since_last_musty = Math.max(0, subF32(frameTime, event.time));
  stats.frames_since_last_musty = Math.max(0, frameNumber - event.frame);
  stats.last_confidence = event.confidence;
  stats.best_confidence = Math.max(stats.best_confidence, event.confidence);
  stats.cumulative_confidence = addF32(stats.cumulative_confidence, event.confidence);
}

function assignMustyFlickStats(
  target: MustyFlickStats,
  source: MustyFlickStatsWithLabels | undefined,
): void {
  Object.assign(target, source ?? defaultMustyFlickStats());
  if (source?.labeled_event_counts) {
    (target as MustyFlickStatsWithLabels).labeled_event_counts = cloneLabeledCounts(
      source.labeled_event_counts,
    );
  } else {
    delete (target as MustyFlickStatsWithLabels).labeled_event_counts;
  }
}

export function applyMustyFlickEventDerivedStats(
  timeline: MaterializedStatsTimeline,
): MaterializedStatsTimeline {
  const accumulator = createMustyFlickEventDerivedStatsAccumulator(timeline);

  for (const frame of timeline.frames) {
    accumulator.applyFrame(frame);
  }

  return timeline;
}

export function createMustyFlickEventDerivedStatsAccumulator(timeline: MaterializedStatsTimeline): {
  applyFrame(frame: StatsFrame): void;
} {
  const events = sortMustyFlickEvents(statsEventPayloads(timeline, "musty_flick"));

  let eventIndex = 0;
  let lastMustyPlayer: string | null = null;
  const players = new Map<string, MustyFlickStatsWithLabels>();

  return {
    applyFrame(frame: StatsFrame): void {
      if (frame.is_live_play) {
        for (const [playerKey, stats] of players) {
          advanceMustyFlickStats(
            stats,
            frame.frame_number,
            frame.time,
            lastMustyPlayer === playerKey,
          );
        }

        let processedEvent = false;
        while (
          eventIndex < events.length &&
          (events[eventIndex]!.sample_frame ?? events[eventIndex]!.frame) <= frame.frame_number
        ) {
          const event = events[eventIndex] as MustyFlickEvent;
          const playerKey = remoteIdKey(event.player);
          const stats = players.get(playerKey) ?? defaultMustyFlickStats();
          players.set(playerKey, stats);
          applyMustyFlickEvent(stats, event, frame.frame_number, frame.time);
          lastMustyPlayer = playerKey;
          eventIndex += 1;
          processedEvent = true;
        }

        if (processedEvent) {
          for (const stats of players.values()) {
            stats.is_last_musty = false;
          }
        }
        if (lastMustyPlayer != null) {
          const stats = players.get(lastMustyPlayer);
          if (stats) {
            stats.is_last_musty = true;
          }
        }
      } else {
        lastMustyPlayer = null;
      }

      for (const player of frame.players) {
        assignMustyFlickStats(player.musty_flick, players.get(remoteIdKey(player.player_id)));
      }
    },
  };
}
