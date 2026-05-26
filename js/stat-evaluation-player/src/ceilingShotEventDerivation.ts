import type { CeilingShotEvent } from "./generated/CeilingShotEvent.ts";
import type { CeilingShotStats } from "./generated/CeilingShotStats.ts";
import type { LabeledCounts } from "./generated/LabeledCounts.ts";
import type { StatLabel } from "./generated/StatLabel.ts";
import type { StatsFrame, MaterializedStatsTimeline } from "./statsTimeline.ts";

const CEILING_SHOT_HIGH_CONFIDENCE = 0.78;

type CeilingShotStatsWithLabels = CeilingShotStats & {
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

function defaultCeilingShotStats(): CeilingShotStatsWithLabels {
  return {
    count: 0,
    high_confidence_count: 0,
    is_last_ceiling_shot: false,
    last_ceiling_shot_time: null,
    last_ceiling_shot_frame: null,
    time_since_last_ceiling_shot: null,
    frames_since_last_ceiling_shot: null,
    last_confidence: null,
    best_confidence: 0,
    cumulative_confidence: 0,
  };
}

function sortCeilingShotEvents(events: readonly CeilingShotEvent[]): CeilingShotEvent[] {
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

function incrementLabels(stats: CeilingShotStatsWithLabels, labels: StatLabel[]): void {
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

function countWithLabel(
  stats: CeilingShotStatsWithLabels,
  value: "standard" | "high",
): number {
  return (
    stats.labeled_event_counts?.entries
      .filter((entry) =>
        entry.labels.some((label) => label.key === "confidence_band" && label.value === value),
      )
      .reduce((total, entry) => total + entry.count, 0) ?? 0
  );
}

function totalLabeledCount(stats: CeilingShotStatsWithLabels): number {
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

function advanceCeilingShotStats(
  stats: CeilingShotStatsWithLabels,
  frameNumber: number,
  frameTime: number,
  isLastCeilingShotPlayer: boolean,
): void {
  stats.is_last_ceiling_shot = isLastCeilingShotPlayer;
  stats.time_since_last_ceiling_shot =
    stats.last_ceiling_shot_time == null
      ? null
      : Math.max(0, subF32(frameTime, stats.last_ceiling_shot_time));
  stats.frames_since_last_ceiling_shot =
    stats.last_ceiling_shot_frame == null
      ? null
      : Math.max(0, frameNumber - stats.last_ceiling_shot_frame);
}

function applyCeilingShotEvent(
  stats: CeilingShotStatsWithLabels,
  event: CeilingShotEvent,
  frameNumber: number,
  frameTime: number,
): void {
  incrementLabels(stats, [
    {
      key: "confidence_band",
      value: event.confidence >= CEILING_SHOT_HIGH_CONFIDENCE ? "high" : "standard",
    },
  ]);
  stats.count = totalLabeledCount(stats);
  stats.high_confidence_count = countWithLabel(stats, "high");
  stats.is_last_ceiling_shot = true;
  stats.last_ceiling_shot_time = event.time;
  stats.last_ceiling_shot_frame = event.frame;
  stats.time_since_last_ceiling_shot = Math.max(0, subF32(frameTime, event.time));
  stats.frames_since_last_ceiling_shot = Math.max(0, frameNumber - event.frame);
  stats.last_confidence = event.confidence;
  stats.best_confidence = Math.max(stats.best_confidence, event.confidence);
  stats.cumulative_confidence = addF32(stats.cumulative_confidence, event.confidence);
}

function assignCeilingShotStats(
  target: CeilingShotStats,
  source: CeilingShotStatsWithLabels | undefined,
): void {
  Object.assign(target, source ?? defaultCeilingShotStats());
  if (source?.labeled_event_counts) {
    (target as CeilingShotStatsWithLabels).labeled_event_counts = cloneLabeledCounts(
      source.labeled_event_counts,
    );
  } else {
    delete (target as CeilingShotStatsWithLabels).labeled_event_counts;
  }
}

export function applyCeilingShotEventDerivedStats(timeline: MaterializedStatsTimeline): MaterializedStatsTimeline {
  const accumulator = createCeilingShotEventDerivedStatsAccumulator(timeline);

  for (const frame of timeline.frames) {
    accumulator.applyFrame(frame);
  }

  return timeline;
}

export function createCeilingShotEventDerivedStatsAccumulator(timeline: MaterializedStatsTimeline): {
  applyFrame(frame: StatsFrame): void;
} {
  const events = sortCeilingShotEvents(timeline.events.ceiling_shot ?? []);

  let eventIndex = 0;
  let lastCeilingShotPlayer: string | null = null;
  const players = new Map<string, CeilingShotStatsWithLabels>();

  return {
    applyFrame(frame: StatsFrame): void {
      if (frame.is_live_play) {
        for (const [playerKey, stats] of players) {
          advanceCeilingShotStats(
            stats,
            frame.frame_number,
            frame.time,
            lastCeilingShotPlayer === playerKey,
          );
        }

        while (eventIndex < events.length && events[eventIndex]!.frame <= frame.frame_number) {
          const event = events[eventIndex] as CeilingShotEvent;
          const playerKey = remoteIdKey(event.player);
          const stats = players.get(playerKey) ?? defaultCeilingShotStats();
          players.set(playerKey, stats);
          applyCeilingShotEvent(stats, event, frame.frame_number, frame.time);
          lastCeilingShotPlayer = playerKey;
          eventIndex += 1;
        }
      } else {
        lastCeilingShotPlayer = null;
      }

      for (const player of frame.players) {
        assignCeilingShotStats(
          player.ceiling_shot,
          players.get(remoteIdKey(player.player_id)),
        );
      }
    },
  };
}
